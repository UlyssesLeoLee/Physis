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
        let (slot, new_cursor) = self.reserve_aligned::<T>(1)?;

        // SAFETY: `reserve_aligned` 已检查 `new_cursor <= capacity`；
        // `slot` 落在 `buf` 的 `as_mut_ptr()..as_mut_ptr().add(capacity)` 内。
        unsafe {
            *self.cursor.get() = new_cursor;
            let ptr = (*self.buf.get()).as_mut_ptr().add(slot).cast::<T>();
            ptr.write(val);
            Ok(&mut *ptr)
        }
    }

    /// 永不 panic 版本的 `alloc`：容量不足返 `None`，不消耗 cursor。
    ///
    /// 与 `alloc` 行为差异：失败时**不**回写 cursor 增量（因为从未递增）。
    #[allow(clippy::mut_from_ref)]
    pub fn try_alloc<T>(&self, val: T) -> Option<&mut T> {
        match self.reserve_aligned::<T>(1) {
            Ok((slot, new_cursor)) => {
                // SAFETY: 同 `alloc`。
                unsafe {
                    *self.cursor.get() = new_cursor;
                    let ptr = (*self.buf.get()).as_mut_ptr().add(slot).cast::<T>();
                    ptr.write(val);
                    Some(&mut *ptr)
                }
            }
            Err(_) => None,
        }
    }

    /// 批量分配 `len` 个 `T` 值的切片。
    ///
    /// 要求 `T: Copy` —— arena 不为 `T` 调用 `drop`，调用方需自行管理元素生命周期。
    /// 容量不足返 [`ArenaError::Overflow`]，不消耗 cursor。
    #[allow(clippy::mut_from_ref)]
    pub fn alloc_slice<T: Copy>(&self, len: usize) -> Result<&mut [T], ArenaError> {
        if len == 0 {
            // 零长切片永远不失败；返回空切片。
            return Ok(&mut []);
        }
        let (slot, new_cursor) = self.reserve_aligned::<T>(len)?;
        // SAFETY: `reserve_aligned` 已校验 `new_cursor <= capacity` 且 `len * size_of::<T>()`
        // 在 `checked_mul` 下未溢出，因此 `[slot, new_cursor)` 范围合法。
        unsafe {
            *self.cursor.get() = new_cursor;
            let ptr = (*self.buf.get()).as_mut_ptr().add(slot).cast::<T>();
            Ok(core::slice::from_raw_parts_mut(ptr, len))
        }
    }

    /// 计算 `count` 个 `T` 所需的对齐偏移 + 新的 cursor。
    ///
    /// 内部辅助函数：集中处理 `checked_mul` / `checked_add` 防止整数 wrap。
    /// 返回 `(aligned_offset, new_cursor)`；容量不足返 [`ArenaError::Overflow`]。
    ///
    /// # Safety
    ///
    /// 调用方负责保证 `count > 0`（本方法不校验），以及最终把 `new_cursor` 写回 cursor。
    fn reserve_aligned<T>(&self, count: usize) -> Result<(usize, usize), ArenaError> {
        let size = core::mem::size_of::<T>();
        let align = core::mem::align_of::<T>();

        // count * size_of::<T>() 溢出检查
        let total_bytes = size.checked_mul(count).ok_or(ArenaError::Overflow {
            needed: usize::MAX,
            available: 0,
        })?;

        // SAFETY: 单线程使用，&self 访问内部可变状态是安全的。
        let cursor_ref = self.cursor.get();
        let buf_ref = self.buf.get();
        let current = unsafe { *cursor_ref };
        let buf_len = unsafe { (*buf_ref).capacity() };

        // 对齐推进（对 align 是 2 的幂，行为良好；非 2 的幂走 slow path）。
        let aligned = if align.is_power_of_two() {
            (current + align - 1) & !(align - 1)
        } else {
            current + ((align - (current % align)) % align)
        };

        // aligned + total_bytes 溢出检查
        let new_cursor = aligned
            .checked_add(total_bytes)
            .ok_or(ArenaError::Overflow {
                needed: usize::MAX,
                available: buf_len,
            })?;

        if new_cursor > buf_len {
            return Err(ArenaError::Overflow {
                needed: new_cursor,
                available: buf_len,
            });
        }

        Ok((aligned, new_cursor))
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
