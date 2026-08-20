# GVPE — Memory Design（基本設計書）

## 0. 文档元数据

| 字段 | 取值 |
|---|---|
| 文档编号 | GVPE-DOC-08 |
| 文档类型 | 基本設計書 |
| 版本 | v0.2（标准化重写版） |
| 状态 | Draft |
| 适用阶段 | MVP |
| 关联系统 | GVPE / gvpe-memory, gvpe-collision, gvpe-constraint, gvpe-solver, gvpe-runtime |
| 上游文档（输入基线） | GVPE-DOC-05（05_runtime_design.md §5.2） |
| 下游文档（被消费于） | GVPE-DOC-04（04_architecture.md §4.6 GPU 约束），GVPE-DOC-09（09_parallel_design.md §9.3 每线程独立 arena） |
| 编写者 | — |
| 审批者 | — |

## 1. 修订历史

| 版本 | 日期 | 修改者 | 修改内容 |
|---|---|---|---|
| v0.1 | — | — | 初稿（原始版本） |
| v0.2 | 2026-08-19 | 标准化 pass | 转为 IPA 基本設計書 格式；叙事中文化；保留全部技术内容；补充文档元数据、修订历史、术语定义、关联文档、审批签字等标准章节 |

## 2. 文档目的

本基本設計書定义 GVPE 在热路径上的内存分配策略，包括零分配约束、arena / pool / slab / frame allocator / 预分配的适用场景、热路径 buffer 的形态约束以及 scratch buffer 的生命周期管理。文档目的是为仿真热路径提供高速缓存友好、SIMD 友好、无堆抖动的内存基线，并保证未来 GPU 集成（NG3）无需重设计。

## 3. 适用范围

本文件适用于 `gvpe-memory` crate（arena / pool / slab / frame allocator），并对 `gvpe-collision` / `gvpe-constraint` / `gvpe-solver` 的每帧热路径施加分配纪律约束。本文件同时为 `09_parallel_design.md`（GVPE-DOC-09）每线程独立 `FrameScratch` 设计提供依据。

## 4. 术语定义

| 术语 | 说明 |
|---|---|
| 热路径（Hot Path） | 每帧每子步必经的代码路径，其分配行为直接决定性能 |
| Arena | 区域分配器，批量分配 + 批量重置，不逐对象释放 |
| Pool | 固定大小对象的可回收分配器 |
| Slab | 跨帧稳定索引的对象分配器 |
| Frame Allocator | 帧级短期临时分配器 |
| 预分配（Preallocation） | 在场景初始化时按 `RuntimeDescriptor` 给出最坏容量 |
| `FrameScratch` | 每帧的 scratch 内存封装，承载一个 arena |
| `begin_frame` | 帧开始时 O(1) 重置 arena 的入口 |
| NG3 | 全局禁令 03：架构必须不为将来 GPU 集成制造障碍 |
| `Vec<Box<dyn Trait>>` | 指针追踪型集合，热路径禁用 |

## 5. 前提与约束

1. 上游基线：本文件以 `05_runtime_design.md` §5.2（Hot / Warm / Cold 分类）为输入。
2. 零 / 近零分配约束：每次 `step(dt)` 调用在稳态（预热后）下应保持零 / 近零堆分配。
3. 缺陷定义：除非为摊销一次的扩容（例如在首个超量帧上扩容持久 buffer，且此后永不缩容 / 不再每帧重分配），否则 `gvpe-collision` / `gvpe-constraint` / `gvpe-solver` 的每帧路径上发生的任何分配都视为缺陷。
4. buffer 形态约束：热路径 buffer 必须是扁平、连续、可对齐的数组（SoA 参见 GVPE-DOC-05 §6.1）—— 绝不允许 `Vec<Box<dyn Trait>>` 或任何在热路径上指针追踪的结构。
5. 未来 GPU 兼容性：本文件 §7.2 是 `04_architecture.md` §4.6 "未来 GPU 无需重设计" 主张成立的前提。

## 6. 系统架构 / 模块设计

### 6.1 热路径分配策略总览

| 策略 | 使用场景 |
|---|---|
| Arena | 每帧 scratch（接触对、流形）—— 帧末重置而非释放 |
| Pool | 固定大小可回收对象（例如 `ConstraintRow` 槽位） |
| Slab | body / entity 存储，跨帧保持稳定索引 |
| Frame Allocator | 每 step 短期临时对象 |
| 预分配（Preallocation） | 在场景初始化时按 `RuntimeDescriptor` 中 body 数量建立最坏容量 buffer |

### 6.2 分配策略 → 子系统对应（决策指引）

- 每帧短期对象（接触对、流形、临时 Jacobian）→ Arena（`FrameScratch`）。
- 固定大小可回收对象（`ConstraintRow` 槽位）→ Pool。
- body / entity 主存储（跨帧稳定索引）→ Slab。
- 预分配（场景初始化时按 `RuntimeDescriptor` 容量）→ Preallocation。

## 7. 接口设计

### 7.1 Scratch buffer 生命周期

```rust
struct FrameScratch { arena: Arena }
impl FrameScratch {
    fn begin_frame(&mut self) { self.arena.reset(); }   // O(1), no dealloc
}
```

- `begin_frame` 在每帧 `step(dt)` 内部、Broad Phase 之前调用一次。
- 从 `arena` 分配的对象**绝不**跨越 `end_frame` 存活。
- Rust 借用检查器通过 arena 分配引用的生命周期作用域到帧结束来强制该约束。

### 7.2 Buffer 形态（喂入 §4.6 未来 GPU 约束）

热路径 buffer 全部为扁平、连续、可对齐数组（SoA，依据 GVPE-DOC-05 §6.1）—— 不存在 `Vec<Box<dyn Trait>>` 或任何指针追踪结构。这正是 `04_architecture.md` §4.6 "未来 GPU 无需重设计" 主张成立的根本原因。

## 8. 数据模型

- `FrameScratch`：封装一个 `Arena` 的每帧 scratch 结构。
- `Arena`：区域分配器，O(1) 重置，不进行逐对象释放。
- 各类策略（Arena / Pool / Slab / Frame Allocator / Preallocation）的具体实现由 `gvpe-memory` crate 提供，对外仅暴露稳定 API。

## 9. 处理流程

```
step(dt):
    begin_frame()                  # O(1) arena reset
    Apply Forces / Broad Phase / Narrow Phase
        / Contact Generation / Island Build
        / Constraint Solve / Integrate / CCD / Output
        (all allocations on arena / pool / slab / prealloc)
    end_frame()                    # logical end; no dealloc
```

## 10. 关联需求

| 需求 ID | 中文描述 | 在本文件中的落地点 |
|---|---|---|
| GVPE-NFR-002 | 热路径数据布局需高速缓存友好、SIMD 友好，支持零分配 | §5.2 零分配约束、§7.2 buffer 形态 |
| NG3 | 架构必须不为将来 GPU 集成制造障碍 | §7.2 buffer 形态 |
| 05_runtime_design.md §5.2 | Hot / Warm / Cold 数据分类 | §5 前提；§6.1 策略选择 |
| 04_architecture.md §4.6 | 未来 GPU 无需重设计 | §7.2 形态约束 |

## 11. 关联文档

- 上游：`docs/01_architecture/05_runtime_design.md`（GVPE-DOC-05），`docs/01_architecture/04_architecture.md`（GVPE-DOC-04）
- 下游：`docs/01_architecture/09_parallel_design.md`（GVPE-DOC-09，§9.3 每线程独立 arena）

## 12. 审批签字

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 编写 |  |  |  |
| 校对 |  |  |  |
| 审批 |  |  |  |

---

## 13. 正文

Input baseline: `05_runtime_design.md` §5.2, GVPE-NFR-002.

## 8.1 Hot path allocation policy

Zero/near-zero heap allocation per `step(dt)` call, steady-state (post-warmup). Any allocation
inside `gvpe-collision`/`gvpe-constraint`/`gvpe-solver`'s per-frame path is a defect unless it's
amortized-once (e.g. growing a persistent buffer on first oversized frame, never shrinking/
reallocating every frame after).

## 8.2 Allocation strategies (candidates, applied per subsystem)

| Strategy | Used by |
|---|---|
| Arena | per-frame scratch (contact pairs, manifolds) — reset, not freed, each frame |
| Pool | fixed-size recyclable objects (e.g. `ConstraintRow` slots) |
| Slab | body/entity storage with stable indices across frames |
| Frame Allocator | short-lived per-step temporaries |
| Preallocation | worst-case-sized buffers established at scene setup from `RuntimeDescriptor` body counts |

## 8.3 Buffer shape (feeds §4.6's future-GPU constraint)

All hot-path buffers are flat, contiguous, alignable arrays (SoA per `05_runtime_design.md` §5.1) —
never a `Vec<Box<dyn Trait>>` or any pointer-chasing structure on the hot path. This is what keeps
`04_architecture.md` §4.6's "no GPU redesign needed" claim true.

## 8.4 Scratch buffer lifecycle

```rust
struct FrameScratch { arena: Arena }
impl FrameScratch {
    fn begin_frame(&mut self) { self.arena.reset(); }   // O(1), no dealloc
}
```
`begin_frame` is called once per `step(dt)`, before Broad Phase. Nothing allocated from `arena`
survives past `end_frame` — the borrow checker enforces this by scoping arena-allocated references
to the frame's lifetime.
