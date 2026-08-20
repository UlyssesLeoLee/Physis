# Pull Request 模板（PR Template）

> **用途**：GVPE 仓库 `.github/PULL_REQUEST_TEMPLATE.md`（或同等位置）的内容。
> **对应工作流步骤**：`28_workflow.md` §10.5 步 56（コードレビュー）、§11.6-§11.7。
> **关联**：`GVPE-DOC-26`（技术选型）、`GVPE-DOC-27`（QA 登记）、`38_code_review_checklist.md`（CR 详细清单）。

---

<!--
PR 标题规范：[crate-name] 简明动词短语（中文或英文）
示例：[gvpe-solver] 修复 sleeping 状态机边界 case
-->

## 1. 概述

<!-- 一段话说明这个 PR 做了什么，为什么做。 -->

## 2. 关联 ID

<!-- 勾选所有适用的： -->

- [ ] 关联需求：`GVPE-FR-XXX` / `GVPE-NFR-XXX` / `GVPE-PERF-XXX` / `GVPE-LIC-XXX` / `GVPE-PROHIBIT-XX`
- [ ] 关联验收标准：`AC-XX`
- [ ] 关联 QA 登记：`QA-X-NN`（来自 `27_qa_register.md`）
- [ ] 关联设计文档：`GVPE-DOC-NN §X.Y`
- [ ] 关联工作流步骤：`28_workflow.md §X.Y`

## 3. 改动点

<!-- 列出主要改动文件 / 模块 / 函数。 -->

- `<crate>/<file>:<line>` — 改动简述
- ...

## 4. 影响范围

<!-- 回答：哪些 crate / API / 文档 / ABI 受到影响？ -->

- [ ] 仅本 crate 内部，不影响公开 API
- [ ] 影响本 crate 公开 API（**需要更新 `GVPE-DOC-NN` 并申请 ABI review**）
- [ ] 影响跨 crate 依赖（**需 `cargo tree` 验证 `AC-02` 仍成立**）
- [ ] 破坏 ABI（**需 §11.32 变更管理 + Release Manager 签字**）
- [ ] 更新了对应设计文档（**doc drift 风险**）

## 5. 是否触及禁用项

- [ ] **未**新增 `unsafe`（如新增，下方说明）
- [ ] **未**引入 `GVPE-DOC-26` §18.13 拒绝清单中的任何库
- [ ] **未**在热路径引入 `String` / `Vec` / `HashMap`（如引入，下方说明）
- [ ] **未**修改 `#[repr(C)]` 类型的字段（除非走变更管理）
- [ ] **未**绕过 `thiserror` 错误模型
- [ ] **未**破坏 `R-FR-001`（feature-gate 后行为不变）

如有违反，**必须**在本节下方说明理由，并获得架构师 review。

## 6. 新增 `unsafe` 块（如有）

| 位置 | 用途 | `// SAFETY:` 注释 | miri 验证 |
|---|---|---|---|
| `<file>:<line>` | ... | ✅ | ✅ / ⬜ 未跑 |

## 7. 测试

- [ ] 新增单元测试
- [ ] 新增集成测试
- [ ] 新增 criterion bench
- [ ] 跑过 `cargo test --all-features`（本地绿）
- [ ] 跑过 `cargo test --no-default-features`（本地绿）
- [ ] 跑过 `cargo test --no-default-features --features simd-only`（本地绿，如适用）
- [ ] 跑过 `cargo +nightly miri test`（如触及 `unsafe`）

## 8. 文档同步

- [ ] 对应 `GVPE-DOC-NN` 已更新（如影响设计 / API）
- [ ] `27_qa_register.md` 对应 QA 项状态已更新（如关闭 / 重新评估）
- [ ] `CHANGELOG.md` 已更新（如用户可见变更）
- [ ] 代码内 `///` 文档已添加 / 更新

## 9. 性能影响（如适用）

- [ ] 已跑 criterion bench；性能数字附后
- [ ] 无显著性能影响（< 5% 变化）
- [ ] 性能提升 ~X%（场景 `<XXX>`，对比 `<YYY>`）
- [ ] 性能下降 ~X%（场景 `<XXX>`，对比 `<YYY>`，**需架构师 review**）

## 10. 关联 Issue / Discussion

- Closes #XXX
- Discussed in #XXX
- 关联集成方反馈：`<partner> ticket #XXX`

## 11. Checklist

> Reviewer 必查项（详见 `38_code_review_checklist.md`）：

- [ ] 标题规范
- [ ] 描述完整，关联 ID 已填
- [ ] 改动点明确
- [ ] 未触及禁用项（或已说明）
- [ ] `unsafe` 块均有 `// SAFETY:`
- [ ] 单元 + 集成测试齐全
- [ ] CI 全绿
- [ ] 文档同步
- [ ] 无 doc drift

## 12. 风险与回滚

<!-- 如有非平凡风险，说明回滚方式。 -->

## 13. 截图 / 视频（如有 UI / 集成方 demo）

<!-- 可选 -->

---

**Reviewer 签字**：

- 至少 1 名 reviewer（普通 PR）/ 2 名 reviewer（涉及 `unsafe` / 公开 API / `Cargo.toml` 依赖）
- 架构师签字：触及 `PROHIBIT-*` / `AC-XX` 时强制
- Release Manager 签字：破坏 ABI 时强制
