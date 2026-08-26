//! `gvpe-collision`：GVPE 碰撞检测模块。
//!
//! 依据 `GVPE-DOC-06`（06_collision_design.md）：
//!
//! - **粗筛**：[`broad_phase`] —— MVP 选 Sweep-and-Prune（SAP），沿 X 轴 sweep + 3D 二次检查。
//! - **精筛**：[`narrow_phase::gjk`] —— GJK（Minkowski difference + 单纯形迭代）；
//!   [`epa::epa`] —— 穿透深度（与 GJK 配对）。
//! - **接触流形**：[`ContactManifold`] / [`ContactPoint`]（per §7.1）。
//!
//! ## 与 `06_collision_design.md` 的偏差
//!
//! | 维度 | design doc MVP 选型 | v0.7 实现 | 备注 |
//! |---|---|---|---|
//! | 精筛算法 | SAT（§6.3） | **GJK + EPA** | brief 显式要求 GJK+EPA；SAT 留 v0.8 补全 |
//! | Plane 形状 | MVP（§6.2） | **跳过** | Plane 非闭合，GJK 不适用；落 post-MVP |
//! | Convex Hull | post-MVP（§6.2） | **v0.7 落地** | point cloud → 隐式凸包 MVP 实现 |
//! | Multi-point manifold | — | **单点** | design doc §7.1 示例用 `SmallVec<[ContactPoint; 4]>`；本 crate v0.7 用 `Vec` 简化 |
//!
//! 详见下方 `KNOWN_GAPS`。
//!
//! ## 不做（per `06_collision_design.md` §3 + `02_physics_ontology.md` §9）
//!
//! - 不消费 `ContactManifold` 的求解器行（属 `gvpe-constraint` crate，本 crate 仅产出）；
//! - 不写 `Constraint` 节点到图（运行时约束不是图节点）；
//! - 不做三角网 / Heightfield 精筛（post-MVP）；
//! - 不暴露 `broad_phase` 之外的内部状态（SAP 的 active list / EPA 的 polytope 全部私有）。
//!
//! ## 安全政策
//!
//! - 无 `unsafe` 代码（GJK / EPA / SAP 全部 safe + 标量数学）；
//! - 所有公开类型实现 `Debug`（`workspace.lints` `missing_debug_implementations` 强制）；
//! - 所有公开项均文档化（`workspace.lints` `missing_docs` 强制）。
//!
//! ## 已知缺口（KNOWN_GAPS）— DDD Review 必查
//!
//! 1. **与 design doc §6.3 MVP 选型偏差**：brief 要求 GJK+EPA，design doc MVP 选 SAT。
//!    v0.7 选 GJK+EPA（brief 优先），SAT 在 v0.8 补全（Box-Box / Box-Plane / Sphere-Box
//!    路径仍可用 GJK 兜底，但 performance 差于 SAT 专路径）。
//! 2. **Plane 形状未实现**：非闭合体，GJK 不适用；post-MVP 由半空间测试实现。
//! 3. **Single-point manifold**：v0.7 manifold 限 1 接触点；v0.8 改 `SmallVec<[ContactPoint; 4]>`
//!    + clipping 生成多接触点。
//! 4. **`gvpe-memory` 暂未实际消费**：broad/narrow 内部用 `Vec` 暂存；v0.8+ 集成
//!    `Arena` / `Pool` 做"零分配热路径"（`gvpe-memory` 依赖仍声明以备扩展）。
//! 5. **未与 `gvpe-core::ShapeDesc` 桥接**：本 crate 自有 [`Shape`] 枚举（需 OBB 旋转 +
//!    凸包点云，`gvpe-core::ShapeDesc` 不含这些）；v0.8+ `gvpe-shape` 落地后写适配器。
//! 6. **未做 miri 测试**（CI 强制 miri：见 `gvpe-memory` §0）；本 crate 全 safe 代码，
//!    v0.8+ 在 miri stable 后补全。
//! 7. **未做性能基准**（`criterion` 已在 workspace deps 但本 crate 未引入 `[[bench]]`）；
//!    SAP / GJK / EPA 性能基线留 v0.8+。
//! 8. **确定性模式 `deterministic` feature 占位**（与 `gvpe-memory` 对齐，DEC-006）；
//!    本 crate v0.7 内部已按 IEEE 754 严格运算（不依赖排序稳定性的浮点路径），
//!    feature 实质效果留 v0.8+ 实测。

#![deny(unsafe_op_in_unsafe_fn, missing_debug_implementations, missing_docs)]
#![warn(non_ascii_idents)]
// 与 gvpe-memory / gvpe-ffi 对齐：
// - `missing_const_for_fn`: 同 crate 1.75 MSRV，`Vec::new()` 等未稳定为 const。
// - `module_name_repetitions`: workspace 全局 allow。
#![allow(clippy::missing_const_for_fn, clippy::module_name_repetitions)]

mod broad_phase;
mod epa;
mod manifold;
mod narrow_phase;
mod shape;

pub use broad_phase::{BodyIndex, broad_phase};
pub use epa::{PenetrationInfo, epa};
pub use manifold::{ContactManifold, ContactPoint};
pub use narrow_phase::{GjkResult, gjk};
pub use shape::Shape;

#[cfg(test)]
mod tests;
