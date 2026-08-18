# Physical Retrieval Engine（PRE）ER図・UML図集

## 文书管理表

| 项目 | 内容 |
|---|---|
| 文书编号 | PRE-DOC-14 |
| 版本 | v0.1 |
| 状态 | Draft |
| 输入基线 | 02_PRE_Basic_Design.md（v0.1.5）、08_PRE_Detailed_Design.md（v0.1.4） |
| 定位说明 | 本文书不引入新需求或新决策，是 02/08 号文档中已有文字描述与 ASCII 框图的**可渲染图形化产物**（GitHub 原生渲染 Mermaid）。若图与正文文字冲突，以正文（02/08号文档）为准，本文书应视为过期并修正，而非另立事实来源。 |

## 改订履历

| 版本 | 日期 | 变更内容 | 作成者 |
|---|---|---|---|
| v0.1 | 2026-08-17 | 初版：ER 图（pre-atlas 关系型 schema）、UML 类图（核心数据模型、SolverPlugin 体系、pre-engine-api 契约、Tier 2 C ABI 表面）、UML 时序图（三条核心路径）、Physical Graph 示例图、Tier 分层组件图 | Claude |

---

## 目录

1. ER 图：`pre-atlas` 关系型 Schema
2. UML 类图：核心数据模型（PhysicsExperience 组合关系）
3. UML 类图：SolverPlugin trait 体系
4. UML 类图：`pre-engine-api` 宿主中立契约
5. UML 类图：Tier 2 C ABI 表面（API 设计可视化）
6. UML 时序图：生成路径
7. UML 时序图：检索验证路径
8. UML 时序图：学习闭环路径
9. UML 时序图：Tier 1 回放与异步查询桥接
10. 组件图：宿主分层集成（Tier 0~3）
11. Physical Graph 构造示例（图论构造能力）

---

## 1. ER 图：`pre-atlas` 关系型 Schema

对应 08号文档 §9.1。仅覆盖 relational store 部分；vector store（HNSW 索引文件）与 blob store（文件系统）不是关系型表，不纳入 ER 图，以 `experience_id` 外键概念上关联，图中以注释标注。

```mermaid
erDiagram
    EXPERIENCES ||--o| VALIDATION_METRICS : "验证后产生"
    EXPERIENCES ||--o{ GRAPH_EDGES_VIEW : "查询时派生（非持久表）"

    EXPERIENCES {
        text id PK "UUID v7"
        integer created_at
        text provenance_source
        text provenance_generator
        text parent_experience_id FK "自引用，指向 refinement 的源 experience"
        text solver_id
        text solver_version
        text material_model
        text validation_status "Candidate | Validated"
        text response_blob_path "指向 blob store，非关系型外键"
        integer signature_schema_version
        text signature_json
        integer encoder_version "可为空：尚未编码"
        text determinism_json
    }

    VALIDATION_METRICS {
        text experience_id PK_FK
        real position_error
        real velocity_error
        real deformation_error
        real frequency_error
        real damping_error
        real contact_timing_error
        real verification_score
        text identifiability "low | normal"
    }

    GRAPH_EDGES_VIEW {
        text experience_id FK "查询时限定范围（PRE-GRAPH-003）"
        text from_node
        text to_node
        text edge_kind "Contact|Constraint|Attachment|FieldAction"
    }
```

**读图要点**：`GRAPH_EDGES_VIEW` 用虚线语义标注为"非持久表"——它是 08号文档 §23.2 `build_graph_view()` 在查询时从 `signature_json`/`response_blob_path` 指向的数据现算得到的逻辑视图，ER 图中画出是为了表达"图论查询的数据来源"，不代表数据库中真实存在这张表（对应 ADR-013）。

对应需求：PRE-DATA-001~003, PRE-GRAPH-001。

---

## 2. UML 类图：核心数据模型

对应 08号文档 §2，展示 `PhysicsExperience` 的组合关系（非继承）。

```mermaid
classDiagram
    class PhysicsExperience {
        +ExperienceId id
        +Timestamp created_at
        +Provenance provenance
        +InitialState initial_state
        +BoundaryConditions boundary_conditions
        +Excitation excitation
        +MaterialSpec material
        +SolverSpec solver
        +BlobRef response_ref
        +PhysicalSignature signature
        +EmbeddingSet embeddings
        +ValidationRecord validation
        +DeterminismMetadata determinism
    }
    class Provenance {
        +ProvenanceSource source
        +GeneratorId generator
        +String generator_version
        +Option~ExperienceId~ parent_experience_id
    }
    class InitialState {
        +GeometryRef geometry_ref
        +DiscretizationSpec discretization
        +Transform initial_pose
        +VelocityField initial_velocity
    }
    class StandardPhysicalResponse {
        +SignatureSchemaVersion schema_version
        +f64 duration
        +Vec~f64~ sample_times
        +Vec~LandmarkId~ landmarks
        +Vec~Vec~Vec3~~ position
        +Option~DeformationSeries~ deformation
        +Vec~ContactEvent~ contact_events
        +Vec~ConstraintEvent~ constraint_events
    }
    class PhysicalSignature {
        +GeometryFeatures geometry
        +KinematicsFeatures kinematics
        +DeformationFeatures deformation
        +TemporalFeatures temporal
        +ContactFeatures contact
        +MaterialFeatures material
        +ConstraintFeatures constraints
        +FieldFeatures field
    }
    class EmbeddingSet {
        +EncoderVersion encoder_version
        +Vec~f32~ behavior_vector
        +Vec~f32~ deformation_vector
        +Vec~f32~ temporal_vector
        +Vec~f32~ global_vector
    }

    PhysicsExperience *-- Provenance
    PhysicsExperience *-- InitialState
    PhysicsExperience *-- PhysicalSignature
    PhysicsExperience *-- EmbeddingSet
    PhysicsExperience ..> StandardPhysicalResponse : response_ref（blob 引用，非内联组合）
    PhysicalSignature <.. StandardPhysicalResponse : extract_signature()（派生，非组合）
    EmbeddingSet <.. PhysicalSignature : encode_v1()（派生，非组合）

    note for PhysicsExperience "四层分离（ADR-003）：Raw State 不在此图中出现——\nsolver 私有，不对外暴露"
```

**读图要点**：`PhysicsExperience *-- PhysicalSignature`（实心菱形，组合）表示 signature 内联存储；`PhysicsExperience ..> StandardPhysicalResponse`（虚线箭头，依赖/引用）表示 response 只存 blob 引用，不内联——这条区别直接对应 PRE-DATA-002（metadata 查询不触发 blob 读取）在类图层面的体现。

对应需求：PRE-DATA-001, PRE-PHY-001, PRE-PHY-005。

---

## 3. UML 类图：SolverPlugin trait 体系

对应 08号文档 §3~§6，含 §12 的 `MockSolverPlugin`（虚线区分生产实现与测试替身）。

```mermaid
classDiagram
    class SolverPlugin {
        <<trait>>
        +id() SolverId
        +version() SolverVersion
        +init(InitialState, BoundaryConditions, MaterialSpec, SolverParameters, seed) Result~SolverHandle, SolverError~
        +step(SolverHandle, ExcitationEvent[], dt, substeps) Result~RawSolverState, SolverError~
        +to_standard_response(RawSolverState[], sample_times) Result~StandardPhysicalResponse, ResponseConversionError~
    }
    class RigidSolverPlugin {
        semi-implicit Euler
        解析碰撞检测
    }
    class XpbdSolverPlugin {
        distance/bending/volume/attachment constraints
    }
    class MpmSolverPlugin {
        APIC 转移, P2G/G2P
    }
    class FemStubSolverPlugin {
        <<stub>>
        NotImplemented
    }
    class MockSolverPlugin {
        <<test double>>
        +ResponseScript script
    }
    class ResponseScript {
        <<enum>>
        Fixed(RawSolverState[])
        DivergeAtStep
        FailOnInit
        FailAtStep
    }

    SolverPlugin <|.. RigidSolverPlugin
    SolverPlugin <|.. XpbdSolverPlugin
    SolverPlugin <|.. MpmSolverPlugin
    SolverPlugin <|.. FemStubSolverPlugin
    SolverPlugin <|.. MockSolverPlugin
    MockSolverPlugin *-- ResponseScript

    note for MockSolverPlugin "位于 pre-testkit（12号文档 §1），\n[dev-dependencies] only，不进入生产依赖树"
```

对应需求：PRE-FR-002, PRE-FR-003, PRE-TK-002。

---

## 4. UML 类图：`pre-engine-api` 宿主中立契约

对应 08号文档 §18。展示 ADR-010 的核心主张——中立类型与状态机不依赖任何宿主 SDK。

```mermaid
classDiagram
    class LandmarkTransform {
        +f64[3] position
        +Option~f64[4]~ rotation
    }
    class SpatialConvention {
        +Handedness handedness
        +Axis up_axis
        +Axis forward_axis
        +f64 length_scale
        +convert(LandmarkTransform) LandmarkTransform
        PRE_CANONICAL$
        BEVY$
        GODOT$
        UNITY$
        UNREAL$
        BLENDER$
    }
    class PlaybackCursor {
        -StandardPhysicalResponse response
        -SpatialConvention convention
        +sample(LandmarkId, f64 t) Result~LandmarkTransform, PlaybackError~
    }
    class QuerySession {
        -QueryId id
        -QueryState state
        +submit(QueryRequest, QueryExecutor)$
        +poll() QueryState
        +cancel()
    }
    class QueryExecutor {
        <<trait>>
        +spawn(work) void
    }
    class QueryState {
        <<enum>>
        Pending
        Running
        Done(CandidateExplanation)
        Failed(QueryError)
    }

    PlaybackCursor --> SpatialConvention : uses
    PlaybackCursor ..> LandmarkTransform : produces
    QuerySession --> QueryExecutor : delegates to
    QuerySession --> QueryState

    class BevyQueryExecutor {
        AsyncComputeTaskPool
    }
    class GodotQueryExecutor {
        独立线程 + call_deferred
    }
    QueryExecutor <|.. BevyQueryExecutor
    QueryExecutor <|.. GodotQueryExecutor

    note for LandmarkTransform "刻意使用 f64[3] 而非数学库类型：\n必须是 POD，才能跨 Tier 2 C ABI 传递（08号§18.1）"
```

对应需求：PRE-ENG-003~006, PRE-ENG-009。

---

## 5. UML 类图：Tier 2 C ABI 表面（API 设计可视化）

对应 08号文档 §21.1。这是本项目目前唯一面向外部语言的公开 API 边界，用类图表达其"仅不透明句柄 + POD"的约束（PRE-ENG-009）。

```mermaid
classDiagram
    class PreContext {
        <<opaque handle>>
    }
    class PrePlayback {
        <<opaque handle>>
    }
    class PreTransform {
        <<POD struct>>
        +double[3] position
        +double[4] rotation
        +int32 has_rotation
    }
    class PreFfiFunctions {
        <<extern "C" surface>>
        +pre_ffi_abi_version() uint32
        +pre_context_create(PreContextDesc*) PreContext*
        +pre_context_destroy(PreContext*) void
        +pre_playback_sample(PrePlayback*, uint64, double, PreTransform*) int32
        +pre_last_error_message(char*, size_t, size_t*) int32
    }

    PreFfiFunctions ..> PreContext : creates/destroys
    PreFfiFunctions ..> PrePlayback : operates on
    PreFfiFunctions ..> PreTransform : fills (out-param)

    note for PreFfiFunctions "每个函数体以 catch_unwind 包裹（PRE-EMB-004）\n不传递泛型/trait对象/枚举载荷/Result/String"
```

**读图要点**：故意不画"继承"或"组合"——C ABI 没有面向对象概念，句柄之间只有"由哪个函数创建/操作"这一种关系，图中全部用依赖箭头（`..>`）表达，这本身就是对 Tier 2 设计约束的图形化提醒。

对应需求：PRE-ENG-009, PRE-EMB-004。

---

## 6. UML 时序图：生成路径

对应 08号文档 §16.1。

```mermaid
sequenceDiagram
    participant Gen as pre-gen
    participant Solver as pre-solver-*
    participant Sig as pre-signature
    participant Enc as pre-encoder
    participant Atlas as pre-atlas

    Gen->>Gen: sample_parameter_space(LHS)
    loop 每个采样参数
        Gen->>Solver: init(InitialState, BoundaryConditions, Material, Params, seed)
        Solver-->>Gen: SolverHandle
        loop 每个仿真步
            Gen->>Solver: step(handle, excitation, dt, substeps)
            Solver-->>Gen: RawSolverState
            alt 检测到 NaN/Inf
                Gen->>Gen: 标记 Invalid，记入 GenerationFailureLog（不进入 Atlas）
            end
        end
        Gen->>Solver: to_standard_response(history, sample_times)
        Solver-->>Gen: StandardPhysicalResponse
        Gen->>Sig: extract_signature(response, material, bc, excitation)
        Sig-->>Gen: PhysicalSignature
        Gen->>Enc: encode_v1(signature, atlas_stats)
        Enc-->>Gen: EmbeddingSet
        Gen->>Atlas: store(PhysicsExperience{...})
    end
```

---

## 7. UML 时序图：检索验证路径

对应 08号文档 §16.2，含窗口切分（ISS-009 修正）与参数不可辨识检测（ISS-006 修正）。

```mermaid
sequenceDiagram
    participant Obs as ObservedResponse
    participant Eng as pre-engine-api / caller
    participant Sig as pre-signature
    participant Enc as pre-encoder
    participant Ret as pre-retrieval
    participant Ver as pre-verify
    participant Ref as pre-refine

    Obs->>Eng: StandardPhysicalResponse
    Eng->>Eng: split_observation(match_ratio=0.65)
    Note right of Eng: MatchWindow / HeldOutWindow

    Eng->>Sig: extract_signature(MatchWindow)
    Sig-->>Eng: query signature
    Eng->>Enc: encode_v1(query signature)
    Enc-->>Eng: query embedding

    Eng->>Ret: search(query embedding, Top-N)
    Ret->>Ret: ANN 粗召回 → post-filter → 精排（子向量融合）
    alt 全部候选相似度 < novel_similarity_threshold
        Ret-->>Eng: NovelDynamics{LowSimilarity}
    else
        Ret-->>Eng: Top-K candidates
    end

    loop 每个 Top-K 候选
        Eng->>Ver: verify(candidate, MatchWindow, HeldOutWindow)
        Ver->>Ver: resimulate + 逐维误差（仅用 HeldOutWindow 打分）
        Ver-->>Eng: VerificationMetrics
    end
    alt 最优候选误差 > novel_error_threshold
        Ver-->>Eng: NovelDynamics{HighSimulationError}
    end
    Eng->>Ver: detect_identifiability(Top-M)
    alt 参数分散、误差相近
        Ver-->>Eng: Identifiability::Low（展示全部 Top-M）
    else
        Ver-->>Eng: Identifiability::Normal
    end

    Eng->>Ref: optimize(Top-M candidates, budget)
    Ref-->>Eng: refined parameters
    Eng-->>Obs: CandidateExplanation[]（含评分明细，08号§26）
```

---

## 8. UML 时序图：学习闭环路径

对应 08号文档 §16.3。

```mermaid
sequenceDiagram
    participant Pipeline as 检索验证路径（图7）
    participant Atlas as pre-atlas

    Pipeline->>Pipeline: Best/Top-M 候选 verification_score 达到 Validated 阈值
    Pipeline->>Pipeline: 构造新 PhysicsExperience（parent_experience_id 指向原候选）
    Pipeline->>Atlas: store(new experience, status=Validated)
    Atlas->>Atlas: 写入 relational + vector + blob；HNSW 增量插入（非全量重建）
    Note over Atlas: 后续检索可命中该新记录（AC-05 闭环验证点）
```

---

## 9. UML 时序图：Tier 1 回放与异步查询桥接

对应 08号文档 §18.3, §19。以 Bevy 为例，Godot 语义相同（仅调度机制不同，见 08号文档 §20）。

```mermaid
sequenceDiagram
    participant Schedule as Bevy Update Schedule
    participant Playback as playback_system
    participant Cursor as PlaybackCursor
    participant Pool as AsyncComputeTaskPool
    participant Ret as pre-retrieval/pre-verify

    loop 每帧
        Schedule->>Playback: run
        Playback->>Cursor: sample(landmark, time)
        Cursor-->>Playback: LandmarkTransform（插值，钳制边界）
        Playback->>Playback: 写入 Transform 组件
    end

    par 非阻塞查询（PRE-ENG-005）
        Schedule->>Pool: QuerySession::submit(request)
        Pool->>Ret: search() + verify()（后台线程）
        Note over Schedule,Pool: 主 Schedule 不等待，继续下一帧
    and
        loop 每帧
            Schedule->>Pool: QuerySession::poll()
            alt 完成
                Pool-->>Schedule: Done(CandidateExplanation)
            else 进行中
                Pool-->>Schedule: Running
            end
        end
    end
```

---

## 10. 组件图：宿主分层集成（Tier 0~3）

对应 02号文档 §33.1 的 ASCII 图的 Mermaid 化版本，与该图逐一对应，不新增内容。

```mermaid
graph TB
    subgraph Core["核心（对宿主 SDK 零依赖，PRE-ENG-002）"]
        Core_pkg["pre-core"]
        Solver["pre-solver-*"]
        Retrieval["pre-retrieval / pre-verify / pre-refine / pre-atlas"]
    end

    subgraph Contract["pre-engine-api（宿主中立契约）"]
        PC["PlaybackCursor"]
        QS["QuerySession"]
        SC["SpatialConvention"]
    end

    subgraph Tier1["Tier 1: Rust 链接适配层"]
        Bevy["pre-bevy"]
        Godot["pre-godot"]
    end

    subgraph Tier2["Tier 2: C ABI（MVP 仅设计）"]
        Ffi["pre-ffi"]
    end

    subgraph Tier3["Tier 3: Python 绑定（MVP 仅设计）"]
        Py["pre-python"]
    end

    Core_pkg --> Contract
    Contract --> Bevy
    Contract --> Godot
    Contract --> Ffi
    Contract --> Py

    Bevy -.-> BevyApp["Bevy App"]
    Godot -.-> GodotProj["Godot 项目"]
    Ffi -.-> Unreal["Unreal (C++)"]
    Ffi -.-> Unity["Unity (C#)"]
    Py -.-> DCC["Blender / Maya / Houdini"]

    style Core fill:#e8f4ea,stroke:#2d6a4f
    style Contract fill:#fff3cd,stroke:#997404
    style Tier1 fill:#e7f0fd,stroke:#1a56db
    style Tier2 fill:#f0f0f0,stroke:#666
    style Tier3 fill:#f0f0f0,stroke:#666
```

**读图要点**：Tier 2/3 用灰色标注"MVP 仅设计"，与 Tier 1 的实现状态做视觉区分——避免读者把这张图误读为"四个 Tier 现在都已实现"。

---

## 11. Physical Graph 构造示例（图论构造能力）

对应 02号文档 §36、08号文档 §23。以「布料一角固定、自由端接触地面」这一 XPBD 场景为例，展示 `build_graph_view()` 的产出与 `to_mermaid()` 的导出效果——本图是 08号文档 §23.4 `GraphExporter::to_mermaid()` 的**真实产出格式示例**，非独立绘制。

```mermaid
graph TD
    L0["landmark(0,0,1)<br/>固定点"]
    L1["landmark(1,0,1)"]
    L5["landmark(0.5,0.5,0)<br/>自由角"]
    Ground["Field: Ground Contact"]
    Gravity["Field: Gravity"]

    L0 -->|Attachment| L1
    L1 -->|Constraint: distance| L5
    L5 -->|Contact: restitution=0.2| Ground
    L0 -->|FieldAction| Gravity
    L1 -->|FieldAction| Gravity
    L5 -->|FieldAction| Gravity
```

**遍历查询示例**（对应 08号文档 §23.3）：`traverse(start=L5, max_depth=2)` 从自由角出发、2 跳内可达节点为 `{L5, L1, Ground, Gravity}`——`L0` 因距离 3 跳而不在结果中，演示了 `max_depth` 硬上限（PRE-GRAPH-006）如何实际限制遍历范围，而非仅是文档中的口头约束。

对应需求：PRE-GRAPH-001, PRE-GRAPH-002, PRE-GRAPH-004.
