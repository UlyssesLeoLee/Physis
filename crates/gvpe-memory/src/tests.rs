//! `gvpe-memory` 单元测试。

use crate::{Arena, Pool, Slab, SlabHandle};

// ============================================================================
// Arena 测试
// ============================================================================

#[test]
fn arena_alloc_and_reset() {
    let mut arena = Arena::with_capacity(1024);
    assert_eq!(arena.capacity(), 1024);
    assert_eq!(arena.used(), 0);

    // 分配 i32
    let r: &mut i32 = arena.alloc(42i32).unwrap();
    assert_eq!(*r, 42);
    *r = 100;
    assert_eq!(arena.used(), 4);

    // 分配 u64（需要 8 字节对齐）
    let r2: &mut u64 = arena.alloc(123u64).unwrap();
    assert_eq!(*r2, 123);
    assert!(arena.used() >= 12);

    // reset
    arena.reset();
    assert_eq!(arena.used(), 0);
}

#[test]
fn arena_overflow() {
    let arena = Arena::with_capacity(8);
    let r: Result<&mut i32, _> = arena.alloc(0i32);
    assert!(r.is_ok());
    // 再分配超过容量
    let r2: Result<&mut [u8; 16], _> = arena.alloc([0u8; 16]);
    assert!(r2.is_err());
}

// ============================================================================
// Pool 测试
// ============================================================================

#[test]
fn pool_basic() {
    let mut pool: Pool<i32> = Pool::new();
    assert_eq!(pool.capacity(), 0);
    assert_eq!(pool.active_count(), 0);

    let h1 = pool.acquire(42);
    let h2 = pool.acquire(100);
    assert_eq!(pool.active_count(), 2);
    assert_eq!(pool.capacity(), 2);

    assert_eq!(*pool.get(h1).unwrap(), 42);
    assert_eq!(*pool.get(h2).unwrap(), 100);
}

#[test]
fn pool_acquire_release_reuse() {
    let mut pool: Pool<i32> = Pool::new();
    let h1 = pool.acquire(42);
    pool.release(h1).unwrap();
    assert_eq!(pool.active_count(), 0);

    // 复用了同一个 slot
    let h2 = pool.acquire(100);
    assert_eq!(h1, h2);
    assert_eq!(pool.active_count(), 1);
}

#[test]
fn pool_double_release_error() {
    let mut pool: Pool<i32> = Pool::new();
    let h = pool.acquire(42);
    pool.release(h).unwrap();
    let r = pool.release(h);
    assert!(r.is_err());
}

// ============================================================================
// Slab 测试
// ============================================================================

#[test]
fn slab_allocate_and_free() {
    let mut slab: Slab<i32> = Slab::new();
    let h1 = slab.allocate(42);
    let h2 = slab.allocate(100);
    assert_eq!(*slab.get(h1).unwrap(), 42);
    assert_eq!(*slab.get(h2).unwrap(), 100);
}

#[test]
fn slab_use_after_free_detected() {
    let mut slab: Slab<i32> = Slab::new();
    let h = slab.allocate(42);
    slab.free(h).unwrap();

    // 再次访问应失败（世代不匹配）
    let r = slab.get(h);
    assert!(r.is_err());
}

#[test]
fn slab_recycle_after_free() {
    let mut slab: Slab<i32> = Slab::new();
    let h1 = slab.allocate(42);
    slab.free(h1).unwrap();

    // 重新分配应拿到新句柄（generation 已递增）
    let h2 = slab.allocate(100);
    assert_eq!(h2.index, h1.index);
    assert_ne!(h2.generation, h1.generation);
    assert_eq!(*slab.get(h2).unwrap(), 100);
}
