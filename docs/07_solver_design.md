# GVPE — Solver Design（ソルバー詳細設計）

Input baseline: `06_collision_design.md` §6.4, `04_architecture.md` §4.5.

## 7.1 Generation 1 (MVP): Sequential Impulse / Projected Gauss-Seidel

```rust
struct ConstraintRow {
    body_a: BodyIndex, body_b: BodyIndex,
    jacobian: [f32; 12],          // linear+angular for both bodies
    bias: f32, compliance: f32,
    lambda: f32,                  // accumulated impulse, for warm-starting
    lower: f32, upper: f32,       // impulse bounds (friction cone uses this)
}
```
All constraint types (`ContactConstraint`, friction rows, later `JointConstraint`) unify into
`ConstraintRow` — this is the single row format the solver iterates, regardless of semantic origin
(the semantic origin lives in the graph, per `02_physics_ontology.md` §9's binding rule; the row
here is Runtime Constraint Graph territory only).

Solve loop: warm-start from previous frame's `lambda` → N Gauss-Seidel sweeps over rows within an
island → project impulse bounds each sweep → integrate.

## 7.2 Generation 2 (post-MVP): XPBD

Reserved for Rope, Cloth, SoftBody (`02_physics_ontology.md` §4 MechanicalBehavior types). Not
implemented in MVP; `ConstraintRow`'s compliance field is already XPBD-compatible (XPBD's
compliance parameter maps directly), so this is a solver-swap, not a data-model change, when it
lands.

## 7.3 Friction

Coulomb friction cone approximated as box constraints (bounds derived from normal impulse ×
friction coefficient) within the sequential-impulse loop — standard approach for a from-scratch
Generation-1 solver, avoids a separate friction-cone solve pass.

## 7.4 Sleeping

Bodies whose linear+angular velocity stay below a threshold for N consecutive frames transition to
`Sleeping`; excluded from `gvpe-island` active-body counts until woken by a new contact/force
(`02_physics_ontology.md` §6's `State` sequence — `Sleeping` is a `State`, not a permanent
`Property`).

## 7.5 What this solver explicitly does not do (MVP)

No joint types beyond what's needed to validate the `ConstraintRow` abstraction generically (a
minimal fixed/hinge joint may be added for testing the abstraction, not as a feature commitment).
No CCD in MVP (reserved, listed in the Execution Graph §5.5 as a stage that MVP may no-op).
