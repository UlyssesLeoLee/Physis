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
| 17 | [detailed_design](docs/17_detailed_design.md) | Concrete struct/trait/algorithm-level detail for every MVP-critical crate — `BodyHandle`/`PhysicsProfile`/`RuntimeDescriptor` layouts, arena/pool/slab allocator internals, SAP broad phase and SAT narrow phase pseudocode, `ConstraintRow` construction, the full Sequential Impulse solve loop, island Union-Find and sleep logic, the scheduler's job DAG execution, the no-global-state runtime lifecycle, the `catch_unwind`-wrapped C ABI implementation, an error model table, and a one-frame call sequence diagram. Non-MVP crates (`gvpe-graph`/`gvpe-compiler`/`gvpe-vector`) get interface-only detail, deliberately not deepened yet. |
| 18 | [joints_ccd_design](docs/18_joints_ccd_design.md) | `JointRow` (Fixed/Distance/Hinge/Slider) decomposed into `ConstraintRow`s so the Sequential Impulse solver needs no changes; joint lifecycle via generational handles; conservative-advancement CCD design and its Execution Graph placement. |
| 19 | [softbody_xpbd_design](docs/19_softbody_xpbd_design.md) | XPBD Generation 2 solver design proven compatible with `ConstraintRow.compliance` from day one; `ParticleStateSoA`; the `xpbd_step()` algorithm; Distance/Bending/Volume/Attachment constraint kinds mapped to Rope/Cloth/SoftBody/Granular; Fluid/FEM explicitly deferred with reasoning, not designed here. |
| 20 | [shape_advanced_design](docs/20_shape_advanced_design.md) | The full shape set beyond MVP's Sphere/Box/Plane — Capsule/ConvexHull/TriangleMesh/Heightfield/Compound — plus GJK and EPA algorithms and the narrow-phase dispatch table routing SAT vs. GJK/EPA vs. mesh/heightfield/compound paths. |
| 21 | [graph_compiler_detailed_design](docs/21_graph_compiler_detailed_design.md) | `GraphStore` internals with an ontology-mirroring closed `NodeKind` enum, the depth-bounded traversal query, the `write_state_batch` guard that closes Ontology Review finding ONT-ISS-001 in code, and the `compile()` algorithm turning graph data into a `PhysicsProfile` with round-trip test guarantee. |
| 22 | [vector_detailed_design](docs/22_vector_detailed_design.md) | Deterministic signature extraction (V1, no learned parameters), a flat-scan `VectorIndex` fallback with fused multi-vector similarity, and the type-level guarantee that `gvpe-vector` cannot be called mid-step. |
| 23 | [energy_wave_field_process_algorithms](docs/23_energy_wave_field_process_algorithms.md) | Concrete, feature-gated (opt-in, zero-default-cost) numerics for energy-conservation checking, event-sourced wave-amplitude sampling, a `FieldSampler` trait generalizing the existing gravity hook, and a worked Process state machine (Melting) wired into the graph write-path guard. |
| 24 | [fluid_fem_reservation_design](docs/24_fluid_fem_reservation_design.md) | Why Fluid and general FEM are not a `ConstraintRow`/XPBD extension (unlike Rope/Cloth/SoftBody), the reserved-but-unimplemented `ShapeDesc::FluidRegion` interface, the explicit `CompileError::UnsupportedModel` failure path, and the bounded list of design questions a future document would need to answer. |
| 25 | [gpu_backend_detailed_design](docs/25_gpu_backend_detailed_design.md) | Deepens `04_architecture.md` §4.6's GPU constraint into a concrete `GpuSolverBackend` trait boundary, a data-layout audit proving the CPU SoA types are already GPU-uploadable, Physics-Islands-as-dispatch-unit reuse, and the determinism-mode rule requiring `Deterministic` mode to stay CPU-only. |

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

## 中文简介

**GVPE —— 图治理向量物理引擎（Graph-Governed Vector Physics Engine）。** 一个自主研发、实时运行的
Rust 物理引擎内核（并非对 Rapier/Bullet/PhysX/Jolt/Box2D 等第三方引擎的封装），由离线的物理知识图谱
（Physics Knowledge Graph）治理、通过物理向量空间（Physics Vector Space）检索，并由物理编译器
（Physics Compiler）将高层知识编译为纯数值的 `PhysicsProfile` 数据桥接到运行时——图谱与向量空间
永远不是仿真热路径的运行时依赖。

**不可打破的唯一不变量**：即使完全关闭图谱、向量空间、AI 推理与 3DGS 闭环，仅剩的 Rust Runtime
本身仍必须是一个完整的、可独立运行的、商用实时级物理引擎，并可通过 C ABI 被游戏引擎调用。详见
`docs/00_vision.md` §0.5。

当前状态：需求/架构基线阶段（V0.1 草案），尚无实现代码。

**三大空间，单向依赖**：仿真空间（Simulation Space，60–240Hz 计算）、向量空间（Vector Space，
事件触发的检索）、图谱空间（Graph Space，离线的知识组织）——依赖方向永远是 图谱/向量/AI → 编译器 →
运行时，绝不反向。

**核心原则**：
- **内核永远自主研发**，不引入、不轻度封装任何第三方物理引擎作为自己的求解器。
- **图谱是知识面，不是数据面**——运行时热路径永不触碰逐帧状态或实时查询。
- **本体先行、只增不破**——Energy/Wave/Field/Process/Law 从第一天起就有完整 schema，即使 MVP
  阶段只填充 Entity/Material/Phase/Property/PhysicalModel/Solver/PhysicsProfile/Simulation/
  Observation。
- **检索只提议，物理来验证**（Retrieval proposes, physics verifies）——向量空间给出的候选结果
  永远不是最终答案，必须经过仿真验证闭环。

文档编号 00–25（见上方“Documents”表格）覆盖需求、本体、图谱 Schema、架构、各子系统设计、以及
关节/CCD、XPBD 软体、高级碰撞形状、图谱编译器、向量检索、能量/波动/场/过程数值算法、流体/FEM
边界预留、GPU 计算后端等详细设计；`docs/archive/` 保留了被 GVPE 取代的前身项目 PRE
（Physical Retrieval Engine）的历史文档。
