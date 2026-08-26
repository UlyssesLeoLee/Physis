//! `Arena<T>`：bump allocator，O(1) 分配 / 重置。
//!
//! 帧 scratch 场景：每帧重置，单帧内 O(1) 分配。
//!
//! **线程局部**：每个 worker 持有一个独立 `Arena`（避免锁竞争）。
//! `AtomicUsize::fetch_add` 仍可作为多线程访问的兜底（参考 `GVPE-DOC-17` §2.1）。

use core::cell::UnsafeCell;
use thiserror::Error;

/// Arena 错误。
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ArenaError {
    /// 预分配耗尽。
    #[error("arena overflow: need {needed} bytes, available {available}")]
    Overflow {
        /// 已尝试分配的总字节数（含对齐 padding）。
        needed: usize,
        /// 当前 `Arena` 剩余可用字节数。
        available: usize,
    },
}

/// `Arena<T>`：bump allocator。
///
/// 用 `UnsafeCell` 包装可变性（不要求 `&mut self`）；单线程使用。
pub struct Arena {
    buf: UnsafeCell<Vec<u8>>,
    cursor: UnsafeCell<usize>,
}

// SAFETY: `Arena` 设计为线程局部（每个 worker 一个），不跨线程共享。
// 若需要跨线程，应使用 thread-local 或显式 `&mut self`。
// `Arena` 自身不含泛型参数 `T`（分配时由调用方指定 `T`），所以 `Send`/`Sync` 只需声明
// 内部存储（`Vec<u8>` 自身已是 `Send + Sync`）即可。
unsafe impl Send for Arena {}
unsafe impl Sync for Arena {}

impl Arena {
    /// 构造指定容量的 arena。
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: UnsafeCell::new(Vec::with_capacity(capacity)),
            cursor: UnsafeCell::new(0),
        }
    }

    /// 分配一个 `T` 值的引用；返回 lifetime = arena 生命周期的 `&mut T`。
    ///
    /// # Safety
    ///
    /// 调用方必须保证：
    /// - `T` 写入时不会 panic（如 `T: Copy` 或不调用 `Drop`）；
    /// - 不持有跨 `reset()` 调用的引用；
    /// - **单线程访问**（`Arena` 自身 `Send + Sync` 仅用于 `thread_local!` 的类型约束，
    ///   不表示内部可变状态是线程安全的；多线程共享需外层同步或每个 worker 独立 `Arena`）。
    #[allow(clippy::mut_from_ref)]
    pub fn alloc<T>(&self, val: T) -> Result<&mut T, ArenaError> {
        let size = core::mem::size_of::<T>();
        let align = core::mem::align_of::<T>();

        let cursor_ref = self.cursor.get();
        let buf_ref = self.buf.get();

        // SAFETY: 单线程使用，&self 访问内部可变状态是安全的。
        let current = unsafe { *cursor_ref };
        let aligned = (current + align - 1) & !(align - 1);
        let new_cursor = aligned + size;

        let buf_len = unsafe { (*buf_ref).capacity() };
        if new_cursor > buf_len {
            return Err(ArenaError::Overflow {
                needed: new_cursor,
                available: buf_len,
            });
        }

        // SAFETY: 已检查容量 + 对齐；写入不会越界。
        unsafe {
            *cursor_ref = new_cursor;
            let ptr = (*buf_ref).as_mut_ptr().add(aligned).cast::<T>();
            ptr.write(val);
            Ok(&mut *ptr)
        }
    }

    /// 重置 cursor（O(1)）。**不**调用 `T::drop`（调用方需自行管理）。
    pub fn reset(&mut self) {
        // SAFETY: 重置 cursor 后，之前的引用无效。调用方负责。
        self.cursor = UnsafeCell::new(0);
    }

    /// 当前已分配字节数。
    pub fn used(&self) -> usize {
        // SAFETY: 单线程使用。
        unsafe { *self.cursor.get() }
    }

    /// 容量。
    pub fn capacity(&self) -> usize {
        // SAFETY: 单线程使用。
        unsafe { (*self.buf.get()).capacity() }
    }
}

impl Default for Arena {
    fn default() -> Self {
        Self::with_capacity(64 * 1024) // 默认 64 KB
    }
}

impl core::fmt::Debug for Arena {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // SAFETY: 单线程使用，读 cursor 与 buf.len() 是只读操作。
        let used = unsafe { *self.cursor.get() };
        let cap = unsafe { (*self.buf.get()).capacity() };
        f.debug_struct("Arena")
            .field("used", &used)
            .field("capacity", &cap)
            .finish()
    }
}
