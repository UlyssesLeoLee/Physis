# 设计评审模板（Design Review Template）

> **用途**：用于所有设计阶段评审（RD / BD / DD / ST Approval）。按评审类型填入对应章节。
> **对应工作流步骤**：
> - 20 要件レビュー（RD Review）→ `28_workflow.md` §11.2
> - 41 基本設計レビュー（BD Review）→ §11.4
> - 52 詳細設計レビュー（DD Review）→ §11.5
> - 89 システム試験完了承認 → §11.17
> **关联**：`28_workflow.md` §11.2-§11.5 / §11.17；`GVPE-DOC-26`（技术选型）；`GVPE-DOC-27`（QA 登记）。

## 0. 评审元数据

| 字段 | 取值 |
|---|---|
| 评审类型 | ☐ RD Review ☐ BD Review ☐ DD Review ☐ ST Approval |
| 评审编号 | `DR-XXX` |
| 评审日期 | `<YYYY-MM-DD>` |
| 主持人 | — |
| 评审人 | 必填（依评审类型，见下表） |
| 评审对象 | `<设计文档 GVPE-DOC-NN / release 版本 vX.Y.Z>` |
| 关联 PR | #XXX |

### 评审人最低配置

| 评审类型 | 评审人 |
|---|---|
| RD Review | 架构师 + 至少 2 名核心 crate 维护者 |
| BD Review | 架构师 + 对应模块负责人 |
| DD Review | 架构师 + 对应 crate 实现者 |
| ST Approval | Release Manager + 架构师 + QA 负责人 |

## 1. 评审范围

<!-- 列出本次评审的文档 / PR / commit。 -->

## 2. 评审检查表

> 按评审类型勾选对应章节。

### 2.1 RD Review（20 步）

- [ ] `01_requirements.md` §6 / §7 / §8 ID 完整性（FR / NFR / GPH / VEC / PERF / LIC / PROHIBIT / NG / AC）；
- [ ] 每个 ID 可验证（`AC-XX` 可执行测试，`FR-XXX` 可对应实现）；
- [ ] 跨文档引用一致（需求 ID 在所有 `GVPE-DOC-NN` 中一致）；
- [ ] `27_qa_register.md` Blocker 全部 Closed 或显式 Deferred；
- [ ] 范围纪律：`NG1` ~ `NG5` 与 `PROHIBIT-01` ~ `PROHIBIT-06` 在需求中显式落实；
- [ ] MVP 范围合理（6-12 月可达）；
- [ ] §11 审批签字表格已留位。

### 2.2 BD Review（41 步）

- [ ] `04_architecture.md` §4.1-§4.9 完整；
- [ ] crate map 符合 `cargo tree` 机械验证（`AC-02`）；
- [ ] `26_tech_selection.md` §6 技术栈选型与 BD 文档一致；
- [ ] `08_memory_design.md` 分配器策略可实现；
- [ ] `09_parallel_design.md` 并行模型与 crate 拓扑一致；
- [ ] `10_ffi_design.md` C ABI 形状稳定（句柄、POD、错误码）；
- [ ] 各子系统功能设计（`05` / `06` / `07` / `11` / `12`）已就位；
- [ ] 外部接口（`10_ffi_design.md`）与跨文档引用一致；
- [ ] BD 与 RD 双向追溯（`R-FR-XXX` / `R-NFR-XXX` 等 ID 在 BD 中有体现）；
- [ ] §11 审批签字表格已留位。

### 2.3 DD Review（52 步）

- [ ] `17_detailed_design.md` 14 节齐全（`gvpe-core` / `gvpe-memory` / `gvpe-shape` / `gvpe-collision` / `gvpe-dynamics` / `gvpe-constraint` / `gvpe-solver` / `gvpe-island` / `gvpe-scheduler` / `gvpe-runtime` / `gvpe-ffi` / `gvpe-graph-compiler` / `gvpe-vector` / 错误模型 / 处理序列）；
- [ ] 其他详细模块设计（`18` ~ `25`）已就位；
- [ ] struct / trait 字段明确（`#[repr(C)]` 与普通类型分清）；
- [ ] 算法伪代码 / 关键路径描述清楚；
- [ ] 错误模型与 `26_tech_selection.md` §18.8 一致；
- [ ] 与 BD 双向追溯（每个 crate / 类型 / 函数可回溯到 BD 子系统设计）；
- [ ] `27_qa_register.md` §9.2（I 类） / §9.3（T 类）QA 项在 DD 中得到回应；
- [ ] §11 审批签字表格已留位。

### 2.4 ST Approval（89 步）

- [ ] `32_system_test_spec_template.md` §2-§8 全部章节已执行；
- [ ] 所有功能测试（ST-FN-*）通过；
- [ ] 所有场景测试（ST-SC-*）通过；
- [ ] 性能测试（ST-PT-*）达成 `14_performance_budget.md` 目标；
- [ ] 负荷 / 压力测试（ST-LT-* / ST-ST-*）通过；
- [ ] 安全 / 许可证测试（ST-SC-* / `cargo deny` / `cargo audit`）通过；
- [ ] 障碍测试（ST-FT-*）通过；
- [ ] 集成方 pilot 反馈已处理（若有）；
- [ ] `27_qa_register.md` Blocker 全部 Closed；
- [ ] release notes 草稿已出具。

## 3. 评审意见

| 编号 | 类别 | 严重度 | 意见 | 决议 | 责任人 | 状态 |
|---|---|---|---|---|---|---|
| C-01 | 设计 / 正确性 / 文档 | ☐ Blocker ☐ High ☐ Medium ☐ Low | ... | ☐ 必须修改 ☐ 建议修改 ☐ 同意 | | ☐ Open ☐ Closed |
| C-02 | ... | ... | ... | ... | | ... |
| ... | ... | ... | ... | ... | | ... |

## 4. 评审结论

- [ ] ☐ **通过**：可进入下一阶段；
- [ ] ☐ **有条件通过**：评审意见中标注"必须修改"项全部解决后可进入下一阶段；
- [ ] ☐ **不通过**：必须重新设计后再次评审。

## 5. 评审产出

- [ ] 本纪要 commit 到 `docs/reviews/<评审类型>/DR-XXX.md`；
- [ ] 评审意见的"必须修改"项登记到 `27_qa_register.md`（如未登记）；
- [ ] 关联 PR / 文档 §修订历史 引用本评审编号。

## 6. 关联

- `28_workflow.md` §11.2 / §11.4 / §11.5 / §11.17
- `GVPE-DOC-26`（技术选型）
- `GVPE-DOC-27`（QA 登记）

## 7. 签字

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 主持人 | | | |
| 评审人 1 | | | |
| 评审人 2 | | | |
| 评审人 3（如适用） | | | |
| 架构师 | | | |
