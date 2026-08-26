//! `Vec3`：三维向量。
//!
//! POD 类型，跨 FRI 边界可直接 `memcpy`。
//! 布局：`#[repr(C)]`，字段顺序 x, y, z，无 padding。

use bytemuck::{Pod, Zeroable};

/// 三维向量。
///
/// 字段顺序按 `(x, y, z)`，与数学惯例一致。
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(C)]
pub struct Vec3 {
    /// x 分量。
    pub x: f32,
    /// y 分量。
    pub y: f32,
    /// z 分量。
    pub z: f32,
}

// SAFETY: `Vec3` 仅含 `f32` 字段，标准布局，无 padding，`bytemuck::Pod` 安全。
unsafe impl Pod for Vec3 {}
unsafe impl Zeroable for Vec3 {}

impl Vec3 {
    /// 零向量。
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    /// 单位向量 (1, 0, 0)。
    pub const X: Self = Self {
        x: 1.0,
        y: 0.0,
        z: 0.0,
    };

    /// 单位向量 (0, 1, 0)。
    pub const Y: Self = Self {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };

    /// 单位向量 (0, 0, 1)。
    pub const Z: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    };

    /// 构造新向量。
    #[inline]
    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// 三分量都填 `v`。
    #[inline]
    #[must_use]
    pub const fn splat(v: f32) -> Self {
        Self { x: v, y: v, z: v }
    }

    /// 全部分量为 0。
    #[inline]
    #[must_use]
    pub const fn zero() -> Self {
        Self::ZERO
    }

    /// 点积。
    #[inline]
    #[must_use]
    pub fn dot(self, rhs: Self) -> f32 {
        self.x.mul_add(rhs.x, self.y.mul_add(rhs.y, self.z * rhs.z))
    }

    /// 叉积。
    #[inline]
    #[must_use]
    pub fn cross(self, rhs: Self) -> Self {
        Self {
            x: self.y.mul_add(rhs.z, -(self.z * rhs.y)),
            y: self.z.mul_add(rhs.x, -(self.x * rhs.z)),
            z: self.x.mul_add(rhs.y, -(self.y * rhs.x)),
        }
    }

    /// 长度平方（避免开方）。
    #[inline]
    #[must_use]
    pub fn length_squared(self) -> f32 {
        self.dot(self)
    }

    /// 长度。
    #[inline]
    #[must_use]
    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    /// 距离平方（避免开方）。
    #[inline]
    #[must_use]
    pub fn distance_squared(self, other: Self) -> f32 {
        (self - other).length_squared()
    }

    /// 欧氏距离。
    #[inline]
    #[must_use]
    pub fn distance(self, other: Self) -> f32 {
        self.distance_squared(other).sqrt()
    }

    /// 归一化；零向量返回 `Vec3::ZERO`。
    #[inline]
    #[must_use]
    pub fn normalize(self) -> Self {
        let len_sq = self.length_squared();
        if len_sq == 0.0 {
            return Self::ZERO;
        }
        let inv_len = 1.0 / len_sq.sqrt();
        self * inv_len
    }

    /// 归一化（带 fast inverse sqrt）。
    #[inline]
    #[must_use]
    pub fn normalize_fast(self) -> Self {
        let len_sq = self.length_squared();
        if len_sq == 0.0 {
            return Self::ZERO;
        }
        let inv_len = 1.0 / len_sq.sqrt();
        self * inv_len
    }

    /// 法线向量反射。
    ///
    /// 公式：`v - 2 * dot(v, n) * n`，要求 `normal` 已归一化（不归一化会得到错误结果）。
    /// 典型用途：刚体碰撞反弹 `v_out = v_in.reflect(n)`。
    #[inline]
    #[must_use]
    pub fn reflect(self, normal: Self) -> Self {
        let d = self.dot(normal);
        // self - 2*d*normal  →  self + (-2d)*normal
        let k = -2.0 * d;
        Self {
            x: normal.x.mul_add(k, self.x),
            y: normal.y.mul_add(k, self.y),
            z: normal.z.mul_add(k, self.z),
        }
    }

    /// 投影到另一个向量上。
    ///
    /// 返回 `other` 方向上的投影分量：`other * dot(self, other) / |other|²`。
    /// 若 `other` 为零向量，返回 `Vec3::ZERO`。
    /// 注：不归一化 `other`，因开销由调用方控制。
    #[inline]
    #[must_use]
    pub fn project_onto(self, other: Self) -> Self {
        let denom = other.length_squared();
        if denom == 0.0 {
            return Self::ZERO;
        }
        let s = self.dot(other) / denom;
        other * s
    }

    /// 线性插值：`a + (b - a) * t`。
    ///
    /// `t` 越界不裁剪（NaN/Inf 由调用方负责）；典型 `t ∈ [0, 1]`。
    /// `t = 0` 返回 `a`，`t = 1` 返回 `b`。
    #[inline]
    #[must_use]
    pub fn lerp(a: Self, b: Self, t: f32) -> Self {
        Self {
            x: (b.x - a.x).mul_add(t, a.x),
            y: (b.y - a.y).mul_add(t, a.y),
            z: (b.z - a.z).mul_add(t, a.z),
        }
    }
}

impl core::ops::Add for Vec3 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl core::ops::Sub for Vec3 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl core::ops::Neg for Vec3 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }
}

impl core::ops::Mul<f32> for Vec3 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f32) -> Self {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl core::ops::Div<f32> for Vec3 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: f32) -> Self {
        Self::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}
