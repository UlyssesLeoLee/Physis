//! [`Sphere`]：球体。

use gvpe_math::{Aabb, Vec3};

use crate::shape::{Shape, ShapeType};

/// 球体（中心在局部原点）。
///
/// 局部坐标 AABB：`[-r, -r, -r]` × `[+r, +r, +r]`。
/// 旋转无影响（球对称）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sphere {
    /// 半径。
    pub radius: f32,
}

impl Sphere {
    /// 构造新球。
    #[inline]
    #[must_use]
    pub const fn new(radius: f32) -> Self {
        Self { radius }
    }
}

impl Shape for Sphere {
    #[inline]
    fn shape_type(&self) -> ShapeType {
        ShapeType::Sphere
    }

    #[inline]
    fn local_aabb(&self) -> Aabb {
        let r = Vec3::splat(self.radius);
        Aabb::new(-r, r)
    }
}
