//! `Quat`：四元数（x, y, z, w）。
//!
//! 16 字节对齐（SIMD 友好），适用于 vendor intrinsics 路径。

use bytemuck::{Pod, Zeroable};

/// 四元数。
///
/// 字段顺序 `(x, y, z, w)`，符合数学惯例。
/// `#[repr(C, align(16))]` 保证 16 字节对齐，便于 SIMD 优化。
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C, align(16))]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

// SAFETY: `Quat` 仅含 `f32` 字段，align(16) 无 padding，`bytemuck::Pod` 安全。
unsafe impl Pod for Quat {}
unsafe impl Zeroable for Quat {}

impl Quat {
    /// 单位四元数（无旋转）。
    pub const IDENTITY: Self = Self { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };

    /// 构造新四元数。
    #[inline]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    /// 从旋转轴（必须归一化）和角度（弧度）构造。
    #[inline]
    pub fn from_axis_angle(axis: super::Vec3, angle_rad: f32) -> Self {
        let half = angle_rad * 0.5;
        let s = half.sin();
        let c = half.cos();
        Self::new(axis.x * s, axis.y * s, axis.z * s, c)
    }

    /// 四元数共轭（对单位四元数 = 逆）。
    #[inline]
    pub fn conjugate(self) -> Self {
        Self::new(-self.x, -self.y, -self.z, self.w)
    }

    /// 四元数模长平方。
    #[inline]
    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
    }

    /// 四元数模长。
    #[inline]
    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    /// 归一化；零四元数返回 `IDENTITY`。
    #[inline]
    pub fn normalize(self) -> Self {
        let len_sq = self.length_squared();
        if len_sq == 0.0 {
            return Self::IDENTITY;
        }
        let inv_len = 1.0 / len_sq.sqrt();
        Self::new(self.x * inv_len, self.y * inv_len, self.z * inv_len, self.w * inv_len)
    }

    /// Hamilton 积。
    #[inline]
    pub fn mul(self, rhs: Self) -> Self {
        Self::new(
            self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,
            self.w * rhs.y - self.x * rhs.z + self.y * rhs.w + self.z * rhs.x,
            self.w * rhs.z + self.x * rhs.y - self.y * rhs.x + self.z * rhs.w,
            self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
        )
    }

    /// 用此四元数旋转向量 v。
    #[inline]
    pub fn rotate_vec3(self, v: super::Vec3) -> super::Vec3 {
        // q * v * q^-1 优化版（仅用 q 而非 q^-1，因为 q 是单位）
        let qv = super::Vec3::new(self.x, self.y, self.z);
        let t = qv.cross(v).mul_scalar(2.0);
        v + t.mul_scalar(self.w) + qv.cross(t)
    }
}

// 注：`Quat` 与 `f32` 的乘法 / 除法未实现（语义不明确）。

// 私有扩展 trait
trait Vec3Ext {
    fn mul_scalar(self, s: f32) -> super::Vec3;
}

impl Vec3Ext for super::Vec3 {
    #[inline]
    fn mul_scalar(self, s: f32) -> super::Vec3 {
        super::Vec3::new(self.x * s, self.y * s, self.z * s)
    }
}
