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
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// 全部分量为 0。
    #[inline]
    pub const fn zero() -> Self {
        Self::ZERO
    }

    /// 点积。
    #[inline]
    pub fn dot(self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    /// 叉积。
    #[inline]
    pub fn cross(self, rhs: Self) -> Self {
        Self {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    /// 长度平方（避免开方）。
    #[inline]
    pub fn length_squared(self) -> f32 {
        self.dot(self)
    }

    /// 长度。
    #[inline]
    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    /// 归一化；零向量返回 `Vec3::ZERO`。
    #[inline]
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
    pub fn normalize_fast(self) -> Self {
        let len_sq = self.length_squared();
        if len_sq == 0.0 {
            return Self::ZERO;
        }
        let inv_len = 1.0 / len_sq.sqrt();
        self * inv_len
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
