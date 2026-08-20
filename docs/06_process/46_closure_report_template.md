# 项目完成报告 / 收尾报告模板（Closure Report Template）

> **用途**：项目或 Phase 结束时的收尾报告。
> **对应工作流步骤**：
> - 145 プロジェクト完了判定（项目完成判定）→ `28_workflow.md` §11.49
> - 147 完了報告（完成报告）→ §11.51
> - 146 成果物引渡し（成果物交接）→ §11.50
> **关联**：`36_project_plan.md`（项目计划 / Phase 划分）；`45_retrospective_template.md`（回顾）；`27_qa_register.md`（最终 QA 状态）。

## 0. 报告元数据

| 字段 | 取值 |
|---|---|
| 报告编号 | `CLOSURE-XXX` |
| 关联项目 / Phase | `<Phase X>` |
| 关联周期 | `<YYYY-MM-DD ~ YYYY-MM-DD>` |
| 报告人 | — |
| 报告日期 | `<YYYY-MM-DD>` |

## 1. 项目目标达成度

> 引用 `36_project_plan.md` §3 的里程碑，逐项核对。

| 里程碑 | 计划日期 | 实际日期 | 状态 | 备注 |
|---|---|---|---|---|
| M0 启动 | | | ☐ 达成 / ☐ 推迟 / ☐ 取消 | |
| M1 设计基线 | | | | |
| M2 核心 crate 骨架 | | | | |
| M3 SI 求解器最小骨架 | | | | |
| M4 MVP 功能完整 | | | | |
| M5 集成方 pilot | | | | |
| M6 MVP release | | | | |
| ... | | | | |

## 2. 关键指标总结

| 指标 | 计划 | 实际 | 偏差 | 备注 |
|---|---|---|---|---|
| 范围（`GVPE-FR-XXX` 覆盖） | 100% | | | |
| 质量（test coverage / clippy 0 warning） | 100% | | | |
| 性能（60Hz @ 中端 PC） | 达成 | | | |
| 集成方 pilot | ≥ 1 | | | |
| UAT 通过 | 100% | | | |
| 文档完整（28+ 份） | 100% | | | |
| `cargo deny` 0 violation | 100% | | | |
| 工数（人月） | 估 X | 实 Y | 偏差 Z | |

## 3. 重大变更

> 引用 `28_workflow.md` §11.32 变更管理记录。

| CR 编号 | 变更内容 | 原因 | 状态 |
|---|---|---|---|
| CR-XXX | ... | ... | ☐ 已发布 |
| ... | | | |

## 4. 重大事件

> 引用 `40_postmortem_template.md` 报告。

| 事件 | 日期 | 影响 | postmortem 链接 | 改进措施 |
|---|---|---|---|---|
| | | | | |
| | | | | |

## 5. 范围 vs 实际

| 计划范围 | 实际范围 | 差异 | 解释 |
|---|---|---|---|
| MVP 8 个 FR | | | |
| MVP 4 个 NFR | | | |
| ... | | | |

## 6. 关键学习

> 引用 `45_retrospective_template.md` 的产出。

1. ...
2. ...
3. ...

## 7. 后续建议

> 给下一 Phase / 维护团队 / 后续项目的建议。

### 7.1 Phase 1+ 建议

- ...
- ...

### 7.2 维护团队建议

- ...
- ...

### 7.3 流程改进建议

- ...
- ...

## 8. 成果物清单

### 8.1 文档全集

- `docs/00` ~ `docs/40`（共 41 份主文档 + 14 份 archive）
- 引用本文件作为成果物登记

### 8.2 代码 / crate 集合

- `gvpe-math`, `gvpe-core`, `gvpe-memory`, `gvpe-shape`, `gvpe-collision`, `gvpe-dynamics`, `gvpe-constraint`, `gvpe-solver`, `gvpe-island`, `gvpe-scheduler`, `gvpe-runtime`, `gvpe-ffi`, `gvpe-vector`, `gvpe-graph`, `gvpe-compiler`, `gvpe-inference`, `gvpe-3dgs`（17 个 crate，依 `04_architecture.md` §4.1）
- crates.io 版本号
- GitHub Release URL

### 8.3 C ABI

- cbindgen 头文件（随 release 发布）
- 集成方 C++ wrapper 示例

### 8.4 CI / 工具链

- GitHub Actions 配置
- `deny.toml`、`rust-toolchain.toml`
- 文档构建配置

### 8.5 维护期 contact

- 核心维护者名单
- 集成方接口
- GitHub repo URL
- 沟通渠道（GitHub Issues / Discussions / 邮件）

## 9. 关联

- `28_workflow.md` §11.49 / §11.50 / §11.51
- `36_project_plan.md`
- `45_retrospective_template.md`
- `40_postmortem_template.md`
- `27_qa_register.md`（最终状态）

## 10. 审批

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 报告人 | | | |
| 项目负责人 | | | |
| 架构师 | | | |
| 关键集成方代表（如适用） | | | |
