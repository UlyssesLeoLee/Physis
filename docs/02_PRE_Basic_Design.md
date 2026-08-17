# Physical Retrieval Engine（PRE）基本设计书

## 文书管理表

| 项目 | 内容 |
|---|---|
| 文书编号 | PRE-DOC-02 |
| 文书名称 | Physical Retrieval Engine 基本设计书 |
| 版本 | v0.1.4 |
| 状态 | Draft — 基于 01号文档 v0.1.4 需求基线编制，尚未经承认 |
| 输入基线 | 01_PRE_Requirements.md（需求变更后须重新走查本文书是否受影响） |
| 关联文书 | 03（ADR，决策理由）、04（追踪矩阵）、05（自审发现的 ISS 已回填至相应章节） |

## 改订履历

| 版本 | 日期 | 变更内容 | 作成者 |
|---|---|---|---|
| v0.1 | 2026-08-17 | 初版：32章节架构设计 | Claude |
| v0.1.1 | 2026-08-17 | 补充文书管理表、承认栏；按自审结果（05号文档 ISS-006/009）在 §13/§26 补充参数不可辨识暴露机制与召回不足监控说明 | Claude |
| v0.1.2 | 2026-08-17 | 响应 01号文档 v0.1.2 新增的 Bevy 集成需求（PRE-BEVY-001~006）：§4 追加 `pre-bevy` crate，新增 §33 Engine Integration Architecture | Claude |
| v0.1.3 | 2026-08-17 | 交叉审查修正：§33.5 的 bevy 依赖检查范围由枚举 4 个 crate 改为「除 pre-bevy 外的全部 workspace 成员」，并要求检查脚本从成员列表动态枚举 | Claude |
| v0.1.4 | 2026-08-17 | 响应 01号文档 v0.1.4：§4 追加 pre-engine-api/pre-godot/pre-ffi/pre-python/pre-gpu；§33 由 Bevy 专属重写为多宿主分层集成架构；新增 §34 GPU 后端架构、§35 可嵌入性架构 | Claude |

## 承认栏

| 角色 | 承认日期 | 签署 |
|---|---|---|
| 项目负责人（ST-05） | — | 未承认 |
| Rust系统负责人（ST-03） | — | 未承认 |

---

## 1. Architecture Overview

PRE 遵循单一主线：

```
Observation → Encode → Retrieve → Simulate → Verify → Learn → Physics Atlas
```

Retrieval 只负责生成候选（candidate generation），从不单独决定最终答案（对应 PRE-VEC-002，ADR-002）。Simulation 是唯一真值验证手段（ADR-001 的直接推论）。MVP 架构刻意分层单薄：单机、单进程可跑通全链路，插件化仅体现在 solver/material/constraint/field/collision 与 encoder/optimizer 两组扩展点，不做微服务化（对应约束 C-02）。

## 2. Context Diagram

```
        Experiment Definition (人工/程序化)
                    │
                    ▼
        ┌───────────────────────┐
        │   PRE Runtime (Rust)   │◄──── Dataset Generator (batch)
        │                        │
        │  Solver Plugins        │
        │  Response Normalizer   │
        │  Signature Extractor   │
        │  Physical Encoder      │
        │  Physics Atlas (store) │
        │  Retrieval Engine      │
        │  Verification Engine   │
        │  Refinement Engine     │
        └───────────────────────┘
                    │
                    ▼
        Best Physical Explanation + Evidence
                    │
                    ▼
        下游消费者（研究者 / 未来3DGS / 下游应用）

未来（Phase 2+, 不在 V0.1 范围）:
        ObservationBackend (Dynamic3DGS/4DGS/RGBD/LiDAR) ──► ObservedPhysicalResponse
```

## 3. Container Diagram（MVP）

MVP 只有一个部署单元：`pre-runtime`（单机 Rust 进程 + 本地/嵌入式存储）。子容器（逻辑边界，非物理部署）：

- `pre-solver`：Rigid/XPBD/MPM/FEM(stub) solver 插件宿主
- `pre-response`：Standard Physical Response 归一化
- `pre-signature`：Physical Signature 特征提取
- `pre-encoder`：Physical Encoder（V1 deterministic）
- `pre-atlas`：存储访问层（relational + vector + blob，见第 17 节）
- `pre-retrieval`：ANN 检索 + hybrid filter
- `pre-verify`：Simulation Verification + CandidateScore
- `pre-refine`：Parameter Refinement（optimizer 插件）
- `pre-gen`：Dataset Generator（批量调度，进程内多线程/多进程）
- `pre-cli`：命令行入口（V0.1 以此代替完整 REST API，对应 PRE-API-002）

不做独立部署/独立数据库实例，避免过早微服务化（NG6）。

## 4. Component Diagram（核心 crate 划分建议）

```
pre-core        # PhysicsExperience, StandardPhysicalResponse, PhysicalSignature 等核心类型
pre-solver-api  # SolverPlugin trait + Response 转换 trait
pre-solver-rigid
pre-solver-xpbd
pre-solver-mpm
pre-solver-fem-stub   # 仅接口与数据模型
pre-signature   # 特征提取
pre-encoder     # V1 deterministic encoder（可选 pre-encoder-ml 后续独立 crate）
pre-atlas       # 存储访问抽象 + 具体后端实现
pre-retrieval   # ANN + hybrid filter + multi-vector fusion
pre-verify      # 重仿真 + 误差计算 + CandidateScore
pre-refine      # optimizer 插件（local search / CMA-ES）
pre-gen         # dataset generator + sampler
pre-cli         # CLI/命令入口
pre-engine-api  # 宿主中立的集成契约：回放游标/插值、查询会话、中立变换类型、坐标单位换算（见 §33）
pre-bevy        # Tier 1 适配层：Bevy（依赖 bevy + pre-engine-api）
pre-godot       # Tier 1 适配层：Godot 4 GDExtension（依赖 godot + pre-engine-api）
pre-ffi         # Tier 2 C ABI 边界（cdylib + 头文件），供 Unity/Unreal 等外语言宿主接入；MVP 仅设计不实现
pre-python      # Tier 3 PyO3 绑定，供 Blender/Maya/Houdini 等 Python 宿主接入；MVP 仅设计不实现
pre-gpu         # GPU 计算后端抽象（wgpu → Vulkan/D3D12/Metal），见 §34；MVP 不实现
```

依赖方向单向：`pre-solver-*` 与 `pre-signature/pre-encoder/pre-retrieval` 互不依赖具体实现，仅通过 `pre-core` 的 trait 交互（对应 PRE-FR-003）。各适配层 crate 是唯一允许依赖对应宿主 SDK（`bevy`/`godot`/`pyo3`）的位置，只能被宿主应用依赖，不得被任何核心 crate 反向依赖（对应 PRE-ENG-002, ADR-009/ADR-010）。适配层之间亦不得相互依赖——它们的共同逻辑一律下沉到 `pre-engine-api`。

## 5. Runtime Architecture

单进程内的数据流（同步调用，MVP 不引入消息队列）：

```
ExperimentDefinition
  → SolverPlugin::step* → RawSolverState (每帧)
  → ResponseNormalizer::normalize(RawSolverState*) → StandardPhysicalResponse
  → SignatureExtractor::extract(StandardPhysicalResponse) → PhysicalSignature
  → PhysicalEncoder::encode(PhysicalSignature) → MultiVectorEmbedding
  → AtlasWriter::store(PhysicsExperience{...})
```

检索/验证路径：

```
ObservedResponse → Signature → Embedding(query)
  → RetrievalEngine::search(query, TopN) → candidates
  → MetadataFilter → TopK
  → VerificationEngine::simulate_and_compare(candidate, ObservedResponse) → errors
  → CandidateScore
  → RefinementEngine::optimize(best candidates) → refined params
  → BestPhysicalExplanation (+ evidence breakdown)
```

## 6. Solver Architecture（插件化）

```rust
trait SolverPlugin {
    fn id(&self) -> SolverId;
    fn version(&self) -> SolverVersion;
    fn init(&self, initial_state: &InitialState, params: &SolverParameters) -> SolverHandle;
    fn step(&self, handle: &mut SolverHandle, dt: f64, substeps: u32) -> RawSolverState;
    fn to_standard_response(&self, history: &[RawSolverState]) -> StandardPhysicalResponse;
}
```

要点（对应 PRE-FR-002/003）：

- 每个 solver 插件自带 `to_standard_response`，把 solver 特定状态（XPBD 约束残差、MPM 粒子/网格双表示、Rigid 刚体状态）转换为统一表示；转换逻辑属于插件，不属于核心。
- Reference 实现（单线程、可读性优先）与后续并行/SIMD 实现必须共享同一 trait，且并行版本需通过与 reference 的数值对齐回归测试（PRE-PHY-002）。
- FEM/Thin Shell 在 V0.1 仅提供 trait 实现的 stub（返回 NotImplemented 或极简线性示例），验证接口可行性即可（NG3）。

## 7. Physics Experience Model

```
PhysicsExperience
├── id, created_at, provenance{source, generator_version, author}
├── InitialState        # geometry ref, initial pose/velocity, discretization
├── BoundaryConditions   # fixed points, attachments, environment
├── Excitation           # applied impulse/force/field over time
├── MaterialModel + MaterialParameters
├── Solver + SolverParameters + seed + determinism_metadata
├── Response             # -> StandardPhysicalResponse (ref to blob storage)
├── PhysicalSignature    # -> ref to structured feature record
├── Embeddings           # -> multi-vector, encoder_version
├── ValidationMetrics    # simulation verification errors (若来自校验流程)
└── ValidationStatus     # Candidate | Validated (V0.1，Trusted 留待 Phase 2, 见 OQ-01)
```

字段组允许为空但不可省略（PRE-DATA-001）；Response/大体积数据以引用形式存在于记录中，实体存于 blob storage（PRE-DATA-002）。

## 8. Standard Physical Response Model（关键设计）

四层严格分离（对应 PRE-FR-003、PRE-PHY-001）：

1. **Raw Solver State**：solver 私有，不对外暴露格式约束。
2. **Standard Physical Response**：solver-independent，字段：
   - `position(t)`, `velocity(t)`, `acceleration(t)`（采样点/代表点，非全场必需，MVP 可用 landmark 采样代替全场，降低维度）
   - `deformation(t)`（若适用：stretch/compression/shear/bending 摘要统计，而非全网格张量）
   - `energy(t)`, `momentum(t)`（标量时间序列）
   - `contact_events[]`, `constraint_events[]`（离散事件表）
   - `spatial_response_field`（V0.1 可选，降采样网格/点云）
   - MVP 明确豁免：`frequency spectrum`/`damping curve` 作为 Signature 阶段派生量而非 Response 原始字段，避免 Response 层过重（工程简化，记入自审 ISS 观察项）。
3. **Physical Signature**：由 Response 派生的结构化特征（见第9节），含 PRE-PHY-001 要求的最小字段集。
4. **Physical Embedding**：由 Signature 编码得到的向量，不可逆、不用于人工检查（人工检查用 Signature 层）。

**MVP 明确简化（对应自审 ISS-流程，需在设计中标注）**：Landmark 采样代替全场记录，是为控制存储与计算成本的工程取舍，若消融实验证明信息丢失显著影响 H1，需要在后续迭代恢复更高分辨率字段。

## 9. Physical Signature Design

MVP 覆盖 PRE-FR-004 所要求的八个特征域中的子集（其余留 Open Question，非阻塞）：

| 域 | MVP 覆盖字段（示例） |
|---|---|
| Geometry | bounding volume, topology class (point/curve/surface/volume), approx thickness |
| Kinematics | displacement/velocity/acceleration 统计量（均值/峰值/RMS） |
| Deformation | strain 统计量, 曲率变化（若适用）；来自 Response.deformation(t) 聚合 |
| Temporal Response | 振荡频率（FFT 主频）、阻尼率估计、恢复时间——由 Response 时间序列派生 |
| Contact | 接触次数、平均恢复系数估计、平均穿透深度 |
| Material Behavior | 由 MaterialModel/Parameters 直接带入（非从响应反推，MVP 简化） |
| Constraints | 约束类型枚举 + 约束违反统计 |
| External Field | 施加场类型 + 强度统计 |

特征提取是确定性函数（无学习参数），保证可测试、可解释（PRE-ML-001）。

## 10. Embedding Architecture（Multi-Vector）

MVP 子向量（对应 PRE-FR-005 与 PRE-VEC-004，选取三类降低复杂度）：

- `behavior_vector`：宏观运动学统计特征拼接（低维，~16-32 dim）
- `deformation_vector`：形变/材料相关特征拼接
- `temporal_vector`：频域/阻尼/事件时序特征拼接
- `global_vector`：上述子向量拼接后的降维投影（PCA，用于粗召回）

V1 Encoder：确定性特征向量 + 每子空间独立归一化（z-score，基于 Atlas 统计量），不使用神经网络（PRE-ML-001）。

Fusion / Scoring（不等于单一 cosine）：

```
retrieval_similarity =
    Wg × cos(global_vector) 用于粗召回（ANN 索引维度）
候选精排阶段:
    behavior_sim, deformation_sim, temporal_sim 分别计算
    → 加权融合，权重可配置（PRE-VEC-002, PRE-FR-009）
```

粗召回只用 `global_vector` 建 ANN 索引（控制索引维护成本），精排阶段对 Top-N 候选逐一计算子向量相似度（计算量可控，因为 N 远小于全库规模）。

## 11. Vector Retrieval Architecture

MVP 规模 10K~100K：选用 **HNSW**（如 `hnsw_rs` 或等价 Rust 实现）作为 `global_vector` 索引，理由：

- 该规模下 HNSW 召回率/延迟优于 IVF（IVF 需要更大规模才能摊薄训练/聚类开销）。
- 不使用 PQ（量化）：MVP 向量维度小、规模小，PQ 收益有限且增加误差，留作百万级以上的优化项（Open Question 记录，非 MVP 阻塞）。
- Metadata filter 采用 **post-filter**（先 ANN 召回 Top-N，再按 metadata 过滤到 Top-K）：理由是 MVP 规模小，pre-filter 需要为每种 metadata 组合维护子索引，工程复杂度不划算；若未来规模增长到需要 pre-filter，需重新评估（ADR 记录）。

具体数据库/库的最终选型不早锁：候选包括嵌入式 Rust 原生实现 vs 绑定 FAISS/usearch，选型标准与决策见 ADR-006。

## 12. Physics Atlas Architecture

Atlas = 三种存储的组合视图，而非单一数据库：

- **Relational/Document store**（metadata + PhysicsExperience 结构化字段）：候选 SQLite（MVP 单机简单可靠）或嵌入式 KV+索引；不用 Postgres 等需要独立部署的方案，避免过早分布式化。
- **Vector store**（embedding）：MVP 阶段与 relational store 同进程，索引持久化为本地文件（HNSW graph snapshot）。
- **Blob store**（大体积 Response 原始数据）：本地文件系统，按 experience_id 分片目录，MVP 不引入对象存储服务。

三者通过 `experience_id` 关联，查询 metadata/embedding 不触发 blob 读取（PRE-DATA-002）。

**图数据库结论（对应 PRE-DATA-004 + ADR-006）**：V0.1 不引入图数据库。Entity/Relation/Field 结构在 MVP 规模下用 relational store 的外键/JSON字段即可表达，图查询模式（多跳遍历）尚未被 MVP 场景要求。留 Open Question，待 Phase 3（Inverse Physics 复杂约束图）重新评估。

## 13. Simulation Verification Pipeline

```
Candidate(solver, material_params, constraints)
  → Re-simulate (相同 InitialState/Excitation/BoundaryConditions as observation)
  → Predicted Response (Standard Physical Response)
  → 与 Observed Response 逐维比较:
       position_error, velocity_error, deformation_error,
       frequency_error, damping_error, contact_timing_error
  → PhysicsVerificationScore = 加权聚合（权重可配置，PRE-NFR-004）
```

MVP 明确不实现的误差维度（记 Open Question，非阻塞）：`topology_error`（拓扑变化误差，MVP solver 集合基本不涉及拓扑改变/断裂）。

**匹配窗口 / held-out 未来窗口分离（对应自审 ISS-009 修正）**：Observed Response 在进入本 Pipeline 前必须先切分为匹配窗口（前段，供检索与 Signature/Embedding 生成使用）与 held-out 未来窗口（后段，仅在本节的"与 Observed Response 逐维比较"步骤中使用，不参与检索或参数猜测）。若不做此切分，Verification 实质上是在检验"能否拟合已观测到的同一段轨迹"，而非"能否预测未见过的未来状态"，二者对 H4 假设的证明力完全不同。具体窗口比例与实验设计见 06_PRE_MVP_Experiment_Plan.md。

**参数不可辨识暴露机制（对应自审 ISS-006 修正）**：Verification 阶段除计算单一 PhysicsVerificationScore 外，必须对 Top-M 候选（进入 Refinement 前的候选集合）额外统计其材料/求解器参数在参数空间中的分散度（如各参数维度的标准差或极差）。若分散度超过预设阈值（即"多组参数产生几乎相同误差"），Explanation 输出必须显式标注 `identifiability: low`，并列出全部 Top-M 候选及其参数，而非仅返回单一 best——避免向下游呈现虚假的唯一确定性。该阈值与 CandidateScore 权重同属 PRE-NFR-004 的可配置项。

## 14. Parameter Refinement Architecture

```
trait ParamOptimizer {
    fn optimize(&self, initial_guess: Params, objective: impl Fn(&Params) -> f64, budget: Budget) -> Params;
}
```

MVP 实现一种：优先 **局部搜索（coordinate descent / Nelder-Mead 简化版）**，理由：实现成本低、确定性、易于与 reference solver 数值对齐调试；CMA-ES 作为 Phase 2 候选（若局部搜索在消融实验中收敛质量不足）。Optimizer 通过 trait 插件化，替换不影响上层调用方。

## 15. Observation Backend Interface（预留，不实现）

```rust
trait ObservationBackend {
    fn observe(&self, source: ObservationSource) -> ObservedPhysicalResponse;
}
```

V0.1 唯一实现：`SimulationBackend`（用留出仿真的 Standard Physical Response 冒充"观测"，用于验证 H1~H5 而不依赖真实数据）。`Dynamic3DGSBackend` 等仅在 trait 层面预留签名，不实现（对应非目标 NG2 与 PRE-FR-015 的解耦要求）。

## 16. Future Dynamic 3DGS Adapter（占位）

未来 Adapter 职责边界（不在 V0.1 实现，仅记录接口约定）：Gaussian tracking → motion field → physical feature extraction → 必须落到与仿真侧完全相同的 `StandardPhysicalResponse` schema，才能进入现有 Signature/Encoder 流程。这是保证 ADR-004（3DGS 是 Observation Backend 而非核心耦合）成立的关键约束，需在 Phase 2 设计中重申。

## 17. Data Architecture

```
PhysicsExperience (relational, metadata + refs)
        │
        ├── Response blob (file, referenced by experience_id + response_hash)
        ├── Embedding vector (vector store, referenced by experience_id + encoder_version)
        └── ValidationMetrics (relational, small, inline)
```

Encoder 多版本共存：embedding 记录带 `encoder_version`，检索时按 `encoder_version` 分索引，不跨版本混合检索（PRE-ML-003, PRE-REPRO-002）。

## 18. Storage Architecture

MVP：单机本地存储（SQLite + 本地 HNSW 索引文件 + 本地文件系统 blob），无网络存储依赖。扩展路径（不在 MVP 实现，仅架构预留）：

```
10K~100K (MVP)：单机 SQLite + 内存/本地 HNSW
1M：单机，索引常驻内存，blob 走本地高速盘或简单对象存储
100M：分片向量索引 + 分布式 metadata store（需重新评估图数据库必要性）
1B：分布式 ANN（IVF+PQ 或分片 HNSW）+ 专用向量数据库
```

每一级演进都是独立评估触发的（性能/规模超阈值），不在 MVP 提前实现（NG6，Evolution Strategy 见第31节）。

## 19. API Architecture

MVP 以 Rust 库 API + CLI 暴露职责（对应 PRE-API-001/002），职责与数据流（不锁 schema）：

| 职责 | 输入 | 输出 |
|---|---|---|
| create experience | ExperimentDefinition | PhysicsExperience(id) |
| simulate | Experiment params | RawSolverState history |
| encode | StandardPhysicalResponse | Embedding |
| search | Query embedding + filter | TopN candidates |
| verify | Candidate + Observed | VerificationScore |
| refine | Candidate + budget | Refined params |

REST 化为后续独立任务，不在 V0.1 设计中锁定路由/schema 细节。

## 20. Scheduling Architecture

MVP：进程内线程池并行执行 Dataset Generator 的批量仿真任务（每个 Experiment 独立、无共享可变状态，天然可并行）。不引入任务队列服务/分布式调度器（NG6）。

## 21. CPU/GPU Architecture

```
Reference Solver (单线程 CPU, 正确性优先)
        ↓ 数值对齐回归测试
Parallel CPU (rayon 等，MVP 可选)
        ↓
SIMD（Open Question，非 MVP 阻塞）
        ↓
GPU（Phase 2+，需先证明 CPU 路径成为瓶颈）
```

GPU 实现不得成为唯一真值来源（PRE-PHY-002 的架构落实）；MVP 不做 GPU compute。

## 22. Error Handling

- Solver 数值发散（NaN/Inf）在 `step` 边界检测，标记 Experience 为 `Invalid`，不进入 Atlas（PRE-REL-001）。
- Verification/Refinement 失败返回结构化错误（含阶段、原因），不吞异常（PRE-REL-002）。
- 错误类型按 crate 划分（`pre-solver` 错误 ≠ `pre-atlas` 错误），避免核心类型污染。

## 23. Versioning

三类独立版本号：`solver_version`、`encoder_version`、`signature_schema_version`。任意一个变化都可能使历史 Experience 的 embedding 失效或需要迁移，Atlas 需记录三者并支持按版本过滤/迁移任务（离线批处理，非实时）。

## 24. Provenance

每条 PhysicsExperience 记录 `provenance{source(simulation|observation), generator, author/system, created_at, parent_experience_id(若来自refinement派生)}`，用于追溯与审计（对应 PRE-FR-012, Observability）。

## 25. Determinism

同机同版本可重放（PRE-REPRO-001）；不承诺跨硬件位级一致。并行/SIMD 实现允许与 reference 有数值容差内偏差，但容差必须显式配置并记录（对应 PRE-PHY-002 的回归测试基线）。

## 26. Observability

统一 `CandidateExplanation` 结构（对应 PRE-OBS-001）：

```
CandidateExplanation {
    experience_id,
    retrieval_score, behavior_sim, deformation_sim, temporal_sim,
    simulation_error{position, velocity, deformation, ...},
    stability_score, computational_cost, confidence,
    identifiability,          // low | normal，见 §13 参数不可辨识暴露机制
    stage_timings{encode, search, simulate, verify, refine}
}
```

**检索召回不足监控（对应自审 ISS-007 修正）**：`pre-retrieval` 在 post-filter（§11, ADR-007）执行后，若过滤后剩余候选数低于可配置阈值（`PRE-NFR-004` 管辖的配置项之一），必须记录一条结构化日志（含触发时的 metadata 过滤条件、ANN 召回数 N、过滤后剩余数），供后续判断是否需要转向 pre-filter 或增大 N。此为观测手段，不改变 MVP 的 post-filter 决策本身。

## 27. Testing Strategy

- 单元测试：`pre-core` 数据结构、`pre-signature` 特征提取函数（纯函数，易测）。
- 数值回归测试：reference solver 输出快照 + 容差比较；并行实现对齐测试。
- 集成测试：端到端 pipeline（synthetic observation → explanation），覆盖 AC-01/AC-05。
- 假设验证实验：H1~H5，作为独立可重复运行的 benchmark（脚本化，见06号文档），不是普通单元测试。

## 28. Benchmark Strategy

见 06_PRE_MVP_Experiment_Plan.md；核心指标：检索延迟（PRE-VEC-001）、生成单条 Experience 耗时（PRE-PERF-001）、H3 的仿真次数对比。

## 29. Deployment Model

MVP：单机命令行工具/库，无服务化部署。CI 中跑单元测试 + 数值回归 + 端到端 smoke test。

## 30. MVP Architecture（摘要图）

```
┌─────────────── pre-runtime (single process) ───────────────┐
│  Dataset Generator → Solver Plugins(Rigid/XPBD/MPM)         │
│        → Response Normalizer → Signature Extractor          │
│        → Encoder(V1 deterministic) → Atlas(SQLite+HNSW+FS)  │
│                                                               │
│  Query: Observed(=heldout sim) → Signature → Embedding       │
│        → Retrieval(HNSW+post-filter) → Verify(re-sim)        │
│        → CandidateScore → Refine(local search) → Explanation │
└───────────────────────────────────────────────────────────┘
```

## 31. Evolution Strategy

每次架构演进必须回答「解决了哪个已被实验观察到的问题」（本项目「涌现式设计」原则：Hypothesis → Minimal Architecture → Experiment → Evidence → Architecture Evolution；无法回答者不加入）。已知触发条件（非承诺时间表）：

- 检索延迟超出 PRE-VEC-001 目标 → 评估 PQ/量化或分片索引。
- Metadata 组合导致 post-filter 召回不足 → 评估 pre-filter 或专用过滤索引。
- Multi-hop 关系查询需求出现（如约束图上的复杂查询）→ 重新评估图数据库（ADR-006 触发条件）。
- 单机计算成为瓶颈 → 引入分布式调度（不早于验证 H1~H5）。
- H1 不成立或边界不清 → 回退到更简单的 Signature（先确定性特征、后学习式表示，见 PRE-ML-002 的回退路径要求）。

## 32. ADR List

见 03_PRE_Architecture_ADR.md：ADR-001~ADR-006（含新增 ADR-007：Post-filter vs Pre-filter；ADR-008：图数据库暂缓引入；ADR-009：Bevy 是 Engine Adapter 而非核心依赖）。

## 33. Host Integration Architecture（多宿主分层集成架构）

### 33.1 分层模型与依赖方向

PRE 是被宿主嵌入的运行时。宿主横跨 Rust / C++ / C# / Python 四种生态，无法用单一接入机制覆盖，故按**接入机制**分为四层（对应 PRE-ENG-001）：

```
                    ┌──────────────── pre-core ────────────────┐
                    │  PhysicsExperience / StandardPhysical-   │
                    │  Response / Signature / Embedding …      │
                    └──────────────────┬───────────────────────┘
                                       │ (Rust 类型，无宿主依赖)
                    ┌──────────────────▼───────────────────────┐
                    │            pre-engine-api                │
                    │  ★ 宿主中立的集成逻辑集中地：             │
                    │    PlaybackCursor（采样点插值）           │
                    │    QuerySession（异步查询状态机）         │
                    │    LandmarkTransform（中立变换类型）      │
                    │    SpatialConvention（坐标/单位换算）     │
                    └───┬───────────┬──────────┬───────────┬────┘
        Tier 1 (Rust)   │           │  Tier 2  │  Tier 3   │
        ┌───────────────▼──┐  ┌─────▼─────┐  ┌─▼────────┐ │
        │    pre-bevy      │  │ pre-godot │  │ pre-ffi  │ │  pre-python
        │   (bevy crate)   │  │  (gdext)  │  │ (C ABI)  │ │   (PyO3)
        └────────┬─────────┘  └─────┬─────┘  └─┬──────┬─┘ └────┬─────┘
                 ▼                  ▼          ▼      ▼        ▼
             Bevy App          Godot 项目   Unreal  Unity   Blender/Maya/
                                            (C++)   (C#)    Houdini
```

依赖方向严格单向向上，且**适配层之间零依赖**：任何被两个以上适配层需要的逻辑，一律下沉到 `pre-engine-api`，而不是从一个适配层引用另一个。

### 33.2 `pre-engine-api`：中立契约（本次架构的核心）

这一层的存在理由，是把「所有宿主都要做、且做法应当相同」的逻辑收敛到一处。归入本层的有四类：

| 归入 `pre-engine-api` | 为何不能留在适配层 |
|---|---|
| **PlaybackCursor**：在 `sample_times` 中定位当前时刻并插值 | 四个宿主各写一遍插值，等于同一个边界 bug（早于首帧/晚于末帧/单采样点）有四份不一致的实现 |
| **QuerySession**：提交请求→轮询结果的状态机 | 状态机语义（何时算完成、失败如何表达、能否取消）必须跨宿主一致，否则同一份文档无法描述所有宿主的行为 |
| **LandmarkTransform**：中立变换类型 | 若直接用宿主类型，`pre-engine-api` 就得依赖宿主 SDK，违反 PRE-ENG-002 |
| **SpatialConvention**：坐标手性/上轴/单位换算 | 见 §33.3，这是跨引擎最易出错且最难归因的一类缺陷 |

留在各适配层的只有两类：**中立类型 ↔ 宿主类型的映射**，以及**宿主特有的调度接入**（Bevy 的 System/任务池、Godot 的 `_process`/线程、Python 的 GIL 处理）。

### 33.3 规范空间与单位约定（PRE-ENG-006）

PRE 规范约定：**右手系、Y 轴向上、-Z 为前方、长度单位 SI 米**。各宿主差异：

| 宿主 | 手性 | 上轴 | 默认长度单位 | 换算复杂度 |
|---|---|---|---|---|
| PRE（规范） | 右手 | Y | 米 | — |
| Bevy | 右手 | Y | 米 | 接近恒等 |
| Godot | 右手 | Y | 米 | 接近恒等 |
| Unity | **左手** | Y | 米 | 需手性翻转 |
| Unreal | **左手** | **Z** | **厘米** | 需手性翻转 + 轴置换 + ×100 |
| Blender | 右手 | **Z** | 米 | 需轴置换 |

`SpatialConvention` 以数据（而非各适配层的散落代码）描述每个宿主的换算，每个适配层声明自己使用哪一个，换算函数集中实现并统一测试。

> 这张表本身就是把 Unity/Unreal 纳入规划的直接收益：如果只做 Bevy 和 Godot（两者都接近恒等换算），换算层很可能被实现成隐式恒等，等到接入 Unreal 时才发现整个数据通路里没有可插入换算的位置。

### 33.4 Tier 1 适配层（Bevy / Godot）

两者都能直接链接 Rust 类型，差异只在宿主的对象模型与调度设施：

- **`pre-bevy`（PRE-ENG-101）**：提供 Bevy `Plugin`；`PreLandmark` 组件标记受控实体；System 每帧从 `PlaybackCursor` 取中立变换写入 `Transform`；查询桥接使用 Bevy `AsyncComputeTaskPool` 驱动 `QuerySession`。
- **`pre-godot`（PRE-ENG-201）**：经 Godot 4 GDExtension 的 Rust 绑定，提供 `Node3D` 派生的回放节点；子节点通过导出属性绑定 `LandmarkId`；`_process` 中仅做「取插值结果 → 写 `Transform3D`」，查询桥接放在独立线程并经 `call_deferred` 回主线程，**不得在 `_process` 中同步等待**。

两者共用同一 `PlaybackCursor` 与 `QuerySession`，因此 AC-07 的一致性验收才有意义——若两者输出不一致，说明有本应中立的逻辑事实上泄漏进了某个适配层。

### 33.5 Tier 2 C ABI 边界（`pre-ffi`，MVP 仅设计）

Unity（C#）与 Unreal（C++）无法消费 Rust 类型，必须经稳定 C ABI。边界约束（PRE-ENG-009）：

- 仅传递**不透明句柄**（`PreContext*`、`PrePlayback*`）与 **POD 结构体**（如 `PreTransform { float pos[3]; float rot[4]; }`）；不跨边界传递 Rust 泛型、trait 对象、带载荷枚举、`Result`、`String`、`Vec`。
- 错误一律为整型错误码；字符串经「调用方提供缓冲区 + 长度」模式取回。
- 所有权与释放责任在头文件中逐函数注明；每个 `create_*` 有配对的 `destroy_*`。
- 提供 `pre_ffi_abi_version()`，宿主启动时校验。
- **每个 `extern "C"` 函数以 `catch_unwind` 包裹**——panic 跨 FFI 边界是未定义行为（PRE-EMB-004）。

MVP 不实现的理由：C ABI 一经发布即难以变更，在没有真实 Unity/Unreal 调用方反馈的情况下固化接口形态，风险高于收益（OQ-08 记录了实现时机的触发条件）。但**边界约束现在就必须写下来**，因为它反向约束了 `pre-engine-api` 的设计——中立契约里若出现无法用 POD 表达的类型，Tier 2 就永远接不上。

### 33.6 Tier 3 Python 绑定（`pre-python`，MVP 仅设计）

面向 Blender/Maya/Houdini。除与 Tier 2 相同的「不跨边界传递 Rust 专有类型」原则外，关键约束是 **GIL 释放**（PRE-ENG-010）：检索与仿真验证是长耗时操作，若持有 GIL 执行，会冻结宿主 3D 软件的整个 UI 线程。该 crate 同时服务于约束 C-01 已允许的 ML/研究工作流。

### 33.7 一致性测试套件（PRE-ENG-008）

所有适配层针对同一条黄金响应数据，经各自 `SpatialConvention` 换算后，产出的变换序列必须在容差内一致。新增适配层时必须通过同一套件，不得各自定义验收口径。套件实现归属 `pre-testkit`（见 11 号文档 §2 的双层测试策略——Tier 1 可在 mock 层跑，真实宿主集成仍需真实层兜底）。

### 33.8 依赖隔离的自动化验证（AC-06）

CI 以 `cargo metadata` **动态枚举** workspace 成员，排除各适配层 crate 自身后，检查其余成员的依赖树不包含任何宿主 SDK（`bevy`/`godot`/`pyo3`）。必须动态枚举而非硬编码名单——否则新增 crate 会自动逃逸出检查范围（这正是 v0.1.3 修复过的缺陷类型）。

## 34. GPU Backend Architecture（GPU 计算后端）

### 34.1 定位

PRE 使用 GPU **仅用于计算**（solver 加速），不承担渲染。这一点决定了架构选择：需要的是 compute pipeline 的可移植抽象，而非完整渲染抽象。

### 34.2 可移植抽象（PRE-GPU-001）

```
        pre-solver-*（solver 实现）
                │  只调用 pre-gpu 的抽象接口
                ▼
            pre-gpu
                │  wgpu（候选，OQ-09 待最终确认）
                ▼
    Vulkan │ Direct3D 12 │ Metal │ (GL 等回退)
```

solver 代码中不得出现任何单一图形 API 的专有调用。选 `wgpu` 的初步理由：Rust 原生、单一抽象即覆盖三大后端、且 Bevy 本身即基于 wgpu（利于 §34.3 的设备共享）。**但选型在 GPU 工作实际启动时才最终确认**（OQ-09），因为需要验证其抽象能否表达 MPM 的 P2G/G2P 所需的原子操作与工作组内存模式。

### 34.3 宿主设备注入（PRE-GPU-002，本节最关键的架构决策）

```rust
enum GpuDeviceSource {
    HostProvided { device: ..., queue: ... },   // 宿主已有设备，PRE 复用
    CreateOwn { preferred_backend: Option<Backend> },  // 无宿主设备时自建
}
```

宿主（游戏引擎、3D 软件）运行时几乎必然已持有 GPU 设备。若 PRE 另建一个：显存重复占用、跨设备同步开销、部分驱动下资源共享直接失败。

**为何这必须现在决定**：「PRE 自建并独占设备」与「PRE 接受注入设备」是两种不兼容的初始化架构。前者会把设备创建、生命周期、错误处理散布在初始化路径各处；改造为后者时，这些位置全部要重写。即使 MVP 不实现 GPU，初始化架构也必须预留设备注入形态——这与 PRE-EMB-001（无全局状态、显式上下文句柄）是同一类约束。

### 34.4 CPU 真值与回退（PRE-GPU-003 / PRE-GPU-005）

- CPU reference 始终是数值真值来源，GPU 实现必须与之在容差内对齐并有自动化回归——这是 PRE-PHY-002 在 GPU 语境的延伸，不是新原则。
- 无 GPU 或初始化失败时自动回退 CPU 并记录原因，不得直接失败退出：PRE 的核心价值（检索与验证）在纯 CPU 下必须完整可用。

### 34.5 零拷贝互操作（PRE-GPU-004，设计预留）

理想路径是 PRE 的计算结果以宿主可直接使用的 GPU 缓冲交付，免去 GPU→CPU→GPU 往返。这依赖外部内存共享（Vulkan external memory、D3D12 shared handle 等），复杂度与宿主耦合度都远高于普通计算后端，且各宿主可达性不同（OQ-10）。

架构上的要求仅有一条，但必须现在遵守：**数据通路设计不得假设结果必然经过 CPU**。若把「结果落到 CPU 内存」写死进 `StandardPhysicalResponse` 的产出路径，未来零拷贝将无处插入。

## 35. Embeddability Architecture（可嵌入性）

PRE 作为被宿主嵌入的库，以下四条约束几乎全部属于「早期不遵守、后期无法补救」：

| 约束 | 架构落实 | 若违反的后果 |
|---|---|---|
| PRE-EMB-001 无全局可变状态 | 所有运行时状态挂在显式 `PreContext` 句柄下，允许同进程多实例 | 3D 软件中插件被多次实例化时相互踩踏；测试无法并行 |
| PRE-EMB-002 不拥有主循环 | 长耗时操作一律提供「分步执行」或「后台执行 + 轮询」形式（与 §33.2 的 `QuerySession` 同一机制） | 宿主主线程被阻塞，UI/渲染卡死 |
| PRE-EMB-003 线程策略可配置 | 线程数上限可配置，或直接复用宿主线程池 | 在对调度有严格约束的 DCC 宿主中造成线程爆炸 |
| PRE-EMB-004 panic 不跨外语言边界 | 所有 `extern "C"` 与 PyO3 入口以 `catch_unwind` 包裹并转换为该语言错误 | 跨边界 unwind 是未定义行为，表现为宿主进程崩溃且难以定位 |

这些约束对 Tier 0（纯 Rust 调用）看似多余，但正是它们使得 Tier 1~3 得以在不改动核心的前提下叠加——它们约束的是核心，不是适配层。
