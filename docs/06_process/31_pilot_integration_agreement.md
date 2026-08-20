# 集成方 Pilot 协议模板（Integration Partner Pilot Agreement）

> **用途**：与 1-2 个集成方（Unity / Unreal / Godot / 自研引擎团队）签署的 pilot 协议模板。
> **对应工作流步骤**：`28_workflow.md` §10.7 步 73（外部システム連携試験）、§10.9 步 90-95（受入試験）。
> **目的**：在 MVP 正式发布前，用真实集成方项目验证 GVPE 在游戏引擎侧的可用性、性能、集成难度。

## 0. 协议元数据

| 字段 | 取值 |
|---|---|
| 协议编号 | `GVPE-PILOT-XXX` |
| 集成方 | `<公司 / 团队名>` |
| 集成引擎 | `<Unity 6.x / Unreal 5.x / Godot 4.x / 自研>` |
| GVPE 版本 | `<e.g. v0.1.0-rc.1>` |
| 协议起止 | `<YYYY-MM-DD> ~ <YYYY-MM-DD>` |
| 双方签字 | 见 §11 |

## 1. 范围

### 1.1 集成方负责

- 在自己的游戏项目中嵌入 GVPE MVP 库（`cdylib` + cbindgen 头文件）；
- 跑通至少 N 个真实业务场景（见 `33_typical_game_scenarios.md`）；
- 上报所有发现的 bug、性能问题、API 易用性问题；
- 提供集成 demo（视频 + 工程文件，签字后可公开脱敏版）；
- 反馈集成方对 C ABI / 文档 / 工具链的体验。

### 1.2 GVPE 负责

- 提供 MVP 库的 `cdylib` / `staticlib` 产物（按 §11.24 发布）；
- 提供 cbindgen 生成的 C 头文件；
- 提供 `gvpe-ffi` 的最小 C++ wrapper 示例（Unity / Unreal 各一个）；
- 提供集成方反馈的 issue tracking（GitHub Issues + label `pilot/<partner>`）；
- 在 pilot 期间对集成方报告的 bug 提供 24 小时内首次响应（严重 bug 8 小时内）；
- 性能调优支持（按 §3 性能目标）。

## 2. 交付物

| 里程碑 | 日期 | 交付物 | 责任方 |
|---|---|---|---|
| M0 kickoff | T+0 | 协议签字；环境就绪 | 双方 |
| M1 first build | T+2 周 | 集成方跑通最小 demo（1 个 body + 1 step） | 集成方 |
| M2 scenarios | T+6 周 | 跑通 §1.1 中的 N 个场景 | 集成方 |
| M3 performance | T+8 周 | 性能数字（按 §3 表格） | 双方 |
| M4 sign-off | T+10 周 | pilot 报告 + 验收签字 | 双方 |

## 3. 性能目标

| 场景 | 集成方机器规格 | 目标 |
|---|---|---|
| 100 动态 rigid body | 中端游戏 PC | ≥ 60 Hz |
| 1000 动态 rigid body | 高端游戏 PC | ≥ 60 Hz |
| 10000 动态 rigid body（broad phase 剪枝后） | 高端游戏 PC | ≥ 30 Hz |
| 关节驱动角色（10 joints） | 中端 PC | ≥ 60 Hz |
| 高速物体（CCD 触发） | 中端 PC | 触发后无穿透 / 数值稳定 |
| 集成方自定义场景 1 | 集成方机器 | 集成方预期值（签字时确认） |

注：以上数字与 `GVPE-DOC-14`（性能预算）保持一致。

## 4. 报告与 issue 流程

### 4.1 Issue 报告

- 渠道：GitHub Issues（label `pilot/<partner>`）；
- 必填字段：集成引擎版本、GVPE 版本、场景描述、复现步骤、期望 vs 实际、性能数字（若有）、日志 / dump。

### 4.2 严重度分级

| 严重度 | 响应 SLA | 修复 SLA |
|---|---|---|
| Critical（崩溃 / 数据损坏） | 8 小时 | 48 小时（hotfix） |
| High（核心功能失效） | 24 小时 | 下个 release |
| Medium（边缘 case） | 1 周 | 下个 minor |
| Low（文档 / 优化） | 1 周 | backlog |

## 5. 知识产权

- 集成方项目代码：归集成方所有；
- GVPE 库：归 GVPE 项目方所有（依 `LICENSE`）；
- pilot 报告：双方共有；公开版本须双方同意；
- bug fix / 性能改进：默认进入 GVPE 主仓库（MIT / Apache-2.0 双协议）；集成方特殊需求可协商。

## 6. 保密

- 集成方游戏项目代码 / 美术资源**不**进入 GVPE 仓库；
- 集成方报告的具体性能数字**不**在未授权情况下公开；
- GVPE 未发布版本的源码**不**对集成方开放（除已发布版本外）；
- 保密期：协议终止后 2 年。

## 7. 终止

- 任一方可提前 30 天书面通知终止；
- 终止后双方义务：归还机密材料、删除本地副本（除已公开部分）；
- 已交付的 demo / 报告归双方共有，但**不**强制公开。

## 8. 免责

- GVPE MVP 阶段**不**承诺生产可用性；
- 集成方在自己的测试环境使用，不承担 GVPE 对集成方业务的影响责任；
- 具体免责条款按双方商务协议附录。

## 9. 集成方反馈登记表

> 由集成方填写，GVPE 收集。

| 反馈日期 | 类别 | 描述 | GVPE 响应 | 状态 |
|---|---|---|---|---|
| | | | | |
| | | | | |

## 10. 关联

- `28_workflow.md` §10.7 步 73、§10.9 步 90-95
- `33_typical_game_scenarios.md`（典型游戏场景库）
- `34_uat_plan_template.md`、`35_uat_spec_template.md`
- `39_release_checklist.md`（MVP release 流程）

## 11. 签字

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| GVPE 项目方 | | | |
| 集成方代表 | | | |
| 集成方技术负责人 | | | |
