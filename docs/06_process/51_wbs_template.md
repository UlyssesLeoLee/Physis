# WBS 模板（Work Breakdown Structure Template）

> **用途**：项目的工作分解结构。每个 crate = 一个 work package；crate 内每个模块 = 一个 work item。
> **对应工作流步骤**：
> - 132 WBS管理（WBS 管理）→ `28_workflow.md` §11.41
> - 131 プロジェクト計画（项目计划）→ §11.1 / `36_project_plan.md` §3
> **关联**：`36_project_plan.md`（项目计划 / 里程碑）；`50_status_report_template.md`（状态报告）；`49_meeting_notes_template.md`（会议纪要）。

## 0. WBS 元数据

| 字段 | 取值 |
|---|---|
| WBS 编号 | `WBS-XXX` |
| 关联项目 / Phase | `<Phase X>` |
| 周期 | `<YYYY-MM-DD ~ YYYY-MM-DD>` |
| 维护人 | — |
| 最后更新 | `<YYYY-MM-DD>` |

## 1. WBS 层级

| 层级 | 含义 | 示例 |
|---|---|---|
| L1 | 项目 / Phase | Phase 0 MVP |
| L2 | crate（work package） | `gvpe-solver` |
| L3 | 模块 / 任务（work item） | `SI 求解器最小骨架` |
| L4 | 子任务（可选） | `sphere-sphere 接触处理` |

## 2. L1 项目分解

| L1 ID | 名称 | 周期 | Owner | 状态 |
|---|---|---|---|---|
| 1 | Phase 0 MVP | T0 ~ T0+36 周 | 项目负责人 | ☐ 进行中 |
| 1.1 | M1 设计基线 | T0+4 周 | 架构师 | |
| 1.2 | M2 核心 crate 骨架 | T0+10 周 | 核心维护者 | |
| 1.3 | M3 SI 求解器最小骨架 | T0+18 周 | 求解器维护者 | |
| 1.4 | M4 MVP 功能完整 | T0+26 周 | 全员 | |
| 1.5 | M5 集成方 pilot | T0+32 周 | 集成方接口 | |
| 1.6 | M6 MVP release | T0+36 周 | Release Manager | |

## 3. L2 crate 分解

| L1 | L2 ID | 名称 | Owner | 估时 | 实际 | 状态 | 依赖 |
|---|---|---|---|---|---|---|---|
| 1.2 | 1.2.1 | `gvpe-math` | 数学负责人 | X 周 | | ☐ Not Started | — |
| 1.2 | 1.2.2 | `gvpe-core` | 核心维护者 | X 周 | | | 1.2.1 |
| 1.2 | 1.2.3 | `gvpe-memory` | 内存维护者 | X 周 | | | 1.2.1, 1.2.2 |
| 1.2 | 1.2.4 | `gvpe-shape` | 形状维护者 | X 周 | | | 1.2.1 |
| 1.2 | 1.2.5 | `gvpe-collision` | 碰撞维护者 | X 周 | | | 1.2.3, 1.2.4 |
| 1.2 | 1.2.6 | `gvpe-dynamics` | 动力学维护者 | X 周 | | | 1.2.2 |
| 1.2 | 1.2.7 | `gvpe-constraint` | 约束维护者 | X 周 | | | 1.2.6 |
| 1.2 | 1.2.8 | `gvpe-solver` | 求解器维护者 | X 周 | | | 1.2.7 |
| 1.2 | 1.2.9 | `gvpe-island` | 求解器维护者 | X 周 | | | 1.2.7 |
| 1.2 | 1.2.10 | `gvpe-scheduler` | 调度维护者 | X 周 | | | — |
| 1.2 | 1.2.11 | `gvpe-runtime` | runtime 维护者 | X 周 | | | 1.2.5, 1.2.8, 1.2.9, 1.2.10 |
| 1.2 | 1.2.12 | `gvpe-ffi` | FFI 维护者 | X 周 | | | 1.2.11 |

> 17 个 crate 全部 L2 列出；按 `04_architecture.md` §4.1 拓扑。

## 4. L3 work item 模板

> 每个 L2 crate 内的关键 work item 拆解。

| L2 | L3 ID | 名称 | 估时 | 实际 | 状态 | 依赖 | 关联需求 |
|---|---|---|---|---|---|---|---|
| 1.2.8 | 1.2.8.1 | SI 算法骨架 | 2 周 | | ☐ Not Started | 1.2.7 | `R-FR-002` |
| 1.2.8 | 1.2.8.2 | sphere-sphere 接触 | 1 周 | | | 1.2.8.1 | `R-FR-002` |
| 1.2.8 | 1.2.8.3 | box-plane 接触 | 1 周 | | | 1.2.8.1, 1.2.5 | `R-FR-002` |
| 1.2.8 | 1.2.8.4 | friction | 1 周 | | | 1.2.8.2 | `R-FR-002` |
| 1.2.8 | 1.2.8.5 | restitution | 0.5 周 | | | 1.2.8.2 | `R-FR-002` |
| 1.2.8 | 1.2.8.6 | sleeping | 1 周 | | | 1.2.8.4 | `R-FR-002`, `QA-I-03` |
| 1.2.8 | 1.2.8.7 | 单元测试 | 1 周 | | | 1.2.8.6 | `R-FR-002` |
| ... | | | | | | | |

## 5. 进度跟踪

| 状态 | 含义 |
|---|---|
| ☐ Not Started | 未开始 |
| 🟡 In Progress | 进行中 |
| 🟢 Done | 完成 |
| 🔴 Blocked | 阻塞 |
| ⏸ Deferred | 推迟 |

更新频率：每周（周会时）。

## 6. 风险与依赖

| 风险 / 依赖 | 描述 | 影响 | 缓解 |
|---|---|---|---|
| `QA-B-01` | 团队 Rust 物理引擎经验 | 全 MVP | 2 周 spike 验证 |
| `QA-I-01` | SI 求解器从零自研 | MVP 周期 | 写实施里程碑表 |
| ... | | | |

## 7. 关联

- `28_workflow.md` §11.41
- `36_project_plan.md` §3（里程碑）
- `50_status_report_template.md`
- `27_qa_register.md`（风险）

## 8. 维护

- 工具：GitHub Projects（或等价）；
- 更新：每周；
- 评审：每月（与月会同步）。

## 9. 签字

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 维护人 | | | |
| 项目负责人 | | | |
