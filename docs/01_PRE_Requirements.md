# Physical Retrieval Engine（PRE）需求定义书

## 文书管理表

| 项目 | 内容 |
|---|---|
| 文书编号 | PRE-DOC-01 |
| 文书名称 | Physical Retrieval Engine 需求定义书 |
| 版本 | v0.1.4 |
| 状态 | Draft — Baseline for Basic Design（尚未经 Stakeholder 正式承认） |
| 作成者 | PRE 架构设计团队（本轮由 Claude 代笔起草） |
| 承认者 | 未定（待 ST-05 项目负责人指定） |
| 关联文书 | 02（基本设计书）、03（ADR）、04（追踪矩阵）、05（风险登记簿）、06（MVP实验计划）、07（用语集）、08/09（详细设计・测试用例）、10~12（Testkit） |
| 适用范围 | PRE V0.1（MVP）；Phase 2 以降需另行制定需求追补文书 |

## 改订履历

| 版本 | 日期 | 变更内容 | 作成者 |
|---|---|---|---|
| v0.1 | 2026-08-17 | 初版起草：25类需求 + Traceability ID 体系确立 | Claude |
| v0.1.1 | 2026-08-17 | 按 IPA 文书标准补充文书管理表、承认栏、非机能要求グレード对齐、需求优先度与需求一览表 | Claude |
| v0.1.2 | 2026-08-17 | 新增第29节 Bevy 引擎集成需求（PRE-BEVY-001~006），补充 NG1/系统边界澄清、AC-06、OQ-07；优先度表与需求索引同步更新 | Claude |
| v0.1.3 | 2026-08-17 | 交叉审查修正：PRE-BEVY-001 与 AC-06 原先枚举的 crate 清单不一致（6 个 vs 4 个）且均遗漏 pre-signature/pre-encoder/pre-gen/pre-cli 等成员，统一改为「除 pre-bevy 外的全部 workspace 成员」以消除枚举漂移 | Claude |
| v0.1.4 | 2026-08-17 | 重大重构：PRE 定位明确为「可嵌入的底层 Rust 运行时」。第29节由 Bevy 专属重构为多宿主分层集成（Tier 0~3，覆盖 Bevy/Godot/Unity/Unreal/DCC 3D 软件），PRE-BEVY-001~006 废止并迁移至 PRE-ENG-*（对照表见 29.5）；新增第30节 GPU 后端需求（PRE-GPU-001~005，Vulkan/D3D 可移植性与宿主设备注入）、第31节可嵌入性需求（PRE-EMB-001~004）；新增 AC-07、OQ-08~10 | Claude |

## 承认栏

| 角色 | 姓名 | 承认日期 | 签署 |
|---|---|---|---|
| 项目负责人（ST-05） | 未定 | — | 未承认 |
| 物理仿真负责人（ST-01） | 未定 | — | 未承认 |
| 检索/ML负责人（ST-02） | 未定 | — | 未承认 |
| Rust系统负责人（ST-03） | 未定 | — | 未承认 |

> 注：本文书当前处于 Draft 状态，用于支撑基本设计基线的建立；正式进入详细设计前，建议至少完成 ST-05 与相关技术负责人的书面承认，并将本表更新为「承認済」。

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

- NG1：不实现完整游戏物理引擎或 3D 内容创作软件（渲染、场景管理、脚本系统、编辑器 UI 等）。**边界澄清**：为 Bevy/Godot/Unity/Unreal 及 DCC 3D 软件提供宿主集成适配层（见第29节 PRE-ENG-*）不违反本条——PRE 是被宿主嵌入的底层运行时，职责限定在物理数据的进出与计算，渲染/场景管理/脚本/编辑器由宿主自身负责。
- NG2：不实现 3DGS / 4DGS 重建或渲染管线本身。
- NG3：不自研全部 solver；FEM/Thin Shell 仅做接口与数据模型，不做高性能实现。
- NG4：不追求第一阶段覆盖流体、燃烧、破坏、复杂接触（如自接触精细摩擦）等复杂物理。
- NG5：不使用 LLM 或文本 embedding 直接判定物理参数或作为 Physical Encoder。
- NG6：不在 MVP 阶段做分布式部署、微服务化、GPU-only 实现。
- NG7：不在验证 H1~H5 之前推进 Phase 2（3DGS）及以后阶段。

## 4. 系统边界

PRE 是一个**可嵌入的底层 Rust 运行时**（library/runtime，非应用程序），独立于任何具体渲染引擎、游戏引擎或 3D 软件，由宿主驱动。边界如下：

- 输入边界：Experiment Definition（人工/程序化定义）、Observed Physical Response（未来由 Observation Backend 提供，V0.1 为 synthetic）。
- 输出边界：Top-K/Top-M Candidate、Best Physical Explanation、Verification Report、新增/更新的 Physics Experience。
- 不跨越的边界：不负责最终 3D 内容渲染、不负责摄像头/传感器数据采集、不负责上层业务逻辑（游戏规则、UI）。
- 宿主集成边界（第29节详述）：适配层允许宿主（游戏引擎 Bevy/Godot/Unity/Unreal，Python 宿主的 3D 软件）消费 PRE 的仿真回放与检索/验证结果，未来亦可反向提取 Experiment 定义（PRE-ENG-011，Phase 2）；但渲染、输入、游戏逻辑、编辑器 UI、资源管理仍完全属于宿主侧职责。核心 crate 不得依赖任何宿主 SDK（PRE-ENG-002，对应 ADR-009/ADR-010）。
- GPU 边界（第30节详述）：PRE 使用 GPU 仅用于**计算**（solver 加速），不承担宿主的渲染职责；与宿主的 GPU 设备共享（PRE-GPU-002）与零拷贝互操作（PRE-GPU-004）属于数据通路优化，不改变"PRE 不做渲染"这一边界。

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

### 8.1 非机能要求グレード対照表（参照 IPA「非機能要求グレード」六大品质特性）

为与日本 IPA（独立行政法人情報処理推進機構）通行的非机能要求分类惯例对齐，将本文书分散在各章节的非功能类需求，统一映射到 IPA 六大品质特性维度。本表仅为对照索引，不新增需求实体，各 ID 的权威定义仍以原章节为准。

| IPA 品质特性 | 定义范围 | 对应需求 ID | 备注 |
|---|---|---|---|
| 可用性（Availability） | 系统持续可用、故障恢复能力 | PRE-REL-001, PRE-REL-002 | MVP 单机部署，无高可用集群要求，故障处理以"不静默污染数据"为底线（对应第15节） |
| 性能・拡張性（Performance / Extensibility） | 响应时间、吞吐、规模扩展路径 | PRE-VEC-001, PRE-PERF-001, PRE-PERF-002 | 扩展性以 02号文档 §18/§31 Evolution Strategy 分阶段规划，MVP 不承诺 1M 以上规模 SLA |
| 運用・保守性（Operability / Maintainability） | 插件化、可配置、可观测、可测试 | PRE-NFR-001, PRE-NFR-003, PRE-NFR-004, PRE-OBS-001, PRE-OBS-002 | 插件化架构（PRE-NFR-001）是保守性核心手段 |
| 移行性（Portability / Migration） | 跨环境移植、版本迁移 | PRE-NFR-002, PRE-ML-003, PRE-REPRO-002 | 含 encoder/solver/signature 三类版本号的迁移策略（02号文档 §23 Versioning） |
| セキュリティ（Security） | 输入校验、数据保护 | PRE-SEC-001, PRE-SEC-002 | PRE-SEC-002 为 Phase 2 前置评估项，V0.1 不涉及真实隐私数据 |
| システム環境・エコロジー（System Environment） | 运行环境约束、资源占用 | PRE-NFR-002（CPU单机）, C-01～C-03（第23节制约条件） | MVP 明确排除 GPU-only、分布式、微服务化环境依赖 |

> 说明：可重复性（Reproducibility，PRE-REPRO-*）与数据需求（PRE-DATA-*）为本项目物理仿真领域的特化非功能需求，IPA 标准六分类未直接覆盖，故不强行并入上表，仍作为独立分类保留于第16/11节，避免削足适履。

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

见 06_PRE_MVP_Experiment_Plan.md。摘要：Rigid + XPBD(cloth/soft) + MPM(elastic) 三类 solver，10K~100K Physics Experience，V1 Signature + V1 deterministic Encoder，ANN Top-K 检索，Simulation Verification 必须实现，Parameter Refinement 提供 basic 版本，不接入 3DGS。**宿主集成的 MVP 范围（v0.1.4 修订）**：`pre-engine-api` 中立契约（PRE-ENG-003/004/005/006）与 Bevy 适配层（PRE-ENG-101）纳入 MVP；Godot 适配层（PRE-ENG-201）排在 Bevy 之后、**不作为 H1~H5 假设验证的前置条件**；Tier 2（Unity/Unreal，经 `pre-ffi`）与 Tier 3（Python/DCC）在 MVP 阶段**仅做接口设计与边界约束，不实现**；场景导入方向（PRE-ENG-011）为 Phase 2。GPU 后端（第30节 PRE-GPU-*）整体不在 MVP 实现范围内，仅要求架构不排斥（尤其 PRE-GPU-002 的设备注入必须在初始化架构中预留）。
> 范围警示：本节涉及四类宿主与一套 GPU 后端，是全文档中最容易发生范围失控的区域。判定基线不变——凡不影响 H1~H5 可验证性的，一律不进 MVP（对应约束 C-03 的 8~12 周周期，风险见 05号文档 R-13）。

## 20. Acceptance Criteria

- AC-01：能够端到端跑通 Synthetic Observation → Signature → Embedding → Retrieval → Simulation Verification → 输出 Best Explanation，且全流程有自动化测试覆盖。
- AC-02：H1~H3 假设的实验结果被记录并给出结论（成立/不成立/部分成立），见 06 号文档。
- AC-03：新增一种 solver 插件（在已支持的三类之外做小改动验证）不需要修改 Retrieval/Atlas 核心代码，仅需实现 SolverPlugin trait 与 Response 转换器。
- AC-04：任意一次检索结果可通过 Observability 接口展示评分明细。
- AC-05：至少一条 Physics Experience 完成「生成 → 验证 → 写回 Atlas → 重新检索命中」闭环。
- AC-06：在一个最小 Bevy 示例应用中，将一条已生成的 Physics Experience 的 Standard Physical Response 回放为实体 Transform 动画，且**除各适配层 crate 自身外的全部 workspace 成员**的依赖树均不出现任何宿主 SDK（`bevy`/`godot`/`pyo3`）——验证 ADR-009/ADR-010 的解耦是否落实，而非仅停留在文档声明；范围定义与 PRE-ENG-002 一致。
- AC-07：至少两个不同 Tier 1 宿主适配层（Bevy 与 Godot）针对同一条黄金响应数据，经各自坐标/单位换算后产出的变换序列在容差内一致（PRE-ENG-008 一致性套件）——这是验证「宿主中立契约（PRE-ENG-003）是否真的中立」的唯一实证手段；若两者不一致，说明通用逻辑仍有一部分事实上泄漏在适配层里。

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
- OQ-06：正式的需求/设计变更申请与评审流程（变更管理）尚未定义，当前仅有改订履历表记录变更内容，缺少「谁批准变更」的流程规定；进入详细设计前建议明确最小化的变更评审机制（例如：影响 Must 级需求的变更须经 ST-05 确认）。
- OQ-07：各适配层应锁定宿主的哪个版本？Bevy 与 Godot 均处于快速迭代阶段（Bevy 的 ECS/Schedule、Godot 的 GDExtension API 历史上均有破坏性变更）。方针已由 PRE-ENG-007 确定为「声明单一主版本、随宿主发版独立发版」，剩余未决的仅是各宿主的具体版本号，留待实现阶段首次接入时确定。
- OQ-08：Tier 2（`pre-ffi`）的实现时机如何判定？当前设计仅约束其边界（PRE-ENG-009），但未定义"何时开始实现"的触发条件。倾向：以出现真实的 Unity 或 Unreal 接入需求为触发，而非按时间表推进——过早实现 C ABI 会在没有真实调用方的情况下固化错误的接口形态，而 C ABI 一旦发布就难以变更。
- OQ-09：GPU 后端的抽象层选型（`wgpu` vs 直接使用 Vulkan/D3D 绑定 vs 其它）虽已由 PRE-GPU-001 限定为"必须可移植、不得把单一图形 API 写进 solver"，但具体选型需在 GPU 工作实际启动时基于当时的生态成熟度评估。另需评估：`wgpu` 的抽象是否足以表达 PRE 所需的计算模式（如 MPM 的 P2G/G2P 需要的原子操作与工作组内存），若不足，是否需要在特定后端上开专用路径。
- OQ-10：PRE-GPU-002 的宿主设备注入，在 `wgpu` 层面是否所有目标宿主都能提供所需的底层句柄？Bevy 使用 wgpu（可行性高），Godot/Unity/Unreal 使用各自的渲染后端，导出可共享的设备/队列句柄的可行性与代价需逐宿主评估——该评估结论可能反过来影响 PRE-GPU-004 零拷贝互操作的可达性。

## 25. Glossary

见 07_PRE_Glossary.md。

## 26. Requirement Traceability IDs 索引

前缀说明：PRE-FR（功能）、PRE-NFR（非功能通用）、PRE-PHY（物理）、PRE-VEC（检索）、PRE-DATA（数据）、PRE-ML（AI/ML）、PRE-API（接口）、PRE-PERF（性能）、PRE-REL（可靠性）、PRE-REPRO（可重复性）、PRE-SEC（安全）、PRE-OBS（可观测性）、PRE-ENG（宿主集成，见第29节；按 Tier 分块编号）、PRE-GPU（GPU 后端与图形 API，见第30节）、PRE-EMB（可嵌入性，见第31节）。完整映射见 04_PRE_Traceability_Matrix.md。

## 27. 需求优先度一览表（MoSCoW：必须 M / 推奨 S / 任意 C）

按 IPA 惯例，需求定义书须为每条需求标注优先度，避免"全部同等重要"导致范围失控。判定依据：是否为 AC-01～AC-05（第20节验收标准）所直接依赖 → 必须（M）；是否服务于 H1～H5 假设验证但非端到端闭环必需 → 推奨（S）；是否为架构预留/面向未来但 MVP 可延后 → 任意（C）。

| 优先度 | 需求 ID |
|---|---|
| 必须（M） | PRE-FR-001, PRE-FR-002, PRE-FR-003, PRE-FR-004, PRE-FR-005, PRE-FR-006, PRE-FR-008, PRE-FR-009, PRE-FR-012, PRE-FR-013, PRE-FR-014, PRE-PHY-001, PRE-PHY-002, PRE-PHY-003, PRE-PHY-004, PRE-PHY-005, PRE-VEC-001, PRE-VEC-002, PRE-DATA-001, PRE-DATA-002, PRE-DATA-003, PRE-NFR-001, PRE-REL-001, PRE-REL-002, PRE-REPRO-001, PRE-OBS-001, PRE-ENG-002, PRE-EMB-001, PRE-EMB-002 |
| 推奨（S） | PRE-FR-007, PRE-FR-010, PRE-FR-011, PRE-VEC-003, PRE-VEC-004, PRE-ML-001, PRE-ML-002, PRE-ML-003, PRE-NFR-002, PRE-NFR-003, PRE-NFR-004, PRE-PERF-001, PRE-PERF-002, PRE-REPRO-002, PRE-SEC-001, PRE-OBS-002, PRE-ENG-001, PRE-ENG-003, PRE-ENG-004, PRE-ENG-005, PRE-ENG-006, PRE-ENG-007, PRE-ENG-008, PRE-ENG-101, PRE-EMB-003, PRE-EMB-004, PRE-GPU-003, PRE-GPU-005 |
| 任意（C） | PRE-FR-015, PRE-DATA-004, PRE-API-001, PRE-API-002, PRE-SEC-002, PRE-ENG-009, PRE-ENG-010, PRE-ENG-011, PRE-ENG-201, PRE-ENG-301, PRE-ENG-401, PRE-ENG-501, PRE-GPU-001, PRE-GPU-002, PRE-GPU-004 |

判定说明：
- PRE-FR-015（ObservationBackend 接口预留）标为「任意」是因为 V0.1 唯一实现是 SimulationBackend，接口本身不影响端到端闭环能否跑通，属于面向 Phase 2 的架构投资。
- PRE-API-001/002 标为「任意」是因为 01/02 号文档均明确 MVP 以库 API + CLI 形式满足职责即可，REST 化非 MVP 验收所需（对应 PRE-API-002 原文"不锁定 schema"）。
- PRE-DATA-004（图数据库必要性论证）标为「任意」，因为 ADR-006 的 MVP 结论已经是"不引入"，该需求的价值在于为 Phase 3 留下判断依据，而非 MVP 阶段要交付的能力。
- PRE-ENG-002（核心 crate 不得依赖任何宿主 SDK）与 PRE-EMB-001/002（无全局状态、不占用主循环）标为「必须」——它们不是宿主集成功能本身的优先级，而是保护核心架构的护栏，且属于「早期不遵守、后期无法补救」的一类，优先级高于任何具体宿主能否接入。
- PRE-ENG-003~008 与 PRE-ENG-101（中立契约 + Bevy 适配）标为「推奨」：用户明确希望优先支持这些宿主，但回放能力不影响 H1~H5 核心假设的可验证性（AC-01~AC-05 不依赖任何宿主），故不升级为必须；同时也不降到任意，因为已排入 MVP 范围（第19节）并有 AC-06/AC-07 验收。
- PRE-ENG-201（Godot）标为「任意」而非「推奨」：用户要求支持 Godot，但它与 Bevy 同属 Tier 1、共享同一套中立契约，其真正的架构价值（证明契约确实中立）已由 AC-07 承载。若 MVP 周期紧张，先交付 Bevy + 完整中立契约、Godot 紧随其后，是比两个适配层都做一半更合理的取舍。
- PRE-ENG-009/010/301/401/501（Tier 2/3 与 Unity/Unreal/DCC）标为「任意」：MVP 阶段只做设计与边界约束，不实现。
- PRE-GPU-001/002/004 标为「任意」：GPU 后端整体不在 MVP 实现；但 PRE-GPU-002（宿主设备注入）虽不实现，其**初始化架构预留**是硬性设计要求，见第30节说明。
- PRE-GPU-003/005（CPU 为真值来源、无 GPU 时回退）标为「推奨」：它们约束的是"一旦做 GPU 就必须遵守"的规则，成本低且防止后期返工。
- 若必须（M）级需求在详细设计阶段出现无法满足的情况，须立即升级为风险并触发 ST-05 决策（而非静默降级为推奨）。

## 28. IPA 标准符合性自检清单

| 检查项（IPA 要件定義書/基本設計書惯例） | 是否满足 | 说明 |
|---|---|---|
| 文书管理表（文書番号/版本/作成者/承认者） | ✅ | 见本文书首部 |
| 改订履历 | ✅ | 见本文书首部；02～07号文档见各自首部 |
| 承认栏（署名欄） | ✅（未承认） | 承认栏已建立，但尚无实际签署，需项目负责人后续走查 |
| 需求一意 ID 化 | ✅ | 前缀分类，见第26节；PRE-BEVY-* 已于 v0.1.4 废止并迁移至 PRE-ENG-*（对照表见第29.5节） |
| 需求优先度 | ✅ | 见第27节 MoSCoW |
| 非机能要求按标准品质特性分类 | ✅ | 见第8.1节 IPA 六大品质特性对照表 |
| 前提条件・制约条件分离 | ✅ | 见第22节 Assumptions（前提）与第23节 Constraints（制约），未混同 |
| 需求—设计—测试—验收可追溯 | ✅ | 见 04_PRE_Traceability_Matrix.md，全部需求 ID 覆盖（v0.1.4 起纳入 PRE-ENG-*/PRE-GPU-*/PRE-EMB-*） |
| 用语集独立成册 | ✅ | 见 07_PRE_Glossary.md |
| 变更管理流程 | ⚠️ 部分 | 已有改订履历表，但尚未定义正式变更申请/评审流程（记为 Open Question OQ-06，见第24节补充） |

## 29. Host Integration Requirements（宿主集成需求：游戏引擎与 3D 软件）

**定位（本节的前提，也是对第4节系统边界的强化）**：PRE 是一个**可嵌入的底层 Rust 运行时**（embeddable low-level runtime），不是应用程序，也不是游戏引擎。宿主（游戏引擎、DCC 3D 软件、离线工具、研究脚本）驱动 PRE，而非相反。这一定位决定了后续全部集成设计：PRE 不拥有主循环、不假设自己独占进程、不持有全局可变状态。

**为何需要分层的集成模型**：目标宿主横跨四类语言/ABI 生态——Bevy（Rust）、Godot（Rust via GDExtension）、Unity（C#）、Unreal（C++）、以及 Blender/Maya/Houdini 等 3D 软件（主要为 Python）。它们无法用同一种接入机制覆盖：Rust 宿主可以直接消费 Rust 类型与 trait，而 C#/C++/Python 宿主必须经过稳定的 C ABI 或语言绑定，且不能跨边界传递 Rust 泛型、trait 对象、带载荷枚举或 `Result`。若不在设计阶段就区分这两类，最可能的失败模式是：先按 Rust 宿主写好接口，等到接入 Unity 时发现整套 API 无法跨 FFI 边界表达，被迫重做一层。

### 29.1 集成分层模型

- **PRE-ENG-001**：宿主集成必须按**接入机制**分层，每层有明确定义的边界与约束：
  - **Tier 0 — Rust 库 API**：PRE 的原生接口，workspace 内直接调用（研究脚本、CLI、其它 Rust 应用）。
  - **Tier 1 — Rust 链接适配层**：宿主本身是 Rust 或提供 Rust 绑定，适配层可直接链接 `pre-core`。目标：Bevy（`pre-bevy`）、Godot（`pre-godot`，经 GDExtension 的 Rust 绑定）。
  - **Tier 2 — C ABI 适配层**：宿主为 C/C++/C#，经 `pre-ffi`（cdylib + C 头文件）接入。目标：Unreal（C++）、Unity（C# P/Invoke）。
  - **Tier 3 — Python 绑定**：宿主为 Python 宿主的 3D 软件（Blender/Maya/Houdini），经 `pre-python`（PyO3）接入；同时复用于约束 C-01 已允许的 ML/研究工作流。
  层级划分依据是接入机制而非宿主知名度——同一 Tier 内的适配层共享绝大部分实现，跨 Tier 则不共享。

- **PRE-ENG-002**（取代旧 PRE-BEVY-001，范围由 bevy 扩大到全部宿主 SDK）：**除各 Tier 1/2/3 适配层 crate 自身外的全部 workspace 成员**，其 `Cargo.toml` 不得出现任何宿主 SDK 的直接或传递依赖（`bevy`、`godot`、`pyo3` 等）。适配层 crate 是唯一允许依赖对应宿主 SDK 的位置，且不启用任何适配层时整个核心系统必须可正常编译运行。
  > 与上一版本相同的理由：以「除适配层外的全部成员」而非固定枚举定义范围，避免新增 crate 时留下不被任何检查发现的缺口。

- **PRE-ENG-003**：必须存在一个宿主中立的契约 crate `pre-engine-api`，承载**所有宿主共用的集成逻辑**：回放游标与采样点插值、异步查询会话状态机、中立的 `LandmarkTransform` 类型、坐标/单位换算。各适配层只允许做「中立类型 ↔ 宿主类型」的映射与宿主特有的调度接入，**不得各自实现插值或查询状态机**。
  > 这条是本次分层设计的核心收益：四个宿主若各自实现插值，等于同一个数值 bug 有四份拷贝，且四份的边界行为（早于首帧/晚于末帧/单采样点）几乎必然不一致。

### 29.2 宿主中立的能力契约

- **PRE-ENG-004**（取代旧 PRE-BEVY-002/003）：必须提供回放（Playback）能力契约——将 `StandardPhysicalResponse` 的离散采样点，按 `LandmarkId` 映射为宿主场景对象的变换时间序列；由于 PRE 按固定 `dt`/`substeps` 采样而宿主以可变帧率运行，契约必须包含采样点间插值（至少线性插值），不得要求宿主帧率与 PRE 采样率对齐。插值实现位于 `pre-engine-api`（PRE-ENG-003）。
- **PRE-ENG-005**（取代旧 PRE-BEVY-004）：必须提供非阻塞的检索/验证查询桥接契约。检索与仿真验证可能耗时远超一帧，任何 Tier 的适配层都不得在宿主主线程/主循环上同步等待结果；契约以「提交请求 → 轮询/回调取结果」的形式定义，具体调度机制由各适配层用宿主原生设施实现。
- **PRE-ENG-006**（新增，跨引擎必须显式处理的问题）：必须定义 PRE 的**规范空间约定与单位约定**，并要求每个适配层显式声明其到宿主约定的换算：
  - PRE 规范约定：右手系、Y 轴向上、-Z 为前方；长度单位为 SI 米。
  - 已知差异（举例，非穷举）：Bevy 与 Godot 同为右手系 Y-up，换算接近恒等；**Unity 为左手系** Y-up；**Unreal 为左手系 Z-up 且默认长度单位为厘米**。
  - 每个适配层必须提供可测试的换算函数，并纳入 PRE-ENG-008 的一致性测试。
  > 若不在契约层固定规范约定，各适配层会各自"就地修正"手性与单位，最终表现为"同一条物理响应在 Unreal 里镜像了、在 Unity 里旋转方向反了、尺度差 100 倍"——且这类缺陷极难归因到某一层。
- **PRE-ENG-007**（取代旧 PRE-BEVY-006）：每个适配层必须在其文档中声明所支持的宿主版本（单一主版本而非兼容区间），并随宿主发版独立发布匹配版本。
- **PRE-ENG-008**（新增）：必须建立**适配层一致性测试套件**（conformance suite）：给定同一条黄金响应数据，任意适配层经其换算后产出的变换序列，必须在容差内一致。新增适配层时必须通过同一套件，不得各自定义验收口径。

### 29.3 Tier 2 / Tier 3 的边界约束（设计先行，实现后置）

- **PRE-ENG-009**（Tier 2 C ABI，设计阶段确定、实现随首个 Tier 2 宿主落地）：`pre-ffi` 暴露的 C ABI 必须满足：仅传递不透明句柄与 POD 结构体（不跨边界传递 Rust 泛型/trait 对象/带载荷枚举/`Result`/`String`）；错误以整型错误码返回；所有权与释放责任在头文件中显式注明；提供 ABI 版本查询入口；**每个 `extern "C"` 函数必须捕获 panic，禁止 panic 跨越 FFI 边界**（跨边界 unwind 是未定义行为）。
- **PRE-ENG-010**（Tier 3 Python）：`pre-python` 经 PyO3 暴露，必须遵守与 Tier 2 相同的「不跨边界传递 Rust 专有类型」原则，并明确 GIL 释放策略——长耗时的检索/仿真调用必须释放 GIL，否则会阻塞宿主 3D 软件的 UI 线程。
- **PRE-ENG-011**（取代旧 PRE-BEVY-005，Phase 2，不纳入 MVP）：反向的场景导入能力——从宿主场景提取 `InitialState`/`BoundaryConditions` 构造 `ExperimentDefinition`。不纳入 MVP 的理由不变：双向映射规范的工作量与不确定性显著高于回放方向，且不影响 H1~H5 假设验证。本条适用于全部宿主，不再是 Bevy 专属。

### 29.4 各宿主适配层需求（按 Tier 分块编号，预留扩展）

编号分块：`101~199` Bevy／`201~299` Godot／`301~399` Unity（预留）／`401~499` Unreal（预留）／`501~599` DCC 3D 软件（预留）。预留块表示架构已为其留位，但需求尚未细化——细化时机为该宿主实际进入实现排期。

- **PRE-ENG-101**（Tier 1, Bevy）：提供 Bevy `Plugin`，将 `pre-engine-api` 的中立变换映射为 Bevy `Transform`，使用 Bevy 原生任务池实现 PRE-ENG-005 的查询桥接。
- **PRE-ENG-201**（Tier 1, Godot）：经 Godot 4 GDExtension 的 Rust 绑定提供一个 `Node3D` 派生的回放节点，将中立变换映射为 Godot `Transform3D`，并使用 Godot 的线程/延迟调用设施实现 PRE-ENG-005 的查询桥接（不得在 `_process` 中同步等待）。
- **PRE-ENG-301 / 401 / 501**：预留，分别对应 Unity（C#，经 Tier 2）、Unreal（C++，经 Tier 2）、Python 宿主的 3D 软件（经 Tier 3）。架构上要求：新增这些适配层时，除新增其自身 crate/包外，不得修改 `pre-core` 与 `pre-engine-api` 之外的任何核心 crate——若届时发现必须修改核心，说明 PRE-ENG-003 的中立契约设计有缺陷，应作为架构缺陷立项而非就地打补丁。

### 29.5 与旧 PRE-BEVY-* 编号的迁移对照

本节在 v0.1.4 由「Bevy 专属」重构为「多宿主分层」，旧编号全部废止，对照如下（旧编号不再在任何文档中使用）：

| 旧编号 | 新编号 | 说明 |
|---|---|---|
| PRE-BEVY-001 | PRE-ENG-002 | 范围由「不依赖 bevy」扩大为「不依赖任何宿主 SDK」 |
| PRE-BEVY-002 | PRE-ENG-004 + PRE-ENG-101 | 通用回放契约与 Bevy 特有映射分离 |
| PRE-BEVY-003 | PRE-ENG-004 | 插值要求上提为宿主中立契约 |
| PRE-BEVY-004 | PRE-ENG-005 + PRE-ENG-101 | 非阻塞契约与 Bevy 特有调度分离 |
| PRE-BEVY-005 | PRE-ENG-011 | 场景导入方向适用于全部宿主 |
| PRE-BEVY-006 | PRE-ENG-007 | 版本策略适用于全部适配层 |

对应验收标准：AC-06、AC-07（第20节）。

## 30. GPU Backend Requirements（GPU 后端与图形 API 兼容性）

**背景**：PRE 作为底层运行时，其 solver 存在 GPU 加速需求（02号文档 §21 已规划 CPU→GPU 演进路径），且宿主（游戏引擎/3D 软件）本身通常已持有一个 GPU 设备。跨 Vulkan / Direct3D 12 / Metal 的可移植性，以及与宿主共享 GPU 设备的能力，必须在设计阶段确定，不能等 GPU 实现启动后再补——「PRE 自行创建独占设备」与「PRE 接受宿主注入设备」是两种不兼容的初始化架构，后期改造代价极高。

- **PRE-GPU-001**：GPU 计算后端必须经由可移植抽象层接入（首选 `wgpu`，可同时映射到 Vulkan / Direct3D 12 / Metal），核心 solver 代码中不得出现任何单一图形 API 的专有调用。选型在详细设计阶段确认，但「不得把 Vulkan 或 D3D 的 API 直接写进 solver」这一约束本身即为需求。
- **PRE-GPU-002**（关键的可嵌入性需求）：PRE 必须支持**由宿主注入 GPU 设备/队列**并在其上执行计算；仅当宿主未提供时才自行创建设备。理由：宿主已持有设备时，PRE 若另建一个设备，将导致显存重复占用、跨设备同步开销，以及在部分驱动上的资源共享失败。
- **PRE-GPU-003**：CPU reference 实现始终是数值真值来源，GPU 实现必须能与之在配置容差内对齐并有自动化回归覆盖——本条是 PRE-PHY-002 在 GPU 语境下的直接延伸，不引入新原则，仅明确其适用于 GPU 后端。
- **PRE-GPU-004**（设计预留，不纳入 MVP）：GPU 互操作/零拷贝——PRE 计算结果直接以宿主可用的 GPU 缓冲形式交付，免去 GPU→CPU→GPU 往返。该能力依赖外部内存共享机制（Vulkan external memory、D3D12 shared handle 等），复杂度与宿主耦合度均显著高于普通计算后端，因此仅要求架构上不排斥（数据通路设计不得假设结果必然经过 CPU），不要求 MVP 实现。
- **PRE-GPU-005**：无可用 GPU 或初始化失败时，系统必须能自动回退到 CPU 路径并记录明确原因，不得直接失败退出——PRE 的核心价值（检索与验证）在纯 CPU 下必须完整可用。

## 31. Embeddability Requirements（可嵌入性）

这些需求约束的是 PRE 核心自身，而非适配层；它们是「PRE 是被宿主嵌入的库」这一定位的直接推论，且几乎全部属于「早期不遵守、后期无法补救」的类别。

- **PRE-EMB-001**：PRE 不得依赖全局可变状态或单例；所有运行时状态必须挂在显式的上下文句柄下，允许同一进程内存在多个相互独立的 PRE 实例（宿主插件可能被多次实例化，3D 软件中尤其常见）。
- **PRE-EMB-002**：PRE 不得假设自己拥有主循环，不得阻塞调用线程执行长耗时任务；所有长耗时操作必须提供可由宿主驱动的分步执行或后台执行 + 轮询形式。
- **PRE-EMB-003**：PRE 的线程使用策略必须可由宿主配置（线程数上限，或复用宿主线程池）；不得在宿主不知情的情况下擅自创建大量线程——3D 软件宿主对线程与调度通常有严格约束。
- **PRE-EMB-004**：panic 不得跨越任何外语言边界（C ABI、Python 绑定），必须在边界处捕获并转换为该语言的错误表示（与 PRE-ENG-009 呼应，此处从核心侧再次约束）。
