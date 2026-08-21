//! 句柄类型：世代索引（generational index）。

use bytemuck::{Pod, Zeroable};

/// Body 句柄：`{index, generation}`，8 字节。
///
/// `generation` 在 body 被 free 时由 [`gvpe-memory::Slab`] 递增；
/// 通过检查 `generation` 与 slab 中存储的 `generation` 是否一致来检测 use-after-free。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct BodyHandle {
    pub index: u32,
    pub generation: u32,
}

// SAFETY: 仅 u32 字段，标准布局。
unsafe impl Pod for BodyHandle {}
unsafe impl Zeroable for BodyHandle {}

impl BodyHandle {
    /// 无效句柄（index=0, generation=0）。
    pub const INVALID: Self = Self { index: 0, generation: 0 };

    /// 构造新句柄。
    #[inline]
    pub const fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }

    /// 是否等于 [`INVALID`](Self::INVALID)。
    #[inline]
    pub fn is_invalid(self) -> bool {
        self == Self::INVALID
    }
}

/// Constraint 句柄（与 `BodyHandle` 同形）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct ConstraintHandle {
    pub index: u32,
    pub generation: u32,
}

unsafe impl Pod for ConstraintHandle {}
unsafe impl Zeroable for ConstraintHandle {}

impl ConstraintHandle {
    pub const INVALID: Self = Self { index: 0, generation: 0 };

    #[inline]
    pub const fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }

    #[inline]
    pub fn is_invalid(self) -> bool {
        self == Self::INVALID
    }
}

/// Island 句柄（仅 `u32`，无 generation）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct IslandHandle(pub u32);

unsafe impl Pod for IslandHandle {}
unsafe impl Zeroable for IslandHandle {}

impl IslandHandle {
    pub const INVALID: Self = Self(0);

    #[inline]
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    #[inline]
    pub fn is_invalid(self) -> bool {
        self == Self::INVALID
    }
}
