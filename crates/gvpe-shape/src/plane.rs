//! [`Plane`]：无限平面（半空间）。
//!
//! 严格意义上平面无界，**没有有限 AABB**。本实现按工程做法返回一个
//! "保守有限 AABB"（`[-PLANE_AABB_HALF_EXTENT, +PLANE_AABB_HALF_EXTENT]`
//! 立方体），供 broad phase 缓存用——所有真实平面物体都会被这个保守 AABB 覆盖，
//! broad phase 不会因为 AABB 退化而漏检（保守 = 不漏报）。
//!
//! 真正的 narrow phase 平面-凸体 SAT 派发由 gvpe-collision 负责（本 crate 只暴露资产）。
//!
//! ## 约定
//!
//! - `normal` 应已归一化（构造时不强制；`Shape::shape_type` 也不会重归一化）。
//! - `offset` = 平面到原点的有符号距离（`plane(n, d) = { p | dot(n, p) + d = 0 }`，
//!   与 `gvpe-core::ShapeDesc::Plane` 一致）。
//! - 局部 AABB 计算不依赖 `offset`——`offset` 仅描述"平面在哪里"，不改变"平面有多大"。

use gvpe_math::{Aabb, Vec3};

use crate::shape::{Shape, ShapeType};

/// 保守 broad-phase AABB 半尺寸。
///
/// 取 `1.0e6`（1 km）的工程做法：游戏/仿真场景通常 < 10 km，1 km 的保守 AABB 已远超
/// 真实平面所对应的"地面 / 天空盒"范围，但又不至于让 broad phase 缓存溢出 f32 精度。
/// 选用 const 而非常量派生 trait 的关联常量：与同 crate `Sphere` 等常量风格统一。
pub const PLANE_AABB_HALF_EXTENT: f32 = 1.0e6;

/// 无限平面（半空间）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plane {
    /// 平面法线（应已归一化）。
    pub normal: Vec3,
    /// 平面到原点的有符号距离。
    pub offset: f32,
}

impl Plane {
    /// 构造新平面。
    #[inline]
    #[must_use]
    pub const fn new(normal: Vec3, offset: f32) -> Self {
        Self { normal, offset }
    }
}

impl Shape for Plane {
    #[inline]
    fn shape_type(&self) -> ShapeType {
        ShapeType::Plane
    }

    #[inline]
    fn local_aabb(&self) -> Aabb {
        // 平面无界——返回保守立方 AABB。详见类型文档。
        let h = Vec3::splat(PLANE_AABB_HALF_EXTENT);
        Aabb::new(-h, h)
    }
}
