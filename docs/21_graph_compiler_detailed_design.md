# GVPE — Graph Store & Compiler Detailed Design

Input baseline: `03_graph_schema.md`, `04_architecture.md` §4.4, `17_detailed_design.md` §11
(interface-only). This document is where §11's placeholder traits get real internals — required for
"all detailed design complete", even though `gvpe-graph`/`gvpe-compiler` remain outside the MVP
runtime-acceptance gate (`01_requirements.md` §11).

## 21.1 Storage engine (fallback path, per `16_dependency_license.md` §16.4)

```rust
struct GraphStore {
    nodes: HashMap<NodeId, Node>,
    edges: HashMap<NodeId, Vec<Edge>>,          // adjacency list, keyed by source node
    property_index: HashMap<(NodeId, PropertyKind), NodeId>,   // fast Entity -> Property lookup
}

struct Node { id: NodeId, kind: NodeKind, attrs: HashMap<String, AttrValue> }
struct Edge { to: NodeId, relation: RelationKind, condition: Option<Condition> }

enum NodeKind {   // mirrors 02_physics_ontology.md §1's top-level concepts exactly, 1:1
    Entity, Matter, Material, Phase, Property, State, Force, Interaction, Constraint, Field,
    Energy, Wave, Process, PhysicalLaw, ConstitutiveModel, ApproximationModel, SolverModel,
    BoundaryCondition, Observation, Experiment, Hypothesis, Simulation, SimulationState,
    PhysicsProfileNode, VectorDescriptor, Result,
}
```
`NodeKind` is a closed enum enumerating exactly `02_physics_ontology.md` §1's list — adding a node
kind not in that ontology is a compile error, which is the storage-layer enforcement of the
ontology being the single source of truth for what a "Node" can be (operationalizes
`03_graph_schema.md` §2's decision rule at the type level, not just as a design guideline).

## 21.2 Bounded traversal query (implements `03_graph_schema.md` §6)

```rust
fn bounded_traverse(store: &GraphStore, start: NodeId, max_depth: u32,
                      relation_filter: Option<&dyn Fn(&Edge) -> bool>) -> Vec<NodeId> {
    let mut visited = HashSet::from([start]);
    let mut frontier = vec![start];
    for _ in 0..max_depth {          // hard cap, no unbounded-depth code path exists in this function
        let mut next = Vec::new();
        for node in &frontier {
            for edge in store.edges.get(node).into_iter().flatten() {
                if let Some(f) = relation_filter { if !f(edge) { continue; } }
                if edge.condition.as_ref().is_some_and(|c| !c.currently_holds(store)) { continue; }
                if visited.insert(edge.to) { next.push(edge.to); }
            }
        }
        if next.is_empty() { break; }
        frontier = next;
    }
    visited.into_iter().collect()
}
```
This is the graph-store-level twin of `08_PRE_Detailed_Design.md`'s archived `traverse()`
(`docs/archive/`) — the depth-bounding discipline that PRE work established for its own
graph-construction feature is restated here verbatim in shape, because it's the correct pattern
independent of which project's ontology sits on top of it. Conditional edges (§21.1's
`Edge.condition`) implement `02_physics_ontology.md` §22's "conditional relation support"
requirement (e.g. `HigherTemperature DECREASES Viscosity` only holds while the condition is true).

## 21.3 Write-path guard (closes `02_physics_ontology.md` ONT-ISS-001)

```rust
fn write_state_batch(store: &mut GraphStore, states: &[StateWrite]) -> Result<(), GraphError> {
    if states.len() > BULK_STATE_WRITE_THRESHOLD {
        return Err(GraphError::BulkStateWriteRejected {
            count: states.len(), threshold: BULK_STATE_WRITE_THRESHOLD,
            hint: "per-frame State belongs in Runtime/Snapshot storage, not the Graph — see 03_graph_schema.md §4",
        });
    }
    for s in states { store.upsert_state_node(s)?; }
    Ok(())
}
```
This is the concrete enforcement `02_physics_ontology.md` §Review's ONT-ISS-001 said was missing —
a bulk per-frame `State` write is now a rejected `GraphError`, not merely a documented rule.
`15_testing_strategy.md` §15.4(b)'s test asserts this rejection; that test can now be written
against real code, closing ONT-ISS-001 per its own stated closing condition.

## 21.4 Physics Compiler algorithm

```rust
fn compile(store: &GraphStore, entity: NodeId) -> Result<PhysicsProfile, CompileError> {
    let material = follow_edge(store, entity, RelationKind::HasMaterial)
        .ok_or(CompileError::MissingRequiredEdge("HAS_MATERIAL"))?;
    let model = follow_edge(store, entity, RelationKind::ModeledBy)
        .ok_or(CompileError::MissingRequiredEdge("MODELED_BY"))?;

    let mass       = read_property_f32(store, material, PropertyKind::Mass)?;
    let density    = read_property_f32(store, material, PropertyKind::Density)?;
    let friction   = read_property_f32(store, material, PropertyKind::Friction).unwrap_or(DEFAULT_FRICTION);
    let restitution= read_property_f32(store, material, PropertyKind::Restitution).unwrap_or(DEFAULT_RESTITUTION);
    // ... remaining PhysicsProfile fields follow the same read_property_f32-with-fallback pattern

    let solver_type = match node_kind_attr(store, model, "kind") {
        "RigidBodyModel" => SolverTypeId::SequentialImpulse,
        "PBDModel" | "XPBDModel" => SolverTypeId::Xpbd,
        other => return Err(CompileError::UnsupportedModel(other.to_string())),
    };

    Ok(PhysicsProfile { mass, density, friction, restitution, solver_type, /* ... */ ..Default::default() })
}
```
Every field read goes through `read_property_f32`, which itself calls `bounded_traverse` (§21.2)
with `max_depth = 1` (direct property lookup, no multi-hop needed for Compiler purposes) — the
Compiler never performs unbounded graph search, consistent with `03_graph_schema.md` §6's claim
that no query pattern GVPE needs requires unbounded search at any time, including compile time.

`CompileError::UnsupportedModel` is how §4.5's Law→Model→Solver traceability table
(`04_architecture.md`) gets enforced at runtime: a graph node claiming a `PhysicalModel` kind with
no corresponding Solver entry in that table fails compilation explicitly, rather than silently
producing a nonsensical `PhysicsProfile`.

## 21.5 Compiler round-trip guarantee (implements `15_testing_strategy.md` §15.5)

```rust
#[test]
fn compiled_profile_matches_hand_constructed() {
    let store = populate_test_graph(KNOWN_MATERIAL_FIXTURE);
    let compiled = compile(&store, ENTITY_ID).unwrap();
    let manual = PhysicsProfile { mass: 1.0, density: 1.0, friction: 0.5, restitution: 0.3, /* ... */ ..Default::default() };
    assert_eq!(compiled, manual);
}
```
`PhysicsProfile` (`17_detailed_design.md` §1.2) derives `PartialEq` specifically to make this
assertion possible — a detail that section's original definition didn't call out but is required
for AC-03 to be checkable in code, not just assertable in prose.

## 21.6 Explicit non-scope (deferred deliberately, not an oversight)

Backend selection (embedded hand-rolled store from §21.1 vs. an external graph DB) is
`16_dependency_license.md`'s decision, not this document's — §21.1's `HashMap`-based store is the
concrete fallback implementation that decision guarantees remains viable regardless of what the
license review concludes.

Requirements satisfied: GVPE-GPH-001/002/003, GVPE-FR-003, AC-03, closes ONT-ISS-001.
