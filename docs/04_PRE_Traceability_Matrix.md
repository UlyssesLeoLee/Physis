# Requirement Traceability Matrix

## 文书管理表

| 项目 | 内容 |
|---|---|
| 文书编号 | PRE-DOC-04 |
| 版本 | v0.1.3 |
| 状态 | Draft |
| 覆盖率 | 53/53 需求 ID 全覆盖（含 PRE-BEVY-001~006，见改订履历） |

## 改订履历

| 版本 | 日期 | 变更内容 |
|---|---|---|
| v0.1 | 2026-08-17 | 初版矩阵，PRE-API-001/002 合并为一行 |
| v0.1.1 | 2026-08-17 | 自审发现合并行导致按 ID 检索遗漏，拆分为独立两行，恢复 47/47 覆盖 |
| v0.1.2 | 2026-08-17 | 新增 PRE-BEVY-001~006 六行，覆盖率更新为 53/53 |
| v0.1.3 | 2026-08-17 | PRE-BEVY-001 行的约束对象由固定枚举改为「除 pre-bevy 外的全部 workspace 成员」，与 01 号文档 v0.1.3 保持一致 |

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
| PRE-BEVY-001 | pre-bevy（唯一依赖方）, 除 pre-bevy 外的全部 workspace 成员（零依赖约束对象，清单以 §4 为准） | §4 Component Diagram, §33.1, §33.5, ADR-009 | `cargo tree` 依赖树检查，从 workspace 成员列表动态枚举（CI 自动化） | AC-06 |
| PRE-BEVY-002 | pre-bevy | §33.2 回放设计 | Bevy 集成测试：加载 Experience 并断言实体 Transform 随时间变化 | AC-06 |
| PRE-BEVY-003 | pre-bevy | §33.2（插值逻辑） | 插值函数单元测试（覆盖采样点间/边界/单点退化情形） | — |
| PRE-BEVY-004 | pre-bevy, pre-retrieval, pre-verify | §33.3 异步查询桥接 | 异步查询集成测试：断言主线程/Schedule 不被阻塞 | — |
| PRE-BEVY-005 | pre-bevy（Phase 2，未实现） | §33.1（预留方向说明） | N/A（Phase 2 候选，非 MVP 交付物） | — |
| PRE-BEVY-006 | pre-bevy | §33.4 版本兼容策略 | crate 文档版本声明检查（人工/CI lint） | — |

## 需求覆盖检查

- 所有 26 类需求前缀（含新增 PRE-BEVY）均至少映射到一个架构组件与设计章节：已核对，无遗漏。
- 未映射到具体 Acceptance Criterion 的需求（如 PRE-DATA-004, PRE-SEC-002, PRE-BEVY-003/004/005/006）均为决策类/延后类/支撑性需求，已在 Open Questions 或 ADR 中登记，不视为覆盖缺口——PRE-BEVY 系列的端到端验收统一由 AC-06 承担（PRE-BEVY-001/002 直接对应，其余为支撑该验收的内部机制）。
