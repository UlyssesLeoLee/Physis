# 安全公告响应程序（Security Advisory Response Procedure）

> **用途**：依赖库 RUSTSEC 公告 / 自家代码 CVE / 集成方安全报告的标准响应流程。
> **对应工作流步骤**：
> - 123 脆弱性対応（脆弱性对应）→ `28_workflow.md` §11.35
> - 114 / 115 / 116 事件 / 障害 / 问题管理 → §11.27-§11.29
> **关联**：`44_hotfix_runbook.md`（严重时触发）；`40_postmortem_template.md`（事后复盘）；`42_change_request_form.md`（CR 记录）；`27_qa_register.md`（风险登记）。

## 0. 适用范围

- **依赖库公告**：RustSec Advisory Database（`rustsec/advisory-db`）；
- **自家代码 CVE**：自检 / 集成方上报 / 外部 researcher；
- **集成方安全报告**：集成方团队在自家产品中发现 GVPE 引起的安全问题；
- **工具链公告**：cargo / rustc / GitHub Actions 的安全公告。

## 1. 检测与订阅

| 来源 | 订阅方式 | 频率 |
|---|---|---|
| RUSTSEC 公告 | GitHub Watch `rustsec/advisory-db` + 邮件列表 | 实时 |
| `cargo audit` | CI 每日跑 | 每日 |
| GitHub Dependabot | GitHub 仓库自动开启 | 实时 |
| 集成方报告 | 邮件 / 群 | 实时 |
| 外部 researcher | `SECURITY.md` 入口 | 实时 |

## 2. 严重度分级

| 级别 | 含义 | GVPE 影响判断 | 响应 SLA |
|---|---|---|---|
| **Critical** | 远程可利用 / 默认配置即触发 / 已有 exploit | 立即 hotfix | 24h |
| **High** | 本地可利用 / 默认配置不触发但易达成 | 优先 hotfix 或下个 patch | 1 周 |
| **Medium** | 难达成 / 需特定配置 | 下个 patch | 1 月 |
| **Low** | 理论性 / 无 PoC | backlog | 1 季 |

## 3. 响应流程

### 3.1 Critical / High

| 时间 | 角色 | 动作 |
|---|---|---|
| 0-2h | 核心维护者 | 确认公告 + 评估影响（是否触及本项目） |
| 0-4h | 架构师 + Release Manager | 决策：hotfix / patch / 接受 / 推迟 |
| 4-12h | 修复人 | 升级依赖 / 应用 patch / 加固代码 |
| 12-24h | Reviewer | Code Review |
| 24-48h | Release Manager | 出 release |
| 48-72h | 集成方接口 | 通知所有已知集成方 |

### 3.2 Medium / Low

- 记录到 `27_qa_register.md` §9（QA 风险登记）；
- 在下次 patch / minor 时处理；
- 不单独 release。

## 4. 升级 / 加固方案

| 公告类型 | 响应 |
|---|---|
| 依赖库新版本修复 | `cargo update -p <crate>` → 测试 → 升级 PR |
| 依赖库无修复 | 评估：fork 修复 / 替换依赖 / 临时 workaround |
| 自家代码 | 修复 + 加固 + 增加回归测试 |
| 工具链公告 | 升级 toolchain / 临时规避 |

## 5. 披露

### 5.1 GitHub Security Advisory

- 严重度 ≥ Medium：在 GitHub Security Advisories 创建；
- 标题、描述、影响范围、CVSS 评分（如适用）、修复版本、致谢。

### 5.2 集成方通知

- 邮件 + 群通知；
- 通知内容：受影响版本、风险描述、修复版本、升级建议、致谢（如适用）；
- 受限披露（coordinated disclosure）：与 researcher 协商披露时间。

### 5.3 公开公告

- 严重度 ≥ High：GitHub Releases / Discussion / blog；
- 内容：背景 / 根因 / 影响范围 / 修复 / 致谢 / 后续改进。

## 6. CVE 申请

- 严重度 ≥ Medium：申请 CVE（GitHub 自动 / MITRE）；
- 由架构师或 Release Manager 负责。

## 7. 复盘

- 任何 Critical / High 事件：触发 `40_postmortem_template.md`；
- 改进措施登记 `27_qa_register.md`；
- 流程改进登记 `28_workflow.md` §13。

## 8. 预防机制

- [ ] `cargo audit` 在 CI 强制（每日）；
- [ ] `cargo deny` 在 CI 强制（每次 PR）；
- [ ] Dependabot PR 由架构师 review；
- [ ] 季度审视：依赖库更新节奏、公告处理时长；
- [ ] `SECURITY.md` 维护（披露政策 + contact）。

## 9. 关联

- `28_workflow.md` §11.27-§11.29 / §11.35
- `40_postmortem_template.md`
- `42_change_request_form.md`
- `44_hotfix_runbook.md`
- `27_qa_register.md`（风险登记）
- `39_release_checklist.md`（release 流程）

## 10. 签字

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 公告处理人 | | | |
| 架构师 | | | |
| Release Manager | | | |
