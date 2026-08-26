//! `gvpe-memory`：自研内存分配器。
//!
//! 依据 `GVPE-DOC-08`（内存设计）与 `GVPE-DOC-17` §2。
//!
//! ## 分配器
//!
//! - [`Arena<T>`]：bump allocator，O(1) 分配，O(1) reset。**线程局部**（每个 worker 一个 Arena）。
//! - [`Pool<T>`]：固定大小对象复用。
//! - [`Slab<T>`]：带世代（generation）的对象池；支持 use-after-free 检测。
//!
//! ## 安全政策
//!
//! - 所有 `unsafe` 块必须 `// SAFETY:` 注释；
//! - 全部 crate 必须 miri 测试通过（CI 强制）；
//! - 热路径零分配（除首次预分配）。
//!
//! ## 未来
//!
//! - `Pool<T>` 可考虑 `crossbeam` 替代（依 `GVPE-DOC-26` §18.6.2 评估）；
//! - `Slab<T>` 的 `Drop` 策略需评估（见 `GVPE-DOC-08` §2.3）。

#![deny(unsafe_op_in_unsafe_fn, missing_debug_implementations, missing_docs)]
#![warn(non_ascii_idents)]
// 允许 pedantic/nursery 中与本 crate 设计意图无冲突的 lint。
// - `missing_const_for_fn`: 大量 `new()` / 简单 getter 会被建议标 `const`，
//   1.75 之后 `Vec::new()` 才稳定为 `const`，本 crate 显式支持 MSRV 1.75，
//   故不强行 const 化以保持与 MSRV 编译的兼容性。
// - `module_name_repetitions`: 见 workspace.lints（已全局 allow）。
// - `filter_map_bool_then`: 本 crate 在 `active_handles` 等处已用 `filter+map` 风格。
#![allow(clippy::missing_const_for_fn)]

mod arena;
mod pool;
mod slab;

pub use arena::{Arena, ArenaError};
pub use pool::{Pool, PoolError};
pub use slab::{Slab, SlabError, SlabHandle};
// v0.6 新增：re-exports 保持 crate 外部 API 平面向外扩展；
// 新类型 / 新签名均在对应模块里实现，crate 根只做统一公开。

#[cfg(test)]
mod tests;
