# GVPE — Joints & CCD Detailed Design（ジョイント・CCD 詳細設計書）

## 0. 文档元数据

| 字段 | 取值 |
|---|---|
| 文档编号 | GVPE-DOC-18 |
| 文档类型 | 詳細設計書 |
| 版本 | v0.2（标准化重写版） |
| 状态 | Draft |
| 适用阶段 | MVP |
| 关联系统 | GVPE / gvpe-constraint, gvpe-solver, gvpe-runtime, gvpe-collision |
| 上游文档（输入基线） | `07_solver_design.md` §7.5, `17_detailed_design.md` §5–§6, `05_runtime_design.md` §5.4, `02_physics_ontology.md` §9, `14_performance_budget.md`, `15_testing_strategy.md` |
| 下游文档（被消费于） | `19_softbody_xpbd_design.md`, `20_shape_advanced_design.md`, `10_ffi_design.md` |
| 编写者 | — |
| 审批者 | — |

## 1. 修订历史

| 版本 | 日期 | 修改者 | 修改内容 |
|---|---|---|---|
| v0.1 | — | — | 初稿（原始版本） |
| v0.2 | 2026-08-19 | 标准化 pass | 转为 IPA 詳細設計書 格式；叙事中文化；保留全部技术内容；补充文档元数据、修订历史、术语定义、关联文档、审批签字等标准章节 |

## 2. 文档目的

本文档关闭 `17_detailed_design.md` 留下的空白：没有关节与 CCD 的物理引擎不算功能完备，而 `07号文書` §7.5 已将二者标记为"首版不构成功能承诺"——本文档将该承诺转化为具体设计，使其不再缺席。覆盖范围：

- 关节（Joint）的类型分解与求解集成
- CCD（连续碰撞检测）的触发条件、算法与执行图位置
- 与现有 Sequential Impulse 求解器的兼容性证明

## 3. 适用范围

- 适用于 MVP 阶段引入关节约束（Fixed、Distance、Hinge、Slider）与 CCD（保守推进法）所需的详细设计。
- 不适用于流体（SPH/网格法）与完整 FEM——见 `19_softbody_xpbd_design.md` §19.6 的明确说明。
- 不适用于 GPU 端关节/CCD 加速（属于后续 Phase）。

## 4. 术语定义

| 术语 | 定义 |
|---|---|
| 关节（Joint） | 限制两个刚体相对运动的约束类型，如固定、距离、铰链、滑块 |
| CCD（Continuous Collision Detection） | 连续碰撞检测：通过扫描体积防止快速物体在单帧内穿透薄障碍物 |
| 保守推进（Conservative Advancement） | CCD 的核心算法，通过逐步逼近求出最小碰撞时间 |
| TOI（Time of Impact） | 首次接触时间 |
| 顺序冲量（Sequential Impulse, SI） | 关节行仍由同一 SI 求解器迭代处理 |
| 库仑摩擦锥 | 摩擦力的法向/切向约束锥（关节限位复用此边界机制） |
| 接触流形（Contact Manifold） | 接触点的集合（见 `17_detailed_design.md` §5.2） |
| 关节行（Joint Row） | 关节分解后的 `ConstraintRow` 单元 |
| 库仑摩擦锥的矩形近似 | 将锥约束近似为 `upper = μ * normal_lambda` 的矩形 |
| 渐进式推进 | 通过逐步时间步逼近避免穿透的求解策略 |
| 顺序冲量回环 | 关节行与接触行混合在同一 SI 循环中迭代 |

## 5. 模块详细设计

### 5.1 关节类型模块

将 `02_physics_ontology.md` §9 的约束分类扩展为运行时 Joint Row。具体实现见正文 §18.1–§18.2。

### 5.2 关节生命周期模块

关节的创建/销毁通过 `gvpe-runtime` 的 API（`05_runtime_design.md` §5.4）执行，产出与 body 同样具备世代索引安全性的 `ConstraintHandle`（`17号文書` §1.1）。引用已被销毁 body 的关节会被自动失效（通过 `generation` 不匹配检测），不留下悬空引用。见正文 §18.3。

### 5.3 求解器集成模块

关节行与接触行在同一 SI 循环中作为扁平切片处理；求解器不感知关节类型。见正文 §18.4。

### 5.4 CCD 模块

CCD 包含触发条件、保守推进算法与执行图位置三部分。见正文 §18.5.1–§18.5.3。

## 6. 类与数据结构

### 6.1 JointRow 枚举

```rust
enum JointRow {
    Fixed   { anchor_a: [f32; 3], anchor_b: [f32; 3], anchor_rows: [ConstraintRow; 3], angular_rows: [ConstraintRow; 3] },
    Distance{ anchor_a: [f32; 3], anchor_b: [f32; 3], rest_length: f32, row: ConstraintRow },
    Hinge   { anchor_a: [f32; 3], anchor_b: [f32; 3], axis_a: [f32; 3], axis_b: [f32; 3],
              point_rows: [ConstraintRow; 3], perp_axis_rows: [ConstraintRow; 2],
              limit_row: Option<ConstraintRow> },
    Slider  { anchor_a: [f32; 3], anchor_b: [f32; 3], axis: [f32; 3],
              perp_rows: [ConstraintRow; 2], limit_row: Option<ConstraintRow> },
}
```

每个关节都分解为一个或多个 `ConstraintRow`（`17_detailed_design.md` §5.1）——§18.4 的求解器从不针对关节 *类型* 特殊化处理，仅迭代行。这复用了 `gvpe-solver` 现有的 Sequential Impulse 循环，无需修改（`17号文書` §6），是「添加关节无需新求解器」的具体证据。

### 6.2 关节行构建器

- `build_hinge_rows`：构建铰链关节的全部 `ConstraintRow`（点约束 + 垂直轴约束 + 可选限位）
- `build_fixed_rows`、`build_distance_rows`、`build_slider_rows`：其他关节类型的行构建函数
- 限位行（limit row）：复用接触摩擦行的 `lower`/`upper` 钳制机制（`17号文書` §6），不引入新机制——一个钳制冲量抽象同时服务两类场景

## 7. 算法详解

### 7.1 铰链关节行构建

```rust
fn build_hinge_rows(hinge: &HingeSpec, state: &BodyStateSoA) -> JointRow {
    // 点约束：anchor_a（在 body A 局部系）必须与 anchor_b（在 body B 局部系）重合
    //   → 3 个 ConstraintRow，雅可比由 anchor 偏移叉积推导（与 rest_length=0 的三轴 Distance 约束结构相同）
    // 垂直轴约束：hinge axis_a 与 axis_b 必须保持平行
    //   → 2 个 ConstraintRow（绕垂直于 hinge 轴的两轴的旋转被锁定）
    // 可选限位：绕 hinge 轴的角度被钳制到 [min, max]
    //   → 1 个 ConstraintRow，其 lower/upper 由当前角度与限位决定，bias 仅在角度接近限位时激活
    //     （限位带外的非活动约束）
    JointRow::Hinge { /* ... constructed from the above ... */ }
}
```

限位行使用与接触摩擦行相同的 `lower`/`upper` 钳制机制（`17号文書` §6），而非新机制——一个钳制冲量抽象同时服务接触摩擦与关节限位。

### 7.2 求解器集成

```rust
fn island_constraint_rows(island: &Island, contacts: &[ConstraintRow], joints: &[ConstraintRow]) -> Vec<ConstraintRow> {
    // 关节行与接触行在 solve_island() 之前拼接为一个扁平切片
    // （17号文書 §6）——求解器不感知关节, 仅感知行
    let mut rows = contacts.to_vec();
    rows.extend_from_slice(joints);
    rows
}
```

这是 `17_detailed_design.md` §5.1 的 `ConstraintRowKind` 枚举预先保留位置（`/* Joint 系は post-MVP */`）的原因——本文档仅填入该预留位置，不改变枚举的形状，只是新增变体。

### 7.3 CCD 触发条件

```rust
fn needs_ccd(body: &BodyStateSoA, i: usize, dt: f32, shape_radius: f32) -> bool {
    let travel = length(scale(body.linear_velocity[i], dt));
    travel > shape_radius * CCD_TRAVEL_RATIO   // 例：body 自身半径以上的位移
}
```

仅超过阈值的 body 才承担 CCD 成本——大多数 body 在大多数帧中完全跳过，保持普通情况的低成本（与 `14_performance_budget.md`「测量，不要假设」的纪律一致：`CCD_TRAVEL_RATIO` 是待根据真实基准数据校准的可调常数，不在这里手拍）。

### 7.4 CCD 核心算法（保守推进）

```rust
fn ccd_resolve(body: &mut BodyRecord, others: &[BodyRecord], dt: f32) -> f32 {
    // 1. 使用扫描体计算保守的首次接触时间（TOI）下界
    //    （MVP 形状的球扫描：即便 box/sphere 也将 body 运动视为胶囊扫描，
    //     使用其包围球——一种标准的保守近似）
    // 2. 将 body 推进到恰好 TOI 之前，在该子位置重跑窄阶段
    // 3. 若发现真实接触，生成到本帧的流形列表（喂入 gvpe-constraint §5.2, 17号文書），
    //    替代穿透后的位置
    // 4. 若无真实接触（保守下界的假阳性），回退为完整 dt
    conservative_advancement_loop(body, others, dt, MAX_CCD_ITERATIONS)
}
```

CCD 的输出是修正后的位置/TOI，回馈到同一积分步骤（`17_detailed_design.md` §4.2）——它不绕过约束求解器，而是在求解器看到（已经穿透的）状态前阻止穿透。

### 7.5 CCD 在执行图中的位置

CCD 在 `05_runtime_design.md` §5.5 步骤拆解（`... → Integrate → CCD → Output`）中的阶段在主 integrate 之后执行，仅作用于 §18.5.1 标记的 body 子集——保持 Execution Graph 形状（`03_graph_schema.md` §1.C）不变，CCD 是现有阶段内的条件性 fan-out，而非新阶段类型。

## 8. 错误处理

- 关节创建失败（如参数非法）：通过 `gvpe-runtime` 的 API 返回 `InitError` 子类型
- 关节引用已销毁 body：通过 `generation` 不匹配检测为悬空，自动失效并返回错误
- CCD 保守下界假阳性：通过 §18.5.2 第 4 步的"完整 dt 回退"安全降级，不视为错误
- CCD 反迭代超限：返回 `MAX_CCD_ITERATIONS` 上限错误，将 body 退回非 CCD 路径（保守侧）
- 求解器发散：与 `17_detailed_design.md` §13 一致，置物理岛为 Sleeping

## 9. 性能考量

- 关节行零额外分配：所有 `ConstraintRow` 都在 `FrameScratch`（`gvpe-memory` §2.1）内分配
- 求解器开销：关节与接触在同一 SI 循环内迭代，无额外循环层级
- 限位行：仅在接近限位时激活（`bias` 在限位带外为 0），常态下退化为非活动约束
- CCD 选择性触发：仅超过位移阈值的 body 承担扫描成本，普通情况几乎无开销
- CCD 扫描体：对 MVP 形状使用球扫描（box/sphere 也按包围球处理），降低计算复杂度
- 性能预算：见 `14_performance_budget.md`

## 10. 测试考量

- 关节回归测试：固定关节、铰链关节在重力下的稳定行为
- 关节极限测试：铰链角度超限时正确钳制
- CCD 隧道回归：高速球穿透薄板的 case 启用 CCD 必须不穿透
- CCD 选择性：低速/小位移 body 不应触发 CCD
- 关节与求解器集成：关节行与接触行混合在同一 SI 循环中收敛
- 关节限位复用钳制机制：与接触摩擦行共享同一钳制抽象，验证一致性
- 详细测试夹具见正文 §18.6

## 11. 关联需求

| 需求 ID | 中文描述 | 满足位置 |
|---|---|---|
| GVPE-FR-002 | 刚体动力学求解（含关节与 CCD，本文显式扩展） | §18.1–§18.5 |
| `00_vision.md` §0.5 | 物理引擎完备性需包含关节与 CCD | 全文 |
| `02_physics_ontology.md` §9 | 约束分类的运行时行实现 | §18.1 |
| `07号文書` §7.1/§7.3 | 约束行与摩擦机制被关节行复用 | §18.4 |
| `05号文書` §5.4 | 关节生命周期由 gvpe-runtime 管理 | §18.3 |
| `05号文書` §5.5 | Execution Graph 中 CCD 阶段位置 | §18.5.3 |
| `14_performance_budget.md` | 测量而非假设原则 | §18.5.1（CCD_TRAVEL_RATIO 可调） |
| `15号文書` §15.6 | 测试夹具基于真实求解器运行 | §18.6 |

## 12. 关联文档

- 上游：`07_solver_design.md` §7.5（首版不构成承诺）、`17_detailed_design.md` §5–§6（SI 与 ConstraintRow）、`05_runtime_design.md` §5.4/§5.5（API 与执行图）、`02_physics_ontology.md` §9（约束分类）、`14_performance_budget.md`、`15_testing_strategy.md`、`03_graph_schema.md` §1.C
- 下游：`19_softbody_xpbd_design.md`（共享 ConstraintRow 抽象）、`20_shape_advanced_design.md`（共享 dispatch 表设计原则）、`10_ffi_design.md`（关节与 CCD 经 FFI 暴露）
- 平行：`17_detailed_design.md`（求解器本体的 SI 详细算法）

## 13. 审批签字

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 编写 | — | — | — |
| 校对 | — | — | — |
| 审批 | — | — | — |

---

## 14. 正文

> Input baseline: `07_solver_design.md` §7.5 (explicitly deferred), `17_detailed_design.md` §5–§6.
> This closes the gap `17_detailed_design.md` left open: a physics engine without joints and CCD is
> not feature-complete, and `07号文書` §7.5 flagged both as "not a feature commitment" for the first
> pass — this document turns that commitment into a concrete design so it is no longer missing.

## 18.1 关节类型（将 `02_physics_ontology.md` §9 的约束分类扩展为运行时行）

```rust
enum JointRow {
    Fixed   { anchor_a: [f32; 3], anchor_b: [f32; 3], anchor_rows: [ConstraintRow; 3], angular_rows: [ConstraintRow; 3] },
    Distance{ anchor_a: [f32; 3], anchor_b: [f32; 3], rest_length: f32, row: ConstraintRow },
    Hinge   { anchor_a: [f32; 3], anchor_b: [f32; 3], axis_a: [f32; 3], axis_b: [f32; 3],
              point_rows: [ConstraintRow; 3], perp_axis_rows: [ConstraintRow; 2],
              limit_row: Option<ConstraintRow> },
    Slider  { anchor_a: [f32; 3], anchor_b: [f32; 3], axis: [f32; 3],
              perp_rows: [ConstraintRow; 2], limit_row: Option<ConstraintRow> },
}
```

每个关节都分解为一个或多个 `ConstraintRow`（`17_detailed_design.md` §5.1）——§18.4 的求解器从不针对关节 *类型* 特殊化处理，仅迭代行。这复用了 `gvpe-solver` 现有的 Sequential Impulse 循环而不修改（`17号文書` §6），是「添加关节无需新求解器」的具体证据。

## 18.2 构建关节行

```rust
fn build_hinge_rows(hinge: &HingeSpec, state: &BodyStateSoA) -> JointRow {
    // 点约束：anchor_a（在 body A 局部系）必须与 anchor_b（在 body B 局部系）重合
    //   → 3 个 ConstraintRow，雅可比由 anchor 偏移叉积推导（与 rest_length=0 的三轴 Distance 约束结构相同）
    // 垂直轴约束：hinge axis_a 与 axis_b 必须保持平行
    //   → 2 个 ConstraintRow（绕垂直于 hinge 轴的两轴的旋转被锁定）
    // 可选限位：绕 hinge 轴的角度被钳制到 [min, max]
    //   → 1 个 ConstraintRow，其 lower/upper 由当前角度与限位决定，bias 仅在角度接近限位时激活
    //     （限位带外的非活动约束）
    JointRow::Hinge { /* ... constructed from the above ... */ }
}
```

限位行使用与接触摩擦行相同的 `lower`/`upper` 钳制机制（`17号文書` §6），而非新机制——一个钳制冲量抽象同时服务接触摩擦与关节限位。

## 18.3 关节生命周期

关节的创建/销毁通过 `gvpe-runtime` 的 API（`05_runtime_design.md` §5.4）执行，产出与 body 同样具备世代索引安全性的 `ConstraintHandle`（`17号文書` §1.1）。引用已被销毁 body 的关节会被自动失效（通过 `generation` 不匹配检测），不留下悬空引用。

## 18.4 求解器集成

```rust
fn island_constraint_rows(island: &Island, contacts: &[ConstraintRow], joints: &[ConstraintRow]) -> Vec<ConstraintRow> {
    // 关节行与接触行在 solve_island() 之前拼接为一个扁平切片
    // （17号文書 §6）——求解器不感知关节, 仅感知行
    let mut rows = contacts.to_vec();
    rows.extend_from_slice(joints);
    rows
}
```

这是 `17_detailed_design.md` §5.1 的 `ConstraintRowKind` 枚举预先保留位置（`/* Joint 系は post-MVP */`）的原因——本文档仅填入该预留位置，不改变枚举的形状，只是新增变体。

## 18.5 CCD（连续碰撞检测）

MVP 显式不实现 CCD（`07号文書` §7.5）；本节设计 CCD，使一个声称完备的物理引擎拥有真正的 CCD 算法，而非一个什么都不做的空阶段。

### 18.5.1 触发条件

```rust
fn needs_ccd(body: &BodyStateSoA, i: usize, dt: f32, shape_radius: f32) -> bool {
    let travel = length(scale(body.linear_velocity[i], dt));
    travel > shape_radius * CCD_TRAVEL_RATIO   // 例：body 自身半径以上的位移
}
```

仅超过阈值的 body 才承担 CCD 成本——大多数 body 在大多数帧中完全跳过，保持普通情况的低成本（与 `14_performance_budget.md`「测量，不要假设」的纪律一致：`CCD_TRAVEL_RATIO` 是待根据真实基准数据校准的可调常数，不在这里手拍）。

### 18.5.2 算法：保守推进

```rust
fn ccd_resolve(body: &mut BodyRecord, others: &[BodyRecord], dt: f32) -> f32 {
    // 1. 使用扫描体计算保守的首次接触时间（TOI）下界
    //    （MVP 形状的球扫描：即便 box/sphere 也将 body 运动视为胶囊扫描，
    //     使用其包围球——一种标准的保守近似）
    // 2. 将 body 推进到恰好 TOI 之前，在该子位置重跑窄阶段
    // 3. 若发现真实接触，生成到本帧的流形列表（喂入 gvpe-constraint §5.2, 17号文書），
    //    替代穿透后的位置
    // 4. 若无真实接触（保守下界的假阳性），回退为完整 dt
    conservative_advancement_loop(body, others, dt, MAX_CCD_ITERATIONS)
}
```

CCD 的输出是修正后的位置/TOI，回馈到同一积分步骤（`17_detailed_design.md` §4.2）——它不绕过约束求解器，而是在求解器看到（已经穿透的）状态前阻止穿透。

### 18.5.3 执行图位置

CCD 在 `05_runtime_design.md` §5.5 步骤拆解（`... → Integrate → CCD → Output`）中的阶段在主 integrate 之后执行，仅作用于 §18.5.1 标记的 body 子集——保持 Execution Graph 形状（`03_graph_schema.md` §1.C）不变，CCD 是现有阶段内的条件性 fan-out，而非新阶段类型。

## 18.6 欠 `15_testing_strategy.md` 的测试夹具

- 一个不启用 CCD 会穿透薄板的高速球，启用 CCD 后必须不穿透（回归夹具）。
- 一个在重力下由铰链关节将两个 body 固定在固定角度偏移的配置，验证其与手算预期静止配置一致（与 `15号文書` §15.6「验证真实求解器运行，不手编魔法数字」同一纪律）。

Requirements satisfied: `01_requirements.md` GVPE-FR-002 (now explicitly covering joints/CCD, which
that requirement's original text left implicit), `00_vision.md` §0.5 (a physics engine claiming
completeness needs both).
