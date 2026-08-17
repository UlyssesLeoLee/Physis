# Physis

Rust 物理引擎 — Physical Retrieval Engine（PRE）

**可嵌入的底层 Rust 物理运行时**：由宿主（游戏引擎 / 3D 软件）驱动，而非独立应用。

PRE 的目标不是制作一个传统物理引擎，而是构建一个**可计算、可索引、可向量检索、可验证、可持续积累经验的物理运行时**：先用可靠的传统 solver（Rigid / XPBD / MPM / FEM）批量产生「初始状态 + 激励 + 材料 + 约束 + 求解器 + 参数 → 物理响应」的经验，标准化为可检索的 Physical Signature / Embedding，沉淀进 Physics Atlas；再通过 `观测 → 检索 → 仿真验证 → 参数优化` 的闭环反推出最能解释观察结果的物理模型。核心原则：**检索只负责生成候选，物理仿真才是唯一的最终验证手段**（Retrieval proposes, Physics verifies）。

当前阶段：需求、基本设计、详细设计文档基线均已完成（V0.1 Draft），尚未开始实现代码。

**宿主集成与 GPU**：按*接入机制*分层——Tier 0 Rust 库 API、Tier 1 Rust 链接适配层（**Bevy** / **Godot**）、Tier 2 C ABI（**Unity** / **Unreal**）、Tier 3 Python 绑定（Blender/Maya/Houdini 等 3D 软件）。全部宿主共用逻辑（回放插值、查询会话、坐标与单位换算）收敛在中立契约 crate `pre-engine-api`，适配层只做类型映射与调度接入。GPU 计算后端经可移植抽象覆盖 Vulkan / D3D12 / Metal，且**接受宿主注入设备**而非自建。MVP 只实现中立契约 + Bevy；Godot 紧随其后；Tier 2/3 与 GPU 仅做设计约束不实现（理由见 ADR-010/011/012 与 05 号文档 ISS-012）。

## 文档

需求定义、基本设计与详细设计相关文档位于 [`docs/`](docs/) 目录，遵循 IPA（情報処理推進機構）文书惯例编写（文书管理表 / 改订履历 / 承认栏 / 需求优先度 / 追踪矩阵 / テストケース一覧）。阅读顺序建议：01 → 02 → 03 → 04 → 08 → 09，05/06/07 按需查阅；10～12 是自动测试用 Mock 项目（`pre-testkit`）的独立三段式文档，只服务于测试基础设施，不计入核心需求 ID 体系（理由见 10 号文档 §4）。

| 编号 | 文档 | 概要 |
|---|---|---|
| 01 | [PRE_Requirements](docs/01_PRE_Requirements.md) | 需求定义书。项目背景/目标/非目标、系统边界、Stakeholder、Use Cases，带唯一 ID 的需求（功能/非功能/物理/检索/数据/AI-ML/接口/性能/可靠性/可重复性/安全/可观测性 + **PRE-ENG 宿主集成** / **PRE-GPU 图形 API** / **PRE-EMB 可嵌入性**），MVP 范围、验收标准、风险/假设/约束/未决问题、需求优先度（MoSCoW）与 IPA 非机能要求グレード对照表。 |
| 02 | [PRE_Basic_Design](docs/02_PRE_Basic_Design.md) | 基本设计书。架构总览与 Context/Container/Component 图，Solver 插件化架构，Raw State → Standard Response → Signature → Embedding 四层分离模型，Multi-Vector 检索与融合、Physics Atlas 存储架构、仿真验证与参数优化流水线、CPU→GPU 演进路径、**多宿主分层集成架构（§33）**、**GPU 后端架构（§34）**、**可嵌入性架构（§35）**、MVP 架构与后续演进策略。 |
| 03 | [PRE_Architecture_ADR](docs/03_PRE_Architecture_ADR.md) | 架构决策记录（ADR-001～ADR-012）：为何以响应而非材料标签为索引对象、为何检索只生成候选、为何标准响应与 solver 解耦、为何 3DGS 只是 Observation Backend、Multi-Vector 取舍、存储选型（含图数据库暂缓引入）、post-filter vs pre-filter、宿主适配层不得进入核心、**为何按接入机制分层而非按引擎分层**、**为何要定义自己的坐标与单位规范**、**为何 GPU 设备由宿主注入而非自建**等关键决策的理由与后果。 |
| 04 | [PRE_Traceability_Matrix](docs/04_PRE_Traceability_Matrix.md) | 需求—设计—测试—验收追溯矩阵，全部需求 ID 覆盖，可按 ID 反查对应架构组件、设计章节、测试方式与验收标准。 |
| 05 | [PRE_Risk_Issue_Register](docs/05_PRE_Risk_Issue_Register.md) | 风险登记簿（R-01～R-14）与架构自审问题登记（ISS-001～ISS-012，含「多宿主+GPU 规划是否违反最小可验证架构原则」的自审），标注严重度、影响、证据与建议，关键项（如参数不可辨识暴露、检索召回监控、验证窗口切分、Bevy 版本演进风险）已回填至基本设计书正文或如实登记为待观察项。 |
| 06 | [PRE_MVP_Experiment_Plan](docs/06_PRE_MVP_Experiment_Plan.md) | MVP 假设验证实验计划（H1～H5）：相似响应是否在 embedding 空间中更近、跨 solver 检索是否有效、ANN 是否显著降低搜索成本、检索+验证是否优于单独检索、噪声鲁棒性；含性能基准目标。 |
| 07 | [PRE_Glossary](docs/07_PRE_Glossary.md) | 项目术语表。 |
| 08 | [PRE_Detailed_Design](docs/08_PRE_Detailed_Design.md) | 详细设计书。逐 crate 展开 02 号文档的架构：核心数据结构完整字段定义、SolverPlugin trait 与 Rigid/XPBD/MPM 各 solver 的内部算法（积分方法、约束求解、to_standard_response 映射规则）、特征提取与 Encoder 算法细节、SQLite/HNSW/Blob 存储 schema、检索融合公式、验证流水线（含窗口切分与参数不可辨识检测算法）、局部搜索优化器、数据集生成器、**宿主中立契约详细设计（PlaybackCursor 插值与五类边界、QuerySession 状态机、SpatialConvention 换算）**、**Bevy / Godot 两个 Tier 1 适配层**、**Tier 2 C ABI 与 Tier 3 PyO3 边界形态**、**GPU 设备注入与后端抽象**、错误码一览与三条核心处理路径的完整时序。 |
| 09 | [PRE_Test_Case_List](docs/09_PRE_Test_Case_List.md) | 测试用例一览。TC-CORE/SOLVER/SIG/ENC/ATLAS/RET/VER/REF/GEN/**ENG/BEVY/GODOT/CONF/GPU/EMB**/E2E 十六类共 77 条用例，每条含前置条件、输入、期望输出与需求/设计出处，供实现阶段直接转化为自动化测试；不含 H1～H5 假设验证实验（见 06 号文档）。 |
| 10 | [PRE_Testkit_Requirements](docs/10_PRE_Testkit_Requirements.md) | **Mock 项目需求定义书**。`pre-testkit`（自动测试用 Mock/Fixture 项目）的背景、目标、非目标（不替代真实集成测试）、独立需求 ID（PRE-TK-001~010：仅限 dev-dependency、Mock SolverPlugin、内存版 Atlas、Fixture Builder、Golden Dataset、近似断言辅助、Mock Bevy Harness）与验收标准。 |
| 11 | [PRE_Testkit_Basic_Design](docs/11_PRE_Testkit_Basic_Design.md) | **Mock 项目基本设计书**。`pre-testkit` 的依赖方向（叶子 crate，仅 dev-dependency）、双层测试策略（Mock 层验证逻辑正确性 / 真实层验证数值与集成正确性，且强制要求关键行为在真实层兜底）、六大组件设计、Golden Dataset 的确定性与版本绑定策略。 |
| 12 | [PRE_Testkit_Detailed_Design](docs/12_PRE_Testkit_Detailed_Design.md) | **Mock 项目详细设计书**。`MockSolverPlugin`（可编程注入数值发散/仿真失败）、`InMemoryAtlas`（含 blob 读取计数器验证 metadata/blob 分离）、`FixtureBuilder`、`GoldenDataset`（RON 格式，由真实 solver 生成而非手工编造）、近似相等断言、`MockBevyHarness`（headless）的具体类型与算法，末尾附与 09 号文档测试用例的映射表。 |

## 核心设计原则

- **Physics First**：AI/检索不能替代物理验证，检索只产生候选，最终答案必须经过正向仿真验证。
- **Response First**：核心索引对象是「条件 + 作用 → 响应」，而非材料名称标签。
- **Structured Physics Before Embedding**：先建立可解释的结构化 Physical Signature，再编码为 Embedding，禁止用通用文本 embedding 替代。
- **最小可验证架构**：每个主要抽象必须回答「解决了哪个已被实验观察到的问题」，未验证不引入（图数据库、GPU-only、分布式、微服务化等均延后到有明确触发条件时再评估）。
