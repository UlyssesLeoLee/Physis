//! `Transform`：平移 + 旋转（compact，28 字节）。

use bytemuck::{Pod, Zeroable};

/// 紧凑变换：平移 + 旋转（无缩放）。
///
/// 28 字节；`rotation` 16 字节对齐（SIMD 友好）。
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Transform {
    pub translation: super::Vec3,
    pub rotation: super::Quat,
}

// SAFETY: 紧凑布局，`translation` 12 字节后 4 字节 padding 补齐 `rotation` 起始至 16。
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
    pub const fn new(translation: super::Vec3, rotation: super::Quat) -> Self {
        Self { translation, rotation }
    }

    /// 仅平移。
    #[inline]
    pub fn from_translation(translation: super::Vec3) -> Self {
        Self::new(translation, super::Quat::IDENTITY)
    }

    /// 仅旋转（围绕原点）。
    #[inline]
    pub fn from_rotation(rotation: super::Quat) -> Self {
        Self::new(super::Vec3::ZERO, rotation)
    }

    /// 应用此变换到向量（先旋转后平移）。
    #[inline]
    pub fn transform_vec3(self, v: super::Vec3) -> super::Vec3 {
        self.rotation.rotate_vec3(v) + self.translation
    }

    /// 逆变换。
    ///
    /// 公式：T^{-1} = (-q* v q, q^{-1}) 其中 v 是 translation。
    /// 简化：先减平移，再反向旋转。
    #[inline]
    pub fn inverse(self) -> Self {
        let inv_rot = self.rotation.conjugate();
        let inv_trans = inv_rot.rotate_vec3(-self.translation);
        Self::new(inv_trans, inv_rot)
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}
