# Release Checklist（发布检查清单）

> **用途**：每次 release（patch / minor / major）前由 Release Manager 逐项检查的清单。
> **对应工作流步骤**：`28_workflow.md` §10.11 步 102-108（リリース計画 / 判定 / デプロイ / 稼働確認 / Hypercare）、§11.22-§11.26。
> **关联**：`31_pilot_integration_agreement.md`（pilot 协议）、`34_uat_plan_template.md`（UAT 计划）、`36_project_plan.md`（项目计划）。

## 0. Release 元数据

| 字段 | 取值 |
|---|---|
| Release 版本 | `<vX.Y.Z>` |
| Release 类型 | ☐ Major ☐ Minor ☐ Patch ☐ Hotfix |
| Release Manager | — |
| 目标发布日 | `<YYYY-MM-DD>` |
| 关联 milestone | `36_project_plan.md` §3 的 M? |

## 1. Pre-release（发布前 1 周）

### 1.1 设计与文档

- [ ] 28 份基线文档与代码同步（无 doc drift）；
- [ ] 本 release 引入的所有设计变更已记录到对应 `GVPE-DOC-NN` 修订历史；
- [ ] `27_qa_register.md` Blocker 全部 Closed 或显式 Deferred；
- [ ] `CHANGELOG.md` 已起草，含：新增功能 / 修复 bug / 性能改进 / Breaking Change（如有）/ 已知问题。

### 1.2 代码质量

- [ ] `cargo fmt --all -- --check` 通过；
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` 通过；
- [ ] `cargo test --all-features` 全绿；
- [ ] `cargo test --no-default-features` 全绿（`R-FR-001`）；
- [ ] `cargo test --no-default-features --features simd-only` 全绿（如适用）；
- [ ] `cargo +nightly miri test` 在核心 unsafe crate 通过；
- [ ] `cargo bench` 与上 release 对比，无显著回归（< 5%）；
- [ ] `cargo audit` / `cargo deny` 0 violation。

### 1.3 性能

- [ ] `32_system_test_spec_template.md` §4-§5 性能 / 负荷目标达成；
- [ ] `33_typical_game_scenarios.md` L1 全部 ✅；
- [ ] 性能回归报告（与上一 release 对比）已出具。

### 1.4 安全 / 许可证

- [ ] `cargo deny check licenses` 通过；
- [ ] `cargo deny check bans` 通过（无新增 `26_tech_selection.md` §18.13 拒绝项）；
- [ ] `cargo deny check advisories` 通过（无未处理 High / Critical）；
- [ ] RUSTSEC 邮件列表已检查（无新公告影响本 release）；
- [ ] 集成方 pilot 期间报告的所有 Blocker 已修复或显式降级。

### 1.5 依赖 / ABI

- [ ] `cargo tree` 验证 `AC-02` 仍成立（核心 crate 不依赖 Graph / Vector / Compiler / Inference / 3DGS）；
- [ ] ABI 兼容：本 release 不破坏 ABI（除非 major release，且已通知集成方）；
- [ ] 新增依赖在 `16_dependency_license.md` 矩阵通过审查。

### 1.6 测试报告

- [ ] `32_system_test_spec_template.md` 全章节通过；
- [ ] `34_uat_plan_template.md` 全部 ✅ 项通过（若有 pilot / UAT）；
- [ ] determinism harness 在 `--no-default-features` 下行为不变（`AC-03`）。

## 2. Release day（发布日）

### 2.1 制品构建

- [ ] 仓库打 tag：`git tag -a vX.Y.Z -m "Release vX.Y.Z"`；
- [ ] `cargo build --release --target <triple>` 在所有支持平台通过；
- [ ] `cargo build --release -p gvpe-ffi` 产出 `cdylib`；
- [ ] `cargo build --release -p gvpe-ffi` 产出 `staticlib`（如启用）；
- [ ] `cbindgen` 生成 C 头文件，提交至 `gvpe-ffi/include/`，随 tag 一起发布；
- [ ] 校验 `cbindgen` 头文件与上一 release 的 ABI 差异（major release 必查，minor / patch 抽查）；
- [ ] 制品大小与历史趋势对比（异常大 → 排查）。

### 2.2 发布渠道

- [ ] `cargo publish` 所有 `gvpe-*` crate（按依赖顺序：先 `gvpe-math` → ... → `gvpe-ffi`）；
- [ ] crates.io 版本号正确；
- [ ] GitHub Release 创建：标题、tag、body（= CHANGELOG 节选）、artifact 上传；
- [ ] GitHub Release artifact 包含：
  - `gvpe-ffi` 的 `cdylib`（per platform）
  - `gvpe-ffi` 的 `staticlib`（per platform, 如启用）
  - cbindgen 头文件
  - SHA256SUMS

### 2.3 通知

- [ ] 集成方接口（依 `36_project_plan.md` §6）：邮件 / 群通知发布；
- [ ] 集成方通知包含：版本号、CHANGELOG 链接、release artifact URL、关键变更 / Breaking Change 摘要；
- [ ] GitHub Discussions 公告（如有活跃社区）；
- [ ] 内部团队同步：release 完成 + 已知 issue 列表。

## 3. Post-release（发布后 48 小时内）

### 3.1 稼动确认（106 步）

- [ ] 集成方 smoke（依 `28_workflow.md` §11.25）：每个集成方在 48 小时内跑通最小场景；
- [ ] 内部 smoke：本地跑 `32_system_test_spec_template.md` §2 功能测试；
- [ ] 监控：crates.io 下载量、GitHub Issues 新增、Discord / Discussions 反馈。

### 3.2 失败处理

- [ ] 集成方报告 Blocker：触发 `28_workflow.md` §11.37 hotfix 流程；
- [ ] 自身发现严重问题：撤回 release（crates.io yank） + 立即 hotfix。

## 4. Hypercare（108 步，发布后 4 周）

- [ ] 集成方反馈 24 小时内首次响应；
- [ ] 紧急 bug 优先 hotfix（依 `28_workflow.md` §11.37）；
- [ ] 性能调优支持（依 `31_pilot_integration_agreement.md`）；
- [ ] 文档 / FAQ 补全（基于集成方高频问题）；
- [ ] 4 周末：依 `28_workflow.md` §11.52 写 hypercare retrospective，归入下一 release 规划。

## 5. Hotfix 专项（major / minor 不适用）

- [ ] 触发条件：发布后 Blocker / Critical 严重 bug；
- [ ] 分支：`hotfix/vX.Y.Z+1` 从最新 release tag checkout；
- [ ] 修复 + 紧急 review（架构师 + Release Manager）；
- [ ] 紧急 release（patch bump）；
- [ ] 同步合并到 `main` 和 `develop`；
- [ ] 通知所有已知集成方。

## 6. 关联

- `28_workflow.md` §10.11 步 102-108、§11.22-§11.26
- `31_pilot_integration_agreement.md`（pilot 协议）
- `32_system_test_spec_template.md`（ST 规格）
- `34_uat_plan_template.md`（UAT 计划）
- `35_uat_spec_template.md`（UAT 规格）
- `36_project_plan.md`（项目计划）
- `37_pr_template.md`（PR 模板）
- `38_code_review_checklist.md`（CR 清单）
- `40_postmortem_template.md`（事故复盘）

## 7. Release 签字

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| Release Manager | | | |
| 架构师 | | | |
| QA 负责人 | | | |
| 项目负责人 | | | |
