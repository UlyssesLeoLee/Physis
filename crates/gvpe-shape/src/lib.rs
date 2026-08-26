//! `gvpe-shape`：基础 + 凸形状描述、AABB 计算、Shape 抽象、ShapeHandle 共享。
//!
//! 依据 `GVPE-DOC-21` §1（基础 shape 详细设计）与 `GVPE-DOC-20`（高级 shape 详细设计）的分工：
//!
//! - 本 crate（`GVPE-DOC-21`）覆盖 **基础形状** `Sphere` / `Box3` / `Capsule` / `Plane` / `ConvexHull` 5 种，
//!   提供 `Shape` trait + `ShapeHandle`（`Arc<dyn Shape>` 共享）+ 局部坐标 `Aabb` 计算。
//! - 高级形状 `TriangleMesh` / `Heightfield` / `Compound` 及其 BVH 在 `GVPE-DOC-20` 详细设计。
//!
//! ## 类型概览
//!
//! | 类型 | 角色 | 备注 |
//! |---|---|---|
//! | [`Shape`] (trait) | 形状抽象 | 局部坐标 AABB + 类型 tag |
//! | [`ShapeHandle`] | 共享句柄 | `Arc<dyn Shape>`，克隆廉价 |
//! | [`ShapeType`] (enum) | 类型 tag | 派发与 FFI 边界 |
//! | [`Sphere`] | 球 | |
//! | [`Box3`] | 盒 | 名字带 `3` 后缀避开 `box` 关键字 |
//! | [`Capsule`] | 胶囊 | 中间圆柱 + 两端半球，轴向 = `+Y` |
//! | [`Plane`] | 无限平面 | 局部 AABB 退化为有限保守值（见类型文档） |
//! | [`ConvexHull`] | 凸包 | `Arc<[Vec3]>` 共享资产 |
//!
//! ## 性能原则
//!
//! - 局部坐标 AABB 在形状创建时（构造时一次性计算）缓存，broad phase 直接读取。
//! - `ShapeHandle` 的 `Arc` clone 本身不在热路径上（形状不逐帧重建）；热路径上仅读取。
//! - `ConvexHull` 的 `Arc<[Vec3]>` 允许多 body 共享同一组顶点资产（参考 20 号文書 §6.1 资产式分配策略）。
//!
//! ## 与 gvpe-core 的关系
//!
//! `gvpe-core::ShapeDesc` 是**占位 enum**（仅 MVP 三种，见 `crates/gvpe-core/src/descriptor.rs:15` 注释），
//! 待本 crate 稳定后由 gvpe-core 反向依赖并 re-export 完整版（详见 `GVPE-DOC-21` §6 关联需求）。

#![deny(unsafe_op_in_unsafe_fn, missing_debug_implementations, missing_docs)]
#![warn(non_ascii_idents)]

mod box_shape;
mod capsule;
mod convex;
mod plane;
mod shape;
mod sphere;

#[cfg(test)]
mod tests;

pub use box_shape::Box3;
pub use capsule::Capsule;
pub use convex::{ConvexError, ConvexHull};
pub use plane::Plane;
pub use shape::{Shape, ShapeHandle, ShapeType};
pub use sphere::Sphere;
