//! [`Box3`]：轴对齐盒（中心在局部原点）。

use gvpe_math::{Aabb, Vec3};

use crate::shape::{Shape, ShapeType};

/// 轴对齐盒。
///
/// 名字带 `3` 后缀避开 Rust 关键字 `box`（与 `gvpe-core::ShapeDesc::Box3` 保持一致）。
/// 局部坐标 AABB：`[-hx, -hy, -hz]` × `[+hx, +hy, +hz]`。
/// **OBB（旋转后）**由 gvpe-collision 在拿到 transform 后用 8 顶点 + SAT 计算。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Box3 {
    /// 半尺寸 `(hx, hy, hz)`。
    pub half_extents: [f32; 3],
}

impl Box3 {
    /// 构造新盒。
    #[inline]
    #[must_use]
    pub const fn new(half_extents: [f32; 3]) -> Self {
        Self { half_extents }
    }

    /// 三轴相同半尺寸的立方盒。
    #[inline]
    #[must_use]
    pub fn cube(half_extent: f32) -> Self {
        Self::new([half_extent, half_extent, half_extent])
    }
}

impl Shape for Box3 {
    #[inline]
    fn shape_type(&self) -> ShapeType {
        ShapeType::Box3
    }

    #[inline]
    fn local_aabb(&self) -> Aabb {
        let he = Vec3::new(self.half_extents[0], self.half_extents[1], self.half_extents[2]);
        Aabb::new(-he, he)
    }
}
