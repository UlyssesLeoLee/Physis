# Risk & Issue Register

## 文书管理表

| 项目 | 内容 |
|---|---|
| 文书编号 | PRE-DOC-05 |
| 版本 | v0.1.2 |
| 状态 | Draft |

## 改订履历

| 版本 | 日期 | 变更内容 |
|---|---|---|
| v0.1 | 2026-08-17 | 初版：R-01～R-10 风险 + ISS-001～ISS-010 |
| v0.1.1 | 2026-08-17 | ISS-006/ISS-007/ISS-009 的应对措施已回填至 02号文档正文（§13/§26），本文书状态更新为"已落实设计" |
| v0.1.2 | 2026-08-17 | 新增 R-11（Bevy 版本演进风险）、ISS-011（Bevy 集成是否过早引入的自审） |

本登记表分两部分：Risks（项目/技术风险，未必已发生）与 Issues（架构自审中发现的具体问题，ISS-XXX）。

---

## 第一部分：Risks

| ID | 风险 | 影响 | 可能性 | 缓解措施 |
|---|---|---|---|---|
| R-01 | Physical Signature 丢失关键物理信息，导致 H1 不成立 | 高：核心假设失败，项目立论动摇 | 中 | MVP 保留 Signature 层可解释、可回退（PRE-ML-002）；先在小规模数据上做消融，尽早发现 |
| R-02 | 不同 solver 的响应本质不可比较（如 MPM 粒子场与 XPBD 约束网格的"变形"语义不同） | 高：H2 不成立，跨 solver 检索无意义 | 中 | Standard Response 层显式聚合为统计量而非原始张量，降低表示差异敏感度；先用同 solver 内验证 H1 再扩展 H2 |
| R-03 | ANN 检索存在高维退化或错误近邻，实际未显著缩小搜索空间 | 中：H3 不成立，检索价值存疑 | 中 | 通过暴力优化 vs 检索+优化的对比实验直接量化（06号文档） |
| R-04 | 参数不可辨识（多组参数产生几乎相同响应） | 中：Best Physical Explanation 不唯一，置信度虚高 | 高（物理系统常见现象） | CandidateScore 引入 model complexity 惩罚项；Explanation 输出必须展示 Top-M 而非唯一答案，暴露不可辨识性而非掩盖 |
| R-05 | 数据集生成参数空间组合爆炸 | 中：MVP 10K~100K 规模内采样代表性不足 | 中 | 采用 Latin Hypercube 等空间填充采样而非网格枚举（PRE-FR-014） |
| R-06 | 相关性误当因果：embedding 距离近不代表物理机制相同 | 高：违反 Physics First 原则 | 中 | ADR-002 架构约束（检索仅生成候选）+ 强制 Verification 阶段，从架构上阻断误用 |
| R-07 | MVP 工程周期（8~12周）内范围失控 | 中：无法在周期内产出可验证 PoC | 中 | 严格执行 NG1~NG7 非目标清单；FEM 仅做 stub；不做 GPU/分布式 |
| R-08 | 图数据库/Multi-Vector 等被提前引入造成过度设计 | 中：增加维护成本，稀释核心假设验证精力 | 低（已在设计阶段约束） | ADR-005/ADR-006 已限定 MVP 范围，Evolution Strategy 明确触发条件 |
| R-09 | Rust 生态中 ANN/HNSW 库成熟度不足，需要自行实现或大量适配 | 中：延误 pre-retrieval 开发 | 中 | 详细设计阶段先做库选型 spike（1~2天），必要时退回简化线性扫描作为 MVP baseline（10万规模线性扫描仍可接受） |
| R-10 | Reproducibility 在多线程/并行 solver 下难以保证数值一致 | 低（MVP 单线程 reference 为主） | 低 | Reference 实现单线程优先；并行版本延后到 Evolution Strategy 触发后 |
| R-11 | Bevy 快速迭代导致 `pre-bevy` 频繁需要跟随重写（ECS/渲染管线/Schedule 曾多次发生破坏性变更） | 中：`pre-bevy` 维护成本持续高于预期，或长期滞后于 Bevy 最新版 | 高（Bevy 历史上大版本升级破坏性变更常见） | ADR-009 已将影响面限定在单一 crate（核心不受影响）；`pre-bevy` 声明支持单一 Bevy 主版本而非兼容区间，跟随 Bevy 发版独立发版（PRE-BEVY-006, OQ-07），接受"滞后一个版本周期"为可接受状态而非缺陷 |

---

## 第二部分：架构自审 Issues

### Architecture

**ISS-001**
- Severity: Medium
- Impact: 若 Landmark 采样代替全场 Response 记录导致关键局部行为（如布料局部褶皱）丢失，会削弱 Deformation 特征质量
- Evidence: 02号文档 §8 明确将其列为"工程简化"，尚无实验验证信息损失程度
- Recommendation: 在 06 号 MVP 实验中加入一次"全场 vs landmark 采样"的消融对比，作为 H1 验证的附属实验
- Affected Requirements: PRE-PHY-001, PRE-PHY-003
- Affected Components: pre-signature, pre-solver-*

**ISS-002**
- Severity: Low
- Impact: Multi-Vector 简化为3个子向量+global，可能不足以支撑未来 contact/material/constraint 维度的独立检索需求
- Evidence: ADR-005 承认这是工程折衷，非最终设计
- Recommendation: 保留子向量拆分/合并的架构灵活性（每个子向量独立版本化），不需要现在拆分，但接口不应假设"恰好3个子向量"
- Affected Requirements: PRE-VEC-004
- Affected Components: pre-encoder, pre-retrieval

**ISS-003**
- Severity: Low
- Impact: 是否真正需要图数据库——当前结论是"不需要"，但该结论建立在 MVP 查询模式假设上，若假设错误会在 Phase 3 造成返工
- Evidence: ADR-006 依据是"当前无多跳查询需求"，属于前瞻性判断而非已验证事实
- Recommendation: 明确记录触发条件（Evolution Strategy §31）作为 Phase 3 设计输入，避免"结论"被误当作"永久决定"
- Affected Requirements: PRE-DATA-004
- Affected Components: pre-atlas

**ISS-004**
- Severity: Low
- Impact: 是否存在不必要的微服务/过早分布式化——审查结论：当前设计无此问题（单进程 MVP）
- Evidence: 02号文档 §3/§20/§29 均明确单机部署
- Recommendation: 无需行动，仅记录审查通过
- Affected Requirements: NG6
- Affected Components: 全部

**ISS-011**
- Severity: Low
- Impact: Bevy 集成是否属于"过早引入未验证的抽象"——本项目原则（需求文档"涌现式设计"）要求每个主要抽象回答"解决了哪个已被实验观察到的问题"，而 Bevy 适配层目前是响应用户明确要求，而非源于 H1~H5 实验证据
- Evidence: 01号文档第29节坦承"用户明确希望优先支持 Bevy"是该需求的直接来源，不是自下而上从实验证据推导得出
- Recommendation: 判定为可接受的例外，理由有三：(1) 范围已被 ADR-009 严格限定为独立可选 crate，零侵入核心，不产生"污染核心架构"的实际风险；(2) 回放能力对 H1~H5 验证本身有辅助价值（可视化调试候选/观测轨迹差异，帮助人工判断检索结果是否合理，间接支撑 06 号文档的实验分析工作）；(3) 已明确将高不确定性的双向映射方向（PRE-BEVY-005）推迟到 Phase 2，只在 MVP 纳入低风险的单向回放（PRE-BEVY-002/003）。故不视为违反最小可验证架构原则，但如实记录其需求来源与其他需求不同，供后续审查参考
- Affected Requirements: PRE-BEVY-001~006
- Affected Components: pre-bevy

### Physics

**ISS-005**
- Severity: High
- Impact: 不同 solver Response 是否真的可比较（R-02 的具体化）——这是项目最大的未验证假设
- Evidence: ADR-003 的解耦设计是必要条件但非充分条件；真正的可比较性需要实验证据（H2）
- Recommendation: H2 必须作为 MVP 的强制验收项，而非"锦上添花"；若 H2 不成立，需要考虑限定跨 solver 检索仅在"同类宏观现象"范围内进行，而非全局通用
- Affected Requirements: PRE-PHY-003
- Affected Components: pre-signature, pre-encoder

**ISS-006**
- Severity: Medium
- Impact: 参数不可辨识问题（R-04）未在 MVP 设计中给出具体检测机制，仅在 CandidateScore 中"惩罚复杂度"，不足以真正暴露不可辨识性
- Evidence: 02号文档 §14 未描述如何检测"多个 Top-M 候选参数差异大但误差相近"的情况
- Recommendation: Verification 阶段增加对 Top-M 候选参数分散度的统计，若分散度高应在 Explanation 输出中显式标注"参数不可辨识"，而非仅返回单一 best
- Affected Requirements: PRE-OBS-001, PRE-FR-009
- Affected Components: pre-verify
- Status: **已落实设计**（02号文档 v0.1.1 §13 新增"参数不可辨识暴露机制"，`CandidateExplanation.identifiability` 字段，§26）

### Vector Retrieval

**ISS-007**
- Severity: Medium
- Impact: post-filter 策略（ADR-007）在 metadata 分布极不均匀时可能导致召回不足，当前设计未定义"召回不足"的检测与应对
- Evidence: ADR-007 仅承诺"记为 Evolution Strategy 触发条件"，无 MVP 内的监控机制
- Recommendation: MVP 阶段在检索接口中加入"过滤后候选数低于阈值"的日志/告警，为后续判断是否需要 pre-filter 提供数据依据
- Affected Requirements: PRE-VEC-003, PRE-OBS-002
- Affected Components: pre-retrieval
- Status: **已落实设计**（02号文档 v0.1.1 §26 新增"检索召回不足监控"）

### Dataset

**ISS-008**
- Severity: Medium
- Impact: 参数采样策略（random/LHS 二选一）未明确如何避免 simulation bias（如某些参数区间因数值不稳定被系统性跳过而未被察觉）
- Evidence: 01号文档 PRE-FR-014 与 02号文档 §31 均未涉及 bias 检测
- Recommendation: Dataset Generator 需记录采样失败/发散的参数点分布，用于事后分析是否存在系统性偏差区域，而非直接丢弃
- Affected Requirements: PRE-REL-001, PRE-FR-014
- Affected Components: pre-gen

### Verification

**ISS-009**
- Severity: Medium
- Impact: "是否真正通过未来状态验证，还是仅拟合历史帧"——当前 Verification Pipeline（02号文档§13）用与观测相同的 InitialState/Excitation 重新仿真并比较全过程响应，本质是拟合同一条轨迹而非预测未见过的未来状态
- Evidence: 设计中未区分"训练窗口"与"held-out 时间窗口"
- Recommendation: 06号 MVP 实验应将观测响应切分为"匹配窗口"（用于检索与初步验证）与"held-out 未来窗口"（仅用于最终打分），以真正检验预测能力而非拟合能力
- Affected Requirements: PRE-FR-008, H4
- Affected Components: pre-verify, 06号文档实验设计
- Status: **已落实设计**（02号文档 v0.1.1 §13 新增窗口切分说明，与 06号文档实验设计一致）

### MVP

**ISS-010**
- Severity: Low
- Impact: 是否能在 8~12 周内形成核心 PoC——当前范围（3类solver + 3类子向量 + 单机存储 + 局部搜索优化器）经评估属于合理紧缩范围，但 FEM stub、Novel Dynamics 阈值标定（OQ-02）等仍有不确定工作量
- Evidence: 01号文档 C-03 约束 + 未决问题 OQ-02/OQ-03
- Recommendation: 建议将 Novel Dynamics 阈值标定与 CandidateScore 权重初始设定，明确列为"专家经验初始值 + 事后人工微调"，不做自动学习，以控制范围（对应 06 号文档实验计划）
- Affected Requirements: MVP Scope, OQ-02, OQ-03
- Affected Components: pre-verify, pre-refine

---

## 汇总：建议纳入下一迭代但不阻塞当前 Basic Design 通过的项

ISS-001, ISS-002, ISS-003, ISS-007, ISS-008, ISS-010, ISS-011（均为 Low/Medium，且已有明确缓解路径）

## 建议在进入详细设计前必须有初步方案的项

ISS-005（H2 可比较性——已作为 MVP 强制验收项处理）、ISS-006（参数不可辨识暴露机制）、ISS-009（Verification 拟合 vs 预测的区分，直接影响 H4 实验有效性）
