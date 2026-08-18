# GVPE — XPBD / Rope / Cloth / SoftBody / Particle Detailed Design

Input baseline: `07_solver_design.md` §7.2 (Generation 2, reserved), `02_physics_ontology.md` §4
(MechanicalBehavior: Rope/Cloth/Membrane/Rod/Shell/SoftBody/GranularBehavior). MVP does not ship
this solver (`01_requirements.md` NG1), but "complete as a physics engine" requires the *design* to
exist and to be provably compatible with the Generation-1 data model, not just promised in passing.

## 19.1 Why XPBD, and why it doesn't require a new architecture

`ConstraintRow.compliance` (`17_detailed_design.md` §5.1) already carries an XPBD-compatible
compliance parameter — Generation 1 (`gvpe-solver` §6) sets it to `0.0` (rigid), Generation 2 uses
non-zero values directly in the same `delta_lambda /= 1.0 + row.compliance` line
(`17号文書` §6, already written generically). This section designs the *particle* representation
XPBD constraints act over; the *solve loop* is unchanged from `17号文書` §6.

## 19.2 Particle representation (new body kind, not a new solver)

```rust
struct ParticleStateSoA {
    position: Vec<[f32; 3]>, prev_position: Vec<[f32; 3]>,   // XPBD needs both (Verlet-style)
    inv_mass: Vec<f32>,
    velocity: Vec<[f32; 3]>,   // derived post-solve, not integrated directly (XPBD convention)
}
```
Particles are a distinct SoA store from `BodyStateSoA` (`17号文書` §4.1) — rigid bodies and
particles are different `MechanicalBehavior` kinds (`02_physics_ontology.md` §4) with different
integration schemes, and keeping them separate avoids polluting the Generation-1 hot path with
fields Generation 1 never uses.

## 19.3 XPBD solve loop

```rust
fn xpbd_step(particles: &mut ParticleStateSoA, rows: &mut [ConstraintRow], substeps: u32, dt: f32) {
    let h = dt / substeps as f32;
    for _ in 0..substeps {
        predict_positions(particles, h);              // p += v*h + external_accel*h^2 (gravity via the same Field-sample hook, 17号文書 §4.2)
        for row in rows.iter_mut() {
            let c = constraint_value(row, particles);   // e.g. |p_a - p_b| - rest_length for a distance/stretch row
            let alpha_tilde = row.compliance / (h * h);
            let delta_lambda = -(c + alpha_tilde * row.lambda) / (generalized_inv_mass(row, particles) + alpha_tilde);
            row.lambda += delta_lambda;
            apply_position_correction(particles, row, delta_lambda);   // moves p_a/p_b directly, not velocity
        }
        update_velocities(particles, h);   // v = (p - prev_p) / h, then prev_p = p
    }
}
```
This is deliberately parallel in structure to `17号文書` §6's `solve_island` — same
predict→iterate-rows→apply pattern, different unknowns (positions vs. velocities) and a different
`generalized_inv_mass`/`constraint_value` per constraint kind, which is exactly the abstraction
`ConstraintRow` already generalizes over.

## 19.4 Constraint kinds (extends `02_physics_ontology.md` §9's semantic list into XPBD rows)

```rust
enum XpbdConstraintKind {
    Distance { rest_length: f32 },                       // rope segments, cloth structural
    Bending  { rest_angle: f32 },                          // cloth bending resistance
    Volume   { rest_volume: f32 },                          // soft body volume preservation
    Attachment { anchor_world: [f32; 3] },                   // pin a particle to a fixed/kinematic point
}
```

### 19.4.1 Rope: a chain of `Distance` rows between consecutive particles, plus one `Attachment` at
the fixed end (if any).

### 19.4.2 Cloth: a grid of particles, `Distance` rows along structural (horizontal/vertical) and
shear (diagonal) edges, `Bending` rows across each interior edge pair — the same grid topology
`02_physics_ontology.md` §4's `Cloth`/`Membrane` distinguishes by whether bending resistance is
tuned soft (membrane-like) or stiff (shell-like); the constraint *kind* is identical, only
`compliance` differs.

### 19.4.3 SoftBody: a tetrahedral mesh of particles with `Distance` rows on tet edges and `Volume`
rows per tetrahedron (`04号文書` §4.5's `PBDModel`/`XPBDModel` row in the Law→Model→Solver table
gets its concrete Solver entry here).

### 19.4.4 Granular: particles with only `Distance`-style short-range repulsion (no persistent
structural rows) — the closest thing to XPBD "constraints" a granular material has is transient
contact constraints, reusing §19.4's contact-handling path rather than needing a fifth kind.

## 19.5 Collision for soft bodies

Particle-vs-rigid-body and particle-vs-particle contacts reuse `gvpe-collision`'s narrow phase
(`17号文書` §3) with particles treated as zero-radius (or small-radius, for granular) spheres —
no new collision algorithm, only a new `ShapeDesc` variant (`Particle { radius: f32 }`,
`06_collision_design.md` §6.2 extended).

## 19.6 What stays explicitly out of scope

Fluid (SPH or grid-based) and full FEM are **not** designed by this document — `01_requirements.md`
NG1 excludes them from even the reserved-interface treatment §19 gives Rope/Cloth/SoftBody, because
unlike XPBD-family solids, a fluid solver is not a data-model extension of the existing
`ConstraintRow`/particle abstractions; it would need its own numerical method (pressure projection
or SPH kernel evaluation) that doesn't reuse anything in §19.3. That is a future document's job,
written when there's a driving use case, not speculated here (same discipline
`12_energy_wave_field_design.md` §12.6 already applied to Energy/Wave/Field numerics).

## 19.7 Test fixtures owed to `15_testing_strategy.md`

- A rope of N segments dropped under gravity, converging to the expected catenary-like rest shape
  within tolerance.
- A cloth grid pinned at two corners, checked for no explosion (bounded particle velocity) over a
  fixed number of steps — the XPBD-equivalent of `18_joints_ccd_design.md` §18.6's rigid-body
  fixture discipline.

Requirements satisfied: `01_requirements.md` GVPE-FR-002 (extended), `02_physics_ontology.md` §4/§15
(MechanicalBehavior/Model rows now have a concrete runtime path), `04_architecture.md` §4.5 (the
`XPBDModel` table row is no longer "reserved, Phase 6+" without a design behind it).
