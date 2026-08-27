//! `gvpe-graph`：物理知识图谱（PKG）的内存存储与查询层。
//!
//! 依据 `GVPE-DOC-03`（基本設計書 §`03_graph_schema.md`）：
//!
//! - **三类图分离**：本 crate **只**承载 PKG（持久化、高语义、来源/置信度可追溯）。
//!   Runtime Constraint Graph（`gvpe-island` / `gvpe-constraint`）与 Execution Graph
//!   （`gvpe-scheduler`）**不**通过本 crate 暴露任何 API，编译期由
//!   [`gvpe_core::PkqEntity`] 标记 trait 阻断混用（DEC-002 / QA-D-02）。
//! - **禁止热路径查询**（GVPE-GPH-003）：本 crate 是离线 / 编译器阶段使用，
//!   不面向 [`gvpe_runtime`] / [`gvpe_solver`] 热路径（`GVPE-DOC-03` §7.1）。
//! - **有界遍历**（`GVPE-DOC-03` §9.1 / `GVPE-DOC-02` §25）：BFS / DFS 走
//!   [`GraphWalker::bounded_dfs`] / [`GraphTraverser::bounded_bfs`] 时**必须**显式
//!   提供 `max_depth` 上限；无界遍历不在 API 表面（API 名称自带 `bounded_` 前缀强制）。
//!
//! ## MVP 范围
//!
//! - 节点 / 边 trait + 通用 [`Graph<N, E>`] 容器（`Vec`-backed，详见 module
//!   [`graph`] 的"复杂度选择"说明）。
//! - 关系词表 [`RelationKind`] 是 `GVPE-DOC-02` §22 词表的一个小子集（MVP）；
//!   完整词表随 schema 演化按 `GVPE-DOC-03` §9.2 纪律增补。
//! - 不包含持久化后端（`GVPE-DOC-03` §5.3：选型待定）。
//!
//! ## 已知缺口（per commit body DDD Review 必查项）
//!
//! - 容器实现选 `Vec` 而非 `petgraph`（详见 [`graph`] module doc 已知缺口段）。
//! - 节点 / 边 trait **不**要求 `bytemuck::Pod`（PKG 数据是高语义，非热路径 POD，
//!   与 `gvpe-core` 的 `PhysicsProfile` 不同）。
//! - 不提供 Cypher 类查询语言（不在 MVP 范围；编译器走 trait-bounded API）。
//! - 与未来 `gvpe-collision` / `gvpe-scheduler` 集成的 trait 预留：
//!   通过 [`graph::Graph::node_payload`] / [`graph::Graph::edge_payload`]
//!   的类型 `N: Node` / `E: Edge` 关联类型 **绑定**，未在本 crate 内硬编码耦合。

#![deny(unsafe_op_in_unsafe_fn, missing_debug_implementations, missing_docs)]
#![warn(non_ascii_idents)]

mod edge;
mod error;
mod graph;
mod node;
mod walker;

pub use edge::{Edge, EdgeId, RelationKind};
pub use error::{GraphError, GraphResult};
pub use graph::{Graph, GraphStats};
pub use node::{Node, NodeId};
pub use walker::{BfsOrder, ComponentId, DfsOrder, GraphTraverser, GraphWalker, TraversalEvent};

#[cfg(test)]
mod tests;
