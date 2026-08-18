# GVPE — FFI Design（C ABI設計）

Input baseline: `05_runtime_design.md` §5.4, GVPE-FR-005. Note: this reuses the same C-ABI
discipline the archived PRE spec's Tier-2 design already worked out (`docs/archive/`) — that
discipline is engine-agnostic and correct independent of which project owns it, so it is restated
here rather than re-derived from scratch.

## 10.1 Principle: C ABI First

All host bindings (Unity, Unreal, Godot, custom C++ engines) go through one `gvpe-ffi` C ABI
surface — no per-engine bespoke FFI layer.

## 10.2 Surface shape

```c
typedef struct GvpeContext GvpeContext;   /* opaque handle */

typedef struct { float x, y, z; } GvpeVec3;
typedef struct { float x, y, z, w; } GvpeQuat;
typedef struct { GvpeVec3 position; GvpeQuat rotation; GvpeVec3 linear_vel; GvpeVec3 angular_vel; } GvpeBodyState;

uint32_t gvpe_abi_version(void);
GvpeContext* gvpe_context_create(const GvpeRuntimeDescriptor* desc);
void gvpe_context_destroy(GvpeContext* ctx);
void gvpe_step(GvpeContext* ctx, float dt);

/* Batch, not per-body: one call moves N bodies' state */
void gvpe_get_body_states(GvpeContext* ctx, const uint32_t* handles, size_t count, GvpeBodyState* out);
```

## 10.3 Binding rules (identical class of constraints as any C ABI, restated for this engine)

- Only opaque handles and POD structs cross the boundary — no Rust generics, trait objects,
  payload-carrying enums, `Result`, `String`, `Vec` directly.
- Every `extern "C"` function wraps its body in `catch_unwind`; panic must never unwind across the
  boundary (undefined behavior otherwise).
- All bulk data exchange (`10.2`'s `gvpe_get_body_states`) is batched — no per-body FFI call in a
  loop from the host side; that pattern defeats the entire purpose of a low-overhead ABI.
- `gvpe_abi_version()` must be checked by every host binding at startup.

## 10.4 Per-engine binding layers (thin, generated or hand-written on top of §10.2)

Unity (C#, P/Invoke), Unreal (C++, direct link), Godot (GDExtension, Rust-native — may skip the C
ABI entirely and link `gvpe-core` directly if Godot's Rust binding permits, in which case it is not
bound by §10.3's POD-only rule for that specific integration path, since no C boundary is actually
crossed).
