# 错误码目录（Error Code Catalog）

> **用途**：所有 crate 的错误类型、错误码、触发条件、恢复建议、FFI 映射的**权威登记表**。
> **对应工作流步骤**：44 クラス設計、50 エラー処理設計 → `28_workflow.md` §10.4 步 44/50。
> **关联**：`GVPE-DOC-17` §13（错误模型总览）；`GVPE-DOC-10`（C ABI 错误码）；`GVPE-DOC-26` §18.8（`thiserror` 选型）；`GVPE-DOC-27` §9.2（I 类 QA 项）。

## 0. 元数据

| 字段 | 取值 |
|---|---|
| 文档编号 | GVPE-DOC-55 |
| 文档类型 | 詳細設計書 |
| 版本 | v0.1 |
| 状态 | Draft |
| 适用阶段 | MVP / 详细设计 |
| 上游文档 | `GVPE-DOC-17` §13, `GVPE-DOC-10`, `GVPE-DOC-26` §18.8 |
| 下游文档 | 各 crate 实现、`gvpe-ffi` 头文件生成 |

## 1. 错误模型总览

### 1.1 错误传递模式

- **crate 内部**：`Result<T, E>` + 具体错误枚举（`thiserror` derive）；
- **crate 边界**：将具体错误类型转换为 `gvpe-core` 定义的统一错误类型（`RuntimeError` / `CompileError` / `ShapeError` 等）；
- **FFI 边界**（`gvpe-ffi`）：错误枚举 → C `u32` 错误码（`GVPE_E_*`）；
- **panic**：核心 crate 视为 bug；FFI 边界 `catch_unwind` 转为 `GVPE_E_PANIC`。

### 1.2 错误码分配规则

- 错误码 = `u32`；
- 0 = 成功（`GVPE_OK`）；
- 错误码高 16 位 = crate 标识（与 `04_architecture.md` §4.1 拓扑一致）；
- 低 16 位 = crate 内错误序号；
- 例：`GVPE_E_SOLVER_NO_CONVERGE = 0x0007_0001`（crate 7 = `gvpe-solver` 的 1 号错误）。

## 2. 错误类型与错误码

### 2.1 `gvpe-core`（crate 标识 0x0002）

| 错误类型 | 错误码 | 触发条件 | 恢复建议 | FFI 错误名 |
|---|---|---|---|---|
| `HandleError::Stale` | `0x0002_0001` | handle 引用已 free 的资源 | 重新获取 handle | `GVPE_E_INVALID_HANDLE` |
| `HandleError::Invalid` | `0x0002_0002` | handle 编码非法 | 重新获取 handle | `GVPE_E_INVALID_HANDLE` |
| `ProfileError::MissingField` | `0x0002_0010` | `PhysicsProfile` 必填字段缺失 | 重新构造 | `GVPE_E_INVALID_PROFILE` |
| `ProfileError::InconsistentField` | `0x0002_0011` | 字段值违反不变式（如 mass ≤ 0 + 静态） | 重新构造 | `GVPE_E_INVALID_PROFILE` |
| `DescriptorError::Empty` | `0x0002_0020` | `RuntimeDescriptor` 无 body | 添加 body | `GVPE_E_EMPTY_DESCRIPTOR` |
| `DescriptorError::ShapeConflict` | `0x0002_0021` | shape 引用与 profile 冲突 | 修正 | `GVPE_E_INVALID_DESCRIPTOR` |
| `LodError::NotSupported` | `0x0002_0030` | MVP 仅 LOD0，请求其他 LOD | 改用 LOD0 | `GVPE_E_NOT_SUPPORTED` |

### 2.2 `gvpe-memory`（crate 标识 0x0003）

| 错误类型 | 错误码 | 触发条件 | 恢复建议 | FFI 错误名 |
|---|---|---|---|---|
| `ArenaError::Overflow` | `0x0003_0001` | arena 预分配耗尽 | 调大 arena | `GVPE_E_OUT_OF_MEMORY` |
| `PoolError::Exhausted` | `0x0003_0002` | pool 满 | 调大 pool | `GVPE_E_OUT_OF_MEMORY` |
| `PoolError::InvalidIndex` | `0x0003_0003` | 索引越界或 double-free | 重新获取 | `GVPE_E_INVALID_INDEX` |
| `SlabError::GenerationMismatch` | `0x0003_0004` | slab 世代不匹配（use-after-free） | 重新获取 | `GVPE_E_INVALID_HANDLE` |

### 2.3 `gvpe-shape`（crate 标识 0x0004）

| 错误类型 | 错误码 | 触发条件 | 恢复建议 | FFI 错误名 |
|---|---|---|---|---|
| `ShapeError::Unsupported` | `0x0004_0001` | shape 类型未实现（如 MVP 中 ConvexHull） | 改用支持 shape | `GVPE_E_NOT_SUPPORTED` |
| `ShapeError::Degenerate` | `0x0004_0002` | shape 参数退化（如 half_extent = 0） | 修正 shape | `GVPE_E_INVALID_SHAPE` |
| `ShapeError::NaN` | `0x0004_0003` | shape 参数含 NaN | 修正 shape | `GVPE_E_INVALID_SHAPE` |

### 2.4 `gvpe-collision`（crate 标识 0x0005）

| 错误类型 | 错误码 | 触发条件 | 恢复建议 | FFI 错误名 |
|---|---|---|---|---|
| `BroadPhaseError::TooManyPairs` | `0x0005_0001` | broad phase 输出对数超阈值 | 调阈值 / 减少 body | `GVPE_E_TOO_MANY` |
| `NarrowPhaseError::Numerical` | `0x0005_0010` | SAT/GJK 数值不收敛 | 调整 tolerance | `GVPE_E_NUMERICAL` |
| `ManifoldError::Empty` | `0x0005_0020` | 应有接触但 manifold 为空 | 调整 skin width | `GVPE_E_NO_CONTACT` |
| `CcdError::MaxIterations` | `0x0005_0030` | conservative advancement 达到最大迭代 | 调大 max iter | `GVPE_E_TOO_MANY` |

### 2.5 `gvpe-dynamics`（crate 标识 0x0006）

| 错误类型 | 错误码 | 触发条件 | 恢复建议 | FFI 错误名 |
|---|---|---|---|---|
| `BodyError::InvalidMass` | `0x0006_0001` | mass ≤ 0 且非 static | 修正 profile | `GVPE_E_INVALID_PROFILE` |
| `BodyError::InvalidInertia` | `0x0006_0002` | inertia tensor 退化 | 修正 profile | `GVPE_E_INVALID_PROFILE` |
| `IntegrateError::NaN` | `0x0006_0010` | 积分后产生 NaN | 报错（视为 bug） | `GVPE_E_INTERNAL` |
| `IntegrateError::Inf` | `0x0006_0011` | 积分后产生 Inf | 报错（视为 bug） | `GVPE_E_INTERNAL` |

### 2.6 `gvpe-solver`（crate 标识 0x0007）

| 错误类型 | 错误码 | 触发条件 | 恢复建议 | FFI 错误名 |
|---|---|---|---|---|
| `ConstraintError::Degenerate` | `0x0007_0001` | constraint 退化（mass=0 + 静态） | 修正 | `GVPE_E_INVALID_PROFILE` |
| `SolverError::NoConverge` | `0x0007_0010` | 迭代达到上限仍未收敛 | 增 iter / 减小 dt | `GVPE_E_NOT_CONVERGE` |
| `SolverError::Numerical` | `0x0007_0011` | 求解过程数值异常 | 报 bug | `GVPE_E_NUMERICAL` |
| `SleepError::InconsistentState` | `0x0007_0020` | sleep 状态机不一致 | 报 bug | `GVPE_E_INTERNAL` |

### 2.7 `gvpe-island`（crate 标识 0x0008）

| 错误类型 | 错误码 | 触发条件 | 恢复建议 | FFI 错误名 |
|---|---|---|---|---|
| `IslandError::Overflow` | `0x0008_0001` | 岛数 / 大小超阈值 | 调阈值 | `GVPE_E_TOO_MANY` |

### 2.8 `gvpe-scheduler`（crate 标识 0x0009）

| 错误类型 | 错误码 | 触发条件 | 恢复建议 | FFI 错误名 |
|---|---|---|---|---|
| `JobError::Invalid` | `0x0009_0001` | job 句柄无效 | 重新提交 | `GVPE_E_INVALID_HANDLE` |
| `SchedulerError::Shutdown` | `0x0009_0010` | scheduler 已关闭 | 重新初始化 | `GVPE_E_SHUTDOWN` |

### 2.9 `gvpe-runtime`（crate 标识 0x000A）

| 错误类型 | 错误码 | 触发条件 | 恢复建议 | FFI 错误名 |
|---|---|---|---|---|
| `RuntimeError::NotInit` | `0x000A_0001` | 未初始化 | 初始化 | `GVPE_E_NOT_INIT` |
| `RuntimeError::AlreadyInit` | `0x000A_0002` | 已初始化重复 init | 先 destroy | `GVPE_E_ALREADY_INIT` |
| `RuntimeError::StepFailed` | `0x000A_0010` | 内部 step 失败（聚合自下层） | 查更具体错误 | `GVPE_E_STEP_FAILED` |
| `RuntimeError::AbiMismatch` | `0x000A_0020` | 集成方 ABI 版本不匹配 | 升级 GVPE | `GVPE_E_ABI_MISMATCH` |

### 2.10 `gvpe-ffi`（crate 标识 0x000B）

| 错误类型 | 错误码 | 触发条件 | 恢复建议 | FFI 错误名 |
|---|---|---|---|---|
| （panic 已捕获） | `0x000B_0001` | Rust panic 跨边界 | 报 bug | `GVPE_E_PANIC` |
| （空指针） | `0x000B_0010` | 集成方传入 NULL | 集成方修正 | `GVPE_E_NULL` |
| （buffer 越界） | `0x000B_0011` | 集成方 buffer 长度不足 | 集成方分配足够 | `GVPE_E_BUFFER_TOO_SMALL` |

### 2.11 `gvpe-compiler`（crate 标识 0x000D）

| 错误类型 | 错误码 | 触发条件 | 恢复建议 | FFI 错误名 |
|---|---|---|---|---|
| `CompileError::UnsupportedModel` | `0x000D_0001` | 模型未实现（如 Fluid / FEM MVP） | 改用支持模型 | `GVPE_E_NOT_SUPPORTED` |
| `CompileError::InvalidGraph` | `0x000D_0002` | 图结构违反不变式 | 修正图 | `GVPE_E_INVALID_GRAPH` |
| `CompileError::Validation` | `0x000D_0003` | 验证失败 | 修正 | `GVPE_E_VALIDATION` |

### 2.12 `gvpe-vector`（crate 标识 0x000E）

| 错误类型 | 错误码 | 触发条件 | 恢复建议 | FFI 错误名 |
|---|---|---|---|---|
| `VectorError::Empty` | `0x000E_0001` | index 空 | 添加数据 | `GVPE_E_EMPTY` |
| `VectorError::DimensionMismatch` | `0x000E_0002` | query 与 index 维度不匹配 | 修正 | `GVPE_E_INVALID_QUERY` |
| `VectorError::TypeMismatch` | `0x000E_0003` | 跨类型签名比较（`VEC-002` 防护） | 修正 | `GVPE_E_TYPE_MISMATCH` |

### 2.13 通用错误码

| 错误码 | 含义 |
|---|---|
| `0x0000_0000` (`GVPE_OK`) | 成功 |
| `0x0001_0001` (`GVPE_E_INVALID_ARG`) | 通用非法参数 |
| `0x0001_0002` (`GVPE_E_OUT_OF_MEMORY`) | 通用 OOM |
| `0x0001_0003` (`GVPE_E_INTERNAL`) | 通用内部错误（视为 bug） |
| `0x0001_0004` (`GVPE_E_NOT_SUPPORTED`) | 通用不支持 |
| `0x0001_0005` (`GVPE_E_PANIC`) | panic 跨 FFI 边界 |
| `0x0001_0006` (`GVPE_E_NULL`) | NULL 指针 |

## 3. FFI 错误传递协议

### 3.1 函数返回错误码

```c
// gvpe 风格：u32 返回码
uint32_t gvpe_step(gvpe_runtime_t rt, float dt);  // 0 = OK, non-zero = error code
```

### 3.2 详细错误信息（可选）

```c
typedef struct {
    uint32_t code;
    uint32_t line;           // 触发位置（编译期固定）
    const char* file;        // 触发文件（编译期固定）
    const char* message;     // 静态字符串
} gvpe_error_t;

void gvpe_get_last_error(gvpe_runtime_t rt, gvpe_error_t* out);
```

- 集成方可在 step 后调 `gvpe_get_last_error` 获取详细；
- 错误字符串为 **静态常量**，集成方**不**释放；
- 多线程场景：每个 thread 独立的 last-error buffer。

### 3.3 panic 边界

```rust
#[no_mangle]
pub extern "C" fn gvpe_step(rt: *mut Runtime, dt: f32) -> u32 {
    std::panic::catch_unwind(|| {
        // ... actual step
    }).unwrap_or(GVPE_E_PANIC)
}
```

- **所有** `extern "C"` 函数必须用 `catch_unwind` 包裹；
- 详见 `17_detailed_design.md` §10 + `10_ffi_design.md`。

## 4. 错误处理最佳实践

- **crate 内部**：`Result<T, E>` + `?` + `thiserror::Error`；
- **不**在核心 crate 使用 `anyhow`（与 `26_tech_selection.md` §18.8 一致）；
- **不**在核心 crate 使用 `unwrap` / `expect`（除非有充分理由并 `// SAFETY:` 注释）；
- **错误信息** 应可定位（包含字段名 / 索引 / 数值等）；
- **集成方文档**：所有 `GVPE_E_*` 错误码在头文件 doc 中说明。

## 5. 关联

- `GVPE-DOC-17` §13（错误模型总览）
- `GVPE-DOC-10`（C ABI 错误传递）
- `GVPE-DOC-26` §18.8（`thiserror` 选型）
- `28_workflow.md` §10.4 步 50（エラー処理設計）

## 6. 审批

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 编写 | — | | |
| 评审 | | | |
| 批准 | | | |
