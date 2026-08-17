# Physical Retrieval Engine（PRE）需求定义书

版本：v0.1 draft
状态：Baseline for Basic Design
关联文档：02_PRE_Basic_Design.md, 04_PRE_Traceability_Matrix.md

---

## 1. 项目背景

传统物理引擎（Rigid Body / XPBD / MPM / FEM）能够可靠地正向求解「已知参数 → 物理响应」，但反向问题——「给定观察到的动力学，反推材料模型、参数、约束、场」——始终缺少系统化、可复用、可积累的解法。同时，Dynamic 3DGS / 4DGS 等技术使得从真实世界视频重建带时序的几何与运动成为可能，但重建结果止于「看起来像什么」，而非「物理上是什么」。

PRE 的立项前提：如果把大量传统 solver 产生的「初始状态 + 激励 + 材料 + 约束 + 求解器 + 参数 → 响应」经验，标准化为可检索的向量空间，就可以用检索大幅压缩 inverse physics 的搜索空间，再用正向仿真验证候选解，从而把「猜参数」问题转化为「检索 + 验证」问题。

## 2. 项目目标

- G1：构建可重放的物理实验记录格式（Physics Experience），覆盖 Rigid / XPBD / MPM 三类 solver 的 MVP 范围。
- G2：构建 solver-independent 的标准物理响应表示（Standard Physical Response），使不同 solver 的输出可比较。
- G3：构建可解释的 Physical Signature 与至少一个版本的 Physical Encoder，生成 Multi-Vector Embedding。
- G4：构建向量检索 + metadata filtering 的 Physics Atlas，支持 10K~100K 级 Physics Experience 的 interactive retrieval。
- G5：构建 Retrieval → Simulation Verification → Parameter Refinement 的闭环，并可量化验证 H1~H5 五条核心假设（见 06 号文档）。
- G6：为未来 Dynamic 3DGS 集成预留 Observation Backend 接口，但第一阶段不实现。

## 3. 非目标（Out of Scope，V0.1）

- NG1：不实现完整游戏物理引擎（渲染、场景管理、脚本系统等）。
- NG2：不实现 3DGS / 4DGS 重建或渲染管线本身。
- NG3：不自研全部 solver；FEM/Thin Shell 仅做接口与数据模型，不做高性能实现。
- NG4：不追求第一阶段覆盖流体、燃烧、破坏、复杂接触（如自接触精细摩擦）等复杂物理。
- NG5：不使用 LLM 或文本 embedding 直接判定物理参数或作为 Physical Encoder。
- NG6：不在 MVP 阶段做分布式部署、微服务化、GPU-only 实现。
- NG7：不在验证 H1~H5 之前推进 Phase 2（3DGS）及以后阶段。

## 4. 系统边界

PRE 是一个**独立于渲染引擎和游戏引擎**的后端系统，边界如下：

- 输入边界：Experiment Definition（人工/程序化定义）、Observed Physical Response（未来由 Observation Backend 提供，V0.1 为 synthetic）。
- 输出边界：Top-K/Top-M Candidate、Best Physical Explanation、Verification Report、新增/更新的 Physics Experience。
- 不跨越的边界：不负责最终 3D 内容渲染、不负责摄像头/传感器数据采集、不负责上层业务逻辑（游戏规则、UI）。

## 5. Stakeholder

| ID | 角色 | 关注点 |
|---|---|---|
| ST-01 | 物理仿真研究者 | Solver 正确性、可重放性、参数可辨识性 |
| ST-02 | 检索/ML 工程师 | Embedding 质量、ANN 性能、re-ranking 效果 |
| ST-03 | Rust 系统工程师 | Runtime 性能、插件化架构、内存/并发安全 |
| ST-04 | 未来 3DGS/CV 团队 | Observation Backend 接口稳定性 |
| ST-05 | 产品/项目负责人 | MVP 范围、里程碑、风险 |
| ST-06 | 下游应用（游戏/仿真/数字孪生） | API 稳定性、结果可解释性 |

## 6. Use Cases

- UC-01：研究者定义一个 Experiment（如「布料在风场下的悬垂」），系统自动采样参数、批量仿真、生成并入库 Physics Experience。
- UC-02：给定一段 Observed Physical Response（V0.1 为「留出仿真」模拟观测），系统检索 Top-K 候选，仿真验证，输出 Best Physical Explanation 及置信度。
- UC-03：工程师比较「暴力优化」与「ANN 检索 + 局部优化」两条路径的收敛速度与仿真次数，验证 H3。
- UC-04：系统判定当前观测在 Atlas 中找不到足够好的解释，进入 Novel Dynamics 探索流程（V0.1：仅记录与标记，不做自动 solver 探索）。
- UC-05：运维/研究者审计某个 Candidate 为什么被判定为最优，系统返回可分解的评分明细（非黑盒）。

## 7. Functional Requirements

- PRE-FR-001：系统必须能够定义、执行并记录一次 Physics Experience，且记录内容足以在相同 solver 版本下重放并复现在给定数值容差内一致的结果。
- PRE-FR-002：系统必须支持至少 Rigid Body、XPBD（cloth/soft body/attachment）、MPM（elastic/plastic/granular）三类 solver 插件，并通过统一 SolverPlugin trait 接入。
- PRE-FR-003：系统必须将任意 solver 的原始状态转换为 Standard Physical Response，且该转换逻辑与具体 solver 内部状态解耦（不得在检索/编码代码中出现 solver 特定字段）。
- PRE-FR-004：系统必须从 Standard Physical Response 提取 Physical Signature（结构化特征，覆盖 Geometry/Kinematics/Deformation/Temporal/Contact/Material/Constraint/Field 八个特征域中 MVP 声明支持的子集）。
- PRE-FR-005：系统必须提供至少一版 Physical Encoder，将 Physical Signature 编码为 Multi-Vector Embedding（至少 behavior_vector、deformation_vector、temporal_vector 三类，MVP 范围见 02 号文档）。
- PRE-FR-006：系统必须提供 ANN 检索接口，输入 Embedding，返回 Top-N 候选 Physics Experience ID 及各子向量相似度。
- PRE-FR-007：系统必须支持基于 metadata（solver 类型、材料类别、几何类别等）的候选过滤（pre-filter 或 post-filter，由实现选择并在设计文档中说明取舍）。
- PRE-FR-008：系统必须对 Top-K 候选执行 Simulation Verification（重新仿真并与观测比较），产出多维误差（至少 position/velocity/deformation 三类）。
- PRE-FR-009：系统必须计算可配置权重的 CandidateScore，且评分明细必须可查询、可解释，不得只返回单一标量而无分解。
- PRE-FR-010：系统必须提供参数优化插件接口，并至少实现一种优化算法（局部搜索或 CMA-ES 二选一，V0.1 取一）用于 Parameter Refinement。
- PRE-FR-011：系统必须能够判定「检索相似度低 或 仿真误差高于阈值」为 Novel Dynamics，并标记而非强行返回最近结果。
- PRE-FR-012：系统必须支持将验证通过的 Physics Experience 写回 Atlas 并重新建立索引，且新写入必须携带 provenance、confidence、validation status。
- PRE-FR-013：系统必须提供 Experience 的可信度生命周期（至少 Candidate / Validated 两级，V0.1 可省略 Trusted 级但需在设计中说明理由）。
- PRE-FR-014：系统必须提供程序化 Dataset Generator，支持至少一种参数采样策略（random 或 Latin Hypercube 二选一起步）用于批量生成 Physics Experience。
- PRE-FR-015：系统必须预留 ObservationBackend 抽象接口（trait/interface），V0.1 仅实现 SimulationBackend（用留出仿真模拟"观测"），不实现 3DGS 相关 backend。

## 8. Non-Functional Requirements

- PRE-NFR-001（可维护性）：Solver、Material、Constraint、Field、Collision 必须以插件形式接入，新增一种 solver 不得要求修改 Retrieval/Atlas 核心代码。
- PRE-NFR-002（可移植性）：核心 Runtime 使用 Rust 编写，MVP 阶段仅要求 CPU 单机可运行，SIMD/多线程为期望而非强制。
- PRE-NFR-003（可测试性）：核心数据结构（PhysicsExperience、StandardPhysicalResponse、PhysicalSignature、Embedding）必须可独立单元测试，不依赖完整 pipeline。
- PRE-NFR-004（可配置性）：CandidateScore 权重、ANN 参数（Top-N/Top-K/Top-M）、Novel Dynamics 阈值必须可在不重新编译的前提下配置。

## 9. Physics Requirements

- PRE-PHY-001：Standard Physical Response 必须至少包含 position(t)、velocity(t)、deformation(t)（若适用）、contact events、boundary/constraint events 五类字段，缺失字段必须显式标记为 N/A 而非省略。
- PRE-PHY-002：Reference solver 实现必须以正确性为首要目标，任何并行/SIMD/GPU 优化版本必须能够与 reference 实现在给定容差内数值对齐，并有自动化回归测试覆盖。
- PRE-PHY-003：任意两个不同 solver 对同一「宏观物理场景类别」（如：自由落体接触地面反弹）产生的 Standard Physical Response，必须能够被同一套 Physical Signature 提取逻辑处理并生成可比较的 embedding（可比较不等于数值相等，但同源相似场景在 embedding 空间距离应显著小于随机场景，此即 H1）。
- PRE-PHY-004：仿真必须记录足够的确定性元数据（seed、solver 版本、迭代次数、子步数、时间步长、硬件信息）以支持重放；不要求跨硬件位级一致，但要求同硬件同版本可复现。
- PRE-PHY-005：Physical Signature 的特征提取必须是 representation-independent 的最小要求：不得直接依赖 solver 内部网格拓扑索引、粒子编号等无物理意义的标识符作为特征。

## 10. Retrieval Requirements

- PRE-VEC-001：ANN 检索在 10K~100K 规模 Physics Experience 下，单次 Top-N（N≤100）查询延迟目标 P95 < 200ms（单机、无 GPU，具体基准见 06 号文档）。
- PRE-VEC-002：检索评分不得等价于单一 embedding 的 cosine similarity；最终 CandidateScore 必须融合 retrieval similarity、prediction accuracy、physical consistency 等至少三个维度（对应 PRE-FR-009）。
- PRE-VEC-003：系统必须支持 metadata filter 与向量检索组合使用（hybrid search），具体是 pre-filter 还是 post-filter 由 02 号文档给出选型并说明权衡。
- PRE-VEC-004：Multi-Vector 各子向量必须独立可查询、独立可评估（即可以单独衡量 behavior_vector 检索质量而不依赖其它子向量），以支持消融实验（H1/H2 验证需要）。

## 11. Data Requirements

- PRE-DATA-001：PhysicsExperience 记录必须包含 InitialState、BoundaryConditions、Excitation、MaterialModel/Parameters、Solver/SolverParameters、Response、PhysicalSignature、Embeddings、ValidationMetrics、Provenance 十个字段组，字段组内允许为空但字段组本身不可省略（结构完整性）。
- PRE-DATA-002：大体积原始响应数据（如逐帧全场位置/速度场）与 embedding、metadata 必须分离存储，metadata/embedding 查询不得触发大体积数据的读取。
- PRE-DATA-003：系统必须为 metadata 选择关系型/文档型存储，为 embedding 选择向量存储，为大体积响应选择对象/blob存储，三者选型与边界必须在 02 号文档中明确（对应 ADR-006），不得提前锁定具体数据库产品于本需求文档。
- PRE-DATA-004：是否引入图数据库存储 Entity/Relation/Field 结构，必须先证明关系型/向量存储无法满足查询模式后才可引入（对应自审 ISS 流程）。

## 12. AI/ML Requirements

- PRE-ML-001：V1 Physical Encoder 必须提供确定性特征向量（非神经网络）版本作为 baseline，且必须在文档中说明为何 MVP 优先选择可解释方法而非黑盒深度模型。
- PRE-ML-002：若使用降维/度量学习方法，必须保留可回退到未压缩 Physical Signature 的路径，用于调试与消融。
- PRE-ML-003：Embedding 版本必须显式编号（encoder_version），新版本 encoder 不得覆盖旧 embedding，需支持双版本并存与迁移。

## 13. API Requirements

- PRE-API-001：系统必须提供以下职责的接口（内部或 REST，V0.1 允许仅实现为 Rust 内部 trait + CLI，REST 化不强制）：创建/查询 Experience、触发仿真、编码、检索、验证、参数优化。
- PRE-API-002：REST schema 不在 V0.1 锁定；接口设计文档仅定义职责与数据流（对应第 24 节要求），实现可先以库 API 形式提供。

## 14. Performance Requirements

- PRE-PERF-001：单个 MVP solver（如 XPBD cloth，~1K 粒子级别）在参考实现下，生成一条 5 秒仿真时长的 Physics Experience 耗时目标 < 30s（单核，具体见 06 号文档基准）。
- PRE-PERF-002：Dataset Generator 批量生成需支持并行仿真（多进程/多线程均可），并行效率不做具体 SLA，但架构上不得阻塞并行扩展。

## 15. Reliability Requirements

- PRE-REL-001：任一 solver 崩溃或数值发散（NaN/Inf）必须被检测并标记该 Experience 为 invalid，不得静默写入 Atlas。
- PRE-REL-002：Simulation Verification 失败（如无法重放）必须记录明确错误原因，不得吞异常。

## 16. Reproducibility Requirements

- PRE-REPRO-001：任意 Physics Experience 必须可凭记录的 solver/version/seed/parameters 在同一环境下重放，重放结果与原始记录在容差内一致（见 PRE-PHY-004）。
- PRE-REPRO-002：Encoder 输出必须携带 encoder_version，重放时若 encoder 版本不同必须明确标注 embedding 不可直接比较。

## 17. Security Requirements

- PRE-SEC-001：Dataset Generator 与 Experiment Definition 的输入（尤其未来若开放 API）必须校验参数范围，防止资源耗尽型输入（如超大网格、超大迭代数）导致拒绝服务；V0.1 至少提供参数上限校验。
- PRE-SEC-002：系统不处理个人隐私数据；若未来 Observation Backend 接入真实摄像头/传感器数据，需在 Phase 2 单独评估数据合规性（V0.1 不涉及，记为 Open Question）。

## 18. Observability Requirements

- PRE-OBS-001：任意 Candidate 排序结果必须可分解展示：retrieval score、各子向量相似度、simulation error（分维度）、stability score、computational cost、confidence，禁止仅返回 ID。
- PRE-OBS-002：系统必须记录 pipeline 各阶段耗时（encode/search/simulate/verify/refine）用于性能分析。

## 19. MVP Scope

见 06_PRE_MVP_Experiment_Plan.md。摘要：Rigid + XPBD(cloth/soft) + MPM(elastic) 三类 solver，10K~100K Physics Experience，V1 Signature + V1 deterministic Encoder，ANN Top-K 检索，Simulation Verification 必须实现，Parameter Refinement 提供 basic 版本，不接入 3DGS。

## 20. Acceptance Criteria

- AC-01：能够端到端跑通 Synthetic Observation → Signature → Embedding → Retrieval → Simulation Verification → 输出 Best Explanation，且全流程有自动化测试覆盖。
- AC-02：H1~H3 假设的实验结果被记录并给出结论（成立/不成立/部分成立），见 06 号文档。
- AC-03：新增一种 solver 插件（在已支持的三类之外做小改动验证）不需要修改 Retrieval/Atlas 核心代码，仅需实现 SolverPlugin trait 与 Response 转换器。
- AC-04：任意一次检索结果可通过 Observability 接口展示评分明细。
- AC-05：至少一条 Physics Experience 完成「生成 → 验证 → 写回 Atlas → 重新检索命中」闭环。

## 21. Risks

见 05_PRE_Risk_Issue_Register.md。

## 22. Assumptions

- A-01：MVP 阶段用「留出仿真」（hold-out simulation）模拟 Observed Physical Response 是可接受的 H1~H5 验证方式，无需真实观测数据。
- A-02：单机（多核 CPU，无强制 GPU）足以验证 10K~100K 规模检索与仿真验证闭环的核心假设。
- A-03：确定性特征工程（非深度学习）足以在 MVP 阶段产生有意义的可检索空间；若不成立需在自审中记录并调整（见 ISS 流程）。

## 23. Constraints

- C-01：核心 Runtime 使用 Rust；ML 训练/实验/数据集探索允许使用 Python，但不得成为核心 Retrieval Runtime 的运行时依赖。
- C-02：不得引入图数据库、分布式存储、微服务架构，除非能证明关系型+向量+对象存储无法满足 MVP 查询模式。
- C-03：工程周期目标 8~12 周内完成可验证 PoC（对应 H1~H5）。

## 24. Open Questions

- OQ-01：Trusted Experience 级别的晋升规则（confidence 阈值、人工审核与否）尚未确定，V0.1 是否需要人工审核环节？
- OQ-02：Novel Dynamics 检测阈值（相似度低/误差高的具体数值）如何标定，是否需要专门的校准实验？
- OQ-03：CandidateScore 权重的初始值如何设定（专家设定 vs 数据学习），V0.1 是否只做专家设定？
- OQ-04：跨 solver 的 Physical Signature 可比性边界在哪里（例如 MPM 颗粒材料与 XPBD 布料的「变形」是否应共享同一 deformation_vector 维度定义）？
- OQ-05：未来真实观测数据（3DGS）合规性与噪声模型，何时启动评估？

## 25. Glossary

见 07_PRE_Glossary.md。

## 26. Requirement Traceability IDs 索引

前缀说明：PRE-FR（功能）、PRE-NFR（非功能通用）、PRE-PHY（物理）、PRE-VEC（检索）、PRE-DATA（数据）、PRE-ML（AI/ML）、PRE-API（接口）、PRE-PERF（性能）、PRE-REL（可靠性）、PRE-REPRO（可重复性）、PRE-SEC（安全）、PRE-OBS（可观测性）。完整映射见 04_PRE_Traceability_Matrix.md。
