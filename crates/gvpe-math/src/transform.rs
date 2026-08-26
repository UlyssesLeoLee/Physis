//! `Transform`：平移 + 旋转（compact，32 字节）。

use bytemuck::{Pod, Zeroable};

/// 紧凑变换：平移 + 旋转（无缩放）。
///
/// 32 字节：Vec3(12) + 4 字节 padding + Quat(16)；`rotation` 16 字节对齐（SIMD 友好）。
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Transform {
    /// 平移分量（世界坐标）。
    pub translation: super::Vec3,
    /// 旋转分量（围绕原点）。
    pub rotation: super::Quat,
}

// SAFETY: `Vec3` 12 字节后由编译器插入 4 字节 padding，将 `rotation` 起始对齐到 16 字节；
// 整体布局稳定，所有字段为 `Pod`，`bytemuck::Pod` 安全。
unsafe impl Pod for Transform {}
unsafe impl Zeroable for Transform {}

impl Transform {
    /// 单位变换（无平移无旋转）。
    pub const IDENTITY: Self = Self {
        translation: super::Vec3::ZERO,
        rotation: super::Quat::IDENTITY,
    };

    /// 构造新变换。
    #[inline]
    #[must_use]
    pub const fn new(translation: super::Vec3, rotation: super::Quat) -> Self {
        Self {
            translation,
            rotation,
        }
    }

    /// 仅平移。
    #[inline]
    #[must_use]
    pub const fn from_translation(translation: super::Vec3) -> Self {
        Self::new(translation, super::Quat::IDENTITY)
    }

    /// 仅旋转（围绕原点）。
    #[inline]
    #[must_use]
    pub const fn from_rotation(rotation: super::Quat) -> Self {
        Self::new(super::Vec3::ZERO, rotation)
    }

    /// 应用此变换到向量（先旋转后平移）。
    #[inline]
    #[must_use]
    pub fn transform_vec3(self, v: super::Vec3) -> super::Vec3 {
        self.rotation.rotate_vec3(v) + self.translation
    }

    /// 逆变换。
    ///
    /// 公式：T^{-1} = (-q* v q, q^{-1}) 其中 v 是 translation。
    /// 简化：先减平移，再反向旋转。
    #[inline]
    #[must_use]
    pub fn inverse(self) -> Self {
        let inv_rot = self.rotation.conjugate();
        let inv_trans = inv_rot.rotate_vec3(-self.translation);
        Self::new(inv_trans, inv_rot)
    }

    /// 双 component 插值：`translation` 走 [`super::Vec3::lerp`]，`rotation` 走 [`super::Quat::slerp`]。
    ///
    /// 适合：动画 blend、相机过渡、骨骼插值。
    /// `t ∈ [0, 1]`：`t = 0` 返回 `a`，`t = 1` 返回 `b`。
    /// 假设 `a.rotation`、`b.rotation` 是单位四元数（构造 API 默认保证）；
    /// 否则 `slerp` 内的 `dot < 0` 取反分支仍会执行，但归一化效果不可预测。
    #[inline]
    #[must_use]
    pub fn lerp(a: Self, b: Self, t: f32) -> Self {
        Self::new(
            super::Vec3::lerp(a.translation, b.translation, t),
            super::Quat::slerp(a.rotation, b.rotation, t),
        )
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}
