# GVPE — Graph Schema（グラフスキーマ）

Input baseline: `02_physics_ontology.md`, GVPE-GPH-*. This document is where the ontology becomes
storable and queryable, and where the three-graph distinction (`00_vision.md` prohibits blurring
it) gets concrete rules instead of just a diagram.

## 1. Three graphs — never merge, never cross-query

### A. Physics Knowledge Graph — persistent, semantic

```
Entity
 ├─ HAS_MATERIAL → Material     ├─ INTERACTS_VIA → Interaction
 ├─ HAS_PHASE → Phase           ├─ EXISTS_IN → Field
 ├─ HAS_STATE → State           ├─ CARRIES → Energy
 ├─ HAS_PROPERTY → Property     ├─ GENERATES → Wave
 ├─ PARTICIPATES_IN → Process   └─ MODELED_BY → PhysicalModel
```
Backing store: a persistent graph database (or hand-rolled embedded store — decision pending
`16_dependency_license.md`). Query surface: whatever that store offers (Cypher-equivalent or
custom), used **only offline / at Compiler time**, never from the hot path (GVPE-GPH-003).

### B. Runtime Constraint Graph — pure in-memory, per-frame

```
Body A ─ Contact ─ Body B
   │                 │
 Joint             Contact
   │                 │
Body C ─────────── Body D
```
Purpose: connected components → physics islands → constraint partition → parallel solve
(`09_parallel_design.md` §2). Lives entirely in `gvpe-island`/`gvpe-constraint`. Rebuilt or
incrementally updated every frame; never persisted; never shares a node type with the Knowledge
Graph (a `Body` handle here is a runtime index, not a `Entity` node reference).

### C. Execution Graph — the task DAG, not physics semantics at all

```
Apply Forces → Broad Phase → Narrow Phase → Contact Generation → Island Build
  → Constraint Solve → Integrate → CCD → Output
```
This is `gvpe-scheduler`'s job graph (`09_parallel_design.md` §3). It has no ontology content
whatsoever — conflating it with A or B is the specific mistake §4/§5 below guard against.

## 2. Node vs. Property vs. Runtime-only data — the decision rule

| Data shape | Destination | Rule |
|---|---|---|
| High-semantic, high-connectivity, has provenance/confidence/history | **Node** (Physics Knowledge Graph) | e.g. a measured `YoungModulus` from an `Experiment` |
| Pure numeric, low-semantic, high-frequency-changing | **Property/State storage**, not a permanent node | e.g. `position.x = 1.284` |
| Per-frame simulation output at scale | **Runtime State** (never Graph) | see §4 |

This rule is the enforcement mechanism for `02_physics_ontology.md` §26 Ontology Review rule 11
(Graph Node vs Runtime State) — **ONT-ISS-001** tracks closing the gap between "rule stated" and
"rule enforced in code" for this exact table.

## 3. Graph → Runtime must be compiled, never queried live

```
FORBIDDEN:  Runtime → Cypher Query (or any live graph query at simulation time)
REQUIRED:   Graph → Physics Compiler → Compact Runtime Descriptor
```
The Runtime only ever consumes: `Handle, ID, Index, Numeric Data, BitFlags, Aligned Buffer`. No
graph node type, no query result set, ever crosses into `gvpe-runtime`/`gvpe-solver` directly. See
`04_architecture.md` §4.4 for the Compiler's exact interface.

## 4. Graph is not a time-series simulation database

Millions of per-frame `State` writes must never land in the Graph DB. Distinguish:

`Runtime State` (in-memory only) → `Simulation Snapshot` (periodic, binary) → `Keyframe State`
(sparse, semantically meaningful) → `Semantic State` (graph-eligible, e.g. "body came to rest") →
`Observation State` (graph-eligible, tied to an `Observation` node).

High-frequency data goes to: Binary Snapshot / Time-series Storage / Simulation Cache — never the
Graph DB. The Graph stores *indices into* or *summaries of* this data, not the data itself.

## 5. Example node schemas (illustrative, not exhaustive — full schema is code, this is intent)

```
(:Entity {id, name})
(:Material {id, name})
(:Property {id, kind: "YoungModulus", value, unit, range, confidence, source,
            measurement_method, estimation_method, timestamp, validity, uncertainty})
(:PhysicalModel {id, kind: "RigidBodyModel"})
(:PhysicsProfile {id, mass, density, inertia, friction, restitution, damping, stiffness,
                  compliance, viscosity, solver_type, solver_iterations,
                  collision_profile, approximation_level})
(:Observation {id, kind, source, timestamp, coordinate_system, confidence, noise,
               resolution, sampling_rate})
```
Edges carry the relation vocabulary from `02_physics_ontology.md` §22, with optional conditional
qualifiers (e.g. `{relation: DECREASES, condition: "temperature > threshold"}`).

## 6. Query patterns the schema must support (drives §16's DB evaluation criteria)

- Direct lookup: `Entity → Property` by kind (used by Compiler, offline).
- Bounded traversal: causal chain reconstruction (§02 §25) up to a fixed hop count — must have an
  explicit depth cap, mirroring the depth-limiting discipline the archived PRE ontology work
  (`docs/archive/`) already established for its own graph-construction feature; that lesson carries
  forward unchanged even though the storage technology decision here is still open.
- Provenance walk: `Result ← VALIDATED_BY ← Simulation ← TESTED_BY ← Hypothesis ← SUPPORTS ←
  Observation`.

No query pattern in this list requires unbounded multi-hop search at simulation time — all of them
are Compiler-time or tooling-time only.

## 7. Schema evolution discipline

Any schema change must state, in the PR/commit, which of `02_physics_ontology.md`'s eleven
Ontology Review categories it touches. A change that would require re-populating already-committed
MVP instance data (§26 there) is a breaking migration and must be flagged as such, not silently
merged.
