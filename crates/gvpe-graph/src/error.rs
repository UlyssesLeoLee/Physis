//! `gvpe-graph` 错误类型。
//!
//! MVP 仅含结构性错误（节点 / 边不变量违反）；持久化错误（与 `GVPE-DOC-16`
//! 选型决定的后端耦合）不在本 crate。

use thiserror::Error;

/// `gvpe-graph` 错误类型。
#[derive(Debug, Error)]
pub enum GraphError {
    /// `NodeId` 在 [`crate::Graph`] 中查不到对应节点（已被移除或从未添加）。
    #[error("node id 未知: {0:?}")]
    UnknownNode(crate::NodeId),

    /// `EdgeId` 在 [`crate::Graph`] 中查不到对应边（已被移除或从未添加）。
    #[error("edge id 未知: {0:?}")]
    UnknownEdge(crate::EdgeId),

    /// 边的源 / 目标端点引用未知节点。
    ///
    /// 元组字段：`(端点 NodeId, 端点身份标签 ∈ {"src", "dst"})`。
    #[error("edge 端点引用未知 node: {1} = {0:?}")]
    UnknownEndpoint(crate::NodeId, &'static str),

    /// 试图添加 ID 已被占用的节点。
    #[error("node id 已存在: {0:?}")]
    DuplicateNode(crate::NodeId),

    /// 试图添加 ID 已被占用的边。
    #[error("edge id 已存在: {0:?}")]
    DuplicateEdge(crate::EdgeId),

    /// 用户调用的 `max_depth == 0` 的有界遍历——这等价于"不访问任何节点"，
    /// 不可能产生有意义的 traversal，API 显式拒绝以避免误用。
    #[error("bounded traversal 的 max_depth 必须 > 0, 实为 {0}")]
    ZeroMaxDepth(usize),
}

/// `gvpe-graph` 使用的 `Result` 类型别名。
pub type GraphResult<T> = Result<T, GraphError>;
