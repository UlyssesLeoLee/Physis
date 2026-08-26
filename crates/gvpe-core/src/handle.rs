//! 句柄类型：世代索引（generational index）。

use bytemuck::{Pod, Zeroable};

/// Body 句柄：`{index, generation}`，8 字节。
///
/// `generation` 在 body 被 free 时由 [`gvpe-memory::Slab`] 递增；
/// 通过检查 `generation` 与 slab 中存储的 `generation` 是否一致来检测 use-after-free。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct BodyHandle {
    /// body 索引（slot 编号）。
    pub index: u32,
    /// 世代号（free 时自增，用于 use-after-free 检测）。
    pub generation: u32,
}

// SAFETY: 仅 u32 字段，标准布局。
unsafe impl Pod for BodyHandle {}
unsafe impl Zeroable for BodyHandle {}

impl BodyHandle {
    /// 无效句柄（index=0, generation=0）。
    pub const INVALID: Self = Self {
        index: 0,
        generation: 0,
    };

    /// 构造新句柄。
    #[inline]
    pub const fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }

    /// 从原始位模式重建句柄（语义上等价于 [`new`](Self::new)，仅命名差异）。
    ///
    /// 用于：
    /// - FFI 边界：直接接收 `index` / `generation` 两个 `u32` 重组句柄
    /// - 序列化反序列化：跳过类型校验直接落位
    /// - 单元测试：精确构造特定 `(index, generation)` 组合
    #[inline]
    pub const fn from_raw(index: u32, generation: u32) -> Self {
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
    /// constraint 索引。
    pub index: u32,
    /// 世代号。
    pub generation: u32,
}

unsafe impl Pod for ConstraintHandle {}
unsafe impl Zeroable for ConstraintHandle {}

impl ConstraintHandle {
    /// 无效句柄。
    pub const INVALID: Self = Self {
        index: 0,
        generation: 0,
    };

    /// 构造新句柄。
    #[inline]
    pub const fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }

    /// 从原始位模式重建句柄（语义同 [`new`](Self::new)）。
    #[inline]
    pub const fn from_raw(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }

    /// 是否等于 [`INVALID`](Self::INVALID)。
    #[inline]
    pub fn is_invalid(self) -> bool {
        self == Self::INVALID
    }
}

/// Island 句柄（仅 `u32`，无 generation）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct IslandHandle(
    /// island 索引（island 在生命周期内不被 free，故无 generation）。
    pub u32,
);

unsafe impl Pod for IslandHandle {}
unsafe impl Zeroable for IslandHandle {}

impl IslandHandle {
    /// 无效 island 句柄。
    pub const INVALID: Self = Self(0);

    /// 构造新 island 句柄。
    #[inline]
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    /// 从原始位模式重建 island 句柄。
    #[inline]
    pub const fn from_raw(index: u32) -> Self {
        Self(index)
    }

    /// 是否等于 [`INVALID`](Self::INVALID)。
    #[inline]
    pub fn is_invalid(self) -> bool {
        self == Self::INVALID
    }
}
