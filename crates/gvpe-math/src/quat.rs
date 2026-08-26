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
    /// 四元数 x 分量（虚部 i）。
    pub x: f32,
    /// 四元数 y 分量（虚部 j）。
    pub y: f32,
    /// 四元数 z 分量（虚部 k）。
    pub z: f32,
    /// 四元数 w 分量（实部）。
    pub w: f32,
}

// SAFETY: `Quat` 仅含 `f32` 字段，align(16) 无 padding，`bytemuck::Pod` 安全。
unsafe impl Pod for Quat {}
unsafe impl Zeroable for Quat {}

impl Quat {
    /// 单位四元数（无旋转）。
    pub const IDENTITY: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 1.0,
    };

    /// 构造新四元数。
    #[inline]
    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    /// 从旋转轴（必须归一化）和角度（弧度）构造。
    #[inline]
    #[must_use]
    pub fn from_axis_angle(axis: super::Vec3, angle_rad: f32) -> Self {
        let half = angle_rad * 0.5;
        let s = half.sin();
        let c = half.cos();
        Self::new(axis.x * s, axis.y * s, axis.z * s, c)
    }

    /// 四元数共轭（对单位四元数 = 逆）。
    #[inline]
    #[must_use]
    pub fn conjugate(self) -> Self {
        Self::new(-self.x, -self.y, -self.z, self.w)
    }

    /// 四元数模长平方。
    #[inline]
    #[must_use]
    pub fn length_squared(self) -> f32 {
        self.x.mul_add(
            self.x,
            self.y
                .mul_add(self.y, self.z.mul_add(self.z, self.w * self.w)),
        )
    }

    /// 四元数模长。
    #[inline]
    #[must_use]
    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    /// 归一化；零四元数返回 `IDENTITY`。
    #[inline]
    #[must_use]
    pub fn normalize(self) -> Self {
        let len_sq = self.length_squared();
        if len_sq == 0.0 {
            return Self::IDENTITY;
        }
        let inv_len = 1.0 / len_sq.sqrt();
        Self::new(
            self.x * inv_len,
            self.y * inv_len,
            self.z * inv_len,
            self.w * inv_len,
        )
    }

    /// Hamilton 积。
    #[inline]
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn mul(self, rhs: Self) -> Self {
        Self::new(
            self.w.mul_add(
                rhs.x,
                self.x
                    .mul_add(rhs.w, self.y.mul_add(rhs.z, -(self.z * rhs.y))),
            ),
            self.w.mul_add(
                rhs.y,
                self.y
                    .mul_add(rhs.w, self.z.mul_add(rhs.x, -(self.x * rhs.z))),
            ),
            self.w.mul_add(
                rhs.z,
                self.z
                    .mul_add(rhs.w, self.x.mul_add(rhs.y, -(self.y * rhs.x))),
            ),
            self.w.mul_add(
                rhs.w,
                -(self.x.mul_add(rhs.x, self.y.mul_add(rhs.y, self.z * rhs.z))),
            ),
        )
    }

    /// 用此四元数旋转向量 v。
    #[inline]
    #[must_use]
    pub fn rotate_vec3(self, v: super::Vec3) -> super::Vec3 {
        // q * v * q^-1 优化版（仅用 q 而非 q^-1，因为 q 是单位）
        let qv = super::Vec3::new(self.x, self.y, self.z);
        let t = qv.cross(v) * 2.0;
        v + t * self.w + qv.cross(t)
    }

    /// 从 Tait-Bryan 欧拉角（YPR）构造四元数。
    ///
    /// **约定**：Z-Y-X intrinsic（先绕 fixed-Z = `yaw`，再绕 fixed-Y = `pitch`，
    /// 再绕 fixed-X = `roll`，全部弧度）。
    /// 实现等价于 `Quat::from_axis_angle(Vec3::Z, yaw) * from_axis_angle(Vec3::Y, pitch) * from_axis_angle(Vec3::X, roll)`。
    /// 单位：弧度；输入不做归一化。
    /// **注意**：gimbal lock（`pitch = ±π/2`）时不做特判，结果可能含 NaN；
    /// 调用方若需鲁棒性应在外层检查。配套逆运算见 [`Quat::to_euler_ypr`]。
    #[allow(clippy::many_single_char_names)]
    #[inline]
    #[must_use]
    pub fn from_euler_ypr(yaw: f32, pitch: f32, roll: f32) -> Self {
        // 用三角乘法（half-angle），gimbal lock 不爆 NaN。
        // q = q_z(yaw) * q_y(pitch) * q_x(roll) 展开后：
        //   x = cy*cp*sr - sy*sp*cr
        //   y = cy*sp*cr + sy*cp*sr
        //   z = sy*cp*cr - cy*sp*sr
        //   w = cy*cp*cr + sy*sp*sr
        let (sy, cy) = (yaw * 0.5).sin_cos();
        let (sp, cp) = (pitch * 0.5).sin_cos();
        let (sr, cr) = (roll * 0.5).sin_cos();
        Self::new(
            cy.mul_add(cp * sr, -(sy * sp * cr)),
            cy.mul_add(sp * cr, sy * cp * sr),
            sy.mul_add(cp * cr, -(cy * sp * sr)),
            cy.mul_add(cp * cr, sy * sp * sr),
        )
    }

    /// 四元数 → Tait-Bryan 欧拉角（YPR）。
    ///
    /// **约定**：Z-Y-X intrinsic（与 [`Quat::from_euler_ypr`] 对偶），返回 `[yaw, pitch, roll]`，全部弧度。
    /// 在 `|sin(pitch)| >= 1`（gimbal lock）时设 `pitch = ±π/2`，yaw/roll 置 0（可读性优先，不做 branchless 优化）。
    #[inline]
    #[must_use]
    pub fn to_euler_ypr(self) -> [f32; 3] {
        // 标准 Z-Y-X intrinsic 公式：
        //   sinp = 2*(w*y - z*x)
        //   pitch = asin(clamp(sinp, -1, 1))
        //   yaw   = atan2(2*(w*z + x*y), 1 - 2*(y² + z²))
        //   roll  = atan2(2*(w*x + y*z), 1 - 2*(x² + y²))
        let sinp = (2.0 * self.w).mul_add(self.y, -(2.0 * self.z * self.x));
        let sinp_clamped = sinp.clamp(-1.0, 1.0);
        if sinp_clamped.abs() >= 1.0 {
            // gimbal lock：pitch = ±π/2，yaw/roll 折叠到 0
            let pitch = if sinp > 0.0 {
                core::f32::consts::FRAC_PI_2
            } else {
                -core::f32::consts::FRAC_PI_2
            };
            return [0.0, pitch, 0.0];
        }
        let pitch = sinp_clamped.asin();
        let yz_sq = self.y.mul_add(self.y, self.z * self.z);
        let num_yaw = (2.0 * self.w).mul_add(self.z, 2.0 * self.x * self.y);
        let denom_yaw = (-2.0_f32).mul_add(yz_sq, 1.0);
        let yaw = num_yaw.atan2(denom_yaw);
        let xy_sq = self.x.mul_add(self.x, self.y * self.y);
        let num_roll = (2.0 * self.w).mul_add(self.x, 2.0 * self.y * self.z);
        let denom_roll = (-2.0_f32).mul_add(xy_sq, 1.0);
        let roll = num_roll.atan2(denom_roll);
        [yaw, pitch, roll]
    }

    /// 四元数 → 3x3 旋转矩阵（[`Mat3::from_quat`] 的镜像）。
    ///
    /// 等价于 `Mat3::from_quat(self)`：复用 [`Mat3::from_quat`] 的实现，零重复代码。
    /// 满足 `m.mul_vec3(v) == self.rotate_vec3(v)`。
    #[inline]
    #[must_use]
    pub fn to_mat3(self) -> super::Mat3 {
        super::Mat3::from_quat(self)
    }

    /// 球面线性插值（SLERP）。
    ///
    /// `t ∈ [0, 1]`：`t = 0` 返回 `a`，`t = 1` 返回 `b`。
    /// 当 `dot(a, b) < 0` 时对 `b` 取反，走较短弧。
    /// 当 `dot(a, b) ≈ 1`（几乎同向）时退化为 nlerp（normalised lerp），避免 `sin(θ) ≈ 0` 除零。
    /// 假设 `a`、`b` 是单位四元数（构造 API 默认保证）。
    #[allow(clippy::many_single_char_names)]
    #[inline]
    #[must_use]
    pub fn slerp(a: Self, b: Self, t: f32) -> Self {
        let mut dot =
            a.x.mul_add(b.x, a.y.mul_add(b.y, a.z.mul_add(b.z, a.w * b.w)));
        // 走较短弧：dot < 0 时取反 b
        let b = if dot < 0.0 {
            dot = -dot;
            Self::new(-b.x, -b.y, -b.z, -b.w)
        } else {
            b
        };
        // 接近 1：避免除零，退化为 nlerp
        if dot > 0.9995 {
            let r = Self::new(
                (b.x - a.x).mul_add(t, a.x),
                (b.y - a.y).mul_add(t, a.y),
                (b.z - a.z).mul_add(t, a.z),
                (b.w - a.w).mul_add(t, a.w),
            );
            return r.normalize();
        }
        let theta = dot.clamp(-1.0, 1.0).acos();
        let sin_theta = theta.sin();
        let s1 = ((1.0 - t) * theta).sin() / sin_theta;
        let s2 = (t * theta).sin() / sin_theta;
        Self::new(
            a.x.mul_add(s1, b.x * s2),
            a.y.mul_add(s1, b.y * s2),
            a.z.mul_add(s1, b.z * s2),
            a.w.mul_add(s1, b.w * s2),
        )
    }
}

// 注：`Quat` 与 `f32` 的乘法 / 除法未实现（语义不明确）。
