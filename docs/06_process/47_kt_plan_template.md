# 知识移交计划模板（Knowledge Transfer Plan Template）

> **用途**：项目或 Phase 收尾、维护团队接手的知识移交计划。
> **对应工作流步骤**：
> - 149 ナレッジ移管（KT）→ `28_workflow.md` §11.53
> - 146 成果物引渡し（成果物交接）→ §11.50（KT 是交接的一部分）
> **关联**：`46_closure_report_template.md`（项目收尾）；`45_retrospective_template.md`（关键学习）；`28_workflow.md`（流程基线）。

## 0. KT 元数据

| 字段 | 取值 |
|---|---|
| KT 编号 | `KT-XXX` |
| 移交方 | `<原团队 / 核心维护者>` |
| 接收方 | `<新团队 / 接手维护者 / 关键人>` |
| KT 周期 | `<YYYY-MM-DD ~ YYYY-MM-DD>` |
| KT 负责人 | — |
| 关联项目 / Phase | `<Phase X>` |

## 1. KT 目标

- 新接手者能在 KT 结束后 **独立维护** GVPE（含 bug 修复、性能调优、新 feature 添加）；
- 关键知识**不**只存在某个人脑中（避免单点失败）；
- 流程 / 文档 / 工具链完整移交。

## 2. 接收方现状

| 维度 | 当前水平 | KT 后目标 |
|---|---|---|
| Rust 熟练度 | ☐ 高 ☐ 中 ☐ 低 | ☐ 高 |
| 物理引擎经验 | ☐ 高 ☐ 中 ☐ 低 | ☐ 中 |
| GVPE 文档阅读 | ☐ 已读 ☐ 部分 ☐ 未读 | ☐ 已读 |
| 既有代码贡献 | ☐ 多 ☐ 少 ☐ 无 | ☐ 可独立 PR |

## 3. KT 内容清单

### 3.1 必读文档（按优先级）

| 文档 | 优先级 | 建议阅读方式 |
|---|---|---|
| `00_vision` | 必读 | 通读 |
| `01_requirements` | 必读 | 通读 |
| `04_architecture` | 必读 | 通读 + 重点记忆 §4.4 Compiler 边界 |
| `17_detailed_design` | 必读 | 通读 + 重点 §1-§10 |
| `26_tech_selection` | 必读 | 重点 §18.13 拒绝清单 + §18 选型理由 |
| `27_qa_register` | 必读 | 重点 Blocker / High 项 |
| `28_workflow` | 必读 | 通读 + 重点 §11 流程定义 |
| `39_release_checklist` | 必读 | 通读 |
| 其他 33 份文档 | 选读 | 按角色需求 |

### 3.2 架构 tour（口头 + 屏幕共享）

| 模块 | 时长 | 讲解人 |
|---|---|---|
| 三大空间 + crate map | 1h | 架构师 |
| `gvpe-core` / `gvpe-memory` 数据结构 | 1h | 核心维护者 |
| `gvpe-collision` broad/narrow phase | 1h | 碰撞模块维护者 |
| `gvpe-solver` Sequential Impulse | 1.5h | 求解器维护者 |
| `gvpe-runtime` 帧循环 + island + scheduler | 1.5h | runtime 维护者 |
| `gvpe-ffi` C ABI + catch_unwind | 1h | FFI 维护者 |
| `gvpe-graph` / `gvpe-compiler` | 0.5h | graph 维护者（overview） |
| `gvpe-vector` | 0.5h | vector 维护者（overview） |
| 集成方典型 demo 走读 | 1h | 集成方接口 |
| `total` | **9h（约 1 个工作日 × 1-2 人）** | |

### 3.3 工具链上手

- [ ] git 仓库 clone + 权限
- [ ] `cargo build` / `cargo test` 跑通
- [ ] `cargo bench` 跑通（criterion）
- [ ] `cargo +nightly miri test` 跑通
- [ ] `cargo deny` 跑通
- [ ] CI 触发（push 一个测试 PR）
- [ ] cbindgen 本地生成头文件
- [ ] GitHub Issues / Discussions / Projects 权限
- [ ] crates.io 发布权限（如适用）
- [ ] RUSTSEC 邮件列表订阅

### 3.4 流程上手

- [ ] PR 流程（`37_pr_template.md` + `38_code_review_checklist.md`）
- [ ] Code Review 流程
- [ ] Release 流程（`39_release_checklist.md`）
- [ ] Hotfix 流程（`44_hotfix_runbook.md`）
- [ ] Change Request 流程（`42_change_request_form.md`）
- [ ] Incident / Postmortem 流程（`40_postmortem_template.md`）
- [ ] UAT 流程（与集成方）

### 3.5 关键 PR 历史

- 列出最近 20-50 个关键 PR（重大 feature、bug 修复、ABI 变更）；
- 接收方选 5-10 个通读，理解 PR 流程的实质。

### 3.6 已知坑

> 来自 `27_qa_register.md` 的 Blocker / High 项 + 集成方常见问题 + 团队历史教训。

| 坑 | 来源 | 注意事项 |
|---|---|---|
| | `27_qa_register.md` `QA-X-NN` | |
| | 集成方常见问题 | |
| | 团队 retro（`docs/retros/`） | |

## 4. KT 形式

| 形式 | 时长 | 备注 |
|---|---|---|
| 文档通读 | 1-2 周 | 自学 |
| 架构 tour | 1-2 工作日 | 屏幕共享 + 答疑 |
| 工具链上手 | 0.5-1 工作日 | 接收方实操 |
| 流程 walkthrough | 0.5 工作日 | 走一遍 PR / CR / release 流程 |
| 1 对 1 答疑 | 持续 | 移交方 + 接收方 1-2 周密集答疑 |
| 接收方首次独立 PR | KT 周期内 | 接收方独立完成 1 个非平凡 PR |
| 接收方首次 hotfix 演练 | KT 周期内 | 模拟（不真发布） |

## 5. 接收方验收

- [ ] 通读所有必读文档；
- [ ] 跑通工具链；
- [ ] 理解核心 crate 架构；
- [ ] 完成首次独立 PR（无重大 review 意见）；
- [ ] 模拟 hotfix 流程跑通；
- [ ] 能独立回应集成方技术问题。

## 6. 移交方承诺

- KT 周期内随时答疑；
- KT 结束后 4 周内对接收方的关键决策提供 review；
- 关键人保留 3 个月的"随叫随到"。

## 7. 知识沉淀

KT 过程中发现的新知识 / 文档缺口：
- [ ] 补充到 `27_qa_register.md`（如属风险 / 顾虑）；
- [ ] 补充到 `28_workflow.md`（如属流程）；
- [ ] 补充到对应 `GVPE-DOC-NN`（如属设计）；
- [ ] 补充到 `45_retrospective_template.md` 格式的 KT 报告（`docs/kt/`）。

## 8. 关联

- `28_workflow.md` §11.50 / §11.53
- `36_project_plan.md`（项目计划）
- `46_closure_report_template.md`（项目收尾）
- `45_retrospective_template.md`（关键学习）
- `27_qa_register.md`（已知坑）
- `40_postmortem_template.md`（历史教训）

## 9. 签字

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| KT 负责人 | | | |
| 移交方代表 | | | |
| 接收方代表 | | | |
