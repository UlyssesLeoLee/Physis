# GVPE — Runtime Design（基本設計書）

## 0. 文档元数据

| 字段 | 取值 |
|---|---|
| 文档编号 | GVPE-DOC-05 |
| 文档类型 | 基本設計書 |
| 版本 | v0.2（标准化重写版） |
| 状态 | Draft |
| 适用阶段 | MVP |
| 关联系统 | GVPE / gvpe-runtime, gvpe-dynamics, gvpe-constraint, gvpe-memory, gvpe-scheduler |
| 上游文档（输入基线） | GVPE-DOC-04（04_architecture.md §4.1/§4.3） |
| 下游文档（被消费于） | GVPE-DOC-06（06_collision_design.md），GVPE-DOC-07（07_solver_design.md），GVPE-DOC-08（08_memory_design.md），GVPE-DOC-09（09_parallel_design.md），GVPE-DOC-14（14_performance_budget.md） |
| 编写者 | — |
| 审批者 | — |

## 1. 修订历史

| 版本 | 日期 | 修改者 | 修改内容 |
|---|---|---|---|
| v0.1 | — | — | 初稿（原始版本） |
| v0.2 | 2026-08-19 | 标准化 pass | 转为 IPA 基本設計書 格式；叙事中文化；保留全部技术内容；补充文档元数据、修订历史、术语定义、关联文档、审批签字等标准章节 |

## 2. 文档目的

本基本設計書定义 GVPE 运行时（`gvpe-runtime`）的内部数据布局优先级（SoA / AoSoA / Chunk / SIMD Block）、Hot / Warm / Cold 数据分类、确定性模式枚举、Host 端 API 形态以及单帧 step 的内部阶段分解。文档目的是为热路径上 body / contact / 约束数据的高速缓存友好性、SIMD lane 填充率以及零分配约束提供统一基线。

## 3. 适用范围

本文件适用于 `gvpe-runtime` crate（顶层 API、帧循环宿主）以及 `gvpe-dynamics`、`gvpe-constraint` crate 的热路径数据布局选型。同时为 `gvpe-memory`（参见 GVPE-DOC-08）提供分配策略的需求输入，以及为 `gvpe-scheduler`（参见 GVPE-DOC-09）提供并行可分区的内存布局前提。

## 4. 术语定义

| 术语 | 说明 |
|---|---|
| SoA（Structure of Arrays） | 同一字段的所有元素组织为一个连续数组的数据布局 |
| AoSoA（Array of Structures of Arrays） | SoA 与 AoS 混合的块状布局，便于 SIMD lane 对齐 |
| Chunk | 固定大小的连续数据块，作为调度 / 缓存划分的最小单位 |
| SIMD Block | 按目标 SIMD 位宽对齐的最小数据块 |
| 确定性模式（DeterminismMode） | 决定浮点、SIMD、线程、归约、约束顺序等行为的枚举：`Fast` 或 `Deterministic` |
| `GvpeContext` | 运行时顶层不透明结构，无全局静态可变状态 |
| Hot / Warm / Cold | 数据访问频率的三档分类，决定布局策略 |
| OQ-01 | 待澄清问题 01：跨架构位级一致不在 MVP 承诺范围内 |
| Fast Mode | 平台默认浮点 / SIMD / 线程顺序，平台最优性能 |
| Deterministic Mode | 固定求值顺序、固定 SIMD 宽度与归约、固定约束顺序；仅承诺同架构一致 |

## 5. 前提与约束

1. 上游基线：`04_architecture.md` §4.1 / §4.3 的 crate 列表与依赖方向。
2. 数据布局约束：对 `gvpe-dynamics` / `gvpe-constraint` 中所有热路径 body / contact 数据，优先 SoA → AoSoA → Chunk → SIMD Block 序列，**严格优于** AoS。
3. 零分配约束：参见 GVPE-DOC-08（`08_memory_design.md`），与本文件 §6.1 共同定义。
4. 确定性约束：MVP 实现 `Fast` Mode 完整能力；`Deterministic` Mode 在架构上保留（枚举与求解器顺序抽象存在），但其完整保证**不**是 MVP 验收门禁 —— 该结论已显式登记为 `01_requirements.md` §14 的 OQ-01，不静默延后。
5. 嵌入性约束：`gvpe-runtime` 中不允许任何全局 / 静态可变状态；同进程内多个 `GvpeContext` 实例必须彼此独立。
6. NUMA 显式为未来扩展项，不在 v0.1 范围内 —— 本文件在此处显式声明以避免后续被静默反向设计排除。

## 6. 系统架构 / 模块设计

### 6.1 数据布局优先级

对所有 `gvpe-dynamics` / `gvpe-constraint` 中热路径 body / contact 数据，优先顺序为：

> SoA → AoSoA → Chunk → SIMD Block，**严格优于** AoS。

依据：在本引擎目标 body 数量级（参见 GVPE-DOC-14 性能预算）下，AoS 会同时劣化高速缓存行利用率与 SIMD lane 填充率。

### 6.2 Hot / Warm / Cold 数据分类

| 类别 | 数据示例 | 访问模式 | 布局策略 |
|---|---|---|---|
| Hot | position, velocity, contact impulse | 每子步、每 body 访问 | SoA、按 cache line 对齐、尽量与调度器高频访问数据相邻 |
| Warm | mass, inertia, material profile 引用 | 读多写少，每帧 | SoA，若空间允许可与 Hot 共享 cache line |
| Cold | debug 名称、来源 / 置信度引用、LOD 元数据 | 偶发、面向工具链 | 独立分配，永不与 Hot 交错 |

布局决策需考虑：cache line 大小、对齐、伪共享（特别在调度器并行的物理岛之间，参见 GVPE-DOC-09）、内存带宽、目标 SIMD 位宽。NUMA 显式标注为未来扩展而非 v0.1 关注点 —— 此处保留说明以避免后续被静默反向设计。

## 7. 接口设计

### 7.1 确定性模式枚举（GVPE-NFR-001）

```rust
enum DeterminismMode { Fast, Deterministic }
```

| 关注点 | Fast Mode | Deterministic Mode |
|---|---|---|
| 浮点 | 平台默认，允许使用 FMA / fast-math | 固定求值顺序，禁止 FMA 重排 |
| SIMD | 允许按平台最优宽度向量化 | 固定宽度、固定顺序归约 |
| 线程顺序 | 物理岛求解无序 | 固定物理岛处理顺序 |
| 归约顺序 | 未指定 | 指定且经过测试 |
| 约束顺序 | 插入顺序，并行构建时可能变化 | 规范排序键，稳定 |
| 跨架构 | 不承诺 | 仅承诺同架构一致（不承诺跨平台位级一致，依据 OQ-01） |

- MVP 完整实现 `Fast` Mode。
- `Deterministic` Mode 在架构上保留，但完整保证**不**是 MVP 验收门禁，已显式登记为 OQ-01。

### 7.2 Runtime API 形态（面向 host，未经过 FFI）

```rust
struct GvpeContext { /* opaque, no globals — every instance independent */ }

impl GvpeContext {
    fn new(descriptor: RuntimeDescriptor) -> Self;
    fn step(&mut self, dt: f32);                 // host drives the loop, not GVPE
    fn body_state(&self, handle: BodyHandle) -> BodyState;
    fn set_determinism_mode(&mut self, mode: DeterminismMode);
}
```

- `gvpe-runtime` 中无任何全局 / 静态可变状态。
- 同进程内多个 `GvpeContext` 实例必须彼此独立。

## 8. 数据模型

参见 §6.2 Hot / Warm / Cold 分类与 §7.1 `DeterminismMode` 枚举；具体 POD 字段定义分散在各模块的 detail 设计文件（GVPE-DOC-06 / GVPE-DOC-07 / GVPE-DOC-08）。

## 9. 处理流程

### 9.1 单帧 step 分解（对应执行图，GVPE-DOC-03 §1.C）

`step(dt)` 内部按以下阶段顺序执行：

```
Apply Forces → Broad Phase → Narrow Phase → Contact Generation
  → Island Build → Constraint Solve → Integrate → CCD → Output
```

各阶段的预算分配与跟踪由 `14_performance_budget.md`（GVPE-DOC-14）维护。

## 10. 关联需求

| 需求 ID | 中文描述 | 在本文件中的落地点 |
|---|---|---|
| GVPE-NFR-001 | 引擎应提供至少一种确定性模式（`Deterministic` Mode），允许在同架构下复现仿真结果 | §7.1 确定性模式枚举 |
| GVPE-NFR-002 | 热路径数据布局需高速缓存友好、SIMD 友好，支持零分配 | §6.1 数据布局优先级、§6.2 数据分类 |
| OQ-01（`01_requirements.md` §14） | 跨平台位级一致不在 MVP 承诺范围内 | §7.1 跨架构行 |
| 00_vision.md §0.5 | 自研、无全局静态状态、host 驱动帧循环 | §7.2 Runtime API |

## 11. 关联文档

- 上游：`docs/01_architecture/04_architecture.md`（GVPE-DOC-04）
- 下游：`docs/02_modules/06_collision_design.md`（GVPE-DOC-06），`docs/02_modules/07_solver_design.md`（GVPE-DOC-07），`docs/01_architecture/08_memory_design.md`（GVPE-DOC-08），`docs/01_architecture/09_parallel_design.md`（GVPE-DOC-09），`docs/00_vision/14_performance_budget.md`（GVPE-DOC-14）
- 平行引用：`docs/01_architecture/03_graph_schema.md`（GVPE-DOC-03），`docs/00_vision/00_vision.md`（GVPE-DOC-00）

## 12. 审批签字

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 编写 |  |  |  |
| 校对 |  |  |  |
| 审批 |  |  |  |

---

## 13. 正文

Input baseline: `04_architecture.md` §4.1/§4.3, GVPE-NFR-001/002.

## 5.1 Data layout priority

Preference order: **SoA → AoSoA → Chunk → SIMD Block**, over AoS, for all hot-path body/contact
data in `gvpe-dynamics`/`gvpe-constraint`. Rationale: cache-line utilization and SIMD-lane fill
both degrade under AoS at the body counts this engine targets (§14 performance budget).

## 5.2 Hot / Warm / Cold data classification

| Class | Examples | Access pattern | Layout consequence |
|---|---|---|---|
| Hot | position, velocity, contact impulse | every substep, every body | SoA, cache-line aligned, packed near scheduler-touched data |
| Warm | mass, inertia, material profile refs | read-mostly, per-frame | SoA, may share cache lines with Hot if space allows |
| Cold | debug names, provenance refs, LOD metadata | rare, tooling-facing | separate allocation, never interleaved with Hot |

Layout decisions consider: cache line size, alignment, false sharing (especially across
scheduler-parallel islands, `09_parallel_design.md`), memory bandwidth, target SIMD width. NUMA is
explicitly a future extension, not a v0.1 concern — noted so it isn't silently designed against
later.

## 5.3 Determinism modes (GVPE-NFR-001)

```rust
enum DeterminismMode { Fast, Deterministic }
```

| Concern | Fast Mode | Deterministic Mode |
|---|---|---|
| Floating point | platform default, may use FMA/fast-math | fixed evaluation order, no FMA reordering |
| SIMD | free to vectorize with platform-optimal width | fixed-width, fixed-order reduction |
| Thread ordering | unordered island solve | fixed island processing order |
| Reduction order | unspecified | specified, tested |
| Constraint order | insertion order, may vary with parallel build | canonical sort key, stable |
| Cross-architecture | not guaranteed | same-architecture guarantee only (no cross-platform bit-exactness promise, per OQ-01) |

MVP implements `Fast` fully; `Deterministic` is architecturally reserved (the enum and the
solver-order abstraction exist) but its full guarantee is **not** an MVP acceptance gate — this is
explicitly flagged as OQ-01 in `01_requirements.md` §14, not silently deferred.

## 5.4 Runtime API shape (host-facing, pre-FFI)

```rust
struct GvpeContext { /* opaque, no globals — every instance independent */ }

impl GvpeContext {
    fn new(descriptor: RuntimeDescriptor) -> Self;
    fn step(&mut self, dt: f32);                 // host drives the loop, not GVPE
    fn body_state(&self, handle: BodyHandle) -> BodyState;
    fn set_determinism_mode(&mut self, mode: DeterminismMode);
}
```
No global/static mutable state anywhere in `gvpe-runtime` — multiple `GvpeContext` instances in one
process must be independent (same embeddability discipline the archived PRE work already
established for its own runtime, carried forward here because it's a correct requirement
independent of which project it's attached to).

## 5.5 Frame step breakdown (maps to the Execution Graph, `03_graph_schema.md` §1.C)

`step(dt)` internally runs: Apply Forces → Broad Phase → Narrow Phase → Contact Generation →
Island Build → Constraint Solve → Integrate → CCD → Output. Each stage's budget is tracked in
`14_performance_budget.md`.
