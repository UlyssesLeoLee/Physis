# Physical Retrieval Engine（PRE）基本设计书

版本：v0.1 draft
输入基线：01_PRE_Requirements.md
关联：03_PRE_Architecture_ADR.md（决策理由）、04_PRE_Traceability_Matrix.md

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
```

依赖方向单向：`pre-solver-*` 与 `pre-signature/pre-encoder/pre-retrieval` 互不依赖具体实现，仅通过 `pre-core` 的 trait 交互（对应 PRE-FR-003）。

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

四层严格分离（对应 PRE-FR-003、需求第9节）：

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

MVP 覆盖需求第3节八个特征域中的子集（其余留 Open Question，非阻塞）：

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

MVP 子向量（对应需求第5节，选取三类降低复杂度）：

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

**图数据库结论（对应需求第19节 + ADR-006）**：V0.1 不引入图数据库。Entity/Relation/Field 结构在 MVP 规模下用 relational store 的外键/JSON字段即可表达，图查询模式（多跳遍历）尚未被 MVP 场景要求。留 Open Question，待 Phase 3（Inverse Physics 复杂约束图）重新评估。

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

V0.1 唯一实现：`SimulationBackend`（用留出仿真的 Standard Physical Response 冒充"观测"，用于验证 H1~H5 而不依赖真实数据）。`Dynamic3DGSBackend` 等仅在 trait 层面预留签名，不实现（NG2，对应需求第12节的解耦要求）。

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
    stage_timings{encode, search, simulate, verify, refine}
}
```

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

每次架构演进必须回答「解决了哪个已被实验观察到的问题」（需求第33节原则）。已知触发条件（非承诺时间表）：

- 检索延迟超出 PRE-VEC-001 目标 → 评估 PQ/量化或分片索引。
- Metadata 组合导致 post-filter 召回不足 → 评估 pre-filter 或专用过滤索引。
- Multi-hop 关系查询需求出现（如约束图上的复杂查询）→ 重新评估图数据库（ADR-006 触发条件）。
- 单机计算成为瓶颈 → 引入分布式调度（不早于验证 H1~H5）。
- H1 不成立或边界不清 → 回退到更简单的 Signature（先确定性特征、后学习式表示，见 PRE-ML-002 的回退路径要求）。

## 32. ADR List

见 03_PRE_Architecture_ADR.md：ADR-001~ADR-006（含新增 ADR-007：Post-filter vs Pre-filter；ADR-008：图数据库暂缓引入）。
