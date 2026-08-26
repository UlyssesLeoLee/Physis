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

#[test]
fn arena_try_alloc_none_when_full() {
    // 仅 8 字节容量，先 alloc 一个 i32（4 字节），再 try_alloc 一个 i32（还能放下），
    // 第三次 try_alloc 一个 [u8; 8]（放不下）应返回 None，且不消耗 cursor。
    let arena = Arena::with_capacity(8);
    let r1: &mut i32 = arena.try_alloc(1i32).expect("first i32 fits");
    assert_eq!(*r1, 1);
    let r2: &mut i32 = arena.try_alloc(2i32).expect("second i32 fits");
    assert_eq!(*r2, 2);
    let used_before = arena.used();
    let r3: Option<&mut [u8; 8]> = arena.try_alloc([0u8; 8]);
    assert!(r3.is_none());
    // try_alloc 失败不消耗 cursor。
    assert_eq!(arena.used(), used_before);
}

#[test]
fn arena_alloc_slice_basic() {
    let arena = Arena::with_capacity(256);
    let slice: &mut [u32] = arena.alloc_slice::<u32>(4).expect("4 u32 fits in 256");
    assert_eq!(slice.len(), 4);
    // 写入并验证每个槽位（不假设初值；`T: Copy` 由调用方负责填）。
    for (i, v) in slice.iter_mut().enumerate() {
        *v = (i as u32) * 10;
    }
    assert_eq!(slice, &[0, 10, 20, 30]);
}

#[test]
fn arena_alloc_slice_overflow() {
    // 16 字节容量，尝试 alloc_slice 8 个 u32（需 32 字节）应失败。
    let arena = Arena::with_capacity(16);
    let r = arena.alloc_slice::<u32>(8);
    assert!(r.is_err());
    // 失败不消耗 cursor。
    assert_eq!(arena.used(), 0);
}

#[test]
fn arena_alloc_slice_zero_len() {
    let arena = Arena::with_capacity(16);
    let slice: &mut [u64] = arena.alloc_slice::<u64>(0).expect("zero-len always ok");
    assert_eq!(slice.len(), 0);
    // 不消耗 cursor。
    assert_eq!(arena.used(), 0);
}

#[test]
fn arena_overflow_checked_arithmetic() {
    // 用 size_of::<T>()=1 的类型制造大数组触发 size*len 溢出。
    // count = usize::MAX, size = 1 -> size * count 在 checked_mul 下溢出，
    // 应返回 Overflow 而非 wrap 后续。
    let arena = Arena::with_capacity(64);
    let r = arena.alloc_slice::<u8>(usize::MAX);
    assert!(r.is_err());
    // 同样的：单个 alloc 的 size 不会 overflow，但分配的 len=usize::MAX 时 alloc_slice 必然失败。
    // 验证 cursor 未被推进。
    assert_eq!(arena.used(), 0);
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

#[test]
fn pool_iter_active_count_and_order() {
    let mut pool: Pool<i32> = Pool::new();
    pool.acquire(10);
    let h1 = pool.acquire(20);
    pool.acquire(30);
    pool.release(h1).unwrap();

    // iter: 只产 Some，按索引升序：slot0->10, slot2->30
    let collected: Vec<(u32, i32)> = pool.iter().map(|(i, v)| (i, *v)).collect();
    assert_eq!(collected, vec![(0, 10), (2, 30)]);
}

#[test]
fn pool_iter_mut_mutates() {
    let mut pool: Pool<i32> = Pool::new();
    pool.acquire(1);
    pool.acquire(2);
    pool.acquire(3);
    for (_, v) in pool.iter_mut() {
        *v += 100;
    }
    let collected: Vec<i32> = pool.iter().map(|(_, v)| *v).collect();
    assert_eq!(collected, vec![101, 102, 103]);
}

#[test]
fn pool_get_or_insert_with_fills_once() {
    let mut pool: Pool<i32> = Pool::new();
    let h = pool.acquire(7);
    // 已 Some -> 直接返回，不调用 f。
    let mut calls = 0u32;
    let r = pool.get_or_insert_with(h, || {
        calls += 1;
        999
    });
    assert_eq!(*r.unwrap(), 7);
    assert_eq!(calls, 0);

    // release 后再 get_or_insert_with -> 调 f。
    pool.release(h).unwrap();
    let r2 = pool.get_or_insert_with(h, || 42);
    assert_eq!(*r2.unwrap(), 42);

    // 越界返 OutOfBounds。
    let r3 = pool.get_or_insert_with(9999u32, || 0);
    assert!(r3.is_err());
}

#[test]
fn pool_for_each_active_call_count() {
    let mut pool: Pool<i32> = Pool::new();
    pool.acquire(1);
    pool.acquire(2);
    let h = pool.acquire(3);
    pool.release(h).unwrap();

    let mut count = 0usize;
    let mut sum = 0i32;
    pool.for_each_active(|_, v| {
        count += 1;
        sum += *v;
    });
    assert_eq!(count, pool.active_count());
    assert_eq!(count, 2);
    assert_eq!(sum, 1 + 2);
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

#[test]
fn slab_iter_skips_free() {
    let mut slab: Slab<i32> = Slab::new();
    let h0 = slab.allocate(10);
    let _h1 = slab.allocate(20);
    let h2 = slab.allocate(30);
    slab.free(h0).unwrap();
    slab.free(h2).unwrap();

    // 只剩中间 slot alive。
    let collected: Vec<(SlabHandle, i32)> = slab.iter().map(|(h, v)| (h, *v)).collect();
    assert_eq!(collected.len(), 1);
    assert_eq!(collected[0].0.index, 1);
    assert_eq!(collected[0].0.generation, 0);
    assert_eq!(collected[0].1, 20);
}

#[test]
fn slab_iter_mut_mutates() {
    let mut slab: Slab<i32> = Slab::new();
    slab.allocate(1);
    slab.allocate(2);
    for (h, v) in slab.iter_mut() {
        // index 是 u32,本测试仅创建 2 个 slot,index ∈ {0, 1},转换 lossless 无 wrap 风险。
        #[allow(clippy::cast_possible_wrap)]
        let idx = h.index as i32;
        *v = idx * 100;
    }
    let mut values: Vec<i32> = slab.iter().map(|(_, v)| *v).collect();
    values.sort_unstable();
    assert_eq!(values, vec![0, 100]);
}

#[test]
fn slab_is_valid_matrix() {
    let mut slab: Slab<i32> = Slab::new();
    // INVALID 句柄恒为 false。
    assert!(!slab.is_valid(SlabHandle::INVALID));
    // 空 slab 中合法 index=0 也越界 -> false。
    assert!(!slab.is_valid(SlabHandle {
        index: 0,
        generation: 0
    }));

    let h = slab.allocate(42);
    // alive -> true。
    assert!(slab.is_valid(h));
    slab.free(h).unwrap();
    // free 后 generation 不匹配 -> false。
    assert!(!slab.is_valid(h));
    // 越界 -> false。
    assert!(!slab.is_valid(SlabHandle {
        index: 99,
        generation: 0
    }));
}

#[test]
fn slab_active_handles_matches_manual() {
    let mut slab: Slab<i32> = Slab::new();
    let h0 = slab.allocate(10);
    let h1 = slab.allocate(20);
    let h2 = slab.allocate(30);
    slab.free(h1).unwrap();

    let mut handles = slab.active_handles();
    handles.sort_by_key(|h| h.index);
    let mut expected = vec![h0, h2];
    expected.sort_by_key(|h| h.index);
    assert_eq!(handles, expected);

    // 全 free 后应为空。
    slab.free(h0).unwrap();
    slab.free(h2).unwrap();
    assert!(slab.active_handles().is_empty());
}

#[test]
fn slab_reserve_does_not_grow_active() {
    let mut slab: Slab<i32> = Slab::new();
    let h = slab.allocate(1);
    let active_before = slab.active_count();
    slab.reserve(100);
    assert_eq!(slab.active_count(), active_before);
    assert!(slab.is_valid(h));
}
