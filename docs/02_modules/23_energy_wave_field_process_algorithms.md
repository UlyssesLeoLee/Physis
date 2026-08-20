# GVPE — Energy / Wave / Field / Process Runtime Algorithms（詳細設計書）

## 0. 文档元数据

| 字段 | 取值 |
|---|---|
| 文档编号 | GVPE-DOC-23 |
| 文档类型 | 詳細設計書 |
| 版本 | v0.2（标准化重写版） |
| 状态 | Draft |
| 适用阶段 | Phase N+（feature-gated，MVP 默认关闭） |
| 关联系统 | GVPE / gvpe-dynamics, gvpe-runtime（特征门控） |
| 上游文档（输入基线） | GVPE-DOC-12（`12_energy_wave_field_design.md` 钩子段）、GVPE-DOC-02（`02_physics_ontology.md` §10–§13） |
| 下游文档（被消费于） | GVPE-DOC-17（`17_detailed_design.md` §4.2 integrate 与 §6 摩擦冲量行）、GVPE-DOC-21（`21_graph_compiler_detailed_design.md` §21.3 受保护写入路径） |
| 编写者 | — |
| 审批者 | — |

## 1. 修订历史

| 版本 | 日期 | 修改者 | 修改内容 |
|---|---|---|---|
| v0.1 | — | — | 初稿（原始版本） |
| v0.2 | 2026-08-19 | 标准化 pass | 转为 IPA 詳細設計書 格式；叙事中文化；保留全部技术内容；补充文档元数据、修订历史、术语定义、关联文档、审批签字等标准章节 |

## 2. 文档目的

本文档为 GVPE 能量账本（Energy Ledger）、波传播（Wave Propagation）、场采样（Field Sampling）、过程状态机（Process State Machine）四大子系统提供可落地的数值算法实现。`12_energy_wave_field_design.md` §12.6 曾刻意止步于「求解器将来附着的接缝」；本文档即在该接缝上挂载具体数值，使 GVPE 从 MVP 最小范围走向物理引擎的完整覆盖。这些能力默认全部关闭（feature-gated），符合 `01_requirements.md` NG1 的范围纪律：「在标志位后设计与实现」≠「默认纳入 MVP 热路径预算」，本文档严格维持这一区分。

## 3. 适用范围

- 适用 crate：`gvpe-dynamics`（integrate 阶段与可选诊断 pass）、`gvpe-runtime`（特征门控开关）。
- 适用阶段：MVP 之后（默认关闭）；启用任一能力时通过 Cargo feature 打开。
- 不适用：流体 SPH / 网格压力投影（见 `24号文書`）、通用 FEM 求解器（同上）、离散波动方程（FDTD 等）的完整 PDE 求解——本设计明确不实现。

## 4. 术语定义

- **能量账本（Energy Ledger）**：在帧末可选地对系统总能量（动能 + 引力势能 + 耗散估计）做聚合的诊断通道。
- **波事件（WaveEvent）**：由碰撞等动作在空间中发射的点源波事件，携带原点、发射时间、振幅、传播速度。
- **场采样器（FieldSampler）**：在空间-时间点 `(at, t)` 处采样某向量场（如重力、爆炸冲击）的 trait 抽象。
- **过程状态机（Process State Machine）**：刻画实体所参与的物理过程（如熔化、凝固）的有限状态机。
- **特征门控（feature-gated）**：通过 Cargo feature 控制某模块是否参与编译，从而保证关闭时零运行时成本。
- **熔化（Melting）过程状态机**：实体在累积热能达到阈值后从固态转为液态的有限状态机。
- **零分配（zero-allocation）**：在稳态运行路径上不进行堆分配，常用于热路径性能约束。

## 5. 模块详细设计

四大子模块彼此独立，皆以独立 Cargo feature 控制编译期参与：

| 子模块 | Cargo feature | 默认 | 与热路径关系 |
|---|---|---|---|
| 能量账本 | `energy-ledger` | 关 | 帧末可选诊断 pass，永不进入求解器循环 |
| 波传播 | `wave-propagation` | 关 | 独立于求解器的事件源模型 |
| 场采样 | `field-sampling` | 关 | `UniformField` 路径启用时零成本；`RadialField` 等场景级 opt-in |
| 过程仿真 | `process-simulation` | 关 | 独立于 `BodyStateSoA` 的边表结构 |

所有子模块的 `Integrate` 钩子（除 `field-sampling` 之外）均位于帧末的诊断通道，不进入 `solve_island` 循环；`field-sampling` 在启用时以「采样而非常数」的形式替换 `RuntimeDescriptor.gravity`，但 `UniformField` 实现保持 MVP 的零行为变更。

## 6. 类与数据结构

### 6.1 `EnergyLedger`

```rust
struct EnergyLedger {
    kinetic: f32,
    gravitational_potential: f32,
    // 其余分量（弹性、阻尼耗散等）按需扩展
}
```

### 6.2 `WaveEvent`

```rust
struct WaveEvent {
    origin: [f32; 3],
    t_emitted: f32,
    kind: WaveKind,
    initial_amplitude: f32,
    propagation_speed: f32,
}
```

### 6.3 `FieldSampler` trait 与实现

```rust
trait FieldSampler {
    fn sample(&self, at: [f32; 3], t: f32) -> [f32; 3];
}

struct UniformField(pub [f32; 3]);                  // MVP 常量重力场景
impl FieldSampler for UniformField {
    fn sample(&self, _: [f32; 3], _: f32) -> [f32; 3] { self.0 }
}

struct RadialField {
    center: [f32; 3],
    strength: f32,
    falloff: FalloffKind,                            // 爆炸 / 涡旋场
}
impl FieldSampler for RadialField {
    fn sample(&self, at: [f32; 3], _: f32) -> [f32; 3] {
        let d = sub(at, self.center);
        let r = length(d).max(f32::EPSILON);
        scale(normalize(d), self.strength * apply_falloff(self.falloff, r))
    }
}
```

### 6.4 过程状态机

```rust
enum ProcessState {
    Idle,
    InProgress { started_at: f32, energy_accumulated: f32 },
    Complete,
}

struct MeltingProcess {
    entity: EntityId,
    energy_required: f32,
    state: ProcessState,
}
```

`MeltingProcess` 以 `EntityId` 为键独立成边表，不在 `BodyStateSoA`（`17号文書` §4.1）上新增字段——本节末以构造方式证明这一点。

## 7. 算法详解

### 7.1 能量账本 — 具体计算

```rust
fn compute_energy_ledger(
    state: &BodyStateSoA,
    profiles: &[PhysicsProfile],
    gravity: [f32; 3],
) -> EnergyLedger {
    let mut ledger = EnergyLedger::default();
    for i in 0..state.position.len() {
        if state.sleeping[i] { continue; }
        let m = 1.0 / state.inv_mass[i].max(f32::EPSILON);
        ledger.kinetic += 0.5 * m * length_sq(state.linear_velocity[i])
            + 0.5 * dot(
                state.angular_velocity[i],
                apply_inertia(&state.inv_inertia[i], state.angular_velocity[i]),
            );
        ledger.gravitational_potential += -m * dot(gravity, state.position[i]);   // 相对 y=0 参考面
    }
    ledger
}
```

该函数作为可选的 `Integrate` 后诊断 pass 调用（`05_runtime_design.md` §5.5 的阶段分解新增一个条件阶段）——永不进入求解器循环本身，故尊重 `GVPE-PROHIBIT-06`（不牺牲实时性能）：特性关闭时函数根本不被调用，零成本。

#### 7.1.1 守恒检验（验证 `02_physics_ontology.md` §10 的转换关系）

```rust
fn energy_conservation_error(
    before: &EnergyLedger,
    after: &EnergyLedger,
    dissipated_estimate: f32,
) -> f32 {
    let total_before = before.total();
    let total_after = after.total() + dissipated_estimate;   // 耗散（摩擦 / 恢复系数<1）须单独核算
    (total_after - total_before).abs() / total_before.max(f32::EPSILON)
}
```

`dissipated_estimate` 在求解器摩擦行（`17_detailed_design.md` §6）累计——摩擦冲量功已可在施加冲量的当时由 `applied * relative_velocity` 实时算得。这是 `02_physics_ontology.md` §25 描述的因果链（`...ElasticEnergy --DISSIPATES_TO--> ThermalEnergy`）的具体实现：从图关系下沉为可检验的数值。

### 7.2 波传播 — 事件源冲量模型（非完整波动方程 PDE 求解）

```rust
fn sample_wave_amplitude(
    events: &[WaveEvent],
    at: [f32; 3],
    t_now: f32,
) -> f32 {
    events.iter().map(|e| {
        let dist = length(sub(at, e.origin));
        let travel_time = dist / e.propagation_speed;
        let arrived_at = e.t_emitted + travel_time;
        if t_now < arrived_at { return 0.0; }
        let age = t_now - arrived_at;
        e.initial_amplitude * attenuation(dist) * decay_envelope(age)   // 简单反平方衰减 + 指数包络
    }).sum()
}
```

`02_physics_ontology.md` §11 的工作样例「`Collision GENERATES MechanicalWave`」在此实现为：每个超过冲量阈值的接触事件发射一个 `WaveEvent`。该实现刻意不做离散波动方程求解（无网格、无 FDTD）——它是分析型点源近似，足以承载面向游戏体验的音频 / 振动提示，明确不足以声学精确仿真。此限制在此明文写定而非日后才发现，正与 `12号文書` §12.6 确立的「不脱离驱动用例臆造数值」一致：完整波动方程求解器因与流体/FEM 同理而不在本设计范围（见 `24号文書`）。

### 7.3 场采样 — 推广既有重力钩子

`17_detailed_design.md` §4.2 的 `integrate` 已经以「采样一个场」的姿态调用 `scale(gravity, 1.0)`——本节兑现该承诺：`RuntimeDescriptor`（`17号文書` §1.3）的 `gravity: [f32; 3]` 演化为 `gravity: Box<dyn FieldSampler>`（MVP 默认 `UniformField`，零行为变更），`integrate` 改为调用 `gravity.sample(state.position[i], t)`。这正是 §12.4「不需要改写既有代码」承诺的具体演示。

### 7.4 过程状态机 — 工作样例（熔化）

```rust
fn tick_melting(
    p: &mut MeltingProcess,
    incoming_thermal_energy: f32,
    t_now: f32,
) {
    match &mut p.state {
        ProcessState::Idle if incoming_thermal_energy > 0.0 =>
            p.state = ProcessState::InProgress {
                started_at: t_now,
                energy_accumulated: incoming_thermal_energy,
            },
        ProcessState::InProgress { energy_accumulated, .. } => {
            *energy_accumulated += incoming_thermal_energy;
            if *energy_accumulated >= p.energy_required {
                p.state = ProcessState::Complete;
                // 发射：Entity UNDERGOES Melting PRODUCES LiquidWater
                // （02号文書 §16 的工作样例）
                // —— 通过 21号文書 §21.3 的受保护写入路径，
                //    作为单条 Process-completion 事件写入 gvpe-graph（非按帧批量，
                //    故 BULK_STATE_WRITE_THRESHOLD 检查平凡通过）
            }
        }
        _ => {}
    }
}
```

`incoming_thermal_energy` 应由未来的热传导 pass 提供（此处不设计——与 §7.2 波动方程同理属于范围外），或由宿主应用直接脚本化以驱动 MVP 邻近用例。`12_energy_wave_field_design.md` §12.5 承诺预留的 `ProcessState` 槽位即本枚举，以每实体为键附加，不触动 `gvpe-dynamics` 的核心 `BodyStateSoA` 布局（`17号文書` §4.1）——本节以构造方式证明：`MeltingProcess` 是独立的边表，不是 `BodyStateSoA` 上的新字段。

## 8. 错误处理

- 能量账本：纯聚合计算，不产生可恢复错误；当 `state.inv_mass` 接近 0（`f32::EPSILON` 下限保护）时质量趋近无穷，函数仍可返回有限结果。
- 波传播：`f32` 距离 / 时间运算中 `t_now < arrived_at` 早退保护非负时间下的传播；`attenuation` / `decay_envelope` 由实现者保证返回值合法。
- 场采样：`RadialField` 用 `length(d).max(f32::EPSILON)` 防止除零；其余路径遵循纯函数约定。
- 过程状态机：状态转移由 `match` 显式穷尽，未匹配分支为 `_ => {}` 显式无操作；非法转移在编译期被 `match` 穷尽性检查捕获。

## 9. 性能考量

- **特征门控（见 §11）**：四个特性皆不在 `default` feature 集合；`01_requirements.md` AC-01 的确定性测试与 `14_performance_budget.md` 的基准均默认关闭状态下运行。
- **零分配**：四个子模块在稳态路径上均不引入 `Box` / `HashMap`；场采样的 `Box<dyn FieldSampler>` 一次性装箱，`UniformField` 零成本。
- **场采样的零成本声明**：`UniformField::sample` 为内联纯函数，启用 `field-sampling` 时与原 `scale(gravity, 1.0)` 路径生成的机器码等价（实现可标注 `#[inline]`）。
- **热路径隔离**：能量账本仅在帧末诊断 pass 调用，永不进入 `solve_island` 循环；过程状态机以独立边表运行，不参与 `BodyStateSoA` 写入。
- **未来扩展**：若启用 `process-simulation` 的实体数进入百万级，需评估 `MeltingProcess` 边表的分桶/分片策略（本文档范围外）。

## 10. 测试考量

- **能量守恒回环测试**：在隔离场景（如双体自由落体、弹簧振子）中运行 N 步，断言 `energy_conservation_error` < 1e-3。
- **波事件时间窗口测试**：构造已知 `WaveEvent`，断言 `sample_wave_amplitude` 在 `t_now < arrived_at` 时返回 0、`t_now > arrived_at` 时随时间单调衰减。
- **场采样等价性测试**：`UniformField([0, -9.81, 0])` 在 `field-sampling` 启用前后的 `integrate` 输出须位一致。
- **熔化状态机状态转移测试**：遍历 `Idle → InProgress → Complete` 三态，断言能量累加与阈值比较正确，非法转移在 `match` 穷尽性下编译失败。
- **特征门控集成测试**：在 `--no-default-features` 构建下，不应引用任何 `energy-ledger` / `wave-propagation` / `field-sampling` / `process-simulation` 的符号。

## 11. 关联需求

- **`02_physics_ontology.md` §10–§13**：schema→runtime 桥接（从 `12号文書` 的接缝到本文档的具体算法）。
- **`00_vision.md` §0.5**：物理引擎完备性。
- **GVPE-PROHIBIT-06**：实时性能不可牺牲——通过 feature-gating 与零分配路径实现。
- **`01_requirements.md` NG1**：范围纪律——四大子模块皆不在 MVP 热路径预算中。
- **`01_requirements.md` AC-01**：确定性测试在所有特性关闭时通过。

## 12. 关联文档

- 上游：`docs/02_modules/12_energy_wave_field_design.md`（接缝与钩子定义）、`docs/02_physics_ontology/02_physics_ontology.md` §10–§13（schema 守恒关系）、§25（DISSIPATES_TO 因果链）。
- 平级：`docs/02_modules/22_vector_detailed_design.md`（向量空间签名提取器，间接消费者）。
- 下游：`docs/02_modules/17_detailed_design.md` §4.2（integrate 调用现场）、§6（摩擦行耗散累计）、`docs/02_modules/21_graph_compiler_detailed_design.md` §21.3（受保护写入路径）。

## 13. 审批签字

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 编写 | | | |
| 校对 | | | |
| 审批 | | | |

---

## 14. 正文

> 以下为原文档正文，章节编号保留（§23.1, §23.1.1, §23.2, §23.3, §23.4, §23.5），叙事已并入上方对应章节，本节保留原始技术片段与原文档的引用脉络。

### 23.1 能量账本 — 具体计算

参见上方 §7.1 与 §7.1.1。原始代码片段完整保留。

### 23.2 波传播 — 事件源冲量模型（非完整波动方程 PDE 求解）

参见上方 §7.2。原始代码片段完整保留。

### 23.3 场采样 — 推广既有重力钩子

参见上方 §7.3。

### 23.4 过程状态机 — 工作样例（熔化）

参见上方 §7.4。原始代码片段完整保留。

### 23.5 特征门控（保持 `GVPE-PROHIBIT-06` 完整）

```toml
[features]
energy-ledger = []
wave-propagation = []
field-sampling = []   # UniformField 路径即使启用亦零成本；RadialField 等按场景 opt-in
process-simulation = []
```

以上皆不在 `default` 特征集合——`01_requirements.md` AC-01 的确定性测试与 `14_performance_budget.md` 的基准均默认关闭状态下运行，故即便后续发现其数值需修订，也不致回退 MVP 性能/确定性基线。
