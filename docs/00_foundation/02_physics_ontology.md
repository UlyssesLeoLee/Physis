# GVPE — Physics Ontology（要件定義書）

> 输入基线：`01_requirements.md` 中 `GVPE-ONT-*` 与 `GVPE-GPH-001/002`。本文档**只**描述物理知识图谱（Physics Knowledge Graph, PKG）的 schema（`03_graph_schema.md` §1.A）——**不**描述运行时约束图（Runtime Constraint Graph）与执行图（Execution Graph），那两者是 `03_graph_schema.md` §1.B / §1.C 各自完全独立的章节。将本文档与 Runtime 数据混为一谈，正是 `04_architecture.md` §4.3 与本文档自身的 Ontology Review（§Review）所要捕捉的失败模式。

## 0. 文档元数据

| 字段 | 取值 |
|---|---|
| 文档编号 | GVPE-DOC-02 |
| 文档类型 | 要件定義書 |
| 版本 | v0.2（标准化重写版） |
| 状态 | Draft |
| 适用阶段 | MVP / Phase 1+ |
| 关联系统 | GVPE / 物理本体（Physics Knowledge Graph schema） |
| 上游文档（输入基线） | GVPE-DOC-00, GVPE-DOC-01 |
| 下游文档（被消费于） | GVPE-DOC-03, GVPE-DOC-04, GVPE-DOC-05, GVPE-DOC-11, GVPE-DOC-15, GVPE-DOC-21 |
| 编写者 | — |
| 审批者 | — |

## 1. 修订历史

| 版本 | 日期 | 修改者 | 修改内容 |
|---|---|---|---|
| v0.1 | — | — | 初稿（原始版本） |
| v0.2 | 2026-08-19 | 标准化 pass | 转为 IPA 要件定義書 格式；叙事中文化；保留全部技术内容；补充文档元数据、修订历史、术语定义、关联文档、审批签字等标准章节；将原 §1–§26 的章节映射为 IPA 模板下的 §12 正文，并在每节保留英文原标题以便交叉引用 |

## 2. 文档目的

本文档定义 GVPE 物理知识图谱（Physics Knowledge Graph, PKG）的**完整顶层本体**：节点类型、属性分类、关系词表、空间与时间关系、以及由它们共同支撑的物理因果链。其设计目标是：当未来扩展 `Energy` / `Wave` / `Field` / `Process` / `Law` 等领域时，**永不**需要破坏性 schema 迁移。同时，本文在多处显式列出 11 类概念混淆（11 confusion categories）作为 Ontology Review 的检查表。

## 3. 适用范围

- **适用阶段**：MVP 阶段 schema 必须完整；实例填充按 §26 的 MVP ontology instance scope 限定子集。
- **适用读者**：图谱 schema 设计者、`gvpe-graph` crate 实现者、本体评审者、`gvpe-compiler` 桥接实现者、3DGS 未来方向设计者。
- **不适用**：本文档不规定图数据库的物理存储选择（见 `GVPE-DOC-16`）；不规定 Cypher / GQL 等具体查询语言；不重复 `03_graph_schema.md` 中关于存储 / 索引 / 一致性的细节。
- **强制边界**：schema 中的所有"本体规则（Ontology rule）"对 `gvpe-graph` 写入路径具有 binding 效力；违规写入应被 schema validator 或写入守卫拒绝。

## 4. 术语定义

| 术语 | 释义 |
|---|---|
| 本体（Ontology） | 物理知识图谱的 schema 层定义，含节点类型、属性类型、关系类型。 |
| 物理知识图谱（Physics Knowledge Graph, PKG） | 承载本体的图谱实例；其 schema 即本文档所述。 |
| 运行时约束图（Runtime Constraint Graph） | 与 PKG **完全独立**的另一类图（`03_graph_schema.md` §1.B），承载数值化求解条目。 |
| 执行图（Execution Graph） | 与 PKG **完全独立**的另一类图（`03_graph_schema.md` §1.C），承载执行流。 |
| 物理签名（Physics Signature） | 多向量表征，由本文档 §20 给出 schema，与 `GVPE-DOC-11` 互引。 |
| 物理剖面（`PhysicsProfile`） | Graph → Runtime 的**唯一**交付结构，扁平、POD 友好，详见 §21 与 `GVPE-FR-003`。 |
| 物理因果链 | `Cause → Process → StateChange → EnergyTransfer → ObservableEffect`，详见 §25。 |
| `ONT-ISS` | Ontology Issue 编号，本体评审中登记的缺陷前缀（如 `ONT-ISS-001`）。 |
| 条件关系（Conditional relation） | 受上下文限定（如温度、压力）的关系，避免将其表达为无条件边。 |
| 属性（Property） | "实体**特征性拥有**"的量（耐久、不随时变）。 |
| 状态（State） | "实体**当前所是**"的快照（time-indexed）。 |
| 行为（MechanicalBehavior） | "实体**机械行为上如何表现**"，独立于 Matter / Phase。 |

## 5. 项目背景与约束

- **方向性**：本体服务于知识 / 控制平面（Graph Space），不是仿真数据平面。任何将 per-frame 数值状态 bulk 写入图谱的做法，违反 `GVPE-PROHIBIT-03` 的本体论基础（详见 §6 的 State / Property 区分）。
- **可演化性**：Energy / Wave / Field / Process / Law 等节点类型在 MVP 即作为 schema 存在但**实例不填充**；这是为避免后续破坏性 schema 迁移所支付的成本。
- **节点 / 属性判定规则**（`03_graph_schema.md` §2）：高语义、高连通性、来源 / 置信度可追溯的数据**才**升格为图节点；bulked 的单次仿真数值直接嵌入 `PhysicsProfile`，**不**单独 node-ify。
- **写入守卫**：`gvpe-graph` 写入路径必须实现 schema validator 与"禁止 bulk per-frame State 写入"守卫；`ONT-ISS-001` 即针对该守卫的落实状态（见 §Review）。
- **三类图严格分离**：PKG（本文档）、Runtime Constraint Graph、Execution Graph 不共享存储、不共享查询表面。

## 6. 功能需求 (GVPE-ONT-XXX / GVPE-FR-XXX 涉及本体的部分)

| ID | 描述 |
|---|---|
| GVPE-ONT-001 | 顶层本体（§1 ~ §25 全部节点与关系类型）必须在 schema 中**完整**存在；MVP 实例填充按 §26 子集限定。 |
| GVPE-ONT-002 | 11 类概念混淆（§Review 检查表）必须在 schema 层与写入守卫层同时给出**可机器检查**的判定规则。 |
| GVPE-ONT-003 | 关系词表（§22）必须支持**条件关系**（conditional），并能用于构建 §25 物理因果链。 |
| GVPE-ONT-004 | `PhysicsProfile`（§21）必须保持扁平、POD 友好、且**不得**包含图节点引用或向量句柄（与 `GVPE-FR-003` 等价）。 |
| GVPE-ONT-005 | PKG schema 不得在后续扩展 Energy / Wave / Field / Process / Law / BoundaryCondition / Experiment / Hypothesis 时**要求破坏性迁移**（可由 §26 的 schema-present-but-unpopulated 子集 + schema validator 零实例通过验证来证明）。 |
| GVPE-ONT-006 | `Constraint` 节点仅描述类型与语义；运行时约束行**只**存在 Runtime Constraint Graph（`03_graph_schema.md` §1.B）；任何将图 `Constraint` 节点直接当可求解条目的代码路径均属缺陷。 |
| GVPE-ONT-007 | `PhysicalLaw` 节点的存在**不**蕴含 Runtime 实现了该定律；`04_architecture.md` §4.5 跟踪"已注册定律 vs 已实现定律"的对应关系。 |

## 7. 非功能需求 (NFR / PERF)

| ID | 类别 | 描述 |
|---|---|---|
| GVPE-NFR-003 | NFR | 本体 schema 与运行时核心之间**不得**产生反向依赖；`gvpe-core` 等核心 crate 不可依赖 `gvpe-graph`（与 `GVPE-DOC-01` 一致；`cargo tree` 可机械验证）。 |
| GVPE-PERF-ONT-001 | PERF | 本体 schema 验证与关系词表检查的代价必须**不**进入仿真热路径；验证在写入期完成，运行时仅消费已编译后的 `PhysicsProfile`。 |
| GVPE-NFR-DOC-002 | NFR | 任何对本体的破坏性 schema 变更**必须**在 `GVPE-DOC-15` 的测试策略中显式记录并拒绝（"零实例通过 schema validator"是允许保留 schema 占位实例的硬条件）。 |

## 8. 业务约束

- **GCN-ONT-01**：本体的 11 类混淆检查（§Review 表中的 1–11）必须保留为"Ontology Review"的必过项；任何 High 严重度未关闭项均阻塞本基线被接受（与 `AC-04` 等价）。
- **GCN-ONT-02**：本体 schema 中所有"Ontology rule"措辞代表 binding 规则；对应 `gvpe-graph` 写入守卫或 schema validator。
- **GCN-ONT-03**：本体与 `gvpe-compiler` / `gvpe-runtime` 之间通过 `PhysicsProfile` 单向流动，**不得**反向。

## 9. 验收标准

- **AC-ONT-01**：所有 §1 ~ §25 的节点类型与关系类型在 schema 中**可枚举**且**可实例化**（即使部分实例数 = 0，schema validator 仍能 0 错误通过）。
- **AC-ONT-02**：§Review 列出的 11 类混淆检查中，无 High 严重度 `ONT-ISS-*` 残留（与 `AC-04` 等价；当前 `ONT-ISS-001` 严重度为 Medium）。
- **AC-ONT-03**：§25 物理因果链工作示例（`ExternalForce --CAUSES--> Acceleration --CAUSES--> VelocityChange --CAUSES--> Collision --GENERATES--> Deformation --STORES--> ElasticEnergy --RELEASES--> KineticEnergy --GENERATES--> SoundWave`）可由 §22 关系词表**无断点**表达；若表达链出现断点，则 §22 词表不完备，触发 `ONT-ISS`。
- **AC-ONT-04**：节点 / 属性判定规则（`03_graph_schema.md` §2）在写入路径上有可执行的拒绝逻辑（`ONT-ISS-001` 关闭条件之一）。
- **AC-ONT-05**：Energy / Wave / Field / Process / Law / BoundaryCondition / Experiment / Hypothesis 节点类型 schema-present、unpopulated，零实例通过 schema validator。

## 10. 关联文档

- **上游（输入基线）**：
  - `GVPE-DOC-00` `docs/00_foundation/00_vision.md`（总纲：六大空间与禁令）
  - `GVPE-DOC-01` `docs/00_foundation/01_requirements.md`（需求规约：`GVPE-FR-004`, `GVPE-FR-006`, `GVPE-GPH-001/002`, `GVPE-VEC-002`）
- **下游（被消费于）**：
  - `GVPE-DOC-03` `docs/01_architecture/03_graph_schema.md`（图谱模式：§1.A 即本文档；§1.B/C 另立）
  - `GVPE-DOC-04` `docs/01_architecture/04_architecture.md`（架构总览：§4.3 依赖方向、§4.5 已注册定律 vs 已实现定律）
  - `GVPE-DOC-05` `docs/01_architecture/05_runtime_design.md`（Runtime 描述符中 `PhysicsLOD` 槽位）
  - `GVPE-DOC-11` `docs/02_modules/11_vector_design.md`（向量空间设计：物理签名子签名族）
  - `GVPE-DOC-15` `docs/03_cross_cutting/15_testing_strategy.md`（测试策略：本体评审与零实例验证）
  - `GVPE-DOC-21` `docs/04_detailed_design/21_graph_compiler_detailed_design.md`（Graph/Compiler 详细设计：本体 → `PhysicsProfile` 编译路径）

## 11. 审批签字

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 编写 | — | | |
| 校对 | | | |
| 审批 | | | |

---

## 12. 正文

> 本节保留原文档结构（§1 ~ §26 + §Review），所有叙述翻译为中文；技术名词、ID 标识符、代码块、ASCII 树图保持英文原样。

### 1. Top-level concepts (must all exist in schema; MVP instance population is a strict subset, §MVP)

```
Physics
├── Entity            ├── Field              ├── SolverModel
├── Matter             ├── Energy             ├── BoundaryCondition
├── Material           ├── Wave               ├── Observation
├── Phase               ├── Process            ├── Experiment
├── Property            ├── PhysicalLaw        ├── Hypothesis
├── State               ├── ConstitutiveModel  ├── Simulation
├── Force               ├── ApproximationModel ├── SimulationState
├── Interaction         ├── PhysicsProfile     ├── Constraint
└── VectorDescriptor    └── Result
```

### 2. Matter — "what something is"

```
Matter
├── SolidMatter    ├── PlasmaMatter     ├── PorousMatter
├── LiquidMatter    ├── GranularMatter   ├── CompositeMatter
├── GasMatter                            └── MultiphaseMatter
```

**本体规则（binding）**：Solid / Liquid / Gas 是 **Phase**，不是 Matter 的子类型。`Water HAS_PHASE Liquid` 是正确的；`Water IS_A LiquidMatter` 把对象与对象的当前状态混为一谈，会被 Ontology Review 拒绝（`ONT-ISS` 类别：Matter / Phase 混淆）。

### 3. Phase — "what state it's currently in", plus PhaseTransition

```
Phase: Solid | Liquid | Gas | Plasma | Supercritical | GranularState | MultiphaseState | MixedPhase
```

`PhaseTransition` 关系：`MELTS_TO`, `EVAPORATES_TO`, `IONIZES_TO`, `FREEZES_TO`, `CONDENSES_TO`, `SUBLIMATES_TO`——每一种 transition 都可关联 `Temperature`, `Pressure`, `Energy`, `Rate`, `BoundaryCondition`, `Material` 作为限定性上下文（条件关系，§10）。

### 4. MechanicalBehavior — "how it behaves mechanically", independent of Matter/Phase

```
MechanicalBehavior: Rigid | Elastic | Plastic | ElastoPlastic | Viscoelastic | Hyperelastic |
                     Brittle | Ductile | SoftBody | Cloth | Membrane | Rope | Rod | Shell |
                     Continuum | GranularBehavior
```

一个实体**可**被多种行为建模（`Steel CAN_BE_MODELED_AS RigidBody` **同时** `Steel CAN_BE_MODELED_AS ElastoPlasticSolid`）——该选择是 `PhysicalModel` / `ApproximationModel` 决策（§13 / §14），不是关于 Steel 本身的本体事实。

### 5. Property

类别（叶级枚举保留；per-property unit/range/confidence 表按 `00_vision.md` §0.6 深度政策推迟到下一轮——以下每个 `Property` 节点类型具有**相同**的属性形态：`value, unit, range, confidence, source, measurement_method, estimation_method, timestamp, validity, uncertainty`）：

- **Mass**：Mass, Density, CenterOfMass, InertiaTensor
- **Mechanical**：YoungModulus, PoissonRatio, ShearModulus, BulkModulus, Hardness, YieldStrength, Toughness, Stiffness, Compliance, Damping, Friction, Restitution, Viscosity, SurfaceTension
- **Thermal**：Temperature, HeatCapacity, ThermalConductivity, ThermalExpansion
- **Fluid**：Pressure, Viscosity, Compressibility, FlowRate, Density, SurfaceTension
- **Electromagnetic**：Charge, Conductivity, Permittivity, Permeability

节点 / 边判定（`03_graph_schema.md` §2）按**每个 property** 适用、**不**按类别适用：一个**被测量、被来源标注、可跨实体复用**的 property（如来自某 `Experiment` 的特定 `YoungModulus` 测量）是节点；为一次仿真烘焙进 `PhysicsProfile` 的 property 值**不**单独 node-ify。

### 6. State — time-indexed snapshot

`Position, Rotation, LinearVelocity, AngularVelocity, Acceleration, ForceState, Momentum, AngularMomentum, Temperature, Pressure, Density, Stress, Strain, Deformation, PhaseState, EnergyState, ChargeState`——每个 `State` 节点都带时间索引（`State@t0`, `State@t1`, ...）。

**本体规则**：`State` 是"它**当前所是**"；`Property` 是"它**特征性拥有**"。将 per-frame `Position` 与耐久 `Property` 混淆属于 `ONT-ISS`（State / Property 混淆）——这正是为什么 raw per-frame `State` **永远不能**被 bulk 持久化到图谱中（`03_graph_schema.md` §4，`GVPE-PROHIBIT-03/04` 的本体论基础）。

### 7. Force

```
Force: Gravity | ContactForce | FrictionForce | ElasticForce | DragForce | BuoyancyForce |
       PressureForce | ElectromagneticForce | SpringForce | UserAppliedForce
```

关系：`Force ACTS_ON Entity`。

### 8. Interaction — what happens *between* entities

```
Interaction: Collision | Contact | Friction | Adhesion | Cohesion | Drag | Buoyancy |
             HeatTransfer | Radiation | ElectromagneticInteraction | FluidStructureInteraction |
             ParticleInteraction
```

**本体规则**：`Interaction` 本质上为二元 / 多实体（"对象之间如何相互作用"）；`Process`（§16）是**单个**实体随时间所发生之事。将二者混为一谈属于 `ONT-ISS`（Process / Interaction 混淆）——例如 `Melting` 是 `Entity` **经历**的 `Process`，不是两个实体之间的 `Interaction`。

### 9. Constraint (ontology layer — semantic, not the runtime row)

```
Constraint: ContactConstraint | DistanceConstraint | JointConstraint | FixedConstraint |
            HingeConstraint | SliderConstraint | VolumeConstraint | StretchConstraint |
            BendingConstraint | AttachmentConstraint | BoundaryConstraint
```

**绑定规则**：图中的 `Constraint` 节点描述**类型与语义**；**运行时**约束行（数值化的求解条目）**只**存在于 Runtime Constraint Graph（`03_graph_schema.md` §1.B）。任何将图 `Constraint` 节点视为可直接求解的代码路径都是缺陷——必须经过 Compiler（`GVPE-FR-003`）。

### 10. Energy — first-class node, with conversion relations

```
Energy: KineticEnergy | GravitationalPotentialEnergy | ElasticPotentialEnergy | ThermalEnergy |
        InternalEnergy | ElectromagneticEnergy | ChemicalEnergy | AcousticEnergy
```

关系：`CONVERTS_TO`, `TRANSFERS_TO`, `DISSIPATES_TO`, `STORES`, `RELEASES`, `ABSORBS`。

因果链示例（另见 §25）：
`GravitationalPotentialEnergy --CONVERTS_TO--> KineticEnergy --(via Collision)--> ElasticEnergy --DISSIPATES_TO--> ThermalEnergy`

**本体规则**：Energy 永远不是 `Matter` 的子类型、也永远不是 `Entity`——它是**自身**的一类节点（`ONT-ISS` 类别：Energy / Matter 混淆，Review 中显式检查）。

### 11. Wave — independent of Matter, propagates through it

```
Wave: MechanicalWave | AcousticWave | ElectromagneticWave | SurfaceWave | PressureWave |
      ShockWave | ElasticWave
```

属性：frequency, wavelength, amplitude, phase, direction, propagation_speed, attenuation, energy_flux, polarization。
关系：`PROPAGATES_THROUGH`, `GENERATED_BY`, `REFLECTED_BY`, `REFRACTED_BY`, `ABSORBED_BY`, `SCATTERED_BY`, `CARRIES_ENERGY`。

示例：`Collision --GENERATES--> MechanicalWave --PROPAGATES_THROUGH--> Solid`。

**本体规则**：Wave 不是 `Entity`（`ONT-ISS` 类别：Wave / Entity 混淆）——它是**传播中的扰动**，与传播介质**关联**而非**等同**。

### 12. Field — continuous physical quantity over space

```
Field: GravitationalField | ElectromagneticField | PressureField | VelocityField |
       TemperatureField | DensityField | AcousticField | StressField | StrainField
```

类别：`ScalarField | VectorField | TensorField`。关系：`Entity EXISTS_IN Field`, `Field ACTS_ON Entity`。

**本体规则**：Field 与 Force 截然不同——`Field` 是连续的空间物理量；`Force` 是某具体 `Entity` 因"身处该 Field 中"而**实际承受**的量（`ONT-ISS` 类别：Field / Force 混淆）。

### 13. Process — what happens to an entity over time (single-entity temporal change)

```
PhysicalProcess: Motion | Collision | Deformation | Flow | Oscillation | Vibration | Diffusion |
                 Fracture | Compression | Expansion | Melting | Freezing | Evaporation |
                 Condensation | HeatTransfer | PhaseTransition | WavePropagation |
                 EnergyTransfer | Dissipation
```

关系：`Entity UNDERGOES Process`。示例：`Ice UNDERGOES Melting PRODUCES LiquidWater`。

### 14. PhysicalLaw — knowledge layer, not an implementation obligation

```
PhysicalLaw: NewtonLaw | ConservationOfMomentum | ConservationOfEnergy | HookeLaw |
             CoulombFriction | NavierStokes | HeatEquation | WaveEquation | MaxwellEquation |
             ConstitutiveLaw
```

**本体规则**：图中存在 `PhysicalLaw` 节点**不**意味着 Runtime 实现了它——`04_architecture.md` §4.5 跟踪哪些定律有对应的 `SolverModel`。将"已知定律"与"已实现定律"混为一谈属于 `ONT-ISS`（Law / Model 混淆）。

### 15. Model — how an entity is approximated for computation

```
PhysicalModel: RigidBodyModel | ParticleModel | SpringMassModel | PBDModel | XPBDModel |
               ElasticSolidModel | PlasticModel | FEMModel | FluidModel |
               IncompressibleFluidModel | CompressibleFluidModel | ShellModel |
               ReducedOrderModel
```

关系：`Entity MODELED_BY PhysicalModel`。正是这一点使 `PhysicsLOD`（§19）成为可能。

**本体规则**：`Model`（所选数学近似）与 `Solver`（对该 Model 进行数值求解的算法）截然不同——例如 `XPBDModel SOLVED_BY XpbdSolver` 是正确的；将 Model 与 Solver 合并为同一节点类型属于 `ONT-ISS`（Model / Solver 混淆）。

### 16. ApproximationModel — accuracy/performance trade-off, LOD-facing

`FullModel | ReducedModel | SimplifiedModel | ProxyModel | LODModel`。示例：靠近相机的 `Water` → `FullFluidModel`；中等距离 → `ParticleApproximation`；远距离 → `SurfaceApproximation`。仿真精度 vs 性能预算的决策在图谱层**可见**（喂给 `PhysicsLOD`，§19）。

### 17. BoundaryCondition

`FixedBoundary | FreeBoundary | PeriodicBoundary | PressureBoundary | TemperatureBoundary | VelocityBoundary | CollisionBoundary`——与 Fluid / FEM / Wave / Heat / Field 相关，现阶段作为 schema 占位以避免后续迁移。

### 18. Observation, Experiment, Hypothesis, Simulation

- **Observation**：`CameraObservation | VideoObservation | 3DGSObservation | SensorObservation | SimulationObservation | ManualObservation | MeasurementObservation`，每条记录 `source, timestamp, coordinate_system, confidence, noise, resolution, sampling_rate`。
- **Experiment**：连接 `Material`, `Property`, `Observation`, `BoundaryCondition`, `Result`。示例：`Experiment MEASURES YoungModulus`。
- **Hypothesis**：`Observation SUPPORTS Hypothesis`; `Hypothesis ASSUMES Material`; `Hypothesis ASSUMES PhysicsProfile`; `Hypothesis TESTED_BY Simulation`。
- **Simulation / SimulationState / SimulationResult**：`Simulation USES PhysicalModel`; `Simulation USES PhysicsProfile`; `Simulation PRODUCES SimulationState`。

**本体规则**：`Observation` 是**关于**现实的证据，**不**是现实本身；它**永远**不能在不经过 `Hypothesis` / `Experiment` 来源链的情况下直接写入 `State` / `Property`（`ONT-ISS` 类别：Observation / Reality 混淆）。

### 19. Physics LOD (consumer of §15/§16)

`PhysicsLOD: LOD0 Full Simulation | LOD1 Reduced Simulation | LOD2 Approximation | LOD3 Cached Behavior | LOD4 Static`——选择输入：距离、屏幕重要性、交互重要性、仿真预算、观测置信度、玩法重要性。MVP 仅实现 LOD0；其余级别的描述符槽位已预留（`GVPE-FR-007`）。

### 20. Physics Signature (Vector Space schema, cross-referenced from `11_vector_design.md`)

```
PhysicsSignature
├── MaterialSignature   ├── ContactSignature      ├── EnvironmentSignature
├── MotionSignature      ├── EnergySignature        └── SolverSignature
├── DeformationSignature ├── WaveSignature
└── InteractionSignature └── FieldSignature
```

实例：`ObservedPhysicsSignature | SimulatedPhysicsSignature | KnownPhysicsSignature`（`GVPE-VEC-002` 要求它们在**类型层面**可区分，**不**仅靠 tag 字段）。

### 21. Physics Profile (the only Graph→Runtime handoff shape, GVPE-FR-003)

```
PhysicsProfile { mass, density, inertia, friction, restitution, damping, stiffness, compliance,
                 viscosity, solver_type, solver_iterations, collision_profile,
                 approximation_level }
```

流水线：`Physics Knowledge Graph → Physics Compiler → PhysicsProfile → RuntimeDescriptor → Rust Runtime`。任一步骤均**不可跳过**（`03_graph_schema.md` §3 禁止 Runtime → Cypher）。

### 22. Causality relation vocabulary (must be conditional-relation-capable, not a bare taxonomy)

```
IS_A, INSTANCE_OF, HAS_MATERIAL, HAS_PHASE, HAS_PROPERTY, HAS_STATE, HAS_ENERGY, EXISTS_IN,
ACTS_ON, INTERACTS_WITH, INTERACTS_VIA, PARTICIPATES_IN, UNDERGOES, GENERATES,
PROPAGATES_THROUGH, TRANSFERS_TO, CONVERTS_TO, DISSIPATES_TO, MODELED_BY, GOVERNED_BY,
SOLVED_BY, APPROXIMATED_BY, OBSERVED_BY, MEASURED_BY, ESTIMATED_BY, INFERRED_FROM,
VALIDATED_BY, DEPENDS_ON, CAUSES, RESULTS_IN, REQUIRES, ENABLES, SUPPRESSES,
INCREASES, DECREASES, AFFECTS
```

条件示例：`HigherTemperature DECREASES Viscosity`（关系由单调依赖所限定，**非**无条件边）。

### 23. Spatial relations (3DGS-facing, §13)

`INSIDE, OUTSIDE, INTERSECTS, CONTACTS, ABOVE, BELOW, NEAR, FAR, ATTACHED_TO, CONTAINS, CONNECTED_TO`

### 24. Temporal relations

`BEFORE, AFTER, DURING, STARTS_AT, ENDS_AT, PERSISTS_UNTIL`——适用于 `Observation`, `SimulationState`, `Process`, `Collision`, `Wave`, `PhaseTransition`。

### 25. Physical Causality — the chain the whole ontology exists to support

```
Cause → Process → StateChange → EnergyTransfer → ObservableEffect
```

工作示例：
```
ExternalForce --CAUSES--> Acceleration --CAUSES--> VelocityChange --CAUSES--> Collision
  --GENERATES--> Deformation --STORES--> ElasticEnergy --RELEASES--> KineticEnergy
  --GENERATES--> SoundWave
```

该链是 §22 关系词表的**具体测试用例**——若一个真实物理场景无法**用 §22 的关系**表达为一条**不断裂**的链，则词表不完备、且构成 `ONT-ISS`。

### 26. MVP ontology instance scope (schema above is NOT scoped down — only instance population is)

填充：`Entity, Material, Phase, Property (subset), PhysicalModel (RigidBodyModel only), Solver, PhysicsProfile, Simulation, Observation (SimulationObservation only)`。

Schema 存在但 MVP 不填充：`Energy, Wave, Field, Process, PhysicalLaw, BoundaryCondition, Experiment, Hypothesis` 以及 `MechanicalBehavior` 的大部分。这些必须能**零实例**通过 schema validator，且后续填充时**不**需要任何 schema 变更——这正是"不得要求破坏性迁移"的具体可测试含义（验证手段见 `GVPE-DOC-15`）。

### Review — Ontology Review (mandatory before this baseline is accepted)

逐项检查文中出现的 11 类混淆：

| # | 检查项 | 结果 | 登记为 |
|---|---|---|---|
| 1 | 将 Solid / Liquid / Gas 误用为对象分类 | 在 §2 显式拒绝 | — |
| 2 | Phase vs Material 混淆 | 在 §2 / §3 区分 | — |
| 3 | Energy 误作 Matter | 在 §10 区分 | — |
| 4 | Wave 误作 Entity | 在 §11 区分 | — |
| 5 | Field vs Force 混淆 | 在 §12 区分 | — |
| 6 | State vs Property 混淆 | 在 §6 区分 | — |
| 7 | Process vs Interaction 混淆 | 在 §8 / §13 区分 | — |
| 8 | Law vs Model 混淆 | 在 §14 区分 | — |
| 9 | Model vs Solver 混淆 | 在 §15 区分 | — |
| 10 | Observation vs Reality 混淆 | 在 §18 区分 | — |
| 11 | Graph Node vs Runtime State 混淆 | 在 §6 / §9 区分；由 `03_graph_schema.md` §2 / §4 强制落地 | **ONT-ISS-001**（未关闭，见下） |

**ONT-ISS-001**

- **严重度**：Medium
- **发现**：规则 11 在本文档中表述正确，但**本文档本身**无法**强制**该规则——强制属于 `gvpe-graph` 实现层的工作（写入路径守卫拒绝 bulk per-frame `State` 写入）。在该守卫在代码中实际存在之前，该规则仅是文档级的。
- **建议**：`03_graph_schema.md` §4 必须规定守卫机制；`15_testing_strategy.md` 必须包含一项"尝试 bulk per-frame 写入并断言被拒绝"的测试；当两者均落实后再关闭本问题。
- **影响范围**：`gvpe-graph`, `03_graph_schema.md`, `15_testing_strategy.md`。
