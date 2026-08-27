//! 图遍历抽象：DFS ([`GraphWalker`]) + BFS ([`GraphTraverser`])。
//!
//! 依据 `GVPE-DOC-03` §9.1：所有遍历**必须**是**有界**的（`max_depth` 上限）；
//! API 表面无"无界遍历"——`bounded_dfs` / `bounded_bfs` 命名自带 `bounded_`
//! 前缀强制。
//!
//! ## 遍历输出
//!
//! - DFS：返回 [`DfsOrder`]（preorder，按访问顺序）；
//! - BFS：返回 [`BfsOrder`]（按层，每层一行 + 完整访问顺序）；
//! - 通用：返回 [`TraversalEvent`] 流，可在遍历中途**短路**（`callback`
//!   返回 `false`）。
//!
//! ## 自环 / 多重边防御
//!
//! `visited` 集合基于 **节点 ID**（不是边 ID），自环 `A → A` 在第二步即被
//! `visited.contains(A)` 拦截。多重边不会引入重复访问。

use crate::edge::{Edge, EdgeId};
use crate::error::{GraphError, GraphResult};
use crate::graph::Graph;
use crate::node::{Node, NodeId};

/// 连通分量 ID（仅在无向视角下有意义；PKG 是有向图，仅作为"通过出边可达"分量的
/// 标签使用）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ComponentId(
    /// 分量编号（0-based，按首次发现顺序）。
    pub u32,
);

impl ComponentId {
    /// 构造。
    #[inline]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// 取出 `u32`。
    #[inline]
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl core::fmt::Display for ComponentId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ComponentId({})", self.0)
    }
}

/// 单次遍历事件（回调入参）。
///
/// ## `should_continue` 语义
///
/// 返回 `false` **立即短路**当前遍历分支（但不取消兄弟分支——DFS 兄弟
/// 节点仍会访问；如需全遍历取消，把 `visited` 全部置 `true` 即可，或
/// 用 [`GraphWalker::bounded_dfs_collect`] 一把跑完）。
#[derive(Clone, Copy, Debug)]
pub struct TraversalEvent<'a, N: Node, E: Edge> {
    /// 当前访问到的节点。
    pub node_id: NodeId,
    /// 从父节点到当前节点的边（根节点时为 `None`）。
    pub via_edge: Option<EdgeId>,
    /// 当前深度（根 = 0）。
    pub depth: usize,
    /// 节点 payload 引用。
    pub node: &'a N,
    /// 边 payload 引用（`via_edge` 非 `None` 时为 `Some`）。
    pub edge: Option<&'a E>,
}

impl<N: Node, E: Edge> TraversalEvent<'_, N, E> {
    /// 是否到达深度上限（`depth + 1 == max_depth`）。
    #[inline]
    pub const fn is_at_depth_limit(&self, max_depth: usize) -> bool {
        self.depth + 1 >= max_depth
    }
}

/// DFS 顺序记录（preorder 节点 ID 序列）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DfsOrder {
    /// preorder 访问序列。
    pub order: Vec<(NodeId, usize)>,
    /// 被访问节点总数（含 `start`）。
    pub visited_count: usize,
}

impl DfsOrder {
    /// 取出 preorder 节点 ID 序列（不含深度）。
    #[inline]
    pub fn nodes(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.order.iter().map(|(id, _)| *id)
    }
}

/// BFS 顺序记录：每层一行 + 完整顺序。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BfsOrder {
    /// 各层节点 ID 列表（`layers[0]` = 起点层）。
    pub layers: Vec<Vec<NodeId>>,
    /// 完整访问顺序（flatten）。
    pub order: Vec<(NodeId, usize)>,
}

impl BfsOrder {
    /// 取出 flat 节点 ID 序列（不含深度）。
    #[inline]
    pub fn nodes(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.order.iter().map(|(id, _)| *id)
    }
}

/// 图遍历器（trait）：通用 DFS / BFS 抽象。
///
/// 该 trait **不**依赖 [`Graph`] 的所有权——可对 `&Graph<N, E>` 调用，
/// 适配不同存储后端（未来切换 `HashMap` 存储时无须变更调用方）。
///
/// ## 已知缺口
///
/// - MVP 不暴露反向 DFS / 后序 DFS；
/// - 不暴露"沿指定 `RelationKind` 边过滤"（用户可在回调内自行检查
///   `event.edge.map(|e| e.relation())`）。
pub trait GraphTraverser<N: Node, E: Edge> {
    /// 有界 DFS（preorder）：从 `start` 出发，最多走 `max_depth` 层。
    ///
    /// 行为：
    /// - 起点深度 = 0；
    /// - 每条边跨 1 层；
    /// - `visited` 集合基于节点 ID 拦截回环 / 自环 / 多重边；
    /// - 回调返回 `false` 短路当前分支。
    ///
    /// # 错误
    ///
    /// - `start` 不在图中 → [`GraphError::UnknownNode`]；
    /// - `max_depth == 0` → [`GraphError::ZeroMaxDepth`]。
    fn bounded_dfs<F>(&self, start: NodeId, max_depth: usize, mut callback: F) -> GraphResult<()>
    where
        F: FnMut(TraversalEvent<'_, N, E>) -> bool,
    {
        if max_depth == 0 {
            return Err(GraphError::ZeroMaxDepth(0));
        }
        self.bounded_dfs_impl(start, max_depth, &mut callback)
    }

    /// 有界 BFS：从 `start` 出发，最多走 `max_depth` 层。
    fn bounded_bfs<F>(&self, start: NodeId, max_depth: usize, mut callback: F) -> GraphResult<()>
    where
        F: FnMut(TraversalEvent<'_, N, E>) -> bool,
    {
        if max_depth == 0 {
            return Err(GraphError::ZeroMaxDepth(0));
        }
        self.bounded_bfs_impl(start, max_depth, &mut callback)
    }

    /// DFS 实现（trait 默认方法委托；自定义存储后端可覆盖）。
    fn bounded_dfs_impl(
        &self,
        start: NodeId,
        max_depth: usize,
        callback: &mut dyn FnMut(TraversalEvent<'_, N, E>) -> bool,
    ) -> GraphResult<()>;

    /// BFS 实现。
    fn bounded_bfs_impl(
        &self,
        start: NodeId,
        max_depth: usize,
        callback: &mut dyn FnMut(TraversalEvent<'_, N, E>) -> bool,
    ) -> GraphResult<()>;
}

/// 有界 DFS 的"全收集"便捷方法。
///
/// `bounded_dfs` 不返回 order（回调式），`bounded_dfs_collect` 返回完整
/// [`DfsOrder`] 记录——大多数用例（编译器 / 离线工具）只需要这个。
pub trait GraphWalker<N: Node, E: Edge>: GraphTraverser<N, E> {
    /// DFS 收集完整 preorder。
    fn bounded_dfs_collect(&self, start: NodeId, max_depth: usize) -> GraphResult<DfsOrder> {
        let mut order = DfsOrder::default();
        self.bounded_dfs(start, max_depth, |event| {
            order.order.push((event.node_id, event.depth));
            order.visited_count += 1;
            true
        })?;
        Ok(order)
    }

    /// BFS 收集完整层序。
    fn bounded_bfs_collect(&self, start: NodeId, max_depth: usize) -> GraphResult<BfsOrder> {
        let mut order = BfsOrder::default();
        // BFS 阶段一：调用 bounded_bfs，按深度分发到 layers。
        // 注意：callback 接收的 event.depth 允许"一次访问跨越多深"是不可能的
        // （每条边 +1），所以 `event.depth` 一定 ∈ [0, max_depth) 且单调。
        let mut max_observed_depth = 0usize;
        self.bounded_bfs(start, max_depth, |event| {
            while order.layers.len() <= event.depth {
                order.layers.push(Vec::new());
            }
            order.layers[event.depth].push(event.node_id);
            order.order.push((event.node_id, event.depth));
            if event.depth > max_observed_depth {
                max_observed_depth = event.depth;
            }
            true
        })?;
        // 截断空尾层（防御性）。
        while order.layers.last().is_some_and(std::vec::Vec::is_empty) {
            order.layers.pop();
        }
        let _ = max_observed_depth; // 已通过 layers.len() - 1 表达
        Ok(order)
    }

    /// 计算无向视角下的连通分量集合（PKG 是有向图，但编译器 / 离线工具
    /// 经常用"通过出边可达"作为分量的近似）。
    ///
    /// 返回 `Vec<(ComponentId, Vec<NodeId>)>`，按首次发现节点 ID 升序排序。
    fn connected_components(&self) -> Vec<(ComponentId, Vec<NodeId>)> {
        let mut visited: Vec<NodeId> = Vec::new();
        let mut result: Vec<(ComponentId, Vec<NodeId>)> = Vec::new();
        // 收集所有节点 ID（不假设节点 ID 单调）。
        let all_nodes: Vec<NodeId> = self.all_node_ids();
        for &start in &all_nodes {
            if visited.contains(&start) {
                continue;
            }
            let cid = ComponentId::new(result.len() as u32);
            let mut component: Vec<NodeId> = Vec::new();
            // 用 max_depth = usize::MAX 上限仅作"全图遍历"占位（无界是实现
            // 细节，不暴露 API 表面）。bfs/dfs 内部 visited 集合仍保证
            // 有限性。
            let _ = self.bounded_dfs(start, usize::MAX, |event| {
                visited.push(event.node_id);
                component.push(event.node_id);
                true
            });
            result.push((cid, component));
        }
        result
    }

    /// 全图节点 ID 列表（按插入顺序）。
    fn all_node_ids(&self) -> Vec<NodeId>;
}

// ---------------------------------------------------------------------------
// `Graph<N, E>` 的 GraphTraverser / GraphWalker blanket impl
// ---------------------------------------------------------------------------

impl<N: Node, E: Edge> GraphTraverser<N, E> for Graph<N, E> {
    fn bounded_dfs_impl(
        &self,
        start: NodeId,
        max_depth: usize,
        callback: &mut dyn FnMut(TraversalEvent<'_, N, E>) -> bool,
    ) -> GraphResult<()> {
        if !self.contains_node(start) {
            return Err(GraphError::UnknownNode(start));
        }
        // 防御性：尽管 bounded_dfs 已校验 max_depth > 0，impl 再校验一次。
        if max_depth == 0 {
            return Err(GraphError::ZeroMaxDepth(0));
        }
        let mut visited: Vec<bool> = vec![false; self.node_count()];
        let start_idx = self
            .node_index(start)
            .ok_or(GraphError::UnknownNode(start))?;
        visited[start_idx] = true;
        // 根节点事件。
        let root_node = self.node_payload(start)?;
        let cont = callback(TraversalEvent {
            node_id: start,
            via_edge: None,
            depth: 0,
            node: root_node,
            edge: None,
        });
        if !cont || max_depth == 1 {
            return Ok(());
        }
        // 递归 DFS（栈深度上限 = max_depth，节点数 ≤ 10²，递归可接受；
        // 未来切迭代式可避免栈溢出）。
        self.dfs_recurse(start, 0, max_depth, &mut visited, callback)?;
        Ok(())
    }

    fn bounded_bfs_impl(
        &self,
        start: NodeId,
        max_depth: usize,
        callback: &mut dyn FnMut(TraversalEvent<'_, N, E>) -> bool,
    ) -> GraphResult<()> {
        if !self.contains_node(start) {
            return Err(GraphError::UnknownNode(start));
        }
        if max_depth == 0 {
            return Err(GraphError::ZeroMaxDepth(0));
        }
        let mut visited: Vec<bool> = vec![false; self.node_count()];
        let start_idx = self
            .node_index(start)
            .ok_or(GraphError::UnknownNode(start))?;
        visited[start_idx] = true;
        let root_node = self.node_payload(start)?;
        let cont = callback(TraversalEvent {
            node_id: start,
            via_edge: None,
            depth: 0,
            node: root_node,
            edge: None,
        });
        if !cont {
            return Ok(());
        }
        // 队列元素 = (NodeId, 当前深度)。
        let mut queue: std::collections::VecDeque<(NodeId, usize)> =
            std::collections::VecDeque::new();
        queue.push_back((start, 0));
        while let Some((current, depth)) = queue.pop_front() {
            if depth + 1 >= max_depth {
                continue;
            }
            for edge_id in self.outgoing_edges(current)? {
                let edge = self.edge_payload(*edge_id)?;
                let next = edge.dst();
                let Some(next_idx) = self.node_index(next) else {
                    // 防御：端点已删除（不应发生，但保险）
                    continue;
                };
                if visited[next_idx] {
                    continue;
                }
                visited[next_idx] = true;
                let next_node = self.node_payload(next)?;
                let cont = callback(TraversalEvent {
                    node_id: next,
                    via_edge: Some(*edge_id),
                    depth: depth + 1,
                    node: next_node,
                    edge: Some(edge),
                });
                if !cont {
                    return Ok(());
                }
                queue.push_back((next, depth + 1));
            }
        }
        Ok(())
    }
}

impl<N: Node, E: Edge> GraphWalker<N, E> for Graph<N, E> {
    fn all_node_ids(&self) -> Vec<NodeId> {
        // 通过公开 iterator 收集（避免直接访问私有字段）。
        self.nodes().map(|(id, _)| id).collect()
    }
}

impl<N: Node, E: Edge> Graph<N, E> {
    /// DFS 递归 helper：仅 [`GraphTraverser::bounded_dfs_impl`] 调用。
    fn dfs_recurse(
        &self,
        current: NodeId,
        depth: usize,
        max_depth: usize,
        visited: &mut [bool],
        callback: &mut dyn FnMut(TraversalEvent<'_, N, E>) -> bool,
    ) -> GraphResult<()> {
        if depth + 1 >= max_depth {
            return Ok(());
        }
        for edge_id in self.outgoing_edges(current)? {
            let edge = self.edge_payload(*edge_id)?;
            let next = edge.dst();
            let Some(next_idx) = self.node_index(next) else {
                continue;
            };
            if visited[next_idx] {
                continue;
            }
            visited[next_idx] = true;
            let next_node = self.node_payload(next)?;
            let cont = callback(TraversalEvent {
                node_id: next,
                via_edge: Some(*edge_id),
                depth: depth + 1,
                node: next_node,
                edge: Some(edge),
            });
            if !cont {
                return Ok(());
            }
            self.dfs_recurse(next, depth + 1, max_depth, visited, callback)?;
        }
        Ok(())
    }
}
