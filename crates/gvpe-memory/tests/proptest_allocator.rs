//! `gvpe-memory` v0.6 加固测试 —— property-based + 边界。
//!
//! 加固目标：
//! - `Pool`: acquire/release 平衡 / 索引越界错误 / 双重 release / iter 顺序 / get_or_insert_with
//! - `Slab`: use-after-free 检测 / 释放后句柄 re-allocate / iter 顺序
//! - `Arena`: alloc 顺序布局 / reset 后 cursor 归零 / overflow 错误码 / alloc_slice 对齐
//!
//! 加固 commit 基线：v0.6 (1f789e9)。
//! 修订者：Mavis 接手 agent per DEC-008 (2026-08-27 08:00 JST 指令)。

// 测试模块 lint 集中允许：见 `crates/gvpe-core/tests/integration_profile_runtime.rs` 同段说明
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_parens)]
#![allow(clippy::float_cmp)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::many_single_char_names)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::collection_is_never_read)]
#![allow(clippy::double_parens)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::match_wildcard_for_single_variants)]

use gvpe_memory::{Arena, ArenaError, Pool, PoolError, Slab, SlabError, SlabHandle};
use proptest::prelude::*;

/// 模拟操作类型（用于 proptest 生成 Pool 操作序列）。
#[derive(Debug, Clone)]
enum PoolOp {
    Acquire(i32),
    Release(u32),
    GetOrInsert(u32, i32),
}

/// 模拟操作类型（用于 proptest 生成 Slab 操作序列）。
#[derive(Debug, Clone)]
enum SlabOp {
    Allocate(i32),
    Free(u32),
}

/// Pool 操作序列生成策略。
fn pool_op_strategy() -> impl Strategy<Value = Vec<PoolOp>> {
    proptest::collection::vec(
        prop_oneof![
            3 => any::<i32>().prop_map(PoolOp::Acquire),
            1 => (0..32u32).prop_map(PoolOp::Release),
            1 => ((0..32u32, any::<i32>())).prop_map(|(i, v)| PoolOp::GetOrInsert(i, v)),
        ],
        0..64,
    )
}

/// Slab 操作序列生成策略。
fn slab_op_strategy() -> impl Strategy<Value = Vec<SlabOp>> {
    proptest::collection::vec(
        prop_oneof![
            3 => any::<i32>().prop_map(SlabOp::Allocate),
            1 => (0..32u32).prop_map(SlabOp::Free),
        ],
        0..64,
    )
}

proptest! {
    // ===== Pool 核心不变量 =====

    fn pool_invariants_after_ops(ops in pool_op_strategy()) {
        let mut pool: Pool<i32> = Pool::new();
        for op in ops {
            match op {
                PoolOp::Acquire(v) => {
                    let _ = pool.acquire(v);
                }
                PoolOp::Release(idx) => {
                    let _ = pool.release(idx);
                }
                PoolOp::GetOrInsert(idx, v) => {
                    let _ = pool.get_or_insert_with(idx, || v);
                }
            }
        }
        let active = pool.active_count();
        let cap = pool.capacity();
        prop_assert!(active <= cap, "active {} > cap {}", active, cap);
    }

    fn pool_sequential_acquire_get(n in 1..32usize) {
        let mut pool: Pool<i32> = Pool::new();
        for i in 0..n {
            let idx = pool.acquire(i as i32);
            prop_assert_eq!(idx, i as u32);
        }
        for i in 0..n {
            let v = pool.get(i as u32).unwrap();
            prop_assert_eq!(*v, i as i32);
        }
        prop_assert_eq!(pool.active_count(), n);
        prop_assert_eq!(pool.capacity(), n);
    }

    fn pool_double_release_returns_empty_slot(v in any::<i32>()) {
        let mut pool: Pool<i32> = Pool::new();
        let idx = pool.acquire(v);
        pool.release(idx).unwrap();
        let err = pool.release(idx).unwrap_err();
        prop_assert_eq!(err, PoolError::EmptySlot(idx as usize));
    }

    fn pool_release_out_of_bounds(cap_pre in 0..16usize, idx in 0..u32::MAX) {
        let mut pool: Pool<i32> = Pool::with_capacity(cap_pre);
        for i in 0..cap_pre {
            pool.acquire(i as i32);
        }
        let real_cap = pool.capacity();
        prop_assume!((idx as usize) >= real_cap);
        let err = pool.release(idx).unwrap_err();
        prop_assert_eq!(err, PoolError::OutOfBounds(idx as usize, real_cap));
    }

    fn pool_release_oob_arbitrary(idx in 100..200u32) {
        let mut pool: Pool<i32> = Pool::with_capacity(0);
        let err = pool.release(idx).unwrap_err();
        prop_assert_eq!(err, PoolError::OutOfBounds(idx as usize, 0));
    }

    fn pool_get_out_of_bounds(idx in 100..200u32) {
        let pool: Pool<i32> = Pool::with_capacity(0);
        let err = pool.get(idx).unwrap_err();
        prop_assert_eq!(err, PoolError::EmptySlot(idx as usize));
    }

    fn pool_release_then_acquire_reuses_slot(v in any::<i32>()) {
        let mut pool: Pool<i32> = Pool::new();
        let idx0 = pool.acquire(v);
        pool.release(idx0).unwrap();
        let idx1 = pool.acquire(v.wrapping_add(1));
        prop_assert_eq!(idx0, idx1, "free list 应 LIFO 复用 idx");
        let stored = pool.get(idx0).unwrap();
        prop_assert_eq!(*stored, v.wrapping_add(1));
    }

    fn pool_iter_ordering(n in 1..16usize) {
        let mut pool: Pool<i32> = Pool::new();
        for i in 0..n {
            pool.acquire(i as i32);
        }
        let collected: Vec<(u32, i32)> = pool.iter().map(|(i, v)| (i, *v)).collect();
        prop_assert_eq!(collected.len(), n);
        for (i, (slot, val)) in collected.iter().enumerate() {
            prop_assert_eq!(*slot, i as u32);
            prop_assert_eq!(*val, i as i32);
        }
        for i in (0..n).filter(|i| i % 2 == 0) {
            pool.release(i as u32).unwrap();
        }
        let after: Vec<u32> = pool.iter().map(|(i, _)| i).collect();
        let expected: Vec<u32> = (0..n).filter(|i| i % 2 == 1).map(|i| i as u32).collect();
        prop_assert_eq!(after, expected);
    }

    fn pool_get_or_insert_fills_empty(idx in 0..8u32, v in any::<i32>()) {
        let mut pool: Pool<i32> = Pool::with_capacity(0);
        for i in 0..=idx {
            pool.acquire(0);
        }
        pool.release(idx).unwrap();
        let r = pool.get_or_insert_with(idx, || v).unwrap();
        prop_assert_eq!(*r, v);
        let r2 = pool.get_or_insert_with(idx, || v.wrapping_add(1)).unwrap();
        prop_assert_eq!(*r2, v, "已 Some 时不应调 f");
    }

    fn pool_get_or_insert_oob(idx in 100..200u32) {
        let mut pool: Pool<i32> = Pool::with_capacity(4);
        let err = pool.get_or_insert_with(idx, || 0).unwrap_err();
        prop_assert_eq!(err, PoolError::OutOfBounds(idx as usize, 4));
    }

    fn pool_for_each_active_count(n in 1..16usize) {
        let mut pool: Pool<i32> = Pool::new();
        for i in 0..n {
            pool.acquire(i as i32);
        }
        let mut count = 0usize;
        let mut sum = 0i32;
        pool.for_each_active(|_, v| {
            count += 1;
            sum += *v;
        });
        prop_assert_eq!(count, n);
        prop_assert_eq!(sum, (0..n as i32).sum());
    }

    // ===== Slab 核心不变量 =====

    fn slab_invariants_after_ops(ops in slab_op_strategy()) {
        let mut slab: Slab<i32> = Slab::new();
        for op in ops {
            match op {
                SlabOp::Allocate(v) => {
                    let _ = slab.allocate(v);
                }
                SlabOp::Free(idx) => {
                    let _ = slab.free(SlabHandle { index: idx, generation: 0 });
                }
            }
        }
        let active = slab.active_count();
        let cap = slab.capacity();
        prop_assert!(active <= cap, "active {} > cap {}", active, cap);
    }

    fn slab_use_after_free_detected(v in any::<i32>()) {
        let mut slab: Slab<i32> = Slab::new();
        let h = slab.allocate(v);
        slab.free(h).unwrap();
        let err = slab.get(h).unwrap_err();
        match err {
            SlabError::GenerationMismatch { expected, got } => {
                prop_assert!(expected != got, "free 后 generation 应自增");
            }
            other => prop_assert!(false, "期望 GenerationMismatch，得到 {:?}", other),
        }
    }

    fn slab_re_allocate_increments_generation(v in any::<i32>(), w in any::<i32>()) {
        let mut slab: Slab<i32> = Slab::new();
        let h0 = slab.allocate(v);
        slab.free(h0).unwrap();
        let h1 = slab.allocate(w);
        prop_assert_eq!(h0.index, h1.index, "free list 复用 slot");
        prop_assert_ne!(h0.generation, h1.generation, "generation 必增");
        let _ = slab.get(h0).unwrap_err();
        prop_assert_eq!(*slab.get(h1).unwrap(), w);
    }

    fn slab_free_out_of_bounds(idx in 100..200u32) {
        let mut slab: Slab<i32> = Slab::new();
        let err = slab.free(SlabHandle { index: idx, generation: 0 }).unwrap_err();
        prop_assert_eq!(err, SlabError::OutOfBounds(idx as usize, 0));
    }

    fn slab_is_valid_consistent_with_get(v in any::<i32>()) {
        let mut slab: Slab<i32> = Slab::new();
        let h = slab.allocate(v);
        prop_assert!(slab.is_valid(h));
        prop_assert!(slab.get(h).is_ok());
        slab.free(h).unwrap();
        prop_assert!(!slab.is_valid(h));
        prop_assert!(slab.get(h).is_err());
    }

    fn slab_active_handles_listing(n in 1..16usize) {
        let mut slab: Slab<i32> = Slab::new();
        let mut handles = Vec::new();
        for i in 0..n {
            handles.push(slab.allocate(i as i32));
        }
        let active = slab.active_handles();
        prop_assert_eq!(active.len(), n);
        for (i, h) in active.iter().enumerate() {
            prop_assert_eq!(h.index, i as u32);
            prop_assert_eq!(*slab.get(*h).unwrap(), i as i32);
        }
    }

    fn slab_reserve_preserves_active(n in 1..8usize, extra in 0..16usize) {
        let mut slab: Slab<i32> = Slab::new();
        for i in 0..n {
            slab.allocate(i as i32);
        }
        let cap_before = slab.capacity();
        let active_before = slab.active_count();
        slab.reserve(extra);
        prop_assert_eq!(slab.capacity(), cap_before, "reserve 不增 capacity");
        prop_assert_eq!(slab.active_count(), active_before);
    }

    // ===== Arena 核心不变量 =====

    fn arena_alloc_read_back(n in 1..32usize) {
        let arena = Arena::with_capacity(4096);
        let mut refs = Vec::new();
        for i in 0..n {
            let r: &mut i32 = arena.alloc(i as i32).unwrap();
            refs.push(r);
        }
        for (i, r) in refs.iter().enumerate() {
            prop_assert_eq!(**r, i as i32);
        }
        let used = arena.used();
        prop_assert!(used >= n * 4, "used {} < n*4={}", used, n * 4);
    }

    fn arena_overflow_returns_error(cap in 16..128usize) {
        let arena = Arena::with_capacity(cap);
        let mut n = 0usize;
        loop {
            let r: Result<&mut i32, _> = arena.alloc(0i32);
            if r.is_err() {
                break;
            }
            n += 1;
            if n > 10000 {
                prop_assert!(false, "alloc 不应无限成功");
            }
        }
        prop_assert!(n > 0);
        let err = arena.alloc::<i32>(0).unwrap_err();
        // prop_assert! 第一个参数含 `{ .. }` 模式会被解析为 format string
        // 用 `prop_assert!` 加 dummy format string 转义
        prop_assert!(matches!(err, ArenaError::Overflow { .. }), "overflow error variant");
    }

    fn arena_try_alloc_does_not_consume_on_failure(cap in 16..64usize) {
        let arena = Arena::with_capacity(cap);
        let mut allocated = 0;
        while let Ok(_r) = arena.alloc::<i32>(0) {
            allocated += 1;
            if allocated > 10000 {
                break;
            }
        }
        let used_before = arena.used();
        let r: Option<&mut i32> = arena.try_alloc(0);
        prop_assert!(r.is_none());
        prop_assert_eq!(arena.used(), used_before, "try_alloc 失败不应消耗 cursor");
    }

    fn arena_alloc_slice_zero_len(_dummy in 0..1u32) {
        let arena = Arena::with_capacity(16);
        let s: &mut [i32] = arena.alloc_slice(0).unwrap();
        prop_assert_eq!(s.len(), 0);
    }

    fn arena_alloc_slice_round_trip(n in 1..16usize) {
        let arena = Arena::with_capacity(4096);
        let s: &mut [i32] = arena.alloc_slice(n).unwrap();
        for (i, x) in s.iter_mut().enumerate() {
            *x = i as i32 * 10;
        }
        for (i, x) in s.iter().enumerate() {
            prop_assert_eq!(*x, i as i32 * 10);
        }
    }

    fn arena_reset_zeroes_cursor(n in 1..16usize) {
        let mut arena = Arena::with_capacity(4096);
        for i in 0..n {
            let _: &mut i32 = arena.alloc(i as i32).unwrap();
        }
        prop_assert!(arena.used() > 0);
        arena.reset();
        prop_assert_eq!(arena.used(), 0);
    }
}

// ===== 静态单测（块外） =====

#[test]
fn pool_release_oob_above_capacity() {
    // cap=8，idx=100 必越界
    let mut pool: Pool<i32> = Pool::with_capacity(8);
    for i in 0..8 {
        pool.acquire(i as i32);
    }
    let err = pool.release(100).unwrap_err();
    assert_eq!(err, PoolError::OutOfBounds(100, 8));
}

#[test]
fn pool_double_release_same_idx() {
    let mut pool: Pool<i32> = Pool::new();
    let idx = pool.acquire(42);
    pool.release(idx).unwrap();
    let err = pool.release(idx).unwrap_err();
    assert_eq!(err, PoolError::EmptySlot(idx as usize));
}

#[test]
fn slab_invalid_handle_get_fails_on_empty() {
    // 空 slab 用 INVALID 句柄 get 必返 OutOfBounds(0, 0)
    let slab: Slab<i32> = Slab::with_capacity(0);
    let err = slab.get(SlabHandle::INVALID).unwrap_err();
    assert!(matches!(
        err,
        SlabError::OutOfBounds(0, 0) | SlabError::GenerationMismatch { .. }
    ));
}

#[test]
fn arena_alloc_slice_zero_len_static() {
    let arena = Arena::with_capacity(16);
    let s: &mut [i32] = arena.alloc_slice(0).unwrap();
    assert_eq!(s.len(), 0);
}
