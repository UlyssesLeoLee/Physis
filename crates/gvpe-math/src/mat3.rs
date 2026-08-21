//! `Mat3`：3x3 矩阵（行优先）。

use bytemuck::{Pod, Zeroable};

/// 3x3 矩阵，行优先存储。
///
/// `[m11, m12, m13, m21, m22, m23, m31, m32, m33]` 即
/// ```text
/// | m11 m12 m13 |
/// | m21 m22 m23 |
/// | m31 m32 m33 |
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Mat3 {
    pub m: [super::Vec3; 3],
}

// SAFETY: `Mat3` 仅含 `Vec3` 字段，标准布局，无 padding。
unsafe impl Pod for Mat3 {}
unsafe impl Zeroable for Mat3 {}

impl Mat3 {
    /// 零矩阵。
    pub const ZERO: Self = Self { m: [super::Vec3::ZERO; 3] };

    /// 单位矩阵。
    pub const IDENTITY: Self = Self {
        m: [
            super::Vec3::new(1.0, 0.0, 0.0),
            super::Vec3::new(0.0, 1.0, 0.0),
            super::Vec3::new(0.0, 0.0, 1.0),
        ],
    };

    /// 构造新矩阵（行优先）。
    #[inline]
    pub const fn new(m11: f32, m12: f32, m13: f32, m21: f32, m22: f32, m23: f32, m31: f32, m32: f32, m33: f32) -> Self {
        Self {
            m: [
                super::Vec3::new(m11, m12, m13),
                super::Vec3::new(m21, m22, m23),
                super::Vec3::new(m31, m32, m33),
            ],
        }
    }

    /// 矩阵 × 向量。
    #[inline]
    pub fn mul_vec3(self, v: super::Vec3) -> super::Vec3 {
        super::Vec3::new(
            self.m[0].x * v.x + self.m[0].y * v.y + self.m[0].z * v.z,
            self.m[1].x * v.x + self.m[1].y * v.y + self.m[1].z * v.z,
            self.m[2].x * v.x + self.m[2].y * v.y + self.m[2].z * v.z,
        )
    }

    /// 矩阵 × 矩阵。
    #[inline]
    pub fn mul_mat3(self, rhs: Self) -> Self {
        Self {
            m: [
                self.mul_vec3(super::Vec3::new(rhs.m[0].x, rhs.m[1].x, rhs.m[2].x)),
                self.mul_vec3(super::Vec3::new(rhs.m[0].y, rhs.m[1].y, rhs.m[2].y)),
                self.mul_vec3(super::Vec3::new(rhs.m[0].z, rhs.m[1].z, rhs.m[2].z)),
            ],
        }
    }

    /// 转置。
    #[inline]
    pub fn transpose(self) -> Self {
        Self::new(
            self.m[0].x, self.m[1].x, self.m[2].x,
            self.m[0].y, self.m[1].y, self.m[2].y,
            self.m[0].z, self.m[1].z, self.m[2].z,
        )
    }

    /// 求逆（伴随矩阵法 / 行列式）。
    ///
    /// 返回 `None` 表示矩阵奇异。
    pub fn inverse(self) -> Option<Self> {
        let m = &self.m;
        let a = m[0].x; let b = m[0].y; let c = m[0].z;
        let d = m[1].x; let e = m[1].y; let f = m[1].z;
        let g = m[2].x; let h = m[2].y; let i = m[2].z;

        let det = a * (e * i - f * h) - b * (d * i - f * g) + c * (d * h - e * g);
        if det.abs() < f32::EPSILON {
            return None;
        }
        let inv_det = 1.0 / det;

        Some(Self::new(
            (e * i - f * h) * inv_det,
            (c * h - b * i) * inv_det,
            (b * f - c * e) * inv_det,
            (f * g - d * i) * inv_det,
            (a * i - c * g) * inv_det,
            (c * d - a * f) * inv_det,
            (d * h - e * g) * inv_det,
            (b * g - a * h) * inv_det,
            (a * e - b * d) * inv_det,
        ))
    }
}

impl Default for Mat3 {
    fn default() -> Self {
        Self::IDENTITY
    }
}
