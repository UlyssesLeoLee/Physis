# GVPE — XPBD / Rope / Cloth / SoftBody / Particle Detailed Design（詳細設計書）

## 0. 文档元数据

| 字段 | 取值 |
|---|---|
| 文档编号 | GVPE-DOC-19 |
| 文档类型 | 詳細設計書 |
| 版本 | v0.2（标准化重写版） |
| 状态 | Draft |
| 适用阶段 | MVP（设计就绪，运行时不在 MVP 范围） |
| 关联系统 | GVPE / gvpe-dynamics, gvpe-constraint, gvpe-solver, gvpe-collision, gvpe-shape |
| 上游文档（输入基线） | `07_solver_design.md` §7.2, `02_physics_ontology.md` §4, `17_detailed_design.md` §3/§4/§5/§6, `01_requirements.md` NG1, `06_collision_design.md` §6.2, `15_testing_strategy.md`, `12_energy_wave_field_design.md` §12.6 |
| 下游文档（被消费于） | `10_ffi_design.md` |
| 编写者 | — |
| 审批者 | — |

## 1. 修订历史

| 版本 | 日期 | 修改者 | 修改内容 |
|---|---|---|---|
| v0.1 | — | — | 初稿（原始版本） |
| v0.2 | 2026-08-19 | 标准化 pass | 转为 IPA 詳細設計書 格式；叙事中文化；保留全部技术内容；补充文档元数据、修订历史、术语定义、关联文档、审批签字等标准章节 |

## 2. 文档目的

MVP 不交付 XPBD 求解器（`01_requirements.md` NG1），但「作为物理引擎完备」要求 *设计* 存在且 *可证明* 与 Gen 1 数据模型兼容——而不仅是顺带承诺。本文给出：

- XPBD 在 GVPE 数据模型中的位置：它是 `ConstraintRow.compliance` 的非零取值
- 粒子表示（`ParticleStateSoA`）与刚体 SoA 分离的合理性
- Rope / Cloth / SoftBody / Granular 等 `MechanicalBehavior` 种类到具体约束行类型的映射
- 软体碰撞复用窄阶段的方式
- 明确不在本文范围内的事项（流体、FEM）

## 3. 适用范围

- 适用于 XPBD Generation 2 求解器的详细设计，含粒子表示、约束种类、求解循环、碰撞复用、测试夹具
- 不适用于流体（SPH / 网格法）——见 §19.6 显式说明
- 不适用于完整 FEM
- 运行时不在 MVP 范围（`01_requirements.md` NG1），但设计与 Gen 1 数据模型必须兼容

## 4. 术语定义

| 术语 | 定义 |
|---|---|
| XPBD（Extended Position Based Dynamics） | 位置式动力学方法，通过子步迭代求解位置约束 |
| PBD（Position Based Dynamics） | 位置式动力学基础方法（无 compliance 概念） |
| 粒子（Particle） | 无旋转自由度的质点，XPBD 求解对象 |
| 软体（SoftBody） | 由粒子网格与约束集合表达的可形变体 |
| 绳索（Rope） | 一维链状粒子集合，由 `Distance` 约束连接 |
| 布料（Cloth） | 二维网格粒子集合，含结构/剪切/弯曲约束 |
| 颗粒（Granular） | 仅含短程排斥的粒子集合，无持久结构约束 |
| Verlet 风格积分 | 通过位置差分推算速度（`v = (p - prev_p) / h`） |
| 子步（Substep） | XPBD 将 dt 拆分的更小时间步 |
| 广义逆质量（Generalized Inverse Mass） | XPBD 约束求解中粒子逆质量的等效表达 |
| 约束值（Constraint Value, C） | 当前约束违反量（如 `|p_a - p_b| - rest_length`） |
| Alpha tilde | XPBD 引入的 `α̃ = α/h²` 形式的 compliance 时间步归一化项 |
| 接触流形（Contact Manifold） | 接触点集合（见 `17_detailed_design.md` §5.2） |
| 体积保持（Volume Preservation） | 软体维持参考体积的约束 |
| 弯曲阻力（Bending Resistance） | 抵抗网格弯曲形变的约束 |
| 距离约束（Distance） | 维持两个粒子之间距离的约束 |
| 附着（Attachment） | 将粒子固定到世界空间某点的约束 |

## 5. 模块详细设计

### 5.1 粒子表示模块

`ParticleStateSoA` 与 `BodyStateSoA` 分离，见正文 §19.2。

### 5.2 XPBD 求解循环模块

`xpbd_step`：预测位置 → 迭代约束行 → 推导速度。见正文 §19.3。

### 5.3 约束种类模块

`XpbdConstraintKind` 枚举（Distance / Bending / Volume / Attachment），与各种 `MechanicalBehavior` 的映射见正文 §19.4。

### 5.4 软体碰撞模块

粒子-刚体、粒子-粒子接触复用 `gvpe-collision` 的窄阶段，见正文 §19.5。

### 5.5 显式排除项

流体与 FEM 不在本文范围，见正文 §19.6。

## 6. 类与数据结构

### 6.1 粒子 SoA

```rust
struct ParticleStateSoA {
    position: Vec<[f32; 3]>, prev_position: Vec<[f32; 3]>,   // XPBD needs both (Verlet-style)
    inv_mass: Vec<f32>,
    velocity: Vec<[f32; 3]>,   // derived post-solve, not integrated directly (XPBD convention)
}
```

粒子是与 `BodyStateSoA`（`17号文書` §4.1）不同的 SoA 存储——刚体与粒子是不同的 `MechanicalBehavior` 种类（`02_physics_ontology.md` §4），积分方案也不同，分开存储可避免 Gen 1 热路径被 Gen 1 永不使用的字段污染。

### 6.2 约束种类

```rust
enum XpbdConstraintKind {
    Distance { rest_length: f32 },                       // 绳索段、布料结构约束
    Bending  { rest_angle: f32 },                          // 布料弯曲阻力
    Volume   { rest_volume: f32 },                          // 软体体积保持
    Attachment { anchor_world: [f32; 3] },                   // 将粒子固定到固定/运动学点
}
```

## 7. 算法详解

### 7.1 XPBD 求解循环

```rust
fn xpbd_step(particles: &mut ParticleStateSoA, rows: &mut [ConstraintRow], substeps: u32, dt: f32) {
    let h = dt / substeps as f32;
    for _ in 0..substeps {
        predict_positions(particles, h);              // p += v*h + external_accel*h^2 (重力通过同一 Field-sample 钩子, 17号文書 §4.2)
        for row in rows.iter_mut() {
            let c = constraint_value(row, particles);   // 例：距离约束为 |p_a - p_b| - rest_length
            let alpha_tilde = row.compliance / (h * h);
            let delta_lambda = -(c + alpha_tilde * row.lambda) / (generalized_inv_mass(row, particles) + alpha_tilde);
            row.lambda += delta_lambda;
            apply_position_correction(particles, row, delta_lambda);   // 直接移动 p_a/p_b, 而非速度
        }
        update_velocities(particles, h);   // v = (p - prev_p) / h, 然后 prev_p = p
    }
}
```

此函数在结构上刻意与 `17号文書` §6 的 `solve_island` 并行——相同的 predict → 迭代行 → apply 模式，未知数（位置 vs 速度）与每种约束的 `generalized_inv_mass` / `constraint_value` 不同，这正是 `ConstraintRow` 已抽象的差异点。

### 7.2 约束种类到 `MechanicalBehavior` 的映射

- **绳索**：连续粒子间的 `Distance` 链 + 固定端的一个 `Attachment`（若有）。
- **布料**：粒子网格，沿结构（横/纵）与剪切（对角）边的 `Distance` 约束，每个内部边对的 `Bending` 约束——`02_physics_ontology.md` §4 中 `Cloth` / `Membrane` 的网格拓扑相同，区分仅在弯曲阻力的刚度（`compliance`）取值；约束 *种类* 相同。
- **软体**：四面体网格，tet 边上的 `Distance` 约束 + 每个四面体的 `Volume` 约束（`04号文書` §4.5 的 Law→Model→Solver 表中 `PBDModel` / `XPBDModel` 行在此获得具体 Solver 条目）。
- **颗粒**：仅有 `Distance` 风格的短程排斥（无持久结构约束）——颗粒材料最接近 XPBD「约束」的概念是瞬态接触约束，复用 §19.4 的接触处理路径，不需第五种。

## 8. 错误处理

- 子步 `lambda` 发散：置对应粒子集为静态，下一帧重新启用（保守侧）
- 约束违反超出容差：标记为退化约束（`Volume` 退化为 `Distance`、`Distance` 退化为 `Attachment`），不静默接受
- 软体碰撞未找到接触对：粒子视为自由运动，不视为错误
- `predict_positions` 中 NaN/Inf：将该粒子置为 `inv_mass = 0`（不可移动）并继续，避免崩溃
- 与 Gen 1 一致：`01_requirements.md` NG1 范围内不交付运行时 XPBD，设计错误通过 `CompileError`/`InitError` 路径报告

## 9. 性能考量

- 粒子与刚体 SoA 分离：避免热路径字段污染
- 子步合并：`substeps` 可由 `compliance` 推导（合规性越高 → 需更多子步），自适应而非固定
- 约束求解：与 Gen 1 共享 SI 迭代结构，相同的 cache-friendly 顺序遍历
- 软体碰撞：复用窄阶段，避免新增算法
- 零分配：`ParticleStateSoA` 的所有缓冲在初始化时分配，运行期仅 `reset`
- 性能预算：见 `14_performance_budget.md`

## 10. 测试考量

- 绳索收敛测试：N 段绳索在重力下收敛到预期悬链线形态
- 布料稳定性测试：网格布料在两角固定，无爆炸（粒子速度有界）经固定步数
- 软体体积保持：四面体网格在形变后恢复参考体积
- 颗粒短程排斥：堆叠颗粒不穿透
- 与 Gen 1 兼容性：Gen 1 的 `compliance = 0` 在 XPBD 代码路径上退化为 SI 行为
- 详细测试夹具见正文 §19.7

## 11. 关联需求

| 需求 ID | 中文描述 | 满足位置 |
|---|---|---|
| GVPE-FR-002 | 刚体动力学求解（本文扩展至软体） | §19.2–§19.5 |
| `01_requirements.md` NG1 | MVP 不交付 XPBD 运行时（仅设计就绪） | 全文 |
| `02_physics_ontology.md` §4 | MechanicalBehavior 分类：Rope/Cloth/Membrane/Rod/Shell/SoftBody/GranularBehavior | §19.4 |
| `02_physics_ontology.md` §15 | Model 行在运行时落地 | §19.4.3 |
| `04号文書` §4.5 | Law→Model→Solver 追溯表，XPBDModel 行具体化 | §19.4.3 |
| `06_collision_design.md` §6.2 | 形状描述扩展 Particle 变体 | §19.5 |
| `07号文書` §7.2 | Generation 2 求解器预留 | §19.3 |
| `12号文書` §12.6 | 显式排除范围纪律 | §19.6 |
| `15号文書` §15.6 | 测试夹具基于真实求解器 | §19.7 |

## 12. 关联文档

- 上游：`07_solver_design.md` §7.2（Gen 2 预留）、`02_physics_ontology.md` §4（MechanicalBehavior）、`17_detailed_design.md` §3/§4/§5/§6（碰撞、动力学、约束、求解）、`06_collision_design.md` §6.2（形状扩展）、`15_testing_strategy.md`（测试纪律）、`12_energy_wave_field_design.md` §12.6（显式排除纪律）
- 下游：`10_ffi_design.md`（若 FFI 暴露 XPBD API）
- 平行：`18_joints_ccd_design.md`（共享 ConstraintRow 抽象）、`20_shape_advanced_design.md`（共享 ShapeDesc 扩展）

## 13. 审批签字

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 编写 | — | — | — |
| 校对 | — | — | — |
| 审批 | — | — | — |

---

## 14. 正文

> Input baseline: `07_solver_design.md` §7.2 (Generation 2, reserved), `02_physics_ontology.md` §4
> (MechanicalBehavior: Rope/Cloth/Membrane/Rod/Shell/SoftBody/GranularBehavior). MVP does not ship
> this solver (`01_requirements.md` NG1), but "complete as a physics engine" requires the *design* to
> exist and to be provably compatible with the Generation-1 data model, not just promised in passing.

## 19.1 为什么是 XPBD，以及为何不需要新架构

`ConstraintRow.compliance`（`17_detailed_design.md` §5.1）已承载与 XPBD 兼容的 compliance 参数——Gen 1（`gvpe-solver` §6）将其置为 `0.0`（刚性），Gen 2 在同一行 `delta_lambda /= 1.0 + row.compliance`（`17号文書` §6 已通用化地写好）中使用非零值。本节设计 XPBD 约束所作用的 *粒子* 表示；*求解循环* 相对 `17号文書` §6 保持不变。

## 19.2 粒子表示（新的 body 种类，而非新求解器）

```rust
struct ParticleStateSoA {
    position: Vec<[f32; 3]>, prev_position: Vec<[f32; 3]>,   // XPBD needs both (Verlet-style)
    inv_mass: Vec<f32>,
    velocity: Vec<[f32; 3]>,   // derived post-solve, not integrated directly (XPBD convention)
}
```

粒子是与 `BodyStateSoA`（`17号文書` §4.1）不同的 SoA 存储——刚体与粒子是不同的 `MechanicalBehavior` 种类（`02_physics_ontology.md` §4），积分方案也不同，分开存储可避免 Gen 1 热路径被 Gen 1 永不使用的字段污染。

## 19.3 XPBD 求解循环

```rust
fn xpbd_step(particles: &mut ParticleStateSoA, rows: &mut [ConstraintRow], substeps: u32, dt: f32) {
    let h = dt / substeps as f32;
    for _ in 0..substeps {
        predict_positions(particles, h);              // p += v*h + external_accel*h^2 (重力通过同一 Field-sample 钩子, 17号文書 §4.2)
        for row in rows.iter_mut() {
            let c = constraint_value(row, particles);   // 例：距离约束为 |p_a - p_b| - rest_length
            let alpha_tilde = row.compliance / (h * h);
            let delta_lambda = -(c + alpha_tilde * row.lambda) / (generalized_inv_mass(row, particles) + alpha_tilde);
            row.lambda += delta_lambda;
            apply_position_correction(particles, row, delta_lambda);   // 直接移动 p_a/p_b, 而非速度
        }
        update_velocities(particles, h);   // v = (p - prev_p) / h, 然后 prev_p = p
    }
}
```

此函数在结构上刻意与 `17号文書` §6 的 `solve_island` 并行——相同的 predict → 迭代行 → apply 模式，未知数（位置 vs 速度）与每种约束的 `generalized_inv_mass` / `constraint_value` 不同，这正是 `ConstraintRow` 已抽象的差异点。

## 19.4 约束种类（将 `02_physics_ontology.md` §9 的语义列表扩展为 XPBD 行）

```rust
enum XpbdConstraintKind {
    Distance { rest_length: f32 },                       // 绳索段、布料结构约束
    Bending  { rest_angle: f32 },                          // 布料弯曲阻力
    Volume   { rest_volume: f32 },                          // 软体体积保持
    Attachment { anchor_world: [f32; 3] },                   // 将粒子固定到固定/运动学点
}
```

### 19.4.1 绳索：连续粒子间的 `Distance` 链 + 固定端的一个 `Attachment`（若有）。

### 19.4.2 布料：粒子网格，沿结构（横/纵）与剪切（对角）边的 `Distance` 约束，每个内部边对的 `Bending` 约束——`02_physics_ontology.md` §4 中 `Cloth` / `Membrane` 的网格拓扑相同，区分仅在弯曲阻力的刚度（`compliance`）取值；约束 *种类* 相同。

### 19.4.3 软体：四面体网格，tet 边上的 `Distance` 约束 + 每个四面体的 `Volume` 约束（`04号文書` §4.5 的 Law→Model→Solver 表中 `PBDModel` / `XPBDModel` 行在此获得具体 Solver 条目）。

### 19.4.4 颗粒：仅有 `Distance` 风格的短程排斥（无持久结构约束）——颗粒材料最接近 XPBD「约束」的概念是瞬态接触约束，复用 §19.4 的接触处理路径，不需第五种。

## 19.5 软体碰撞

粒子对刚体、粒子对粒子的接触复用 `gvpe-collision` 的窄阶段（`17号文書` §3），将粒子视为零半径（或颗粒时小半径）球——无需新碰撞算法，仅新增 `ShapeDesc` 变体（`Particle { radius: f32 }`，`06_collision_design.md` §6.2 扩展）。

## 19.6 明确不在范围内

流体（SPH 或网格法）与完整 FEM **不**在本文档设计范围——`01_requirements.md` NG1 将二者排除在 §19 给 Rope/Cloth/SoftBody 的预留接口之外，原因是与 XPBD 族固体不同，流体求解器不是现有 `ConstraintRow` / 粒子抽象的数据模型扩展；它需要独立的数值方法（压力投影或 SPH 核函数评估），无法复用 §19.3 的任何内容。这属于未来文档的工作，待有驱动用例时再写，不在此推测（与 `12_energy_wave_field_design.md` §12.6 已用于 Energy/Wave/Field 数值方法的纪律相同）。

## 19.7 欠 `15_testing_strategy.md` 的测试夹具

- N 段绳索在重力下落到预期悬链线静止形态，容差内收敛。
- 布料网格固定两角，固定步数内无爆炸（粒子速度有界）——XPBD 等价于 `18_joints_ccd_design.md` §18.6 的刚体夹具纪律。

Requirements satisfied: `01_requirements.md` GVPE-FR-002 (extended), `02_physics_ontology.md` §4/§15
(MechanicalBehavior/Model rows now have a concrete runtime path), `04_architecture.md` §4.5 (the
`XPBDModel` table row is no longer "reserved, Phase 6+" without a design behind it).
