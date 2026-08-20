# 验收测试计划模板（UAT Plan Template）

> **用途**：与集成方协商的 UAT 计划，命名 `uat_plan_<partner>.md`。
> **对应工作流步骤**：`28_workflow.md` §10.9 步 90-95（受入試験計画 / 仕様書 / テスト / 判定 / 検収）。
> **关联**：`31_pilot_integration_agreement.md`（pilot 协议）、`32_system_test_spec_template.md`（ST 规格）、`33_typical_game_scenarios.md`（场景库）、`35_uat_spec_template.md`（UAT 规格）。

## 0. 元数据

| 字段 | 取值 |
|---|---|
| 协议编号 | `GVPE-UAT-XXX` |
| 集成方 | `<公司 / 团队名>` |
| 集成引擎 | `<Unity / Unreal / Godot / 自研>` |
| GVPE 版本 | `<vX.Y.Z>` |
| 计划起止 | `<YYYY-MM-DD> ~ <YYYY-MM-DD>` |
| 双方签字 | 见 §10 |

## 1. 范围与目标

### 1.1 集成方负责

- 在自己的游戏项目生产环境（开发分支或预发布分支）嵌入 GVPE；
- 跑通 §3 中的 UAT 场景；
- 出具书面验收报告；
- 接受 / 拒绝签字。

### 1.2 GVPE 负责

- 提供稳定 release 的 `cdylib` / `staticlib` 产物 + cbindgen 头文件；
- 提供集成方报告的 issue 处理（依 `31_pilot_integration_agreement.md` §4 SLA）；
- 提供 release notes（与 `39_release_checklist.md` 协同）；
- 配合集成方做性能调优（如需要）；
- 出具 UAT 完成证书。

## 2. 验收标准（Acceptance Criteria）

| 维度 | 标准 |
|---|---|
| 功能完整性 | 所有 `GVPE-FR-*` 至少在 1 个场景中验证通过 |
| 性能 | 满足 §3 性能目标 |
| 稳定性 | 24 小时长跑无内存泄漏、无 panic |
| 兼容性 | 集成方目标平台 × 集成引擎版本全部通过 |
| 许可证 | `cargo deny` 0 violation；集成方可商用 |
| 文档 | 集成方能在不联系 GVPE 团队的情况下完成集成 |
| 支持 | pilot 期间 issue 响应 SLA 达标 |

## 3. UAT 场景与性能目标

> 来自 `33_typical_game_scenarios.md` 的 L1 + L2 必跑；集成方可加 L3 自定义场景。

| 场景 ID | 场景 | 来源 | 性能目标 | 通过 |
|---|---|---|---|---|
| UAT-01 | Box stack 10 | §33.2.2 | 60 Hz 中端 PC | ☐ |
| UAT-02 | Sphere pile 500 | §33.2.3 | 60 Hz 中端 PC | ☐ |
| UAT-03 | Jointed pendulum | §33.2.4 | 60 Hz 中端 PC | ☐ |
| UAT-04 | Sleeping validation | §33.2.5 | 60 Hz 中端 PC | ☐ |
| UAT-05 | Tower of Pisa | §33.3.1 | 60 Hz 中端 PC | ☐ |
| UAT-06 | Bullet through wall（CCD） | §33.3.2 | 60 Hz 中端 PC | ☐ |
| UAT-07 | Newton cradle | §33.3.5 | 60 Hz 中端 PC | ☐ |
| UAT-08 | 1 万 body 压力 | §33.3.7 | 30 Hz 高端 PC | ☐ |
| UAT-09 | 集成方自定义场景 1 | `<TBD>` | `<TBD>` | ☐ |
| UAT-10 | 集成方自定义场景 2 | `<TBD>` | `<TBD>` | ☐ |
| UAT-11 | 24h 长跑稳定性 | §33.3.6 | 内存增长 < 5% | ☐ |
| UAT-12 | feature-gate 验证 | `R-FR-001` | bit-for-bit identical | ☐ |

## 4. 测试环境

| 维度 | 集成方指定 |
|---|---|
| 硬件 | `<CPU / GPU / 内存>` |
| 操作系统 | `<Win11 / macOS / Linux>` |
| 集成引擎版本 | `<e.g. Unity 6.0.23>` |
| GVPE 版本 | `<vX.Y.Z>` |
| 编译器 | `<rustc 1.7X.Y>` |
| 其他 | `<TBD>` |

## 5. 测试流程

| 阶段 | 日期 | 动作 | 产出 |
|---|---|---|---|
| 准备 | T-1 周 | 环境就绪、库部署 | 环境就绪确认 |
| Dry run | T+0 | 跑通最小 demo | Dry run 报告 |
| 正式 UAT | T+1 ~ T+6 周 | 跑 §3 所有场景 | 每个场景的 pass / fail |
| 性能复测 | T+6 周 | 性能数字签字 | 性能报告 |
| 签字 | T+7 周 | 验收 / 拒绝 | UAT 证书 |

## 6. 问题管理

- 渠道：GitHub Issues（label `uat/<partner>`）；
- 严重度：依 `31_pilot_integration_agreement.md` §4；
- UAT 期间所有 Blocker 必须解决才能接受。

## 7. 验收条件

- [ ] §3 所有 ✅ 项全部通过；
- [ ] 所有 Blocker 解决或经双方同意降级；
- [ ] 性能数字达成；
- [ ] 集成方书面接受；
- [ ] release notes 经集成方 review。

## 8. 验收结果

| 维度 | 结论 | 备注 |
|---|---|---|
| 功能 | ☐ 通过 / ☐ 部分通过 / ☐ 拒绝 | |
| 性能 | ☐ 达成 / ☐ 部分达成 / ☐ 未达成 | |
| 稳定性 | ☐ 通过 / ☐ 部分通过 / ☐ 拒绝 | |
| 文档 | ☐ 充分 / ☐ 基本可 / ☐ 不足 | |
| 综合 | ☐ 接受 / ☐ 有条件接受 / ☐ 拒绝 | |

## 9. 关联

- `28_workflow.md` §10.9 步 90-95、§11.18-§11.21
- `31_pilot_integration_agreement.md`
- `32_system_test_spec_template.md`
- `33_typical_game_scenarios.md`
- `35_uat_spec_template.md`
- `39_release_checklist.md`

## 10. 签字

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| GVPE Release Manager | | | |
| 集成方项目负责人 | | | |
| 集成方技术负责人 | | | |
| 集成方 QA 负责人 | | | |
