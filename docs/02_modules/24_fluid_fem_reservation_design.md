# GVPE — Fluid & FEM Interface Reservation（詳細設計書）

## 0. 文档元数据

| 字段 | 取值 |
|---|---|
| 文档编号 | GVPE-DOC-24 |
| 文档类型 | 詳細設計書 |
| 版本 | v0.2（标准化重写版） |
| 状态 | Draft |
| 适用阶段 | 范围预留（interface-only，无运行期行为） |
| 关联系统 | GVPE / gvpe-shape, gvpe-graph, gvpe-compiler（接口预留） |
| 上游文档（输入基线） | GVPE-DOC-19（`19_softbody_xpbd_design.md` §19.6）、GVPE-DOC-02（`02_physics_ontology.md` §4 `MechanicalBehavior` 的 `FluidBehavior` 与 `DeformableBehavior` FEM 分支） |
| 下游文档（被消费于） | GVPE-DOC-21（`21_graph_compiler_detailed_design.md` §21.4 `compile()` 产生 `CompileError::UnsupportedModel("FluidBehavior")`） |
| 编写者 | — |
| 审批者 | — |

## 1. 修订历史

| 版本 | 日期 | 修改者 | 修改内容 |
|---|---|---|---|
| v0.1 | — | — | 初稿（原始版本） |
| v0.2 | 2026-08-19 | 标准化 pass | 转为 IPA 詳細設計書 格式；叙事中文化；保留全部技术内容；补充文档元数据、修订历史、术语定义、关联文档、审批签字等标准章节 |

## 2. 文档目的

本文档为流体（Fluid）与通用 FEM（Finite Element Method，有限元方法）这两个**故意不实现**的物理子系统划定文档化的边界：给出为什么它们不应被强行塞入 `ConstraintRow` / `ParticleStateSoA` 的技术理由、当前已预留的最小接口面（`ShapeDesc::FluidRegion`、`MechanicalBehaviorHint::Fluid | GeneralFem`）、以及一份「未来设计文档需回答的问题清单」。这样，「范围外」是一个有据可查的、有界的位置，而不是沉默的缺位——而「完备」（`00_vision.md` §0.5）所要求的正是连 GVPE 主动选择不构建的部分也留下文档。

## 3. 适用范围

- 适用 crate：`gvpe-shape`（接受 `FluidRegion` 形状描述）、`gvpe-graph`（接受 `FluidBehavior` / FEM 分支作为 `MechanicalBehavior` 属性值）、`gvpe-compiler`（在编译期返回 `CompileError::UnsupportedModel`）。
- 适用阶段：接口预留阶段；任何运行期消费路径均不存在。
- 不适用：MVP 性能预算、本文档集涵盖的任何版本。

## 4. 术语定义

- **流体（Fluid）**：本设计中专指 SPH（Smoothed Particle Hydrodynamics，平滑粒子流体动力学）或基于网格的压力投影方法所建模的连续介质。
- **FEM（Finite Element Method，有限元方法）**：通过组装全局刚度矩阵并对每一步进行稀疏线性求解的变形体建模方法（区别于 `19号文書` §19.4.3 的四面体体积保持特例）。
- **`ShapeDesc::FluidRegion`**：`gvpe-shape` 中以 `Aabb` + `FluidKind` 描述的预留形状变体。
- **`MechanicalBehaviorHint`**：面向图谱的机械行为提示枚举，仅作图谱侧属性值，编译期校验后才会触发 `CompileError::UnsupportedModel`。
- **`CompileError::UnsupportedModel`**：`gvpe-compiler` 已有的显式失败信号；当图谱节点请求尚未实现的模型时返回，与「图谱作者笔误」的失败纪律同源。

## 5. 模块详细设计

### 5.1 形状层接口预留

`gvpe-shape` 的 `ShapeDesc` 枚举新增 `FluidRegion { bounds: Aabb, kind: FluidKind }` 变体，但仅以类型系统形式存在；无对应运行期行为。

### 5.2 图谱层接口预留

`02_physics_ontology.md` §4 的 `FluidBehavior` 与 `DeformableBehavior` 的 FEM 分支继续作为 `NodeKind::Entity` 上的 `MechanicalBehavior` 属性值——是属性值而非 `NodeKind` 变体，因此 §21.1 的 `NodeKind` 枚举不受影响。

### 5.3 编译期显式失败

`PhysicsProfile::solver_type`（`17_detailed_design.md` §1.2）的 `SolverTypeId` 尚无 `Fluid` 或 `GeneralFem` 变体；因此 `21_graph_compiler_detailed_design.md` §21.4 的 `compile()` 对任何请求此类模型的图谱节点返回 `CompileError::UnsupportedModel("FluidBehavior")`——这与 §21.4 已建立的「显式失败」纪律同源，仅应用在「故意未实现」而非「图谱笔误」的场景。

## 6. 类与数据结构

```rust
enum ShapeDesc {
    // ... 20_shape_advanced_design.md §20.1 既有变体 ...
    FluidRegion { bounds: Aabb, kind: FluidKind },   // 预留——无运行期行为
}

enum MechanicalBehaviorHint {
    RigidBody,
    Xpbd,
    Fluid,
    GeneralFem,                                       // 仅图谱侧，§5.4
}
```

## 7. 算法详解

### 7.1 为什么流体与 FEM 不是 `ConstraintRow` / `ParticleStateSoA` 的扩展

`19号文書` §19.1 可对 XPBD 复用 `ConstraintRow.compliance`，原因是绳/布/软体都是**粒子之上的稀疏约束网络**——同一「按行迭代」抽象、不同约束类型。流体与通用 FEM 在同一意义上不是稀疏约束问题：

- **流体**（SPH 或基于网格）需要每子步重算**邻域密度场**（`gvpe-collision` 的宽相只找对、不算连续密度），以及压力投影或核函数求和步骤——在 `ConstraintRow` 中无对应物；其「约束」隐含在压力场中，而非显式行。
- **FEM**（通用情形，非 `19号文書` §19.4.3 已覆盖的四面体体积保持特例）需要组装全局刚度矩阵并每步线性求解——这在数值结构上（稀疏线性代数）与 Sequential Impulse / XPBD 的局部「按行迭代」循环有根本差异。

强行将二者塞入 `ConstraintRow` 即是 `00_vision.md` GVPE-PROHIBIT-06 所禁止的「优雅压倒性能」错误——为复用而复用，而非因适配而复用。

### 7.2 未来设计文档需回答的问题（不在本文档范围）

> 列示以便将缺口**有界化**而非开放式蔓延：

1. **流体**：邻域搜索策略（grid-hash vs 复用 `gvpe-collision` 的 SAP）、SPH 核函数或网格压力投影方法选择、表面重建（若面向渲染），以及流-刚耦合如何把力反馈到 `BodyStateSoA` 而不至于在同一帧内与 Sequential Impulse / XPBD 抢夺同一批刚体。
2. **通用 FEM**：单元类型支持（tet/hex）、刚度矩阵组装、稀疏线性求解器选择（直接 vs 迭代——涉及 `05_runtime_design.md` §5.3 的 DeterminismMode 含义，因稀疏迭代求解器有自身的收敛容差确定性故事），以及它是否以仿真空间频率运行，还是按 `00_vision.md` §0.3 的 Vector Space 节律（类比）作「昂贵、低频」物理。

以上均需驱动用例出现后再设计，遵循 `12_energy_wave_field_design.md` §12.6 与 `19号文書` §19.6 已应用的纪律——本文档有意止步于接口预留与问题清单，不给答案。

## 8. 错误处理

流体 / FEM 请求统一以编译期显式失败路径处理：

- `compile()` 遇到 `MechanicalBehavior = FluidBehavior` 或 FEM 分支的 `NodeKind::Entity` 时返回 `CompileError::UnsupportedModel("FluidBehavior")`（FEM 分支以相应标签发出）。
- 类型系统层：`FluidRegion` 与 `MechanicalBehaviorHint::Fluid | GeneralFem` 允许出现（因此图谱/工具链代码能编译并描述这些材料），但运行期无对应行为；任何运行期误用将由 `gvpe-runtime` 的健壮性检查拒绝。
- 失败模式与 `21_graph_compiler_detailed_design.md` §21.4 已建立的显式失败纪律保持一致：不静默退化为错误近似，而是直接报错。

## 9. 性能考量

- 当前预留接口在运行期不引入任何性能成本：`FluidRegion` 仅在 `ShapeDesc` 枚举中占位；`MechanicalBehaviorHint` 的 `Fluid | GeneralFem` 仅在图谱侧属性值中出现。
- 编译期错误返回快，不进入 `solve_island` 循环。
- 未来若实现流 / FEM，需独立设计其性能预算与是否进入 Simulation-Space 频率的判定——不在本文档范围。

## 10. 测试考量

- **类型系统存在性测试**：`ShapeDesc::FluidRegion` 与 `MechanicalBehaviorHint::Fluid | GeneralFem` 须可构造并通过模式匹配穷尽性检查。
- **编译期显式失败测试**：构造请求 `FluidBehavior` 的最小 `gvpe-graph` 节点，运行 `compile()`，断言返回 `CompileError::UnsupportedModel("FluidBehavior")`。
- **运行期静默缺位测试**：以 `FluidRegion` 描述创建实体并运行仿真 N 步，断言 `BodyStateSoA` 不被修改（即不存在未文档化的隐式行为）。
- **非目标锁定测试**：CI 流程应包含「本版本不实现流 / FEM」的契约性提醒（注释 / changelog / doctest），防止未来贡献者误把预留接口误读为承诺。

## 11. 关联需求

- **`01_requirements.md` NG1**：流体 / FEM 显式不在 MVP 范围——现以文档化边界替代隐式边界。
- **`00_vision.md` §0.5**：完备性——缺口现已**有界**且**有接口**，不再是沉默的缺位。
- **`02_physics_ontology.md` §4**：`FluidBehavior` / FEM 分支仍为有效 schema，并具有明确的运行期状态。
- **GVPE-PROHIBIT-06**：不为复用而复用——拒绝将流体 / FEM 强行塞入 `ConstraintRow`。

## 12. 关联文档

- 上游：`docs/02_modules/19_softbody_xpbd_design.md` §19.6（明确点名本缺口但刻意不设计）、`docs/02_physics_ontology/02_physics_ontology.md` §4（`MechanicalBehavior` 定义）。
- 平级：`docs/02_modules/23_energy_wave_field_process_algorithms.md`（同属「范围外但有文档」类）。
- 下游：`docs/02_modules/21_graph_compiler_detailed_design.md` §21.4（`compile()` 显式失败信号）。

## 13. 审批签字

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 编写 | | | |
| 校对 | | | |
| 审批 | | | |

---

## 14. 正文

> 以下为原文档正文，章节编号保留（§24.1, §24.2, §24.3, §24.4, §24.5），叙事已并入上方对应章节，本节保留原始技术片段与原文档的引用脉络。

### 24.1 为什么流体与 FEM 不是 `ConstraintRow` / `ParticleStateSoA` 的扩展

参见上方 §7.1。原文以项目符号形式列示的两条技术理由（流体需要邻域密度场与压力投影；FEM 需要全局刚度矩阵与稀疏线性求解）已并入叙述。

### 24.2 当前预留内容（接口而非实现）

参见上方 §5 与 §6。`ShapeDesc::FluidRegion` 与 `MechanicalBehaviorHint` 枚举定义完整保留。

### 24.3 未来设计文档需完成的工作（本文档不设计）

参见上方 §7.2。原文项目符号完整保留为编号列表（1. 流体 / 2. 通用 FEM），分别列出邻域搜索、SPH 核函数、刚度矩阵组装、稀疏线性求解器选择等需驱动用例出现后再决策的问题。

### 24.4 图谱侧预留（schema 完整、运行期缺位 —— 与 Energy/Wave/Field 同模式）

参见上方 §5.2。`02_physics_ontology.md` §4 的 `FluidBehavior` 与 `DeformableBehavior` 的 FEM 分支仍为有效的 `NodeKind::Entity` 属性值；这一位置与 `01_requirements.md` MVP 图谱范围对 Energy/Wave/Field/Process 节点的处理同源。

### 24.5 非目标（显式列出，以免被误读为承诺）

- 本文档集涵盖的任何版本不交付 SPH / 网格流体求解器。
- 本文档集涵盖的任何版本不交付通用 FEM 求解器。
- `FluidRegion` / `MechanicalBehaviorHint::Fluid | GeneralFem` 在类型系统中的存在不是交付日期的承诺——它只是当前最小接口面：让图谱/工具链代码能**今天**就描述这些材料，而将来真有运行期实现时不必做破坏性类型变更。
