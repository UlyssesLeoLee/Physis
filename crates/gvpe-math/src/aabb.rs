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
    #[must_use]
    pub const fn new(min: super::Vec3, max: super::Vec3) -> Self {
        Self { min, max }
    }

    /// 从单点构造零体积 AABB。
    #[inline]
    #[must_use]
    pub const fn from_point(p: super::Vec3) -> Self {
        Self::new(p, p)
    }

    /// 从点切片构造包含所有点的 AABB。
    ///
    /// **空切片返回 `None`**（工程做法：不允许零大小 AABB）。
    /// **单点**返回零体积 AABB（等价于 [`Aabb::from_point`]）。
    /// **多点**返回各分量取 `min` / `max` 的 AABB。
    ///
    /// # 例子
    ///
    /// ```
    /// use gvpe_math::{Aabb, Vec3};
    /// // 空切片：返回 None
    /// assert_eq!(Aabb::from_points(&[]), None);
    /// // 单点：零体积 AABB
    /// let a = Aabb::from_points(&[Vec3::new(1.0, 2.0, 3.0)]).unwrap();
    /// assert_eq!(a.min, Vec3::new(1.0, 2.0, 3.0));
    /// assert_eq!(a.max, Vec3::new(1.0, 2.0, 3.0));
    /// // 多点：包含所有点的 AABB
    /// let pts = [Vec3::new(0.0, 0.0, 0.0), Vec3::new(2.0, -1.0, 5.0), Vec3::new(-1.0, 3.0, 1.0)];
    /// let a = Aabb::from_points(&pts).unwrap();
    /// assert_eq!(a.min, Vec3::new(-1.0, -1.0, 0.0));
    /// assert_eq!(a.max, Vec3::new(2.0, 3.0, 5.0));
    /// ```
    #[must_use]
    pub fn from_points(pts: &[super::Vec3]) -> Option<Self> {
        let first = *pts.first()?;
        let mut min = first;
        let mut max = first;
        for &p in &pts[1..] {
            min = super::Vec3::new(min.x.min(p.x), min.y.min(p.y), min.z.min(p.z));
            max = super::Vec3::new(max.x.max(p.x), max.y.max(p.y), max.z.max(p.z));
        }
        Some(Self::new(min, max))
    }

    /// 从中心 + 半尺寸构造。
    #[inline]
    #[must_use]
    pub fn from_center_half_extents(center: super::Vec3, half_extents: super::Vec3) -> Self {
        Self::new(center - half_extents, center + half_extents)
    }

    /// 中心。
    #[inline]
    #[must_use]
    pub fn center(self) -> super::Vec3 {
        (self.min + self.max) * 0.5
    }

    /// 半尺寸。
    #[inline]
    #[must_use]
    pub fn half_extents(self) -> super::Vec3 {
        (self.max - self.min) * 0.5
    }

    /// 是否与另一个 AABB 重叠（含边界）。
    #[inline]
    #[must_use]
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
    #[must_use]
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
    #[must_use]
    pub fn expand_to_include(self, p: super::Vec3) -> Self {
        Self::new(
            super::Vec3::new(
                self.min.x.min(p.x),
                self.min.y.min(p.y),
                self.min.z.min(p.z),
            ),
            super::Vec3::new(
                self.max.x.max(p.x),
                self.max.y.max(p.y),
                self.max.z.max(p.z),
            ),
        )
    }

    /// 合并两个 AABB。
    #[inline]
    #[must_use]
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

    /// 与另一 AABB 求交集。
    ///
    /// 返回**包围盒几何交集**对应的 AABB（`min = max(self.min, other.min)`，
    /// `max = min(self.max, other.max)`，逐分量）；**任一分量**上交集为空（`new_min > new_max`）则返回 `None`。
    ///
    /// 与 [`Self::overlaps`] 的关系：`intersection` 返回 `None` 当且仅当 `overlaps` 返回 `false`；
    /// 但 `intersection` 还会给出"重合区域"，`overlaps` 只会返 bool。典型用途：BVH 求交后递推。
    ///
    /// **边界语义**：与 `overlaps` 一致——边界相切（`new_min == new_max`）返回零体积 AABB（`Some`），
    /// 而非 `None`；调用方若需"严格体相交通"应在外层检查 `half_extents()` 是否全 `0`。
    #[inline]
    #[must_use]
    pub fn intersection(self, other: Self) -> Option<Self> {
        let new_min = super::Vec3::max(self.min, other.min);
        let new_max = super::Vec3::min(self.max, other.max);
        if new_min.x > new_max.x || new_min.y > new_max.y || new_min.z > new_max.z {
            return None;
        }
        Some(Self::new(new_min, new_max))
    }
}

impl Default for Aabb {
    fn default() -> Self {
        Self::ZERO
    }
}
