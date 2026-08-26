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
    /// 3 行，每行是一个 `Vec3`（行优先存储）。
    pub m: [super::Vec3; 3],
}

// SAFETY: `Mat3` 仅含 `Vec3` 字段，标准布局，无 padding。
unsafe impl Pod for Mat3 {}
unsafe impl Zeroable for Mat3 {}

impl Mat3 {
    /// 零矩阵。
    pub const ZERO: Self = Self {
        m: [super::Vec3::ZERO; 3],
    };

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
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        m11: f32,
        m12: f32,
        m13: f32,
        m21: f32,
        m22: f32,
        m23: f32,
        m31: f32,
        m32: f32,
        m33: f32,
    ) -> Self {
        Self {
            m: [
                super::Vec3::new(m11, m12, m13),
                super::Vec3::new(m21, m22, m23),
                super::Vec3::new(m31, m32, m33),
            ],
        }
    }

    /// 从单位四元数构造旋转矩阵。
    ///
    /// `q` 应为单位四元数；非单位四元数得到的是 scaled rotation（每行长度 = |q|²）。
    /// 矩阵布局与 `Mul<Vec3>` 一致：`M * v` 等价于 `q.rotate_vec3(v)`。
    #[allow(clippy::many_single_char_names)]
    #[inline]
    #[must_use]
    pub fn from_quat(q: super::Quat) -> Self {
        // 经典四元数→旋转矩阵公式（行主序）：
        //   r = 2 / |q|²
        //   M = [ 1 - r*(y² + z²),     r*(x*y - z*w),     r*(x*z + y*w) ]
        //       [     r*(x*y + z*w), 1 - r*(x² + z²),     r*(y*z - x*w) ]
        //       [     r*(x*z - y*w),     r*(y*z + x*w), 1 - r*(x² + y²) ]
        // 假设 |q|=1 故 r=2
        let (x, y, z, w) = (q.x, q.y, q.z, q.w);
        // 用 mul_add 减少算入：xx = 2*x², yy = 2*y², zz = 2*z²
        let xx = 2.0 * x * x;
        let yy = 2.0 * y * y;
        let zz = 2.0 * z * z;
        let xy = 2.0 * x * y;
        let xz = 2.0 * x * z;
        let yz = 2.0 * y * z;
        let wx = 2.0 * w * x;
        let wy = 2.0 * w * y;
        let wz = 2.0 * w * z;
        Self::new(
            1.0 - (yy + zz),
            xy - wz,
            xz + wy,
            xy + wz,
            1.0 - (xx + zz),
            yz - wx,
            xz - wy,
            yz + wx,
            1.0 - (xx + yy),
        )
    }

    /// 矩阵 × 向量。
    #[inline]
    #[must_use]
    pub fn mul_vec3(self, v: super::Vec3) -> super::Vec3 {
        super::Vec3::new(
            self.m[0]
                .x
                .mul_add(v.x, self.m[0].y.mul_add(v.y, self.m[0].z * v.z)),
            self.m[1]
                .x
                .mul_add(v.x, self.m[1].y.mul_add(v.y, self.m[1].z * v.z)),
            self.m[2]
                .x
                .mul_add(v.x, self.m[2].y.mul_add(v.y, self.m[2].z * v.z)),
        )
    }

    /// 矩阵 × 矩阵。
    #[inline]
    #[must_use]
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
    #[must_use]
    pub const fn transpose(self) -> Self {
        Self::new(
            self.m[0].x,
            self.m[1].x,
            self.m[2].x,
            self.m[0].y,
            self.m[1].y,
            self.m[2].y,
            self.m[0].z,
            self.m[1].z,
            self.m[2].z,
        )
    }

    /// 求逆（伴随矩阵法 / 行列式）。
    ///
    /// 返回 `None` 表示矩阵奇异。
    #[allow(clippy::many_single_char_names)]
    #[must_use]
    pub fn inverse(self) -> Option<Self> {
        let m = &self.m;
        let a = m[0].x;
        let b = m[0].y;
        let c = m[0].z;
        let d = m[1].x;
        let e = m[1].y;
        let f = m[1].z;
        let g = m[2].x;
        let h = m[2].y;
        let i = m[2].z;

        // 行列式 = a(ei - fh) - b(di - fg) + c(dh - eg)
        let det = a.mul_add(
            f.mul_add(-h, e * i),
            c.mul_add(f.mul_add(-g, d * h), -(b * f.mul_add(-g, d * i))),
        );
        if det.abs() < f32::EPSILON {
            return None;
        }
        let inv_det = 1.0 / det;

        Some(Self::new(
            f.mul_add(-h, e * i) * inv_det,
            b.mul_add(-i, c * h) * inv_det,
            c.mul_add(-e, b * f) * inv_det,
            d.mul_add(-i, f * g) * inv_det,
            c.mul_add(-g, a * i) * inv_det,
            a.mul_add(-f, c * d) * inv_det,
            e.mul_add(-g, d * h) * inv_det,
            a.mul_add(-h, b * g) * inv_det,
            b.mul_add(-d, a * e) * inv_det,
        ))
    }
}

impl Default for Mat3 {
    fn default() -> Self {
        Self::IDENTITY
    }
}
