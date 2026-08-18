# Physis

**GVPE — Graph-Governed Vector Physics Engine.** A self-developed, real-time Rust physics engine
core (not a wrapper around Rapier/Bullet/PhysX/Jolt/Box2D), governed offline by a Physics Knowledge
Graph and searched via a Physics Vector Space, bridged to the runtime by a Physics Compiler that
compiles high-level knowledge down to plain numeric `PhysicsProfile` data — never a live dependency
of the simulation hot path.

**The one invariant that must never break**: even with Graph, Vector, AI, and 3DGS entirely
disabled, the Rust Runtime alone remains a complete, independently runnable, commercial-real-time
-grade physics engine, callable from a game engine via C ABI. See `docs/00_vision.md` §0.5.

Status: requirements/architecture baseline (V0.1 Draft), no implementation code yet.

## Documents

| # | Document | Summary |
|---|---|---|
| 00 | [vision](docs/00_vision.md) | Role, six binding prohibitions, three-space model (Simulation/Vector/Graph), the long-term closed-loop pipeline, the one invariant that must survive every future change. |
| 01 | [requirements](docs/01_requirements.md) | GVPE-FR/NFR/GPH/VEC/PERF/LIC requirement IDs, MVP scope (self-developed rigid-body solver + minimal graph schema), acceptance criteria, risks, open questions. |
| 02 | [physics_ontology](docs/02_physics_ontology.md) | The Physics Knowledge Graph's top-level ontology — Matter/Phase/MechanicalBehavior/Property/State/Force/Interaction/Constraint/Energy/Wave/Field/Process/Law/Model/Approximation/BoundaryCondition/Observation/Experiment/Hypothesis/Simulation, causality & spatial/temporal relation vocabularies, plus a mandatory Ontology Review (11 confusion categories checked, one open finding). |
| 03 | [graph_schema](docs/03_graph_schema.md) | The three graphs that must never be merged — Physics Knowledge Graph (persistent, semantic) vs. Runtime Constraint Graph (per-frame, in-memory) vs. Execution Graph (task DAG) — plus the node/property/runtime-only decision rule and why the graph is never queried live from the hot path. |
| 04 | [architecture](docs/04_architecture.md) | `gvpe-*` module map, the one-directional dependency rule (Graph/Vector/AI → Compiler → Runtime, mechanically checked), the Compiler boundary, Law→Model→Solver traceability table, GPU and PhysicsLOD hooks. |
| 05 | [runtime_design](docs/05_runtime_design.md) | SoA/AoSoA data layout, Hot/Warm/Cold data classification, Fast vs. Deterministic mode, no-global-state runtime API shape. |
| 06 | [collision_design](docs/06_collision_design.md) | Self-developed broad phase (SAP for MVP) and narrow phase (SAT for MVP; GJK/EPA reserved), contact manifold shape. |
| 07 | [solver_design](docs/07_solver_design.md) | Sequential Impulse / PGS as Generation 1 (MVP), unified `ConstraintRow`, XPBD reserved as Generation 2, friction, sleeping. |
| 08 | [memory_design](docs/08_memory_design.md) | Zero/near-zero hot-path allocation policy, arena/pool/slab strategies, buffer shapes kept GPU-migration-friendly. |
| 09 | [parallel_design](docs/09_parallel_design.md) | Physics Islands as the parallel unit, the Execution-Graph job DAG, work-stealing scheduler direction, no global mutex on the hot path. |
| 10 | [ffi_design](docs/10_ffi_design.md) | C-ABI-first design for Unity/Unreal/Godot/custom engines — opaque handles, POD-only, batched calls, panic-safe boundary. |
| 11 | [vector_design](docs/11_vector_design.md) | Multi-vector Physics Signature (never a single embedding), type-distinct Observed/Simulated/Known signature instances, retrieval kept strictly out of the hot path. |
| 12 | [energy_wave_field_design](docs/12_energy_wave_field_design.md) | The proof-of-shape bridging today's Energy/Wave/Field/Process *schema* to a future *runtime* extension, without redesigning the core solve loop. |
| 13 | [3dgs_future_design](docs/13_3dgs_future_design.md) | The full Observation→Retrieval→Hypothesis→Simulation→Comparison→Optimization closed loop, explicitly non-blocking for MVP. |
| 14 | [performance_budget](docs/14_performance_budget.md) | 60–240Hz Simulation-Space target, per-stage budget breakdown (to be measured, not assumed), regression policy. |
| 15 | [testing_strategy](docs/15_testing_strategy.md) | Determinism, dependency-isolation, Ontology-Review, Compiler round-trip, and solver-fixture test layers; closes the one open Ontology Review finding. |
| 16 | [dependency_license](docs/16_dependency_license.md) | The hard-gate license review matrix any embedded graph/vector database must clear before selection, plus the hand-rolled-store fallback position. |

## Archived

`docs/archive/` holds the prior **PRE (Physical Retrieval Engine)** specification — a
retrieval-first design built around pluggable third-party-style solvers, superseded by GVPE's
self-developed-solver-first direction. See `docs/archive/README.md` for why the two directions
were incompatible enough to warrant a replacement rather than an extension.

## Core principles

- **Self-developed core, always.** No third-party physics engine is vendored or thinly wrapped as
  GVPE's own solver (`docs/00_vision.md` §0.2).
- **Three spaces, one direction.** Simulation Space computes; Vector Space searches; Graph Space
  understands and organizes. Dependency flows Graph/Vector → Compiler → Runtime, never backward.
- **Graph is a knowledge plane, not a data plane.** No per-frame state, no live query, ever touches
  the simulation hot path.
- **The ontology is built to extend without breaking.** Energy, Wave, Field, Process, and Law are
  schema-complete from day one even though MVP only populates Entity/Material/Phase/Property/
  PhysicalModel/Solver/PhysicsProfile/Simulation/Observation.
