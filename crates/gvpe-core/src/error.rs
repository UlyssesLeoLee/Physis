//! 核心错误类型（`CoreError`）。
//!
//! 详见 `GVPE-DOC-55` §2.1。

use thiserror::Error;

/// `gvpe-core` 错误类型。
///
/// MVP 仅含 handle 相关错误；后续 crate 会在其内部定义更具体错误并通过 `From` 转换。
#[derive(Debug, Error)]
pub enum CoreError {
    /// 句柄引用已 free 的资源（use-after-free）。
    #[error("handle 引用已 free: {0:?}")]
    HandleStale(crate::BodyHandle),

    /// 句柄编码非法。
    #[error("handle 编码非法: {0:?}")]
    HandleInvalid(crate::BodyHandle),

    /// `PhysicsProfile` 必填字段缺失。
    #[error("PhysicsProfile 必填字段缺失: {field}")]
    ProfileMissingField { field: &'static str },

    /// `PhysicsProfile` 字段值违反不变式（如 mass ≤ 0 且非 static）。
    #[error("PhysicsProfile 字段值违反不变式: {field} = {value}")]
    ProfileInconsistent { field: &'static str, value: f32 },

    /// `RuntimeDescriptor` 无 body。
    #[error("RuntimeDescriptor 为空")]
    DescriptorEmpty,

    /// 物理 LOD 不支持（MVP 仅 LOD0）。
    #[error("物理 LOD {0:?} 不支持（MVP 仅 Lod0Full）")]
    LodNotSupported(crate::PhysicsLodTag),
}

/// 核心 crate 使用的 `Result` 类型别名。
pub type CoreResult<T> = Result<T, CoreError>;
