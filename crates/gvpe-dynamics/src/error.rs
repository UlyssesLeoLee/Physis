//! `gvpe-dynamics` 错误类型。
//!
//! 与 `gvpe-core::CoreError` 解耦：上层 `DynamicsWorld` 内部可同时持有
//! `gvpe-memory::SlabError` / `gvpe-core::CoreError` 字样，调用方按需
//! 通过 `From` 链聚合成本 crate 的 [`DynamicsError`]。
//!
//! MVP 阶段仅含 handle / 数值两类错误；constraint / solver 阶段会扩展。

use thiserror::Error;

/// `gvpe-dynamics` 错误类型。
#[derive(Debug, Error)]
pub enum DynamicsError {
    /// 句柄引用已 free 的资源（use-after-free）。
    #[error("dynamics handle 引用已 free: {0:?}")]
    HandleStale(gvpe_core::BodyHandle),

    /// 句柄编码非法（island 不存在）。
    #[error("island 句柄非法: {0:?}")]
    IslandInvalid(gvpe_core::IslandHandle),

    /// 时间步长非法（`<= 0` 或 `NaN`）。
    #[error("时间步长非法: dt = {0}")]
    InvalidTimeStep(f32),

    /// 状态数值非法（`NaN` / `Inf` 出现在位置 / 速度 / 旋转等）。
    #[error("动力学状态数值非法: field = {field}, value = {value}")]
    StateNotFinite {
        /// 违反不变式的字段名。
        field: &'static str,
        /// 实际值。
        value: f32,
    },

    /// 底层 `Slab` 报错（generation mismatch / 越界）。
    #[error("slab 错误: {0}")]
    Slab(#[from] gvpe_memory::SlabError),

    /// 底层 `CoreError` 转发（占位，MVP 暂未直接消费）。
    #[error("core 错误: {0}")]
    Core(#[from] gvpe_core::CoreError),
}

/// `gvpe-dynamics` 使用的 `Result` 类型别名。
pub type DynamicsResult<T> = Result<T, DynamicsError>;
