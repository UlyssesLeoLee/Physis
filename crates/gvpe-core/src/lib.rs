//! `gvpe-core`：GVPE 核心类型。
//!
//! 依据 `GVPE-DOC-17` §1（核心类型 + `PhysicsProfile` POD + `RuntimeDescriptor`）
//! 与 `GVPE-DOC-55`（错误码目录）。
//!
//! ## 类型概览
//!
//! - 句柄：`BodyHandle`, `ConstraintHandle`, `IslandHandle`（世代索引，8 字节 POD）
//! - 数据结构：`PhysicsProfile`（80 字节 POD），`RuntimeDescriptor`
//! - 错误类型：`CoreError` + 枚举变体
//! - 标记 trait：`PkqEntity` / `RuntimeConstraintEntity` / `ExecutionEntity`（DEC-002 / QA-D-02）

#![deny(unsafe_op_in_unsafe_fn, missing_debug_implementations, missing_docs)]
#![warn(non_ascii_idents)]

mod descriptor;
mod error;
mod handle;
mod marker;
mod profile;

pub use descriptor::{BodySpec, RuntimeDescriptor};
pub use error::CoreError;
pub use handle::{BodyHandle, ConstraintHandle, IslandHandle};
pub use marker::{ExecutionEntity, PkqEntity, RuntimeConstraintEntity};
pub use profile::{PhysicsLodTag, PhysicsProfile, SolverTypeId};

#[cfg(test)]
mod tests;
