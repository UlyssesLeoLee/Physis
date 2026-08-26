//! `Aabb`：轴对齐包围盒。

use bytemuck::{Pod, Zeroable};

/// 轴对齐包围盒（AABB）。
///
/// 24 字节，min / max 各 12 字节。
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Aabb {
    /// AABB 最小角点。
    pub min: super::Vec3,
    /// AABB 最大角点。
    pub max: super::Vec3,
}

// SAFETY: 仅含 `Vec3` 字段，标准布局。
unsafe impl Pod for Aabb {}
unsafe impl Zeroable for Aabb {}

impl Aabb {
    /// 零体积 AABB（min = max = (0, 0, 0)）。
    pub const ZERO: Self = Self {
        min: super::Vec3::ZERO,
        max: super::Vec3::ZERO,
    };

    /// 构造新 AABB。
    #[inline]
    pub const fn new(min: super::Vec3, max: super::Vec3) -> Self {
        Self { min, max }
    }

    /// 从单点构造零体积 AABB。
    #[inline]
    pub fn from_point(p: super::Vec3) -> Self {
        Self::new(p, p)
    }

    /// 从中心 + 半尺寸构造。
    #[inline]
    pub fn from_center_half_extents(center: super::Vec3, half_extents: super::Vec3) -> Self {
        Self::new(center - half_extents, center + half_extents)
    }

    /// 中心。
    #[inline]
    pub fn center(self) -> super::Vec3 {
        (self.min + self.max) * 0.5
    }

    /// 半尺寸。
    #[inline]
    pub fn half_extents(self) -> super::Vec3 {
        (self.max - self.min) * 0.5
    }

    /// 是否与另一个 AABB 重叠（含边界）。
    #[inline]
    pub fn overlaps(self, other: Self) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    /// 是否包含点。
    #[inline]
    pub fn contains(self, p: super::Vec3) -> bool {
        p.x >= self.min.x
            && p.x <= self.max.x
            && p.y >= self.min.y
            && p.y <= self.max.y
            && p.z >= self.min.z
            && p.z <= self.max.z
    }

    /// 扩展 AABB 以包含给定点。
    #[inline]
    pub fn expand_to_include(mut self, p: super::Vec3) -> Self {
        self.min.x = self.min.x.min(p.x);
        self.min.y = self.min.y.min(p.y);
        self.min.z = self.min.z.min(p.z);
        self.max.x = self.max.x.max(p.x);
        self.max.y = self.max.y.max(p.y);
        self.max.z = self.max.z.max(p.z);
        self
    }

    /// 合并两个 AABB。
    #[inline]
    pub fn merged(self, other: Self) -> Self {
        Self::new(
            super::Vec3::new(
                self.min.x.min(other.min.x),
                self.min.y.min(other.min.y),
                self.min.z.min(other.min.z),
            ),
            super::Vec3::new(
                self.max.x.max(other.max.x),
                self.max.y.max(other.max.y),
                self.max.z.max(other.max.z),
            ),
        )
    }
}

impl Default for Aabb {
    fn default() -> Self {
        Self::ZERO
    }
}
