//! [`Graph<N, E>`] 容器：节点 + 边的内存存储。
//!
//! ## MVP 存储选择：`Vec`-backed（**已知缺口**）
//!
//! - **节点**：按 `NodeId` 顺序存于 `Vec<(NodeId, N)>`。`NodeId` 唯一性由
//!   `add_node` 校验；按 ID 查找用 `linear_search`（`O(n)`）。
//! - **邻接表**：每个节点的 outgoing edge id 列表 + 邻接 set。`add_edge` 时
//!   双侧（src / dst）同步登记。
//! - **理由**：MVP 仅 ~10² 节点规模（编译器阶段组装），`O(n)` 查找可接受；
//!   上 `petgraph` 会引入依赖 + 抽象开销（PKG 节点 / 边带自定义 payload，
//!   `petgraph::Graph<N, E>` 的索引/句柄体系与本 crate 设计冲突）。
//! - **未来升级路径**：当节点规模进入 10³ 阈值或需子图提取时，替换为
//!   `HashMap<NodeId, (N, SmallVec<[EdgeId; 4]>)>` + 反向索引；trait 表面
//!   **不**变。
//!
//! ## 三类图分离（DEC-002 / QA-D-02）
//!
//! [`Graph`] 内部不导入 [`gvpe_core::BodyHandle`] / [`gvpe_core::ConstraintHandle`]，
//! 因此与 `gvpe-runtime` / `gvpe-solver` 隔离。节点 / 边 trait **不**继承
//! `PkqEntity` / `RuntimeConstraintEntity`——本 trait 由 [`crate::Node`] 隐式
//! 实现（PKG 节点身份由 [`crate::Node::id`] 表达），但**不**与 Runtime
//! Constraint Graph / Execution Graph 共享任何类型构造。
//!
//! ## 自环 / 多重边策略
//!
//! - **自环（src == dst）**：**允许**（本体论允许 `Entity → 自己` 表达"同一
//!   实体的多重属性引用"）。`DFS` / `BFS` 必须显式防御自环导致无限循环
//!   —— 由 [`crate::GraphWalker`] / [`crate::GraphTraverser`] 的 `visited`
//!   集合保证。
//! - **多重边（src/dst/relation 全等）**：**允许**（本体论允许 `Entity A
//!   → B` 多重出现承载不同条件限定符）。`Graph::edges_between(src, dst)`
//!   返回**所有**边。

use crate::edge::{Edge, EdgeId};
use crate::error::{GraphError, GraphResult};
use crate::node::{Node, NodeId};

/// 图统计信息（用于诊断 / benchmark）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GraphStats {
    /// 节点数量。
    pub node_count: usize,
    /// 边数量。
    pub edge_count: usize,
}

/// PKG 图容器。
///
/// 类型参数：
/// - `N`：节点 payload（实现 [`Node`]）；
/// - `E`：边 payload（实现 [`Edge`]）。
///
/// ## 不变量
///
/// - `nodes` 中 `NodeId` **互异**；
/// - `edges` 中 `EdgeId` **互异**；
/// - 每条 `edge` 的 `src` / `dst` 必须在 `nodes` 中存在。
///
/// 不变量由 `add_node` / `add_edge` 维护；`Graph` **不**对外暴露可变迭代器
/// —— 添加通过 `add_*`，移除通过 `remove_*`（MVP **不**提供 remove：PKG
/// 节点的"删除"语义由 schema 演化纪律定义，详见 `GVPE-DOC-03` §9.2）。
#[derive(Debug)]
pub struct Graph<N: Node, E: Edge> {
    /// 节点存储：`(NodeId, N)`，保持插入顺序。
    nodes: Vec<(NodeId, N)>,
    /// 边存储：`(EdgeId, E)`，保持插入顺序。
    edges: Vec<(EdgeId, E)>,
    /// 出邻接表：`NodeId -> [EdgeId]`。
    outgoing: Vec<(NodeId, Vec<EdgeId>)>,
    /// 入邻接表：`NodeId -> [EdgeId]`。
    incoming: Vec<(NodeId, Vec<EdgeId>)>,
}

impl<N: Node, E: Edge> Default for Graph<N, E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<N: Node, E: Edge> Graph<N, E> {
    /// 新建空图。
    #[inline]
    pub const fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            outgoing: Vec::new(),
            incoming: Vec::new(),
        }
    }

    /// 节点数量。
    #[inline]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// 边数量。
    #[inline]
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// 统计信息（一次性快照）。
    #[inline]
    pub fn stats(&self) -> GraphStats {
        GraphStats {
            node_count: self.node_count(),
            edge_count: self.edge_count(),
        }
    }

    /// 图是否为空。
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.edges.is_empty()
    }

    /// 添加节点。`node.id()` 必须唯一；否则返回 [`GraphError::DuplicateNode`]。
    pub fn add_node(&mut self, node: N) -> GraphResult<NodeId> {
        let id = node.id();
        if self.find_node_index(id).is_some() {
            return Err(GraphError::DuplicateNode(id));
        }
        self.nodes.push((id, node));
        self.outgoing.push((id, Vec::new()));
        self.incoming.push((id, Vec::new()));
        Ok(id)
    }

    /// 添加边。端点必须存在；ID 必须唯一。
    pub fn add_edge(&mut self, edge: E) -> GraphResult<EdgeId> {
        let id = edge.id();
        if self.find_edge_index(id).is_some() {
            return Err(GraphError::DuplicateEdge(id));
        }
        let src = edge.src();
        let dst = edge.dst();
        // 端点存在性校验（端点 ID 必须在 `nodes` 中）。
        self.ensure_node(src, "src")?;
        self.ensure_node(dst, "dst")?;
        // 邻接表登记。
        self.outgoing_mut(src)?.push(id);
        self.incoming_mut(dst)?.push(id);
        self.edges.push((id, edge));
        Ok(id)
    }

    /// 节点是否存在。
    #[inline]
    pub fn contains_node(&self, id: NodeId) -> bool {
        self.find_node_index(id).is_some()
    }

    /// 边是否存在。
    #[inline]
    pub fn contains_edge(&self, id: EdgeId) -> bool {
        self.find_edge_index(id).is_some()
    }

    /// 取节点 payload 引用。
    pub fn node_payload(&self, id: NodeId) -> GraphResult<&N> {
        let idx = self
            .find_node_index(id)
            .ok_or(GraphError::UnknownNode(id))?;
        Ok(&self.nodes[idx].1)
    }

    /// 取边 payload 引用。
    pub fn edge_payload(&self, id: EdgeId) -> GraphResult<&E> {
        let idx = self
            .find_edge_index(id)
            .ok_or(GraphError::UnknownEdge(id))?;
        Ok(&self.edges[idx].1)
    }

    /// 迭代所有 `(NodeId, &N)`。
    #[inline]
    pub fn nodes(&self) -> impl Iterator<Item = (NodeId, &N)> {
        self.nodes.iter().map(|(id, n)| (*id, n))
    }

    /// 迭代所有 `(EdgeId, &E)`。
    #[inline]
    pub fn edges(&self) -> impl Iterator<Item = (EdgeId, &E)> {
        self.edges.iter().map(|(id, e)| (*id, e))
    }

    /// 节点 `src` 的出边 ID 列表。
    pub fn outgoing_edges(&self, src: NodeId) -> GraphResult<&[EdgeId]> {
        let idx = self
            .find_outgoing_index(src)
            .ok_or(GraphError::UnknownNode(src))?;
        Ok(&self.outgoing[idx].1)
    }

    /// 节点 `dst` 的入边 ID 列表。
    pub fn incoming_edges(&self, dst: NodeId) -> GraphResult<&[EdgeId]> {
        let idx = self
            .find_incoming_index(dst)
            .ok_or(GraphError::UnknownNode(dst))?;
        Ok(&self.incoming[idx].1)
    }

    /// 节点 `src` 的邻居 ID 列表（去重，按首次出现顺序）。
    ///
    /// "邻居" = 通过任一出边可达的 `dst` 节点（含 `src == dst` 自环）。
    pub fn neighbors(&self, src: NodeId) -> GraphResult<Vec<NodeId>> {
        let out = self.outgoing_edges(src)?;
        let mut seen = vec![false; self.nodes.len()];
        let mut result = Vec::new();
        for edge_id in out {
            let e = self.edge_payload(*edge_id)?;
            if let Some(idx) = self.find_node_index(e.dst()) {
                if !seen[idx] {
                    seen[idx] = true;
                    result.push(e.dst());
                }
            }
        }
        Ok(result)
    }

    // ---------------------------------------------------------------
    // 私有 helpers
    // ---------------------------------------------------------------

    fn find_node_index(&self, id: NodeId) -> Option<usize> {
        self.nodes.iter().position(|(nid, _)| *nid == id)
    }

    fn find_edge_index(&self, id: EdgeId) -> Option<usize> {
        self.edges.iter().position(|(eid, _)| *eid == id)
    }

    fn find_outgoing_index(&self, id: NodeId) -> Option<usize> {
        self.outgoing.iter().position(|(nid, _)| *nid == id)
    }

    fn find_incoming_index(&self, id: NodeId) -> Option<usize> {
        self.incoming.iter().position(|(nid, _)| *nid == id)
    }

    fn ensure_node(&self, id: NodeId, which: &'static str) -> GraphResult<()> {
        if self.find_node_index(id).is_none() {
            return Err(GraphError::UnknownEndpoint(id, which));
        }
        Ok(())
    }

    fn outgoing_mut(&mut self, id: NodeId) -> GraphResult<&mut Vec<EdgeId>> {
        let idx = self
            .find_outgoing_index(id)
            .ok_or(GraphError::UnknownNode(id))?;
        Ok(&mut self.outgoing[idx].1)
    }

    fn incoming_mut(&mut self, id: NodeId) -> GraphResult<&mut Vec<EdgeId>> {
        let idx = self
            .find_incoming_index(id)
            .ok_or(GraphError::UnknownNode(id))?;
        Ok(&mut self.incoming[idx].1)
    }
}

// ---------------------------------------------------------------------------
// `pub(crate)` 访问 helper：仅供 crate 内 `walker` / `tests` 模块使用
// ---------------------------------------------------------------------------

impl<N: Node, E: Edge> Graph<N, E> {
    /// `NodeId` → 在内部 `nodes` Vec 中的索引（crate 内可见）。
    ///
    /// `walker` 的 DFS / BFS visited-set 用 `Vec<bool>` 按索引 O(1) 标记，
    /// 需要这个快速索引方法。
    #[inline]
    pub(crate) fn node_index(&self, id: NodeId) -> Option<usize> {
        self.find_node_index(id)
    }
}

// ============================================================================
// 测试用构造函数：test-only 便捷添加
// ============================================================================
//
// (MVP 不在 `Graph` 上提供 remove / clear：PKG 节点的"删除"语义
// 由 schema 演化纪律定义，详见 `GVPE-DOC-03` §9.2。crate 内部如需
// reset，可由测试模块直接构造新 `Graph` 实例。)
