//! `gvpe-graph` 集成测试。
//!
//! 覆盖：
//! 1. 空图状态
//! 2. 节点添加 / 重复 ID 拒绝
//! 3. 边添加 / 端点不存在拒绝
//! 4. 邻接表 / 邻居查询
//! 5. 自环 / 多重边
//! 6. DFS preorder
//! 7. BFS 层次
//! 8. 有界遍历深度上限
//! 9. DFS 短路回调
//! 10. 连通分量

use crate::{
    BfsOrder, DfsOrder, Edge, EdgeId, Graph, GraphError, GraphTraverser, GraphWalker, Node, NodeId,
    RelationKind,
};

// ============================================================================
// 测试用 Node / Edge impl（最小化字段以隔离测试）
// ============================================================================

#[derive(Clone, Debug)]
struct TestNode {
    id: NodeId,
    name: &'static str,
}

impl Node for TestNode {
    fn id(&self) -> NodeId {
        self.id
    }
    fn label(&self) -> &'static str {
        self.name
    }
}

#[derive(Clone, Debug)]
struct TestEdge {
    id: EdgeId,
    src: NodeId,
    dst: NodeId,
    rel: RelationKind,
}

impl Edge for TestEdge {
    fn id(&self) -> EdgeId {
        self.id
    }
    fn relation(&self) -> RelationKind {
        self.rel
    }
    fn src(&self) -> NodeId {
        self.src
    }
    fn dst(&self) -> NodeId {
        self.dst
    }
    fn label(&self) -> &'static str {
        "test-edge"
    }
}

fn nid(v: u64) -> NodeId {
    NodeId::new(v)
}

fn eid(v: u64) -> EdgeId {
    EdgeId::new(v)
}

// ============================================================================
// 1. 空图状态
// ============================================================================

#[test]
fn empty_graph_state() {
    let g: Graph<TestNode, TestEdge> = Graph::new();
    assert_eq!(g.node_count(), 0);
    assert_eq!(g.edge_count(), 0);
    assert!(g.is_empty());
    let stats = g.stats();
    assert_eq!(stats.node_count, 0);
    assert_eq!(stats.edge_count, 0);
    assert!(!g.contains_node(nid(1)));
}

// ============================================================================
// 2. 节点添加 + 重复 ID 拒绝
// ============================================================================

#[test]
fn add_node_basic_and_duplicate_rejected() {
    let mut g: Graph<TestNode, TestEdge> = Graph::new();
    let id = g
        .add_node(TestNode {
            id: nid(1),
            name: "alpha",
        })
        .expect("add_node(1)");
    assert_eq!(id, nid(1));
    assert_eq!(g.node_count(), 1);

    // 重复 ID 拒绝。
    let dup = g.add_node(TestNode {
        id: nid(1),
        name: "alpha-dup",
    });
    assert!(matches!(dup, Err(GraphError::DuplicateNode(_))));
    assert_eq!(g.node_count(), 1);

    // 第二个不同 ID。
    g.add_node(TestNode {
        id: nid(2),
        name: "beta",
    })
    .expect("add_node(2)");
    assert_eq!(g.node_count(), 2);
}

// ============================================================================
// 3. 边添加 + 端点不存在拒绝
// ============================================================================

#[test]
fn add_edge_basic_and_endpoint_rejected() {
    let mut g: Graph<TestNode, TestEdge> = Graph::new();
    g.add_node(TestNode {
        id: nid(1),
        name: "a",
    })
    .unwrap();
    g.add_node(TestNode {
        id: nid(2),
        name: "b",
    })
    .unwrap();

    // 正常添加。
    g.add_edge(TestEdge {
        id: eid(100),
        src: nid(1),
        dst: nid(2),
        rel: RelationKind::HasProperty,
    })
    .expect("add_edge(100)");
    assert_eq!(g.edge_count(), 1);

    // 边 ID 重复拒绝。
    let dup = g.add_edge(TestEdge {
        id: eid(100),
        src: nid(2),
        dst: nid(1),
        rel: RelationKind::Generic,
    });
    assert!(matches!(dup, Err(GraphError::DuplicateEdge(_))));

    // 端点不存在拒绝（src）。
    let bad_src = g.add_edge(TestEdge {
        id: eid(101),
        src: nid(999),
        dst: nid(1),
        rel: RelationKind::Generic,
    });
    assert!(matches!(
        bad_src,
        Err(GraphError::UnknownEndpoint(id, "src")) if id == nid(999)
    ));

    // 端点不存在拒绝（dst）。
    let bad_dst = g.add_edge(TestEdge {
        id: eid(102),
        src: nid(1),
        dst: nid(888),
        rel: RelationKind::Generic,
    });
    assert!(matches!(
        bad_dst,
        Err(GraphError::UnknownEndpoint(id, "dst")) if id == nid(888)
    ));
}

// ============================================================================
// 4. 邻接表 / 邻居 / 自环 / 多重边
// ============================================================================

#[test]
fn adjacency_neighbors_self_loop_multi_edge() {
    let mut g: Graph<TestNode, TestEdge> = Graph::new();
    for i in 1u64..=3 {
        g.add_node(TestNode {
            id: nid(i),
            name: "n",
        })
        .unwrap();
    }
    // 1 → 2, 1 → 3, 1 → 1 (自环), 2 → 1 (反向), 2 → 3 (多重)
    g.add_edge(TestEdge {
        id: eid(10),
        src: nid(1),
        dst: nid(2),
        rel: RelationKind::HasProperty,
    })
    .unwrap();
    g.add_edge(TestEdge {
        id: eid(11),
        src: nid(1),
        dst: nid(3),
        rel: RelationKind::HasProperty,
    })
    .unwrap();
    g.add_edge(TestEdge {
        id: eid(12),
        src: nid(1),
        dst: nid(1),
        rel: RelationKind::Generic,
    })
    .unwrap(); // 自环
    g.add_edge(TestEdge {
        id: eid(13),
        src: nid(2),
        dst: nid(1),
        rel: RelationKind::Generic,
    })
    .unwrap();
    g.add_edge(TestEdge {
        id: eid(14),
        src: nid(2),
        dst: nid(3),
        rel: RelationKind::Generic,
    })
    .unwrap(); // 多重边（2→3 多条留作下个测试）

    // node 1 出边 3 条：10, 11, 12
    let out1 = g.outgoing_edges(nid(1)).unwrap();
    assert_eq!(out1.len(), 3);
    assert!(out1.contains(&eid(10)));
    assert!(out1.contains(&eid(11)));
    assert!(out1.contains(&eid(12)));

    // node 1 邻居去重 = {1, 2, 3}（自环 1 包含）
    let nb1 = g.neighbors(nid(1)).unwrap();
    assert_eq!(nb1.len(), 3);
    assert!(nb1.contains(&nid(1)));
    assert!(nb1.contains(&nid(2)));
    assert!(nb1.contains(&nid(3)));

    // node 2 入边 1 条
    let in2 = g.incoming_edges(nid(2)).unwrap();
    assert_eq!(in2.len(), 1);
    assert_eq!(in2[0], eid(10));
}

// ============================================================================
// 5. 多重边：2→3 重复
// ============================================================================

#[test]
fn multi_edge_between_same_pair_allowed() {
    let mut g: Graph<TestNode, TestEdge> = Graph::new();
    for i in 1u64..=2 {
        g.add_node(TestNode {
            id: nid(i),
            name: "n",
        })
        .unwrap();
    }
    g.add_edge(TestEdge {
        id: eid(1),
        src: nid(1),
        dst: nid(2),
        rel: RelationKind::HasProperty,
    })
    .unwrap();
    g.add_edge(TestEdge {
        id: eid(2),
        src: nid(1),
        dst: nid(2),
        rel: RelationKind::Generic,
    })
    .unwrap();
    assert_eq!(g.edge_count(), 2);
    let out1 = g.outgoing_edges(nid(1)).unwrap();
    assert_eq!(out1.len(), 2);
    // 邻居去重后只 1 个
    let nb = g.neighbors(nid(1)).unwrap();
    assert_eq!(nb, vec![nid(2)]);
}

// ============================================================================
// 6. DFS preorder
// ============================================================================

#[test]
fn dfs_preorder_collects_reachable_nodes() {
    // 链：1 → 2 → 3；从 1 出发 DFS 深度 10，应得 [1, 2, 3]
    let mut g: Graph<TestNode, TestEdge> = Graph::new();
    for i in 1u64..=3 {
        g.add_node(TestNode {
            id: nid(i),
            name: "n",
        })
        .unwrap();
    }
    g.add_edge(TestEdge {
        id: eid(1),
        src: nid(1),
        dst: nid(2),
        rel: RelationKind::HasProperty,
    })
    .unwrap();
    g.add_edge(TestEdge {
        id: eid(2),
        src: nid(2),
        dst: nid(3),
        rel: RelationKind::HasProperty,
    })
    .unwrap();

    let order: DfsOrder = g.bounded_dfs_collect(nid(1), 10).expect("dfs");
    let nodes: Vec<NodeId> = order.nodes().collect();
    assert_eq!(nodes, vec![nid(1), nid(2), nid(3)]);
    assert_eq!(order.visited_count, 3);
}

// ============================================================================
// 7. BFS 层次
// ============================================================================

#[test]
fn bfs_layers() {
    // 树：
    //      1
    //     / \
    //    2   3
    //   / \
    //  4   5
    let mut g: Graph<TestNode, TestEdge> = Graph::new();
    for i in 1u64..=5 {
        g.add_node(TestNode {
            id: nid(i),
            name: "n",
        })
        .unwrap();
    }
    g.add_edge(TestEdge {
        id: eid(1),
        src: nid(1),
        dst: nid(2),
        rel: RelationKind::HasProperty,
    })
    .unwrap();
    g.add_edge(TestEdge {
        id: eid(2),
        src: nid(1),
        dst: nid(3),
        rel: RelationKind::HasProperty,
    })
    .unwrap();
    g.add_edge(TestEdge {
        id: eid(3),
        src: nid(2),
        dst: nid(4),
        rel: RelationKind::HasProperty,
    })
    .unwrap();
    g.add_edge(TestEdge {
        id: eid(4),
        src: nid(2),
        dst: nid(5),
        rel: RelationKind::HasProperty,
    })
    .unwrap();

    let bfs: BfsOrder = g.bounded_bfs_collect(nid(1), 10).expect("bfs");
    assert_eq!(bfs.layers.len(), 3, "depth 0, 1, 2 共三层");
    assert_eq!(bfs.layers[0], vec![nid(1)]);
    assert_eq!(bfs.layers[1].len(), 2);
    assert!(bfs.layers[1].contains(&nid(2)));
    assert!(bfs.layers[1].contains(&nid(3)));
    assert_eq!(bfs.layers[2].len(), 2);
    assert!(bfs.layers[2].contains(&nid(4)));
    assert!(bfs.layers[2].contains(&nid(5)));
}

// ============================================================================
// 8. 有界遍历深度上限
// ============================================================================

#[test]
fn bounded_traversal_depth_cap() {
    // 链 1 → 2 → 3 → 4；max_depth=2 应只访问 1 和 2
    let mut g: Graph<TestNode, TestEdge> = Graph::new();
    for i in 1u64..=4 {
        g.add_node(TestNode {
            id: nid(i),
            name: "n",
        })
        .unwrap();
    }
    for (i, (s, d)) in [(1u64, 2u64), (2, 3), (3, 4)].iter().enumerate() {
        g.add_edge(TestEdge {
            id: eid((i + 1) as u64),
            src: nid(*s),
            dst: nid(*d),
            rel: RelationKind::HasProperty,
        })
        .unwrap();
    }
    let dfs: DfsOrder = g.bounded_dfs_collect(nid(1), 2).expect("dfs");
    let nodes: Vec<NodeId> = dfs.nodes().collect();
    assert_eq!(nodes, vec![nid(1), nid(2)], "max_depth=2 应停在 2");

    let bfs: BfsOrder = g.bounded_bfs_collect(nid(1), 2).expect("bfs");
    assert_eq!(bfs.layers.len(), 2);
    assert_eq!(bfs.layers[0], vec![nid(1)]);
    assert_eq!(bfs.layers[1], vec![nid(2)]);
}

// ============================================================================
// 9. max_depth=0 拒绝
// ============================================================================

#[test]
fn zero_max_depth_rejected() {
    let mut g: Graph<TestNode, TestEdge> = Graph::new();
    g.add_node(TestNode {
        id: nid(1),
        name: "n",
    })
    .unwrap();
    let r1 = g.bounded_dfs(nid(1), 0, |_| true);
    assert!(matches!(r1, Err(GraphError::ZeroMaxDepth(0))));
    let r2 = g.bounded_bfs(nid(1), 0, |_| true);
    assert!(matches!(r2, Err(GraphError::ZeroMaxDepth(0))));
}

// ============================================================================
// 10. DFS 短路回调
// ============================================================================

#[test]
fn dfs_callback_short_circuit() {
    let mut g: Graph<TestNode, TestEdge> = Graph::new();
    for i in 1u64..=3 {
        g.add_node(TestNode {
            id: nid(i),
            name: "n",
        })
        .unwrap();
    }
    g.add_edge(TestEdge {
        id: eid(1),
        src: nid(1),
        dst: nid(2),
        rel: RelationKind::HasProperty,
    })
    .unwrap();
    g.add_edge(TestEdge {
        id: eid(2),
        src: nid(1),
        dst: nid(3),
        rel: RelationKind::HasProperty,
    })
    .unwrap();
    let mut count = 0usize;
    g.bounded_dfs(nid(1), 10, |event| {
        count += 1;
        // 访问到节点 2 后短路（不继续兄弟 3）
        event.node_id != nid(2)
    })
    .expect("dfs");
    assert_eq!(count, 2, "应访问 root + 节点 2 即停");
}

// ============================================================================
// 11. 连通分量
// ============================================================================

#[test]
fn connected_components_basic() {
    // 三个分量：
    //   A: 1 → 2
    //   B: 3 → 4
    //   C: 5 (孤点)
    let mut g: Graph<TestNode, TestEdge> = Graph::new();
    for i in 1u64..=5 {
        g.add_node(TestNode {
            id: nid(i),
            name: "n",
        })
        .unwrap();
    }
    g.add_edge(TestEdge {
        id: eid(1),
        src: nid(1),
        dst: nid(2),
        rel: RelationKind::HasProperty,
    })
    .unwrap();
    g.add_edge(TestEdge {
        id: eid(2),
        src: nid(3),
        dst: nid(4),
        rel: RelationKind::HasProperty,
    })
    .unwrap();
    let comps = g.connected_components();
    assert_eq!(comps.len(), 3);
    let mut sizes: Vec<usize> = comps.iter().map(|(_, vs)| vs.len()).collect();
    sizes.sort_unstable();
    assert_eq!(sizes, vec![1, 2, 2]);
}

// ============================================================================
// 12. self-loop 防御：DFS 不会无限循环
// ============================================================================

#[test]
fn dfs_does_not_loop_on_self_edge() {
    let mut g: Graph<TestNode, TestEdge> = Graph::new();
    for i in 1u64..=2 {
        g.add_node(TestNode {
            id: nid(i),
            name: "n",
        })
        .unwrap();
    }
    g.add_edge(TestEdge {
        id: eid(1),
        src: nid(1),
        dst: nid(1),
        rel: RelationKind::Generic,
    })
    .unwrap();
    g.add_edge(TestEdge {
        id: eid(2),
        src: nid(1),
        dst: nid(2),
        rel: RelationKind::HasProperty,
    })
    .unwrap();
    let dfs: DfsOrder = g.bounded_dfs_collect(nid(1), 10).expect("dfs");
    let nodes: Vec<NodeId> = dfs.nodes().collect();
    // 期望 [1, 2]——自环被 visited 拦截
    assert_eq!(nodes, vec![nid(1), nid(2)]);
}

// ============================================================================
// 13. 不存在的 start 节点
// ============================================================================

#[test]
fn traversal_with_unknown_start_rejected() {
    let g: Graph<TestNode, TestEdge> = Graph::new();
    let r1 = g.bounded_dfs_collect(nid(999), 5);
    assert!(matches!(r1, Err(GraphError::UnknownNode(_))));
    let r2 = g.bounded_bfs_collect(nid(999), 5);
    assert!(matches!(r2, Err(GraphError::UnknownNode(_))));
}
