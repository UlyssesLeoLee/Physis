# Hotfix Runbook（紧急修复操作手册）

> **用途**：发布后严重问题的紧急修复操作手册。**不**是 bug 修复常规流程（PR 流程见 `37_pr_template.md`）。
> **对应工作流步骤**：
> - 125 緊急改修（Hotfix）→ `28_workflow.md` §11.37
> - 106 / 108 发布后稼动确认 / Hypercare 触发 → §11.25 / §11.26
> **关联**：`39_release_checklist.md`（release 流程）；`40_postmortem_template.md`（事后复盘）；`42_change_request_form.md`（变更记录）；`28_workflow.md` §11.25.1（发布失败应急）。

## 0. Hotfix 触发条件

**至少满足其一**：

- Critical 严重 bug（崩溃 / 数据损坏 / 安全漏洞）；
- High 严重 bug 影响核心功能且无法用 workaround；
- 性能 / 稳定性问题导致集成方无法继续使用。

**不**适用于：
- 边缘 case（走常规 PR 流程）；
- 新功能需求（走 `42_change_request_form.md`）；
- 文档问题（走文档 PR 流程）。

## 1. 决策流程（24 小时内完成）

| 时间 | 角色 | 动作 |
|---|---|---|
| 0-2h | 集成方 / 用户 | 报告问题（GitHub Issue `severity: critical`） |
| 0-4h | 核心 crate 维护者 | 确认问题 + 初步根因 |
| 4-8h | 架构师 + Release Manager | 决策：是否 hotfix / 是否撤回 release |
| 8-12h | 修复人 | 最小修复 + 测试 + review |
| 12-18h | 架构师 + Reviewer | Code Review |
| 18-24h | Release Manager | 出 hotfix release |
| 24h+ | 集成方接口 | 通知集成方 + 验证 |

## 2. 分支策略

### 2.1 从最新 release tag checkout

```bash
git fetch --tags
git checkout vX.Y.Z            # 最新 release
git checkout -b hotfix/vX.Y.Z+1
```

### 2.2 修复

- 最小变更（**只**修当前问题，不顺手重构）；
- 修复 commit 标题：`hotfix(vX.Y.Z+1): <简明>`；
- 修复 commit body：根因 + 影响的范围 + 关联 issue。

### 2.3 测试

- [ ] 单元测试（含回归测试，专门防此 bug）；
- [ ] 集成测试；
- [ ] `cargo +nightly miri test`（如触及 `unsafe`）；
- [ ] determinism harness（`R-FR-001` 行为不变）；
- [ ] 集成方 smoke（若集成方有可用环境）。

## 3. Review

- 强制 reviewer 数：≥ 2（架构师 + 1 名 crate 维护者）；
- 触发架构师 + Release Manager 双签字（见 §6）；
- Review checklist：`38_code_review_checklist.md`（含 hotfix 专项检查）；
- 特殊关注：
  - 修复**不**引入新 `unsafe`；
  - 修复**不**破坏 ABI（除非是 ABI 本身的 hotfix——需 Release Manager 特批）；
  - 修复**不**引入新依赖；
  - 修复**不**降低测试覆盖率。

## 4. 发布

### 4.1 Tag + Release

```bash
git tag -a vX.Y.Z+1 -m "Hotfix vX.Y.Z+1: <简明>"
git push origin hotfix/vX.Y.Z+1 --follow-tags
```

### 4.2 crates.io 发布

按 `39_release_checklist.md` §2.2 流程，但仅 `cargo publish` 必要的 crate（最小变更面）。

### 4.3 GitHub Release

- 标题：`vX.Y.Z+1 (hotfix) - <简明>`；
- body：包含 root cause + affected versions + fix details；
- 标记 `hotfix` label。

## 5. 通知

| 受众 | 渠道 | 时限 |
|---|---|---|
| 已知集成方 | 邮件 / 群 | 立刻（出 release 后 1h 内） |
| 公开社区 | GitHub Discussions / Release | 立刻 |
| 内部团队 | 同步会 / 邮件 | 24h 内 |

通知内容：
- 受影响版本范围；
- 根因（已知）；
- 修复要点（无需过度技术细节）；
- 升级建议；
- 致谢（如适用）。

## 6. 同步合并

```bash
git checkout main
git merge --no-ff hotfix/vX.Y.Z+1
git push origin main

git checkout develop  # 如有
git merge --no-ff hotfix/vX.Y.Z+1
git push origin develop
```

## 7. 后续

- 24h 内启动 `40_postmortem_template.md` 流程（即使修复完成）；
- postmortem 产出登记到 `27_qa_register.md`（避免再次发生）；
- 流程改进登记到 `28_workflow.md` §13（持续改进）；
- 集成方接口确认修复有效 + 关闭相关 issue。

## 8. 关联

- `28_workflow.md` §11.25.1 / §11.26 / §11.37
- `37_pr_template.md`（hotfix PR 仍用此模板）
- `38_code_review_checklist.md`（CR 清单）
- `39_release_checklist.md`（release 流程）
- `40_postmortem_template.md`（事后复盘）
- `42_change_request_form.md`（CR 记录）
- `43_hypercare_plan_template.md`（Hypercare 期 hotfix 频次作为退稳态信号）

## 9. Hotfix 签字

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| Release Manager | | | |
| 架构师 | | | |
| 修复人 | | | |
| 集成方接口（如已通知） | | | |
