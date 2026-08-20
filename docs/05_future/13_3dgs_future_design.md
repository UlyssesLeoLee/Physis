# GVPE — 3DGS 物理反演未来设计（基本設計書）

## 0. 文档元数据

| 字段 | 取值 |
|---|---|
| 文档编号 | GVPE-DOC-13 |
| 文档类型 | 基本設計書 |
| 版本 | v0.2（标准化重写版） |
| 状态 | Draft |
| 适用阶段 | 非 MVP（Phase 2+） |
| 关联系统 | GVPE / `gvpe-3dgs`、`gvpe-inference` |
| 上游文档（输入基线） | GVPE-DOC-11（`11_vector_design.md`）、GVPE-DOC-02（`02_physics_ontology.md` §18）、GVPE-DOC-01（`01_requirements.md` §11、NG2）、GVPE-DOC-04（`04_architecture.md` §4.3） |
| 下游文档（被消费于） | — |
| 编写者 | — |
| 审批者 | — |

## 1. 修订历史

| 版本 | 日期 | 修改者 | 修改内容 |
|---|---|---|---|
| v0.1 | — | [原作者] | 初稿（原始版本） |
| v0.2 | 2026-08-19 | 标准化 pass | 转为 IPA 基本設計書 格式；叙事中文化；保留全部技术内容；补充文档元数据、修订历史、术语定义、关联文档、审批签字等标准章节 |

## 2. 文档目的

本文档规定 3DGS 物理反演闭环的形状，使该闭环在将来被实现时不需要回炉重做 §02 / §03 / §04。目标：

- 把 Observation → Retrieval → Graph Hypothesis → Simulation → Comparison → Optimization 的链路过早设计清楚，作为后续实现的蓝图。
- 给出 `gvpe-3dgs` 与现有模块的边界，使它不影响 MVP 验收（AC-01/02/03）。
- 明确“闭环存在但暂不实现”对当前 MVP 无任何阻塞。

## 3. 适用范围

- 适用：3DGS 物理反演闭环的形状、`gvpe-3dgs` 的模块边界、假设驱动的仿真回路。
- 不适用：具体优化算法、迭代次数、收敛判据——这些属于 `gvpe-inference` 的实现细节，应当在真实 Observation 数据出现后再决定（见 §12）。

## 4. 术语定义

| 术语 | 说明 |
|---|---|
| 3DGS | 3D Gaussian Splatting，一种基于高斯椭球的动态三维重建表示 |
| 3DGS 物理反演 | 由 3DGS 重建数据反推其背后物理参数（材质 / 约束 / 场 / 过程）的过程 |
| `gvpe-3dgs` | 本项目内的 crate，负责消化外部 3DGS 重建产出并产 `Observation` 图节点与原始特征 |
| Observation | 本体中的观测节点，是反演闭环的输入 |
| `PhysicsProfile` | 物理配置（材质、约束参数等）的本体对象 |
| `Hypothesis` | 本体中的假设节点，介于 `Observation` 与 `PhysicsProfile` 之间 |
| `Simulation` | 本体中的仿真节点，由 `Hypothesis` 驱动并产出 `SimulationState` |
| `SimulationState` | 单次仿真运行的中间 / 最终状态，可从中提取 `SimulatedPhysicsSignature` |
| 闭环 | 观测 → 检索 → 假设 → 仿真 → 误差 → 参数优化 的循环 |

## 5. 前提与约束

- 显式声明为非 MVP（`01_requirements.md` §11、NG2），不构成 MVP 阻塞。
- 闭环所需的全部图节点类型已在 `02_physics_ontology.md` §18 中定义；不引入新的本体节点类型。
- `gvpe-3dgs` 不直接接触 `gvpe-runtime` / `gvpe-solver`——其与核心仿真空间的交互必须经由 `gvpe-compiler` 介导的边界（与所有其他 Graph / Vector 空间消费者一致，见 `04_architecture.md` §4.3/§4.4）。
- 闭环的缺失或不完整不得影响 AC-01/02/03。

## 6. 系统架构 / 模块设计

### 6.1 闭环

```
Dynamic 3DGS → Temporal Feature → Motion Feature → Deformation Feature
    → ObservedPhysicsSignature → Vector Retrieval → Graph Hypothesis
    → Candidate PhysicsProfile → Rust Simulation → SimulatedPhysicsSignature
    → Error → Parameter Optimization
```

### 6.2 模块边界

`gvpe-3dgs` 消费外部的 3DGS 重建产出（GVPE 不实现 3DGS 重建本身），并产出 `Observation` 图节点与送往 `11_vector_design.md` 提取边界的原始特征数据。它绝不直接接触 `gvpe-runtime` / `gvpe-solver`——遵循与其他 Graph / Vector 空间消费者相同的、由 `gvpe-compiler` 介导的边界（见 `04_architecture.md` §4.3/§4.4）。

### 6.3 假设驱动的仿真回路

```
Observation --SUPPORTS--> Hypothesis --ASSUMES--> PhysicsProfile
Hypothesis --TESTED_BY--> Simulation --PRODUCES--> SimulationState
  --(extract)--> SimulatedPhysicsSignature
  --(compare against ObservedPhysicsSignature)--> Error
  --(feeds)--> gvpe-inference parameter optimization --> refined PhysicsProfile
```

这是 `02_physics_ontology.md` §18 中 `Hypothesis` / `Simulation` schema 的直接实例化——本闭环不需要新增图节点类型；这本身也是 §26 “无破坏性迁移” 承诺对本扩展成立的证据。

## 7. 接口设计

| 接口方向 | 接口形态 | 频次 |
|---|---|---|
| 外部 3DGS 重建 → `gvpe-3dgs` | 3DGS 重建产出流（外部，由 GVPE 外部系统产出） | 离线 / 事件触发 |
| `gvpe-3dgs` → `gvpe-graph` | `Observation` 节点 | 离线 |
| `gvpe-3dgs` → `gvpe-vector` | 原始特征数据 | 离线 / 低频 |
| `gvpe-inference` → `gvpe-graph` | `Hypothesis` / `PhysicsProfile` 候选 | 迭代粒度 |
| `gvpe-graph` → `gvpe-runtime`（经 compiler） | `PhysicsProfile` → `PhysicsCompiler` 产出 | 每次仿真前 |
| `gvpe-runtime` → `gvpe-inference` | `SimulationState` / `SimulatedPhysicsSignature` / `Error` | 每次迭代 |

## 8. 数据模型

闭环所涉及的图节点类型均已在 `02_physics_ontology.md` §18 中定义：

| 节点 | 作用 |
|---|---|
| `Observation` | 来自 3DGS 的观测输入 |
| `Hypothesis` | 介于 `Observation` 与 `PhysicsProfile` 之间的假设 |
| `PhysicsProfile` | 物理参数配置（材质、约束参数等） |
| `Simulation` | 由 `Hypothesis` 驱动的仿真运行 |
| `SimulationState` | 仿真运行的中间 / 最终状态 |

## 9. 处理流程

1. 外部系统提供 3DGS 重建数据（`gvpe-3dgs` 不参与重建本身）。
2. `gvpe-3dgs` 解析 3DGS，抽取时序 / 运动 / 形变特征，并产 `Observation` 图节点。
3. `gvpe-vector` 从 `SimulationState` / 原始特征中提取 `ObservedPhysicsSignature`。
4. 在 `KnownPhysicsSignature` 之上做检索，得到若干 `Hypothesis` 候选。
5. 候选 → `PhysicsProfile` → `PhysicsCompiler` → `gvpe-runtime` 跑出 `Simulation`。
6. 提取 `SimulatedPhysicsSignature` 并与 `ObservedPhysicsSignature` 对比，得到 `Error`。
7. `gvpe-inference` 以 `Error` 为输入驱动参数优化，得到精炼后的 `PhysicsProfile`，回到第 5 步迭代。

## 10. 关联需求

| 需求编号 / 本体节 | 描述 |
|---|---|
| `01_requirements.md` §11 | 显式将 3DGS 反演排除在 MVP 范围之外 |
| `01_requirements.md` NG2 | 非 MVP 阻塞的硬性约束 |
| `02_physics_ontology.md` §18 | `Hypothesis` / `Simulation` schema |
| `02_physics_ontology.md` §26 | “无破坏性迁移” 约束 |
| AC-01 / AC-02 / AC-03 | 仅针对 Simulation Space 的验收标准，不受本闭环影响 |

## 11. 关联文档

- `docs/01_requirements.md`（GVPE-DOC-01）§11、NG2、AC-01/02/03
- `docs/02_physics_ontology.md`（GVPE-DOC-02）§18、§26
- `docs/04_architecture.md`（GVPE-DOC-04）§4.3：Compiler 介导边界
- `docs/11_vector_design.md`（GVPE-DOC-11）：提取边界与签名类型

## 12. 明确不规定项

以下内容不在本文档中规定，留待 `gvpe-inference` 在真实 Observation 数据出现后再做：

- 驱动精炼步骤的具体优化算法（梯度法 / CMA-ES / 贝叶斯等）。
- 每次迭代送入仿真的检索候选数量。
- 收敛判定准则。

## 13. 非阻塞性保证

本文档所命名的所有模块（`gvpe-3dgs`、`gvpe-inference` 优化回路）在依赖图中严格位于 `gvpe-compiler` 之上（见 `04_architecture.md` §4.3）——它们的缺失或不完整不会影响 `01_requirements.md` 中 AC-01 / AC-02 / AC-03 这三项 Simulation-Space 专属的验收标准。

## 14. 审批签字

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 编写 |  |  |  |
| 校对 |  |  |  |
| 审批 |  |  |  |

---

## 15. 正文

> 原始输入基线：`11_vector_design.md`、`02_physics_ontology.md` §18。显式声明为非 MVP 阻塞（`01_requirements.md` §11、NG2）——本文档的存在，是为了让闭环的形状被一次性设计好，避免将来真要实现时回炉重做 §02 / §03 / §04。

### 13.1 闭环

```
Dynamic 3DGS → Temporal Feature → Motion Feature → Deformation Feature
    → ObservedPhysicsSignature → Vector Retrieval → Graph Hypothesis
    → Candidate PhysicsProfile → Rust Simulation → SimulatedPhysicsSignature
    → Error → Parameter Optimization
```

### 13.2 模块边界

`gvpe-3dgs` 消费外部的 3DGS 重建产出（GVPE 不实现 3DGS 重建本身），并产出 `Observation` 图节点与送往 `11_vector_design.md` 提取边界的原始特征数据。它绝不直接接触 `gvpe-runtime` / `gvpe-solver`——遵循与其他 Graph / Vector 空间消费者相同的、由 `gvpe-compiler` 介导的边界（见 `04_architecture.md` §4.3 / §4.4）。

### 13.3 假设驱动的仿真回路

```
Observation --SUPPORTS--> Hypothesis --ASSUMES--> PhysicsProfile
Hypothesis --TESTED_BY--> Simulation --PRODUCES--> SimulationState
  --(extract)--> SimulatedPhysicsSignature
  --(compare against ObservedPhysicsSignature)--> Error
  --(feeds)--> gvpe-inference parameter optimization --> refined PhysicsProfile
```

这是 `02_physics_ontology.md` §18 中 `Hypothesis` / `Simulation` schema 的直接实例化——本闭环不需要新增图节点类型；这本身也是 §26 “无破坏性迁移” 承诺对本扩展成立的证据。

### 13.4 明确不规定项

驱动精炼步骤的具体优化算法（梯度法 / CMA-ES / 贝叶斯等）、每次迭代送入仿真的检索候选数量、收敛判定准则，都属于 `gvpe-inference` 的实现细节，应当在真实 Observation 数据出现后再决定，而不是在此凭空推测。

### 13.5 非阻塞性保证

本文档所命名的所有模块（`gvpe-3dgs`、`gvpe-inference` 优化回路）在依赖图中严格位于 `gvpe-compiler` 之上（见 `04_architecture.md` §4.3）——它们的缺失或不完整不会影响 `01_requirements.md` 中 AC-01 / AC-02 / AC-03 这三项 Simulation-Space 专属的验收标准。
