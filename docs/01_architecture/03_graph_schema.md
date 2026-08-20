# GVPE — Graph Schema（基本設計書）

## 0. 文档元数据

| 字段 | 取值 |
|---|---|
| 文档编号 | GVPE-DOC-03 |
| 文档类型 | 基本設計書 |
| 版本 | v0.2（标准化重写版） |
| 状态 | Draft |
| 适用阶段 | MVP |
| 关联系统 | GVPE / gvpe-graph, gvpe-compiler |
| 上游文档（输入基线） | GVPE-DOC-01（00_vision.md），GVPE-DOC-02（02_physics_ontology.md），GVPE-DOC-16（16_dependency_license.md，待定） |
| 下游文档（被消费于） | GVPE-DOC-04（04_architecture.md），GVPE-DOC-09（09_parallel_design.md），GVPE-DOC-13（13_3dgs_future_design.md） |
| 编写者 | — |
| 审批者 | — |

## 1. 修订历史

| 版本 | 日期 | 修改者 | 修改内容 |
|---|---|---|---|
| v0.1 | — | — | 初稿（原始版本） |
| v0.2 | 2026-08-19 | 标准化 pass | 转为 IPA 基本設計書 格式；叙事中文化；保留全部技术内容；补充文档元数据、修订历史、术语定义、关联文档、审批签字等标准章节 |

## 2. 文档目的

本基本設計書定义 GVPE 引擎在图（Graph）层面所采用的三类图（物理知识图谱、运行时约束图、执行图）的存储形态、节点 / 属性 / 运行时数据分类规则，以及"图 → 运行时"必须经过编译、禁止运行时实时查询的强制约束。文档目的是把 `02_physics_ontology.md` 所声明的本体论在存储和查询层落地为可执行的规则，并落实 `00_vision.md` 关于"三类图禁止混为一谈"的禁令。

## 3. 适用范围

本文件适用于：

- `gvpe-graph` crate（物理知识图谱的存储与查询层）；
- `gvpe-compiler` crate（将图与向量空间结果编译为 `RuntimeDescriptor` 的边界模块）；
- `gvpe-island`、`gvpe-constraint` crate（运行时约束图的内存表示）；
- `gvpe-scheduler` crate（执行图的任务 DAG 编排）。

不适用于仿真空间内其他模块的纯数值数据布局（参见 GVPE-DOC-05 `05_runtime_design.md`）、碰撞与求解算法的内部实现（参见 GVPE-DOC-06、GVPE-DOC-07）。

## 4. 术语定义

| 术语 | 说明 |
|---|---|
| 物理知识图谱（Physics Knowledge Graph, PKG） | 持久化、语义层级最高的图，承载实体、材料、相位、状态、属性、过程、相互作用、场、能量、波等本体的关系结构 |
| 运行时约束图（Runtime Constraint Graph） | 每帧重建或增量更新的纯内存图，描述刚体与刚体之间的接触 / 关节约束关系，是物理岛划分的输入 |
| 执行图（Execution Graph） | 任务 DAG 形式的图，描述仿真流水线各阶段（施加力 → 粗筛 → 精筛 → 接触生成 → 物理岛构建 → 约束求解 → 积分 → CCD → 输出）的执行顺序 |
| 物理编译器（Physics Compiler） | `gvpe-compiler` crate，将图查询结果与向量空间嵌入编译为纯 POD 的 `RuntimeDescriptor` |
| 节点（Node） | 图中具备高语义、高连接度、具备来源 / 置信度 / 历史等溯源信息的数据 |
| 属性 / 状态（Property / State） | 纯数值、低语义、高频变化的数据，应进入属性 / 状态存储而非永久图节点 |
| 运行时状态（Runtime State） | 仿真输出，不入图 |

## 5. 前提与约束

1. 上游基线：本文件以 `02_physics_ontology.md` 的本体论为输入，并把本体论中"Graph Node vs Runtime State"（`02_physics_ontology.md` §26 Ontology Review 第 11 条规则）落地为可执行规则。该规则从"被声明"到"被代码强制"的差距由 **ONT-ISS-001** 跟踪。
2. 强制禁令：三类图（PKG / 运行时约束图 / 执行图）**不得合并、不得跨图查询**（`00_vision.md` 已明文禁止）。
3. 持久层选型待定：物理知识图谱的存储后端（成熟图数据库或自研嵌入式存储）由 `16_dependency_license.md` 决定。本文件不绑定具体存储后端。
4. 运行时不得向图数据库发出任何实时查询。所有图查询仅在离线 / 编译器阶段进行（GVPE-GPH-003）。

## 6. 系统架构 / 模块设计

### 6.1 三类图总览

#### A. 物理知识图谱（Physics Knowledge Graph）—— 持久化、语义

```
Entity
 ├─ HAS_MATERIAL → Material     ├─ INTERACTS_VIA → Interaction
 ├─ HAS_PHASE → Phase           ├─ EXISTS_IN → Field
 ├─ HAS_STATE → State           ├─ CARRIES → Energy
 ├─ HAS_PROPERTY → Property     ├─ GENERATES → Wave
 ├─ PARTICIPATES_IN → Process   └─ MODELED_BY → PhysicalModel
```

- 后端存储：持久化图数据库（或自研嵌入式存储，具体选型由 `16_dependency_license.md` 决定）。
- 查询接口：所选存储后端提供的查询能力（类 Cypher 接口或自研查询接口）。
- 用途范围：**仅在离线 / 编译器阶段使用**（GVPE-GPH-003），不允许从仿真热路径访问。

#### B. 运行时约束图（Runtime Constraint Graph）—— 纯内存、每帧构建

```
Body A ─ Contact ─ Body B
   │                 │
 Joint             Contact
   │                 │
Body C ─────────── Body D
```

- 用途：连通分量分析 → 物理岛划分 → 约束分区 → 并行求解（参见 `09_parallel_design.md` §2）。
- 归属模块：完全位于 `gvpe-island` / `gvpe-constraint` crate 内。
- 生命周期：每帧重建或增量更新，不持久化。
- 类型隔离：与物理知识图谱**不共享任何节点类型**。此处 `Body` 句柄是运行时索引，不是 `Entity` 节点的引用。

#### C. 执行图（Execution Graph）—— 任务 DAG，不承载任何物理语义

```
Apply Forces → Broad Phase → Narrow Phase → Contact Generation → Island Build
  → Constraint Solve → Integrate → CCD → Output
```

- 归属模块：`gvpe-scheduler`（任务图编排），对应 `09_parallel_design.md` §3。
- 关键性质：执行图不包含任何本体论内容。将其与 A 或 B 混淆是 §6 / §7 节以下规则专门防范的具体错误。

### 6.2 节点 / 属性 / 运行时数据的归类规则

| 数据形态 | 归类位置 | 判定规则 |
|---|---|---|
| 高语义、高连接度，具备来源 / 置信度 / 历史等溯源信息 | **节点**（物理知识图谱） | 例如：来自一次 `Experiment` 测得的 `YoungModulus` |
| 纯数值、低语义、高频变化 | **属性 / 状态存储**，不入永久节点 | 例如：`position.x = 1.284` |
| 大规模逐帧仿真输出 | **运行时状态**（永不进入图） | 参见 §7 |

该规则是 `02_physics_ontology.md` §26 Ontology Review 第 11 条规则（Graph Node vs Runtime State）的强制执行机制。

## 7. 接口设计

### 7.1 "图 → 运行时"必须编译，禁止实时查询

```
FORBIDDEN:  Runtime → Cypher Query (or any live graph query at simulation time)
REQUIRED:   Graph → Physics Compiler → Compact Runtime Descriptor
```

- 运行时仅消费：`Handle, ID, Index, Numeric Data, BitFlags, Aligned Buffer`。
- 任何图节点类型、任何查询结果集，都**不得**直接或间接跨越进入 `gvpe-runtime` / `gvpe-solver`。
- 编译器接口的精确定义参见 `04_architecture.md` §4.4。

### 7.2 图不是时序仿真数据库

每帧数百万次 `State` 写入绝不能落入图数据库。需要明确区分以下数据形态：

| 数据形态 | 存储位置 | 是否入图 |
|---|---|---|
| 运行时状态（Runtime State） | 仅内存 | 否 |
| 仿真快照（Simulation Snapshot） | 周期性的二进制快照 | 否 |
| 关键帧状态（Keyframe State） | 稀疏、语义上有意义的状态 | 否 |
| 语义状态（Semantic State） | 例如"刚体已停止" | 是（符合条件即可入图） |
| 观测状态（Observation State） | 与 `Observation` 节点绑定 | 是 |

高频数据应进入：二进制快照 / 时序存储 / 仿真缓存，**永不进入图数据库**。图存储的是这些数据的**索引**或**摘要**，而不是数据本身。

## 8. 数据模型

### 8.1 节点示例（仅展示意图，完整 schema 以代码为准）

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

边沿用 `02_physics_ontology.md` §22 的关系词表，可附带可选的条件限定符（例如 `{relation: DECREASES, condition: "temperature > threshold"}`）。

## 9. 处理流程

### 9.1 图查询模式（驱动 GVPE-DOC-16 中数据库选型评估标准）

- **直接查找**：`Entity → Property`（按属性 kind 查找），由编译器在离线阶段使用。
- **有界遍历**：因果链重建（`02_physics_ontology.md` §25），遍历深度必须设置显式上限 hop 数。该上限策略承袭自归档 PRE 本体工作（`docs/archive/`）中已建立的图构建特性深度限制纪律；尽管本文件所涉及的存储技术选型尚未确定，但该经验教训向前继承不变。
- **溯源遍历**：`Result ← VALIDATED_BY ← Simulation ← TESTED_BY ← Hypothesis ← SUPPORTS ← Observation`。

### 9.2 Schema 演化纪律

- 任何 schema 变更必须在 PR / 提交中明确说明其触及 `02_physics_ontology.md` 中哪几类 Ontology Review（共 11 类）。
- 一旦变更需要重新填充已提交的 MVP 实例数据（参见 `02_physics_ontology.md` §26），即视为破坏性迁移，必须显式标注，不得静默合入。

## 10. 关联需求

| 需求 ID | 中文描述 | 在本文件中的落地点 |
|---|---|---|
| GVPE-GPH-003 | 物理知识图谱不得在仿真热路径被直接查询 | §7.1 强制接口约束 |
| ONT-ISS-001 | Ontology Review 第 11 条规则（Graph Node vs Runtime State）从"声明"到"代码强制"之间的差距 | §6.2 归类规则表 |
| 00_vision.md §0.3 | 三类图（PKG / 运行时约束图 / 执行图）禁止混为一谈 | §6.1 全文 |
| 02_physics_ontology.md §9 | 图节点与运行时约束的绑定规则（运行时约束不是图节点） | §6.1.B、§7.1 |
| 02_physics_ontology.md §14/§15 | 物理定律 → 物理模型 → 求解器的可追溯关系 | §6.1.A 关系结构 |
| 02_physics_ontology.md §22 | 边沿关系词表 | §8.1 边沿说明 |
| 02_physics_ontology.md §25 | 因果链重建（有界遍历） | §9.1 有界遍历 |
| 02_physics_ontology.md §26 | Ontology Review 11 类规则 | §6.2、§9.2 |

## 11. 关联文档

- 上游：`docs/00_vision.md`（GVPE-DOC-01），`docs/01_architecture/02_physics_ontology.md`（GVPE-DOC-02），`docs/00_vision/16_dependency_license.md`（GVPE-DOC-16，待定）
- 下游：`docs/01_architecture/04_architecture.md`（GVPE-DOC-04），`docs/01_architecture/09_parallel_design.md`（GVPE-DOC-09），`docs/00_vision/13_3dgs_future_design.md`（GVPE-DOC-13）
- 平行引用：`docs/01_architecture/05_runtime_design.md`（GVPE-DOC-05），`docs/02_modules/06_collision_design.md`（GVPE-DOC-06），`docs/02_modules/07_solver_design.md`（GVPE-DOC-07）

## 12. 审批签字

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 编写 |  |  |  |
| 校对 |  |  |  |
| 审批 |  |  |  |

---

## 13. 正文

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
