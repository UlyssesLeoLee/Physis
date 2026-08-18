# GVPE — Fluid & FEM Interface Reservation

Input baseline: `19_softbody_xpbd_design.md` §19.6, `02_physics_ontology.md` §4
(`MechanicalBehavior`: `FluidBehavior`, `DeformableBehavior` FEM case). `19号文書` §19.6 named this
gap but deliberately did not design it. This document exists so "out of scope" is a documented,
bounded position — with a real interface reservation and an explicit non-goal list — rather than a
silent absence, which is what "must be comprehensive" (`00_vision.md` §0.5) requires even for
things GVPE chooses not to build yet.

## 24.1 Why Fluid and FEM are not a `ConstraintRow`/`ParticleStateSoA` extension

`19号文書` §19.1 could reuse `ConstraintRow.compliance` for XPBD because rope/cloth/soft-body are
all *sparse constraint networks* over particles — the same iterate-rows abstraction, different
constraint kinds. Fluid and full FEM are not sparse-constraint problems in the same sense:

- **Fluid** (SPH or grid-based) needs a *neighbor-density field* recomputed every substep
  (`gvpe-collision`'s broad phase finds pairs, not continuous density) and a pressure-projection or
  kernel-sum step with no `ConstraintRow` analogue — the "constraint" is implicit in the pressure
  field, not an explicit row.
- **FEM** (general, not the tetrahedral-volume-preservation special case `19号文書` §19.4.3 already
  covers) needs an assembled global stiffness matrix and a linear solve per step — a fundamentally
  different numerical structure (sparse linear algebra) from Sequential Impulse/XPBD's local
  iterate-rows loop.

Forcing either into `ConstraintRow` would be the "elegance over performance" mistake
`00_vision.md` GVPE-PROHIBIT-06 forbids — reusing an abstraction because it exists, not because it
fits.

## 24.2 What is reserved today (interface, not implementation)

```rust
enum ShapeDesc {
    // ... 20_shape_advanced_design.md §20.1 variants ...
    FluidRegion { bounds: Aabb, kind: FluidKind },   // RESERVED — no runtime behavior yet
}

enum MechanicalBehaviorHint { RigidBody, Xpbd, Fluid, GeneralFem }   // graph-facing only, §24.4
```

`FluidRegion` is accepted by `gvpe-shape`'s type system today (so graph/tooling code that wants to
*describe* a fluid volume compiles) but `gvpe-dynamics`/`gvpe-solver` have no code path that
consumes it — `PhysicsProfile::solver_type` (`17_detailed_design.md` §1.2) has no `Fluid` or
`GeneralFem` variant in `SolverTypeId`, so `21_graph_compiler_detailed_design.md` §21.4's `compile()`
returns `CompileError::UnsupportedModel("FluidBehavior")` for any graph node requesting one — the
same explicit-failure discipline §21.4 already established, applied here to a case that's
deliberately unimplemented rather than a graph-authoring mistake.

## 24.3 What a future document would need to design (not designed here)

Listed so the gap is bounded, not open-ended:

1. **Fluid**: neighbor search strategy (grid-hash vs. reuse of `gvpe-collision`'s SAP), SPH kernel
   choice or grid pressure-projection method, surface reconstruction (if rendering-facing), and how
   fluid-rigid coupling feeds force back into `BodyStateSoA` without becoming a second solver that
   fights Sequential Impulse/XPBD over the same bodies in the same frame.
2. **General FEM**: element type support (tet/hex), stiffness matrix assembly, sparse linear solver
   choice (direct vs. iterative — a determinism-mode implication per `05_runtime_design.md` §5.3,
   since sparse iterative solvers have their own convergence-tolerance determinism story), and
   whether it runs at Simulation-Space rate at all or is Vector/Graph-Space-adjacent "expensive,
   infrequent" physics (`00_vision.md` §0.3's Vector Space cadence, by analogy).

Both require a driving use case before design, per the discipline `12_energy_wave_field_design.md`
§12.6 and `19号文書` §19.6 already applied — this document intentionally stops at the interface
reservation and the design questions, not answers to them.

## 24.4 Graph-side reservation (schema-complete, runtime-absent — same pattern as Energy/Wave/Field)

`02_physics_ontology.md` §4's `FluidBehavior` and the FEM case of `DeformableBehavior` remain valid
`NodeKind::Entity`-attached `MechanicalBehavior` values in `gvpe-graph` (§21.1's `NodeKind` enum is
unaffected — `MechanicalBehavior` is an attribute value, not a `NodeKind` variant) — a knowledge
graph can *record* that some material is a fluid or an FEM-modeled deformable without GVPE's
runtime being able to simulate it yet. This is the same "ontology is schema-complete, runtime
catches up later" position `01_requirements.md`'s MVP graph scope already takes for Energy/Wave/
Field/Process nodes.

## 24.5 Non-goals (explicit, so nothing reads this as a promise)

- No SPH/grid-fluid solver ships in any version this document set describes.
- No general FEM solver ships in any version this document set describes.
- `FluidRegion`/`MechanicalBehaviorHint::Fluid|GeneralFem` existing in the type system is not a
  commitment to a delivery date — it is the minimum interface surface needed for graph/tooling code
  to describe these materials today without a breaking type change when a runtime is eventually
  designed.

Requirements satisfied: `01_requirements.md` NG1 (fluid/FEM explicitly out of MVP, now with a
documented boundary rather than an implicit one), `00_vision.md` §0.5 (completeness — the gap is
now bounded and interfaced, not silent), `02_physics_ontology.md` §4 (`FluidBehavior`/FEM case
remain valid schema with a stated runtime status).
