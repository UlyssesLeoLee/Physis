# 验收证书模板（Acceptance Certificate Template）

> **用途**：UAT 通过后双方签字的验收证书。
> **对应工作流步骤**：
> - 95 検収（Acceptance）→ `28_workflow.md` §11.21
> - 94 受入判定（UAT 判定）→ §11.20
> **关联**：`34_uat_plan_template.md`（UAT 计划）；`35_uat_spec_template.md`（UAT 规格）；`46_closure_report_template.md`（项目收尾）。

## 0. 证书元数据

| 字段 | 取值 |
|---|---|
| 证书编号 | `AC-CERT-XXX` |
| 关联 UAT 协议 | `GVPE-UAT-XXX` |
| 关联 release | `<vX.Y.Z>` |
| 集成方 | `<公司 / 团队名>` |
| 集成引擎 | `<Unity / Unreal / Godot / 自研>` |
| 签发日期 | `<YYYY-MM-DD>` |

## 1. 验收范围

本证书确认以下交付物已通过验收：

- [ ] GVPE 库 vX.Y.Z（含 `cdylib` / `staticlib` 产物 + cbindgen 头文件）；
- [ ] `docs/` 全集（00-40 + archive）；
- [ ] 集成方 pilot 反馈处理记录；
- [ ] 性能数字达成（按 `34_uat_plan_template.md` §3）；
- [ ] 已知 issue 列表（依 §5）。

## 2. 验收依据

本验收基于：
- `34_uat_plan_template.md`（UAT 计划）；
- `35_uat_spec_template.md`（UAT 规格）；
- 双方签字的 UAT 报告（`GVPE-UAT-XXX §8` 验收结果）；
- pilot 协议（`31_pilot_integration_agreement.md`）。

## 3. 验收结论

| 维度 | 结论 | 备注 |
|---|---|---|
| 功能完整性 | ☐ 通过 ☐ 部分通过 ☐ 不通过 | |
| 性能 | ☐ 达成 ☐ 部分达成 ☐ 未达成 | 详见 §4 |
| 稳定性 | ☐ 通过 ☐ 不通过 | |
| 兼容性 | ☐ 通过 ☐ 不通过 | |
| 许可证 | ☐ 通过 ☐ 不通过 | |
| 文档 | ☐ 充分 ☐ 基本可 ☐ 不足 | |
| 综合 | ☐ 接受 ☐ 有条件接受 ☐ 拒绝 | |

## 4. 性能数字签字

> 引用 UAT 性能报告（`35_uat_spec_template.md` §4）。

| 场景 | 目标 | 实测 | 通过 |
|---|---|---|---|
| Box stack 10 / 60Hz | 60 Hz | ___ Hz | ☐ |
| Sphere pile 500 / 60Hz | 60 Hz | ___ Hz | ☐ |
| 1000 body full step | < 16ms | ___ ms | ☐ |
| 24h 长跑稳定性 | 内存增长 < 5% | ___% | ☐ |
| ... | | | |

## 5. 已知 issue 列表

| 编号 | 描述 | 严重度 | 处理方式 | 责任方 | Deadline |
|---|---|---|---|---|---|
| ISSUE-01 | ... | ☐ H ☐ M ☐ L | ☐ 立即修 ☐ 下次 patch ☐ 下次 minor ☐ 接受 ☐ 推迟 | | |
| ISSUE-02 | ... | | | | |
| ... | | | | | |

## 6. 后续支持承诺

- [ ] Hypercare 4 周（依 `43_hypercare_plan_template.md`）；
- [ ] 长期 issue 跟踪（GitHub Issues）；
- [ ] release notes 订阅；
- [ ] RUSTSEC 公告同步（若集成方有需求）；
- [ ] 关键人 contact（见 §9）。

## 7. 验收声明

**集成方**确认：GVPE vX.Y.Z 满足双方协议约定的功能、性能、稳定性、兼容性、许可证要求；本证书生效后视为正式接受。

**GVPE 项目方**确认：自本证书生效日起，按 §6 提供后续支持。

## 8. 关联

- `28_workflow.md` §11.20 / §11.21
- `31_pilot_integration_agreement.md`
- `34_uat_plan_template.md`
- `35_uat_spec_template.md`
- `39_release_checklist.md`
- `43_hypercare_plan_template.md`
- `46_closure_report_template.md`

## 9. 签字

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| GVPE Release Manager | | | |
| GVPE 架构师 | | | |
| GVPE 项目负责人 | | | |
| 集成方项目负责人 | | | |
| 集成方技术负责人 | | | |
| 集成方 QA 负责人 | | | |
| 集成方法务 / 商务（如适用） | | | |
