//! `gvpe-solver` 错误类型。

use thiserror::Error;

/// 求解器错误。
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SolverError {
    /// 引用的 body 句柄在 slab 中找不到（use-after-free 或未分配）。
    #[error("body handle {index}:{generation} 在 slab 中未找到（use-after-free 或未分配）")]
    BodyNotFound {
        /// 句柄 index。
        index: u32,
        /// 句柄 generation。
        generation: u32,
    },

    /// 约束行引用的 body 句柄非法。
    #[error("约束行 body 句柄 {0:?} 非法（无对应 body）")]
    InvalidConstraintBody(gvpe_core::BodyHandle),

    /// 约束 Jacobian 退化（`J * M^{-1} * J^T ≈ 0`，如两 body 质量均为 0）。
    #[error("约束 Jacobian 退化（effective_mass = inf）")]
    DegenerateJacobian,

    /// 求解器配置非法（迭代次数 = 0 等）。
    #[error("求解器配置非法：{0}")]
    InvalidConfig(&'static str),
}
