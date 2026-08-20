# Hypercare 计划模板（Hypercare Plan Template）

> **用途**：发布后 4 周内的重点 follow-up 阶段计划。
> **对应工作流步骤**：108 初期流動対応（Hypercare）→ `28_workflow.md` §11.26。
> **关联**：`39_release_checklist.md`（发布后流程）；`40_postmortem_template.md`（事故复盘）；`36_project_plan.md`（项目计划）。

## 0. Hypercare 元数据

| 字段 | 取值 |
|---|---|
| 关联 release | `<vX.Y.Z>` |
| Hypercare 周期 | `<YYYY-MM-DD> ~ <YYYY-MM-DD>`（4 周） |
| Hypercare 负责人 | — |
| 核心响应人 | — |
| 集成方 | — |

## 1. 目标

- **首要**：首发版本**零 Blocker**（集成方生产可用）；
- **次要**：收集集成方反馈作为下一 minor 规划的输入；
- **关键指标**：集成方反馈响应 SLA 达成率 ≥ 95%；新 Blocker < 3。

## 2. 资源

| 角色 | 投入 |
|---|---|
| 核心 crate 维护者 | 全员 standby（24 小时内响应） |
| Release Manager | 协调 + release 决策 |
| 集成方接口 | 每日与集成方同步 |
| 文档维护者 | FAQ 实时更新 |

## 3. 响应 SLA

| 严重度 | 响应时间 | 修复时间 |
|---|---|---|
| Critical | 4 小时 | 24 小时（hotfix） |
| High | 8 小时 | 72 小时（hotfix 或 patch） |
| Medium | 24 小时 | 下个 patch |
| Low | 1 周 | 下个 minor |

## 4. 集成方同步

| 频率 | 形式 | 内容 |
|---|---|---|
| 每日 | 异步 issue / GitHub | 集成方报告跟踪 |
| 每周 | 同步会（30 min） | 当前状态、问题、计划 |
| 关键节点 | 邮件 | 性能 / bug 趋势报告 |

## 5. 监控指标

| 指标 | 来源 | 频率 |
|---|---|---|
| GitHub Issues 新增数 | GitHub | 实时 |
| crates.io 下载量 | crates.io | 每日 |
| 集成方反馈 | 邮件 / 群 | 实时 |
| 性能回归（如有 telemetry） | 集成方 | 每周 |
| panic / error 报告 | 集成方 / GitHub | 实时 |

## 6. 紧急响应流程

### 6.1 Critical 事件

1. 4 小时内核心 crate 维护者确认；
2. 24 小时内 hotfix 修复（`28_workflow.md` §11.37 + `44_hotfix_runbook.md`）；
3. 集成方实时同步；
4. 必要时撤回 release（crates.io yank）。

### 6.2 性能 / 稳定性问题

1. 8 小时内确认问题；
2. 性能 baseline 对比（与上 release）；
3. 72 小时内修复或临时 workaround；
4. release patch。

## 7. 知识库更新

- 集成方高频问题 → `docs/06_process/FAQ.md`（或对应文档）；
- 性能调优经验 → `14_performance_budget.md` 附录；
- API 改进点 → `17_detailed_design.md` 修订；
- 流程改进点 → `28_workflow.md` 修订 + `27_qa_register.md` 登记。

## 8. 阶段性产出

| 周次 | 产出 |
|---|---|
| W1 | 集成方 smoke 报告（48 小时内）；首批 issue 整理；FAQ v0.1 |
| W2 | 性能 / 稳定性基线确认；首批 hotfix（如必要） |
| W3 | 集成方反馈趋势报告；下一 minor 规划草案 |
| W4 | Hypercare retrospective（`45_retrospective_template.md`）；进入稳态 |

## 9. 退出条件

- 集成方反馈频率稳定（< 1 件 / 周）；
- 已知 Blocker 全部 Closed 或转 deferred；
- 下一 minor / patch 规划已就位；
- 核心团队回归正常 on-call 节奏。

## 10. 关联

- `28_workflow.md` §11.26、§11.25
- `39_release_checklist.md` §3-§4
- `40_postmortem_template.md`（如发生 Critical 事件）
- `44_hotfix_runbook.md`
- `45_retrospective_template.md`（W4 retrospective）
- `36_project_plan.md` §8（Phase 边界）

## 11. 签字

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| Hypercare 负责人 | | | |
| Release Manager | | | |
| 项目负责人 | | | |
