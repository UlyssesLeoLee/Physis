# GVPE — Memory Design（メモリ詳細設計）

Input baseline: `05_runtime_design.md` §5.2, GVPE-NFR-002.

## 8.1 Hot path allocation policy

Zero/near-zero heap allocation per `step(dt)` call, steady-state (post-warmup). Any allocation
inside `gvpe-collision`/`gvpe-constraint`/`gvpe-solver`'s per-frame path is a defect unless it's
amortized-once (e.g. growing a persistent buffer on first oversized frame, never shrinking/
reallocating every frame after).

## 8.2 Allocation strategies (candidates, applied per subsystem)

| Strategy | Used by |
|---|---|
| Arena | per-frame scratch (contact pairs, manifolds) — reset, not freed, each frame |
| Pool | fixed-size recyclable objects (e.g. `ConstraintRow` slots) |
| Slab | body/entity storage with stable indices across frames |
| Frame Allocator | short-lived per-step temporaries |
| Preallocation | worst-case-sized buffers established at scene setup from `RuntimeDescriptor` body counts |

## 8.3 Buffer shape (feeds §4.6's future-GPU constraint)

All hot-path buffers are flat, contiguous, alignable arrays (SoA per `05_runtime_design.md` §5.1) —
never a `Vec<Box<dyn Trait>>` or any pointer-chasing structure on the hot path. This is what keeps
`04_architecture.md` §4.6's "no GPU redesign needed" claim true.

## 8.4 Scratch buffer lifecycle

```rust
struct FrameScratch { arena: Arena }
impl FrameScratch {
    fn begin_frame(&mut self) { self.arena.reset(); }   // O(1), no dealloc
}
```
`begin_frame` is called once per `step(dt)`, before Broad Phase. Nothing allocated from `arena`
survives past `end_frame` — the borrow checker enforces this by scoping arena-allocated references
to the frame's lifetime.
