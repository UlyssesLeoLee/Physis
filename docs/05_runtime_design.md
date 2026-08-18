# GVPE — Runtime Design（ランタイム詳細設計）

Input baseline: `04_architecture.md` §4.1/§4.3, GVPE-NFR-001/002.

## 5.1 Data layout priority

Preference order: **SoA → AoSoA → Chunk → SIMD Block**, over AoS, for all hot-path body/contact
data in `gvpe-dynamics`/`gvpe-constraint`. Rationale: cache-line utilization and SIMD-lane fill
both degrade under AoS at the body counts this engine targets (§14 performance budget).

## 5.2 Hot / Warm / Cold data classification

| Class | Examples | Access pattern | Layout consequence |
|---|---|---|---|
| Hot | position, velocity, contact impulse | every substep, every body | SoA, cache-line aligned, packed near scheduler-touched data |
| Warm | mass, inertia, material profile refs | read-mostly, per-frame | SoA, may share cache lines with Hot if space allows |
| Cold | debug names, provenance refs, LOD metadata | rare, tooling-facing | separate allocation, never interleaved with Hot |

Layout decisions consider: cache line size, alignment, false sharing (especially across
scheduler-parallel islands, `09_parallel_design.md`), memory bandwidth, target SIMD width. NUMA is
explicitly a future extension, not a v0.1 concern — noted so it isn't silently designed against
later.

## 5.3 Determinism modes (GVPE-NFR-001)

```rust
enum DeterminismMode { Fast, Deterministic }
```

| Concern | Fast Mode | Deterministic Mode |
|---|---|---|
| Floating point | platform default, may use FMA/fast-math | fixed evaluation order, no FMA reordering |
| SIMD | free to vectorize with platform-optimal width | fixed-width, fixed-order reduction |
| Thread ordering | unordered island solve | fixed island processing order |
| Reduction order | unspecified | specified, tested |
| Constraint order | insertion order, may vary with parallel build | canonical sort key, stable |
| Cross-architecture | not guaranteed | same-architecture guarantee only (no cross-platform bit-exactness promise, per OQ-01) |

MVP implements `Fast` fully; `Deterministic` is architecturally reserved (the enum and the
solver-order abstraction exist) but its full guarantee is **not** an MVP acceptance gate — this is
explicitly flagged as OQ-01 in `01_requirements.md` §14, not silently deferred.

## 5.4 Runtime API shape (host-facing, pre-FFI)

```rust
struct GvpeContext { /* opaque, no globals — every instance independent */ }

impl GvpeContext {
    fn new(descriptor: RuntimeDescriptor) -> Self;
    fn step(&mut self, dt: f32);                 // host drives the loop, not GVPE
    fn body_state(&self, handle: BodyHandle) -> BodyState;
    fn set_determinism_mode(&mut self, mode: DeterminismMode);
}
```
No global/static mutable state anywhere in `gvpe-runtime` — multiple `GvpeContext` instances in one
process must be independent (same embeddability discipline the archived PRE work already
established for its own runtime, carried forward here because it's a correct requirement
independent of which project it's attached to).

## 5.5 Frame step breakdown (maps to the Execution Graph, `03_graph_schema.md` §1.C)

`step(dt)` internally runs: Apply Forces → Broad Phase → Narrow Phase → Contact Generation →
Island Build → Constraint Solve → Integrate → CCD → Output. Each stage's budget is tracked in
`14_performance_budget.md`.
