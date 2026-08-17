# Physical Retrieval Engine（PRE）详细设计书

## 文书管理表

| 项目 | 内容 |
|---|---|
| 文书编号 | PRE-DOC-08 |
| 文书名称 | Physical Retrieval Engine 详细设计书 |
| 版本 | v0.1.2 |
| 状态 | Draft |
| 输入基线 | 02_PRE_Basic_Design.md（v0.1.3）、03_PRE_Architecture_ADR.md（v0.1.2） |
| 关联文书 | 04（追踪矩阵，本文书新增内容追加映射见文末）、09（测试用例一览） |
| 前提 | 本文书仅覆盖 MVP 范围内声明支持的能力（Rigid / XPBD cloth·soft body / MPM elastic，02号文档 §30；以及 §33 定义的 Bevy 回放能力）；FEM 与 Bevy 场景导入方向（PRE-BEVY-005）仍为 stub/Phase 2，不在本文书详细展开 |

## 改订履历

| 版本 | 日期 | 变更内容 | 作成者 |
|---|---|---|---|
| v0.1 | 2026-08-17 | 初版：crate 内部设计、数据结构、trait 签名、核心算法、错误模型、存储 schema | Claude |
| v0.1.1 | 2026-08-17 | 新增 §18 `pre-bevy` 详细设计，响应 02号文档 v0.1.2 新增的 §33 Engine Integration Architecture | Claude |
| v0.1.2 | 2026-08-17 | §18 前提约束表述由「核心六个 crate」改为「除 pre-bevy 外的全部 workspace 成员」，与 PRE-BEVY-001 v0.1.3 定义一致 | Claude |

## 承认栏

| 角色 | 承认日期 | 签署 |
|---|---|---|
| Rust系统负责人（ST-03） | — | 未承认 |
| 物理仿真负责人（ST-01） | — | 未承认 |
| 检索/ML负责人（ST-02） | — | 未承认 |

---

## 目录

1. 详细设计范围与阅读方法
2. `pre-core`：核心数据结构详细定义
3. `pre-solver-api`：SolverPlugin trait 与 Response 归一化
4. `pre-solver-rigid`：Rigid Body 详细设计
5. `pre-solver-xpbd`：XPBD 详细设计
6. `pre-solver-mpm`：MPM 详细设计
7. `pre-signature`：特征提取算法详细设计
8. `pre-encoder`：V1 Encoder 详细设计
9. `pre-atlas`：存储 schema 详细设计
10. `pre-retrieval`：检索与融合算法详细设计
11. `pre-verify`：验证流水线详细设计
12. `pre-refine`：参数优化算法详细设计
13. `pre-gen`：数据集生成器详细设计
14. `pre-cli`：命令行详细设计
15. エラーコード一覧（错误码一览）
16. 处理流程时序（主要 3 条路径的逐步时序）
17. 与 02/04 号文档的追加映射
18. `pre-bevy`：Bevy 引擎适配层详细设计

---

## 1. 详细设计范围与阅读方法

本文书是 02_PRE_Basic_Design.md 的下一层：02 号文档回答"系统由哪些组件构成、组件间如何交互"，本文书回答"每个组件内部的数据结构、接口签名、核心算法、异常处理具体是什么"。

约定：
- 代码块使用 Rust 风格伪代码，字段类型为设计意图表达，非最终实现（详细类型如 `f32` vs `f64`、`Vec` vs `SmallVec` 留给实现阶段，不在本文书锁定，除非有明确工程理由）。
- 每节末尾列出「对应需求 ID」，供追溯。
- 本文书不重复 02 号文档已有的架构级决策（如四层分离、Multi-Vector 选型），仅在需要时引用。

---

## 2. `pre-core`：核心数据结构详细定义

### 2.1 PhysicsExperience

```rust
struct PhysicsExperience {
    id: ExperienceId,                       // UUID v7（时间有序，便于按创建时间范围查询）
    created_at: Timestamp,
    provenance: Provenance,

    initial_state: InitialState,
    boundary_conditions: BoundaryConditions,
    excitation: Excitation,

    material: MaterialSpec,                 // { model: MaterialModelId, params: MaterialParameters }
    solver: SolverSpec,                     // { id: SolverId, version: SolverVersion, params: SolverParameters, seed: u64 }

    response_ref: BlobRef,                  // 指向 StandardPhysicalResponse 的 blob 存储位置，不内联
    signature: PhysicalSignature,           // 结构化特征，体积小，内联存储于 relational store
    embeddings: EmbeddingSet,               // { encoder_version, behavior, deformation, temporal, global }

    validation: ValidationRecord,           // { status: Candidate|Validated, metrics: Option<VerificationMetrics> }

    determinism: DeterminismMetadata,       // { dt, substeps, iterations, hardware_fingerprint }
}

struct Provenance {
    source: ProvenanceSource,               // Simulation | Observation(SimulationBackend 冒充，V0.1)
    generator: GeneratorId,                 // 生成该记录的组件（pre-gen / pre-refine / manual）
    generator_version: String,
    parent_experience_id: Option<ExperienceId>,  // 若由 Refinement 派生自某候选，记录父记录
}
```

字段完整性要求（PRE-DATA-001）：所有字段组必须存在，允许其内部为空/None，但字段组本身不可省略——因此上述结构中不使用 `Option<InitialState>` 之类的整组可选，只在字段组内部字段上使用 `Option`。

### 2.2 InitialState / BoundaryConditions / Excitation

```rust
struct InitialState {
    geometry_ref: GeometryRef,              // 指向几何资源（mesh/point cloud/primitive 描述）
    discretization: DiscretizationSpec,     // 粒子数/网格分辨率/离散化方式，solver 相关但需在此声明供复现
    initial_pose: Transform,
    initial_velocity: VelocityField,        // 均匀速度或逐点速度场（MVP：均匀为主）
}

struct BoundaryConditions {
    fixed_points: Vec<PointConstraint>,
    attachments: Vec<AttachmentConstraint>,
    environment: EnvironmentSpec,           // 地面、边界盒等
}

struct Excitation {
    events: Vec<ExcitationEvent>,           // 施加的力/冲量/场，随时间变化
}

struct ExcitationEvent {
    t_start: f64,
    t_end: Option<f64>,
    kind: ExcitationKind,                   // Impulse | ConstantForce | FieldForce(gravity/wind/pressure)
    magnitude_or_field: FieldSpec,
    target: TargetSelector,                 // 作用于哪些粒子/刚体/区域
}
```

对应需求：PRE-DATA-001, PRE-PHY-004。

### 2.3 StandardPhysicalResponse（四层分离的第二层，详细字段）

```rust
struct StandardPhysicalResponse {
    schema_version: SignatureSchemaVersion,
    duration: f64,
    sample_times: Vec<f64>,                 // 采样时间点（非必须等间隔）

    // Landmark 采样：非全场，代表点集合（对应 02号文档 §8 的工程简化）
    landmarks: Vec<LandmarkId>,
    position: Vec<Vec<Vec3>>,               // [landmark][t] -> position
    velocity: Vec<Vec<Vec3>>,
    acceleration: Vec<Vec<Vec3>>,

    deformation: Option<DeformationSeries>, // 若 solver/材料涉及形变（XPBD/MPM），Rigid 为 None
    energy: Vec<f64>,                       // [t] -> 总能量（动能+势能估计）
    momentum: Vec<Vec3>,                    // [t] -> 总动量

    contact_events: Vec<ContactEvent>,
    constraint_events: Vec<ConstraintEvent>,

    spatial_field_snapshot: Option<SpatialField>,  // 可选：降采样网格/点云快照，用于可视化调试，非检索必需
}

struct DeformationSeries {
    stretch: Vec<f64>,       // 聚合统计量（如均值/峰值），非全网格张量
    compression: Vec<f64>,
    shear: Vec<f64>,
    bending: Option<Vec<f64>>,  // 仅 thin-shell/cloth 类适用
    strain_rms: Vec<f64>,
}

struct ContactEvent {
    t: f64,
    contact_pair: (EntityRef, EntityRef),
    normal: Vec3,
    penetration_depth: f64,
    restitution_estimate: f64,
}
```

**四层分离的强制约束（对应 ADR-003）**：`StandardPhysicalResponse` 的任何字段不得引用 solver 私有类型（如 XPBD 的约束索引、MPM 的粒子 ID）。所有 solver 内部标识符必须在 `to_standard_response()` 转换过程中被丢弃或映射为 `LandmarkId`（一个与 solver 无关的、按几何位置或语义角色分配的稳定标识符，定义见 §3.3）。

对应需求：PRE-PHY-001, PRE-PHY-005。

### 2.4 PhysicalSignature（第三层）

```rust
struct PhysicalSignature {
    schema_version: SignatureSchemaVersion,

    geometry: GeometryFeatures,       // { topology_class, bounding_volume, approx_thickness }
    kinematics: KinematicsFeatures,   // { disp_mean, disp_peak, vel_rms, accel_peak, ... }
    deformation: DeformationFeatures, // { strain_mean, strain_peak, curvature_change, ... }
    temporal: TemporalFeatures,       // { dominant_freq, damping_rate, recovery_time }
    contact: ContactFeatures,         // { contact_count, avg_restitution, avg_penetration }
    material: MaterialFeatures,       // 直接取自 MaterialSpec（非从响应反推，MVP 简化，见02号文档§9）
    constraints: ConstraintFeatures,  // { constraint_type_histogram, avg_violation }
    field: FieldFeatures,             // { field_types, avg_magnitude }
}
```

每个子结构均为**纯数值字段的确定性聚合**，不含指针/引用，保证可独立序列化、可单元测试（PRE-ML-001, PRE-NFR-003）。

### 2.5 EmbeddingSet（第四层）

```rust
struct EmbeddingSet {
    encoder_version: EncoderVersion,
    behavior_vector: Vec<f32>,      // dim ~16-32
    deformation_vector: Vec<f32>,
    temporal_vector: Vec<f32>,
    global_vector: Vec<f32>,        // 拼接后 PCA 降维，用于 ANN 粗召回索引
}
```

不可逆：`EmbeddingSet` 上不定义任何反解到 `PhysicalSignature` 的方法；调试/人工检查一律通过 `PhysicalSignature` 层进行（PRE-ML-002 的回退路径落实）。

对应需求：PRE-DATA-001, PRE-ML-001, PRE-ML-003。

---

## 3. `pre-solver-api`：SolverPlugin trait 与 Response 归一化

### 3.1 trait 定义

```rust
trait SolverPlugin: Send + Sync {
    fn id(&self) -> SolverId;
    fn version(&self) -> SolverVersion;

    fn init(&self, initial: &InitialState, bc: &BoundaryConditions,
             material: &MaterialSpec, params: &SolverParameters, seed: u64) -> Result<SolverHandle, SolverError>;

    fn step(&self, handle: &mut SolverHandle, excitation: &[ExcitationEvent],
             dt: f64, substeps: u32) -> Result<RawSolverState, SolverError>;

    fn to_standard_response(&self, history: &[RawSolverState],
             sample_times: &[f64]) -> Result<StandardPhysicalResponse, ResponseConversionError>;
}
```

### 3.2 生命周期（每条 Experience 的调用序列）

```
handle = plugin.init(initial_state, boundary_conditions, material, solver_params, seed)?
history = []
for t in 0..total_steps:
    raw = plugin.step(&mut handle, excitation_window(t), dt, substeps)?
    if raw.has_nan_or_inf():           // PRE-REL-001
        mark_experience_invalid(reason=NumericalDivergence, at_step=t)
        return Err(...)
    history.push(raw)
response = plugin.to_standard_response(&history, sample_times)?
```

数值发散检测在 `step()` 边界内完成（每步都检查，而非仅在末尾检查），确保尽早发现问题、避免继续在发散状态上浪费计算。

### 3.3 LandmarkId 分配规则（跨 solver 统一）

`LandmarkId` 不是 solver 内部粒子/顶点索引，而是按以下优先级分配的语义稳定标识：

1. 若几何资源携带命名锚点（如网格的 UV 坐标或预定义关键点）→ 使用锚点名。
2. 否则按初始几何的相对位置做规则化采样（如包围盒归一化坐标的网格采样），生成形如 `landmark(0.5,0.5,1.0)` 的坐标式 ID。
3. Rigid Body 的单一刚体 → 固定使用 `landmark(center_of_mass)` 与可选的 `landmark(corner_i)`（i=0..7，包围盒角点）。

规则的存在是为了让"同一物理场景类别"在不同 solver/不同网格密度下，Signature 提取阶段仍能以近似一致的方式聚合 landmark 统计量（服务于 H1/H2）。

对应需求：PRE-FR-002, PRE-FR-003, PRE-PHY-002, PRE-REL-001。

---

## 4. `pre-solver-rigid`：Rigid Body 详细设计

### 4.1 RawSolverState（Rigid 专有）

```rust
struct RigidRawState {
    bodies: Vec<RigidBodyState>,   // { position, orientation(quat), linear_vel, angular_vel }
    contacts: Vec<RigidContact>,   // { body_a, body_b, point, normal, impulse }
}
```

### 4.2 积分方法

参考实现（reference）：semi-implicit Euler，单线程，固定子步数（PRE-PHY-002 要求 reference 以正确性优先，不做并行/SIMD 优化）。

### 4.3 碰撞检测

MVP 范围：凸包/基本几何体（球/盒/胶囊）之间的解析碰撞检测 + 简化 SAT（Separating Axis Theorem）；不实现通用凸多面体/三角网格级碰撞（超出 MVP 范围，记为 Open Question，非阻塞——Rigid Body 在本项目中的角色是提供"刚体碰撞/摩擦/恢复"这一类宏观响应的样本来源，而非通用碰撞引擎）。

### 4.4 to_standard_response() 映射规则

- `position(t)/velocity(t)` ← 直接来自各刚体质心与角速度（映射到 `landmark(center_of_mass)` 与包围盒角点 landmark）。
- `deformation` ← `None`（刚体不形变）。
- `contact_events` ← 由 `RigidContact` 逐条转换，`restitution_estimate` 由碰撞前后法向速度比值估算。
- `constraint_events` ← 若存在关节/铰接约束（MVP 可选，非必须），记录违反量；否则为空数组。

对应需求：PRE-FR-002, PRE-PHY-001。

---

## 5. `pre-solver-xpbd`：XPBD 详细设计

### 5.1 RawSolverState（XPBD 专有）

```rust
struct XpbdRawState {
    particles: Vec<ParticleState>,      // { position, prev_position, velocity, inv_mass }
    constraints: Vec<ConstraintState>,  // { kind, particle_ids, rest_length_or_angle, lambda(乘子), compliance }
}
```

### 5.2 约束类型（MVP 范围）

- Distance constraint（stretch，用于 cloth/rope）
- Bending constraint（用于 cloth）
- Volume constraint（用于 soft body）
- Attachment constraint（固定点/绑定）

### 5.3 求解循环（reference 实现）

```
for substep in 0..substeps:
    predict_positions(particles, dt/substeps, external_forces)
    for iteration in 0..solver_iterations:
        for constraint in constraints:
            solve_constraint_xpbd(constraint, particles, dt/substeps)   // 含 compliance 项
    update_velocities(particles, dt/substeps)
```

`solver_iterations` 与 `compliance` 均为 `SolverParameters`/`MaterialParameters` 的一部分，需完整记录以支持重放（PRE-PHY-004）。

### 5.4 to_standard_response() 映射规则

- `position/velocity/acceleration` ← 按 §3.3 规则采样的 landmark 粒子。
- `deformation.stretch/compression/shear` ← 由约束的当前长度/角度相对静止长度/角度的偏差聚合得到（均值、峰值、RMS）。
- `deformation.bending` ← 仅当存在 bending constraint 时计算。
- `constraint_events` ← 每个约束的违反量（`|current - rest| `）超过阈值时记录一条事件。

对应需求：PRE-FR-002, PRE-PHY-001。

---

## 6. `pre-solver-mpm`：MPM 详细设计

### 6.1 RawSolverState（MPM 专有）

```rust
struct MpmRawState {
    particles: Vec<MpmParticle>,   // { position, velocity, deformation_gradient(3x3), mass, volume }
    grid: MpmGrid,                 // 背景网格状态（速度场、质量场），生命周期仅限单步内
}
```

### 6.2 材料模型（MVP 范围）

- Elastic：Neo-Hookean 或 Corotated（二选一，实现阶段确定，需在 SolverParameters 中记录具体模型 ID）
- Plastic：von Mises 屈服（MVP 可选，若时间不足可推迟，不阻塞 H1/H2 验证——弹性材料已足以验证核心假设）
- Granular：Drucker-Prager（MVP 可选，同上）

MVP 最低要求（不可再减）：Elastic 必须实现且通过验收；Plastic/Granular 若因周期原因未实现，需在验收报告中明确声明范围缩减，而非静默省略。

### 6.3 求解循环（reference 实现，APIC 转移）

```
for step:
    particles_to_grid(particles, grid)         // P2G：质量/动量转移
    apply_grid_forces(grid, dt, gravity, boundary)
    grid_to_particles(particles, grid, dt)      // G2P：更新粒子速度/位置
    update_deformation_gradient(particles, dt)
    apply_plasticity_projection(particles)      // 仅 plastic/granular 材料
```

### 6.4 to_standard_response() 映射规则

- `position/velocity` ← 按 §3.3 规则从粒子中采样 landmark（MPM 无固定拓扑，landmark 用初始位置的包围盒归一化坐标最近邻粒子近似）。
- `deformation` ← 由 `deformation_gradient` 计算的应变张量聚合（strain_rms 等）。
- MPM 无显式约束，`constraint_events` 恒为空数组（区别于 XPBD）。

对应需求：PRE-FR-002, PRE-PHY-001。

---

## 7. `pre-signature`：特征提取算法详细设计

### 7.1 提取函数签名

```rust
fn extract_signature(response: &StandardPhysicalResponse, material: &MaterialSpec,
                      boundary: &BoundaryConditions, excitation: &Excitation) -> PhysicalSignature
```

纯函数：仅依赖输入参数，无隐藏状态、无 IO（PRE-NFR-003 可测试性要求）。

### 7.2 各特征域算法要点

- **Kinematics**：对 `position/velocity/acceleration` 逐 landmark 计算均值/峰值/RMS，再跨 landmark 聚合（均值），避免单一异常 landmark 主导特征。
- **Deformation**：直接取 `response.deformation` 各字段的时间序列统计量；若 `deformation` 为 `None`（Rigid），则该子结构全部字段填 0 并在 `PhysicalSignature` 中不设置单独的"缺失标记"字段（按 PRE-PHY-001 的显式 N/A 要求，改为在 `schema_version` 关联的字段说明文档中注明"deformation=0 语义等价于 not-applicable"，避免为每个字段加 `Option` 造成后续编码复杂度剧增）。
- **Temporal**：对能量/动量序列或某一代表 landmark 的位移序列做 FFT，取主频为 `dominant_freq`；阻尼率通过包络线指数拟合估算；恢复时间通过阈值穿越法估算（响应回落到稳态附近某百分比所需时间）。
- **Contact**：由 `contact_events` 直接聚合计数与统计量。
- **Material**：直接复制 `material.params` 中的数值字段（不做变换）。
- **Constraint**：由 `constraint_events` 聚合类型直方图与平均违反量。
- **Field**：由 `excitation.events` 聚合场类型集合与幅值统计。

### 7.3 数值稳定性注意事项

FFT 主频估计在采样点数过少（短时长仿真）时不稳定，需要设置最小采样点数下限；低于下限时 `dominant_freq` 标记为 `None` 而非返回噪声值。这是本文书新增的实现约束，02 号文档未展开到此细节。

对应需求：PRE-FR-004, PRE-PHY-001, PRE-PHY-005, PRE-ML-001。

---

## 8. `pre-encoder`：V1 Encoder 详细设计

### 8.1 编码函数签名

```rust
fn encode_v1(signature: &PhysicalSignature, atlas_stats: &FeatureNormalizationStats) -> EmbeddingSet
```

`atlas_stats` 是从当前 Atlas 中全部（或采样）Signature 计算得到的每维特征的均值/标准差，用于 z-score 归一化；该统计量本身需要版本化（随 `encoder_version` 一起演进）。

### 8.2 子向量拼接规则

- `behavior_vector` = normalize(concat(kinematics.*, contact.*))
- `deformation_vector` = normalize(concat(deformation.*, material.* 中与形变相关字段))
- `temporal_vector` = normalize(concat(temporal.*))
- `global_vector` = PCA_project(concat(behavior_vector, deformation_vector, temporal_vector, geometry.*, constraint.*, field.*), target_dim)

PCA 投影矩阵作为 `encoder_version` 的一部分持久化（训练/拟合时机：Atlas 达到一定规模后离线计算一次，非在线更新——避免检索期间索引失效）。

### 8.3 版本管理

```rust
struct EncoderVersion(u32);

struct FeatureNormalizationStats {
    encoder_version: EncoderVersion,
    per_feature_mean: HashMap<FeatureKey, f64>,
    per_feature_std: HashMap<FeatureKey, f64>,
    pca_projection: Matrix,
    fitted_at: Timestamp,
    fitted_on_experience_count: usize,
}
```

新 `encoder_version` 产生新的 `FeatureNormalizationStats`，历史 Embedding 不重算，检索时按版本分索引（PRE-ML-003, PRE-REPRO-002）。

对应需求：PRE-FR-005, PRE-ML-001, PRE-ML-002, PRE-ML-003。

---

## 9. `pre-atlas`：存储 schema 详细设计

### 9.1 Relational Store（SQLite）表结构

```sql
CREATE TABLE experiences (
    id TEXT PRIMARY KEY,             -- UUID v7
    created_at INTEGER NOT NULL,
    provenance_source TEXT NOT NULL,
    provenance_generator TEXT NOT NULL,
    parent_experience_id TEXT,
    solver_id TEXT NOT NULL,
    solver_version TEXT NOT NULL,
    material_model TEXT NOT NULL,
    validation_status TEXT NOT NULL,   -- Candidate | Validated
    response_blob_path TEXT NOT NULL,
    signature_schema_version INTEGER NOT NULL,
    signature_json TEXT NOT NULL,      -- PhysicalSignature 序列化（结构化字段小，直接内联）
    encoder_version INTEGER,           -- 可为空：写入时若尚未编码
    determinism_json TEXT NOT NULL
);

CREATE INDEX idx_experiences_solver ON experiences(solver_id, material_model);
CREATE INDEX idx_experiences_validation ON experiences(validation_status);
CREATE INDEX idx_experiences_created ON experiences(created_at);

CREATE TABLE validation_metrics (
    experience_id TEXT PRIMARY KEY REFERENCES experiences(id),
    position_error REAL, velocity_error REAL, deformation_error REAL,
    frequency_error REAL, damping_error REAL, contact_timing_error REAL,
    verification_score REAL,
    identifiability TEXT   -- low | normal（对应 02号文档 §13 参数不可辨识暴露机制）
);
```

Metadata 查询（`experiences` 表的过滤/索引）不触碰 `response_blob_path` 指向的大体积文件，满足 PRE-DATA-002。

### 9.2 Vector Store（HNSW，本地文件持久化）

每个 `encoder_version` 对应一个独立的 HNSW 索引文件：`atlas/vector_index/{encoder_version}/global.hnsw`。索引条目：`experience_id -> global_vector`。子向量（behavior/deformation/temporal）不建独立 ANN 索引（MVP 简化，02号文档 §10），而是作为 `experiences.signature_json` 的一部分随 metadata 一起读取，供精排阶段现算相似度。

### 9.3 Blob Store（本地文件系统）

```
atlas/blobs/{experience_id[0:2]}/{experience_id}/response.bin
```

按 ID 前两位分片目录，避免单目录文件数过多。`response.bin` 为 `StandardPhysicalResponse` 的序列化（格式：bincode 或等价二进制格式，选型细节留实现阶段，非架构决策）。

对应需求：PRE-DATA-001, PRE-DATA-002, PRE-DATA-003。

---

## 10. `pre-retrieval`：检索与融合算法详细设计

### 10.1 检索函数签名

```rust
fn search(query_signature: &PhysicalSignature, encoder_version: EncoderVersion,
          top_n: usize, filter: MetadataFilter) -> Vec<CandidateExplanation>
```

### 10.2 两阶段流程

```
query_embedding = encode_v1(query_signature, atlas_stats[encoder_version])
coarse = hnsw_index[encoder_version].search(query_embedding.global_vector, top_n)   // ANN 粗召回
filtered = coarse.filter(|c| metadata_matches(c, filter))                            // post-filter（ADR-007）
if filtered.len() < recall_shortage_threshold:
    log_recall_shortage(filter, coarse.len(), filtered.len())                        // 对应 ISS-007 修正
for c in filtered:
    c.behavior_sim = cosine(query_embedding.behavior_vector, c.embedding.behavior_vector)
    c.deformation_sim = cosine(query_embedding.deformation_vector, c.embedding.deformation_vector)
    c.temporal_sim = cosine(query_embedding.temporal_vector, c.embedding.temporal_vector)
    c.retrieval_score = fuse(c.behavior_sim, c.deformation_sim, c.temporal_sim, weights)
return filtered.sorted_by(retrieval_score).take(top_k)
```

### 10.3 融合公式（默认权重，可配置，PRE-NFR-004）

```
retrieval_score = w_b * behavior_sim + w_d * deformation_sim + w_t * temporal_sim
默认: w_b = 0.4, w_d = 0.35, w_t = 0.25   （初始专家设定值，非学习得到，对应 OQ-03）
```

### 10.4 Novel Dynamics 判定

```
if max(retrieval_score for c in filtered) < novel_similarity_threshold:
    return NovelDynamics { reason: LowSimilarity }
```

（另一半判定——仿真误差高——在 `pre-verify` 阶段进行，见 §11.3）

对应需求：PRE-FR-006, PRE-FR-007, PRE-FR-011, PRE-VEC-001~004, PRE-OBS-002。

---

## 11. `pre-verify`：验证流水线详细设计

### 11.1 窗口切分（对应 02号文档 §13 修正、ISS-009）

```rust
fn split_observation(observed: &StandardPhysicalResponse, match_ratio: f64) -> (MatchWindow, HeldOutWindow)
// match_ratio 默认 0.65，可配置
```

`MatchWindow` 用于 §7 特征提取与 §10 检索；`HeldOutWindow` 仅在 §11.2 的比较步骤中使用。

### 11.2 验证函数签名

```rust
fn verify(candidate: &PhysicsExperience, observed_match: &MatchWindow,
          observed_heldout: &HeldOutWindow) -> VerificationMetrics
```

流程：
```
predicted_full = resimulate(candidate.solver, candidate.material, candidate.initial_state, ... , total_duration)
predicted_heldout = predicted_full.slice(heldout_time_range)
metrics.position_error = rmse(predicted_heldout.position, observed_heldout.position)
metrics.velocity_error = rmse(predicted_heldout.velocity, observed_heldout.velocity)
metrics.deformation_error = rmse(predicted_heldout.deformation, observed_heldout.deformation)   // 若适用
metrics.frequency_error = |predicted.dominant_freq - observed.dominant_freq| / observed.dominant_freq
metrics.damping_error = |predicted.damping_rate - observed.damping_rate|
metrics.contact_timing_error = dtw_distance(predicted.contact_events, observed.contact_events)  // 时序对齐距离
metrics.verification_score = weighted_sum(metrics.*, verification_weights)
```

### 11.3 Novel Dynamics 判定（误差侧）

```
if best_candidate.verification_score worse than novel_error_threshold:
    return NovelDynamics { reason: HighSimulationError }
```

### 11.4 参数不可辨识检测（对应 ISS-006 修正）

```rust
fn detect_identifiability(top_m: &[Candidate]) -> Identifiability {
    let param_spread = compute_normalized_spread(top_m.iter().map(|c| &c.material.params));
    if param_spread > identifiability_threshold && error_spread(top_m) < error_spread_threshold {
        Identifiability::Low   // 多组参数、误差相近 → 不可辨识
    } else {
        Identifiability::Normal
    }
}
```

若判定为 `Low`，`CandidateExplanation` 输出全部 `top_m` 而非单一 best（02号文档 §13 已确立此原则，本节给出判定算法细节）。

对应需求：PRE-FR-008, PRE-FR-009, PRE-FR-011, PRE-VEC-002, PRE-OBS-001。

---

## 12. `pre-refine`：参数优化算法详细设计

### 12.1 局部搜索（MVP 默认实现）

```rust
fn local_search(initial: &MaterialParameters, objective: impl Fn(&MaterialParameters) -> f64,
                 budget: Budget) -> MaterialParameters {
    let mut current = initial.clone();
    let mut current_score = objective(&current);
    let mut step = initial_step_size;
    while !budget.exhausted() {
        let mut improved = false;
        for dim in current.dims() {
            for direction in [-1.0, 1.0] {
                let candidate = current.perturb(dim, direction * step);
                let score = objective(&candidate);
                if score < current_score {
                    current = candidate; current_score = score; improved = true;
                }
            }
        }
        if !improved { step *= 0.5; }   // coordinate descent + 步长衰减（Nelder-Mead 简化变体）
        if step < min_step { break; }
    }
    current
}
```

`objective` 即调用 `pre-verify::verify()` 后取 `verification_score`（一次调用 = 一次仿真，`Budget` 以仿真次数计，供 H3 实验统计"仿真次数"指标）。

### 12.2 trait 与扩展点

```rust
trait ParamOptimizer: Send + Sync {
    fn optimize(&self, initial: &MaterialParameters,
                objective: &dyn Fn(&MaterialParameters) -> f64, budget: Budget) -> MaterialParameters;
}
```

`LocalSearchOptimizer` 是 MVP 唯一实现；CMA-ES 作为后续插件，接入方式相同，不需改动调用方（PRE-NFR-001 的插件化在 optimizer 侧的具体落实）。

对应需求：PRE-FR-010, PRE-NFR-001, PRE-NFR-004。

---

## 13. `pre-gen`：数据集生成器详细设计

### 13.1 采样策略（MVP：Latin Hypercube）

```rust
fn sample_parameter_space(space: &ParameterSpace, n_samples: usize, seed: u64) -> Vec<MaterialParameters>
```

使用 Latin Hypercube Sampling（LHS）而非纯随机或网格枚举（PRE-FR-014, R-05 缓解措施）。

### 13.2 失败点记录（对应 ISS-008 修正）

```rust
struct GenerationFailureLog {
    parameters: MaterialParameters,
    failure_reason: FailureReason,   // NumericalDivergence | Timeout | ...
}
```

每次批量生成任务结束后，`pre-gen` 输出 `GenerationFailureLog` 的完整列表（而非仅计数或丢弃），供离线分析是否存在系统性偏差区域（如某一参数区间总是发散）。该分析不在 MVP pipeline 内自动执行，作为 06 号文档实验的输入数据。

### 13.3 并行执行

进程内线程池，每个 `(parameters, seed)` 组合是独立仿真任务，无共享可变状态，天然可并行（02号文档 §20）。

对应需求：PRE-FR-014, PRE-PERF-002, PRE-REL-001。

---

## 14. `pre-cli`：命令行详细设计

MVP 子命令（对应 02号文档 §19 职责表，此处给出命令行形态）：

```
pre-cli experience create --definition <path>
pre-cli experience simulate --id <experience_id>
pre-cli experience encode --id <experience_id> --encoder-version <v>
pre-cli search --query <observed_response_path> --top-n <n> --filter <metadata_expr>
pre-cli verify --candidate <experience_id> --observed <observed_response_path>
pre-cli refine --candidate <experience_id> --budget <n_simulations>
pre-cli gen --experiment-def <path> --n-samples <n> --strategy lhs
```

不锁定为最终 CLI schema（PRE-API-002），本节仅为详细设计阶段的具体化，供实现参考。

对应需求：PRE-API-001, PRE-API-002。

---

## 15. エラーコード一覧（错误码一览）

| 错误码 | 触发条件 | 处理方 | 对应需求 |
|---|---|---|---|
| `SOLVER_NUMERICAL_DIVERGENCE` | `step()` 检测到 NaN/Inf | pre-solver-*，标记 Experience Invalid，不写入 Atlas | PRE-REL-001 |
| `SOLVER_INIT_INVALID_PARAMS` | 初始化参数超出合法范围 | pre-solver-*，拒绝创建 SolverHandle | PRE-SEC-001 |
| `RESPONSE_CONVERSION_FAILED` | `to_standard_response()` 转换失败（如空历史） | pre-solver-*/pre-core | PRE-FR-003 |
| `SIGNATURE_INSUFFICIENT_SAMPLES` | 采样点数不足以估计频域特征 | pre-signature，字段置 None 而非报错中止 | 见 §7.3 |
| `ENCODER_VERSION_MISMATCH` | 检索时 query 与索引 encoder_version 不一致 | pre-retrieval，拒绝跨版本比较 | PRE-REPRO-002, PRE-ML-003 |
| `VERIFICATION_RESIM_FAILED` | 重仿真过程中 solver 报错 | pre-verify，记录明确原因，不吞异常 | PRE-REL-002 |
| `REFINEMENT_BUDGET_EXHAUSTED` | 优化预算用尽仍未收敛 | pre-refine，返回当前最优解 + 未收敛标记 | PRE-FR-010 |
| `DATASET_PARAM_OUT_OF_RANGE` | Dataset Generator 采样超出参数上限 | pre-gen，跳过该样本并记入 GenerationFailureLog | PRE-SEC-001 |
| `ATLAS_BLOB_NOT_FOUND` | metadata 存在但 blob 文件缺失（数据不一致） | pre-atlas，返回结构化错误，不静默返回空响应 | PRE-REL-002 |

---

## 16. 处理流程时序（主要 3 条路径）

### 16.1 生成路径（Dataset Generator → Atlas）

```
1. pre-gen 采样 MaterialParameters（LHS）
2. 对每个样本：pre-solver-*.init/step* → RawSolverState history
3. pre-solver-*.to_standard_response() → StandardPhysicalResponse
4. pre-signature.extract_signature() → PhysicalSignature
5. pre-encoder.encode_v1() → EmbeddingSet
6. pre-atlas.store() → 写入 relational + vector + blob 三处存储
7. 若任一步骤出错 → 记录 GenerationFailureLog，不写入 Atlas（部分写入需回滚）
```

### 16.2 检索验证路径（Observation → Best Explanation）

```
1. 输入 ObservedPhysicalResponse（V0.1: SimulationBackend 提供的留出仿真）
2. split_observation() → MatchWindow + HeldOutWindow（§11.1）
3. pre-signature.extract_signature(MatchWindow) → query signature
4. pre-encoder.encode_v1() → query embedding
5. pre-retrieval.search() → Top-N → post-filter → Top-K（含 Novel Dynamics 相似度侧判定）
6. pre-verify.verify() 逐个 Top-K 候选（用 HeldOutWindow 打分，含 Novel Dynamics 误差侧判定）
7. pre-verify.detect_identifiability(Top-M) → 判定是否需要展示多解
8. pre-refine.optimize() 对 Top-M 候选做参数精调（可选，取决于预算）
9. 输出 CandidateExplanation 列表（含全部评分明细字段）
```

### 16.3 学习闭环路径（验证通过 → 写回 Atlas）

```
1. 步骤 16.2 产出的 Best/Top-M 候选，其 verification_score 达到 Validated 阈值
2. 构造新 PhysicsExperience（provenance.parent_experience_id 指向原候选，若经过 Refinement 则参数已更新）
3. validation.status = Validated
4. pre-atlas.store() → 写入并重新加入 HNSW 索引（增量插入，非全量重建）
5. 后续检索可命中该新记录（AC-05 闭环验证点）
```

---

## 17. 与 02/04 号文档的追加映射

本文书新增的实现细节均从属于 04_PRE_Traceability_Matrix.md 中已存在的 Requirement ID，不引入新 ID；本节仅列出「详细设计新增关键决策」，供 04 号文档后续版本按需扩展"Design Section"列指向本文书：

| 主题 | 本文书章节 | 关联的 02号文档章节 | 关联需求 |
|---|---|---|---|
| LandmarkId 分配规则 | §3.3 | 02号 §6, §8 | PRE-PHY-002, PRE-PHY-005 |
| 参数不可辨识判定算法 | §11.4 | 02号 §13（v0.1.1 新增段落） | PRE-OBS-001 |
| 检索召回不足日志 | §10.2 | 02号 §26（v0.1.1 新增段落） | PRE-OBS-002 |
| Dataset 失败点记录 | §13.2 | 02号 §20 | PRE-REL-001, PRE-FR-014 |
| Encoder 版本与归一化统计量绑定 | §8.3 | 02号 §23 | PRE-ML-003, PRE-REPRO-002 |
| SQLite/HNSW/Blob 具体 schema | §9 | 02号 §17, §18 | PRE-DATA-001~003 |
| pre-bevy 回放/异步桥接/版本策略 | §18 | 02号 §33（v0.1.2 新增） | PRE-BEVY-001~006 |

> 说明：本次修订时 04 号文档的 47 条既有需求未重写"Design Section"列（避免大范围改动已合并内容），仅追加了 PRE-BEVY-001~006 六行新记录（见 04号文档 v0.1.2），本表作为既有 47 条需求与本文书之间的补充索引；上表末行的 pre-bevy 相关条目已直接体现在 04 号文档新增行中，不存在缺口。

---

## 18. `pre-bevy`：Bevy 引擎适配层详细设计

本节是 02号文档 §33 的下一层，给出 `pre-bevy` 的具体类型定义、系统调度顺序与数值算法。前提约束（不重复展开，见 02号文档 §33.1 与 ADR-009）：`pre-bevy` 单向依赖 `pre-core`，除 `pre-bevy` 外的全部 workspace 成员对 `bevy` 零依赖（范围定义见 PRE-BEVY-001）。

### 18.1 Cargo 依赖与 feature 设计

```toml
# pre-bevy/Cargo.toml（示意）
[dependencies]
pre-core = { path = "../pre-core" }
pre-retrieval = { path = "../pre-retrieval", optional = true }   # 仅 PRE-BEVY-004 需要
pre-verify = { path = "../pre-verify", optional = true }
bevy = { version = "0.X", default-features = false, features = ["bevy_render", "bevy_transform"] }

[features]
default = ["playback"]
playback = []                          # PRE-BEVY-002/003
query_bridge = ["pre-retrieval", "pre-verify"]   # PRE-BEVY-004，可选启用
```

`playback` 与 `query_bridge` 拆为独立 feature：仅需要回放能力的下游用户不必编译进 `pre-retrieval`/`pre-verify`，减少不必要的编译依赖面（呼应 ADR-009 的"最小依赖侵入"精神，在 `pre-bevy` 内部也贯彻同一原则）。

### 18.2 组件/资源类型完整定义

```rust
#[derive(Component, Clone, Copy)]
struct PreLandmark {
    landmark_id: LandmarkId,
    experience_id: ExperienceId,
}

#[derive(Resource)]
struct PrePlaybackState {
    response: StandardPhysicalResponse,
    playback_time: f64,
    speed: f64,
    looping: bool,
}

#[derive(Event)]
struct PrePlaybackFinished { experience_id: ExperienceId }   // 非循环模式播放结束时触发，供 Bevy 应用响应
```

### 18.3 插值算法细节

```rust
fn interpolate_position(response: &StandardPhysicalResponse, landmark: LandmarkId, t: f64) -> Vec3 {
    let idx = response.landmarks.iter().position(|&l| l == landmark)
        .expect("landmark not present in this response");   // 契约：landmark 必须来自同一 response，调用方保证
    let times = &response.sample_times;
    match binary_search_bracket(times, t) {
        Bracket::Before => response.position[idx][0],                          // t 早于首个采样点：钳制到首帧
        Bracket::After  => *response.position[idx].last().unwrap(),            // t 晚于末个采样点：钳制到末帧
        Bracket::Between(i, j) => {
            let alpha = (t - times[i]) / (times[j] - times[i]);
            response.position[idx][i].lerp(response.position[idx][j], alpha as f32)
        }
        Bracket::SinglePoint => response.position[idx][0],                     // 仅 1 个采样点：退化为常量（PRE-BEVY-003 边界情形）
    }
}
```

`binary_search_bracket` 对 `sample_times`（假定非降序，若非等间隔仍适用）做二分查找，返回 `t` 所在区间；三种边界情形（早于/晚于/单点）均不 panic，符合 §15 错误处理原则（对用户输入之外的内部不变量用 `expect` 是可接受的，因为 `landmark` 集合由 `pre-bevy` 自身在生成 `PreLandmark` 组件时保证与 `response.landmarks` 一致，属契约违反而非可恢复错误）。

### 18.4 System 调度顺序

```rust
impl Plugin for PrePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PreQueryRequests>()
           .init_resource::<PreQueryResults>()
           .add_event::<PrePlaybackFinished>()
           .add_systems(Update, (
               pre_playback_system,              // §18.3 插值 + Transform 更新
               pre_playback_finished_detector,   // 检测非循环播放到达末帧，发出事件
               pre_query_dispatch_system,        // 消费 PreQueryRequests，派发后台任务（feature = query_bridge）
               pre_query_poll_system,            // 轮询后台任务完成情况，写入 PreQueryResults
           ).chain());   // 顺序执行：先更新回放状态，再处理事件，再处理查询——避免同帧内状态竞争
    }
}
```

四个系统在同一 `Update` stage 内以 `.chain()` 强制顺序执行，避免 `pre_playback_finished_detector` 读到本帧尚未更新的 `playback_time`（保证系统间数据依赖的确定性，属于 Bevy ECS 调度层面的实现细节，不属于架构决策，故不产生新 ADR）。

### 18.5 异步查询后台任务实现要点

```rust
fn pre_query_dispatch_system(mut requests: ResMut<PreQueryRequests>, task_pool: Res<AsyncComputeTaskPool>,
                              mut pending: Local<Vec<(QueryId, Task<CandidateExplanation>)>>) {
    for req in requests.0.drain(..) {
        let task = task_pool.spawn(async move {
            let candidates = pre_retrieval::search(&req.query_signature, req.encoder_version, req.top_n, req.filter);
            pre_verify::verify_best(&candidates, &req.observed_heldout)
        });
        pending.push((req.id, task));
    }
}

fn pre_query_poll_system(mut pending: Local<Vec<(QueryId, Task<CandidateExplanation>)>>,
                          mut results: ResMut<PreQueryResults>) {
    pending.retain_mut(|(id, task)| {
        if let Some(result) = future::block_on(future::poll_once(task)) {
            results.0.insert(*id, result);
            false   // 完成，移出待处理列表
        } else {
            true    // 未完成，保留
        }
    });
}
```

使用 Bevy 自带的 `AsyncComputeTaskPool`（而非独立 `std::thread`），复用 Bevy 的任务调度基础设施，避免 `pre-bevy` 自行管理线程池——这是本节相对 02号文档 §33.3 的进一步细化：02 号文档只说明"后台任务 + 轮询"模式，未指定具体机制；本节明确选用 Bevy 原生任务池，理由是与宿主应用共享同一调度资源，避免线程数失控。

### 18.6 单元测试要点（对应 09号文档 TC-BEVY）

- `interpolate_position` 的四种边界情形（早于/晚于/区间内/单点）需要逐一测试，覆盖 §18.3 的分支逻辑。
- `PrePlugin` 的 System 顺序需要一个集成测试验证：构造一个已知 `StandardPhysicalResponse`，推进固定数量的虚拟帧（`app.update()` 循环），断言 `Transform` 序列与手算插值结果一致。
- `pre_query_dispatch_system`/`pre_query_poll_system` 需要测试"派发后不阻塞当前帧"（如断言 `app.update()` 单次调用的墙钟耗时不随查询耗时增长）。
