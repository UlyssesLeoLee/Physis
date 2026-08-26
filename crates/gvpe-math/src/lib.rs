//! `gvpe-math`：自研向量 / 四元数 / 矩阵 / 几何原语。
//!
//! 依据 `GVPE-DOC-26` §18.5 选型决策：自研数学库，**不**使用 glam / nalgebra 等第三方库。
//! 依据 `GVPE-DOC-58` 数据布局图谱：所有公开类型为 POD-friendly（`bytemuck::Pod`），
//! 默认 scalar 实现，可选 `simd` feature 切换 vendor intrinsics。
//!
//! ## 类型概览
//!
//! | 类型 | 大小（字节） | 对齐 | 用途 |
//! |---|---|---|---|
//! | [`Vec3`] | 12 | 4 | 三维向量 |
//! | [`Quat`] | 16 | 16 | 四元数（SIMD 友好对齐） |
//! | [`Mat3`] | 36 | 4 | 3x3 矩阵（行优先） |
//! | [`Transform`] | 32 | 16 | 平移 + 旋转（compact） |
//! | [`Aabb`] | 24 | 4 | 轴对齐包围盒 |
//!
//! ## 性能原则
//!
//! - 所有操作 `#[inline]`，允许 LLVM 内联 + 常量折叠；
//! - 标量回退默认（无 SIMD 依赖），`simd` feature 启用 vendor intrinsics（见 `GVPE-DOC-26` §18.5）；
//! - 不分配；所有输入输出按值传递（类型都很小）。
//!
//! ## 未来扩展
//!
//! - `Mat4`（用于投影 / 视锥，post-MVP 视需要）；
//! - `simd` feature 启用后添加 `Vec3x4` SIMD-friendly bundle。

#![deny(unsafe_op_in_unsafe_fn, missing_debug_implementations, missing_docs)]
#![warn(non_ascii_idents)]

mod aabb;
mod mat3;
mod quat;
mod transform;
mod vec3;

pub use aabb::Aabb;
pub use mat3::Mat3;
pub use quat::Quat;
pub use transform::Transform;
pub use vec3::Vec3;

#[cfg(test)]
mod tests;
