# GVPE — 向量空间设计（ベクトル空間設計）（基本設計書）

## 0. 文档元数据

| 字段 | 取值 |
|---|---|
| 文档编号 | GVPE-DOC-11 |
| 文档类型 | 基本設計書 |
| 版本 | v0.2（标准化重写版） |
| 状态 | Draft |
| 适用阶段 | MVP |
| 关联系统 | GVPE / `gvpe-vector` |
| 上游文档（输入基线） | GVPE-DOC-02（`02_physics_ontology.md` §20） |
| 下游文档（被消费于） | GVPE-DOC-13（`13_3dgs_future_design.md`） |
| 编写者 | — |
| 审批者 | — |

## 1. 修订历史

| 版本 | 日期 | 修改者 | 修改内容 |
|---|---|---|---|
| v0.1 | — | [原作者] | 初稿（原始版本） |
| v0.2 | 2026-08-19 | 标准化 pass | 转为 IPA 基本設計書 格式；叙事中文化；保留全部技术内容；补充文档元数据、修订历史、术语定义、关联文档、审批签字等标准章节 |

## 2. 文档目的

本文档规定 GVPE 中“向量空间（Vector Space）”一侧的形状，重点解决“物理签名（Physics Signature）”是单向量嵌入还是多向量组合这一根本选择。目标包括：

- 给出 `PhysicsSignature` 的多向量结构，使各子签名可独立计算与独立比较。
- 用 newtype 包装在类型层区分 `ObservedPhysicsSignature` / `SimulatedPhysicsSignature` / `KnownPhysicsSignature`。
- 明确“提取不进入热路径”的边界，以保护仿真主回路的实时性能预算。
- 明确检索只产候选、解由物理验证的总原则。

## 3. 适用范围

- 适用：`gvpe-vector` crate 的签名结构、提取边界、检索接口形态。
- 不适用：具体的嵌入维度、编码器架构、ANN 索引技术（这些推迟到 `gvpe-graph` 拥有足够真实实例数据后再做 spike，见 §12）。

## 4. 术语定义

| 术语 | 说明 |
|---|---|
| 物理签名（Physics Signature） | 从一次仿真（或一次观测）提取出的多向量特征描述 |
| 子签名 | `MaterialSignature` / `ContactSignature` / `EnvironmentSignature` / `MotionSignature` / `EnergySignature` / `SolverSignature` / `DeformationSignature` / `WaveSignature` / `InteractionSignature` / `FieldSignature` |
| `ObservedPhysicsSignature` | 从观测数据（Observation）提取出的物理签名 |
| `SimulatedPhysicsSignature` | 从一次仿真运行产出的物理签名 |
| `KnownPhysicsSignature` | 取自已通过验证的图谱条目 |
| 提取边界 | `gvpe-runtime` → `gvpe-vector` 的状态快照消费接口；明确位于热路径之外 |
| ANN 检索 | 近似最近邻检索，用于在已知签名集合中找候选 |
| 候选 | 检索阶段返回的若干可能解；最终答案由物理闭环的 Comparison/Optimization 步裁定 |

## 5. 前提与约束

- 满足 `GVPE-VEC-001`：签名提取必须脱离热路径，绝不发生在 `step(dt)` 的同步链路中。
- 满足 `GVPE-VEC-002`：`Observed` / `Simulated` / `Known` 三类签名必须类型可区分。
- 检索只产候选，物理验证是最终仲裁（“retrieval proposes, physics verifies”）。
- 满足 `02_physics_ontology.md` §20 的语义约束。

## 6. 系统架构 / 模块设计

### 6.1 物理签名——多向量，非单嵌入

```
PhysicsSignature
├── MaterialSignature    ├── ContactSignature      ├── EnvironmentSignature
├── MotionSignature       ├── EnergySignature        └── SolverSignature
├── DeformationSignature  ├── WaveSignature
└── InteractionSignature  └── FieldSignature
```

每个子签名均可独立计算、独立比较；如需融合为单一相似度评分，应在检索层进行，而不是在提取阶段把所有子向量拼成一条无差别的长向量。该结论与归档 PRE 工作的 Multi-Vector ADR 已经确立的推理一致：它本身正确且独立于项目，因此在此重述。

### 6.2 三类签名实例（类型可区分，符合 GVPE-VEC-002）

```rust
struct ObservedPhysicsSignature(PhysicsSignature);   // from Observation
struct SimulatedPhysicsSignature(PhysicsSignature);  // from a Simulation run
struct KnownPhysicsSignature(PhysicsSignature);      // from an already-Validated graph entry
```

采用 newtype 包装而非带 tag 字段的共享 struct，目的是把“将 `Observed` 签名与另一份 `Observed` 签名比较、而本意是比较 `Simulated`”一类错误挡在编译期，而非依赖运行时纪律。

### 6.3 提取边界（脱离热路径，符合 GVPE-VEC-001）

```
gvpe-runtime produces SimulationState (per-frame, hot path)
        │  (offline / 1~30Hz / event-triggered — NOT every step)
        ▼
gvpe-vector extracts PhysicsSignature from a SimulationState snapshot
```

`gvpe-vector` 绝不在 `step(dt)` 内部同步调用 `gvpe-solver` / `gvpe-collision`——它以异步或更低频节奏消费已经产出的状态快照。

### 6.4 检索

在 `KnownPhysicsSignature` 条目之上做 ANN 风格相似度检索，返回若干候选送入 `gvpe-inference` 的假设生成环节（`13_3dgs_future_design.md`）。检索始终只产候选，绝不直接给出最终答案——闭环中的 Comparison/Optimization 步（`13_3dgs_future_design.md` §13.1）才是真正的仲裁者，遵循“retrieval proposes, physics verifies”这一已确立的正确原则。

## 7. 接口设计

| 接口方向 | 接口形态 | 频次 |
|---|---|---|
| `gvpe-runtime` → `gvpe-vector` | `SimulationState` 快照（离线 / 1–30Hz / 事件触发） | 离线或低频 |
| `gvpe-vector` → `gvpe-inference` | 候选签名集合 | 检索请求粒度 |
| `gvpe-vector` → 持久化 | `KnownPhysicsSignature` 写入图谱 | 验证通过后 |

具体函数签名在 §9 处理流程中给出形态，不在本节预先展开。

## 8. 数据模型

| 类型 | 内部组成 | 类型可区分机制 |
|---|---|---|
| `PhysicsSignature` | 多个子签名的聚合 | struct |
| `ObservedPhysicsSignature` | 包裹 `PhysicsSignature` | newtype |
| `SimulatedPhysicsSignature` | 包裹 `PhysicsSignature` | newtype |
| `KnownPhysicsSignature` | 包裹 `PhysicsSignature` | newtype |
| `MaterialSignature` 等子签名 | 各自独立的特征向量 | struct |

## 9. 处理流程

1. `gvpe-runtime` 在帧内产出 `SimulationState`（热路径）。
2. 离线 / 1–30Hz / 事件触发：消费 `SimulationState` 快照，调用 `gvpe-vector` 提取 `PhysicsSignature`。
3. `gvpe-vector` 接收一次 `Observation`，产出 `ObservedPhysicsSignature`。
4. 由 `gvpe-inference` 发起检索：在 `KnownPhysicsSignature` 上做 ANN 检索，得到若干候选。
5. 候选进入闭环，物理验证步骤（Comparison/Optimization）做出最终裁决。

## 10. 关联需求

| 需求编号 | 描述 |
|---|---|
| GVPE-VEC-001 | 物理签名的提取必须脱离 `step(dt)` 热路径 |
| GVPE-VEC-002 | `Observed` / `Simulated` / `Known` 三类签名必须类型可区分（newtype） |

## 11. 关联文档

- `docs/02_physics_ontology.md`（GVPE-DOC-02）§20：物理签名的本体定义
- `docs/13_3dgs_future_design.md`（GVPE-DOC-13）：3DGS 物理反演闭环、Comparison/Optimization 仲裁
- `docs/04_architecture.md`（GVPE-DOC-04）§4.3/§4.4：Compiler 介导的 Graph/Vector 边界

## 12. 明确不决策项

以下内容不在本文档中决定，留待实现 spike：

- 嵌入维度。
- 编码器架构：确定性特征向量 vs. 学习式编码。
- 具体 ANN 索引技术。

这些决策推迟到 `gvpe-graph` 的 schema（§03）积累了足够多的真实实例数据之后再做。在没有真实数据的情况下推测编码器架构属于过早优化，正是 `00_vision.md` §0.2 的禁止项所防范的对象。

## 13. 审批签字

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 编写 |  |  |  |
| 校对 |  |  |  |
| 审批 |  |  |  |

---

## 14. 正文

> 原始输入基线：`02_physics_ontology.md` §20、GVPE-VEC-001/002。

### 11.1 物理签名——多向量，非单嵌入

```
PhysicsSignature
├── MaterialSignature    ├── ContactSignature      ├── EnvironmentSignature
├── MotionSignature       ├── EnergySignature        └── SolverSignature
├── DeformationSignature  ├── WaveSignature
└── InteractionSignature  └── FieldSignature
```

每个子签名均可独立计算、独立比较；如需融合为单一相似度评分，应在检索层进行，而不是在提取阶段把所有子向量拼成一条无差别的长向量（与归档 PRE 工作 Multi-Vector ADR 已经确立的推理一致：它本身正确且独立于项目，因此在此重述）。

### 11.2 三类签名实例（GVPE-VEC-002，类型可区分）

```rust
struct ObservedPhysicsSignature(PhysicsSignature);   // from Observation
struct SimulatedPhysicsSignature(PhysicsSignature);  // from a Simulation run
struct KnownPhysicsSignature(PhysicsSignature);      // from an already-Validated graph entry
```

newtype 包装而非带 tag 字段的共享 struct——目的是把“将 `Observed` 签名与另一份 `Observed` 签名比较、而本意是比较 `Simulated`”一类错误挡在编译期，而非依赖运行时纪律。

### 11.3 提取边界（脱离热路径，符合 GVPE-VEC-001）

```
gvpe-runtime produces SimulationState (per-frame, hot path)
        │  (offline / 1~30Hz / event-triggered — NOT every step)
        ▼
gvpe-vector extracts PhysicsSignature from a SimulationState snapshot
```

`gvpe-vector` 绝不在 `step(dt)` 内部同步调用 `gvpe-solver` / `gvpe-collision`——它以异步或更低频节奏消费已经产出的状态快照。

### 11.4 检索

在 `KnownPhysicsSignature` 条目之上做 ANN 风格相似度检索，返回候选送入 `gvpe-inference` 的假设生成环节（`13_3dgs_future_design.md`）。检索始终只产候选，绝不直接给出最终答案——闭环中的 Comparison/Optimization 步（`13_3dgs_future_design.md` §13.1）才是真正的仲裁者，遵循“retrieval proposes, physics verifies”这一已确立且值得保留的原则。

### 11.5 明确不决策项

嵌入维度、编码器架构（确定性特征向量 vs. 学习式）、具体 ANN 索引技术，全部推迟到 `gvpe-graph` 的 schema（§03）积累了足够多的真实实例数据之后再做 spike。在没有真实数据的情况下推测编码器架构属于过早优化。
