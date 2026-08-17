# Glossary

## 文书管理表

| 项目 | 内容 |
|---|---|
| 文书编号 | PRE-DOC-07 |
| 版本 | v0.1.1 |
| 状态 | Draft |

## 改订履历

| 版本 | 日期 | 变更内容 |
|---|---|---|
| v0.1 | 2026-08-17 | 初版术语表 |
| v0.1.1 | 2026-08-17 | 新增 Bevy 集成相关术语（Engine Adapter / pre-bevy / Landmark / Playback） |

---

- **PRE (Physical Retrieval Engine)**：本项目，构建可计算、可索引、可检索、可验证、可持续积累经验的物理运行时。
- **Physics Experience**：一次可重放的物理实验记录，包含初始状态、激励、材料、约束、solver、参数与响应。
- **Raw Solver State**：某个具体 solver 内部的私有状态表示，不对外统一。
- **Standard Physical Response**：solver-independent 的标准化物理响应表示，是四层分离架构的第二层。
- **Physical Signature**：从 Standard Physical Response 提取的结构化、可解释特征集合，第三层。
- **Physical Embedding**：由 Physical Signature 编码得到的向量表示，用于 ANN 检索，第四层，不可逆。
- **Multi-Vector Representation**：将物理响应编码为多个语义子空间向量（如 behavior/deformation/temporal），而非单一向量。
- **Physics Atlas**：存储与索引全部 Physics Experience 的系统（metadata + vector + blob 三种存储的组合）。
- **Observation Backend**：将不同观测来源（仿真留出集、未来的 3DGS/RGBD/LiDAR）统一转换为 ObservedPhysicalResponse 的接口层。
- **Candidate**：检索返回的候选 Physics Experience，尚未经过仿真验证。
- **Simulation Verification**：对候选重新仿真并与观测比较误差的过程，是唯一的最终真值判定手段。
- **CandidateScore**：融合检索相似度、预测精度、物理一致性、数值稳定性、计算成本、模型复杂度的多维加权评分。
- **Parameter Refinement**：在候选参数邻域内做局部/全局优化，寻找更精确解释的过程。
- **Novel Dynamics**：检索相似度低或仿真误差高于阈值时判定的"数据库无法解释"的观测。
- **Validation Status（Candidate/Validated/Trusted）**：Physics Experience 的可信度生命周期。
- **Solver Plugin**：实现统一 trait 接入的具体物理求解器（Rigid/XPBD/MPM/FEM等）。
- **ANN (Approximate Nearest Neighbor)**：近似最近邻检索，本项目 MVP 采用 HNSW。
- **HNSW**：Hierarchical Navigable Small World，一种图结构 ANN 索引算法。
- **Pre-filter / Post-filter**：metadata 过滤与向量检索的组合顺序；MVP 采用 post-filter（先检索后过滤）。
- **Provenance**：记录数据来源（仿真/观测）、生成系统、时间、版本等溯源信息。
- **Determinism**：同硬件同版本下可重放并复现在容差内一致结果的能力，不承诺跨硬件位级一致。
- **Encoder Version / Solver Version / Signature Schema Version**：三类独立版本号，用于管理 Atlas 数据的兼容性与迁移。
- **H1~H5**：MVP 阶段五条核心待验证假设，见 06_PRE_MVP_Experiment_Plan.md。
- **ADR (Architecture Decision Record)**：架构决策记录，见 03_PRE_Architecture_ADR.md。
- **ISS-XXX**：架构自审登记的具体问题条目，见 05_PRE_Risk_Issue_Register.md。
- **Bevy**：Rust 原生、ECS（Entity-Component-System）架构的开源游戏引擎，本项目第一个 Engine Adapter 的对接目标。
- **Engine Adapter**：连接 PRE 核心与某个外部引擎/渲染系统的独立可选 crate（如 `pre-bevy`），核心 crate 对其零依赖，架构地位与 Observation Backend 同构（数据流方向相反）。
- **pre-bevy**：本项目的 Bevy Engine Adapter crate，唯一允许依赖 `bevy` 的 crate，详见 02_PRE_Basic_Design.md §33 与 03_PRE_Architecture_ADR.md ADR-009。
- **LandmarkId**：跨 solver 统一的响应采样点标识，不依赖具体 solver 内部粒子/顶点索引，见 08_PRE_Detailed_Design.md §3.3；也是 `pre-bevy` 回放时 Bevy 实体与 PRE 响应数据建立映射的键。
- **Playback（回放）**：将 `StandardPhysicalResponse` 的离散采样点，通过插值转换为连续的 Bevy 实体 `Transform` 动画的过程。
