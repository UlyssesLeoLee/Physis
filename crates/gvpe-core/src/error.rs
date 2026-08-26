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
    ProfileMissingField {
        /// 缺失字段名。
        field: &'static str,
    },

    /// `PhysicsProfile` 字段值违反不变式（如 mass ≤ 0 且非 static）。
    #[error("PhysicsProfile 字段值违反不变式: {field} = {value}")]
    ProfileInconsistent {
        /// 违反不变式的字段名。
        field: &'static str,
        /// 实际值。
        value: f32,
    },

    /// `RuntimeDescriptor` 无 body。
    #[error("RuntimeDescriptor 为空")]
    DescriptorEmpty,

    /// `RuntimeDescriptor::body` / `body_mut` 索引越界。
    #[error("body 索引越界: index = {index}, len = {len}")]
    BodyIndexOutOfBounds {
        /// 请求的索引。
        index: usize,
        /// 当前 `bodies.len()`。
        len: usize,
    },

    /// `RuntimeDescriptor` 中出现重复的 body 标识（保留字段，MVP 暂以 index 排序后查重）。
    #[error("RuntimeDescriptor 含重复 body index: {index}")]
    DuplicateBodyIndex {
        /// 重复的索引。
        index: usize,
    },

    /// `BodySpecBuilder` 缺少必填字段。
    #[error("BodySpec 必填字段缺失: {field}")]
    BodySpecMissingField {
        /// 缺失字段名。
        field: &'static str,
    },

    /// 物理 LOD 不支持（MVP 仅 LOD0）。
    #[error("物理 LOD {0:?} 不支持（MVP 仅 Lod0Full）")]
    LodNotSupported(crate::PhysicsLodTag),
}

/// 核心 crate 使用的 `Result` 类型别名。
#[allow(dead_code)]
pub type CoreResult<T> = Result<T, CoreError>;
