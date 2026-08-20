# GVPE — FFI 设计（C ABI 設計）（基本設計書）

## 0. 文档元数据

| 字段 | 取值 |
|---|---|
| 文档编号 | GVPE-DOC-10 |
| 文档类型 | 基本設計書 |
| 版本 | v0.2（标准化重写版） |
| 状态 | Draft |
| 适用阶段 | MVP |
| 关联系统 | GVPE / `gvpe-ffi` |
| 上游文档（输入基线） | GVPE-DOC-05（`05_runtime_design.md` §5.4）、GVPE-DOC-01（GVPE-FR-005） |
| 下游文档（被消费于） | 各引擎绑定层（Unity / Unreal / Godot / 自研引擎） |
| 编写者 | — |
| 审批者 | — |

## 1. 修订历史

| 版本 | 日期 | 修改者 | 修改内容 |
|---|---|---|---|
| v0.1 | — | [原作者] | 初稿（原始版本） |
| v0.2 | 2026-08-19 | 标准化 pass | 转为 IPA 基本設計書 格式；叙事中文化；保留全部技术内容；补充文档元数据、修订历史、术语定义、关联文档、审批签字等标准章节 |

## 2. 文档目的

本文档规定 GVPE 面向宿主语言（Unity、Unreal、Godot、自研 C++ 引擎等）暴露的唯一对外接口形态：以 C ABI 为唯一边界、以不透明句柄与 POD 结构为唯一数据载体。本设计的目标是：

- 统一所有宿主绑定路径，避免出现“一种引擎一套 FFI 层”的碎片化局面。
- 在 C ABI 边界上以类型级保证（type-level guarantee）阻断 Rust 内部不兼容类型外溢。
- 将批量数据交换作为一等公民，杜绝“宿主侧循环逐体调用”的反模式。

## 3. 适用范围

- 适用：`gvpe-ffi` crate 对外暴露的 `extern "C"` 符号集，以及各宿主引擎在其之上建立的绑定层。
- 不适用：宿主引擎内部（Unity ECS、Unreal 物理后端等）的对象布局；这些由各自的绑定层负责。
- 注：本文复用了归档 PRE 规范（`docs/archive/`）中 Tier-2 设计已经确立的 C-ABI 纪律——该纪律本身与项目无关且正确，因此在此重述而非重新推导。

## 4. 术语定义

| 术语 | 说明 |
|---|---|
| C ABI | C 语言的二进制调用约定，是 GVPE 对宿主引擎暴露的稳定边界 |
| 不透明句柄（opaque handle） | 宿主侧仅持有指针而无法解引用的对象引用形式 |
| POD（Plain Old Data） | 无自定义析构、无继承、无内部指针的纯数据布局 |
| `catch_unwind` | Rust 标准库提供的恐慌捕获原语，确保恐慌不跨 C ABI 边界展开 |
| 宿主绑定层 | Unity / Unreal / Godot / 自研引擎在 C ABI 之上的薄封装 |

## 5. 前提与约束

- 所有宿主绑定必须经过同一套 `gvpe-ffi` C ABI，禁止出现“直通 Rust crate”的绕过路径。
- 任何 `extern "C"` 函数必须包覆 `catch_unwind`，否则 panic 跨边界展开将构成未定义行为。
- 宿主绑定在启动时必须调用 `gvpe_abi_version()` 校验 ABI 版本。
- 满足 `GVPE-FR-005`（唯一 C ABI 表面）。

## 6. 系统架构 / 模块设计

### 6.1 原则：C ABI First

所有宿主绑定（Unity、Unreal、Godot、自研 C++ 引擎）统一走 `gvpe-ffi` 这一套 C ABI 表面，不为不同引擎分别维护一套独立的 FFI 层。

### 6.2 表面形态（Surface Shape）

```c
typedef struct GvpeContext GvpeContext;   /* opaque handle */

typedef struct { float x, y, z; } GvpeVec3;
typedef struct { float x, y, z, w; } GvpeQuat;
typedef struct { GvpeVec3 position; GvpeQuat rotation; GvpeVec3 linear_vel; GvpeVec3 angular_vel; } GvpeBodyState;

uint32_t gvpe_abi_version(void);
GvpeContext* gvpe_context_create(const GvpeRuntimeDescriptor* desc);
void gvpe_context_destroy(GvpeContext* ctx);
void gvpe_step(GvpeContext* ctx, float dt);

/* Batch, not per-body: one call moves N bodies' state */
void gvpe_get_body_states(GvpeContext* ctx, const uint32_t* handles, size_t count, GvpeBodyState* out);
```

### 6.3 绑定规则

- 跨边界仅允许不透明句柄与 POD 结构，禁止 Rust 泛型、trait object、带负载的 enum、`Result`、`String`、`Vec` 直接外溢。
- 每个 `extern "C"` 函数必须在函数体上包覆 `catch_unwind`，panic 绝不可跨边界展开（否则为未定义行为）。
- 所有批量数据交换（如 `§6.2` 中的 `gvpe_get_body_states`）必须采用批量接口——宿主侧不允许在循环中逐体调用 FFI；那种模式会彻底抵消低开销 ABI 的设计意义。
- `gvpe_abi_version()` 必须在每个宿主绑定启动时被检查。

## 7. 接口设计

### 7.1 各引擎绑定层（薄层，生成或手写于 §6.2 之上）

| 引擎 | 绑定语言 | 路径 | 备注 |
|---|---|---|---|
| Unity | C#（P/Invoke） | 经 C ABI | 必经 `gvpe-ffi` |
| Unreal | C++（直接链接） | 经 C ABI | 必经 `gvpe-ffi` |
| Godot | GDExtension（Rust-native） | 可选直通 `gvpe-core` | 若 Godot 的 Rust 绑定允许直接链接 `gvpe-core`，则可绕过 C ABI 边界；该集成路径不触及 C 边界，因此不受 §6.3 的 POD-only 规则约束 |
| 自研 C++ 引擎 | C++ | 经 C ABI | 必经 `gvpe-ffi` |

## 8. 数据模型

跨边界可见的数据结构仅限以下 POD 形态：

| 类型 | 字段 | 用途 |
|---|---|---|
| `GvpeVec3` | `x, y, z: float` | 三维向量 |
| `GvpeQuat` | `x, y, z, w: float` | 四元数 |
| `GvpeBodyState` | `position: GvpeVec3`、`rotation: GvpeQuat`、`linear_vel: GvpeVec3`、`angular_vel: GvpeVec3` | 单体刚体状态 |
| `GvpeContext` | 不透明 | 运行时上下文句柄 |

注：`GvpeRuntimeDescriptor` 等更复杂的描述符在 MVP 中以 POD-only 形态出现；其内部字段随设计推进在本节追加，不在此处预先展开。

## 9. 处理流程

1. 宿主进程启动时调用 `gvpe_abi_version()`，与本地期望版本对比。
2. 宿主调用 `gvpe_context_create(const GvpeRuntimeDescriptor*)` 取得 `GvpeContext*`。
3. 宿主按帧调用 `gvpe_step(ctx, dt)` 推进仿真。
4. 宿主按需调用 `gvpe_get_body_states(ctx, handles, count, out)` 批量读取状态。
5. 宿主关闭时调用 `gvpe_context_destroy(ctx)`。

每一步的失败处理（C ABI 错误码、宿主侧回退策略）由各绑定层在自身模块文档中规定，不在本文档展开。

## 10. 关联需求

| 需求编号 | 描述 |
|---|---|
| GVPE-FR-005 | 唯一 C ABI 表面；所有宿主绑定必须经由 `gvpe-ffi` |

## 11. 关联文档

- `docs/05_runtime_design.md`（GVPE-DOC-05）§5.4：运行时描述符的形状定义
- `docs/01_requirements.md`（GVPE-DOC-01）FR-005：唯一 ABI 表面约束
- `docs/15_testing_strategy.md`（GVPE-DOC-15）：绑定层端到端验证策略

## 12. 审批签字

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 编写 |  |  |  |
| 校对 |  |  |  |
| 审批 |  |  |  |

---

## 13. 正文

> 原始输入基线：`05_runtime_design.md` §5.4、GVPE-FR-005。注：本文复用了归档 PRE 规范（`docs/archive/`）中 Tier-2 设计已经确立的 C-ABI 纪律——该纪律本身与项目无关且正确，因此在此重述而非重新推导。

### 10.1 原则：C ABI First

所有宿主绑定（Unity、Unreal、Godot、自研 C++ 引擎）统一走 `gvpe-ffi` 这一套 C ABI 表面，不为不同引擎分别维护一套独立的 FFI 层。

### 10.2 表面形态

```c
typedef struct GvpeContext GvpeContext;   /* opaque handle */

typedef struct { float x, y, z; } GvpeVec3;
typedef struct { float x, y, z, w; } GvpeQuat;
typedef struct { GvpeVec3 position; GvpeQuat rotation; GvpeVec3 linear_vel; GvpeVec3 angular_vel; } GvpeBodyState;

uint32_t gvpe_abi_version(void);
GvpeContext* gvpe_context_create(const GvpeRuntimeDescriptor* desc);
void gvpe_context_destroy(GvpeContext* ctx);
void gvpe_step(GvpeContext* ctx, float dt);

/* Batch, not per-body: one call moves N bodies' state */
void gvpe_get_body_states(GvpeContext* ctx, const uint32_t* handles, size_t count, GvpeBodyState* out);
```

### 10.3 绑定规则（与任何 C ABI 同类约束的重述，专门针对本引擎）

- 跨边界仅允许不透明句柄与 POD 结构——禁止 Rust 泛型、trait object、带负载的 enum、`Result`、`String`、`Vec` 直接外溢。
- 每个 `extern "C"` 函数必须在函数体上包覆 `catch_unwind`；panic 绝不可跨边界展开（否则为未定义行为）。
- 所有批量数据交换（`§10.2` 中的 `gvpe_get_body_states`）必须采用批量接口——宿主侧不允许在循环中逐体调用 FFI；那种模式会彻底抵消低开销 ABI 的设计意义。
- `gvpe_abi_version()` 必须在每个宿主绑定启动时被检查。

### 10.4 各引擎绑定层（薄层，生成或手写于 §10.2 之上）

Unity（C#，P/Invoke）、Unreal（C++，直接链接）、Godot（GDExtension，Rust-native——若 Godot 的 Rust 绑定允许直接链接 `gvpe-core`，则可绕过 C ABI 边界；在该具体集成路径下，不触及 C 边界，因此不受 §10.3 的 POD-only 规则约束）。
