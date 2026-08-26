//! `Slab<T>`：带世代的对象池（use-after-free 检测）。
//!
//! 依据 `GVPE-DOC-08` §2.3。

use thiserror::Error;

/// Slab 错误。
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SlabError {
    /// 槽位已 free / 句柄世代不匹配。
    #[error("slab handle 世代不匹配: expected {expected}, got {got}")]
    GenerationMismatch {
        /// 当前 slot 的 generation（句柄释放后已自增）。
        expected: u32,
        /// 句柄携带的 generation（来自调用方）。
        got: u32,
    },

    /// 槽位索引越界。
    #[error("slab index {0} 越界 (len = {1})")]
    OutOfBounds(usize, usize),
}

/// 句柄 + 世代（与 [`gvpe-core::BodyHandle`] 同形，但本 crate 不依赖 gvpe-core 以保持解耦）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SlabHandle {
    /// slot 索引。
    pub index: u32,
    /// 世代号（free 时自增，用于 use-after-free 检测）。
    pub generation: u32,
}

impl SlabHandle {
    /// 无效句柄（index = 0, generation = 0）。用作 `Option<SlabHandle>` 的 None 替代。
    pub const INVALID: Self = Self {
        index: 0,
        generation: 0,
    };
}

/// 槽位内部状态。
struct Slot<T> {
    data: Option<T>,
    generation: u32,
}

/// `Slab<T>`：带世代的对象池。
pub struct Slab<T> {
    slots: Vec<Slot<T>>,
    free_list: Vec<u32>,
}

impl<T> Slab<T> {
    /// 构造空 slab。
    pub const fn new() -> Self {
        Self {
            slots: Vec::new(),
            free_list: Vec::new(),
        }
    }

    /// 预分配容量。
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            slots: Vec::with_capacity(capacity),
            free_list: Vec::with_capacity(capacity),
        }
    }

    /// 分配（acquire）。返回句柄。
    pub fn allocate(&mut self, val: T) -> SlabHandle {
        if let Some(idx) = self.free_list.pop() {
            let i = idx as usize;
            self.slots[i].data = Some(val);
            SlabHandle {
                index: idx,
                generation: self.slots[i].generation,
            }
        } else {
            let idx = self.slots.len() as u32;
            self.slots.push(Slot {
                data: Some(val),
                generation: 0,
            });
            SlabHandle {
                index: idx,
                generation: 0,
            }
        }
    }

    /// 释放（free）。释放后该句柄后续访问将返回 `GenerationMismatch` 错误。
    pub fn free(&mut self, handle: SlabHandle) -> Result<(), SlabError> {
        let i = handle.index as usize;
        if i >= self.slots.len() {
            return Err(SlabError::OutOfBounds(i, self.slots.len()));
        }
        if self.slots[i].generation != handle.generation {
            return Err(SlabError::GenerationMismatch {
                expected: self.slots[i].generation,
                got: handle.generation,
            });
        }
        self.slots[i].data = None;
        self.slots[i].generation = self.slots[i].generation.wrapping_add(1);
        self.free_list.push(handle.index);
        Ok(())
    }

    /// 借用。
    pub fn get(&self, handle: SlabHandle) -> Result<&T, SlabError> {
        let i = handle.index as usize;
        if i >= self.slots.len() {
            return Err(SlabError::OutOfBounds(i, self.slots.len()));
        }
        if self.slots[i].generation != handle.generation {
            return Err(SlabError::GenerationMismatch {
                expected: self.slots[i].generation,
                got: handle.generation,
            });
        }
        self.slots[i]
            .data
            .as_ref()
            .ok_or(SlabError::GenerationMismatch {
                expected: self.slots[i].generation,
                got: handle.generation,
            })
    }

    /// 可变借用。
    pub fn get_mut(&mut self, handle: SlabHandle) -> Result<&mut T, SlabError> {
        let i = handle.index as usize;
        if i >= self.slots.len() {
            return Err(SlabError::OutOfBounds(i, self.slots.len()));
        }
        // 提前拷贝 generation，避免 `as_mut` 与 `&self.slots[i].generation` 借用冲突。
        let expected_generation = self.slots[i].generation;
        if expected_generation != handle.generation {
            return Err(SlabError::GenerationMismatch {
                expected: expected_generation,
                got: handle.generation,
            });
        }
        self.slots[i]
            .data
            .as_mut()
            .ok_or(SlabError::GenerationMismatch {
                expected: expected_generation,
                got: handle.generation,
            })
    }

    /// 容量（slot 总数）。
    pub fn capacity(&self) -> usize {
        self.slots.len()
    }

    /// active 数量。
    pub fn active_count(&self) -> usize {
        self.slots.iter().filter(|s| s.data.is_some()).count()
    }

    /// 预分配容量（不影响 `active_count`）。
    ///
    /// 仅扩展内部 `Vec` 容量，不创建新 slot；常用于延迟热路径首次 realloc。
    pub fn reserve(&mut self, additional: usize) {
        self.slots.reserve(additional);
        // free_list 也预分配同样容量上限：通常与 slots 同量级。
        self.free_list.reserve(additional);
    }

    /// 不取借用地检查 `handle` 是否仍指向一个 alive slot。
    ///
    /// 仅做：index 越界检查 + generation 匹配检查。**不**做"data 是否为 Some"的二次校验
    /// —— 设计上 `generation` 必与"是否被 free"一一对应，命中即 alive。
    ///
    /// 用作热路径上的快速预检（避免 `get` 走 `Result` 解包路径）。
    pub fn is_valid(&self, handle: SlabHandle) -> bool {
        let i = handle.index as usize;
        i < self.slots.len() && self.slots[i].generation == handle.generation
    }

    /// 遍历所有 active slot，产出 `(handle, &T)`。
    ///
    /// 顺序按 slot 索引递增；`handle.generation` 与当前 slot 一致。
    pub fn iter(&self) -> impl Iterator<Item = (SlabHandle, &T)> {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(i, s)| s.data.as_ref().map(|v| (s.handle(i as u32), v)))
    }

    /// 遍历所有 active slot，产出 `(handle, &mut T)`。
    ///
    /// 顺序按 slot 索引递增；`handle.generation` 与当前 slot 一致。
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (SlabHandle, &mut T)> {
        // 不能在同一闭包里同时持有 `s.data` 的 `&mut` 和 `s.generation` 的 `&`，
        // 因此把 generation 提前拷出，再 move 进闭包。
        self.slots.iter_mut().enumerate().filter_map(|(i, s)| {
            let generation = s.generation;
            let handle = SlabHandle {
                index: i as u32,
                generation,
            };
            s.data.as_mut().map(|v| (handle, v))
        })
    }

    /// 返回所有 active handle 列表（用于 debug / GC 扫描）。
    ///
    /// 分配 `Vec` 的成本使本方法**不适合热路径**；仅供调试和一次性遍历使用。
    pub fn active_handles(&self) -> Vec<SlabHandle> {
        self.slots
            .iter()
            .enumerate()
            .filter(|(_, s)| s.data.is_some())
            .map(|(i, s)| s.handle(i as u32))
            .collect()
    }
}

impl<T> Slot<T> {
    /// 由当前 slot 状态构造一个匹配的 `SlabHandle`。
    fn handle(&self, index: u32) -> SlabHandle {
        SlabHandle {
            index,
            generation: self.generation,
        }
    }
}

impl<T> Default for Slab<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> core::fmt::Debug for Slab<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Slab")
            .field("active", &self.active_count())
            .field("capacity", &self.slots.len())
            .field("free_list_len", &self.free_list.len())
            .finish()
    }
}
