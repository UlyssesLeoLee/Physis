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

mod arena;
mod pool;
mod slab;

pub use arena::Arena;
pub use pool::Pool;
pub use slab::Slab;

#[cfg(test)]
mod tests;
