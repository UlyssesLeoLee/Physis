//! `gvpe-scheduler`：GVPE 任务图与 4 阶段调度。
//!
//! 依据 `docs/01_architecture/09_parallel_design.md`（GVPE-DOC-09）§6.2 / §6.4 / §9.1 / §9.2。
//!
//! ## MVP 范围（per `09_parallel_design.md` §6.4）
//!
//! - **单线程顺序调度** —— 调用 [`Scheduler::run`] 时严格按
//!   `Predict → Collision → Solver → Integrate` 顺序在当前线程串行执行。
//! - **Job 句柄** [`JobHandle`] 与 **Worker 池抽象** [`WorkerPool`] 已定义 trait 与
//!   顺序实现，作为 v0.4+ 真并行（rayon / tokio / crossbeam）阶段的占位与
//!   契约锚点。当前实现不创建任何 OS 线程。
//!
//! ## 阶段依赖图（隐式，由 [`Scheduler::run`] 顺序固化）
//!
//! ```text
//!   Predict ──▶ Collision ──▶ Solver ──▶ Integrate
//! ```
//!
//! `09_parallel_design.md` §6.2 给出的 Job DAG（`BroadPhase → NarrowPhase[] →
//! IslandBuild → SolveIsland[] → Integrate[]`）是**并行 fan-out** 视角的细化形态；
//! 本 crate v0.3 暴露的 4 阶段是 **stage 粒度抽象** —— 调用方在自己的
//! `PredictStage` / `CollisionStage` / `SolverStage` / `IntegrateStage` 实现内
//! 决定是否 fan-out 到 island。这是 v0.3 → v0.4 演进的契约边界。
//!
//! ## 已知缺口（per 任务授权 2026-08-27）
//!
//! - 真实并行（rayon / tokio / crossbeam）留 v0.4+。
//! - Job 调度 = 顺序执行（MVP），无 work stealing（per `09_parallel_design.md` §6.3）。
//! - 阶段间并行可能性 = **草案**，未实施。
//! - criterion 性能基准 = **未做**。
//! - miri = **未做**。

#![deny(unsafe_op_in_unsafe_fn, missing_debug_implementations, missing_docs)]
#![warn(non_ascii_idents)]

mod job;
mod pool;
mod scheduler;
mod stage;

pub use job::{JobHandle, JobId, JobKind};
pub use pool::{SequentialPool, WorkerPool, WorkerPoolBuilder, max_concurrent_stages};
pub use scheduler::{ScheduleError, Scheduler, SchedulerStats};
pub use stage::{
    CollisionStage, IntegrateStage, PredictStage, SolverStage, Stage, StageKind, StageOutput,
};

#[cfg(test)]
mod tests;
