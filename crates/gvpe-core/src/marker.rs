//! 标记 trait：三类图（PKG / Runtime Constraint Graph / Execution Graph）的类型层分离。
//!
//! 依据 DEC-002 + `QA-D-02`：避免实现期通过 `pub use` 间接拉通三类图身份。
//!
//! ## 用法
//!
//! ```ignore
//! // PKG 中的 entity 类型
//! pub struct MaterialNode { /* ... */ }
//! impl PkqEntity for MaterialNode { /* marker */ }
//!
//! // Runtime Constraint Graph 中的 entity 类型
//! pub struct ConstraintRow { /* ... */ }
//! impl RuntimeConstraintEntity for ConstraintRow { /* marker */ }
//!
//! // 编译期阻止混用：
//! fn process_pkq<T: PkqEntity>(t: T) { /* ... */ }
//! fn process_rcg<T: RuntimeConstraintEntity>(t: T) { /* ... */ }
//! // process_pkq(constraint_row)  // 编译错误！
//! ```

/// PKG（物理知识图谱）实体的标记 trait。
///
/// PKG 节点 = 长期持久化、高语义、高连通性、来源 / 置信度可追溯的实体。
/// 依据 `GVPE-DOC-02` + `GVPE-DOC-03`。
pub trait PkqEntity {
    /// 节点 ID 类型（与 [`crate::BodyHandle`] 不同；通常为 `NodeId`）。
    type Id: Copy + Eq + core::hash::Hash;
}

/// Runtime Constraint Graph 实体的标记 trait。
///
/// Runtime Constraint Graph 节点 = per-frame 在内存中的约束行。
/// 依据 `GVPE-DOC-03` §1。
pub trait RuntimeConstraintEntity {
    /// 句柄类型（与 [`crate::BodyHandle`] 不同；通常为 `ConstraintHandle`）。
    type Handle: Copy + Eq + core::hash::Hash;
}

/// Execution Graph 实体的标记 trait。
///
/// Execution Graph 节点 = 任务 DAG 中的 job。
/// 依据 `GVPE-DOC-03` §1 + `GVPE-DOC-09`。
pub trait ExecutionEntity {
    /// Job ID 类型。
    type Id: Copy + Eq + core::hash::Hash;
}

// 注：trait **不**实现任何方法，仅作为类型层 marker；
// `pub use` 重导出不会改变类型的 marker 身份，编译期阻止混用。
