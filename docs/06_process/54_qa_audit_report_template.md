# 质量审计报告模板（QA Audit Report Template）

> **用途**：半年度 / 年度 QA 审计报告；针对项目整体的文档、流程、代码质量审计。
> **对应工作流步骤**：
> - 130 品質監査（QA 审计）→ `28_workflow.md` §11.40
> - 128 品質レビュー（QA Review）→ §11.38
> - 129 品質評価（QA 评价）→ §11.39
> **关联**：`53_test_summary_report_template.md`（单次测试总结）；`27_qa_register.md`（QA 状态）；`28_workflow.md` §13（持续改进）；`45_retrospective_template.md`（季度回顾）。

## 0. 审计元数据

| 字段 | 取值 |
|---|---|
| 审计编号 | `AUDIT-XXX` |
| 审计类型 | ☐ 半年度 ☐ 年度 ☐ 专项 |
| 审计周期 | `<YYYY-MM-DD ~ YYYY-MM-DD>` |
| 审计人 | 架构师 + 1 名外部 reviewer（如有） |
| 报告日期 | `<YYYY-MM-DD>` |

## 1. 审计范围

- [ ] 文档完整性 / 一致性
- [ ] 流程执行记录（PR / CR / release / incident）
- [ ] 代码质量（clippy / coverage / miri / cargo deny）
- [ ] 安全 / 许可证（`cargo audit` / RUSTSEC）
- [ ] 性能 baseline 对比
- [ ] 测试覆盖
- [ ] 集成方反馈趋势
- [ ] 团队健康度

## 2. 文档审计

### 2.1 文档清单核对

| 文档 | 应有 | 实际 | 缺失 / 偏差 |
|---|---|---|---|
| `docs/00` ~ `docs/40` | 41 | ___ | |
| 修订历史 | 每份 ≥ v0.1 | | |
| 元数据表 | 每份齐全 | | |
| 审批签字 | 每份留位 | | |

### 2.2 文档与代码同步

- [ ] 最近 1 个 Phase 的所有 PR 对应文档已更新；
- [ ] `27_qa_register.md` Blocker / High 项已落实到对应文档；
- [ ] 无 doc drift（抽样检查 ≥ 5 份文档 vs 代码）。

### 2.3 ID 引用一致性

- [ ] `GVPE-FR-XXX` / `GVPE-NFR-XXX` / `R-PROHIBIT-XX` / `AC-XX` 等 ID 跨文档引用一致；
- [ ] `GVPE-DOC-NN §X.Y` 引用准确；
- [ ] 章节引用 `§X.Y` 准确。

## 3. 流程审计

### 3.1 PR 流程

| 指标 | 上次审计 | 本次审计 | 趋势 |
|---|---|---|---|
| 总 PR 数 | | | |
| 拒绝率 | | | |
| 平均 cycle time | | | |
| `unsafe` PR 数 | | | |
| 涉及 ABI 变更 PR 数 | | | |
| PR template 使用率 | | | |

### 3.2 CR 流程

- `42_change_request_form.md` 使用率；
- 平均 CR 处理时间；
- CR 拒绝 / 推迟比例。

### 3.3 Release 流程

- 上次 release 距今：___；
- 是否有 release 事故；
- Hypercare 反馈。

### 3.4 Incident / Postmortem 流程

- incident 数；
- postmortem 完成率；
- 改进措施落地率。

## 4. 代码质量审计

| 指标 | 目标 | 实测 | 通过 |
|---|---|---|---|
| 行覆盖率 | ≥ 80% | ___% | ☐ |
| 分支覆盖率 | ≥ 70% | ___% | ☐ |
| `cargo clippy` warning | 0 | ___ | ☐ |
| `unsafe` 块总数 | （趋势） | ___ | |
| `unsafe` 块 miri 覆盖 | 100% | ___% | ☐ |
| `cargo audit` High / Critical | 0 | ___ | ☐ |
| `cargo deny` violation | 0 | ___ | ☐ |
| hot patch ratio | （趋势） | ___% | |

## 5. 性能审计

| 指标 | 上次审计 | 本次审计 | 趋势 |
|---|---|---|---|
| `14_performance_budget.md` 目标达成 | | | |
| 性能回归（vs 上 release） | | | |
| 集成方性能反馈 | | | |

## 6. 安全审计

- [ ] RUSTSEC 公告处理率（24h 内）：___%；
- [ ] GitHub Dependabot PR 处理率：___%；
- [ ] `SECURITY.md` 维护状态：☐ 现行 / ☐ 滞后；
- [ ] 集成方安全报告处理 SLA 达成：___%。

## 7. 集成方健康度

| 集成方 | 反馈频次 | 满意度 | 关键反馈 |
|---|---|---|---|
| Unity | | | |
| Unreal | | | |
| Godot | | | |

## 8. 团队健康度

| 维度 | 评估 | 备注 |
|---|---|---|
| 士气 | ☐ 好 ☐ 一般 ☐ 不足 | |
| 沟通效率 | ☐ 好 ☐ 一般 ☐ 不足 | |
| 知识共享 | ☐ 好 ☐ 一般 ☐ 不足 | |
| 决策效率 | ☐ 好 ☐ 一般 ☐ 不足 | |
| 范围纪律 | ☐ 好 ☐ 一般 ☐ 不足 | |
| 文档维护负担 | （评估） | |
| 加班 / 倦怠迹象 | ☐ 无 ☐ 有 | |

## 9. 审计发现

### 9.1 优势（Keep）

1. ...
2. ...

### 9.2 问题（Problem / 风险）

| ID | 描述 | 严重度 | 建议 |
|---|---|---|---|
| FIND-01 | ... | ☐ Blocker ☐ High ☐ M ☐ L | ... |
| FIND-02 | ... | | |
| ... | | | |

### 9.3 改进建议（Try）

| ID | 建议 | 优先级 | 关联 |
|---|---|---|---|
| REC-01 | ... | ☐ H ☐ M ☐ L | |
| REC-02 | ... | | |
| ... | | | |

## 10. 行动项

| ID | 行动 | 优先级 | 责任人 | Deadline | 关联 |
|---|---|---|---|---|---|
| AI-01 | ... | ☐ H ☐ M ☐ L | | | `27_qa_register.md` `QA-X-NN` / `28_workflow.md` §X.Y / `36_project_plan.md` |
| AI-02 | ... | | | | |
| ... | | | | | |

## 11. 关联

- `28_workflow.md` §11.38-§11.40 / §13
- `27_qa_register.md`（QA 状态）
- `45_retrospective_template.md`（季度回顾）
- `53_test_summary_report_template.md`（单次测试）
- `36_project_plan.md`（项目计划）
- `50_status_report_template.md`（定期状态）

## 12. 签字

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 审计人（内部） | | | |
| 审计人（外部，如适用） | | | |
| 架构师 | | | |
| 项目负责人 | | | |
