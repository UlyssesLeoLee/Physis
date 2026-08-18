# GVPE — GPU Compute Backend Detailed Design (data-layout deepening of `04_architecture.md` §4.6)

Input baseline: `04_architecture.md` §4.6 (constraint-only: "no GPU compute crate exists yet",
buffer types must be flat/alignable). MVP does not ship `gvpe-gpu` (`01_requirements.md` NG2), but
completeness requires the *design*, at the same struct/algorithm depth as
`17_detailed_design.md`, to exist and to be checkable against the CPU data model — not only
asserted as "wouldn't preclude it."

## 25.1 What "wouldn't preclude it" means concretely

`04号文書` §4.6 stated the constraint without demonstrating it holds. This section demonstrates it:

```rust
// CPU (17_detailed_design.md §4.1) — already this shape, no change needed
struct BodyStateSoA {
    position: Vec<[f32; 3]>, orientation: Vec<[f32; 4]>,
    linear_velocity: Vec<[f32; 3]>, angular_velocity: Vec<[f32; 3]>,
    inv_mass: Vec<f32>, inv_inertia: Vec<[f32; 9]>, sleeping: Vec<bool>,
}
```
Every field is already a flat `Vec` of POD (`#[repr(C)]`-eligible) elements — the check
`gvpe-gpu` would need is: *does any field require pointer-chasing or heap-nested data to read?* No —
`bool` is the only non-GPU-native element (packed as `u32` per lane on upload, §25.3). This is the
concrete audit `04号文書` §4.6 promised but did not perform.

## 25.2 `gvpe-gpu` crate boundary (reserved, not implemented)

```rust
trait GpuSolverBackend {
    fn upload_bodies(&mut self, states: &BodyStateSoA) -> GpuBufferHandle;
    fn upload_constraints(&mut self, rows: &[ConstraintRow]) -> GpuBufferHandle;
    fn dispatch_solve(&mut self, bodies: GpuBufferHandle, rows: GpuBufferHandle, iterations: u32);
    fn download_bodies(&mut self, handle: GpuBufferHandle, out: &mut BodyStateSoA);
}
```
`gvpe-runtime` depends on `GpuSolverBackend` as a trait object behind a Cargo feature
(`gpu-solver`, off by default — same `GVPE-PROHIBIT-06` discipline `23_energy_wave_field_process_
algorithms.md` §23.5 used), never on a concrete Vulkan/D3D type — this is what keeps
`04_architecture.md`'s one-directional dependency rule (§4.3) intact: `gvpe-runtime` doesn't gain a
graphics-API dependency, only an optional trait it can leave unimplemented.

## 25.3 Backend selection (Vulkan compute / D3D12 compute — `00_vision.md`'s stated compatibility)

`00_vision.md`'s cross-engine requirement already commits to Vulkan and D3D compatibility for the
*runtime's host-integration surface* (`10_ffi_design.md`); `gvpe-gpu` reuses that same commitment
for compute rather than introducing a third API:

- `wgpu` (or a hand-rolled thin abstraction over Vulkan/D3D12 compute, decided at implementation
  time, not here — same "don't pick a library before there's code to pick it for" discipline
  `16_dependency_license.md` applies to graph/vector backends) provides one compute-shader source
  (WGSL or cross-compiled) that runs on both Vulkan and D3D12, matching the sequential-impulse
  `solve_island` loop (`17号文書` §6) row-for-row, so CPU and GPU paths are cross-checkable
  (§25.5).
- `bool` fields (`sleeping`) pack to `u32` (`0`/`1`) on `upload_bodies` and unpack on
  `download_bodies` — the only layout transform needed, confirming §25.1's audit.

## 25.4 Why islands, not the whole world, are the GPU dispatch unit

`09_parallel_design.md`'s Physics Islands (`17号文書` §7's Union-Find `build_islands`) are already
the CPU parallelism unit — `dispatch_solve` takes one island's `bodies`/`rows` slice, not the whole
`BodyStateSoA`, for the same reason: islands are independent, so GPU dispatch granularity reuses a
boundary the CPU design already proved is real (no new partitioning scheme invented for GPU).

## 25.5 Determinism implication (extends `05_runtime_design.md` §5.3)

GPU floating-point reduction order is not guaranteed bit-identical to CPU or even across GPU vendors
— `RuntimeDescriptor.determinism_mode` (`17号文書` §1.3) gains an explicit rule: `Deterministic`
mode **must** run on `GpuSolverBackend::none()` (CPU path) unconditionally; only `Fast` mode may
select a GPU backend. This is a one-line addition to §5.3's existing mode table, not a new
mechanism — stated here because `04号文書` §4.6 didn't address it and a physics engine claiming
GPU compatibility without addressing determinism would be an incomplete claim.

## 25.6 Non-goals

- No compute shader source ships in this document set — `dispatch_solve`'s body is unimplemented
  (`todo!()`), matching `21_graph_compiler_detailed_design.md`/`22_vector_detailed_design.md`'s
  precedent of leaving genuinely non-MVP crates at interface depth deliberately.
- No GPU broad/narrow-phase design — `04号文書` §4.6 and this document both scope "GPU" to the
  solver stage only (the stage `07_solver_design.md` already identified as the one with the
  clearest data-parallel structure); GPU collision detection is a separate future document if ever
  justified by a driving use case.

Requirements satisfied: `04_architecture.md` §4.6 (constraint now demonstrated, not only asserted),
`00_vision.md` §0.5 (completeness) and Vulkan/D3D compatibility commitment, `05_runtime_design.md`
§5.3 (determinism-mode rule extended to cover GPU dispatch).
