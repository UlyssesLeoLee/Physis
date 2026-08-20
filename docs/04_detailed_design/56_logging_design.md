# 日志与可观测性设计（Logging & Observability Design）

> **用途**：GVPE 的日志、追踪、性能分析、崩溃诊断的统一设计。
> **对应工作流步骤**：51 ログ設計 → `28_workflow.md` §10.4 步 51。
> **关联**：`GVPE-DOC-17` §10（FFI 边界）；`GVPE-DOC-26` §18.9（`tracing` 评估）；`GVPE-DOC-27` §9.4（P 类性能 QA）。

## 0. 元数据

| 字段 | 取值 |
|---|---|
| 文档编号 | GVPE-DOC-56 |
| 文档类型 | 詳細設計書 |
| 版本 | v0.1 |
| 状态 | Draft |
| 适用阶段 | MVP |
| 上游文档 | `GVPE-DOC-17`, `GVPE-DOC-26` §18.9 |
| 下游文档 | 实施期 PR / `38_code_review_checklist.md` |

## 1. 总体策略

### 1.1 三层可观测性

| 层 | 用途 | 工具 | 启用条件 |
|---|---|---|---|
| **L1 错误** | 异常路径、panic、bug | `Result` + 错误码 | 总是启用 |
| **L2 事件** | 关键节点（scene 创建 / step 失败 / 配置变化） | `tracing` `event!` | feature-gated |
| **L3 性能** | hot path 性能 span、计数器 | `tracing` `span!` + 自定义 counter | feature-gated，默认 off |

### 1.2 核心原则

- **热路径零分配**：热路径**不**写日志（即使 `Level::Trace` off）；写日志本身会触发格式化 → 分配；
- **热路径零开销**：L2/L3 全部 feature-gated；feature off 时编译期消除；
- **错误必捕获**：核心 crate 不允许 `panic!`（视为 bug）；FFI 边界 `catch_unwind` 兜底；
- **集成方可控**：通过 `gvpe_runtime_set_log_level()` 让集成方在运行时调整日志级别；
- **不**输出到 `stdout` / `stderr`（库产品）：让集成方选择 sink（`tracing-subscriber`）。

## 2. 错误层（L1）

### 2.1 错误类型系统

详见 `55_error_code_catalog.md`：
- crate 内部：`Result<T, E>` + `thiserror` 派生；
- crate 边界：转换为 `gvpe-core` 统一错误；
- FFI 边界：转换为 `u32` 错误码。

### 2.2 panic 政策

- 核心 crate（`gvpe-*` 求解路径）：**panic = bug**；
- 测试代码：允许 `unwrap` / `expect`；
- 冷路径（场景加载、配置解析）：允许 `panic`（如 shape 非法）；
- FFI 边界：所有 `extern "C"` 函数用 `catch_unwind`。

### 2.3 错误诊断信息

- 错误类型字段应包含可定位信息（`field`, `index`, `value`）；
- 错误实现 `Display` + `source()`（`thiserror` 自动 derive）；
- 集成方可选调 `gvpe_get_last_error` 获取详细。

## 3. 事件层（L2）

### 3.1 启用方式

```toml
[features]
default = []
tracing-event = ["dep:tracing"]      # 启用 L2
tracing-perf = ["dep:tracing", "tracing/attributes"]  # 启用 L3
```

```rust
#[cfg(feature = "tracing-event")]
use tracing::{event, Level};

#[cfg(not(feature = "tracing-event"))]
macro_rules! event_disabled { ($($_:tt)*) => {} }
```

### 3.2 事件类型

| 事件 | Level | 字段 | 何时触发 |
|---|---|---|---|
| `runtime_created` | INFO | `thread_pool_size`, `determinism_mode` | 每次 `gvpe_create_runtime` |
| `runtime_destroyed` | INFO | `body_count`, `step_count` | 每次 `gvpe_destroy_runtime` |
| `step_started` | DEBUG | `dt` | 每次 step 开始 |
| `step_completed` | DEBUG | `dt`, `body_count`, `island_count`, `solver_iter` | 每次 step 结束 |
| `solver_no_converge` | WARN | `iter`, `residual` | 求解器未收敛 |
| `body_sleeping` | DEBUG | `body_handle` | body 进入 sleep |
| `body_waking` | DEBUG | `body_handle`, `reason` | body 退出 sleep |
| `contact_lost` | WARN | `body_a`, `body_b` | 应有接触但 manifold 为空 |
| `error` | ERROR | `error_code`, `context` | 任何错误 |
| `panic_caught` | ERROR | `location` | FFI 边界 panic |

### 3.3 字段命名规范

- `body_handle` / `constraint_handle` / `island_handle`：用 handle 类型而非裸 index；
- `dt`：浮点秒；
- 时间戳：用 `tracing` 的内置 timestamp；
- 错误码：用 `error_code: u32` 而非字符串（便于聚合分析）。

## 4. 性能层（L3）

### 4.1 启用方式

- 默认 off；
- `cargo build --features tracing-perf` 启用；
- 仅在 profiling / 性能调试时使用。

### 4.2 Span 设计

| Span | 字段 | 嵌套 |
|---|---|---|
| `step` | `dt` | 顶层 |
| `broad_phase` | `pair_count` | step |
| `narrow_phase` | `manifold_count` | step |
| `solve` | `iter`, `constraint_count` | step |
| `island_step` | `island_id` | step |
| `integrate` | `body_count` | step |
| `ffn_call` | `function_name` | （FFI 边界） |

### 4.3 自定义计数器

```rust
#[cfg(feature = "tracing-perf")]
use metrics::{counter, histogram};

counter!("gvpe.solver.iter_total").increment(iter as u64);
histogram!("gvpe.step.duration_ms").record(elapsed.as_secs_f64() * 1000.0);
```

- 计数器 / 直方图导出到 `tracing-subscriber` 选择的 sink；
- 集成方可在生产环境采样（如 1/1000 step）以控制开销。

### 4.4 性能开销

- L1（错误）：~0% 正常路径；
- L2（事件）：feature off = 编译期消除（0 运行时开销）；feature on 但 log level off = ~5-10ns/event；
- L3（Span + counter）：feature off = 0；feature on = ~100-500ns/span（与嵌套深度相关）。

## 5. 崩溃诊断

### 5.1 panic 时的行为

- 核心 crate panic：在 `catch_unwind` 边界转为 `GVPE_E_PANIC`；
- 集成方进程：**不**崩溃（除非 `panic = "abort"`）；
- panic 现场信息：通过 `tracing` 的 `panic` 事件或自定义 `panic_hook` 输出到 stderr（仅在 debug build）。

### 5.2 core dump

- 集成方可启用 OS-level core dump；
- GVPE 自身**不**主动 dump（避免集成方进程体积问题）；
- 集成方上传 core dump → 维护者用 `coredumpctl` / `lldb` / `rust-gdb` 分析。

### 5.3 错误现场捕获（设计建议）

- 在 `catch_unwind` 中捕获 panic payload，**不**直接转字符串（避免分配）；
- 调用 `gvpe_get_last_error` 时再格式化（仅在集成方需要时付出成本）。

## 6. 集成方接口

### 6.1 日志级别控制

```c
typedef enum {
    GVPE_LOG_OFF = 0,
    GVPE_LOG_ERROR = 1,
    GVPE_LOG_WARN = 2,
    GVPE_LOG_INFO = 3,
    GVPE_LOG_DEBUG = 4,
    GVPE_LOG_TRACE = 5,
} gvpe_log_level_t;

void gvpe_runtime_set_log_level(gvpe_runtime_t rt, gvpe_log_level_t level);
```

- 默认：`WARN`；
- 集成方可运行时调整。

### 6.2 日志订阅

- GVPE 自身**不**初始化 `tracing-subscriber`；
- 集成方负责初始化 subscriber 并选择 sink（stdout / file / otel / 内存 buffer）；
- 集成方需在自己进程内调：
  ```rust
  use tracing_subscriber::{EnvFilter, fmt};
  tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).init();
  ```

## 7. 与其他系统的集成

### 7.1 OpenTelemetry

- 集成方启用 `tracing-subscriber` 的 `tracing-opentelemetry` layer；
- GVPE span / event 自动导出；
- **不**在 GVPE crate 内强依赖 OTel（保持轻量）。

### 7.2 性能 profiler

- 集成方可用 `perf` / `flamegraph` / `cargo flamegraph` 采样；
- GVPE 提供 `cargo bench` + criterion 数据；
- 与 `tracing` 集成通过 `tracing-flame` layer（feature opt-in）。

## 8. 隐私 / 安全

- **不**记录集成方业务数据（body 位置 / 速度 / 力等可视为业务数据）；
- panic 现场信息可包含程序内部状态；集成方应自行决定是否上传；
- 日志中**不**含凭证、用户标识等敏感信息。

## 9. 关联

- `GVPE-DOC-17` §10（FFI 边界 + panic 安全）
- `GVPE-DOC-26` §18.9（`tracing` 选型）
- `55_error_code_catalog.md`（错误码）
- `28_workflow.md` §10.4 步 51

## 10. 审批

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 编写 | — | | |
| 评审 | | | |
| 批准 | | | |
