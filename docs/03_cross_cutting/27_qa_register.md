# GVPE — 实施前 QA 登记表（基本設計書）

> 目的：在**任何实现代码落地之前**，把对需求规约 / 设计文档 / 技术选型基线（`GVPE-DOC-00` ~ `GVPE-DOC-26`）的所有顾虑、疑问、隐含假设、未验证项集中登记，作为实现期"该不该动这块"的检查表。
> 关联：每条 QA 项至少映射到一个上游文档节号 / 需求 ID / 禁令。审阅流程：每条 QA 项必须经责任人（默认 编写者）确认状态后，才可"转 Closed"或"改 Deferred"。
> 关系：与 `GVPE-DOC-26` §18.15 "选型变更流程" 配合——技术层面的变更优先走 26 号文档；本文件聚焦**实施前**的疑虑与待验证项，不重复 26 号的选型决策本身。

## 0. 文档元数据

| 字段 | 取值 |
|---|---|
| 文档编号 | GVPE-DOC-27 |
| 文档类型 | 基本設計書 |
| 版本 | v0.1 |
| 状态 | Draft |
| 适用阶段 | MVP / 实施启动前冻结 |
| 关联系统 | GVPE / 实施前风险登记 |
| 上游文档（输入基线） | `GVPE-DOC-00` ~ `GVPE-DOC-26` 全部 |
| 下游文档（被消费于） | `GVPE-DOC-17`（详细设计阶段消费本登记） |
| 编写者 | — |
| 审批者 | — |

## 1. 修订历史

| 版本 | 日期 | 修改者 | 修改内容 |
|---|---|---|---|
| v0.1 | 2026-08-19 | — | 初稿：基于 `GVPE-DOC-00` ~ `GVPE-DOC-26` v0.2 全套设计，登记 8 大类共 60+ 条实施前顾虑与疑问。 |

## 2. 文档目的

本文档回答以下问题：

- 在敲下 `cargo new` 之前，**哪些假设尚未验证**？
- 哪些设计决策**依赖未确定的外部输入**（编译器版本、平台特性、商业伙伴需求）？
- 哪些**已知风险**必须在 MVP 范围或实施顺序上做主动管控？
- 哪些**开放问题**需要在 Phase 1 之前得出答案，否则会卡住关键路径？

本文件**不是**：

- 实施计划 / 排期表（属项目管理范畴，本文件不规定 milestone 日期）；
- 详细 bug 列表（属实施期 `issue tracker` 范畴，本文件只登记"在敲第一行代码前就应警觉"的项）；
- 性能预算（见 `GVPE-DOC-14`）。

## 3. 适用范围

- **适用阶段**：MVP 实施启动**前**冻结；每条 QA 项状态变更需写入 §1 修订历史；
- **适用读者**：核心 crate 实现者、架构师、集成方代表、QA 负责人；
- **强制力**：严重度为 **Blocker** 的 QA 项在 Closed 之前**不得**进入对应模块的代码实现；严重度 **High** 的 QA 项**建议**在对应模块的首个 commit 之前 Closed；
- **状态机**：`Open` → `In-progress` → `Closed`（已解决）/ `Deferred`（明确推迟，含触发再审的条件）。

## 4. 术语定义

| 术语 | 释义 |
|---|---|
| QA 项 | 一条具体的顾虑 / 疑问 / 待验证假设；登记在本文件。 |
| Blocker | 严重度最高：未解决前相关模块不可启动。 |
| High | 严重度高：未解决前相关模块强烈建议不启动。 |
| Medium | 严重度中：实施期可边走边收。 |
| Low | 严重度低：记录备查，不阻塞实施。 |
| 责任人 | 该 QA 项在状态机内推进的具体负责人；默认指派见 §5。3。 |
| Closed | 已验证 / 已解决 / 已落地为代码或决策。 |
| Deferred | 主动推迟到指定阶段；必须附"再审触发条件"。 |
| 关联节 | 该 QA 项涉及的上游文档节号（`GVPE-DOC-NN §X.Y`）或需求 ID。 |

## 5. 类别、严重度与状态机

### 5.1 QA 类别（对应正文 §8.1 ~ §8.8）

| 类别代号 | 类别名称 | 典型问题 |
|---|---|---|
| D | 设计层 | 需求规约 / 架构 / 子系统设计是否完备、是否自洽。 |
| I | 实现层 | 算法路径、边界条件、错误恢复是否清晰。 |
| T | 技术选型 | 编译器 / crate / 平台层面的选择是否经得起实测。 |
| P | 性能与实时性 | 性能预算、内存、并发扩展性是否可达。 |
| F | FFI / 跨语言 | C ABI / 跨编译器 / 跨语言互操作是否健壮。 |
| Q | 测试 / CI | 确定性、回归、CI 矩阵是否覆盖关键路径。 |
| C | 文档 / 过程 | 文档同步、范围纪律、PR 流程。 |
| B | 业务 / 项目 | 团队能力、合作伙伴、范围 vs 时间。 |

### 5.2 严重度判定原则

- **Blocker**：未解决前，**整个 MVP**（或某子系统）不可启动；典型为编译期即发现的类型层矛盾、许可证硬冲突、目标平台不可达。
- **High**：未解决前，**对应子系统**首版实现可能要走回头路；典型为算法选择未定、关键约束未测、关键 API 形状未冻结。
- **Medium**：实施期可边走边收，但需在子系统首个 release 前 Closed；典型为局部代码风格、二级依赖选型、CI 矩阵扩展。
- **Low**：记录备查；不影响主线进度。

### 5.3 默认责任人

- D 类 → 架构师；
- I 类 → 子系统实现者；
- T 类 → 构建 / 工具链维护者；
- P 类 → 性能负责人；
- F 类 → `gvpe-ffi` 维护者；
- Q 类 → QA 负责人；
- C 类 → 文档维护者；
- B 类 → 项目负责人。

可在具体项上覆盖默认指派。

## 6. 关联需求 / 文档

- **上游（输入基线）**：
  - `GVPE-DOC-00` `docs/00_foundation/00_vision.md`
  - `GVPE-DOC-01` `docs/00_foundation/01_requirements.md`
  - `GVPE-DOC-02` `docs/00_foundation/02_physics_ontology.md`
  - `GVPE-DOC-03` ~ `GVPE-DOC-09`（架构 / 子系统设计）
  - `GVPE-DOC-10` ~ `GVPE-DOC-13`（模块设计）
  - `GVPE-DOC-14` ~ `GVPE-DOC-16`（横切）
  - `GVPE-DOC-17` ~ `GVPE-DOC-25`（详细设计）
  - `GVPE-DOC-26` `docs/03_cross_cutting/26_tech_selection.md`（技术选型基线）
- **下游（被消费于）**：
  - `GVPE-DOC-17` §18 详细设计在每个子系统实现前应对照本文件，关闭对应 Blocker / High 项；
  - 实施期 `issue tracker` 应以本文件为顶层 issue 模板。

## 7. 关联文档

- 同 §6 上游列表（24 份基线文档均作为本登记的输入）。
- 历史参考：`docs/archive/05_PRE_Risk_Issue_Register.md`（PRE 项目的同类登记，结构可借鉴，但 ID 体系不沿用）。

## 8. 审批签字

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 编写 | — | | |
| 校对 | | | |
| 审批 | | | |

---

## 9. 正文：QA 登记表

> 每条 QA 项一行；优先级自上而下。状态默认 `Open`，除非另行注明。关联节格式：`D-NN §X.Y` 或 `R-NFR-XXX` / `AC-XX` / `PROHIBIT-XX` / `NG-X`。

### 9.1 设计层顾虑（D 类）

| 编号 | 严重度 | 类别 | 描述 | 影响范围 | 缓解 / 验证方式 | 状态 | 责任人 | 关联节 |
|---|---|---|---|---|---|---|---|---|
| QA-D-01 | Blocker | D | `GVPE-DOC-04` §4.4 规定 `gvpe-runtime` 永不直接或间接 import `gvpe-graph` / `gvpe-vector`（`AC-02`），但 `gvpe-compiler` 是**唯一**可依赖两者的 crate；需在 workspace 拓扑与 `Cargo.toml` 中**机械化**强制此约束（`cargo tree` artifact 已在 `GVPE-DOC-26` §18.12.1 列入 CI 步骤，但仍需 dev-loop 中的快速校验脚本）。 | 编译期类型层 | 编写一个 `xtask`（或 `cargo alias`），每次 `cargo build` 前运行 `cargo tree` 并 grep 验证。 | Open | 架构师 | `D-04 §4.4`, `AC-02` |
| QA-D-02 | Blocker | D | `GVPE-DOC-03` §1 规定 PKG、Runtime Constraint Graph、Execution Graph 三类图**严格分离**——但**共享什么、不共享什么**仅给出原则，未给出共享语法的禁止清单。需明确："PKG 的 `NodeKind` 枚举**绝不**被 Runtime Constraint Graph 的 `ConstraintRow` 引用，即使二者在工作流上是上下游"——以免实现期通过 `pub use` 间接拉通。 | 类型层 / crate 边界 | 在 `gvpe-core` 定义 `marker trait` 区分三类图身份；`xtask` 验证。 | Open | 架构师 | `D-03 §1` |
| QA-D-03 | Blocker | D | `GVPE-DOC-11` §11.6 规定的"多向量物理签名"是**类型层面**区分（`GVPE-VEC-002`）——但**组合查询**（如同时按 material + motion 检索）的 API 形状未冻结：是 `SignatureQuery::And(MaterialFilter, MotionFilter)` 闭 enum，还是位掩码，还是 typestate？需在 `gvpe-vector` 首行代码前确定。 | 公共 API 形状 | spike：在 `gvpe-vector` 写 2-3 个候选 API 草稿，让集成方对比选择。 | Open | `gvpe-vector` 负责人 | `D-11 §11.6`, `R-VEC-002` |
| QA-D-04 | Blocker | D | `GVPE-DOC-21` §2 关闭了 `ONT-ISS-001`（graph write guard），但**其它 10 类**本体混淆（`D-02 §Review` 中列出的 11 类）目前**仅**逐项审过，**未**在代码层做硬性 guard；建议至少在 `gvpe-graph` / `gvpe-compiler` 中加编译期 / 运行期检查覆盖至少"类别混淆"与"过程 vs 实体混淆"两类。 | 本体完整性 | spike：列出 11 类混淆的"代码可检测"子集，写防御代码。 | Open | 架构师 + 本体负责人 | `D-02 §Review`, `AC-04` |
| QA-D-05 | High | D | `GVPE-DOC-04` §4.7 规定 `PhysicsLOD` 在 `RuntimeDescriptor` 中**预留**（`R-FR-007`），但 MVP 硬编码 `Lod0Full`——预留字段的**类型层默认**与**未来填充路径**未说明：是"用 `Option<LodSelector>` 留空"还是"用 `LodSelector::Full` 占位 + 注释"? | 公共 API 形状 | 在 `GVPE-DOC-17` §1.3 落实为具体字段定义。 | Open | 架构师 | `D-04 §4.7`, `R-FR-007` |
| QA-D-06 | High | D | `GVPE-DOC-12` §12.6 与 `D-23` 给出 energy / wave / field / process 的 schema 与 feature-gated 数值算法，但**何时激活**、**与求解器主循环的接入点**未给出：feature flag 命名（`feature = "energy-conservation"` 等）、`compile()` 阶段的介入点、运行时 cost 表达——需要 `GVPE-DOC-17` 补充。 | 子系统集成 | 在 `GVPE-DOC-17` §18 增补一节"feature-gated 子系统接入矩阵"。 | Open | 架构师 | `D-12`, `D-23` |
| QA-D-07 | High | D | `GVPE-DOC-13` §13 规定 3DGS 仅占位（`NG2`），但**占位的接口形状**与**未来 `gvpe-inference` 的边界**是模糊的：MVP 中 `gvpe-3dgs` crate 是否应仅是空 stub + `compile_error!`？还是给一个最小 trait？ | crate 拓扑 | 决定 stub 形态，列入 `GVPE-DOC-26` §6 表格备注。 | Open | 架构师 | `D-13 §13`, `NG2` |
| QA-D-08 | High | D | `GVPE-DOC-24` §24 给出 Fluid / FEM 接口预留（`ShapeDesc::FluidRegion`），但**与 MVP 求解器的边界**——`CompileError::UnsupportedModel` 何时抛、是编译期还是运行期——未细化。 | 错误模型 | 决定 `CompileError` 变体何时填充，列入 `GVPE-DOC-17` §13 错误模型。 | Open | 架构师 | `D-24` |
| QA-D-09 | High | D | `GVPE-DOC-08` §8.2 给出 arena / pool / slab 三种分配器，**但**对"何时用哪种"的选择规则未给出（虽然 §3 提及"帧内临时 = arena；固定大小复用 = pool；带世代 = slab"）。需要写一份"分配器选择决策表"作为开发者参考。 | 实施期 API 选型 | 在 `GVPE-DOC-08` 增补决策表；或在 `GVPE-DOC-17` §2.1-§2.3 用注释体现。 | Open | `gvpe-memory` 负责人 | `D-08` |
| QA-D-10 | High | D | `GVPE-DOC-05` §5.3 规定 Fast Mode / Deterministic Mode 架构上区分（`R-NFR-001`），但 MVP 是否提供 Deterministic Mode 的**最简骨架**（即使部分功能未实现）需要决策：是 `feature = "deterministic"` 完整骨架，还是仅文档说明？ | crate 拓扑 / feature flag | 决策 feature 命名，列入 `GVPE-DOC-26` §6。 | Open | 架构师 | `D-05 §5.3`, `R-NFR-001` |
| QA-D-11 | High | D | `GVPE-DOC-25` §25 规定 GPU 后端**接口**（`GpuSolverBackend` trait）但**MVP 不实现**（`NG3`）；trait 本身的最小方法集（dispatch / upload / download / barrier）需要冻结，否则未来 `gvpe-gpu` crate 的首版实现会反复改 trait。 | 公共 API 形状 | 在 `GVPE-DOC-17` 增补 trait 形状小节。 | Open | 架构师 | `D-25`, `NG3` |
| QA-D-12 | Medium | D | `GVPE-DOC-22` §22 规定"deterministic V1 签名提取"（`GVPE-DOC-22` 的核心承诺），但**"deterministic" 的边界**需明确：跨平台（Win/Linux/macOS）位级一致？还是仅同平台同 build 一致？前者是 `R-NFR-001` 全文兑现；后者是 MVP 现实。 | 性能 vs 一致性 | 在 `GVPE-DOC-22` 加段落明确；同步 `R-NFR-001` 实现范围。 | Open | `gvpe-vector` 负责人 | `D-22`, `R-NFR-001` |
| QA-D-13 | Medium | D | `GVPE-DOC-19` §19 规定 XPBD Gen 2 与 SI Gen 1 在 `ConstraintRow.compliance` 字段上兼容，但**两种求解器的 trait 抽象**未给出：是 `trait Solver { fn step(...); }` 一个 trait + 两种 impl，还是分 `SequentialImpulseSolver` / `XpbdSolver` 两条线？ | 公共 API 形状 | spike：在 `GVPE-DOC-17` §6 / §7 决定 trait 形状。 | Open | `gvpe-solver` 负责人 | `D-19 §19` |
| QA-D-14 | Medium | D | `GVPE-DOC-18` §18 规定 `JointRow` 分解为 `ConstraintRow`，但**关节生命周期**与**求解器内部岛的拆分**关系未细化：固定关节的两个 body 必须在同一岛？断开关节时岛的拆分如何收敛？ | 子系统行为 | 在 `GVPE-DOC-18` 增补岛屿收敛规则小节。 | Open | `gvpe-constraint` 负责人 | `D-18` |
| QA-D-15 | Medium | D | `GVPE-DOC-17` §17.3（broad phase SAP）使用 insertion sort 假设"前帧顺序近似"，但**冷启动 / 物体突然大量生成**时该假设崩坏——退化策略是 fallback 到完全 sort？还是允许 worst-case O(n²)？ | 性能 vs 实现复杂度 | 决定退化策略，写入 `GVPE-DOC-17` §3.2。 | Open | `gvpe-collision` 负责人 | `D-17 §3.2` |
| QA-D-16 | Medium | D | `GVPE-DOC-02` §18 给出 `Process`（熔化、凝固等）本体，但**过程状态机**与求解器主循环的同步：是否每帧 query 状态？还是事件触发？与 feature gate 关系？ | 子系统集成 | 写入 `GVPE-DOC-23` 的 worked example。 | Open | 本体 + 求解器负责人 | `D-02 §18`, `D-23` |
| QA-D-17 | Low | D | `GVPE-DOC-04` §4.2 给出三大空间到模块的映射，但**`gvpe-compiler` 内部子模块**（`compile()` / `validate()` / `lower()` 等）未规划子模块布局。 | 代码组织 | 由 `gvpe-compiler` 负责人在 spike 阶段决定。 | Open | `gvpe-compiler` 负责人 | `D-04 §4.2` |
| QA-D-18 | Low | D | `GVPE-DOC-21` §21 规定"ontology-mirroring closed `NodeKind` enum"——新增本体类别时**需要 Rust 编译器参与**（每个新 NodeKind 变体都是 enum 变体）。该约束是否过强？是否提供 `#[non_exhaustive]` 逃生口？ | 演化路径 | spike：评估 `#[non_exhaustive]` 与 closed enum 的取舍。 | Open | `gvpe-graph` 负责人 | `D-21 §21` |

### 9.2 实现层顾虑（I 类）

| 编号 | 严重度 | 类别 | 描述 | 影响范围 | 缓解 / 验证方式 | 状态 | 责任人 | 关联节 |
|---|---|---|---|---|---|---|---|---|
| QA-I-01 | Blocker | I | **Sequential Impulse 求解器**是 MVP 核心（`R-FR-002`），工作量以"人月"计。**冷启动**（没有任何参考实现）的 SI 实现需要明确的"小步快跑"策略：先做 sphere-sphere + 单岛 + 无 sleeping 的最小骨架，再逐步加 box、plane、关节、摩擦、sleeping。每个里程碑需对应可运行 demo + 单元测试。 | 排期 / 范围纪律 | 写一份"SI 实施里程碑表"作为 `GVPE-DOC-07` 附录。 | Open | `gvpe-solver` 负责人 | `D-07 §7` |
| QA-I-02 | Blocker | I | **接触流形稳定性**（`R-FR-002` 中的 contact manifold）：当 penetration depth 接近 0 但未越过 0、或者 friction 接近 0 / 接近 1 的边界情况，会出现"震荡"或"穿透"。需要 spike：先做 2-body sphere-box 场景，连续 10000 步确认无穿透累积。 | 数值稳定性 | spike 写一个最小 demo 跑 10K 步。 | Open | `gvpe-solver` 负责人 | `R-FR-002`, `D-17 §3.3` |
| QA-I-03 | Blocker | I | **sleeping 状态机**（`R-FR-002`）：进入 sleeping 的判定阈值、唤醒的判定阈值、跨岛的 sleep 同步、刚体 kinematic 移动是否唤醒邻居——这些边界条件 `GVPE-DOC-07` 给出了原则但未给数值。 | 数值行为 | spike：固定场景跑 100K 步观察 sleep 行为；调阈值。 | Open | `gvpe-solver` 负责人 | `D-07 §7` |
| QA-I-04 | High | I | **CCD（continuous collision detection）**（`D-18`）：`conservative advancement` 在 fast-moving object 上的步长缩减可能放大到不可接受；且**与 sleeping** 的交互（sleeping 物体是否参与 CCD？）需要明确。 | 数值 / 性能 | spike：bullet / box tunnel 场景。 | Open | `gvpe-collision` 负责人 | `D-18` |
| QA-I-05 | High | I | **多物理岛间的同步开销**（`D-09`）：岛数 = N 时，岛同步 vs 岛内并行的临界点在哪？需要实测：N=1, 4, 16, 64, 256 岛的不同 workload 下的并行收益。 | 性能 / 扩展性 | 实测：criterion bench 跑一组场景。 | Open | `gvpe-scheduler` 负责人 | `D-09 §9` |
| QA-I-06 | High | I | **Determinism Mode 的浮点求和顺序**（`OQ-01`）：即使单线程，跨平台（不同 SIMD 实现、不同 libm）浮点结果可能 bit-differ；MVP 是否承诺**同平台同 build** 一致即可？ | 性能 vs 一致性 | 决策；写入 `GVPE-DOC-05` §5.3。 | Open | 架构师 | `OQ-01`, `R-NFR-001` |
| QA-I-07 | High | I | **work-stealing 队列的并发安全**（`D-09`）：自研队列必须经过 `miri` + 大量 fuzz 测试；任何 use-after-free / data race 都是 Block 整个 MVP 的事件。 | 正确性 | `miri` 在 CI 跑 + `cargo-fuzz`（评估）。 | Open | `gvpe-scheduler` 负责人 | `D-09 §9` |
| QA-I-08 | High | I | **AABB 重建与 broad phase 增量更新**：当 body 移动时 AABB 重建的频率——每帧重建 vs dirty-flag 增量更新——选择影响性能与代码复杂度。 | 性能 vs 复杂度 | 决定策略，写入 `GVPE-DOC-17` §3.2。 | Open | `gvpe-collision` 负责人 | `D-17 §3.2` |
| QA-I-09 | High | I | **窄相 SAT 的实现正确性**：box-box 15 轴分离（3+3+9），box-plane 5 轴，sphere-box 3 轴——任何轴遗漏或重叠检测错误都直接导致穿透。**必须有参照实现**（如参考 Bullet / PhysX 的源码）作为对照测试。 | 正确性 | 对照实现写 unit test。 | Open | `gvpe-collision` 负责人 | `D-17 §3.3` |
| QA-I-10 | High | I | **SoA / AoSoA 实际收益**（`D-05`）：声称 cache-friendly，但实际**只有 hotspot 字段**应 SoA；冷字段保持 struct 内嵌。需要 spike：先标 struct-of-vec，跑 criterion 看收益。 | 性能 | 实测。 | Open | `gvpe-dynamics` 负责人 | `D-05` |
| QA-I-11 | Medium | I | **parking_lot 拒绝**后的 hot path 锁选择：标准 `std::sync::Mutex` 在高度竞争下可能成为瓶颈；评估 `RwLock`、`atomic` lock-free、crossbeam `ShardedLock` 之间的取舍。`GVPE-DOC-26` §18.6.2 已拒绝 parking_lot / crossbeam 进核心，但**评估结论**本身需要 spike 留底。 | 性能 | spike：写一个高竞争 benchmark。 | Open | `gvpe-scheduler` 负责人 | `D-26 §18.6.2` |
| QA-I-12 | Medium | I | **编译器错误传播**（`D-24`）：`CompileError::UnsupportedModel` 是编译期还是运行期？如果是运行期，错误码与 panic 的边界在哪里？ | 错误模型 | 决策 + 落实 `GVPE-DOC-17` §13。 | Open | `gvpe-compiler` 负责人 | `D-24` |
| QA-I-13 | Medium | I | **跨 crate 的 allocator**：每个 crate 单独 `#[global_allocator]` 还是共享？是 `jemalloc` / `mimalloc` 评估为 dev-dep？ | 性能 | 决策；写入 `GVPE-DOC-26` §18.6。 | Open | `gvpe-memory` 负责人 | `D-26 §18.6` |
| QA-I-14 | Medium | I | **日志接入点**（`D-26 §18.9`）：MVP 决定核心 crate 不引入日志库，但**调试期**必然需要 `println!` 或条件编译的 trace——如何不污染 release 性能？ | 调试 vs 性能 | 决定 trace 机制（`#[cfg(debug_assertions)]` 块 / `tracing` feature gate）。 | Open | 架构师 | `D-26 §18.9` |
| QA-I-15 | Medium | I | **接触点（contact point）数量上限**：单对 box-box 最多 4 个 contact point，写死 4 还是动态分配？ | 内存 | 决定；写入 `GVPE-DOC-17` §3.3。 | Open | `gvpe-collision` 负责人 | `D-17 §3.3` |
| QA-I-16 | Medium | I | **求解迭代次数**的 runtime 调参：`solver_iterations: u16`（`GVPE-DOC-17` §1.2）是 Runtime 在每帧可调，还是 compile-time？ | API 形状 | 决策 + 落实。 | Open | `gvpe-solver` 负责人 | `D-17 §1.2` |
| QA-I-17 | Low | I | **`#[inline]` 与 `#[cold]` 标注**策略：核心热函数应 `#[inline]`，错误路径应 `#[cold]`。需要 spike 看 cargo-bloat / cargo-asm 确认实际内联。 | 性能 | 实测。 | Open | 各 crate 负责人 | (跨 crate) |
| QA-I-18 | Low | I | **特征门控**（feature-gate）的语义版本化：`feature = "energy-conservation"` 一旦公开，后续删除 / 改名需 semver 慎重。 | 演化路径 | 在 `GVPE-DOC-26` §18.15 加入 feature 命名规约。 | Open | 架构师 | `D-26 §18.15` |

### 9.3 技术选型顾虑（T 类）

| 编号 | 严重度 | 类别 | 描述 | 影响范围 | 缓解 / 验证方式 | 状态 | 责任人 | 关联节 |
|---|---|---|---|---|---|---|---|---|
| QA-T-01 | Blocker | T | **`portable SIMD` 仍未稳定**（`D-26 §18.5.1`）：2026 年仍属 nightly。若 MSRV 1.75 不能用 `core::simd` / `std::simd`，则 SIMD 必须**全部走 vendor intrinsics**，跨平台覆盖（x86_64 SSE/AVX2/AVX-512 + aarch64 NEON + 未来 RISC-V V）工作量翻倍。 | 性能 / 工程量 | spike：实测 `core::simd` 在目标 MSRV 的可用性；不达预期则全 vendor intrinsics。 | Open | 性能负责人 | `D-26 §18.5.1` |
| QA-T-02 | Blocker | T | **跨平台 SIMD 行为差异**（`D-26 §18.5.4`）：NaN 处理、denormal flush、rounding mode 在不同平台可能差 1 ULP。**测试**需要覆盖：x86_64 Linux/MSVC、aarch64 macOS/Linux。 | 一致性 | CI 矩阵 + 对照 unit test。 | Open | 性能负责人 | `D-26 §18.5.4`, `R-PERF-002` |
| QA-T-03 | High | T | **`cbindgen` 与 Rust 新版本兼容**（`D-26 §18.7.1`）：Rust 2024 edition 引入新语法（如 `let-else`、`gen blocks`）时，`cbindgen` 是否能正确生成 C 头？锁定 `cbindgen` 版本号是必要的。 | CI / 工具链 | CI 锁定 `cbindgen` 版本；每次升级前 spike。 | Open | 构建维护者 | `D-26 §18.7.1` |
| QA-T-04 | High | T | **`cargo tree` 在 monorepo 演化中的可靠性**：当 crate 数量增加、feature 组合增多，`cargo tree` 的输出可能变得很复杂；其作为 `AC-02` 验证工具的**可读性**需要持续评估。 | CI / 可读性 | 写一个 `xtask` 把 `cargo tree` 输出 diff 化，CI 中对 PR 跑增量 diff。 | Open | 构建维护者 | `AC-02` |
| QA-T-05 | High | T | **CI 矩阵的运行时间**（`D-26 §18.12.1`）：5 OS × arch × {fmt, clippy, test--all-features, test--no-default-features, test simd-only, test--no-default-features, bench, tree, deny, miri}，单次 PR 跑全量可能 30-60 min。需要分层：PR 跑核心集、main 分支跑全量。 | CI 反馈速度 | 写 `pr-checks.yml` 与 `full-ci.yml` 两份 workflow。 | Open | 构建维护者 | `D-26 §18.12.1` |
| QA-T-06 | High | T | **MSRV 升级路径**（`D-26 §18.2.3`）：游戏引擎集成方（Unity / Unreal）通常滞后 Rust 版本。MSRV 升级若激进，会拒绝一部分集成方。建议**首次发版** MSRV 尽量保守。 | 集成方可达性 | 决策 MVP 首发 MSRV（候选 1.74 / 1.75 / 1.76）。 | Open | 架构师 | `D-26 §18.2.3` |
| QA-T-07 | High | T | **`cargo deny` ban list 维护负担**（`D-26 §18.13`）：ban list 列出**显式拒绝**的库；新增依赖每次都要对照。需要 CI 强制 + 维护者纪律。 | 过程纪律 | CI 跑 `cargo deny check bans`；PR template 强制填"已对照 ban list"。 | Open | 构建维护者 | `D-26 §18.13` |
| QA-T-08 | High | T | **RUSTSEC 公告处理**：依赖库可能爆出 CVE；`cargo audit` / `cargo deny` 检查 advisories。需要订阅 RUSTSEC 邮件列表 + 季度评估。 | 安全 | 订阅 + 季度评估。 | Open | 安全负责人 | (新项, 引入) |
| QA-T-09 | Medium | T | **`proptest` 缩小的实用性**（`D-26 §18.10.1`）：物理仿真的不变量是"无穿透 / 能量守恒"——proptest 能否自动缩到"穿透极小但非零"的状态？经验上 proptest 对连续状态空间效果差。 | 测试效果 | spike：写一个 proptest 跑 penetration depth 边界。 | Open | QA 负责人 | `D-26 §18.10.1` |
| QA-T-10 | Medium | T | **`miri` 性能开销**（`D-26 §18.11.1`）：miri 解释执行，单测 miri 跑可能慢 10-100x。CI 全量跑不现实。 | CI 时间 | 仅核心 unsafe crate 跑；按需触发。 | Open | 构建维护者 | `D-26 §18.11.1` |
| QA-T-11 | Medium | T | **`criterion` 自身噪声**（`D-26 §18.10.1`）：单次 5% 噪声，趋势检测需要 5+ 次重复。CI 上 5 次重复可能不可接受。 | CI 时间 | 决策：CI 仅跑 smoke bench，完整 bench 留 nightly / 手动。 | Open | 性能负责人 | `D-26 §18.10.1` |
| QA-T-12 | Medium | T | **`bytemuck` 的 `Pod` 派生安全**：`Pod` 派生要求类型布局确定；任何 `#[repr(C)]` 结构体加了 `f32` / 指针混用时，bytemuck 会要求手工 `unsafe impl`。需要谨慎逐个标注。 | 正确性 | code review 重点；unsafe 集中模块。 | Open | `gvpe-core` 负责人 | `D-26 §18.5.4` |
| QA-T-13 | Medium | T | **`rust-toolchain.toml` 锁定 vs 灵活性**（`D-26 §18.2.1`）：锁定 stable channel 但 patch 版本让 rustup 自动取，可能引入编译器回归。 | 一致性 | 锁定 stable + 具体 patch 版本号（如 `1.75.0`）。 | Open | 构建维护者 | `D-26 §18.2.1` |
| QA-T-14 | Low | T | **未来 Rust Edition 2024 升级窗口**（`D-26 §18.2.1`）：当 2024 edition 在目标 MSRV 上稳定且生态适配时再切换。**不应**作为 MVP gate。 | 未来 | 列入 Phase 2 评估。 | Deferred | 架构师 | `D-26 §18.2.1` |
| QA-T-15 | Low | T | **`mdbook` 评估**（`D-26 §6`）：本轮以纯 Markdown 为主；若文档膨胀到需要搜索 / 目录 / 主题切换，再评估 mdbook。 | 工程量 | 评估窗口：文档数 > 30 时再决定。 | Deferred | 文档维护者 | `D-26 §6` |

### 9.4 性能与实时性顾虑（P 类）

| 编号 | 严重度 | 类别 | 描述 | 影响范围 | 缓解 / 验证方式 | 状态 | 责任人 | 关联节 |
|---|---|---|---|---|---|---|---|---|
| QA-P-01 | Blocker | P | **MVP 性能基线是否可达**（`R-PERF-001`）：单中端 CPU 核 60 Hz + 数百 dynamic body——Sequential Impulse 在没有多年优化的"新手实现"下，达成此目标的把握有多大？**无历史数据支撑**。 | 排期 / 范围 | Phase 1 spike：先用最简 SI 实现 sphere-sphere 100 body 跑 60 Hz，看 baseline 距离。 | Open | 性能负责人 | `R-PERF-001`, `D-14` |
| QA-P-02 | Blocker | P | **热路径分配审计**（`R-NFR-002`）：即使使用 arena，`fetch_add` 的 `AtomicUsize` 在多线程访问同一 arena 时仍是瓶颈（`D-17 §2.1` 已识别）。多线程下 arena 应该是 thread-local。 | 性能 / 正确性 | spike：实测多线程 arena 性能。 | Open | `gvpe-memory` 负责人 | `R-NFR-002`, `D-17 §2.1` |
| QA-P-03 | High | P | **GC-like pause 红线**（`R-PERF-002`）：Rust 无 GC，但 `String` / `Vec` / `HashMap` 隐式分配在热路径仍可能成为"GC-like pause"。需要 lint + code review 强制热路径只使用预分配容器。 | 性能 | `cargo clippy` 配 `clippy::pedantic` + 手工 code review 重点。 | Open | 架构师 | `R-PERF-002` |
| QA-P-04 | High | P | **接触点数量与求解开销**：SI 求解器对每个 `ConstraintRow` 做 K 次迭代；contact manifold 4 点 × 数百对 = 万级 ConstraintRow × K=10 迭代 = 10 万次 / 帧。需要 spike 实测 K=4 vs K=8 vs K=16 的 perf / 质量 trade-off。 | 性能 / 求解质量 | 实测。 | Open | `gvpe-solver` 负责人 | `D-07` |
| QA-P-05 | High | P | **AABB rebuild vs dirty-flag**：每帧重建所有 AABB 简单但 O(n)；dirty-flag 增量更新复杂但 O(active bodies)。需要实测选定策略。 | 性能 | 实测。 | Open | `gvpe-collision` 负责人 | `QA-I-08` |
| QA-P-06 | High | P | **NUMA 跨 socket 性能**：服务器场景下，跨 socket 内存访问延迟是同 socket 的 2-3x。MVP 不做 NUMA-aware allocation，但需要在文档中明确"性能数字仅同 socket 有效"。 | 性能预期管理 | 文档化。 | Open | 性能负责人 | (新项) |
| QA-P-07 | High | P | **跨平台性能差异**：同一段代码在 MSVC vs clang vs gcc 下生成的目标码性能可差 20%+。`GVPE-DOC-26` §18.12.1 CI 矩阵会暴露此问题。 | 性能预期 | CI 全平台跑 bench；按平台分别报告。 | Open | 性能负责人 | `D-26 §18.12.1` |
| QA-P-08 | Medium | P | **link-time optimization (LTO) vs 构建时间**：`[profile.release] lto = "fat"` 提升性能但显著延长构建时间。CI 上 release build 多久能跑完？ | CI 时间 | spike：实测 LTO fat vs thin 的 perf 差 / 构建时间差。 | Open | 构建维护者 | `D-26 §18.3.3` |
| QA-P-09 | Medium | P | **PGO (Profile-Guided Optimization)**：Rust 支持 PGO 但工具链复杂；MVP 不一定需要，列入 Phase 2 评估。 | 性能 | 评估窗口：MVP 性能基线达成后。 | Deferred | 性能负责人 | (新项) |
| QA-P-10 | Medium | P | **SIMD 利用率**：是否所有热路径都成功 SIMD 化？`cargo-asm` / `cargo-bloat` 评估。 | 性能 | spike：broad phase / narrow phase / solver 各取 1 函数看汇编。 | Open | 性能负责人 | `D-26 §18.5.3` |
| QA-P-11 | Low | P | **C 端**（游戏引擎）调用 GVPE 的 FFI 开销：每次 `gvpe_step()` 跨 FFI 边界的固定成本。 | 集成方性能 | 文档化，列入集成方 benchmark。 | Open | 集成负责人 | `D-10` |

### 9.5 FFI / 跨语言顾虑（F 类）

| 编号 | 严重度 | 类别 | 描述 | 影响范围 | 缓解 / 验证方式 | 状态 | 责任人 | 关联节 |
|---|---|---|---|---|---|---|---|---|
| QA-F-01 | Blocker | F | **C ABI 版本稳定性**（`D-10`）：当 `PhysicsProfile` 字段调整时，旧的 C 客户端会崩溃。需要明确的 ABI 版本号机制：是 `gvpe_abi_version()` 函数返回 `u32`，还是 `#[repr(C)] struct GvpeHeader`? | 集成方兼容 | spike：实现 `gvpe_abi_version()` + 头文件 semver 注释。 | Open | `gvpe-ffi` 负责人 | `D-10`, `R-FR-008` |
| QA-F-02 | Blocker | F | **panic 安全**（`D-26 §18.7.1`）：`catch_unwind` 在 C 边界捕获 panic 转化为 C 错误码——但**跨 `extern "C"` 边界的栈展开**本身是 UB（即使 catch 住）。需要确保 `panic = "abort"` 在 release build，或**所有** FFI 函数都用 `catch_unwind` 包裹。 | 正确性 / UB | `[profile.release] panic = "abort"`（仅 `gvpe-ffi`）+ 全 FFI 函数 catch_unwind。 | Open | `gvpe-ffi` 负责人 | `D-26 §18.7.1` |
| QA-F-03 | Blocker | F | **句柄在 FFI 边界的有效性**：集成方持有 `BodyHandle` 后，Rust 端 free 该 body，集成方调用 `gvpe_set_position(old_handle, ...)`——是返回错误还是 UB？ | 集成方正确性 | 句柄世代检查 + 明确错误码（`GVPE_E_INVALID_HANDLE`）。 | Open | `gvpe-ffi` 负责人 | `D-17 §1.1` |
| QA-F-04 | High | F | **批处理数据交换**（`R-FR-008`）：批量传入 / 传出数据的内存由谁分配？集成方 malloc 传入，GVPE 写后集成方 free？ | 内存所有权 | 决策 + 头文件 doc 明确。 | Open | `gvpe-ffi` 负责人 | `R-FR-008` |
| QA-F-05 | High | F | **跨编译器的 struct layout 差异**：`#[repr(C)]` 在 MSVC / clang / gcc 之间布局一致，但**对齐**可能差。需要 `(align(N))` 显式指定。 | 正确性 | 对每个 repr(C) struct 显式 align。 | Open | `gvpe-ffi` 负责人 | `D-26 §18.4.2` |
| QA-F-06 | High | F | **C++ 集成方（Unreal / Godot）通过 C ABI 调用**：C++ 侧需自己写一层 thin wrapper。本项目**不**提供 cxx/autocxx 包装（`D-26 §18.7.2`）。集成方需要自维护 wrapper。 | 集成方工作量 | 文档化；提供最少 C++ example。 | Open | 集成负责人 | `D-26 §18.7.2` |
| QA-F-07 | Medium | F | **`cbindgen` 生成的 C 头可读性**：自动生成的 C 头可能对集成方不友好（命名风格、注释）。需要 `cbindgen.toml` 调优。 | 集成方体验 | 调优 `cbindgen.toml`。 | Open | `gvpe-ffi` 负责人 | `D-26 §18.7.1` |
| QA-F-08 | Medium | F | **C 端字符串处理**：GVPE 错误信息需要返回 C 字符串——生命周期？`const char*` 由 GVPE 静态持有 vs 集成方释放？ | 内存所有权 | 决策（推荐 GVPE 静态持有 + 错误码）。 | Open | `gvpe-ffi` 负责人 | `D-10` |
| QA-F-09 | Low | F | **C 端多线程**：`gvpe_step()` 是否线程安全？还是每个 thread 一个 Runtime？ | 集成方架构 | 文档化：MVP 不支持多 Runtime 并发。 | Open | 集成负责人 | `D-09` |

### 9.6 测试 / CI 顾虑（Q 类）

| 编号 | 严重度 | 类别 | 描述 | 影响范围 | 缓解 / 验证方式 | 状态 | 责任人 | 关联节 |
|---|---|---|---|---|---|---|---|---|
| QA-Q-01 | Blocker | Q | **确定性回放 harness 何时落地**（`R-NFR-001` / `AC-01`）：自研 determinism harness 是 MVP gate，但需要 spike：先选 1-2 个最小场景，把回放跑通，再扩展。 | 验收 | spike：1 个场景跑通。 | Open | QA 负责人 | `R-NFR-001`, `AC-01` |
| QA-Q-02 | Blocker | Q | **`cargo tree` 的 artifact 验证**（`AC-02`）：CI 上捕获 `cargo tree` 输出到 artifact，code review 检查。但**何时**触发 review？每次 PR？只在合并前？ | 验收 | 决策：每次 PR 跑 + 增量 diff。 | Open | 构建维护者 | `AC-02` |
| QA-Q-03 | High | Q | **测试覆盖率基线**：MVP 完成时整体测试覆盖率目标（行覆盖率 80%？分支 70%？）未设定。 | 质量 | 决策；列入 `GVPE-DOC-15` §15 量化。 | Open | QA 负责人 | `D-15` |
| QA-Q-04 | High | Q | **Solver 单元测试的"正确性参考"**：自研 SI 没有金标准。需要：(1) 对照 Bullet / PhysX 行为；(2) 对照 Box2D 经典 demo（如 stacked boxes）。 | 正确性 | 编写对照 test（仅对比行为不嵌入第三方库）。 | Open | `gvpe-solver` 负责人 | `D-07` |
| QA-Q-05 | High | Q | **CI 时间预算**：完整 CI 跑多久算合格？单 PR 反馈 < 10 min？ | 反馈速度 | 实测。 | Open | 构建维护者 | `QA-T-05` |
| QA-Q-06 | High | Q | **nightly CI / main 分支 CI 区别**：PR 跑什么、main 跑什么、release 跑什么？分层策略。 | CI 资源 | 写分层 workflow。 | Open | 构建维护者 | `D-26 §18.12.1` |
| QA-Q-07 | Medium | Q | **fuzz testing**（`D-26 §18.10.1`）：`cargo-fuzz` 仅 nightly；CI 上跑 fuzz 多少时间合理？30 min / 1 hour / overnight？ | 鲁棒性 | spike：估算 fuzz 收益 / 成本。 | Open | QA 负责人 | `D-26 §18.10.1` |
| QA-Q-08 | Medium | Q | **GPU / 异构 CI**：MVP 不做 GPU，但 CI 矩阵是否要为未来 GPU 预留（如 GitHub-hosted GPU runner）？ | 未来 | Phase 2 评估。 | Deferred | 构建维护者 | (新项) |
| QA-Q-09 | Low | Q | **CHANGELOG 自动生成**：`git-cliff` 等工具。是否值得引入？ | 文档 | 评估窗口：release 数 > 5 时。 | Deferred | 文档维护者 | (新项) |

### 9.7 文档 / 过程顾虑（C 类）

| 编号 | 严重度 | 类别 | 描述 | 影响范围 | 缓解 / 验证方式 | 状态 | 责任人 | 关联节 |
|---|---|---|---|---|---|---|----|---|
| QA-C-01 | High | C | **doc drift 风险**（`D-26 §18.15`）：27 份文档在实施期可能与代码脱节。**何时同步**？每个 PR 强制更新？仅 release 时？ | 文档质量 | PR template 强制"关联文档更新"项。 | Open | 文档维护者 | `D-26 §18.15` |
| QA-C-02 | High | C | **`GVPE-DOC-26` 选型变更的纪律**：26 号文档第 18.15 节给出的变更流程是"提案 → DOC-16 审查 → review → 落地"。**实际 PR 中是否被遵守**？需要 CI 检查或 PR 模板强制。 | 过程纪律 | PR template + CI check。 | Open | 架构师 | `D-26 §18.15` |
| QA-C-03 | High | C | **本 QA 登记表的更新纪律**（`§3`）：每条 QA 项状态变更需写修订历史。**实际**会持续更新吗？需要"每两周 review 一次"机制。 | 过程纪律 | 双周 review 会议（决策）；或设置 reminder。 | Open | 文档维护者 | `§3` |
| QA-C-04 | Medium | C | **归档策略**：v0.1 → v0.2 → v1.0，**已废弃**的文档是否归档到 `docs/archive/`？类似 PRE 那套？ | 文档 | 决策。 | Open | 文档维护者 | `docs/archive/` |
| QA-C-05 | Medium | C | **范围纪律**（`R-NG1` ~ `R-NG5` + `R-PROHIBIT-01~06`）：MVP 严格不引入 fluid / 3DGS / GPU / LLM。但实施期总有"顺手加一下"的诱惑。**怎么守**？ | 范围 | 每周 scope review；CI 检查 banned feature。 | Open | 项目负责人 | `R-NG1` ~ `R-NG5`, `R-PROHIBIT-*` |
| QA-C-06 | Low | C | **跨语言注释**：代码注释用中文还是英文？`GVPE-DOC-26` §18 决定"叙事中文，专名保留英文"——代码注释是叙事吗？ | 一致性 | 决策：技术注释英文；用户文档中文。 | Open | 文档维护者 | `D-26 §18` |
| QA-C-07 | Low | C | **`unsafe` 集中策略**：是否所有 `unsafe` 集中到一个 crate（如 `gvpe-unsafe`），其它 crate 通过 safe wrapper 访问？ | 正确性 / 可审计性 | 决策。 | Open | 架构师 | `R-NFR-003` |

### 9.8 业务 / 项目顾虑（B 类）

| 编号 | 严重度 | 类别 | 描述 | 影响范围 | 缓解 / 验证方式 | 状态 | 责任人 | 关联节 |
|---|---|---|---|---|---|---|---|---|
| QA-B-01 | Blocker | B | **团队对 Rust 物理引擎开发的经验曲线**：从零自研 rigid-body solver 是多年的工作量。团队 Rust 熟练度、刚体物理熟悉度、与游戏引擎集成经验是否到位？**MVP 6-12 月的排期假设**可能严重低估。 | 排期 | 评估团队实际经验；设定 2 周 spike 验证 solver 最小骨架。 | Open | 项目负责人 | (隐含) |
| QA-B-02 | Blocker | B | **MVP 范围 vs 时间的硬约束**：刚性范围（v0.1 草案承诺的 MVP）vs 实际时间的平衡。MVP 范围已经在 27 份文档里反复强化，但**实施期必然出现的 trade-off**（如 sleeping 推迟、CCD 推迟）需要预先有应对机制。 | 范围 vs 时间 | 决策：MVP 推迟某项时的"放行 vs 砍范围"流程。 | Open | 项目负责人 | `R-NG1` ~ `R-NG5` |
| QA-B-03 | High | B | **集成方早期介入**（`R-FR-008`）：Unity / Unreal / Godot 集成方是否在 MVP 期间提供反馈？C ABI 设计是否经过集成方 review？ | 集成方体验 | 寻找 1-2 个集成方 pilot；ABI review。 | Open | 集成负责人 | `R-FR-008`, `D-10` |
| QA-B-04 | High | B | **与已归档 PRE 的对照**（`docs/archive/`）：PRE 是 retrieval-first 思路，GVPE 是 self-developed-first 思路。**PRE 失败/废弃的原因**有哪些可以借鉴？ | 经验教训 | 阅读 `archive/05_PRE_Risk_Issue_Register.md`，对照本文件查漏。 | Open | 架构师 | `docs/archive/` |
| QA-B-05 | High | B | **许可证风险（具体库）**（`R-LIC-001`）：`cbindgen` MPL-2.0、`criterion` MIT/Apache-2.0、`thiserror` MIT/Apache-2.0——这些**已经**确认 OK，但 **`miri` 自身**（Rust 项目一部分）的许可证、`cargo deny` 的依赖传递许可证、CI 上 GitHub Actions 的输出 artifact 许可证——**次级**许可证审查待做。 | 许可证合规 | 写 `deny.toml` 锁定；CI 跑 `cargo deny check licenses`。 | Open | 许可证负责人 | `R-LIC-001`, `D-16` |
| QA-B-06 | Medium | B | **出口管制 / 加密相关**：物理引擎本身**可能**不涉及加密，但**依赖传递**中若有加密库（`ring` / `rustls`）可能触发出口管制。`D-26 §18.13` 已拒绝核心使用加密库，但**离线工具 / CI** 中是否有？ | 法务 | 检查所有 dev-dep。 | Open | 许可证负责人 | (新项) |
| QA-B-07 | Medium | B | **社区贡献治理**：开源后（如果开源）的 PR review 流程、CLA、贡献者协议。 | 社区 | 决策。 | Deferred | 项目负责人 | (新项) |
| QA-B-08 | Low | B | **品牌 / 命名**：GVPE / Physis / 子模块命名是否一致？ | 一致性 | 命名清单 review。 | Open | 文档维护者 | (跨文档) |

## 10. 附录：状态汇总

| 状态 | 计数 | 说明 |
|---|---|---|
| Open | 60+ | 默认状态；待处理。 |
| In-progress | 0 | 进行中（实现期填入）。 |
| Closed | 0 | 已解决（实施期填入）。 |
| Deferred | 4 | 主动推迟，含再审触发条件（QA-T-14, QA-T-15, QA-P-09, QA-Q-08, QA-B-07）。 |

| 严重度 | 计数 |
|---|---|
| Blocker | ~17 |
| High | ~25 |
| Medium | ~15 |
| Low | ~7 |

| 类别 | 计数 |
|---|---|
| D 设计 | 18 |
| I 实现 | 18 |
| T 技术 | 15 |
| P 性能 | 11 |
| F FFI | 9 |
| Q 测试 | 9 |
| C 文档/过程 | 7 |
| B 业务/项目 | 8 |

> 计数为 v0.1 初稿登记数；后续登记或合并请更新本节。

## 11. 附录：与本登记关联的"前置"决策

以下决策**强烈建议**在 MVP 实施启动**前**做出，否则相应 QA 项会升级严重度：

1. **MSRV 首发版本**（`QA-T-06`）——影响所有 crate 的代码风格与依赖选择；
2. **`PhysicsSignature` 组合查询 API 形状**（`QA-D-03`）——`gvpe-vector` 首行代码前置；
3. **CCD MVP 范围**（`QA-I-04`）——是否在 MVP 内；若推迟，需明确推迟到 Phase 几；
4. **C ABI 版本号机制**（`QA-F-01`）——`gvpe-ffi` 首行代码前置；
5. **panic 政策**（`QA-F-02`）——`gvpe-ffi` 的 `Cargo.toml` `[profile.release]` 设置前置；
6. **`feature = "deterministic"` 是否骨架化**（`QA-D-10`）——影响 crate 拓扑；
7. **cargo tree artifact review 机制**（`QA-D-01`, `QA-Q-02`）——CI 流程前置。

## 12. 附录：实施期复审节奏（建议）

- **每周**：review 全部 Open 项中**新增**的、或**状态变更**的；
- **每两周**：对所有 Blocker / High 项做一次集中审视；
- **每个 Phase 边界**（MVP → Phase 1 → ...）：全表重审一次，关闭已解决项，重新评估未解决项的严重度。

## 13. 附录：v0.5 决策关闭的 Blocker 项

> 2026-08-20 实施期启动 v0.5 决策登记，7 个前置决策全部敲定（详见 `28_workflow.md` §16）。下表列出被决策缓解的 Blocker / High QA 项状态变更。

| QA ID | 决策编号 | 状态变更 | 关闭方式 |
|---|---|---|---|
| QA-D-01 | DEC-007 | Open → Closed（缓解） | CI 跑 `cargo tree` 强制 AC-02；xtask 跑 dev-loop |
| QA-D-03 | DEC-002 | Open → Closed（缓解） | 闭 enum `SignatureQuery` + `#[non_exhaustive]` |
| QA-D-10 | DEC-006 | Open → Closed（缓解） | 所有 crate 加 `feature = "deterministic"` |
| QA-F-01 | DEC-004 | Open → Closed（缓解） | `gvpe_abi_version()` + `GVPE_ABI_VERSION` 常量 |
| QA-F-02 | DEC-005 | Open → Closed（缓解） | 核心 panic = bug；FFI `catch_unwind` + `panic = "abort"` |
| QA-I-04 | DEC-003 | Open → Closed（缓解） | CCD 推迟 Phase 2；MVP 仅离散 broad + narrow |
| QA-I-06 | DEC-006 | Open → Closed（缓解） | `DeterminismMode` 架构区分；MVP BestEffort |
| QA-Q-02 | DEC-007 | Open → Closed（缓解） | CI artifact + 增量 diff |
| QA-T-06 | DEC-001 | Open → Closed（缓解） | MSRV 锁定 1.75.0 |
| QA-T-13 | DEC-001 | Open → Closed（缓解） | `rust-toolchain.toml` 锁 patch 版本 |

> 注：**已缓解 ≠ 实施期可忽略**。上表条目在实施期 PR review 时仍需逐项检查（特别是 QA-F-02 FFI panic 安全、QA-D-01 cargo tree 阻断、QA-T-13 toolchain 锁）。
