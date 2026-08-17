# Requirement Traceability Matrix

## 文书管理表

| 项目 | 内容 |
|---|---|
| 文书编号 | PRE-DOC-04 |
| 版本 | v0.1.4 |
| 状态 | Draft |
| 覆盖率 | 全部需求 ID 覆盖；PRE-BEVY-001~006 已废止并迁移至 PRE-ENG-*（见 01号文档 §29.5） |

## 改订履历

| 版本 | 日期 | 变更内容 |
|---|---|---|
| v0.1 | 2026-08-17 | 初版矩阵，PRE-API-001/002 合并为一行 |
| v0.1.1 | 2026-08-17 | 自审发现合并行导致按 ID 检索遗漏，拆分为独立两行，恢复 47/47 覆盖 |
| v0.1.2 | 2026-08-17 | 新增 PRE-BEVY-001~006 六行，覆盖率更新为 53/53 |
| v0.1.3 | 2026-08-17 | PRE-BEVY-001 行的约束对象由固定枚举改为「除 pre-bevy 外的全部 workspace 成员」，与 01 号文档 v0.1.3 保持一致 |
| v0.1.4 | 2026-08-17 | PRE-BEVY-* 六行废止，替换为 PRE-ENG-*（20 条）、PRE-GPU-*（5 条）、PRE-EMB-*（4 条），对应 01号文档 v0.1.4 的多宿主分层重构 |

Requirement → Architecture Component → Design Section → Test → Acceptance Criterion

| Requirement ID | Architecture Component | Design Section (02号文档) | Test | Acceptance Criterion |
|---|---|---|---|---|
| PRE-FR-001 | pre-core, pre-gen | §5 Runtime Architecture, §7 Physics Experience Model | 重放数值对齐回归测试 | AC-01, AC-05 |
| PRE-FR-002 | pre-solver-api, pre-solver-rigid/xpbd/mpm | §6 Solver Architecture | solver 插件单元测试 | AC-03 |
| PRE-FR-003 | pre-solver-*, pre-core | §6, §8 Standard Physical Response Model | 跨 solver response schema 一致性测试 | AC-03 |
| PRE-FR-004 | pre-signature | §9 Physical Signature Design | 特征提取单元测试 | AC-01 |
| PRE-FR-005 | pre-encoder | §10 Embedding Architecture | encoder 确定性/回归测试 | AC-01 |
| PRE-FR-006 | pre-retrieval | §11 Vector Retrieval Architecture | 检索召回率/延迟测试 | AC-01, PRE-VEC-001 |
| PRE-FR-007 | pre-retrieval | §11, ADR-007 | post-filter 召回充分性测试 | AC-01 |
| PRE-FR-008 | pre-verify | §13 Simulation Verification Pipeline | 端到端验证测试 | AC-01 |
| PRE-FR-009 | pre-verify | §13, §26 Observability | CandidateScore 分解输出测试 | AC-04 |
| PRE-FR-010 | pre-refine | §14 Parameter Refinement Architecture | optimizer 收敛测试（H3实验） | H3, AC-01 |
| PRE-FR-011 | pre-verify | §13 | Novel Dynamics 阈值触发测试 | H5 相关 |
| PRE-FR-012 | pre-atlas | §12 Physics Atlas Architecture, §24 Provenance | 写回+重索引集成测试 | AC-05 |
| PRE-FR-013 | pre-core, pre-atlas | §7, §24 | validation status 状态机测试 | AC-05 |
| PRE-FR-014 | pre-gen | §20 Scheduling Architecture | 批量生成集成测试 | MVP Scope |
| PRE-FR-015 | pre-core (trait only) | §15 Observation Backend Interface | trait 编译期检查 + SimulationBackend 测试 | AC-01 |
| PRE-NFR-001 | 全部 solver/material/constraint plugin | §6, §31 Evolution Strategy | AC-03 新增插件不改核心 | AC-03 |
| PRE-NFR-002 | pre-runtime (Rust) | §21 CPU/GPU Architecture | 单机构建/运行测试 | — |
| PRE-NFR-003 | pre-core | §27 Testing Strategy | 单元测试覆盖率检查 | — |
| PRE-NFR-004 | pre-verify, pre-retrieval | §18 Storage/配置项 | 配置热加载测试 | — |
| PRE-PHY-001 | pre-core (StandardPhysicalResponse) | §8 | schema 字段完整性测试 | AC-01 |
| PRE-PHY-002 | pre-solver-* (reference + parallel) | §21, §27 | reference vs parallel 数值对齐回归 | — |
| PRE-PHY-003 | pre-signature, pre-encoder | §9, §10, ADR-003 | H1/H2 假设实验 | H1, H2 |
| PRE-PHY-004 | pre-core (determinism metadata) | §25 Determinism | 重放一致性测试 | AC-01 |
| PRE-PHY-005 | pre-signature | §9 | representation-independence 单元测试 | H1 |
| PRE-VEC-001 | pre-retrieval | §11 | 延迟基准测试 | 06号文档 Benchmark |
| PRE-VEC-002 | pre-verify | §13, ADR-002 | CandidateScore 融合逻辑测试 | H4 |
| PRE-VEC-003 | pre-retrieval | §11, ADR-007 | hybrid search 集成测试 | AC-01 |
| PRE-VEC-004 | pre-retrieval | §10 | 子向量独立查询测试（消融） | H1, H2 |
| PRE-DATA-001 | pre-core | §7 | schema 结构完整性测试 | AC-01 |
| PRE-DATA-002 | pre-atlas | §12, §17 Data Architecture | metadata查询不触发blob读取的性能测试 | — |
| PRE-DATA-003 | pre-atlas | §12, §17, ADR-006 | 存储分离集成测试 | — |
| PRE-DATA-004 | pre-atlas | §12, ADR-006 | N/A（决策文档，非代码测试） | — |
| PRE-ML-001 | pre-encoder | §10, §9 | encoder V1 确定性回归测试 | AC-01 |
| PRE-ML-002 | pre-encoder | §10 | 回退路径（signature直查）测试 | — |
| PRE-ML-003 | pre-atlas, pre-encoder | §23 Versioning | 多版本embedding隔离测试 | — |
| PRE-API-001 | pre-cli | §19 API Architecture | CLI smoke test | — |
| PRE-API-002 | pre-cli | §19 API Architecture | N/A（决策类：V0.1 不锁定 REST schema） | — |
| PRE-PERF-001 | pre-solver-*, pre-gen | §14 (需求), 06号Benchmark | 单条Experience生成耗时基准 | 06号文档 |
| PRE-PERF-002 | pre-gen | §20 Scheduling Architecture | 并行生成吞吐测试 | — |
| PRE-REL-001 | pre-solver-*, pre-core | §22 Error Handling | NaN/Inf检测单元测试 | — |
| PRE-REL-002 | pre-verify, pre-refine | §22 | 异常路径集成测试 | — |
| PRE-REPRO-001 | pre-core, pre-solver-* | §25 Determinism | 重放一致性测试 | AC-01 |
| PRE-REPRO-002 | pre-atlas, pre-encoder | §23 Versioning | 跨版本比较拦截测试 | — |
| PRE-SEC-001 | pre-gen, pre-cli | §17（隐含）参数校验 | 输入边界校验测试 | — |
| PRE-SEC-002 | N/A（V0.1不涉及） | — | — | OQ-05 |
| PRE-OBS-001 | pre-verify | §26 Observability | CandidateExplanation 结构测试 | AC-04 |
| PRE-OBS-002 | pre-runtime全链路 | §26 | stage timing 记录测试 | — |

| PRE-ENG-001 | 全部适配层 crate | §33.1 分层模型 | 分层归属评审（设计类） | — |
| PRE-ENG-002 | 全部核心 crate（约束对象）, 各适配层（唯一豁免） | §4, §33.1, §33.8, ADR-009/010 | `cargo tree` 依赖树检查，从 cargo metadata 动态枚举（CI 自动化） | AC-06 |
| PRE-ENG-003 | pre-engine-api | §33.2 中立契约 | 中立性验证：契约层不含任何宿主 SDK 类型 | AC-07 |
| PRE-ENG-004 | pre-engine-api（PlaybackCursor） | §33.2 | 插值单元测试（含四类边界情形） | AC-06, AC-07 |
| PRE-ENG-005 | pre-engine-api（QuerySession）, 各适配层 | §33.2, §35 | 非阻塞性测试：宿主主循环不被阻塞 | — |
| PRE-ENG-006 | pre-engine-api（SpatialConvention） | §33.3, ADR-011 | 坐标/单位换算测试（含非恒等换算用例） | AC-07 |
| PRE-ENG-007 | 各适配层 | §33.4, §33.6 | crate 文档版本声明检查 | — |
| PRE-ENG-008 | pre-testkit（套件实现）, 各适配层（被测对象） | §33.7 | 一致性套件：同一黄金数据跨适配层输出一致 | AC-07 |
| PRE-ENG-009 | pre-ffi（MVP 仅设计） | §33.5 | N/A（MVP 不实现；约束反向体现于 §33.2 契约设计评审） | — |
| PRE-ENG-010 | pre-python（MVP 仅设计） | §33.6 | N/A（MVP 不实现） | — |
| PRE-ENG-011 | 各适配层（Phase 2） | §33.1（预留方向） | N/A（Phase 2） | — |
| PRE-ENG-101 | pre-bevy | §33.4 | Bevy 集成测试：Transform 随时间变化 | AC-06, AC-07 |
| PRE-ENG-201 | pre-godot | §33.4 | Godot 集成测试：Transform3D 随时间变化 | AC-07 |
| PRE-ENG-301 | 预留：Unity 适配（Tier 2，未实现） | §33.5 | N/A（预留块，需求未细化） | — |
| PRE-ENG-401 | 预留：Unreal 适配（Tier 2，未实现） | §33.5 | N/A（预留块，需求未细化） | — |
| PRE-ENG-501 | 预留：DCC 3D 软件适配（Tier 3，未实现） | §33.6 | N/A（预留块，需求未细化） | — |
| PRE-GPU-001 | pre-gpu（MVP 不实现） | §34.2 | N/A（MVP 不实现；约束为"solver 中不得出现单一图形 API 调用"，由代码评审/lint 保证） | — |
| PRE-GPU-002 | pre-gpu, PreContext 初始化路径 | §34.3, ADR-012 | 初始化架构评审：构造函数可接收外部设备（即使恒为 None） | — |
| PRE-GPU-003 | pre-gpu, pre-solver-* | §34.4 | GPU/CPU 数值对齐回归（GPU 实现后生效） | — |
| PRE-GPU-004 | pre-gpu（设计预留） | §34.5 | N/A（架构约束：数据通路不得假设结果必经 CPU，由设计评审保证） | — |
| PRE-GPU-005 | pre-gpu | §34.4 | 无 GPU 环境下的回退测试 | — |
| PRE-EMB-001 | pre-core（PreContext） | §35 | 多实例并存测试（同进程创建多个独立 PRE 实例） | — |
| PRE-EMB-002 | pre-core, pre-engine-api | §35, §33.2 | 长耗时操作不阻塞调用线程的测试 | — |
| PRE-EMB-003 | pre-core | §35 | 线程数上限配置生效测试 | — |
| PRE-EMB-004 | pre-ffi, pre-python | §35, §33.5 | panic 边界捕获测试（MVP 仅设计，实现后生效） | — |

## 需求覆盖检查

- 所有需求前缀（含新增 PRE-ENG / PRE-GPU / PRE-EMB）均至少映射到一个架构组件与设计章节：已核对，无遗漏。
- 未映射到具体 Acceptance Criterion 的需求均为决策类/延后类/支撑性需求，已在 Open Questions 或 ADR 中登记，不视为覆盖缺口：
- PRE-ENG 系列的端到端验收由 AC-06（依赖隔离）与 AC-07（跨适配层一致性）承担；Tier 2/3 与预留块（PRE-ENG-009/010/011/301/401/501）MVP 不实现，其"验证方式"为设计评审而非测试。
- PRE-GPU 系列 MVP 整体不实现；其中 PRE-GPU-002（设备注入）虽不实现，但**初始化架构预留**属于必须在 MVP 代码中体现的设计约束，验证方式为架构评审（ADR-012 已记录理由）。
- PRE-EMB-004（panic 不跨边界）随 Tier 2/3 实现后才可测试，MVP 阶段为设计约束。
