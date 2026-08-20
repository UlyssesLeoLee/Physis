# GVPE — Graph Store & Compiler Detailed Design（詳細設計書）

## 0. 文档元数据

| 字段 | 取值 |
|---|---|
| 文档编号 | GVPE-DOC-21 |
| 文档类型 | 詳細設計書 |
| 版本 | v0.2（标准化重写版） |
| 状态 | Draft |
| 适用阶段 | MVP（设计就绪，运行时不在 MVP 范围） |
| 关联系统 | GVPE / gvpe-graph, gvpe-compiler |
| 上游文档（输入基线） | `03_graph_schema.md`, `04_architecture.md` §4.4, `17_detailed_design.md` §11, `01_requirements.md` §11, `02_physics_ontology.md` §1/§22, `15_testing_strategy.md` §15.4/§15.5, `16_dependency_license.md` §16.4 |
| 下游文档（被消费于） | `10_ffi_design.md` |
| 编写者 | — |
| 审批者 | — |

## 1. 修订历史

| 版本 | 日期 | 修改者 | 修改内容 |
|---|---|---|---|
| v0.1 | — | — | 初稿（原始版本） |
| v0.2 | 2026-08-19 | 标准化 pass | 转为 IPA 詳細設計書 格式；叙事中文化；保留全部技术内容；补充文档元数据、修订历史、术语定义、关联文档、审批签字等标准章节 |

## 2. 文档目的

本文档是 `17_detailed_design.md` §11 的占位 trait 获得真实内部实现的位置——「全部详细设计完备」所必需，即便 `gvpe-graph` / `gvpe-compiler` 仍在 MVP runtime 验收门之外（`01_requirements.md` §11）。覆盖范围：

- `GraphStore` 的存储引擎（HashMap 后备路径）
- 受限遍历查询（`bounded_traverse`）实现
- 写入路径守卫（关闭 `02_physics_ontology.md` 的 ONT-ISS-001）
- `PhysicsCompiler` 算法（含错误类型）
- 编译往返保证测试
- 显式非范围说明

## 3. 适用范围

- 适用于 `gvpe-graph` 与 `gvpe-compiler` 的存储引擎、查询、写入守卫、编译算法的详细设计
- 不适用于后端 DB 选型（外部图 DB vs 内嵌）——见 `16_dependency_license.md` 的决定
- 不适用于 GPU 端的图查询加速（属于后续 Phase）
- 运行时不在 MVP 范围，但设计必须在 MVP 阶段定稿

## 4. 术语定义

| 术语 | 定义 |
|---|---|
| 物理知识图谱（Physics Knowledge Graph, PKG） | 描述物理实体、属性、关系的有向图结构 |
| 物理编译器（Physics Compiler） | 将图谱查询结果编译为 `PhysicsProfile` 的组件 |
| 节点（Node） | 图中的顶点，包含 ID、Kind、属性 |
| 边（Edge） | 节点之间的有向关系，可选条件 |
| 邻接表（Adjacency List） | 以源节点 ID 为键的出边集合 |
| 属性索引（Property Index） | `(NodeId, PropertyKind) → NodeId` 的快速查找结构 |
| 受限遍历（Depth-Bounded Traversal） | `depth_bounded_traversal`：设定最大深度的图遍历 |
| 写入状态批（Write State Batch） | `write_state_batch` 函数：批量写入 State 到图谱 |
| 批量状态写入门槛 | 拒绝将高频 State 写入图谱的阈值（`BULK_STATE_WRITE_THRESHOLD`） |
| 条件边（Conditional Edge） | 仅在条件满足时生效的边（`02_physics_ontology.md` §22） |
| 编译器边界（Compiler Boundary） | Compiler 产物（`PhysicsProfile`）与 Runtime 之间的接口 |
| 编译往返（Compile Round-Trip） | 编译产物与手构 `PhysicsProfile` 的等值测试 |
| 单源真相（Single Source of Truth） | 节点类型枚举的来源是 `02_physics_ontology.md` §1 |
| 关系种类（RelationKind） | 边的语义类型，如 `HasMaterial`、`ModeledBy` |

## 5. 模块详细设计

### 5.1 存储引擎模块

`GraphStore` 使用 HashMap 后备实现（per `16_dependency_license.md` §16.4）。详见正文 §21.1。

### 5.2 受限遍历模块

`bounded_traverse` 实现 `03_graph_schema.md` §6 的"无无界深度"约束。详见正文 §21.2。

### 5.3 写入守卫模块

`write_state_batch` 在批量超过门槛时拒绝，从代码层关闭 ONT-ISS-001。详见正文 §21.3。

### 5.4 物理编译器模块

`compile` 算法从图谱节点读取属性并构造 `PhysicsProfile`。详见正文 §21.4。

### 5.5 编译往返测试模块

验证编译产物与手构结果等值的测试。详见正文 §21.5。

### 5.6 显式非范围模块

后端选型不属于本文。详见正文 §21.6。

## 6. 类与数据结构

### 6.1 存储引擎结构

```rust
struct GraphStore {
    nodes: HashMap<NodeId, Node>,
    edges: HashMap<NodeId, Vec<Edge>>,          // 邻接表, 以源节点为键
    property_index: HashMap<(NodeId, PropertyKind), NodeId>,   // Entity -> Property 快速查找
}

struct Node { id: NodeId, kind: NodeKind, attrs: HashMap<String, AttrValue> }
struct Edge { to: NodeId, relation: RelationKind, condition: Option<Condition> }

enum NodeKind {   // 与 02_physics_ontology.md §1 的顶层概念 1:1 完全对齐
    Entity, Matter, Material, Phase, Property, State, Force, Interaction, Constraint, Field,
    Energy, Wave, Process, PhysicalLaw, ConstitutiveModel, ApproximationModel, SolverModel,
    BoundaryCondition, Observation, Experiment, Hypothesis, Simulation, SimulationState,
    PhysicsProfileNode, VectorDescriptor, Result,
}
```

`NodeKind` 是闭枚举，恰好枚举 `02_physics_ontology.md` §1 的列表——添加不在该本体中的节点类型会导致编译错误，这是存储层将本体作为"Node 是什么"的唯一真相的执行机制（将 `03_graph_schema.md` §2 的决策规则提升到类型层，而不仅是设计指南）。

### 6.2 编译器接口

```rust
trait GraphStore {
    fn query_profile_inputs(&self, entity: EntityId) -> Result<GraphQueryResult, GraphError>;
}

trait PhysicsCompiler {
    fn compile(&self, input: GraphQueryResult) -> Result<PhysicsProfile, CompileError>;
}
```

`CompileError` 与 Runtime 侧的 `InitError` 是不同类型（以类型分离 `04号文書` §4.4 的边界）。

## 7. 算法详解

### 7.1 受限遍历查询（实现 `03_graph_schema.md` §6）

```rust
fn bounded_traverse(store: &GraphStore, start: NodeId, max_depth: u32,
                      relation_filter: Option<&dyn Fn(&Edge) -> bool>) -> Vec<NodeId> {
    let mut visited = HashSet::from([start]);
    let mut frontier = vec![start];
    for _ in 0..max_depth {          // 硬上限, 此函数中不存在无界深度代码路径
        let mut next = Vec::new();
        for node in &frontier {
            for edge in store.edges.get(node).into_iter().flatten() {
                if let Some(f) = relation_filter { if !f(edge) { continue; } }
                if edge.condition.as_ref().is_some_and(|c| !c.currently_holds(store)) { continue; }
                if visited.insert(edge.to) { next.push(edge.to); }
            }
        }
        if next.is_empty() { break; }
        frontier = next;
    }
    visited.into_iter().collect()
}
```

这是 `08_PRE_Detailed_Design.md` 归档的 `traverse()`（`docs/archive/`）在 graph-store 层的孪生——PRE 工作为自身图构建功能建立的深度限制纪律在此以相同形态重申，因为它是与项目本体无关的正确模式。条件边（§21.1 的 `Edge.condition`）实现 `02_physics_ontology.md` §22 的「条件关系支持」要求（如 `HigherTemperature DECREASES Viscosity` 仅在条件为真时成立）。

### 7.2 写入路径守卫（关闭 `02_physics_ontology.md` ONT-ISS-001）

```rust
fn write_state_batch(store: &mut GraphStore, states: &[StateWrite]) -> Result<(), GraphError> {
    if states.len() > BULK_STATE_WRITE_THRESHOLD {
        return Err(GraphError::BulkStateWriteRejected {
            count: states.len(), threshold: BULK_STATE_WRITE_THRESHOLD,
            hint: "per-frame State 属于 Runtime/Snapshot 存储, 而非 Graph — 见 03_graph_schema.md §4",
        });
    }
    for s in states { store.upsert_state_node(s)?; }
    Ok(())
}
```

这是 `02_physics_ontology.md` §Review 中 ONT-ISS-001 所述缺失的具体执行——单帧批量 `State` 写入现在是被拒绝的 `GraphError`，而非仅是文档化的规则。`15_testing_strategy.md` §15.4(b) 的测试断言此拒绝；该测试现在可以针对真实代码编写，按其自身的关闭条件关闭 ONT-ISS-001。

### 7.3 物理编译器算法

```rust
fn compile(store: &GraphStore, entity: NodeId) -> Result<PhysicsProfile, CompileError> {
    let material = follow_edge(store, entity, RelationKind::HasMaterial)
        .ok_or(CompileError::MissingRequiredEdge("HAS_MATERIAL"))?;
    let model = follow_edge(store, entity, RelationKind::ModeledBy)
        .ok_or(CompileError::MissingRequiredEdge("MODELED_BY"))?;

    let mass       = read_property_f32(store, material, PropertyKind::Mass)?;
    let density    = read_property_f32(store, material, PropertyKind::Density)?;
    let friction   = read_property_f32(store, material, PropertyKind::Friction).unwrap_or(DEFAULT_FRICTION);
    let restitution= read_property_f32(store, material, PropertyKind::Restitution).unwrap_or(DEFAULT_RESTITUTION);
    // ... 其余 PhysicsProfile 字段遵循 read_property_f32-with-fallback 模式

    let solver_type = match node_kind_attr(store, model, "kind") {
        "RigidBodyModel" => SolverTypeId::SequentialImpulse,
        "PBDModel" | "XPBDModel" => SolverTypeId::Xpbd,
        other => return Err(CompileError::UnsupportedModel(other.to_string())),
    };

    Ok(PhysicsProfile { mass, density, friction, restitution, solver_type, /* ... */ ..Default::default() })
}
```

每个字段读取都通过 `read_property_f32`，其内部以 `max_depth = 1` 调用 `bounded_traverse`（§21.2）（直接属性查找，Compiler 不需多跳）——Compiler 永远不执行无界图搜索，与 `03_graph_schema.md` §6 的承诺一致：GVPE 所需的任何查询模式都不需要无界搜索，包括编译期。

`CompileError::UnsupportedModel` 是 `04_architecture.md` §4.5 的 Law→Model→Solver 追溯表在运行时的强制执行方式——声明一个 `PhysicalModel` 节点类型却在该表中没有对应 Solver 条目会导致编译显式失败，而非静默生成无意义的 `PhysicsProfile`。

### 7.4 编译往返保证（实现 `15_testing_strategy.md` §15.5）

```rust
#[test]
fn compiled_profile_matches_hand_constructed() {
    let store = populate_test_graph(KNOWN_MATERIAL_FIXTURE);
    let compiled = compile(&store, ENTITY_ID).unwrap();
    let manual = PhysicsProfile { mass: 1.0, density: 1.0, friction: 0.5, restitution: 0.3, /* ... */ ..Default::default() };
    assert_eq!(compiled, manual);
}
```

`PhysicsProfile`（`17_detailed_design.md` §1.2）特意派生 `PartialEq` 以使该断言可行——这是 §1.2 原定义未显式但 AC-03 可在代码中检查（而非仅在文档中断言）所必需的细节。

## 8. 错误处理

- `GraphError::BulkStateWriteRejected`：写入守卫在批量超门槛时返回（`02_physics_ontology.md` ONT-ISS-001 关闭）
- `CompileError::MissingRequiredEdge`：必要边（`HAS_MATERIAL`、`MODELED_BY`）缺失
- `CompileError::UnsupportedModel`：模型节点类型无对应 Solver 条目
- `GraphError`（其他子类型）：节点/边不存在、属性缺失等
- `CompileError` 与 `InitError` 类型隔离，遵循 `04号文書` §4.4 的边界

## 9. 性能考量

- HashMap 后备路径：保证 O(1) 单点查询，O(deg) 邻居查询
- 受限遍历：硬上限深度，避免指数级展开
- `write_state_batch` 门槛：拦截高频写，保护图谱不被每帧状态淹没
- 编译期：仅 1 跳属性查找，无多跳查询
- 节点类型闭枚举：编译器可优化为整数 tag 比较
- 性能预算：见 `14_performance_budget.md`

## 10. 测试考量

- 编译往返测试（§21.5）：编译结果与手构结果等值
- 写入守卫测试：`15_testing_strategy.md` §15.4(b) 断言拒绝超门槛批量
- 受限遍历边界：达到 max_depth 时正确停止
- 条件边：条件为真时边生效，为假时跳过
- `UnsupportedModel` 错误路径：未注册模型被显式拒绝
- 缺失必要边：`HAS_MATERIAL` / `MODELED_BY` 缺失时返回错误
- 后端无关性：相同图状态经 HashMap 与（未来）外部 DB 编译结果一致

## 11. 关联需求

| 需求 ID | 中文描述 | 满足位置 |
|---|---|---|
| GVPE-GPH-001 | 物理知识图谱的存在与查询 | §21.1–§21.2 |
| GVPE-GPH-002 | 节点类型受本体约束 | §21.1（NodeKind 闭枚举） |
| GVPE-GPH-003 | 写入路径守卫 | §21.3 |
| GVPE-FR-003 | 编译器与运行时边界 | §21.4（CompileError 类型隔离） |
| AC-03 | 编译产物可验证 | §21.5（PartialEq 派生） |
| ONT-ISS-001 | 关闭本体层面的状态写入问题 | §21.3（写入守卫） |
| `01_requirements.md` §11 | MVP 不含 gvpe-graph/gvpe-compiler 运行时验收 | 全文 |
| `02号文書` §1 | 本体节点类型作为唯一真相 | §21.1 |
| `02号文書` §22 | 条件关系支持 | §21.2（条件边） |
| `03_graph_schema.md` §2 | 节点种类决策规则 | §21.1（类型层提升） |
| `03_graph_schema.md` §4 | 状态属于 Runtime/Snapshot | §21.3（错误提示） |
| `03_graph_schema.md` §6 | 无无界深度 | §21.2（受限遍历） |
| `04号文書` §4.4 | Compiler/Runtime 边界 | §21.4（错误类型隔离） |
| `04号文書` §4.5 | Law→Model→Solver 追溯 | §21.4（UnsupportedModel） |
| `15号文書` §15.4(b) | 写入守卫测试 | §21.3 |
| `15号文書` §15.5 | 编译往返保证 | §21.5 |
| `16号文書` §16.4 | 后端选型归属 | §21.1/§21.6 |

## 12. 关联文档

- 上游：`03_graph_schema.md`、`04_architecture.md` §4.4/§4.5、`17_detailed_design.md` §11（接口占位）、`01_requirements.md` §11（MVP 范围）、`02_physics_ontology.md` §1/§22/§Review（ONT-ISS-001）、`15_testing_strategy.md` §15.4(b)/§15.5、`16_dependency_license.md` §16.4
- 下游：`10_ffi_design.md`（若 FFI 暴露编译器 API）
- 平行：`17_detailed_design.md` §11（接口级定义）、`08_PRE_Detailed_Design.md`（归档的 `traverse()` 设计参考）

## 13. 审批签字

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 编写 | — | — | — |
| 校对 | — | — | — |
| 审批 | — | — | — |

---

## 14. 正文

> Input baseline: `03_graph_schema.md`, `04_architecture.md` §4.4, `17_detailed_design.md` §11
> (interface-only). This document is where §11's placeholder traits get real internals — required for
> "all detailed design complete", even though `gvpe-graph`/`gvpe-compiler` remain outside the MVP
> runtime-acceptance gate (`01_requirements.md` §11).

## 21.1 存储引擎（后备路径，per `16_dependency_license.md` §16.4）

```rust
struct GraphStore {
    nodes: HashMap<NodeId, Node>,
    edges: HashMap<NodeId, Vec<Edge>>,          // 邻接表, 以源节点为键
    property_index: HashMap<(NodeId, PropertyKind), NodeId>,   // Entity -> Property 快速查找
}

struct Node { id: NodeId, kind: NodeKind, attrs: HashMap<String, AttrValue> }
struct Edge { to: NodeId, relation: RelationKind, condition: Option<Condition> }

enum NodeKind {   // 与 02_physics_ontology.md §1 的顶层概念 1:1 完全对齐
    Entity, Matter, Material, Phase, Property, State, Force, Interaction, Constraint, Field,
    Energy, Wave, Process, PhysicalLaw, ConstitutiveModel, ApproximationModel, SolverModel,
    BoundaryCondition, Observation, Experiment, Hypothesis, Simulation, SimulationState,
    PhysicsProfileNode, VectorDescriptor, Result,
}
```

`NodeKind` 是闭枚举，恰好枚举 `02_physics_ontology.md` §1 的列表——添加不在该本体中的节点类型会导致编译错误，这是存储层将本体作为"Node 是什么"的唯一真相的执行机制（将 `03_graph_schema.md` §2 的决策规则提升到类型层，而不仅是设计指南）。

## 21.2 受限遍历查询（实现 `03_graph_schema.md` §6）

```rust
fn bounded_traverse(store: &GraphStore, start: NodeId, max_depth: u32,
                      relation_filter: Option<&dyn Fn(&Edge) -> bool>) -> Vec<NodeId> {
    let mut visited = HashSet::from([start]);
    let mut frontier = vec![start];
    for _ in 0..max_depth {          // 硬上限, 此函数中不存在无界深度代码路径
        let mut next = Vec::new();
        for node in &frontier {
            for edge in store.edges.get(node).into_iter().flatten() {
                if let Some(f) = relation_filter { if !f(edge) { continue; } }
                if edge.condition.as_ref().is_some_and(|c| !c.currently_holds(store)) { continue; }
                if visited.insert(edge.to) { next.push(edge.to); }
            }
        }
        if next.is_empty() { break; }
        frontier = next;
    }
    visited.into_iter().collect()
}
```

这是 `08_PRE_Detailed_Design.md` 归档的 `traverse()`（`docs/archive/`）在 graph-store 层的孪生——PRE 工作为自身图构建功能建立的深度限制纪律在此以相同形态重申，因为它是与项目本体无关的正确模式。条件边（§21.1 的 `Edge.condition`）实现 `02_physics_ontology.md` §22 的「条件关系支持」要求（如 `HigherTemperature DECREASES Viscosity` 仅在条件为真时成立）。

## 21.3 写入路径守卫（关闭 `02_physics_ontology.md` ONT-ISS-001）

```rust
fn write_state_batch(store: &mut GraphStore, states: &[StateWrite]) -> Result<(), GraphError> {
    if states.len() > BULK_STATE_WRITE_THRESHOLD {
        return Err(GraphError::BulkStateWriteRejected {
            count: states.len(), threshold: BULK_STATE_WRITE_THRESHOLD,
            hint: "per-frame State 属于 Runtime/Snapshot 存储, 而非 Graph — 见 03_graph_schema.md §4",
        });
    }
    for s in states { store.upsert_state_node(s)?; }
    Ok(())
}
```

这是 `02_physics_ontology.md` §Review 中 ONT-ISS-001 所述缺失的具体执行——单帧批量 `State` 写入现在是被拒绝的 `GraphError`，而非仅是文档化的规则。`15_testing_strategy.md` §15.4(b) 的测试断言此拒绝；该测试现在可以针对真实代码编写，按其自身的关闭条件关闭 ONT-ISS-001。

## 21.4 物理编译器算法

```rust
fn compile(store: &GraphStore, entity: NodeId) -> Result<PhysicsProfile, CompileError> {
    let material = follow_edge(store, entity, RelationKind::HasMaterial)
        .ok_or(CompileError::MissingRequiredEdge("HAS_MATERIAL"))?;
    let model = follow_edge(store, entity, RelationKind::ModeledBy)
        .ok_or(CompileError::MissingRequiredEdge("MODELED_BY"))?;

    let mass       = read_property_f32(store, material, PropertyKind::Mass)?;
    let density    = read_property_f32(store, material, PropertyKind::Density)?;
    let friction   = read_property_f32(store, material, PropertyKind::Friction).unwrap_or(DEFAULT_FRICTION);
    let restitution= read_property_f32(store, material, PropertyKind::Restitution).unwrap_or(DEFAULT_RESTITUTION);
    // ... 其余 PhysicsProfile 字段遵循 read_property_f32-with-fallback 模式

    let solver_type = match node_kind_attr(store, model, "kind") {
        "RigidBodyModel" => SolverTypeId::SequentialImpulse,
        "PBDModel" | "XPBDModel" => SolverTypeId::Xpbd,
        other => return Err(CompileError::UnsupportedModel(other.to_string())),
    };

    Ok(PhysicsProfile { mass, density, friction, restitution, solver_type, /* ... */ ..Default::default() })
}
```

每个字段读取都通过 `read_property_f32`，其内部以 `max_depth = 1` 调用 `bounded_traverse`（§21.2）（直接属性查找，Compiler 不需多跳）——Compiler 永远不执行无界图搜索，与 `03_graph_schema.md` §6 的承诺一致：GVPE 所需的任何查询模式都不需要无界搜索，包括编译期。

`CompileError::UnsupportedModel` 是 `04_architecture.md` §4.5 的 Law→Model→Solver 追溯表在运行时的强制执行方式——声明一个 `PhysicalModel` 节点类型却在该表中没有对应 Solver 条目会导致编译显式失败，而非静默生成无意义的 `PhysicsProfile`。

## 21.5 编译往返保证（实现 `15_testing_strategy.md` §15.5）

```rust
#[test]
fn compiled_profile_matches_hand_constructed() {
    let store = populate_test_graph(KNOWN_MATERIAL_FIXTURE);
    let compiled = compile(&store, ENTITY_ID).unwrap();
    let manual = PhysicsProfile { mass: 1.0, density: 1.0, friction: 0.5, restitution: 0.3, /* ... */ ..Default::default() };
    assert_eq!(compiled, manual);
}
```

`PhysicsProfile`（`17_detailed_design.md` §1.2）特意派生 `PartialEq` 以使该断言可行——这是 §1.2 原定义未显式但 AC-03 可在代码中检查（而非仅在文档中断言）所必需的细节。

## 21.6 显式非范围（刻意推迟，非遗漏）

后端选型（§21.1 的内嵌手写 store vs 外部图 DB）是 `16_dependency_license.md` 的决定，而非本文档——§21.1 的 `HashMap` 后备实现是该决定保证无论许可证审查结论如何都仍可用的具体回退方案。

Requirements satisfied: GVPE-GPH-001/002/003, GVPE-FR-003, AC-03, closes ONT-ISS-001.
