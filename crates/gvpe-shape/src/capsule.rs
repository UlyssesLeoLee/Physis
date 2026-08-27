//! [`Capsule`]：胶囊（圆柱段 + 两端半球）。
//!
//! 局部坐标 **轴向 = `+Y`**（与 20 号文書 §6.1 `Capsule { radius, half_height }` 字段一致；
//! `half_height` 是中间圆柱段的半高——总长 = `2 * half_height + 2 * radius`）。
//!
//! 局部 AABB：`[-r, -(half_height + r), -r]` × `[+r, +(half_height + r), +r]`。

use gvpe_math::{Aabb, Vec3};

use crate::shape::{Shape, ShapeType};

/// 胶囊（中心在局部原点，轴向 = `+Y`）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Capsule {
    /// 圆柱半径。
    pub radius: f32,
    /// 中间圆柱段半高（**不含**半球端）。
    pub half_height: f32,
}

impl Capsule {
    /// 构造新胶囊。
    #[inline]
    #[must_use]
    pub const fn new(radius: f32, half_height: f32) -> Self {
        Self {
            radius,
            half_height,
        }
    }

    /// 总长（半球端在内）：`2 * half_height + 2 * radius`。
    #[inline]
    #[must_use]
    pub fn total_length(&self) -> f32 {
        2.0 * (self.half_height + self.radius)
    }
}

impl Shape for Capsule {
    #[inline]
    fn shape_type(&self) -> ShapeType {
        ShapeType::Capsule
    }

    #[inline]
    fn local_aabb(&self) -> Aabb {
        let r = self.radius;
        let h_extent = self.half_height + r;
        Aabb::new(Vec3::new(-r, -h_extent, -r), Vec3::new(r, h_extent, r))
    }
}
