# GVPE — Performance Budget（性能予算）

Input baseline: `01_requirements.md` GVPE-PERF-*, `05_runtime_design.md` §5.5.

## 14.1 Target frequencies by space

| Space | Target rate | Rationale |
|---|---|---|
| Simulation Space | 60–240Hz | `00_vision.md` §0.3 |
| Vector Space | 1–30Hz or event-triggered | never hot path (GVPE-VEC-001) |
| Graph Space | offline / tooling cadence | never per-frame (GVPE-GPH-003) |

## 14.2 MVP scene budget (GVPE-PERF-001)

Baseline target: hundreds of dynamic rigid bodies (spheres/boxes/planes), sustained 60Hz, single
mid-range CPU core, before multi-threading is credited. This is deliberately a conservative,
easily-falsifiable target — the point is to have a number to fail against early, not to declare
victory prematurely.

## 14.3 Per-stage budget breakdown (to be measured, not assumed)

```
step(dt) budget @ 60Hz = 16.6ms
  Apply Forces        : measure
  Broad Phase          : measure
  Narrow Phase[]        : measure
  Contact Generation    : measure
  Island Build           : measure
  Constraint Solve[]      : measure (dominant cost, expect largest share)
  Integrate[]              : measure
  CCD                       : measure (MVP: near-zero, feature not implemented)
```
No stage gets a target number assigned here without a benchmark harness to measure against —
assigning fictitious budget numbers before `15_testing_strategy.md`'s benchmark harness exists
would be exactly the kind of unfounded precision this whole document set tries to avoid elsewhere.

## 14.4 Regression policy (GVPE-PERF-002)

Any commit that introduces unbounded per-step allocation, lock contention on a hot-path structure,
or a measured regression beyond a to-be-defined threshold against the previous benchmark baseline
is treated as a performance bug, filed the same way a correctness bug would be — not accepted as an
acceptable trade-off for readability or architectural convenience (directly enforces
GVPE-PROHIBIT-06).

## 14.5 Benchmark harness requirement (feeds `15_testing_strategy.md`)

A criterion-style (or equivalent) micro-benchmark suite covering each stage in §14.3 independently,
plus an end-to-end scene benchmark at the MVP scale (§14.2), must exist before AC-01
(`01_requirements.md`) can be considered verifiable rather than merely asserted.
