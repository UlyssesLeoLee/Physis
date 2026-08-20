# GVPE — Solver Design（基本設計書）

## 0. 文档元数据

| 字段 | 取值 |
|---|---|
| 文档编号 | GVPE-DOC-07 |
| 文档类型 | 基本設計書 |
| 版本 | v0.2（标准化重写版） |
| 状态 | Draft |
| 适用阶段 | MVP |
| 关联系统 | GVPE / gvpe-solver, gvpe-constraint, gvpe-island |
| 上游文档（输入基线） | GVPE-DOC-06（06_collision_design.md §6.4），GVPE-DOC-04（04_architecture.md §4.5） |
| 下游文档（被消费于） | GVPE-DOC-09（09_parallel_design.md），GVPE-DOC-14（14_performance_budget.md） |
| 编写者 | — |
| 审批者 | — |

## 1. 修订历史

| 版本 | 日期 | 修改者 | 修改内容 |
|---|---|---|---|
| v0.1 | — | — | 初稿（原始版本） |
| v0.2 | 2026-08-19 | 标准化 pass | 转为 IPA 基本設計書 格式；叙事中文化；保留全部技术内容；补充文档元数据、修订历史、术语定义、关联文档、审批签字等标准章节 |

## 2. 文档目的

本基本設計書定义 GVPE 求解器（`gvpe-solver`）的两个世代（MVP 的 Sequential Impulse / PGS，post-MVP 的 XPBD）、`ConstraintRow` 数据结构、摩擦近似方案、刚体睡眠（Sleeping）机制以及 MVP 显式不做的事项。文档目的是为刚体动力学求解提供统一的数据契约、迭代算法与摩擦 / 睡眠策略基线。

## 3. 适用范围

本文件适用于 `gvpe-solver` crate 以及 `gvpe-constraint` crate（`ConstraintRow` 与 `ContactConstraint` / `JointConstraint` 行格式定义）。`gvpe-island` 中关于物理岛构建与并行求解调度参见 GVPE-DOC-09。

## 4. 术语定义

| 术语 | 说明 |
|---|---|
| Sequential Impulse（SI） | 顺序冲量法，按顺序处理约束行并累加冲量的迭代式约束求解策略 |
| PGS（Projected Gauss-Seidel） | 投影 Gauss-Seidel 求解器，SI 的等价描述 |
| XPBD（Extended Position-Based Dynamics） | 扩展 PBD，引入 compliance 显式处理位置误差 |
| `ConstraintRow` | 求解器迭代的最小行格式（无论语义来源如何，所有约束统一到该格式） |
| 接触约束（ContactConstraint） | 接触法向 / 切向冲量约束 |
| 关节约束（JointConstraint） | 固定 / 铰链等关节的约束（post-MVP） |
| 摩擦锥（Friction Cone） | 切向冲量允许域；本求解器用 box 约束近似 |
| Warm-start | 用上一帧的累积冲量 `lambda` 作为本帧迭代初值 |
| Sleeping | 刚体的"睡眠"状态，低于阈值的 body 在 N 帧后进入该状态 |
| 连续碰撞检测（CCD） | 防止高速运动穿透的检测，MVP 不实现 |
| `BodyIndex` | body 运行时索引 |
| `Jacobian` | 约束的雅可比矩阵，本求解器以 `[f32; 12]` 平铺表示（两 body 的线 + 角自由度） |
| `lambda` | 累积冲量，用于 warm-starting |
| `compliance` | XPBD 风格的柔度参数，`ConstraintRow` 已为该字段预留 |

## 5. 前提与约束

1. 上游基线：本文件以 `06_collision_design.md` §6.4（`ContactManifold`）为输入，并受 `04_architecture.md` §4.5（Law → Model → Solver 可追溯表）约束。
2. 统一行格式：所有约束类型（`ContactConstraint`、摩擦行、未来的 `JointConstraint`）必须统一为 `ConstraintRow`。求解器仅迭代 `ConstraintRow`，与具体语义解耦（具体语义归属图本体，依据 `02_physics_ontology.md` §9 绑定规则；本行仅属于运行时约束图空间）。
3. XPBD 字段预留：`ConstraintRow` 中的 `compliance` 字段已为 XPBD 预留（XPBD 的 compliance 参数可直接映射），因此 XPBD 落地是求解器替换而非数据模型变更。
4. MVP 显式不做：本章 §6.5 列举的关节 / CCD 范围外行为。
5. Sleeping 状态语义：依据 `02_physics_ontology.md` §6 的 `State` 序列，`Sleeping` 是 `State` 而非永久 `Property`。

## 6. 系统架构 / 模块设计

### 6.1 第一世代（MVP）：Sequential Impulse / Projected Gauss-Seidel

```rust
struct ConstraintRow {
    body_a: BodyIndex, body_b: BodyIndex,
    jacobian: [f32; 12],          // linear+angular for both bodies
    bias: f32, compliance: f32,
    lambda: f32,                  // accumulated impulse, for warm-starting
    lower: f32, upper: f32,       // impulse bounds (friction cone uses this)
}
```

- 求解循环：warm-start 取上一帧的 `lambda` → 在物理岛内对所有行进行 N 次 Gauss-Seidel 扫描 → 每次扫描后投影冲量边界 → 积分。
- 所有约束类型（`ContactConstraint`、摩擦行、未来的 `JointConstraint`）统一为 `ConstraintRow` —— 求解器仅迭代该单一格式，与语义来源无关。

### 6.2 第二世代（post-MVP）：XPBD

- 保留用于 Rope、Cloth、SoftBody（`02_physics_ontology.md` §4 MechanicalBehavior 类型）。
- MVP 不实现。
- `ConstraintRow` 中的 `compliance` 字段已为 XPBD 预留（XPBD 的 compliance 参数可直接映射），因此该升级属于求解器替换而非数据模型变更。

### 6.3 摩擦

- Coulomb 摩擦锥近似为 box 约束（边界由法向冲量 × 摩擦系数推导）。
- 该 box 约束在 sequential-impulse 循环内一并处理，**避免**额外的摩擦锥独立求解 pass。
- 上述做法是从零自研第一世代求解器的标准方案。

### 6.4 刚体睡眠（Sleeping）

- 线速度 + 角速度连续 N 帧低于阈值的 body 转入 `Sleeping` 状态。
- 处于 `Sleeping` 的 body 从 `gvpe-island` 的活跃 body 计数中排除，直到新接触 / 新外力唤醒。
- 语义依据：`02_physics_ontology.md` §6 的 `State` 序列 —— `Sleeping` 是 `State` 而非永久 `Property`。

### 6.5 MVP 显式不做的范围

- 关节类型：除"用于验证 `ConstraintRow` 抽象的最小关节"外不做。最小固定 / 铰链关节可作为抽象验证用例加入，但**不**作为功能承诺。
- CCD：MVP 不实现，仅在执行图 GVPE-DOC-05 §9.1 中保留为可空跑（no-op）阶段。

## 7. 接口设计

### 7.1 求解器行格式

`ConstraintRow` 字段见 §6.1。所有约束在进入求解器前必须转换为该格式。

### 7.2 求解循环伪代码（约束循环部分）

```
loop N iterations within an island:
    for each ConstraintRow in island:
        compute new impulse with projected GS
        clamp / project into [lower, upper]
        accumulate into lambda
```

`lambda` 在帧末保留，供下一帧 warm-start 使用。

## 8. 数据模型

- `ConstraintRow`：详见 §6.1 字段表。
- `ContactConstraint`：精筛 `ContactManifold` 转换后得到的 `ConstraintRow` 集合。
- `JointConstraint`：post-MVP，由关节类型生成 `ConstraintRow`。
- 摩擦行：作为接触行的派生约束行并入求解循环。

## 9. 处理流程

求解循环在物理岛内（参见 GVPE-DOC-09）执行：

```
warm-start (use previous lambda) → N × GS sweep over rows → project impulse bounds → integrate
```

## 10. 关联需求

| 需求 ID | 中文描述 | 在本文件中的落地点 |
|---|---|---|
| GVPE-FR-002 | 物理求解为自研核心 | §6.1 SI/PGS 自研实现 |
| 02_physics_ontology.md §4 | MechanicalBehavior 类型（Rope / Cloth / SoftBody）映射到 XPBD | §6.2 XPBD 保留 |
| 02_physics_ontology.md §6 | State 序列中 `Sleeping` 的语义 | §6.4 Sleeping |
| 02_physics_ontology.md §9 | 图节点与运行时约束的绑定规则 | §6.1 统一 `ConstraintRow` |
| 04_architecture.md §4.5 | Law → Model → Solver 可追溯表 | §5 前提；§6.1 SI 与 RigidBodyModel 对应 |

## 11. 关联文档

- 上游：`docs/02_modules/06_collision_design.md`（GVPE-DOC-06），`docs/01_architecture/04_architecture.md`（GVPE-DOC-04），`docs/01_architecture/02_physics_ontology.md`（GVPE-DOC-02）
- 下游：`docs/01_architecture/09_parallel_design.md`（GVPE-DOC-09），`docs/00_vision/14_performance_budget.md`（GVPE-DOC-14）

## 12. 审批签字

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 编写 |  |  |  |
| 校对 |  |  |  |
| 审批 |  |  |  |

---

## 13. 正文

Input baseline: `06_collision_design.md` §6.4, `04_architecture.md` §4.5.

## 7.1 Generation 1 (MVP): Sequential Impulse / Projected Gauss-Seidel

```rust
struct ConstraintRow {
    body_a: BodyIndex, body_b: BodyIndex,
    jacobian: [f32; 12],          // linear+angular for both bodies
    bias: f32, compliance: f32,
    lambda: f32,                  // accumulated impulse, for warm-starting
    lower: f32, upper: f32,       // impulse bounds (friction cone uses this)
}
```
All constraint types (`ContactConstraint`, friction rows, later `JointConstraint`) unify into
`ConstraintRow` — this is the single row format the solver iterates, regardless of semantic origin
(the semantic origin lives in the graph, per `02_physics_ontology.md` §9's binding rule; the row
here is Runtime Constraint Graph territory only).

Solve loop: warm-start from previous frame's `lambda` → N Gauss-Seidel sweeps over rows within an
island → project impulse bounds each sweep → integrate.

## 7.2 Generation 2 (post-MVP): XPBD

Reserved for Rope, Cloth, SoftBody (`02_physics_ontology.md` §4 MechanicalBehavior types). Not
implemented in MVP; `ConstraintRow`'s compliance field is already XPBD-compatible (XPBD's
compliance parameter maps directly), so this is a solver-swap, not a data-model change, when it
lands.

## 7.3 Friction

Coulomb friction cone approximated as box constraints (bounds derived from normal impulse ×
friction coefficient) within the sequential-impulse loop — standard approach for a from-scratch
Generation-1 solver, avoids a separate friction-cone solve pass.

## 7.4 Sleeping

Bodies whose linear+angular velocity stay below a threshold for N consecutive frames transition to
`Sleeping`; excluded from `gvpe-island` active-body counts until woken by a new contact/force
(`02_physics_ontology.md` §6's `State` sequence — `Sleeping` is a `State`, not a permanent
`Property`).

## 7.5 What this solver explicitly does not do (MVP)

No joint types beyond what's needed to validate the `ConstraintRow` abstraction generically (a
minimal fixed/hinge joint may be added for testing the abstraction, not as a feature commitment).
No CCD in MVP (reserved, listed in the Execution Graph §5.5 as a stage that MVP may no-op).
