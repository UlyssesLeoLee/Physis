//! `Pool<T>`：固定大小对象复用。

use thiserror::Error;

/// Pool 错误。
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PoolError {
    /// 索引越界。
    #[error("pool index {0} 越界 (len = {1})")]
    OutOfBounds(usize, usize),

    /// 槽位为空（已 release 或未 acquire）。
    #[error("pool slot {0} 为空")]
    EmptySlot(usize),
}

/// `Pool<T>`：固定大小对象池。
///
/// acquire / release O(1)；不支持跨线程共享（MVP）。
pub struct Pool<T> {
    slots: Vec<Option<T>>,
    free_list: Vec<u32>,
}

impl<T> Pool<T> {
    /// 构造空 pool。
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

    /// 获取（acquire）一个对象。
    ///
    /// 若 `free_list` 有空位，复用；否则扩展。
    pub fn acquire(&mut self, val: T) -> u32 {
        if let Some(idx) = self.free_list.pop() {
            self.slots[idx as usize] = Some(val);
            idx
        } else {
            let idx = self.slots.len() as u32;
            self.slots.push(Some(val));
            idx
        }
    }

    /// 归还（release）一个对象。
    pub fn release(&mut self, idx: u32) -> Result<(), PoolError> {
        let i = idx as usize;
        if i >= self.slots.len() {
            return Err(PoolError::OutOfBounds(i, self.slots.len()));
        }
        if self.slots[i].is_none() {
            return Err(PoolError::EmptySlot(i));
        }
        self.slots[i] = None;
        self.free_list.push(idx);
        Ok(())
    }

    /// 借用。
    pub fn get(&self, idx: u32) -> Result<&T, PoolError> {
        let i = idx as usize;
        self.slots
            .get(i)
            .and_then(|s| s.as_ref())
            .ok_or(PoolError::EmptySlot(i))
    }

    /// 可变借用。
    pub fn get_mut(&mut self, idx: u32) -> Result<&mut T, PoolError> {
        let i = idx as usize;
        self.slots
            .get_mut(i)
            .and_then(|s| s.as_mut())
            .ok_or(PoolError::EmptySlot(i))
    }

    /// 当前 active 数量。
    pub fn active_count(&self) -> usize {
        self.slots.iter().filter(|s| s.is_some()).count()
    }

    /// 容量（slot 总数）。
    pub fn capacity(&self) -> usize {
        self.slots.len()
    }

    /// 遍历所有 active slot，产出 `(index, &T)`。
    ///
    /// 顺序按 slot 索引递增；只访问 `Some(_)` 的 slot。
    pub fn iter(&self) -> impl Iterator<Item = (u32, &T)> {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(i, s)| s.as_ref().map(|v| (i as u32, v)))
    }

    /// 遍历所有 active slot，产出 `(index, &mut T)`。
    ///
    /// 顺序按 slot 索引递增；只访问 `Some(_)` 的 slot。
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (u32, &mut T)> {
        self.slots
            .iter_mut()
            .enumerate()
            .filter_map(|(i, s)| s.as_mut().map(|v| (i as u32, v)))
    }

    /// 取或填：若 `idx` 处为 `Some` 返回其 `&mut T`；若为 `None` 调用 `f` 填充后返回。
    ///
    /// `idx` 越界返回 [`PoolError::OutOfBounds`]。`idx` 此前**未 acquire**（槽位总数不足）也视为越界。
    pub fn get_or_insert_with<F: FnOnce() -> T>(
        &mut self,
        idx: u32,
        f: F,
    ) -> Result<&mut T, PoolError> {
        let i = idx as usize;
        if i >= self.slots.len() {
            return Err(PoolError::OutOfBounds(i, self.slots.len()));
        }
        if self.slots[i].is_none() {
            self.slots[i] = Some(f());
        }
        // SAFETY: 上文已保证 `slots[i]` 是 `Some`。
        Ok(self.slots[i].as_mut().expect("slot just populated"))
    }

    /// 函数式遍历所有 active slot，避免迭代器状态机开销（适合热路径）。
    ///
    /// 调用次数等于 `active_count()`；顺序按 slot 索引递增。
    pub fn for_each_active<F: FnMut(u32, &T)>(&self, mut f: F) {
        for (i, s) in self.slots.iter().enumerate() {
            if let Some(v) = s.as_ref() {
                f(i as u32, v);
            }
        }
    }
}

impl<T> Default for Pool<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> core::fmt::Debug for Pool<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Pool")
            .field("active", &self.active_count())
            .field("capacity", &self.slots.len())
            .field("free_list_len", &self.free_list.len())
            .finish()
    }
}
