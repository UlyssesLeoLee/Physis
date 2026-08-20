# 状态报告模板（Status Report Template）

> **用途**：周 / 月 / 季度的项目状态报告。
> **对应工作流步骤**：
> - 133 進捗管理（进度管理）→ `28_workflow.md` §11.42
> - 140 会議・報告（会议 / 报告）→ §11.46
> **关联**：`36_project_plan.md`（项目计划）；`27_qa_register.md`（风险 / QA 状态）；`45_retrospective_template.md`（季度回顾）；`49_meeting_notes_template.md`（单次会议）。

## 0. 报告元数据

| 字段 | 取值 |
|---|---|
| 报告周期 | ☐ 周报 ☐ 月报 ☐ 季度报 ☐ 阶段报 |
| 报告编号 | `REPORT-XXX` |
| 覆盖周期 | `<YYYY-MM-DD ~ YYYY-MM-DD>` |
| 报告人 | — |
| 报告日期 | `<YYYY-MM-DD>` |
| 分发范围 | `<核心团队 / 全员 / 集成方 / Stakeholder>` |

## 1. TL;DR

<!-- 3-5 句话：本期最关键的事（完成 / 风险 / 决策）。 -->

## 2. 关键指标

| 指标 | 计划 | 实际 | 趋势 | 备注 |
|---|---|---|---|---|
| 关键里程碑达成率 | | | ⬆️ / ➡️ / ⬇️ | |
| 完成的工作项 | | | | |
| 开放的 Blocker 数 | 0 | | | 来自 `27_qa_register.md` |
| 开放的 High 数 | | | | |
| PR cycle time（平均） | | | | |
| `cargo deny` 0 violation | ✅ | | | |
| Test coverage | | | | |
| 集成方 pilot 状态 | ☐ 进行中 ☐ 满意 ☐ 待跟进 | | | |
| 文档同步性 | ☐ 同步 ☐ 滞后 | | | |

## 3. 本期完成

- ✅ ...
- ✅ ...
- ✅ ...

## 4. 本期未完成 / 推迟

- ⏸ ...
- ⏸ ...
- 原因：...

## 5. 风险 / 阻塞

> 来自 `27_qa_register.md` 的 Blocker / High 项。

| ID | 描述 | 严重度 | 状态 | 缓解 |
|---|---|---|---|---|
| `QA-X-NN` | ... | ☐ Blocker ☐ High | ☐ Open / In-progress | ... |

## 6. 决策

| 决策 | 背景 | 决定人 | 决定日期 |
|---|---|---|---|
| ... | ... | ... | ... |

## 7. 变更

> 来自 `42_change_request_form.md` 已批准项。

| CR 编号 | 内容 | 状态 |
|---|---|---|
| `CR-XXX` | ... | ☐ 实施中 / 已发布 |

## 8. 集成方同步

| 集成方 | 状态 | 关键反馈 | 行动 |
|---|---|---|---|
| Unity | ☐ 满意 ☐ 待跟进 ☐ 阻塞 | ... | ... |
| Unreal | ... | ... | ... |
| Godot | ... | ... | ... |

## 9. 下期重点

1. ...
2. ...
3. ...

## 10. 关联

- `28_workflow.md` §11.42 / §11.46
- `27_qa_register.md`（风险 / QA 状态）
- `36_project_plan.md`（里程碑）
- `42_change_request_form.md`（CR 汇总）
- `43_hypercare_plan_template.md`（Hypercare 期）
- `45_retrospective_template.md`（季度回顾）
- `49_meeting_notes_template.md`（单次会议）

## 11. 签字

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 报告人 | | | |
| 项目负责人（review） | | | |
