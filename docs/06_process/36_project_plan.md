# GVPE 项目计划（Project Plan）

> **用途**：项目层面的整体计划，承担 `28_workflow.md` §10.15 步 131（プロジェクト計画）的角色。
> **关联**：`GVPE-DOC-00`（总论）、`GVPE-DOC-01`（需求）、`GVPE-DOC-26`（技术选型）、`GVPE-DOC-27`（QA 登记）、`28_workflow.md`（工作流）、`39_release_checklist.md`（release 流程）。

## 0. 元数据

| 字段 | 取值 |
|---|---|
| 计划版本 | v0.1 |
| 适用阶段 | MVP（Phase 0）+ Phase 1 展望 |
| 项目代号 | Physis / GVPE |
| 项目负责人 | — |
| 架构师 | — |
| 上次更新 | 2026-08-19 |

## 1. 项目愿景（继承自 `GVPE-DOC-00`）

一句话：**自研、商用实时级 Rust 物理引擎，通过 C ABI 暴露给 Unity / Unreal / Godot / 自研游戏引擎**。

**唯一不变量**：即使 Graph / Vector / AI / 3DGS 全部关闭，仅剩的 Rust Runtime 仍必须是完整、可独立运行、商用实时级的物理引擎。

## 2. 阶段划分

| 阶段 | 名称 | 周期（参考） | 关键交付 |
|---|---|---|---|
| Phase 0 | MVP | T0 ~ T0+6 月 | 27 份基线文档 + 28-40 流程文档 + MVP crate 全集 + 1-2 集成方 pilot |
| Phase 1 | 稳态化 | T0+6 ~ T0+12 月 | v1.0 release + 多集成方 + 性能优化 + 文档完善 |
| Phase 2 | 扩展 | T0+12 ~ T0+24 月 | 关节、CCD、XPBD、Shape Advanced、Energy/Wave/Field、Vector、Graph 全功能 |
| Phase 3+ | 长尾 | T0+24 月+ | 社区、3DGS 闭环、Fluid/FEM、GPU 后端（待评估） |

注：周期为参考估计，**未**做承诺；MVP 实际周期应通过 §3 里程碑逐次评估。

## 3. MVP 里程碑（Phase 0）

| 里程碑 | 周期（参考） | 关键交付 | 关联工作流 |
|---|---|---|---|
| M0 启动 | T0 | 28-40 号流程文档齐备；git / CI 就绪；集成方签字 | §11.1 |
| M1 设计基线 | T0+4 周 | 27 份基线文档已审；QA 登记 Blocker 全部 Closed 或显式 Deferred | §11.2-§11.3 |
| M2 核心 crate 骨架 | T0+10 周 | `gvpe-core` / `gvpe-memory` / `gvpe-math` 可编译，单元测试通过 | §10.5-§10.6 |
| M3 SI 求解器最小骨架 | T0+18 周 | sphere-sphere + 1 岛 + 无 sleeping，跑通 §33.2.1 单球下落 | §10.7, §33.2 |
| M4 MVP 功能完整 | T0+26 周 | box / plane / joint / friction / sleeping / islands / C ABI / 多线程 | §10.4, §10.5 |
| M5 集成方 pilot | T0+32 周 | 1-2 集成方跑通 L1 场景 | §11.18-§11.21, `31` |
| M6 MVP release | T0+36 周 | v0.1.0 发布，crates.io + GitHub Release | §11.22-§11.26 |

## 4. 资源（参考）

| 角色 | 人数 | 职责 |
|---|---|---|
| 项目负责人 | 1 | 范围 / 时间 / 商务 / 集成方接口 |
| 架构师 | 1 | crate map / 依赖方向 / 技术选型 / 重大决策 |
| 核心 crate 维护者 | 2-3 | `gvpe-core` / `gvpe-math` / `gvpe-memory` / `gvpe-collision` / `gvpe-solver` / `gvpe-runtime` |
| 周边 crate 维护者 | 1-2 | `gvpe-graph` / `gvpe-vector` / `gvpe-compiler` |
| 工具链 / CI 维护者 | 0.5 | `cargo deny` / CI matrix / cbindgen |
| QA | 0.5 | 性能 baseline / determinism harness / 集成方支持 |
| 文档维护者 | 0.5 | 28 份文档维护 + 新文档起草 |
| 集成方接口 | 0.5 | pilot / UAT / 商务 |

注：单人或小团队场景下，部分角色由同一人兼任。

## 5. 风险摘要

> 详细见 `GVPE-DOC-27` §9。**MVP 启动前必须关闭的 Blocker**：

| ID | 风险 | 严重度 | 缓解 |
|---|---|---|---|
| QA-B-01 | 团队 Rust 物理引擎经验曲线 | Blocker | 2 周 spike 验证最小骨架 |
| QA-B-02 | MVP 范围 vs 时间硬约束 | Blocker | 设立 trade-off 决策机制 |
| QA-D-03 | PhysicsSignature 组合查询 API | Blocker | spike 2-3 候选 API |
| QA-F-01 | C ABI 版本号机制 | Blocker | spike 实现 `gvpe_abi_version()` |
| QA-I-01 | SI 求解器从零自研工作量 | Blocker | 写 SI 实施里程碑表 |
| QA-T-01 | portable SIMD 仍 nightly | Blocker | spike 实测 |
| QA-Q-01 | 确定性回放 harness 未跑通 | Blocker | spike 1 个场景跑通 |

## 6. 沟通计划

| 频率 | 活动 | 参与者 | 产出 |
|---|---|---|---|
| 每日 | 异步 issue / PR review | 维护者 | 代码 review |
| 每周 | 同步会 | 核心团队 | 进度、风险、issue |
| 每两周 | QA 状态审视 | 核心团队 + 维护者 | `27_qa_register` 状态更新 |
| 每月 | 性能 / 质量指标审视 | 核心团队 | performance / quality report |
| 季度 | 范围 / 战略审视 | 全员 | 范围 / 战略调整 |
| 不定期 | 集成方同步 | 项目负责人 + 集成方接口 | pilot 进度 / 反馈 |
| 半年 | 质量审计 | 架构师 + 外部 reviewer | audit report |

## 7. 范围纪律

- **MVP 严格不引入**：fluid / FEM（`NG1`）、3DGS 闭环（`NG2`）、GPU compute 后端（`NG3`）、LLM 推理（`PROHIBIT-05`）；
- **范围变更必须**：走 `28_workflow.md` §11.32 变更管理流程；登记 `27_qa_register`；
- **feature creep 对策**：每周审视 scope；任何"顺手加一下"必须有 trade-off 评估。

## 8. 移交 / 退场（Phase 边界）

- **Phase 0 → Phase 1**：MVP v1.0 release + 集成方正式支持；
- **Phase 1 → Phase 2**：功能扩展启动；Postmortem + 回顾（依 `28_workflow.md` §11.52）；
- **任何 Phase 结束**：依 `28_workflow.md` §11.49-§11.53（项目完成判定 / 成果物交接 / 完成报告 / 回顾 / 知识移交 / 归档）。

## 9. 关联

- `GVPE-DOC-00`（总论 / 不变式）
- `GVPE-DOC-01`（需求规约）
- `GVPE-DOC-26`（技术选型）
- `GVPE-DOC-27`（QA 登记 / 风险）
- `28_workflow.md`（工作流基线）
- `31_pilot_integration_agreement.md`（集成方 pilot）
- `34_uat_plan_template.md`（UAT 计划）
- `39_release_checklist.md`（release 流程）

## 10. 审批

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 编写 | — | | |
| 评审 | | | |
| 批准 | | | |
