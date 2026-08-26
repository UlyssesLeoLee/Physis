# Physis

**GVPE — Graph-Governed Vector Physics Engine.** A self-developed, real-time Rust physics engine
core (not a wrapper around Rapier/Bullet/PhysX/Jolt/Box2D), governed offline by a Physics Knowledge
Graph and searched via a Physics Vector Space, bridged to the runtime by a Physics Compiler that
compiles high-level knowledge down to plain numeric `PhysicsProfile` data — never a live dependency
of the simulation hot path.

**The one invariant that must never break**: even with Graph, Vector, AI, and 3DGS entirely
disabled, the Rust Runtime alone remains a complete, independently runnable, commercial-real-time
-grade physics engine, callable from a game engine via C ABI. See `docs/00_foundation/00_vision.md` §0.5.

Status: requirements/architecture baseline (V0.1 Draft), no implementation code yet.

## Documents

| # | Document | Summary |
|---|---|---|
| 00 | [vision](docs/00_foundation/00_vision.md) | Role, six binding prohibitions, three-space model (Simulation/Vector/Graph), the long-term closed-loop pipeline, the one invariant that must survive every future change. |
| 01 | [requirements](docs/00_foundation/01_requirements.md) | GVPE-FR/NFR/GPH/VEC/PERF/LIC requirement IDs, MVP scope (self-developed rigid-body solver + minimal graph schema), acceptance criteria, risks, open questions. |
| 02 | [physics_ontology](docs/00_foundation/02_physics_ontology.md) | The Physics Knowledge Graph's top-level ontology — Matter/Phase/MechanicalBehavior/Property/State/Force/Interaction/Constraint/Energy/Wave/Field/Process/Law/Model/Approximation/BoundaryCondition/Observation/Experiment/Hypothesis/Simulation, causality & spatial/temporal relation vocabularies, plus a mandatory Ontology Review (11 confusion categories checked, one open finding). |
| 03 | [graph_schema](docs/01_architecture/03_graph_schema.md) | The three graphs that must never be merged — Physics Knowledge Graph (persistent, semantic) vs. Runtime Constraint Graph (per-frame, in-memory) vs. Execution Graph (task DAG) — plus the node/property/runtime-only decision rule and why the graph is never queried live from the hot path. |
| 04 | [architecture](docs/01_architecture/04_architecture.md) | `gvpe-*` module map, the one-directional dependency rule (Graph/Vector/AI → Compiler → Runtime, mechanically checked), the Compiler boundary, Law→Model→Solver traceability table, GPU and PhysicsLOD hooks. |
| 05 | [runtime_design](docs/01_architecture/05_runtime_design.md) | SoA/AoSoA data layout, Hot/Warm/Cold data classification, Fast vs. Deterministic mode, no-global-state runtime API shape. |
| 06 | [collision_design](docs/02_modules/06_collision_design.md) | Self-developed broad phase (SAP for MVP) and narrow phase (SAT for MVP; GJK/EPA reserved), contact manifold shape. |
| 07 | [solver_design](docs/02_modules/07_solver_design.md) | Sequential Impulse / PGS as Generation 1 (MVP), unified `ConstraintRow`, XPBD reserved as Generation 2, friction, sleeping. |
| 08 | [memory_design](docs/01_architecture/08_memory_design.md) | Zero/near-zero hot-path allocation policy, arena/pool/slab strategies, buffer shapes kept GPU-migration-friendly. |
| 09 | [parallel_design](docs/01_architecture/09_parallel_design.md) | Physics Islands as the parallel unit, the Execution-Graph job DAG, work-stealing scheduler direction, no global mutex on the hot path. |
| 10 | [ffi_design](docs/02_modules/10_ffi_design.md) | C-ABI-first design for Unity/Unreal/Godot/custom engines — opaque handles, POD-only, batched calls, panic-safe boundary. |
| 11 | [vector_design](docs/02_modules/11_vector_design.md) | Multi-vector Physics Signature (never a single embedding), type-distinct Observed/Simulated/Known signature instances, retrieval kept strictly out of the hot path. |
| 12 | [energy_wave_field_design](docs/02_modules/12_energy_wave_field_design.md) | The proof-of-shape bridging today's Energy/Wave/Field/Process *schema* to a future *runtime* extension, without redesigning the core solve loop. |
| 13 | [3dgs_future_design](docs/05_future/13_3dgs_future_design.md) | The full Observation→Retrieval→Hypothesis→Simulation→Comparison→Optimization closed loop, explicitly non-blocking for MVP. |
| 14 | [performance_budget](docs/03_cross_cutting/14_performance_budget.md) | 60–240Hz Simulation-Space target, per-stage budget breakdown (to be measured, not assumed), regression policy. |
| 15 | [testing_strategy](docs/03_cross_cutting/15_testing_strategy.md) | Determinism, dependency-isolation, Ontology-Review, Compiler round-trip, and solver-fixture test layers; closes the one open Ontology Review finding. |
| 16 | [dependency_license](docs/03_cross_cutting/16_dependency_license.md) | The hard-gate license review matrix any embedded graph/vector database must clear before selection, plus the hand-rolled-store fallback position. |
| 17 | [detailed_design](docs/04_detailed_design/17_detailed_design.md) | Concrete struct/trait/algorithm-level detail for every MVP-critical crate — `BodyHandle`/`PhysicsProfile`/`RuntimeDescriptor` layouts, arena/pool/slab allocator internals, SAP broad phase and SAT narrow phase pseudocode, `ConstraintRow` construction, the full Sequential Impulse solve loop, island Union-Find and sleep logic, the scheduler's job DAG execution, the no-global-state runtime lifecycle, the `catch_unwind`-wrapped C ABI implementation, an error model table, and a one-frame call sequence diagram. Non-MVP crates (`gvpe-graph`/`gvpe-compiler`/`gvpe-vector`) get interface-only detail, deliberately not deepened yet. |
| 18 | [joints_ccd_design](docs/02_modules/18_joints_ccd_design.md) | `JointRow` (Fixed/Distance/Hinge/Slider) decomposed into `ConstraintRow`s so the Sequential Impulse solver needs no changes; joint lifecycle via generational handles; conservative-advancement CCD design and its Execution Graph placement. |
| 19 | [softbody_xpbd_design](docs/02_modules/19_softbody_xpbd_design.md) | XPBD Generation 2 solver design proven compatible with `ConstraintRow.compliance` from day one; `ParticleStateSoA`; the `xpbd_step()` algorithm; Distance/Bending/Volume/Attachment constraint kinds mapped to Rope/Cloth/SoftBody/Granular; Fluid/FEM explicitly deferred with reasoning, not designed here. |
| 20 | [shape_advanced_design](docs/02_modules/20_shape_advanced_design.md) | The full shape set beyond MVP's Sphere/Box/Plane — Capsule/ConvexHull/TriangleMesh/Heightfield/Compound — plus GJK and EPA algorithms and the narrow-phase dispatch table routing SAT vs. GJK/EPA vs. mesh/heightfield/compound paths. |
| 21 | [graph_compiler_detailed_design](docs/04_detailed_design/21_graph_compiler_detailed_design.md) | `GraphStore` internals with an ontology-mirroring closed `NodeKind` enum, the depth-bounded traversal query, the `write_state_batch` guard that closes Ontology Review finding ONT-ISS-001 in code, and the `compile()` algorithm turning graph data into a `PhysicsProfile` with round-trip test guarantee. |
| 22 | [vector_detailed_design](docs/02_modules/22_vector_detailed_design.md) | Deterministic signature extraction (V1, no learned parameters), a flat-scan `VectorIndex` fallback with fused multi-vector similarity, and the type-level guarantee that `gvpe-vector` cannot be called mid-step. |
| 23 | [energy_wave_field_process_algorithms](docs/02_modules/23_energy_wave_field_process_algorithms.md) | Concrete, feature-gated (opt-in, zero-default-cost) numerics for energy-conservation checking, event-sourced wave-amplitude sampling, a `FieldSampler` trait generalizing the existing gravity hook, and a worked Process state machine (Melting) wired into the graph write-path guard. |
| 24 | [fluid_fem_reservation_design](docs/02_modules/24_fluid_fem_reservation_design.md) | Why Fluid and general FEM are not a `ConstraintRow`/XPBD extension (unlike Rope/Cloth/SoftBody), the reserved-but-unimplemented `ShapeDesc::FluidRegion` interface, the explicit `CompileError::UnsupportedModel` failure path, and the bounded list of design questions a future document would need to answer. |
| 25 | [gpu_backend_detailed_design](docs/04_detailed_design/25_gpu_backend_detailed_design.md) |
| 26 | [tech_selection](docs/03_cross_cutting/26_tech_selection.md) |
| 27 | [qa_register](docs/03_cross_cutting/27_qa_register.md) |
| 28 | [workflow](docs/06_process/28_workflow.md) |
| 29 | [ut_spec_template](docs/06_process/29_unit_test_spec_template.md) | 单元测试规格书模板：每个 crate 填写实例 `ut_spec_<crate>.md`；含公共 API 测试矩阵、错误路径、边界 / 数值稳定性、criterion bench、miri 验证、feature-gate 验证、覆盖率目标、CI 集成。 |
| 30 | [it_spec_template](docs/06_process/30_integration_test_spec_template.md) | 集成测试规格书模板：跨 crate 接口契约、依赖方向验证（`AC-02` 机械可验证）、ABI 兼容测试、跨 crate 性能 / 内存、C ABI 集成测试、失败注入。 |
| 31 | [pilot_integration_agreement](docs/06_process/31_pilot_integration_agreement.md) | 集成方 pilot 协议模板：范围、交付物、性能目标、issue SLA、IP / 保密、免责；用于 MVP 启动前与 1-2 个 Unity / Unreal / Godot 集成方签署。 |
| 32 | [st_spec_template](docs/06_process/32_system_test_spec_template.md) | 系统测试规格书模板：端到端功能 + 场景 + 性能 + 负荷 + 压力 + 安全 + 障碍；与 `14_performance_budget.md` 性能数字对齐。 |
| 33 | [typical_game_scenarios](docs/06_process/33_typical_game_scenarios.md) | 典型游戏场景库：L1 核心（5 场景）+ L2 推荐（8 场景）+ L3 扩展（5 场景）；含 Box stack / Sphere pile / Jointed pendulum / Sleeping validation / Tower of Pisa / Bullet CCD / Newton cradle / 1 万 body 压力等。 |
| 34 | [uat_plan_template](docs/06_process/34_uat_plan_template.md) | UAT 计划模板：与集成方协商的验收测试计划；含验收标准、12 项 UAT 场景、测试流程、问题管理、验收结果、签字。 |
| 35 | [uat_spec_template](docs/06_process/35_uat_spec_template.md) | UAT 规格书模板：UAT 计划下的具体测试规格；含 11 个 UAT-FN 用例、9 个 UAT-SC 场景、5 个 UAT-PT 性能、10 个 UAT-FT 障碍、5 个 UAT-FFI 测试。 |
| 36 | [project_plan](docs/06_process/36_project_plan.md) | GVPE 项目计划：MVP 6 大里程碑（M0-M6）、资源、风险摘要、沟通计划、范围纪律、Phase 边界。 |
| 37 | [pr_template](docs/06_process/37_pr_template.md) | PR 模板：标题规范、关联 ID、影响范围、禁用项声明、新增 `unsafe` 块、测试、文档同步、性能影响、签字；reviewer 必查项。 |
| 38 | [code_review_checklist](docs/06_process/38_code_review_checklist.md) | Code Review 详细清单：reviewer 准入、通用规范、选型 / 依赖、正确性、性能 / 内存、API 设计、FFI、测试、文档、许可证、Review 结论；12 大类检查项。 |
| 39 | [release_checklist](docs/06_process/39_release_checklist.md) | Release 检查清单：pre-release（设计 / 代码 / 性能 / 安全 / 依赖 / 测试）、release day（制品 / 渠道 / 通知）、post-release（稼动确认 / 失败处理）、Hypercare 4 周、Hotfix 专项。 |
| 40 | [postmortem_template](docs/06_process/40_postmortem_template.md) |
| 41 | [design_review_template](docs/06_process/41_design_review_template.md) | 设计评审模板：覆盖 RD Review (20) / BD Review (41) / DD Review (52) / ST Approval (89) 4 种评审；含评审人配置、检查表、评审意见、结论、签字。 |
| 42 | [change_request_form](docs/06_process/42_change_request_form.md) | 变更请求单 (CR)：覆盖变更要求 (118) / 变更管理 (120/136)；含动机、影响分析、crate/API/ABI 影响、风险评估、回滚方案、决策签字。 |
| 43 | [hypercare_plan_template](docs/06_process/43_hypercare_plan_template.md) | Hypercare 计划：覆盖初期流动对应 (108)；含响应 SLA、监控指标、紧急响应、阶段性产出、退出条件。 |
| 44 | [hotfix_runbook](docs/06_process/44_hotfix_runbook.md) | Hotfix 操作手册：覆盖紧急改修 (125)；含决策流程（24h 内完成）、分支策略、最小修复、强制双 review、release 流程、同步合并、后续 postmortem。 |
| 45 | [retrospective_template](docs/06_process/45_retrospective_template.md) | KPT 回顾模板：覆盖振り返り (148)；含 Keep/Problem/Try 三栏、行动项、关键数据回顾、团队健康度、归档。 |
| 46 | [closure_report_template](docs/06_process/46_closure_report_template.md) | 项目收尾报告：覆盖项目完了判定 (145) / 完了報告 (147) / 成果物引渡し (146)；含目标达成度、关键指标、重大变更、成果物清单。 |
| 47 | [kt_plan_template](docs/06_process/47_kt_plan_template.md) | 知识移交计划：覆盖ナレッジ移管 (149)；含必读文档、架构 tour、工具链上手、流程 walkthrough、首次 PR、已知坑、接收方验收。 |
| 48 | [acceptance_certificate_template](docs/06_process/48_acceptance_certificate_template.md) | 验收证书：覆盖検収 (95) / 受入判定 (94)；含验收范围、依据、结论、性能数字签字、已知 issue 列表、后续支持承诺。 |
| 49 | [meeting_notes_template](docs/06_process/49_meeting_notes_template.md) | 会议纪要：覆盖会議・報告 (140)；含定期会议的进度 / 风险 / 决策 / 阻塞四类同步、行动项、下次会议预告。 |
| 50 | [status_report_template](docs/06_process/50_status_report_template.md) | 状态报告：覆盖進捗管理 (133) / 报告 (140)；含 TL;DR、关键指标、本期完成 / 未完成、风险、决策、变更、集成方同步、下期重点。 |
| 51 | [wbs_template](docs/06_process/51_wbs_template.md) | WBS 模板：覆盖 WBS 管理 (132)；含 4 级分解（L1 Phase / L2 crate / L3 模块 / L4 任务）、进度跟踪、风险与依赖。 |
| 52 | [security_advisory_response](docs/06_process/52_security_advisory_response.md) | 安全公告响应：覆盖脆弱性対応 (123)；含订阅、严重度分级、响应 SLA、升级 / 加固方案、披露、CVE 申请、复盘、预防机制。 |
| 53 | [test_summary_report_template](docs/06_process/53_test_summary_report_template.md) | 测试总结报告：覆盖 UT 完 (65) / IT (75) / ST 完 (89) / 回归 (126)；含用例统计、性能数字、覆盖率、feature-gate 验证、工具链验证、结论。 |
| 54 | [qa_audit_report_template](docs/06_process/54_qa_audit_report_template.md) |
| 55 | [error_code_catalog](docs/04_detailed_design/55_error_code_catalog.md) | 错误码目录：所有 crate 的错误类型、错误码（u32）、触发条件、恢复建议、FFI 映射；含 crate 标识 + 错误序号分配规则。 |
| 56 | [logging_design](docs/04_detailed_design/56_logging_design.md) | 日志与可观测性设计：三层策略（错误 / 事件 / 性能）、panic 政策、崩溃诊断、集成方接口、OpenTelemetry 集成。 |
| 57 | [coding_standards](docs/04_detailed_design/57_coding_standards.md) | 编码标准：rustfmt / clippy / deny lint 配置、命名规范、模块组织、错误处理模式、unsafe 政策、并发与同步、性能准则、文档、提交规范。 |
| 58 | [data_layout_atlas](docs/04_detailed_design/58_data_layout_atlas.md) | 数据布局图谱：所有公共 struct 的内存布局、对齐、SoA/AoSoA 策略、GPU 上传性、bytemuck::Pod 派生、布局测试。 |
| 59 | [algorithm_pseudocode_atlas](docs/04_detailed_design/59_algorithm_pseudocode_atlas.md) | 算法伪代码图谱：broad phase SAP / narrow phase SAT / Sequential Impulse / island Union-Find / sleep 状态机 / work-stealing / 帧主循环；含复杂度与性能目标。 |
| 60 | [module_dependency_matrix](docs/04_detailed_design/60_module_dependency_matrix.md) | 模块依赖矩阵：17 个 crate 完整依赖图、各 crate 职责、允许 / 禁止依赖、feature-gated 依赖、CI 验证（`cargo tree` + `cargo deny`）。 |
| 61 | [performance_engineering](docs/04_detailed_design/61_performance_engineering.md) | 性能工程：MVP 性能目标、criterion bench、profiling 工具链、6 大类优化技术（算法 / 数据布局 / SIMD / 内存 / 并行 / 编译）、性能陷阱、回归检测。 |
| 62 | [unsafe_inventory](docs/04_detailed_design/62_unsafe_inventory.md) | Unsafe 块清单：所有 `unsafe` 块的权威登记（位置 / 不变式 / miri 验证 / review 要求），按 crate 分类，统计目标 < 30 个。 | 质量审计报告：覆盖 QA Review (128) / QA 评价 (129) / QA 監査 (130)；含文档 / 流程 / 代码 / 性能 / 安全 / 集成方 / 团队健康度审计、发现、行动项。 | 事故复盘模板：摘要、影响、时间线、5 Whys 根因、检测与响应、缓解、改进措施、教训、正面观察、公开性。 | 150 步工程工作流基线（参考日本 PM/SE 圈通用流程分 16 阶段）：将 27 份现有设计文档逐项映射到工作流步骤，状态分 ✅ 已覆盖 / 📋 流程定义中 / ⚠️ 待补 / ❌ N/A 四类；§11 为 44 个 📋 状态步骤给出具体流程定义；§12 覆盖率 92.2%（剔除 N/A），8 个 ⚠️ 待补项已列出。 | 实施前 QA 登记表：8 大类（设计/实现/技术/性能/FFI/测试/过程/业务）共 95 项顾虑与疑问；每项标注严重度（Blocker/High/Medium/Low）、影响范围、缓解/验证方式、责任人，并显式映射到上游需求 ID（FR/NFR/PERF/LIC/AC/PROHIBIT/NG）；为每个子系统的首个 commit 提供「该不该动这块」的检查表。 | Rust 主语言、Cargo 构建、目标平台、自研 math / scheduler、cbindgen FFI、thiserror 错误模型、显式拒绝清单；每个选型显式映射到  1_requirements 的 FR / NFR / PROHIBIT / AC 等需求 ID，并通过 cargo tree + 许可证审查与 GVPE-DOC-16 联动。 | Deepens `04_architecture.md` §4.6's GPU constraint into a concrete `GpuSolverBackend` trait boundary, a data-layout audit proving the CPU SoA types are already GPU-uploadable, Physics-Islands-as-dispatch-unit reuse, and the determinism-mode rule requiring `Deterministic` mode to stay CPU-only. |


## 流程与过程类

`docs/06_process/` 下 13 份文档：工程工作流基线（`28`）+ 8 份测试 / 计划 / 协议模板（`29`-`36`）+ 4 份流程模板（`37`-`40`）。
## Archived

`docs/archive/` holds the prior **PRE (Physical Retrieval Engine)** specification — a
retrieval-first design built around pluggable third-party-style solvers, superseded by GVPE's
self-developed-solver-first direction. See `docs/archive/README.md` for why the two directions
were incompatible enough to warrant a replacement rather than an extension.

## Core principles

- **Self-developed core, always.** No third-party physics engine is vendored or thinly wrapped as
  GVPE's own solver (`docs/00_foundation/00_vision.md` §0.2).
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
`docs/00_foundation/00_vision.md` §0.5。

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

文档编号 00–62（见上方“Documents”表格）覆盖需求、本体、图谱 Schema、架构、各子系统设计、以及
关节/CCD、XPBD 软体、高级碰撞形状、图谱编译器、向量检索、能量/波动/场/过程数值算法、流体/FEM
边界预留、GPU 计算后端等详细设计；`docs/archive/` 保留了被 GVPE 取代的前身项目 PRE
（Physical Retrieval Engine）的历史文档。
