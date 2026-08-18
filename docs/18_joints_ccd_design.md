# GVPE — Joints & CCD Detailed Design（ジョイント・CCD 詳細設計）

Input baseline: `07_solver_design.md` §7.5 (explicitly deferred), `17_detailed_design.md` §5–§6.
This closes the gap `17_detailed_design.md` left open: a physics engine without joints and CCD is
not feature-complete, and `07号文書` §7.5 flagged both as "not a feature commitment" for the first
pass — this document turns that commitment into a concrete design so it is no longer missing.

## 18.1 Joint types (extends `02_physics_ontology.md` §9's Constraint taxonomy into runtime rows)

```rust
enum JointRow {
    Fixed   { anchor_a: [f32; 3], anchor_b: [f32; 3], anchor_rows: [ConstraintRow; 3], angular_rows: [ConstraintRow; 3] },
    Distance{ anchor_a: [f32; 3], anchor_b: [f32; 3], rest_length: f32, row: ConstraintRow },
    Hinge   { anchor_a: [f32; 3], anchor_b: [f32; 3], axis_a: [f32; 3], axis_b: [f32; 3],
              point_rows: [ConstraintRow; 3], perp_axis_rows: [ConstraintRow; 2],
              limit_row: Option<ConstraintRow> },
    Slider  { anchor_a: [f32; 3], anchor_b: [f32; 3], axis: [f32; 3],
              perp_rows: [ConstraintRow; 2], limit_row: Option<ConstraintRow> },
}
```
Every joint decomposes into one or more `ConstraintRow`s (`17_detailed_design.md` §5.1) — the
solver in §18.4 never special-cases joint *kinds*, only iterates rows. This reuses
`gvpe-solver`'s existing Sequential Impulse loop unmodified (`17号文書` §6), which is the concrete
proof that adding joints did not require a new solver.

## 18.2 Building joint rows

```rust
fn build_hinge_rows(hinge: &HingeSpec, state: &BodyStateSoA) -> JointRow {
    // Point constraint: anchor_a (in body A's frame) must coincide with anchor_b (in body B's frame)
    //   → 3 ConstraintRow with jacobians derived from the anchor offset cross products (identical
    //     structure to a 3-axis Distance constraint with rest_length = 0)
    // Perpendicular-axis constraint: hinge axis_a and axis_b must stay parallel
    //   → 2 ConstraintRow (rotation about the two axes perpendicular to the hinge axis is locked)
    // Optional limit: angle around the hinge axis clamped to [min, max]
    //   → 1 ConstraintRow with lower/upper set from the current angle vs. limit, bias only active
    //     when the angle approaches a limit (inactive constraint outside the limit band)
    JointRow::Hinge { /* ... constructed from the above ... */ }
}
```
Limit rows use the same `lower`/`upper` clamping mechanism contact friction rows already use
(`17号文書` §6), not a separate mechanism — one clamped-impulse abstraction serves both.

## 18.3 Joint lifecycle

Joints are created/destroyed through `gvpe-runtime`'s API (`05_runtime_design.md` §5.4), producing
`ConstraintHandle`s (`17号文書` §1.1) with the same generational-index safety as bodies. A joint
whose referenced body is destroyed is automatically invalidated (checked via `generation` mismatch)
rather than left dangling.

## 18.4 Solver integration

```rust
fn island_constraint_rows(island: &Island, contacts: &[ConstraintRow], joints: &[ConstraintRow]) -> Vec<ConstraintRow> {
    // Joint rows and contact rows are concatenated into one flat slice before solve_island()
    // (17号文書 §6) — the solver has no joint-awareness, only row-awareness.
    let mut rows = contacts.to_vec();
    rows.extend_from_slice(joints);
    rows
}
```
This is why `17_detailed_design.md` §5.1's `ConstraintRowKind` enum already had room reserved
(`/* Joint 系は post-MVP */`) — this document fills that reservation without changing the enum's
shape, only adding variants.

## 18.5 CCD (Continuous Collision Detection)

MVP explicitly no-ops CCD (`07号文書` §7.5); this section designs it so a physics engine claiming
completeness has an actual CCD algorithm, not just a stage that does nothing.

### 18.5.1 Trigger condition

```rust
fn needs_ccd(body: &BodyStateSoA, i: usize, dt: f32, shape_radius: f32) -> bool {
    let travel = length(scale(body.linear_velocity[i], dt));
    travel > shape_radius * CCD_TRAVEL_RATIO   // e.g. body would move more than its own radius
}
```
Only bodies exceeding this threshold pay the CCD cost — most bodies in most frames skip it
entirely, keeping the common case cheap (consistent with `14_performance_budget.md`'s "measure,
don't assume" discipline: `CCD_TRAVEL_RATIO` is a tunable constant to be calibrated against real
benchmark data, not hand-picked here).

### 18.5.2 Algorithm: conservative advancement

```rust
fn ccd_resolve(body: &mut BodyRecord, others: &[BodyRecord], dt: f32) -> f32 {
    // 1. Compute a conservative time-of-impact (TOI) lower bound using the swept shape
    //    (sphere-swept for MVP shapes: treat the body's motion as a capsule sweep even for
    //    box/sphere, using the shape's bounding sphere — a standard conservative approximation)
    // 2. Advance the body to just before TOI, re-run narrow phase at that sub-position
    // 3. If a real contact is found, generate it into this frame's manifold list (feeds
    //    gvpe-constraint §5.2, 17号文書) instead of the tunneled post-integration position
    // 4. If no real contact (false positive from the conservative bound), fall back to full dt
    conservative_advancement_loop(body, others, dt, MAX_CCD_ITERATIONS)
}
```
CCD output is a corrected position/TOI fed back into the same integration step
(`17_detailed_design.md` §4.2) — it does not bypass the constraint solver, it prevents tunneling
before the solver ever sees the (otherwise already-penetrated-through) state.

### 18.5.3 Execution Graph placement

CCD's stage in `05_runtime_design.md` §5.5's step breakdown (`... → Integrate → CCD → Output`) runs
*after* the main integrate pass, operating only on the subset of bodies flagged by §18.5.1 — this
keeps the Execution Graph's shape (`03_graph_schema.md` §1.C) unchanged, CCD is a conditional
fan-out within an existing stage, not a new stage type.

## 18.6 Test fixtures owed to `15_testing_strategy.md`

- A fast-moving sphere that would tunnel through a thin plane without CCD, with-CCD case must not
  tunnel (regression fixture).
- A hinge joint holding two bodies at a fixed angular offset under gravity, verified against the
  hand-computable expected rest configuration (same "verified real-solver-run, not hand-authored
  magic numbers" discipline `15号文書` §15.6 already established).

Requirements satisfied: `01_requirements.md` GVPE-FR-002 (now explicitly covering joints/CCD, which
that requirement's original text left implicit), `00_vision.md` §0.5 (a physics engine claiming
completeness needs both).
