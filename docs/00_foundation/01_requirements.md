# GVPE — Requirements（要件定義書）

> 输入基线：`00_vision.md`。ID 前缀：`GVPE-*`，按子系统分组（`FR` functional，`ONT` ontology，`GPH` graph，`VEC` vector，`RT` runtime，`PERF` performance，`LIC` licensing，`NFR` cross-cutting non-functional）。

## 0. 文档元数据

| 字段 | 取值 |
|---|---|
| 文档编号 | GVPE-DOC-01 |
| 文档类型 | 要件定義書 |
| 版本 | v0.2（标准化重写版） |
| 状态 | Draft |
| 适用阶段 | MVP / Phase 1+ |
| 关联系统 | GVPE / 总需求规约 |
| 上游文档（输入基线） | GVPE-DOC-00 |
| 下游文档（被消费于） | GVPE-DOC-02, GVPE-DOC-03, GVPE-DOC-04, GVPE-DOC-05, GVPE-DOC-06, GVPE-DOC-07, GVPE-DOC-08, GVPE-DOC-09, GVPE-DOC-10, GVPE-DOC-11, GVPE-DOC-12, GVPE-DOC-13, GVPE-DOC-14, GVPE-DOC-15, GVPE-DOC-16 |
| 编写者 | — |
| 审批者 | — |

## 1. 修订历史

| 版本 | 日期 | 修改者 | 修改内容 |
|---|---|---|---|
| v0.1 | — | — | 初稿（原始版本） |
| v0.2 | 2026-08-19 | 标准化 pass | 转为 IPA 要件定義書 格式；叙事中文化；保留全部技术内容；补充文档元数据、修订历史、术语定义、关联文档、审批签字等标准章节；将原 §5–§10 的功能/非功能/子系统需求 ID 显式归并到 IPA 模板 §6 / §7 / §8 |

## 2. 文档目的

本文档在 `00_vision.md` 给出的总纲基础上，给出 GVPE 的**完整需求规约**：目标、非目标、系统边界、功能需求、非功能需求、Graph 子系统需求、Vector 子系统需求、性能与许可需求、MVP 范围、验收标准、风险与开放问题。文档是后续 16 份设计文档的需求侧基线，所有 `GVPE-FR-*` / `GVPE-NFR-*` / `GVPE-GPH-*` / `GVPE-VEC-*` / `GVPE-PERF-*` / `GVPE-LIC-*` / `GVPE-ONT-*` / `NG*` / `AC-*` 标识符**均以本文档为权威源**。

## 3. 适用范围

- **适用阶段**：MVP 阶段强制，Phase 1+ 持续生效。
- **适用读者**：所有 GVPE 设计者、模块实现者、集成方、QA 与许可证审计者。
- **ID 权威性**：任何在下游文档中引用的需求 ID，若与本文档冲突，**以本文档为准**；任何新增 ID 必须在本文档更新中登记。
- **不适用**：本文档不规定实现细节、不选定具体 crate 版本、不覆盖许可证矩阵审查方法（见 `GVPE-DOC-16`）。

## 4. 术语定义

| 术语 | 释义 |
|---|---|
| Simulation Space | 仿真空间；Rust 原生数据平面。 |
| Vector Space | 向量空间；推理辅助层，承载物理签名。 |
| Graph Space | 图谱空间；控制 / 知识平面。 |
| 物理知识图谱（Physics Knowledge Graph, PKG） | 物理本体与关系的承载者，详见 `GVPE-DOC-02` / `GVPE-DOC-03`。 |
| 物理签名（Physics Signature） | 多向量表征，详见 `GVPE-DOC-11`。 |
| 物理编译器（Physics Compiler） | Graph / Vector → Runtime 的唯一桥接器。 |
| `PhysicsProfile` | 唯一的 Graph/Vector/Compiler 交付给 Runtime 的数据结构；POD 友好、扁平、无图节点引用、无向量句柄。 |
| `RuntimeDescriptor` | Runtime 侧的等价结构，`PhysicsProfile` 经此结构实际驱动求解。 |
| 物理岛（physics island） | 求解器按连通性/接触关系分组的批处理单位。 |
| 顺序冲量（Sequential Impulse, SI） | 经典实时刚体约束求解范式。 |
| `PhysicsLOD` | 多级物理细节机制；MVP 仅实现 LOD0，描述符槽位须保留。 |
| Fast Mode / Deterministic Mode | 两种求解模式；架构上自 v1 起必须可区分，详见 `GVPE-DOC-05` §5。 |
| `cargo tree` | Cargo 子命令，用于枚举 crate 依赖图；`AC-02` 使用以机械方式证明 `GVPE-NFR-003`。 |

## 5. 项目背景与约束

逆向物理推断（observation → parameters）目前缺乏系统化、可累积的解决方案；现有引擎要么是闭源的"性能黑盒"，要么是缺乏生产级实时核心的研究代码。GVPE 的核心前提是：**首先**自研实时核心，使其本身即具备独立商业价值；再在其上以**可选**形式叠加物理知识图谱与向量空间作为推理辅助，二者最终必须被编译为普通的运行时参数下发到 Runtime——**绝不**能成为 Runtime 的运行时依赖。

本节约束进一步细化：

- GVPE 是一个**可嵌入的库**（embeddable library），不是应用程序。它不拥有窗口、不默认拥有主循环，也不持有持久进程。宿主（游戏引擎、工具、测试套件）通过 Rust API 或 C ABI（`GVPE-DOC-10`）驱动之。
- 三大空间在依赖方向上**单向**：Graph / Vector / AI → Compiler → Runtime，**不得反向**。
- 详细背景参见 `GVPE-DOC-00` §0.1 / §0.2 / §0.3。

## 6. 功能需求 (GVPE-FR-XXX)

| ID | 描述 |
|---|---|
| GVPE-FR-001 | Simulation Space 必须在 Graph Space 与 Vector Space **编译出去**（feature-gated out）的情况下仍可完全运行；且对相同的 `PhysicsProfile` 输入，无论是否启用 Graph/Vector，必须产出**逐位一致**（bit-for-bit-identical）的仿真结果。 |
| GVPE-FR-002 | 求解器至少必须支持：static / dynamic 刚体（sphere / box / plane 等 MVP 原语）、broad phase 剪枝、narrow phase 接触生成、顺序冲量（SI）接触 + 摩擦求解、restitution（弹性恢复）、sleeping、island-based grouping。 |
| GVPE-FR-003 | `PhysicsProfile`（详见 `GVPE-DOC-04`）必须是 Graph / Vector / Compiler 交付给 Runtime 的**唯一**数据结构；其形态必须为扁平、POD 友好的结构（`mass`, `density`, `inertia`, `friction`, `restitution`, `damping`, `stiffness`, `compliance`, `viscosity`, `solver_type`, `solver_iterations`, `collision_profile`, `approximation_level`），**不得**包含图节点引用、向量句柄。 |
| GVPE-FR-004 | 物理知识图谱（PKG）必须实现 `GVPE-DOC-02` §4 规定的**完整顶层本体**作为 schema；即使 MVP 阶段只填充实例的子集（`GVPE-DOC-02` §MVP scope），schema 本身必须完整。 |
| GVPE-FR-005 | 三类图（物理知识图谱 / 运行时约束图 / 执行图，见 `GVPE-DOC-03` §1）必须在实现上**严格分离**——不共享存储、不共享查询表面、不允许一个图的实体被意外提升为另一个图的职责。 |
| GVPE-FR-006 | 物理签名（Physics Signature）必须是**多向量**的（material / motion / deformation / interaction / contact / energy / wave / field / environment / solver 子签名，详见 `GVPE-DOC-11`），**不得**退化为单一未分化的 embedding。 |
| GVPE-FR-007 | `PhysicsLOD` 机制（`GVPE-DOC-02` §19，`GVPE-DOC-04` §4.7）必须在 Runtime 描述符中**预留位置**，即便 MVP 阶段只实现 LOD0（完整仿真）。 |
| GVPE-FR-008 | GVPE 必须支持**经 C ABI** 被 Unity / Unreal / Godot / 自研引擎调用，并支持**批处理数据交换**（详见 `GVPE-DOC-10`）。 |

## 7. 非功能需求 (GVPE-NFR-XXX / GVPE-PERF-XXX / GVPE-LIC-XXX / GVPE-ONT-XXX / GVPE-RT-XXX)

### 7.1 NFR — 跨切面非功能

| ID | 描述 |
|---|---|
| GVPE-NFR-001 | **Determinism（确定性）**：Fast Mode 与 Deterministic Mode 必须在自首版起即在**架构上**区分（`GVPE-DOC-05` §5），即使 Deterministic Mode 在 MVP 阶段未完全实现。 |
| GVPE-NFR-002 | **Memory（内存）**：热路径必须以零分配 / 近零分配（zero / near-zero allocation）作为目标（`GVPE-DOC-08`）。 |
| GVPE-NFR-003 | **Portability（可移植性）**：核心 crate（`gvpe-core`, `gvpe-collision`, `gvpe-dynamics`, `gvpe-constraint`, `gvpe-solver`, `gvpe-island`, `gvpe-scheduler`, `gvpe-runtime`）**不得**依赖 `gvpe-graph`, `gvpe-vector`, `gvpe-compiler`, `gvpe-inference`, `gvpe-3dgs`；依赖方向必须是 Graph / Vector / AI → Compiler → Runtime，**不得反向**（`GVPE-DOC-04` §4.3）。 |
| GVPE-NFR-004 | **License hygiene（许可证卫生）**：任何嵌入的图或向量数据库在被锁定为依赖之前，必须先通过 `GVPE-DOC-16` 的完整许可证审查矩阵。 |

### 7.2 GPH — 图谱空间需求

| ID | 描述 |
|---|---|
| GVPE-GPH-001 | 图节点**只**对满足 `GVPE-DOC-03` §2 节点 / 属性判定规则（高语义、高连通性、来源 / 置信度可追溯）的实体创建；**禁止**将 bulk 数值型 per-frame 状态作为图节点持久化（`GVPE-PROHIBIT-03` 的实际落地形式）。 |
| GVPE-GPH-002 | `GVPE-DOC-02` §25 / §13 中规定的每一种因果 / 能流关系类型必须在图 schema 中**可表达**，且必须支持**条件关系**（conditional / non-unconditional）形态。 |
| GVPE-GPH-003 | 图**不得**被 per-frame 热路径查询；在 `gvpe-solver` / `gvpe-dynamics` / `gvpe-scheduler` 内部出现的任何 Cypher 或其他图查询语言调用都属于**缺陷**，不是风格问题。 |

### 7.3 VEC — 向量空间需求

| ID | 描述 |
|---|---|
| GVPE-VEC-001 | 向量检索必须以 1–30 Hz 或**纯事件触发**的频率运行，**永不允许**每个物理 step 一次。 |
| GVPE-VEC-002 | `ObservedPhysicsSignature` / `SimulatedPhysicsSignature` / `KnownPhysicsSignature` 必须在**类型层面**可区分（type-level distinct），而**不是**仅靠 tag 字段区分；以此防止对不兼容来源的签名发生意外交叉比较。 |

### 7.4 PERF — 性能需求（目标值，详细见 `GVPE-DOC-14`）

| ID | 描述 |
|---|---|
| GVPE-PERF-001 | MVP 范围 rigid-body 场景（数量级：数百个 dynamic body，原语形状）必须在**单中端 CPU 核心预算**下达到 60 Hz，作为基线目标；多线程扩展为 stretch goal，**不**作为 MVP gate。 |
| GVPE-PERF-002 | 热路径上出现的任何类 GC 暂停（GC-like pause）、无界分配或锁竞争都必须视为**性能回归缺陷**，**不**是"可接受取舍"。 |

### 7.5 LIC — 许可证需求

| ID | 描述 |
|---|---|
| GVPE-LIC-001 | 任何图或向量数据库依赖在被选定前，必须通过 `GVPE-DOC-16` §2 中的每一项审查（license、商业使用、OEM、再发行、修改、静态 / 动态链接、SaaS、嵌入式使用）。 |

### 7.6 ONT — 本体相关

| ID | 描述 |
|---|---|
| GVPE-ONT-（集） | 本体相关 ID（`ONT-ISS-*` 等）的权威登记在 `GVPE-DOC-02`；本轮草案未在本文件单独建立 ONT 系列 ID，但 `GVPE-FR-004` / `GVPE-GPH-002` 对本体的完整性与可表达性形成约束。 |

### 7.7 RT — 运行时补充

| ID | 描述 |
|---|---|
| GVPE-RT-（无独立 ID） | 运行时相关具体规约由 `GVPE-DOC-05`（Runtime 设计）、`GVPE-DOC-06`（碰撞设计）、`GVPE-DOC-07`（求解器设计）展开；本需求文档不重复定义。 |

## 8. 业务约束

### 8.1 总纲级禁令（继承自 `GVPE-DOC-00`）

| ID | 描述 |
|---|---|
| GVPE-PROHIBIT-01 | 不得以第三方完整物理引擎作为核心 Runtime。 |
| GVPE-PROHIBIT-02 | 不得对现有物理引擎进行薄包装后以自研名义呈现。 |
| GVPE-PROHIBIT-03 | 不得让任何图数据库执行实时物理求解。 |
| GVPE-PROHIBIT-04 | 不得让任何向量数据库进入 per-frame 热路径。 |
| GVPE-PROHIBIT-05 | 不得让任何 LLM / AI 替代基础数值物理。 |
| GVPE-PROHIBIT-06 | 不得为追求架构优雅而牺牲实时性能。 |

### 8.2 非目标（Non-Goals, v0.1）

| ID | 描述 |
|---|---|
| NG1 | **不**实现 fluid / FEM / 多相求解器（接口预留，不实现）。 |
| NG2 | **不**实现 3DGS 重建 / 推断流水线（`GVPE-DOC-13` 仅作为接口占位）。 |
| NG3 | **不**在 MVP 引入 GPU compute 后端（架构上**不得**阻止其引入——见 `GVPE-DOC-04` §4.6）。 |
| NG4 | **不**在 `GVPE-DOC-16` 完成前锁定任何生产级图 / 向量数据库选型。 |
| NG5 | 求解路径上**不**出现任何 LLM 驱动的参数推断（`GVPE-PROHIBIT-05`）。 |

### 8.3 跨文档硬约束

- **GCN-DOC-01-A**：三大空间的依赖方向**单向**（Graph / Vector / AI → Compiler → Runtime，**不得反向**），由 `GVPE-NFR-003` 强制；下游所有设计必须以 `cargo tree` 机械可验证。
- **GCN-DOC-01-B**：本套件共 16 份设计文档 + 1 份总论 = 17 份文件，构成本轮需求 + 设计基线；编号与目录结构在 `GVPE-DOC-00` §0.6 列出。

## 9. 验收标准

| ID | 描述 |
|---|---|
| AC-01 | 一个包含 N 个刚体的场景，在相同 seed、相同 build、相同机器的两次运行中，**确定性可重复**；且该结果在 Graph / Vector feature 全部编译出去的情况下仍可复现。 |
| AC-02 | `cargo tree -p gvpe-core -p gvpe-collision -p gvpe-dynamics -p gvpe-constraint -p gvpe-solver -p gvpe-island -p gvpe-scheduler -p gvpe-runtime` 的输出中**不包含** `gvpe-graph`, `gvpe-vector`, `gvpe-compiler`, `gvpe-inference`, `gvpe-3dgs` 任何条目（`GVPE-NFR-003` 的机械可验证形式；遵循已归档 PRE 规范"动态枚举而非硬编码"的经验教训——见 `docs/archive/`）。 |
| AC-03 | 从 Graph 经 Compiler 编译出的 `PhysicsProfile` 与手工构造的同形 `PhysicsProfile`，在不接触 Graph 的情况下，产生**逐字节一致**的运行时行为。 |
| AC-04 | 在本基线被接受前，本体评审（`GVPE-DOC-02` §Review）中**不能**残留任何 High 严重度的未解决 `ONT-ISS-*` 缺陷。 |

## 10. 关联文档

- **上游（输入基线）**：
  - `GVPE-DOC-00` `docs/00_foundation/00_vision.md`（总论，定义六条 `GVPE-PROHIBIT-*` 与不变式）
  - `docs/archive/01_PRE_Requirements.md`（已归档，溯源参考）
- **下游（被消费于）**：
  - `GVPE-DOC-02` `docs/00_foundation/02_physics_ontology.md`（物理本体）
  - `GVPE-DOC-03` `docs/01_architecture/03_graph_schema.md`（图谱模式）
  - `GVPE-DOC-04` `docs/01_architecture/04_architecture.md`（架构总览）
  - `GVPE-DOC-05` `docs/01_architecture/05_runtime_design.md`（运行时设计）
  - `GVPE-DOC-06` `docs/02_modules/06_collision_design.md`（碰撞设计）
  - `GVPE-DOC-07` `docs/02_modules/07_solver_design.md`（求解器设计）
  - `GVPE-DOC-08` `docs/01_architecture/08_memory_design.md`（内存设计）
  - `GVPE-DOC-09` `docs/01_architecture/09_parallel_design.md`（并行设计）
  - `GVPE-DOC-10` `docs/02_modules/10_ffi_design.md`（C ABI 设计）
  - `GVPE-DOC-11` `docs/02_modules/11_vector_design.md`（向量空间设计）
  - `GVPE-DOC-12` `docs/02_modules/12_energy_wave_field_design.md`（能量/波/场 设计）
  - `GVPE-DOC-13` `docs/05_future/13_3dgs_future_design.md`（3DGS 未来设计）
  - `GVPE-DOC-14` `docs/03_cross_cutting/14_performance_budget.md`（性能预算）
  - `GVPE-DOC-15` `docs/03_cross_cutting/15_testing_strategy.md`（测试策略）
  - `GVPE-DOC-16` `docs/03_cross_cutting/16_dependency_license.md`（依赖许可证）

## 11. 审批签字

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 编写 | — | | |
| 校对 | | | |
| 审批 | | | |

---

## 12. 正文

> 本节保留原文档结构（§1 ~ §14），所有叙述翻译为中文；技术名词、ID 标识符、代码块保持英文原样。

### 1. Background

逆向物理推断（observation → parameters）目前缺乏系统化、可累积的解决方案；现有引擎要么是闭源的"性能黑盒"，要么是缺乏生产级实时核心的研究代码。GVPE 的核心前提是：**首先**自研实时核心，使其本身即具备独立商业价值；再在其上以**可选**形式叠加物理知识图谱 + 向量空间作为推理辅助，二者最终必须被编译为普通的运行时参数下发到 Runtime——**绝不**能成为 Runtime 的运行时依赖。

### 2. Goals

- **G1**：自研 Rust rigid-body 求解器（broad phase、narrow phase、contact generation、sequential-impulse solve、islands、sleeping），在零第三方物理引擎依赖的前提下，达到 `GVPE-DOC-14` 所述的商用实时性能目标。
- **G2**：构建一个物理知识图谱（PKG），其顶层本体（`GVPE-DOC-02`）必须完整到——`Energy`、`Wave`、`Field`、`Process`、`Law` 等扩展**永不**需要破坏性 schema 迁移。
- **G3**：物理编译器是**唯一**从 Graph / Vector 到 Runtime 的通道；Runtime 永不直接查询 Graph 或 Vector。
- **G4**：多向量的物理签名空间可用于检索，**完全**置于 per-step 热路径之外。
- **G5**：C ABI 表面可被 Unity / Unreal / Godot / 自研引擎以批处理数据交换方式调用。
- **G6**：架构上保留 Observation → Simulation → Comparison 闭环（3DGS 方向）的可扩展性，使其在后续 `GVPE-DOC-02` ~ `GVPE-DOC-04` 之外不再需要重设计即可接入。

### 3. Non-goals (V0.1)

- **NG1**：**不**实现 fluid / FEM / 多相求解器（接口预留，不实现）。
- **NG2**：**不**实现 3DGS 重建 / 推断流水线（`GVPE-DOC-13` 仅作为接口占位）。
- **NG3**：**不**在 MVP 引入 GPU compute 后端（架构上**不得**阻止其引入——见 `GVPE-DOC-04` §4.6）。
- **NG4**：**不**在 `GVPE-DOC-16` 完成前锁定任何生产级图 / 向量数据库选型。
- **NG5**：求解路径上**不**出现任何 LLM 驱动的参数推断（`GVPE-PROHIBIT-05`）。

### 4. System boundary

GVPE 是一个**可嵌入的库**（embeddable library），不是应用程序。它不拥有窗口、不默认拥有主循环，也不持有持久进程。宿主（游戏引擎、工具、测试套件）通过 Rust API 或 C ABI（`GVPE-DOC-10`）驱动之。

### 5. Functional Requirements

见本文档 §6（IPA 模板下的功能需求节）。原文档 `FR-001` ~ `FR-007` 已逐一映射为 `GVPE-FR-001` ~ `GVPE-FR-007`，并补充 `GVPE-FR-008`（C ABI 批处理数据交换）。

### 6. Non-Functional Requirements

见本文档 §7.1（跨切面 NFR）。原文档 `NFR-001` ~ `NFR-004` 已逐一映射为 `GVPE-NFR-001` ~ `GVPE-NFR-004`。

### 7. GPH — Graph-space requirements

见本文档 §7.2。`GVPE-GPH-001` ~ `GVPE-GPH-003` 与原文一致。

### 8. VEC — Vector-space requirements

见本文档 §7.3。`GVPE-VEC-001` ~ `GVPE-VEC-002` 与原文一致。

### 9. PERF — Performance requirements (targets, refined in `14_performance_budget.md`)

见本文档 §7.4。`GVPE-PERF-001` ~ `GVPE-PERF-002` 与原文一致。

### 10. LIC — Licensing requirements

见本文档 §7.5。`GVPE-LIC-001` 与原文一致。

### 11. MVP Scope

- **Simulation Space**：3D 刚体（sphere / box / plane），broad phase，narrow phase，contact manifold，sequential impulse，friction，restitution，sleeping，physics islands，基础多线程，C ABI。
- **Graph Space（schema 完整，实例填充最小）**：`Entity`, `Material`, `Phase`, `Property`, `PhysicalModel`, `Solver`, `PhysicsProfile`, `Simulation`, `Observation`。
- **显式不属于 MVP、但 schema 不得阻断其后续接入**：`Energy`, `Wave`, `Field`, `Process`, `PhysicalLaw`（现已在 `GVPE-DOC-02` 中完整描述，后续填充实例即可）。

### 12. Acceptance Criteria

见本文档 §9。`AC-01` ~ `AC-04` 与原文一致。

### 13. Risks（另见 `GVPE-DOC-15` 中"以测试做缓解"的章节）

- 从零自研完整 rigid-body 求解器即便在 MVP 范围内也属多月量级工作；**范围纪律**（§11）是首要控制手段。
- **本体过度设计**：`GVPE-DOC-02` 故意在任何求解器填充实例前先前置 `Energy` / `Wave` / `Field` / `Process` / `Law` schema，这是为避免后续破坏性迁移所支付的真实成本；若 MVP 时间线滑移，将作为风险被显式重审（见 `GVPE-DOC-15` 中的本体评审纪律）。

### 14. Open Questions

- **OQ-01**：Deterministic Mode 的精确浮点 / 归约序保证尚未在数值层面规定——`GVPE-DOC-05` §5 给出的是**需求**而非**算法**。
- **OQ-02**：若 `GVPE-DOC-16` 后仍有商业现成候选存活，是选定某一图数据库还是自研嵌入式图存储尚未决定；两条路径在许可证审查与存储形态 spike 完成前**均保持开放**。
