# GVPE — Requirements（要件定義）

Input baseline: `00_vision.md`. ID prefix: `GVPE-*`, grouped by subsystem (`FR` functional,
`ONT` ontology, `GPH` graph, `VEC` vector, `RT` runtime, `PERF` performance, `LIC` licensing,
`NFR` cross-cutting non-functional).

## 1. Background

Reverse physics inference (observation → parameters) has no systematic, cumulative solution;
existing engines are either closed-source performance black boxes or research code with no
production-grade real-time core. GVPE's premise: build the self-developed real-time core first,
make it independently valuable, and layer a knowledge graph + vector space on top as an *optional*
reasoning aid that compiles down to plain runtime parameters — never a runtime dependency.

## 2. Goals

- G1: A self-developed Rust rigid-body solver (broad phase, narrow phase, contact generation,
  sequential-impulse solve, islands, sleeping) reaching commercial-real-time performance targets
  (§14) with zero third-party physics-engine dependency.
- G2: A Physics Knowledge Graph whose top-level ontology (§02) is complete enough that Energy,
  Wave, Field, Process, and Law extensions never require a breaking schema migration.
- G3: A Physics Compiler that is the *only* path from Graph/Vector to Runtime — the runtime never
  queries the graph or the vector index directly.
- G4: A multi-vector Physics Signature space usable for retrieval, kept fully outside the per-step
  hot path.
- G5: A C ABI surface usable from Unity/Unreal/Godot/custom engines with batched data exchange.
- G6: An architecture that keeps the Observation→Simulation→Comparison closed loop (3DGS-oriented)
  buildable later without redesigning §02–§04.

## 3. Non-goals (V0.1)

- NG1: No fluid/FEM/multi-phase solver implementation (interfaces reserved, not implemented).
- NG2: No 3DGS reconstruction/inference pipeline (§13 is interface-only).
- NG3: No GPU compute backend in the MVP (architecture must not preclude it — see `04_architecture.md` §4.6).
- NG4: No production graph/vector database selection locked before the license review (§16) completes.
- NG5: No LLM-driven parameter inference anywhere in the solve path (GVPE-PROHIBIT-05).

## 4. System boundary

GVPE is an embeddable library, not an application. It owns no window, no main loop by default, and
no persistent process. A host (game engine, tool, test harness) drives it via the Rust API or the
C ABI (`10_ffi_design.md`).

## 5. Functional Requirements

- **GVPE-FR-001**: The Simulation Space must be fully operable with Graph Space and Vector Space
  compiled out (feature-gated), producing bit-for-bit-identical simulation results to the
  Graph/Vector-enabled build for the same `PhysicsProfile` input.
- **GVPE-FR-002**: The solver must support, at minimum: static/dynamic rigid bodies (sphere, box,
  plane primitives for MVP), broad-phase pruning, narrow-phase contact generation, sequential-impulse
  contact + friction resolution, restitution, sleeping, and island-based grouping.
- **GVPE-FR-003**: `PhysicsProfile` (§04) is the only data structure Graph/Vector/Compiler may hand
  to the Runtime. It must be a flat, POD-friendly structure (mass, density, inertia, friction,
  restitution, damping, stiffness, compliance, viscosity, solver_type, solver_iterations,
  collision_profile, approximation_level) — no graph node references, no vector handles.
- **GVPE-FR-004**: The Physics Knowledge Graph must implement the full top-level ontology (§02 §4)
  as schema even where MVP only populates a subset of instances (§02 §MVP scope).
- **GVPE-FR-005**: Three graph types (Physics Knowledge Graph / Runtime Constraint Graph / Execution
  Graph, `03_graph_schema.md` §1) must remain implementationally distinct — no shared storage, no
  shared query surface, no accidental promotion of one into another's role.
- **GVPE-FR-006**: The Physics Signature must be multi-vector (material/motion/deformation/
  interaction/contact/energy/wave/field/environment/solver sub-signatures per `11_vector_design.md`),
  never a single undifferentiated embedding.
- **GVPE-FR-007**: A `PhysicsLOD` mechanism (§02 §19, `04_architecture.md` §4.7) must be reserved in
  the runtime descriptor even where MVP only implements LOD0 (full simulation).

## 6. Non-Functional Requirements

- **GVPE-NFR-001** (Determinism): Fast Mode and Deterministic Mode must be architecturally
  distinguished from the first version (`05_runtime_design.md` §5), even if Deterministic Mode is
  not fully implemented in MVP.
- **GVPE-NFR-002** (Memory): Hot path must target zero/near-zero per-step heap allocation
  (`08_memory_design.md`).
- **GVPE-NFR-003** (Portability): Core crates (`gvpe-core`, `gvpe-collision`, `gvpe-dynamics`,
  `gvpe-constraint`, `gvpe-solver`, `gvpe-island`, `gvpe-scheduler`, `gvpe-runtime`) must have zero
  dependency on `gvpe-graph`, `gvpe-vector`, `gvpe-compiler`, `gvpe-inference`, `gvpe-3dgs` — the
  dependency direction is Graph/Vector/AI → Compiler → Runtime, never reversed
  (`04_architecture.md` §4.3).
- **GVPE-NFR-004** (License hygiene): Any embedded graph or vector database must clear the full
  license review matrix in `16_dependency_license.md` before being locked as a dependency.

## 7. GPH — Graph-space requirements

- **GVPE-GPH-001**: Graph nodes must only be created for entities meeting the node/property
  decision rule (`03_graph_schema.md` §2) — high-semantic, high-connectivity, provenance/confidence
  -bearing data. Bulk numeric per-frame state must never be persisted as graph nodes
  (GVPE-PROHIBIT-03's practical enforcement).
- **GVPE-GPH-002**: Every causal/energy-flow relation type in §02 §25/§13 must be representable in
  the graph schema, with conditional (non-unconditional) relation support.
- **GVPE-GPH-003**: The graph must never be queried from the per-frame hot path (Cypher or any
  query language call inside `gvpe-solver`/`gvpe-dynamics`/`gvpe-scheduler` is a defect, not a
  style issue).

## 8. VEC — Vector-space requirements

- **GVPE-VEC-001**: Vector retrieval must run at 1–30Hz or purely event-triggered, never per physics
  step.
- **GVPE-VEC-002**: `ObservedPhysicsSignature` / `SimulatedPhysicsSignature` / `KnownPhysicsSignature`
  must be distinguishable at the type level, not just by a tag field, to prevent accidental
  cross-comparison of incompatible signature sources.

## 9. PERF — Performance requirements (targets, refined in `14_performance_budget.md`)

- **GVPE-PERF-001**: MVP rigid-body scene (order: hundreds of dynamic bodies, primitive shapes)
  must sustain 60Hz on a single mid-range CPU core budget as the baseline target; multi-threaded
  scaling is a stretch goal, not an MVP gate.
- **GVPE-PERF-002**: Any GC-like pause, unbounded allocation, or lock contention on the hot path
  must be treated as a performance regression bug, not an acceptable trade-off.

## 10. LIC — Licensing requirements

- **GVPE-LIC-001**: No graph or vector database dependency is selected until it passes every check
  in `16_dependency_license.md` §2 (license, commercial use, OEM, redistribution, modification,
  static/dynamic linking, SaaS, embedded use).

## 11. MVP Scope

Simulation Space: 3D rigid body (sphere/box/plane), broad phase, narrow phase, contact manifold,
sequential impulse, friction, restitution, sleeping, physics islands, basic multithreading, C ABI.

Graph Space (schema complete, instance population minimal): `Entity`, `Material`, `Phase`,
`Property`, `PhysicalModel`, `Solver`, `PhysicsProfile`, `Simulation`, `Observation`.

Explicitly NOT MVP, but the schema must not preclude them without redesign: `Energy`, `Wave`,
`Field`, `Process`, `PhysicalLaw` (populate later, described fully now in §02).

## 12. Acceptance Criteria

- **AC-01**: A scene of N rigid bodies simulates deterministically-repeatable (same seed, same
  build, same machine) results across two runs, and does so with the Graph/Vector features fully
  compiled out.
- **AC-02**: `cargo tree -p gvpe-core -p gvpe-collision -p gvpe-dynamics -p gvpe-constraint
  -p gvpe-solver -p gvpe-island -p gvpe-scheduler -p gvpe-runtime` contains no `gvpe-graph`,
  `gvpe-vector`, `gvpe-compiler`, `gvpe-inference`, `gvpe-3dgs` entries (GVPE-NFR-003 enforced
  mechanically, following the same "dynamically enumerate, don't hardcode" lesson the archived PRE
  spec's audit already learned once — see `docs/archive/`).
- **AC-03**: A `PhysicsProfile` compiled from the Graph via the Compiler produces byte-identical
  runtime behavior to the same `PhysicsProfile` constructed by hand without touching the Graph at
  all.
- **AC-04**: The Ontology Review (§02 §Review) registers zero unresolved `ONT-ISS-*` findings of
  severity High before this baseline is considered accepted.

## 13. Risks (see also `15_testing_strategy.md` for mitigation-by-test)

- Self-developing a full rigid-body solver from scratch is a multi-month effort even at MVP scope;
  scope discipline (§11) is the primary control.
- Ontology over-design: §02 deliberately front-loads Energy/Wave/Field/Process/Law schema before
  any solver populates them, which is a real cost paid now to avoid a breaking migration later —
  tracked explicitly as a risk to revisit if MVP timeline slips (see `15_testing_strategy.md`
  Ontology Review discipline).

## 14. Open Questions

- OQ-01: Deterministic Mode's exact floating-point/reduction-order guarantees are not yet
  specified numerically — `05_runtime_design.md` §5 states the requirement, not the algorithm.
- OQ-02: Which graph database (if any commercial off-the-shelf candidate survives §16) vs. a
  hand-rolled embedded graph store is not decided; both paths are kept open until the license
  review and a storage-shape spike complete.
