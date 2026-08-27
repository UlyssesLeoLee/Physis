//! `gvpe-dynamics`：GVPE 刚体动力学。
//!
//! 依据（**基线缺位**，DDD Review 必查）：
//!
//! - `docs/02_modules/` **当前不存在** `*dynamics*_design.md`，本 crate 落地
//!   时 v0.3 design doc 尚未编写（per 2026-08-27 `git ls-files` 实证：
//!   `docs/02_modules/` 仅有 `06_collision_design.md` / `07_solver_design.md`
//!   等 13 个文件，**无** dynamics 相关）。
//! - `04_architecture.md` §4.5 提及 dynamics 模块（**未经实测引用**，
//!   DDD Review 必查）。
//!
//! 本 crate 实现 v0.3 MVP 刚体动力学主体：
//!
//! - [`RigidBodyState`] —— 平移 / 旋转 / 线速度 / 角速度 / 力 / 力矩 / 派生量
//! - [`TimeStepper`] —— 半隐式 Euler（默认）/ 显式 Euler / RK4
//! - [`crate::force::apply_gravity`] / [`crate::force::apply_damping`] —— 力源
//! - [`step::predict`] / [`step::integrate`] / [`step::finalize`] —— 三阶段
//! - [`DynamicsWorld`] —— 主控制器
//!
//! ## Crate 级 lint 允许说明
//!
//! - `clippy::doc_markdown`：模块级 doc 不需要严格 markdown 反引号
//! - `clippy::module_name_repetitions`：workspace 全局 allow
//! - `clippy::missing_const_for_fn`：1.75 MSRV 兼容
//!
//! ## 不做（v0.3 范围外）
//!
//! - 接触约束求解（属 `gvpe-constraint` / `gvpe-solver`）
//! - Island 重建 / 接触图 / 睡眠机制
//! - 陀螺力（`ω × Iω`）、空气阻力
//! - `RuntimeDescriptor` 桥接（v0.4+ 通过 `BodySpec` 注入）
//!
//! ## 安全政策
//!
//! - 无 `unsafe` 代码；
//! - 所有公开类型实现 `Debug`（`workspace.lints` `missing_debug_implementations` 强制）；
//! - 所有公开项均文档化（`workspace.lints` `missing_docs` 强制）。
//!
//! ## 已知缺口（KNOWN_GAPS）— DDD Review 必查
//!
//! 1. **基线 design doc 缺位**：`docs/02_modules/` 无 dynamics design doc
//!    （创建 commit 时 `git ls-files 'docs/02_modules/' | Select-String 'dynamics'`
//!    返回空，已 `git log -p --follow` 实证）。v0.3 选型与算法选型见本 crate
//!    注释与 [`TimeStepper`] 表格，**待 DDD Review 拍板**。
//! 2. **算法选型为草案**：半隐式 Euler vs RK4 vs Velocity Verlet 无 design doc
//!    实证；MVP 默认半隐式 Euler（能量稳定 + 简单）。DDD Review 必查
//!    是否有性能 / 精度 / 兼容性硬约束要求切到 RK4 / Velocity Verlet。
//! 3. **力累加器不含陀螺力**：高速旋转刚体（`ω > 100 rad/s` 量级）会出现
//!    数值漂移；v0.4+ 评估是否需要 `τ_gyro = ω × (I ω)`。
//! 4. **未做 miri 测试**：本 crate 全 safe 代码，v0.8+ miri stable 后补全。
//! 5. **未做性能基准**：`criterion` 已声明（dev-dep）但本 crate 未引入
//!    `[[bench]]`；per-step 热路径性能基线留 v0.4+。
//! 6. **`deterministic` feature 占位**：与 `gvpe-core` / `gvpe-collision` 对齐
//!    （DEC-006），feature 实质效果留 v0.4+ 实测。
//! 7. **测试覆盖边界**：
//!    - 高速穿透（`v * dt > body_radius`）—— 未覆盖
//!    - 数值爆炸（大力 + 长时间累积）—— 未覆盖
//!    - 大力冲击（`F ≫ m * g`）—— 仅覆盖恒定力
//! 8. **未桥接 `gvpe-core::RuntimeDescriptor`**：本 crate 当前仅支持
//!    `spawn_dynamic` / `spawn_fixed` 直接构造 body；批量场景加载
//!    `add_body` 接口留 v0.4+。
//! 9. **未提供 `BodySpec` → `spawn_*` 适配器**：调用方需手动从
//!    `BodySpec.profile` / `BodySpec.initial_transform` 抽字段。

#![deny(unsafe_op_in_unsafe_fn, missing_debug_implementations, missing_docs)]
#![warn(non_ascii_idents)]
#![allow(clippy::missing_const_for_fn, clippy::module_name_repetitions)]
#![allow(clippy::doc_markdown)]

mod error;
mod force;
mod state;
mod step;
mod world;

pub use error::{DynamicsError, DynamicsResult};
pub use force::{apply_damping, apply_gravity};
pub use state::{RigidBodyState, TimeStepper};
pub use step::{
    finalize, integrate, integrate_explicit, integrate_rk4, integrate_semi_implicit, predict,
    predict_with_damping, validate_dt,
};
pub use world::DynamicsWorld;

#[cfg(test)]
mod tests;
