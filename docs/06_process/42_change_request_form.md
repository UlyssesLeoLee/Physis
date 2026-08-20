# 变更请求单（Change Request Form, CR）

> **用途**：变更管理流程的标准表单。所有变更（含新增 feature、breaking change、依赖变更、ABI 变更）必须填写本表。
> **对应工作流步骤**：
> - 118 変更要求（CR）→ `28_workflow.md` §11.31
> - 120 / 136 変更管理（Change）→ §11.32 / §11.43
> - 119 影響分析（Impact Analysis）→ §11.32 引用
> **关联**：`27_qa_register.md`（影响分析基线）；`39_release_checklist.md`（release 关联）；`40_postmortem_template.md`（关联 postmortem 触发的变更）。

## 0. CR 元数据

| 字段 | 取值 |
|---|---|
| CR 编号 | `CR-XXX` |
| 提出日期 | `<YYYY-MM-DD>` |
| 提出人 | — |
| 关联 issue / 来源 | `#XXX` / 集成方反馈 / postmortem / 法规 / 内部 |
| 紧急度 | ☐ Critical ☐ High ☐ Medium ☐ Low |
| 范围影响 | ☐ Breaking ☐ Major ☐ Minor ☐ Patch |

## 1. 变更描述

### 1.1 动机

<!-- 为什么需要这个变更？解决了什么问题？ -->

### 1.2 提议方案

<!-- 提议的具体修改内容（API 变化、crate 拓扑变化、行为变化等）。 -->

### 1.3 替代方案

<!-- 至少 1 个替代方案 + 不选它的理由。 -->

## 2. 影响分析

### 2.1 关联需求 / 设计文档

- 关联需求 ID：`GVPE-FR-XXX` / `GVPE-NFR-XXX` / `R-PROHIBIT-XX` / `AC-XX`
- 关联设计文档：`GVPE-DOC-NN §X.Y`
- 关联 QA 登记：`QA-X-NN`

### 2.2 crate / API / ABI 影响

- [ ] 涉及 crate 列表：`<gvpe-xxx>` ...
- [ ] 涉及公开 API 变化：是 / 否
- [ ] 涉及 ABI 变化：是 / 否（**Breaking**：需 Release Manager 签字）
- [ ] 涉及 `Cargo.toml` 依赖变化：是 / 否
- [ ] 涉及 `unsafe` 新增：是 / 否
- [ ] 涉及性能 hot path：是 / 否
- [ ] 涉及安全 / 许可证：是 / 否
- [ ] 涉及文档更新：是 / 否（`GVPE-DOC-NN` 哪些节需要改）

### 2.3 集成方影响

- 已知集成方：`Unity / Unreal / Godot`（与 `31_pilot_integration_agreement.md` §10 关联）
- 通知需求：必须 / 建议 / 不需要
- 预计影响版本：`<vX.Y.Z>`

### 2.4 风险评估

| 风险 | 概率 | 影响 | 缓解措施 |
|---|---|---|---|
| ... | ☐ H ☐ M ☐ L | ☐ H ☐ M ☐ L | ... |

### 2.5 回滚方案

<!-- 描述如何回滚。 -->

## 3. 实施计划

| 阶段 | 内容 | 责任人 | 周期 |
|---|---|---|---|
| 设计 / 评审 | ... | | |
| 实现 | ... | | |
| 测试 | ... | | |
| 发布 | ... | | |

## 4. 测试与验收

- [ ] 单元测试新增 / 更新
- [ ] 集成测试新增 / 更新
- [ ] 系统测试更新
- [ ] 回归测试
- [ ] 集成方 pilot 验证（如涉及集成方）
- [ ] `cargo deny` / `cargo audit` 通过
- [ ] 性能 baseline 对比

## 5. 沟通计划

- [ ] 内部：周会 / 双周会 / 邮件
- [ ] 集成方：邮件 / 群 / release notes
- [ ] 公开：GitHub Discussions / blog

## 6. 决策

| 角色 | 决策 | 签字 | 日期 |
|---|---|---|---|
| 架构师 | ☐ 同意 ☐ 拒绝 ☐ 推迟 | | |
| Release Manager（ABI / Breaking 时） | ☐ 同意 ☐ 拒绝 | | |
| 项目负责人（Critical / High 时） | ☐ 同意 ☐ 拒绝 | | |
| 集成方代表（涉及集成方时） | ☐ 知悉 ☐ 同意 | | |

## 7. 关联

- `28_workflow.md` §11.31 / §11.32 / §11.43
- `27_qa_register.md`（影响分析 / 风险登记）
- `31_pilot_integration_agreement.md` §10
- `39_release_checklist.md`
- `40_postmortem_template.md`（如变更由事故触发）

## 8. 状态追踪

| 日期 | 状态变更 | 备注 |
|---|---|---|
| | ☐ 提交 ☐ 评审中 ☐ 实施中 ☐ 已发布 ☐ 关闭 | |
