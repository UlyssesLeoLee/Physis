//! `gvpe-solver`：GVPE 约束求解器。
//!
//! 依据 [`GVPE-DOC-07`]（docs/02_modules/07_solver_design.md）：
//! - **MVP 第一世代**：Sequential Impulse / PGS（自研）。
//! - 统一行格式 [`ConstraintRow`]：所有约束（接触、摩擦、未来关节）展开为该格式。
//! - Warm-starting：上一帧 `lambda` 作本帧初值（`lambda` 跨帧保留）。
//! - 摩擦：Coulomb 摩擦锥 → box 近似（`FrictionConfig` + [`crate::friction::update_friction_bounds`]）。
//! - Sleeping：连续 N 帧速度低于阈值 → [`crate::body::SleepState::Sleeping`]。
//!
//! [`GVPE-DOC-07`]: ../../../docs/02_modules/07_solver_design.md
//!
//! ## MVP 显式不做
//!
//! - XPBD（post-MVP；`ConstraintRow.compliance` 字段已预留）。
//! - 关节（除验证 `ConstraintRow` 抽象的最小关节外）。
//! - 连续碰撞检测（CCD，per §6.5）。
//!
//! ## 求解流程
//!
//! ```text
//! warm-start (use previous lambda)
//!   → N × GS sweep over rows within island
//!   → project impulse bounds each sweep
//!   → integrate (apply accumulated impulse, force/torque, then position/rotation)
//!   → tick sleeping
//! ```
//!
//! ## 模块
//!
//! - [`body`]：刚体状态 [`RigidBody`] + 池 [`BodySlab`] + sleep 状态 [`SleepState`]。
//! - [`constraint`]：单行 [`ConstraintRow`]。
//! - [`solver`]：求解器主入口 [`Solver`] + [`Island`] + [`SolverConfig`]。
//! - [`friction`]：Coulomb box 近似。
//! - [`sleep`]：sleeping 状态机 + [`SleepConfig`]。
//! - [`error`]：求解器错误 [`SolverError`]。

#![deny(unsafe_op_in_unsafe_fn, missing_debug_implementations, missing_docs)]
#![warn(non_ascii_idents)]

mod body;
mod constraint;
mod error;
mod friction;
mod sleep;
mod solver;

pub use body::{BodySlab, RigidBody, SleepState};
pub use constraint::ConstraintRow;
pub use error::SolverError;
pub use friction::{
    build_friction_rows, friction_bounds, tangent_pair, update_friction_bounds, FrictionConfig,
};
pub use sleep::{force_sleep, tick_sleep, wake_up, SleepConfig};
pub use solver::{effective_mass, j_dot_v, Island, Solver, SolverConfig};

#[cfg(test)]
mod tests;
