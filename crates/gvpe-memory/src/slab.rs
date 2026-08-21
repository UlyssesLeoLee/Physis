//! `Slab<T>`：带世代的对象池（use-after-free 检测）。
//!
//! 依据 `GVPE-DOC-08` §2.3。

use thiserror::Error;

/// Slab 错误。
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SlabError {
    /// 槽位已 free / 句柄世代不匹配。
    #[error("slab handle 世代不匹配: expected {expected}, got {got}")]
    GenerationMismatch { expected: u32, got: u32 },

    /// 槽位索引越界。
    #[error("slab index {0} 越界 (len = {1})")]
    OutOfBounds(usize, usize),
}

/// 句柄 + 世代（与 [`gvpe-core::BodyHandle`] 同形，但本 crate 不依赖 gvpe-core 以保持解耦）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SlabHandle {
    pub index: u32,
    pub generation: u32,
}

impl SlabHandle {
    pub const INVALID: Self = Self { index: 0, generation: 0 };
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
    pub fn new() -> Self {
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
            SlabHandle { index: idx, generation: self.slots[i].generation }
        } else {
            let idx = self.slots.len() as u32;
            self.slots.push(Slot { data: Some(val), generation: 0 });
            SlabHandle { index: idx, generation: 0 }
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
        self.slots[i].data.as_ref().ok_or(SlabError::GenerationMismatch {
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
        if self.slots[i].generation != handle.generation {
            return Err(SlabError::GenerationMismatch {
                expected: self.slots[i].generation,
                got: handle.generation,
            });
        }
        self.slots[i].data.as_mut().ok_or(SlabError::GenerationMismatch {
            expected: self.slots[i].generation,
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
}

impl<T> Default for Slab<T> {
    fn default() -> Self {
        Self::new()
    }
}
