# GVPE — Parallel Design（基本設計書）

## 0. 文档元数据

| 字段 | 取值 |
|---|---|
| 文档编号 | GVPE-DOC-09 |
| 文档类型 | 基本設計書 |
| 版本 | v0.2（标准化重写版） |
| 状态 | Draft |
| 适用阶段 | MVP |
| 关联系统 | GVPE / gvpe-island, gvpe-scheduler, gvpe-solver, gvpe-memory |
| 上游文档（输入基线） | GVPE-DOC-04（04_architecture.md §4.1），GVPE-DOC-07（07_solver_design.md） |
| 下游文档（被消费于） | GVPE-DOC-14（14_performance_budget.md），GVPE-DOC-08（08_memory_design.md 每线程 arena 配合） |
| 编写者 | — |
| 审批者 | — |

## 1. 修订历史

| 版本 | 日期 | 修改者 | 修改内容 |
|---|---|---|---|
| v0.1 | — | — | 初稿（原始版本） |
| v0.2 | 2026-08-19 | 标准化 pass | 转为 IPA 基本設計書 格式；叙事中文化；保留全部技术内容；补充文档元数据、修订历史、术语定义、关联文档、审批签字等标准章节 |

## 2. 文档目的

本基本設計書定义 GVPE 引擎的并行化策略，包括物理岛（Physics Island）的生成与语义、Job DAG（执行图）的具体形态、调度器机制（工作窃取、依赖计数器、每线程 scratch 分配器）以及 MVP 实际交付范围。文档目的是把运行时约束图（GVPE-DOC-03 §1.B）转化为可被调度器消费的物理岛单元，并明确"避免全局互斥锁"的并行设计目标。

## 3. 适用范围

本文件适用于 `gvpe-island`（连通分量 → 物理岛）crate 与 `gvpe-scheduler`（任务 DAG、工作窃取）crate。求解器与内存分配的并行交互参见 GVPE-DOC-07 / GVPE-DOC-08。

## 4. 术语定义

| 术语 | 说明 |
|---|---|
| 物理岛（Physics Island） | 由运行时约束图的连通分量导出的独立求解单元 |
| 运行时约束图（Runtime Constraint Graph） | 每帧重建或增量更新的内存图，承载 body 之间的接触 / 关节关系 |
| 执行图（Execution Graph） | 任务 DAG，定义仿真流水线的阶段顺序（参见 GVPE-DOC-03 §1.C） |
| 任务 DAG | 任务有向无环图，调度器按依赖关系执行 |
| 工作窃取（Work Stealing） | 空闲线程从繁忙线程的队列尾部"偷取"任务的负载均衡策略 |
| 依赖计数器 | 一种调度原语：任务 B 仅在任务 A 的计数器归零后才执行 |
| `FrameScratch` | 每帧 scratch 内存，封装一个 arena（参见 GVPE-DOC-08） |
| 基本多线程（Basic Multithreading） | MVP 范围：跨线程池并行 `SolveIsland[]` |
| `BroadPhase` / `NarrowPhase[]` / `IslandBuild` / `SolveIsland[]` / `Integrate[]` | 执行图的具体阶段，`[]` 标记并行 fan-out 点 |

## 5. 前提与约束

1. 上游基线：本文件以 `04_architecture.md` §4.1（crate 列表 `gvpe-island`、`gvpe-scheduler`）为输入。
2. 物理岛独立性：每个物理岛在帧内独立求解，**无跨岛数据依赖**。
3. 物理岛的多重身份：物理岛同时是 sleeping 单位（全 sleeping 的岛被整体跳过）、并行单位（岛被分配到独立调度 job）、工作分区单位、负载均衡单位。
4. 避免全局互斥锁：热路径任何结构上都不允许出现全局互斥锁。岛级并行的选择**正是因为**岛在帧内彼此独立（无跨岛共享约束行）—— 这是"避免细粒度锁"而不是"把细粒度锁做快"的策略选择。
5. MVP 范围：仅交付"基本多线程"—— 跨简单线程池的并行 `SolveIsland[]`，工作窃取与每线程 scratch 的精细化列为目标形态而非 MVP 门禁。
6. 与内存模块的耦合：每线程的 `FrameScratch` 独立，跨线程不共享 arena（与 GVPE-DOC-08 §7.1 配合）。

## 6. 系统架构 / 模块设计

### 6.1 物理岛（运行时约束图 → 并行单位）

```
Runtime Constraint Graph → Connected Components → Physics Islands
```

- 每个岛独立求解：帧内无跨岛数据依赖。
- 物理岛的多重身份：
  - sleeping 单位：全 sleeping 的岛被整体跳过；
  - 并行单位：岛被分配到独立调度 job；
  - 工作分区单位：每个岛是一个连续工作块；
  - 负载均衡单位：调度器按岛数量 / 大小做分配。

### 6.2 Job DAG（执行图，GVPE-DOC-03 §1.C 落地）

```
BroadPhase → NarrowPhase[] → IslandBuild → SolveIsland[] → Integrate[]
```

- `NarrowPhase[]`、`SolveIsland[]`、`Integrate[]` 是 per-island / per-pair 的 fan-out 点，方括号标记调度器分发到多线程的位置。

### 6.3 调度器机制（候选方案，MVP：简单工作窃取线程池）

研究方向：

- 工作窃取（work stealing）。
- 线程池 + 依赖计数器（任务 B 仅在任务 A 的计数器归零后执行）。
- 每线程 scratch 分配器（与 GVPE-DOC-08 §8.2 配合 —— 每线程的 `FrameScratch` 独立，跨线程不共享 arena）。

**显式目标**：热路径上避免任何全局互斥锁。选择岛级并行正是因为岛在帧内可证明彼此独立（无跨岛共享约束行）—— 因此整体上不需要细粒度锁，而不是尝试把细粒度锁做快。

### 6.4 MVP 实际交付

- 依据 `01_requirements.md` §11 MVP 范围："基本多线程" —— 跨简单线程池的并行 `SolveIsland[]`。
- MVP 验收**不**要求工作窃取的复杂实现。
- 工作窃取与每线程 scratch 的精细化列在本文件中作为目标形态，不是 MVP 门禁。

## 7. 接口设计

`gvpe-island` 与 `gvpe-scheduler` 之间的具体 Rust 接口在实现期确定。本文件约束的是行为契约：

- `gvpe-island` 输出：岛集合，每个岛包含独立的 body 列表与约束行集合。
- `gvpe-scheduler` 消费：以岛为调度单位执行 §6.2 的 Job DAG，保证岛的帧内独立性。
- 每线程独立 `FrameScratch`（参见 GVPE-DOC-08）：调度器负责为每个工作线程提供独立的 scratch 内存视图，跨线程不共享。

## 8. 数据模型

- 物理岛：body 列表 + 约束行集合 + sleeping 状态。
- Job DAG 节点：阶段类型（`BroadPhase` / `NarrowPhase[]` / `IslandBuild` / `SolveIsland[]` / `Integrate[]`）+ 依赖关系。
- 调度器内部：线程池、任务队列、依赖计数器、每线程 `FrameScratch` 句柄。

## 9. 处理流程

### 9.1 物理岛构建与并行求解

```
Runtime Constraint Graph
   → Connected Components (gvpe-island)
   → Physics Islands
   → [per island] SolveIsland[]      # scheduler fan-out
   → [per island] Integrate[]        # scheduler fan-out
   → Output
```

### 9.2 与执行图的对应（GVPE-DOC-03 §1.C）

执行图阶段：

```
Apply Forces → Broad Phase → Narrow Phase → Contact Generation
  → Island Build → Constraint Solve → Integrate → CCD → Output
```

其中 `Constraint Solve` 与 `Integrate` 在调度层面以岛为单位 fan-out（对应 `SolveIsland[]` / `Integrate[]`）。

## 10. 关联需求

| 需求 ID | 中文描述 | 在本文件中的落地点 |
|---|---|---|
| 04_architecture.md §4.1 | crate 列表中 `gvpe-island` / `gvpe-scheduler` 的角色 | §6.1、§6.3 调度机制 |
| 07_solver_design.md | 求解器在物理岛内执行 | §6.1 物理岛独立性 |
| 03_graph_schema.md §1.B | 运行时约束图定义 | §6.1 物理岛来源 |
| 03_graph_schema.md §1.C | 执行图（任务 DAG） | §6.2 Job DAG |
| 08_memory_design.md §7.1 | `FrameScratch` 生命周期 | §6.3 每线程独立 scratch |
| 01_requirements.md §11 | MVP 范围（基本多线程） | §6.4 MVP 实际交付 |
| 05_runtime_design.md §9.1 | 单帧 step 阶段分解 | §9.2 与执行图对应 |

## 11. 关联文档

- 上游：`docs/01_architecture/04_architecture.md`（GVPE-DOC-04），`docs/02_modules/07_solver_design.md`（GVPE-DOC-07），`docs/01_architecture/03_graph_schema.md`（GVPE-DOC-03），`docs/01_architecture/08_memory_design.md`（GVPE-DOC-08）
- 下游：`docs/00_vision/14_performance_budget.md`（GVPE-DOC-14）
- 平行引用：`docs/01_architecture/05_runtime_design.md`（GVPE-DOC-05），`docs/01_requirements.md`（GVPE-DOC-01 中需求章节）

## 12. 审批签字

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 编写 |  |  |  |
| 校对 |  |  |  |
| 审批 |  |  |  |

---

## 13. 正文

Input baseline: `04_architecture.md` §4.1 (`gvpe-island`, `gvpe-scheduler`), `07_solver_design.md`.

## 9.1 Physics Islands (Runtime Constraint Graph → parallel unit)

```
Runtime Constraint Graph → Connected Components → Physics Islands
```
Each island is solved independently — no cross-island data dependency within a frame. Islands are
the unit of: sleeping (a fully-sleeping island is skipped entirely), parallelism (islands run on
separate scheduler jobs), work partition, and load balancing.

## 9.2 Job DAG (Execution Graph, `03_graph_schema.md` §1.C, made concrete)

```
BroadPhase → NarrowPhase[] → IslandBuild → SolveIsland[] → Integrate[]
```
`NarrowPhase[]`, `SolveIsland[]`, `Integrate[]` are per-island/per-pair fan-out points — the `[]`
marks where the scheduler distributes work across threads.

## 9.3 Scheduler mechanism (candidates, MVP: simple work-stealing pool)

Research directions: work stealing, thread pool with dependency counters (job B runs only after
job A's counter hits zero), per-thread scratch allocators (ties to `08_memory_design.md` §8.2 —
each thread's `FrameScratch` is independent, no cross-thread arena sharing).

**Explicit goal**: avoid a global mutex on any hot-path structure. Island-level parallelism is
chosen specifically because islands are provably independent (no shared constraint rows across
islands within a frame) — this avoids needing fine-grained locking altogether, rather than trying
to make fine-grained locking fast.

## 9.4 What MVP actually implements

"Basic Multithreading" per `01_requirements.md` §11 MVP scope — parallel `SolveIsland[]` across a
simple thread pool, no work-stealing sophistication required for MVP acceptance. Work-stealing and
per-thread scratch refinement are listed here as the target shape, not an MVP gate.
