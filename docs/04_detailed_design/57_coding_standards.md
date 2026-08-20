# 编码标准（Coding Standards）

> **用途**：GVPE 全套 crate 的编码规范——风格、命名、错误处理、unsafe 政策、模块组织、文档。
> **对应工作流步骤**：44 クラス設計、45 ロジック設計 → `28_workflow.md` §10.4 步 44/45。
> **关联**：`GVPE-DOC-26` §18.11（lint 配置）；`GVPE-DOC-17`（详细设计）；`38_code_review_checklist.md`（review 用）；`62_unsafe_inventory.md`（unsafe 政策）。

## 0. 元数据

| 字段 | 取值 |
|---|---|
| 文档编号 | GVPE-DOC-57 |
| 文档类型 | 詳細設計書 |
| 版本 | v0.1 |
| 状态 | Draft |
| 适用阶段 | MVP / 实施期 |
| 上游文档 | `GVPE-DOC-26` §18.11, `GVPE-DOC-17` |
| 下游文档 | 实施期 PR / `CONTRIBUTING.md` / `38_code_review_checklist.md` |

## 1. 工具链规范

### 1.1 格式化（rustfmt）

仓库根 `rustfmt.toml`：

```toml
edition = "2021"
max_width = 100
tab_spaces = 4
use_small_heuristics = "Default"
format_code_in_doc_comments = true
format_strings = true
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
reorder_impl_items = true
```

CI 强制 `cargo fmt --all -- --check`。

### 1.2 Lint（clippy）

CI 强制：

```bash
cargo clippy --all-targets --all-features -- \
  -D warnings \
  -D clippy::pedantic \
  -D clippy::nursery \
  -W clippy::restriction  # warn 而非 deny，留 escape 口
```

### 1.3 deny lint（crate 入口）

```rust
#![deny(
    unsafe_op_in_unsafe_fn,  // Rust 2024 行为前移
    missing_debug_implementations,
    missing_docs,            // 公共 API 强制文档
    rust_2018_idioms,
    unused_unsafe,
)]
#![warn(non_ascii_idents)]   // 警告不 deny（兼容中文注释）
```

### 1.4 miri

- 核心 unsafe crate 在 CI 跑 `cargo +nightly miri test`；
- 新增 `unsafe` 必须 miri 验证；
- 详见 `62_unsafe_inventory.md`。

## 2. 命名规范

| 类别 | 规则 | 示例 |
|---|---|---|
| Crate | `gvpe-<role>` 全小写，连字符分隔 | `gvpe-solver`, `gvpe-memory` |
| 模块 | `snake_case` | `broad_phase`, `narrow_phase` |
| 类型 / Struct | `UpperCamelCase` | `BodyHandle`, `ConstraintRow` |
| Trait | `UpperCamelCase`，形容词 / 名词 | `PhysicsCompiler`, `Send + Sync` |
| Enum 变体 | `UpperCamelCase` | `Lod0Full`, `Lod1Reduced` |
| 函数 / 方法 | `snake_case`，动词优先 | `add_body`, `compute_step` |
| 变量 / 参数 | `snake_case` | `body_count`, `solver_iter` |
| 常量 | `SCREAMING_SNAKE_CASE` | `MAX_BODY_COUNT` |
| 静态变量 | `SCREAMING_SNAKE_CASE` | `GLOBAL_CONFIG` |
| 类型参数 | `UpperCamelCase`，单字母允许 | `T`, `R`, `PhysicsProfile` |
| 错误类型后缀 | `Error` | `SolverError`, `ShapeError` |
| 错误变体后缀 | `Error` 关联 | `SolverError::NoConverge` |
| Result 别名 | `Result<T>` 默认；crate 特定用 `<Name>Result<T>` | `CompileResult<T>` |

## 3. 模块组织

### 3.1 库结构

```text
crates/gvpe-xxx/
├── Cargo.toml
├── src/
│   ├── lib.rs              # crate 根，pub use 汇总
│   ├── error.rs            # 错误类型
│   ├── handle.rs           # 句柄类型（如适用）
│   ├── <module1>.rs        # 子模块
│   ├── <module2>.rs
│   └── tests/              # 集成测试（如需要）
└── tests/                  # 端到端测试（如需要）
```

### 3.2 可见性原则

- 默认 `pub(crate)`；
- 仅当**确实**需要 crate 外访问时升级为 `pub`；
- 公共 API 应**最小化**（crate 公共 surface ≤ 30 个 trait / struct / fn）；
- `pub use` 重新导出时谨慎（避免类型泄漏）。

### 3.3 子模块嵌套

- 嵌套不超过 2 层（`crate::module::submodule`）；
- 嵌套越深 → 重构为独立子模块或独立 crate。

## 4. 错误处理

### 4.1 错误模式

- **crate 内部**：`Result<T, E>` + 具体错误枚举 + `thiserror`；
- **crate 边界**：转换为 `gvpe-core` 统一错误（详见 `55_error_code_catalog.md`）；
- **FFI 边界**：`u32` 错误码 + `catch_unwind`；
- **测试代码**：`unwrap` / `expect` 允许；
- **不**在核心 crate 使用 `anyhow`（与 `26_tech_selection.md` §18.8 一致）。

### 4.2 错误信息

错误信息应：
- **包含字段名** / **索引** / **数值**等可定位信息；
- 使用 `thiserror` 的 `#[error("...")]` 派生 `Display`；
- 实现 `std::error::Error`（自动 from `thiserror`）；
- **不**包含敏感数据（凭证、用户标识等）。

### 4.3 panic 政策

- 核心 crate 求解路径：**panic = bug**（视为代码错误，CI 测试覆盖）；
- 冷路径（场景加载、配置解析）：允许 `panic`（输入非法）；
- 测试代码：`unwrap` / `expect` 允许；
- 库产品**不**在 panic 时打印到 stderr（让集成方决定）；
- FFI 边界：`catch_unwind` 兜底（详见 `56_logging_design.md`）。

## 5. unsafe 政策

### 5.1 总体原则

- **最小化**：能用 safe Rust 表达的**不**用 `unsafe`；
- **集中**：尽量将 `unsafe` 集中到 `gvpe-memory` 等核心 crate；
- **审计**：每个 `unsafe` 必须有 `// SAFETY:` 注释 + miri 验证；
- 详见 `62_unsafe_inventory.md`。

### 5.2 `// SAFETY:` 注释

每个 `unsafe` 块前必须有：

```rust
// SAFETY: <不变式 + 满足条件 + 风险>
unsafe {
    // ...
}
```

示例：

```rust
// SAFETY: `index` 已被 `slab.get(index)` 验证为有效；
// 不会越界；`generation` 匹配由调用方保证。
let value = *slab.get_unchecked(index);
```

### 5.3 禁止事项

- **不**使用 `mem::transmute`（除非有充分理由 + 详尽注释）；
- **不**使用 `mem::uninitialized`（已 deprecated，用 `MaybeUninit`）；
- **不**手动管理裸指针生命周期（用 `Box` / `Rc` / `Arc` / 引用）；
- **不**绕过借用检查（用 `split_at_mut` 等安全模式）；
- **不**使用 `#[deny(unsafe_code)]` 关闭特定 crate（会失去 miri 验证能力）；
- **不**在 `unsafe` 中调用其他 `unsafe` 除非必要（嵌套降低可读性）。

## 6. 并发与同步

### 6.1 同步原语

- 仅使用 `std::sync`（`Mutex`, `RwLock`, `Arc`, `Atomic*`）；
- **不**使用 `parking_lot` / `crossbeam` 进核心（与 `26_tech_selection.md` §18.6.2 一致）；
- 锁粒度：尽量小，**不**持锁跨函数调用边界。

### 6.2 Send / Sync

- 公共类型**应**显式 `Send + Sync`（如适用）或 `!Send`（如不适用）；
- `unsafe impl Send / Sync` 必须有详尽注释；
- 性能相关类型（`PhysicsProfile`, `RuntimeDescriptor` 等）应是 `Send + Sync`。

### 6.3 async

- **不**使用 `async` / `await`（与 `26_tech_selection.md` §18.6.3 一致）；
- 用 callback / channel 表达异步（如需要）；
- scheduler 用 task DAG（`gvpe-scheduler`）而非 async runtime。

## 7. 性能

### 7.1 热路径

- **零分配**：`String` / `Vec` / `HashMap` / `Box` 仅在冷路径或经设计评审特批；
- **零拷贝**：跨函数传递 `&[T]` / `&str` 而非 owned 数据；
- **SIMD 友好**：热路径算法应可向量化（`gvpe-math` 提供 SIMD 接口）；
- **内联**：核心热函数 `#[inline]`；错误路径 `#[cold]`。

### 7.2 基准

- 性能关键路径必须有 criterion bench（`29_unit_test_spec_template.md` §5）；
- 性能数字与 `14_performance_budget.md` 对齐；
- 性能回归 ≤ 5%（PR 强制）。

### 7.3 测量

- 性能数字必须用 criterion 测量（统计显著性）；
- 集成方性能数据**不**作为优化目标（环境差异大）；
- 用 `cargo bench --bench <name> -- --save-baseline <vX.Y.Z>` 跟踪基线。

## 8. 文档

### 8.1 `///` 文档

- **所有** `pub` 项必须有 `///` 文档；
- 文档应包含：
  - 一句话功能描述；
  - 参数说明（`# Arguments`）；
  - 返回值说明（`# Returns`）；
  - 错误条件（`# Errors`，仅 `Result` 返回的函数）；
  - 至少 1 个使用示例（`# Examples`）；
  - panic 条件（`# Panics`，如适用）。

### 8.2 内部文档

- `//` 行注释解释**为什么**（不是**做什么**）；
- 复杂算法应有 `//!` 模块级文档解释设计意图；
- `TODO` / `FIXME` / `XXX` 注释应附 issue 编号。

### 8.3 示例

- 公共 API 的复杂使用在 `examples/` 目录提供完整示例；
- 示例必须能 `cargo run --example <name>` 通过。

## 9. 测试

详见 `29_unit_test_spec_template.md`、`30_integration_test_spec_template.md`。

- 单元测试覆盖率 ≥ 80% 行 / 70% 分支；
- 每个新公共 API 至少 1 happy + 1 error path；
- `unsafe` 块 100% miri 验证；
- feature-gate 验证（`R-FR-001`）。

## 10. 提交规范

- 1 PR = 1 关注点；
- commit message 格式（推荐 conventional commits）：
  ```
  <type>(<scope>): <subject>
  <BLANK LINE>
  <body>
  <BLANK LINE>
  <footer>
  ```
- type: `feat` / `fix` / `docs` / `style` / `refactor` / `test` / `chore` / `perf`；
- scope: crate 名（如 `solver` / `memory` / `ffi`）；
- 关联 issue / PR 编号在 footer。

## 11. 依赖

- 新增依赖必须先在 `16_dependency_license.md` 矩阵通过审查；
- 严格遵守 `26_tech_selection.md` §18.13 拒绝清单；
- 依赖更新谨慎（semver 兼容 + 测试通过）。

## 12. 关联

- `GVPE-DOC-17`（详细设计）
- `GVPE-DOC-26` §18.11（lint 配置）
- `38_code_review_checklist.md`（review 用）
- `55_error_code_catalog.md`（错误处理）
- `56_logging_design.md`（日志）
- `62_unsafe_inventory.md`（unsafe 政策）
- `28_workflow.md` §10.4 步 44/45

## 13. 审批

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 编写 | — | | |
| 评审 | | | |
| 批准 | | | |
