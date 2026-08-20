# 集成测试规格书模板（Integration Test Spec Template）

> **用途**：跨 crate 集成测试的规格书，命名 `it_spec_<scope>.md`。
> **对应工作流步骤**：`28_workflow.md` §10.7 步 67-75（結合試験仕様書 / 内部 / 外部 / API / 外部連携）。
> **关联**：`GVPE-DOC-15`（测试策略）、`GVPE-DOC-10`（C ABI）、`GVPE-DOC-26`（技术选型）、`GVPE-DOC-04`（crate map）。

## 0. 元数据

| 字段 | 取值 |
|---|---|
| 集成范围 | `<e.g. gvpe-collision + gvpe-dynamics + gvpe-solver>` |
| 文档版本 | v0.X |
| 编写者 | — |
| 关联设计文档 | `GVPE-DOC-NN` |
| 关联需求 ID | `GVPE-FR-XXX`, `GVPE-NFR-XXX`, `GVPE-AC-XX` |

## 1. 集成范围

| Crate | 角色 | 集成方式 |
|---|---|---|
| `gvpe-collision` | broad/narrow phase | API 调用 |
| `gvpe-dynamics` | rigid body 状态 | `pub` 字段访问 |
| `gvpe-solver` | 顺序冲量求解 | 共享 `ConstraintRow` |
| ... | ... | ... |

## 2. 接口契约测试

> 跨 crate 的公开 API 契约必须在集成测试中验证（不是单元测试）。

| 接口 | 契约 | 测试函数 | 期望 |
|---|---|---|---|
| `gvpe-collision::broad_phase` → `gvpe-dynamics::bodies` | 输入 `[BodyHandle]`，输出 `[ContactPair]` | `test_broad_phase_returns_valid_pairs` | 所有 pair 的 body handle 都在输入集合中 |
| `gvpe-solver::solve_step` 接受 `ConstraintRow[]` | 不修改 body 状态，只写 lambda 冲量 | `test_solver_pure_function` | 输入 body 状态哈希不变 |
| ... | ... | ... | ... |

## 3. 依赖方向验证

> 对应 `AC-02`（机械可验证依赖方向）。

| 命令 | 期望 |
|---|---|
| `cargo tree -p gvpe-core -p gvpe-collision -p gvpe-dynamics -p gvpe-constraint -p gvpe-solver -p gvpe-island -p gvpe-scheduler -p gvpe-runtime` | 输出中**不**包含 `gvpe-graph` / `gvpe-vector` / `gvpe-compiler` / `gvpe-inference` / `gvpe-3dgs` |
| `cargo tree -p gvpe-ffi` | 仅依赖 `gvpe-runtime`（+ 必要的 `gvpe-core`） |
| `cargo tree -p gvpe-compiler` | 可依赖 `gvpe-graph` / `gvpe-vector`（单向） |

**测试自动化**：上述命令输出 diff 到 PR 附件，CI 阻断任何**新增**的非法依赖。

## 4. ABI 兼容测试

> 任何 `#[repr(C)]` 类型变更需自动检测 ABI 差异。

| 类型 | 检测方法 | 期望 |
|---|---|---|
| `PhysicsProfile` | `cargo build` 后 `bindgen` 校验布局哈希 | 与上一 release 一致 |
| `RuntimeDescriptor` | 同上 | 同上 |
| `BodyHandle` | `#[repr(C)]` `u32` + `u32` | 大小 = 8 字节，对齐 = 4 |

**测试函数**：`test_abi_size_of_xxx` 用 `std::mem::size_of` + `assert_eq!` 锁定布局。

## 5. 跨 crate 性能 / 内存

| 指标 | 测量方式 | 目标 |
|---|---|---|
| 全集成 step 耗时（500 body） | criterion `bench_full_step` | < 16ms @ 60Hz |
| 集成零分配 | `cargo test --features track-alloc` + 计数器 | 热路径 0 alloc |
| 多线程扩展性（1/2/4/8 核） | criterion `bench_parallel_scaling` | 接近线性到 4 核 |

## 6. C ABI 集成测试（71 步）

> 对应工作流步 71（API 結合試験）。

| 场景 | 测试函数 | 期望 |
|---|---|---|
| C 端创建 scene | `test_ffi_create_scene` | 返回非空 handle |
| C 端 step | `test_ffi_step_returns_zero` | 返回 `GVPE_OK` |
| panic 跨 FFI 边界 | `test_ffi_panic_safety` | panic 被 catch，转化为 `GVPE_E_PANIC` 错误码 |
| 句柄无效 | `test_ffi_invalid_handle` | 返回 `GVPE_E_INVALID_HANDLE`，不 UB |
| 长字符串输入 | `test_ffi_long_string` | 截断或返回错误，不溢出 |
| 并发 step | `test_ffi_concurrent_*` | MVP 不支持并发 Runtime，文档化 |

## 7. 失败注入 / 健壮性

| 注入 | 期望 |
|---|---|
| 0 数量 body | step 正常退出，输出空 |
| 全部 sleeping | step 快速通过（< 1ms） |
| 极端重力（±1e10） | 不 NaN，不 panic |
| 极端 timestep（0 / 负 / 1e6） | 错误码 |
| 极大迭代次数 | 正确性，耗时线性 |

## 8. 关联工作流步骤

- `28_workflow.md` §10.7 步 66-75
- `28_workflow.md` §11.12（内部集成）、§11.13（外部集成）、§11.14（不具合对应）、§11.15（功能测试）

## 9. 审批

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 编写 | — | | |
| 评审 | | | |
| 批准 | | | |
