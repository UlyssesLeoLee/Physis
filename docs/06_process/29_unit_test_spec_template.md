# 单元测试规格书模板（Unit Test Spec Template）

> **用途**：每个 crate 在 MVP 启动前必须填写本模板的实例，命名 `ut_spec_<crate>.md`，与该 crate 同目录。
> **对应工作流步骤**：`28_workflow.md` §10.6 步 60（単体試験仕様書作成）+ §11.7-§11.11 流程。
> **关联**：`GVPE-DOC-15`（测试策略）、`GVPE-DOC-26`（技术选型）、`GVPE-DOC-27`（QA 登记）。

## 0. 元数据

| 字段 | 取值 |
|---|---|
| Crate 名 | `<gvpe-xxx>` |
| 文档版本 | v0.X |
| 编写者 | — |
| 对应设计文档 | `GVPE-DOC-NN` |
| 关联需求 ID | `GVPE-FR-XXX`, `GVPE-NFR-XXX`, `GVPE-AC-XX` |

## 1. 测试环境

| 维度 | 取值 |
|---|---|
| Rust toolchain | `<stable / pinned version>` |
| 目标 triple | `<e.g. x86_64-unknown-linux-gnu>` |
| Feature 组合 | `--all-features` / `--no-default-features` / `--features <X>` |
| 依赖工具 | `cargo test`, `cargo +nightly miri test`, `cargo llvm-cov` |
| 性能基准工具 | `cargo bench` (criterion) |

## 2. 公共 API 测试矩阵

> 每个 `pub` 函数 / 类型 / trait 必须有行。**至少** 1 个 happy path + 1 个 error path + 1 个 edge case。

| API | 测试函数 | 输入 | 期望输出 | 覆盖需求 |
|---|---|---|---|---|
| `BodyHandle::new` | `test_handle_new_unique` | 连续 1000 次 | 每个 handle 唯一 | `R-NFR-002` |
| `BodyHandle::is_valid` | `test_handle_valid_after_free` | free 后再用 | `false` | `R-FR-002` |
| `...` | ... | ... | ... | ... |

## 3. 错误路径

> 每个返回 `Result<T, E>` 的函数必须列出所有 `Err` 变体及对应触发条件。

| 函数 | 错误变体 | 触发输入 | 期望行为 |
|---|---|---|---|
| `ConstraintRow::new` | `ConstraintError::Degenerate` | mass = 0 | 返回 `Err`，不 panic |
| `BodyHandle::resolve` | `HandleError::Stale` | 已 free 的 handle | 返回 `Err` |
| `...` | ... | ... | ... |

## 4. 边界 / 数值稳定性

> 物理仿真特有的边界条件。

| 测试名 | 输入 | 期望 | 关联 QA |
|---|---|---|---|
| `test_sphere_box_no_penetration_10k_steps` | 10000 步连续仿真 | 无穿透累积 | `QA-I-02` |
| `test_sleeping_threshold_*` | 阈值附近 | 进入 / 退出 sleep 状态正确 | `QA-I-03` |
| `test_aabb_rebuild_*` | 移动 body | AABB 正确更新 | `QA-I-08` |
| `test_sat_box_box_*` | 任意 box-box 姿态 | 15 轴分离检测 | `QA-I-09` |
| `...` | ... | ... | ... |

## 5. 性能基准（criterion）

| bench 名 | 测量 | 目标 | 关联 QA |
|---|---|---|---|
| `bench_broad_phase_sap_1000` | 1000 body SAP | < 1ms | `QA-P-01` |
| `bench_sat_box_box` | 单对 box-box | < 10μs | `QA-P-04` |
| `bench_si_step_500` | 500 body SI 一步 | < 8ms（@ 60Hz） | `QA-P-01` |
| `...` | ... | ... | ... |

## 6. `unsafe` 块测试

> 每个 `unsafe` 块必须有 miri 验证（CI `cargo +nightly miri test`）。

| `unsafe` 位置 | 测试函数 | miri 通过标准 |
|---|---|---|
| `gvpe-memory` Arena alloc | `test_arena_miri` | 无 UB |
| `gvpe-scheduler` work-steal | `test_steal_miri` | 无 data race |
| `...` | ... | ... |

## 7. Feature-gate 验证

> 关键 feature-gate 验证（与 `GVPE-DOC-26` §18 选型对齐）。

| 命令 | 期望 | 关联 |
|---|---|---|
| `cargo test --no-default-features` | 通过（核心仍可运行） | `R-FR-001` |
| `cargo test --no-default-features --features simd-only` | 通过（SIMD 独立验证） | `R-FR-001` |
| `cargo tree -p gvpe-core` | 不含 `gvpe-graph` / `gvpe-vector` / `gvpe-compiler` / `gvpe-inference` / `gvpe-3dgs` | `AC-02` |

## 8. 覆盖率目标

| 指标 | 目标 | 测量方式 |
|---|---|---|
| 行覆盖率 | ≥ 80% | `cargo llvm-cov` |
| 分支覆盖率 | ≥ 70% | `cargo llvm-cov` |
| 公共 API 覆盖率 | 100% | 手动 + 自动化 |
| `unsafe` 块 miri 覆盖 | 100% | CI 强制 |

## 9. CI 集成

- [ ] PR 跑：`cargo test --all-features` + `cargo test --no-default-features` + `cargo clippy -- -D warnings`
- [ ] main 跑：上述 + `cargo +nightly miri test` + `cargo bench`（按分支策略）
- [ ] PR 模板勾选 §2 / §3 / §4 全部测试已添加

## 10. 关联

- `GVPE-DOC-15` §3（单元测试策略）
- `GVPE-DOC-26` §18.10（criterion / cargo test / miri / proptest 选型）
- `GVPE-DOC-27` §9.2 / §9.6（I / Q 类 QA 项）
- `28_workflow.md` §11.6 ~ §11.11

## 11. 审批

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 编写 | — | | |
| 评审 | | | |
| 批准 | | | |
