# GVPE — 能量 / 波 / 场 / 过程 扩展设计（基本設計書）

## 0. 文档元数据

| 字段 | 取值 |
|---|---|
| 文档编号 | GVPE-DOC-12 |
| 文档类型 | 基本設計書 |
| 版本 | v0.2（标准化重写版） |
| 状态 | Draft |
| 适用阶段 | MVP+（预留扩展点，MVP 暂不实现） |
| 关联系统 | GVPE / `gvpe-dynamics`、`gvpe-runtime`（未来 crate：能量 / 波 / 场 / 过程） |
| 上游文档（输入基线） | GVPE-DOC-02（`02_physics_ontology.md` §10–§13、§26）、GVPE-DOC-04（`04_architecture.md` §4.5）、GVPE-DOC-01（`01_requirements.md` §11） |
| 下游文档（被消费于） | — |
| 编写者 | — |
| 审批者 | — |

## 1. 修订历史

| 版本 | 日期 | 修改者 | 修改内容 |
|---|---|---|---|
| v0.1 | — | [原作者] | 初稿（原始版本） |
| v0.2 | 2026-08-19 | 标准化 pass | 转为 IPA 基本設計書 格式；叙事中文化；保留全部技术内容；补充文档元数据、修订历史、术语定义、关联文档、审批签字等标准章节 |

## 2. 文档目的

本文档是“本体已包含这些概念”与“运行时可对它们进行计算”之间的桥梁：Energy / Wave / Field / Process 在本体中已经存在，但 MVP 的运行时不算、不必算。本文提前把这些概念预留的接入“缝隙（seams）”设计清楚，目的：

- 让 schema-to-runtime 路径被预先证明存在，而不是事后被断言。
- 让 MVP 之后的扩展是“附加”而非“迁移”——避免破坏性 schema 改动。
- 防止后续在 schema 压力下临时拼凑这些扩展点。

## 3. 适用范围

- 适用：Energy / Wave / Field / Process 在当前 MVP 代码中的预留位（hook）形状与边界。
- 不适用：上述任一现象的数值求解器设计——那属于未来的子项目（见 §12）。

## 4. 术语定义

| 术语 | 说明 |
|---|---|
| Energy | 物理本体中的能量概念（动能、引力势能、弹性能、热能等） |
| Wave | 物理本体中的波概念（频率、振幅、传播速度等） |
| Field | 物理本体中场概念（重力场、压力场、速度场等） |
| Process | 物理本体中跨多帧状态变化的过程（熔化、断裂、相变等） |
| `EnergyLedger` | 预留的能量账本结构（见 §6.1） |
| `ProcessState` | 预留的过程状态槽（见 §6.4） |
| Hook / 扩展点 | 现有模块中为未来功能预留、不在 MVP 填充的接入位 |

## 5. 前提与约束

- `01_requirements.md` §11 显式将 Energy / Wave / Field / Process 排除在 MVP 运行时范围之外。
- `02_physics_ontology.md` §26 要求这些概念在 schema 中存在，且未来不发生破坏性迁移。
- 满足上述两条的同时，本文不引入 MVP 不需要的数值求解器。
- 任何扩展必须对 MVP 的 `ConstraintRow` 与求解主循环不构成修改需求。

## 6. 系统架构 / 模块设计

### 6.1 能量追踪 Hook（预留运行时扩展点）

```rust
struct EnergyLedger {   // not populated in MVP; the type exists so Solver can be extended later
    kinetic: f32, gravitational_potential: f32, elastic_potential: f32, thermal: f32,
}
```

能量守恒的账目（对应 `02_physics_ontology.md` §10 的能量转换关系）将以“步后诊断 pass”形式计算：读取已经积分完成的 body state 做聚合，不要求改动 `ConstraintRow` 或求解循环本身，只需追加一个可选聚合 pass。这是“能量支持属于附加扩展、非迁移”的具体证据。

### 6.2 波传播 Hook

`02_physics_ontology.md` §11 的 `Wave` 节点（frequency、amplitude、propagation_speed 等）将映射到未来的 `gvpe-wave` crate，它消费 MVP 求解器已经产出的 body / contact 事件（依据本体因果链 `Collision GENERATES MechanicalWave`）——事件驱动型，而非对核心求解循环的修改。

### 6.3 场 Hook

`02_physics_ontology.md` §12 的 `Field` 类型（重力 / 压力 / 速度 / ...）将 MVP 当前硬编码的 `Gravity` 力（见 `02_physics_ontology.md` §7）泛化为可查询的空间函数。MVP 中的 `Gravity` 实现应当已经从一开始就以“在某位置对场做采样（当前为常量场）”的形式编写，而非“加一个常量向量”——同样的运行时成本，但抽象已经与未来的真实空间变化场兼容，无需重写。

### 6.4 过程 Hook

`02_physics_ontology.md` §13 的 `Process` 类型（熔化、断裂、相变等）本质上是作用于 `Entity` 的多帧状态机。预留的扩展点为 entity 上的通用 `ProcessState` 槽（MVP 暂不使用），未来的过程仿真 crate 可在不改动 `gvpe-dynamics` 的核心 per-body 状态布局的前提下挂载上来。

## 7. 接口设计

| 接口 | 形态 | MVP 状态 |
|---|---|---|
| `EnergyLedger` | 内存聚合结构 | 类型存在，字段不填 |
| 波事件流 | 由 MVP 求解器产出的 `body` / `contact` 事件订阅 | 事件已产出，订阅者未来挂载 |
| 场采样接口 | `sample(position) -> value` 形式的统一采样 | MVP 的 `Gravity` 走此形态 |
| `ProcessState` 槽 | entity 上的预留字段 | 字段存在，未填充 |

## 8. 数据模型

| 类型 / 槽 | 字段 | 备注 |
|---|---|---|
| `EnergyLedger` | `kinetic`、`gravitational_potential`、`elastic_potential`、`thermal: f32` | 聚合型，不直接进 SoA 热路径 |
| `ProcessState` | 通用槽，类型由未来 crate 决定 | 位于 entity 描述上 |
| 场采样输入 | `position` 空间坐标 | 由各 `Field` 子类型分别定义 |

## 9. 处理流程

1. MVP 主循环产出 body / contact 事件（与现有 `ConstraintRow` 求解无耦合）。
2. 未来 `gvpe-wave` crate 订阅这些事件并驱动波仿真。
3. 步结束后，未来 `EnergyLedger` pass 读取已积分的 body state 做能量聚合。
4. `Field` 采样接口在每步 `Apply Forces` 阶段被调用（当前 `Gravity` 是其特例）。
5. `ProcessState` 槽由未来过程仿真 crate 独立驱动，不影响主循环。

## 10. 关联需求

| 需求编号 / 本体节 | 描述 |
|---|---|
| `01_requirements.md` §11 | 显式将 Energy / Wave / Field / Process 排除在 MVP 运行时范围之外 |
| `02_physics_ontology.md` §10 | 能量转换关系（动 / 引力势 / 弹 / 热） |
| `02_physics_ontology.md` §11 | Wave 节点定义 |
| `02_physics_ontology.md` §12 | Field 类型定义 |
| `02_physics_ontology.md` §13 | Process 类型定义 |
| `02_physics_ontology.md` §26 | schema 演进“无破坏性迁移”约束 |

## 11. 关联文档

- `docs/01_requirements.md`（GVPE-DOC-01）§11：MVP 范围排除
- `docs/02_physics_ontology.md`（GVPE-DOC-02）§10–§13、§26：本体与“无破坏性迁移”约束
- `docs/04_architecture.md`（GVPE-DOC-04）§4.5：模块拓扑与扩展点位置

## 12. 本次明确非目标

本文档不为任何 Energy / Wave / Field / Process 现象设计数值求解器——只设计“将来挂载求解器的缝隙”。在缺乏真实驱动用例的情况下设计波动方程等具体数值方法，本身就是 `00_vision.md` §0.2 禁止项所防范的那种过早复杂化。

## 13. 审批签字

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 编写 |  |  |  |
| 校对 |  |  |  |
| 审批 |  |  |  |

---

## 14. 正文

> 原始输入基线：`02_physics_ontology.md` §10–§13、`04_architecture.md` §4.5。本文档是“本体已包含这些概念”（今日已成立）与“运行时可对它们进行计算”（MVP 不成立，也不必成立）之间的桥梁——目的是把这条桥一次性、有意识地设计好，避免在未来的 schema 压力下临时拼凑。

### 12.1 为什么本文档现在存在，但还没有要实现的东西

`01_requirements.md` §11 显式将 Energy / Wave / Field / Process 排除在 MVP 运行时范围之外，但 `02_physics_ontology.md` §26 要求其 schema 存在且不发生未来的破坏性迁移。本文档即“schema-to-runtime 路径是真实路径”的形状证明，而不是单纯的断言。

### 12.2 能量追踪 Hook（预留运行时扩展点）

```rust
struct EnergyLedger {   // not populated in MVP; the type exists so Solver can be extended later
    kinetic: f32, gravitational_potential: f32, elastic_potential: f32, thermal: f32,
}
```

能量守恒的账目（对应 `02_physics_ontology.md` §10 的能量转换关系）将以“步后诊断 pass”形式计算：读取已经积分完成的 body state 做聚合，不要求改动 `ConstraintRow` 或求解循环本身，只需追加一个可选聚合 pass。这是“能量支持属于附加扩展、非迁移”的具体证据。

### 12.3 波传播 Hook

`02_physics_ontology.md` §11 的 `Wave` 节点（frequency、amplitude、propagation_speed 等）将映射到未来的 `gvpe-wave` crate，它消费 MVP 求解器已经产出的 body / contact 事件（依据本体因果链 `Collision GENERATES MechanicalWave`）——事件驱动型，而非对核心求解循环的修改。

### 12.4 场 Hook

`02_physics_ontology.md` §12 的 `Field` 类型（重力 / 压力 / 速度 / ...）将 MVP 当前硬编码的 `Gravity` 力（见 `02_physics_ontology.md` §7）泛化为可查询的空间函数。MVP 中的 `Gravity` 实现应当已经从一开始就以“在某位置对场做采样（当前为常量场）”的形式编写，而非“加一个常量向量”——同样的运行时成本，但抽象已经与未来的真实空间变化场兼容，无需重写。

### 12.5 过程 Hook

`02_physics_ontology.md` §13 的 `Process` 类型（熔化、断裂、相变等）本质上是作用于 `Entity` 的多帧状态机。预留的扩展点为 entity 上的通用 `ProcessState` 槽（MVP 暂不使用），未来的过程仿真 crate 可在不改动 `gvpe-dynamics` 的核心 per-body 状态布局的前提下挂载上来。

### 12.6 本次明确非目标

本文档不为任何 Energy / Wave / Field / Process 现象设计数值求解器——只设计“将来挂载求解器的缝隙”。在缺乏真实驱动用例的情况下设计波动方程等具体数值方法，本身就是 `00_vision.md` §0.2 禁止项所防范的那种过早复杂化。
