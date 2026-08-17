# PRE Testkit（自动测试用 Mock 项目）详细设计书

## 文书管理表

| 项目 | 内容 |
|---|---|
| 文书编号 | PRE-DOC-12 |
| 版本 | v0.1 |
| 状态 | Draft |
| 输入基线 | 11_PRE_Testkit_Basic_Design.md |
| 关联文书 | 08（核心详细设计，本文书的 trait/类型均引用其定义）、09（测试用例一览） |

## 改订履历

| 版本 | 日期 | 变更内容 | 作成者 |
|---|---|---|---|
| v0.1 | 2026-08-17 | 初版：具体类型定义、算法、错误处理、与 09 号文档的用例映射 | Claude |

## 承认栏

| 角色 | 承认日期 | 签署 |
|---|---|---|
| Rust系统负责人（ST-03） | — | 未承认 |

---

## 目录

1. `MockSolverPlugin` 详细设计
2. `InMemoryAtlas` 详细设计
3. `FixtureBuilder` 详细设计
4. `GoldenDataset` 详细设计与数据格式
5. 近似相等断言辅助详细设计
6. `MockBevyHarness` 详细设计
7. 错误处理与 panic 策略
8. 与 09 号文档测试用例的映射表

---

## 1. `MockSolverPlugin` 详细设计

### 1.1 类型定义

```rust
struct MockSolverPlugin {
    id: SolverId,
    version: SolverVersion,
    script: ResponseScript,
}

enum ResponseScript {
    Fixed(Vec<RawSolverState>),
    DivergeAtStep { before: Vec<RawSolverState>, diverge_step: u32 },
    FailOnInit(SolverError),
    FailAtStep { before: Vec<RawSolverState>, fail_step: u32, error: SolverError },
}
```

### 1.2 trait 实现

```rust
impl SolverPlugin for MockSolverPlugin {
    fn id(&self) -> SolverId { self.id }
    fn version(&self) -> SolverVersion { self.version }

    fn init(&self, _initial: &InitialState, _bc: &BoundaryConditions,
             _material: &MaterialSpec, _params: &SolverParameters, _seed: u64)
             -> Result<SolverHandle, SolverError> {
        match &self.script {
            ResponseScript::FailOnInit(err) => Err(err.clone()),
            _ => Ok(SolverHandle::mock(0)),   // 内部游标从 0 开始
        }
    }

    fn step(&self, handle: &mut SolverHandle, _excitation: &[ExcitationEvent],
            _dt: f64, _substeps: u32) -> Result<RawSolverState, SolverError> {
        let cursor = handle.advance_mock_cursor();
        match &self.script {
            ResponseScript::Fixed(states) => Ok(states[cursor].clone()),
            ResponseScript::DivergeAtStep { before, diverge_step } => {
                if cursor as u32 == *diverge_step {
                    Ok(RawSolverState::nan_poisoned())   // 触发 §15 SOLVER_NUMERICAL_DIVERGENCE 检测路径
                } else {
                    Ok(before[cursor].clone())
                }
            }
            ResponseScript::FailAtStep { before, fail_step, error } => {
                if cursor as u32 == *fail_step { Err(error.clone()) } else { Ok(before[cursor].clone()) }
            }
            ResponseScript::FailOnInit(_) => unreachable!("init() 已提前返回 Err，不会到达 step()"),
        }
    }

    fn to_standard_response(&self, history: &[RawSolverState], sample_times: &[f64])
             -> Result<StandardPhysicalResponse, ResponseConversionError> {
        // 复用与真实 solver 相同的通用转换辅助函数（若已抽取为 pre-solver-api 的共享工具），
        // 或直接返回 FixtureBuilder 预置的对应 StandardPhysicalResponse——
        // 具体选择取决于测试意图：若测试目标是 to_standard_response 本身的逻辑，用真实转换；
        // 若测试目标是下游消费者，直接返回固定 Response 更简单。两种模式都需支持，
        // 由 MockSolverPlugin 的构造参数决定（见 1.3）。
        default_or_fixed_conversion(history, sample_times, &self.fixed_response_override)
    }
}
```

### 1.3 数值发散注入的具体行为

`RawSolverState::nan_poisoned()` 返回一个所有数值字段均为 `NaN` 的状态。08号文档 §3.2 规定 `step()` 边界内每步检测 NaN/Inf；`MockSolverPlugin` 精确在 `diverge_step` 返回该状态，使调用方（`pre-solver-api` 的生命周期封装逻辑）在该步检测到发散并返回 `SOLVER_NUMERICAL_DIVERGENCE`，从而让 TC-SOLVER-005 可以断言"恰好在第 N 步失败"而非"某个不确定的时刻失败"。

## 2. `InMemoryAtlas` 详细设计

### 2.1 类型定义（对应 11号文档 §3.2）

```rust
struct InMemoryAtlas {
    metadata: RwLock<HashMap<ExperienceId, ExperienceMetadata>>,
    vectors: RwLock<HashMap<EncoderVersion, HashMap<ExperienceId, EmbeddingSet>>>,
    blobs: RwLock<HashMap<ExperienceId, StandardPhysicalResponse>>,
    blob_read_counter: AtomicUsize,
}
```

### 2.2 trait 实现要点

```rust
impl AtlasStorage for InMemoryAtlas {
    fn store(&self, exp: &PhysicsExperience) -> Result<(), AtlasError> {
        self.metadata.write().insert(exp.id, ExperienceMetadata::from(exp));
        self.vectors.write().entry(exp.embeddings.encoder_version).or_default()
            .insert(exp.id, exp.embeddings.clone());
        self.blobs.write().insert(exp.id, exp.response.clone());  // 测试环境下 response 可直接持有，无需引用+blob文件
        Ok(())
    }

    fn get_metadata(&self, id: ExperienceId) -> Result<ExperienceMetadata, AtlasError> {
        // 注意：不递增 blob_read_counter
        self.metadata.read().get(&id).cloned().ok_or(AtlasError::NotFound(id))
    }

    fn get_response(&self, id: ExperienceId) -> Result<StandardPhysicalResponse, AtlasError> {
        self.blob_read_counter.fetch_add(1, Ordering::SeqCst);   // TC-ATLAS-002 断言点
        self.blobs.read().get(&id).cloned().ok_or(AtlasError::NotFound(id))
    }
}

impl InMemoryAtlas {
    fn blob_reads(&self) -> usize { self.blob_read_counter.load(Ordering::SeqCst) }
}
```

`get_metadata()` 不触碰 `blob_read_counter`，`get_response()`（或任何触及 `blobs` 的路径）才递增——这一区分直接对应 PRE-DATA-002 的架构约束，是 TC-ATLAS-002（"仅查询 metadata 不触发 blob 读取"）得以用 mock 断言的关键设计点，回应了 10 号文档 OQ-TK-01。

### 2.3 与真实 `pre-atlas` 的已知差异（明确记录，不隐藏）

- 无并发写入冲突模拟：真实 SQLite 在高并发写入下可能有锁等待，`InMemoryAtlas` 用简单 `RwLock` 不复现这类行为，相关测试需在真实层（11号文档 §2）覆盖。
- 无持久化：进程结束数据即丢失，这是有意为之（每个测试独立空实例）。
- 不做 encoder_version 索引重建等昂贵操作的性能模拟——mock 不用于性能测试，性能基准测试（06号文档）必须针对真实实现。

## 3. `FixtureBuilder` 详细设计

### 3.1 API 形态

```rust
struct ExperienceFixtureBuilder {
    template: PhysicsExperience,   // 从 GoldenDataset 中选取的默认模板
}

impl ExperienceFixtureBuilder {
    fn new() -> Self { Self { template: GoldenDataset::default_template() } }
    fn with_solver(mut self, id: SolverId) -> Self { self.template.solver.id = id; self }
    fn with_material(mut self, spec: MaterialSpec) -> Self { self.template.material = spec; self }
    fn with_validation_status(mut self, status: ValidationStatus) -> Self { ... self }
    fn build(self) -> PhysicsExperience { self.template }
}
```

对 `StandardPhysicalResponse`/`PhysicalSignature`/`EmbeddingSet` 提供结构相同的 Builder，均以 Golden Dataset 中的真实条目为默认模板（对应 11号文档 §3.3 "复用 Golden Dataset 而非另造默认值"的设计决策）。

### 3.2 为何不用第三方 derive-builder 类宏

呼应 PRE-TK-010：手写 Builder（每个类型约 5~10 个 `with_*` 方法）的代码量可控，引入宏依赖换来的样板代码节省对本项目规模而言不划算，且手写版本的编译错误信息更直接（对测试代码的可调试性更友好）。

## 4. `GoldenDataset` 详细设计与数据格式

### 4.1 数据格式选型

采用 RON（Rusty Object Notation）而非 JSON：RON 支持 Rust 枚举的自然表示（无需额外的 tag 字段约定），与 `PhysicsExperience` 中大量使用的枚举类型（`SolverId`、`ValidationStatus` 等）契合度更高，减少手工编辑 fixture 文件时的心智负担。数据文件位置：`pre-testkit/fixtures/golden/*.ron`，每个 solver 类型一个子目录。

### 4.2 生成脚本

```
pre-testkit/scripts/generate_golden_dataset.rs（或等价的 xtask 命令）
    输入：固定 seed + 覆盖 Rigid/XPBD/MPM 三类的最小参数集合
    流程：调用真实 solver（不是 mock！）生成一次真实仿真 → 真实 extract_signature() → 真实 encode_v1()
         → 序列化为 RON → 写入 fixtures/golden/
```

**关键设计点**：Golden Dataset 由**真实实现**生成，而非手工编造——这保证黄金数据集反映的是"某个已知良好版本的真实行为"，回归测试比较的是"当前实现 vs 曾经验证过的真实实现"，而不是"当前实现 vs 凭空编造的期望值"。生成脚本本身不是测试的一部分，是一次性/按需重跑的开发工具。

### 4.3 加载与版本校验

```rust
impl GoldenDataset {
    fn load() -> Self {
        let entries = load_ron_files("pre-testkit/fixtures/golden/");
        for e in &entries {
            assert_eq!(e.signature.schema_version, CURRENT_SIGNATURE_SCHEMA_VERSION,
                "Golden fixture schema_version 落后于当前代码，需要重新生成（见 §4.2）");
        }
        Self { entries }
    }
}
```

版本不匹配时 `load()` 直接 panic 并给出可操作的错误信息（重新生成命令），而不是静默忽略版本差异——落实 PRE-TK-008 的"显式失败"要求。

## 5. 近似相等断言辅助详细设计

```rust
fn assert_approx_eq(a: f64, b: f64, epsilon: f64, ctx: &str) {
    if (a - b).abs() > epsilon {
        panic!("assert_approx_eq failed [{ctx}]: |{a} - {b}| > {epsilon}");
    }
}

fn assert_vec3_approx_eq(a: Vec3, b: Vec3, epsilon: f64, ctx: &str) {
    assert_approx_eq(a.x, b.x, epsilon, &format!("{ctx}.x"));
    assert_approx_eq(a.y, b.y, epsilon, &format!("{ctx}.y"));
    assert_approx_eq(a.z, b.z, epsilon, &format!("{ctx}.z"));
}
```

`ctx` 参数强制调用方提供上下文字符串，使断言失败时的输出能直接定位到"哪个字段、哪个 landmark、哪个时间点"，而非只有裸数字（呼应本项目一贯的"可观测性优先"精神，08号文档 §26 的同一原则延伸到测试代码本身）。

## 6. `MockBevyHarness` 详细设计

```rust
struct MockBevyHarness {
    app: bevy::app::App,
    virtual_time: f64,
}

impl MockBevyHarness {
    fn new() -> Self {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);   // 不加载真实渲染/窗口插件，headless
        Self { app, virtual_time: 0.0 }
    }

    fn with_plugin(mut self, plugin: impl Plugin) -> Self { self.app.add_plugins(plugin); self }

    fn spawn_landmark_entities(mut self, response: &StandardPhysicalResponse) -> Self {
        for &landmark in &response.landmarks {
            self.app.world_mut().spawn((PreLandmark { landmark_id: landmark, .. }, Transform::default()));
        }
        self
    }

    fn advance_frames(mut self, n: u32, dt: f64) -> Self {
        for _ in 0..n {
            self.virtual_time += dt;
            self.app.world_mut().resource_mut::<Time>().advance_by(Duration::from_secs_f64(dt));
            self.app.update();
        }
        self
    }

    fn assert_transform(&self, landmark: LandmarkId, expected: Vec3, epsilon: f64) {
        // 查询对应实体的 Transform，用 §5 的 assert_vec3_approx_eq 断言
    }
}
```

使用 Bevy 自带的 `MinimalPlugins` 而非完整 `DefaultPlugins`：跳过渲染后端初始化（在无 GPU/无显示器的 CI 容器中这一步经常失败或极慢），只保留 ECS 调度与 `Time` 资源，满足 08号文档 §18.4 System 调度测试的需要。

## 7. 错误处理与 panic 策略

`pre-testkit` 是测试专用代码，与 08号文档 §15 面向生产环境的错误码表原则不同：**测试基础设施代码里的契约违反直接 panic**（如访问不存在的 landmark、加载版本不匹配的 fixture），因为：

1. panic 会让测试立即失败并给出清晰堆栈，这正是测试代码期望的行为（不需要像生产代码那样考虑"调用方如何优雅恢复"）。
2. 引入 `Result` 返回类型并强制每个测试辅助函数都处理错误，会显著增加测试代码本身的样板代码量，与"testkit 应该降低测试编写成本"的初衷矛盾。

`MockSolverPlugin`/`InMemoryAtlas` 是例外：它们实现的是生产 trait（`SolverPlugin`/`AtlasStorage`），必须遵守该 trait 的 `Result` 签名，因为被测代码（真实调用方）需要按真实契约处理它们返回的错误——这里的 `Result` 不是"testkit 自己的错误处理"，而是"如实模拟生产契约"，两者不矛盾。

## 8. 与 09 号文档测试用例的映射表

| Testkit 组件 | 09号文档用例 |
|---|---|
| `MockSolverPlugin`（`FailOnInit`） | TC-SOLVER-003 |
| `MockSolverPlugin`（`DivergeAtStep`） | TC-SOLVER-005 |
| `MockSolverPlugin`（`FailAtStep`） | TC-VER-006 |
| `InMemoryAtlas` | TC-ATLAS-001, TC-ATLAS-002, TC-ATLAS-004, TC-E2E-002（作为快速前置状态搭建，最终校验仍需真实层，见11号文档 §2） |
| `FixtureBuilder` | TC-CORE-001, TC-SIG-002, TC-ENC-003, TC-VER-004/005 |
| `GoldenDataset` | TC-SIG-004, TC-SIG-005, TC-RET-002, TC-REF-001 |
| 近似相等断言辅助 | 几乎所有含数值比较的用例（TC-SOLVER-001/002/004, TC-BEVY-003/004/005 等），不逐条列出 |
| `MockBevyHarness` | TC-BEVY-006, TC-BEVY-007, TC-BEVY-008, TC-BEVY-009 |

> 说明：本表是 `pre-testkit` 内部维护的需求-用例映射，独立于 04_PRE_Traceability_Matrix.md（后者只追溯 01 号文档的 53 条核心需求，理由见 10号文档 §4）。若某条 09 号文档用例出现在本表中，代表实现阶段建议使用对应 testkit 组件，而非强制约束——具体测试代码是否采用 mock 仍由实现者根据 11号文档 §2 的双层测试策略判断。
