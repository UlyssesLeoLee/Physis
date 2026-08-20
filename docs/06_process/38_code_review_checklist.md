# Code Review 详细清单（Code Review Checklist）

> **用途**：Reviewer 在 review PR 时逐项检查的清单。**对 PR 模板（`37_pr_template.md`）的扩展**。
> **对应工作流步骤**：`28_workflow.md` §10.5 步 56、§11.7。

## 0. Reviewer 准入

- **最低 reviewer 数**：1（普通 PR）/ 2（涉及 `unsafe` / 公开 API 形状 / `Cargo.toml` 依赖变更 / 任何 `GVPE-PROHIBIT-*` 触及点）；
- **必须 review 的角色**：
  - 普通 crate 修改：1 名 crate 维护者；
  - `unsafe` 块：1 名 crate 维护者 + 1 名架构师；
  - 公开 API 形状变化：1 名 crate 维护者 + 1 名架构师；
  - `Cargo.toml` 依赖变更：1 名 crate 维护者 + 1 名架构师 + 1 名许可证负责人（如涉及图 / 向量库）；
  - `#[repr(C)]` ABI 变化：1 名 crate 维护者 + 架构师 + Release Manager；
  - 工作流 / 文档 PR：1 名文档维护者 + 1 名核心 reviewer。

## 1. 通用规范

- [ ] **PR 标题规范**：`[crate-name] 简明动词短语`；
- [ ] **PR 描述完整**：概述 / 关联 ID / 改动点 / 影响范围 / 禁用项声明 / 测试 / 文档同步；
- [ ] **改动集中**：单一关注点；不夹杂无关重构；
- [ ] **commit message 规范**：与 `CONTRIBUTING.md` 一致（推荐 conventional commits）；
- [ ] **无冲突**：已 rebase 到最新 `main` / `develop`；
- [ ] **CI 全绿**：fmt / clippy / test 全过。

## 2. 选型 / 依赖（与 `GVPE-DOC-26` 对齐）

- [ ] **未引入** `GVPE-DOC-26` §18.13 拒绝清单中的任何库；
- [ ] **未引入** 任何图 / 向量数据库到核心 crate（PROHIBIT-03/04）；
- [ ] **未引入** 任何 ML / 推理库到仿真路径（PROHIBIT-05）；
- [ ] **新增依赖** 已在 `GVPE-DOC-16` 通过许可证审查；
- [ ] **新增依赖** 在 `Cargo.toml` 中 feature-gated（如适用）；
- [ ] **`cargo tree` 验证 `AC-02`** 仍成立：核心 crate 不依赖 Graph / Vector / Compiler / Inference / 3DGS。

## 3. 正确性

- [ ] **逻辑正确**：算法实现符合对应设计文档（`GVPE-DOC-NN`）；
- [ ] **边界 case**：处理了 0 / 负 / 极大 / NaN / Inf 输入；
- [ ] **错误传播**：`Result<T, E>` 而非 `panic!`；错误类型与 `thiserror` 错误模型一致；
- [ ] **`unsafe` 块**：
  - [ ] 每个 `unsafe` 块前有 `// SAFETY: ...` 注释，说明不变式；
  - [ ] `unsafe` 块最小化（不超出必要范围）；
  - [ ] 通过 `cargo +nightly miri test` 验证；
  - [ ] 集中趋势：是否应集中到 `gvpe-unsafe` 而非散落？
- [ ] **未引入新 `unwrap` / `expect`** 到非测试代码（除非有充分理由并注释）；
- [ ] **数值稳定性**：物理算法在边界条件下不出现 NaN / Inf / 数值爆炸（参考 `QA-I-02`）。

## 4. 性能 / 内存

- [ ] **热路径无分配**：`String` / `Vec` / `HashMap` / `Box` 仅在冷路径或经设计评审特批；
- [ ] **零拷贝**：跨函数传递 slice / `&[T]` 而非 owned 数据（热路径）；
- [ ] **SoA / AoSoA**：热数据按 `GVPE-DOC-05` 拆开；
- [ ] **SIMD 化**：热路径是否 SIMD（参考 `GVPE-DOC-26` §18.5）；
- [ ] **criterion bench**：性能关键路径有 bench；PR 附本地 bench 数字（与基线对比）；
- [ ] **无回归**：性能变化 < 5%；如有显著性能变化，已说明原因。

## 5. API 设计

- [ ] **公开 API 形状** 符合对应设计文档（`GVPE-DOC-NN`）；
- [ ] **类型层清晰**：利用 enum / sealed trait / PhantomData 表达不变量；
- [ ] **错误类型**：使用 `thiserror` derive；不引入 `anyhow` 到核心 crate；
- [ ] **trait 抽象**：trait 形状与 `GVPE-DOC-17` 一致；
- [ ] **生命周期**：标注清晰，无 `'static` 滥用；
- [ ] **可见性**：最小公开面；`pub(crate)` 优先于 `pub`；
- [ ] **API 文档**：所有 `pub` 项有 `///` 文档；
- [ ] **Breaking change**：避免；如必须，需走变更管理。

## 6. FFI（仅 `gvpe-ffi` PR）

- [ ] **panic 安全**：`catch_unwind` 包裹每个 `extern "C"` 函数（`QA-F-02`）；
- [ ] **`#[repr(C)]`**：所有跨 FFI 类型标注；
- [ ] **句柄**：世代索引，跨 FRI 边界使用 u32 + u32 编码；
- [ ] **ABI 兼容**：未修改任何 `#[repr(C)]` 类型字段（除非走变更管理）；
- [ ] **cbindgen**：`Cargo.toml` 与 `cbindgen.toml` 同步；
- [ ] **panic = "abort"**：`[profile.release]` 在 `gvpe-ffi` 中配置（`QA-F-02`）。

## 7. 测试

- [ ] **单元测试**：每个新公共 API 至少 1 happy + 1 error path；
- [ ] **集成测试**：跨 crate 集成有集成测试；
- [ ] **miri**：`unsafe` 块通过 miri（`QA-Q-01`）；
- [ ] **determinism harness**：核心 crate 在 `--no-default-features` 下行为不变（`R-FR-001`）；
- [ ] **覆盖率**：不下降；
- [ ] **对照测试**：求解器 / 碰撞等算法有对照实现测试（不嵌入第三方）；
- [ ] **`cargo tree`** artifact 更新（如依赖变化）。

## 8. 文档

- [ ] **`///` 文档**：所有 `pub` 项；
- [ ] **对应 `GVPE-DOC-NN` 更新**：如影响设计 / API；
- [ ] **`27_qa_register.md`**：对应 QA 项状态更新（如关闭 / 重评）；
- [ ] **无 doc drift**：与代码同步；
- [ ] **`CHANGELOG.md`**：用户可见变更；
- [ ] **示例**：复杂 API 有 `examples/` 或 `tests/` 中的使用示例。

## 9. 许可证 / 安全

- [ ] **新增依赖** 已在 `GVPE-DOC-16` 通过审查；
- [ ] **`cargo deny`** 通过：licenses / bans / advisories / sources；
- [ ] **无 copyleft** 进入核心 crate；
- [ ] **无网络 / 加密库** 不必要地引入；
- [ ] **RUSTSEC 公告**：新增依赖无未处理 High / Critical。

## 10. Review 结论

- [ ] **Approve**：可合并；
- [ ] **Request Changes**：必须修改后重审；
- [ ] **Comment**：意见但不阻塞合并（明确说明）；
- [ ] **Reject**：设计层面拒绝，关闭 PR。

## 11. 合并后

- [ ] 关闭关联 issue（如有）；
- [ ] 更新 `27_qa_register.md` 状态；
- [ ] Release Manager 知晓（破坏 ABI / 性能关键 PR）；
- [ ] 集成方接口通知（用户可见变更）。

## 12. 关联

- `28_workflow.md` §10.5 步 56、§11.7
- `37_pr_template.md`（PR 模板）
- `GVPE-DOC-26`（技术选型）
- `GVPE-DOC-27`（QA 登记）
- `GVPE-DOC-15`（测试策略）
