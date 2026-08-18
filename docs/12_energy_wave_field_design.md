# GVPE — Energy / Wave / Field / Process Extension Design

Input baseline: `02_physics_ontology.md` §10–§13, `04_architecture.md` §4.5. This document is the
bridge between "the ontology already has these concepts" (true today) and "the runtime can compute
with them" (not true in MVP, and not required to be) — it exists so that bridge is designed once,
deliberately, instead of improvised later under schema pressure.

## 12.1 Why this document exists now, with nothing to implement yet

`01_requirements.md` §11 explicitly excludes Energy/Wave/Field/Process from MVP runtime scope, but
`02_physics_ontology.md` §26 requires their *schema* to exist without future breaking migration.
This document is the proof-of-shape that the schema-to-runtime path is real, not just asserted.

## 12.2 Energy tracking hook (reserved runtime extension point)

```rust
struct EnergyLedger {   // not populated in MVP; the type exists so Solver can be extended later
    kinetic: f32, gravitational_potential: f32, elastic_potential: f32, thermal: f32,
}
```
Conservation-of-energy bookkeeping (`02_physics_ontology.md` §10's conversion relations) would be
computed as a post-step diagnostic pass reading already-integrated body state — it does not require
changing `ConstraintRow` or the solve loop itself, only adding an optional aggregation pass. This is
the concrete evidence that Energy support is additive, not migratory.

## 12.3 Wave propagation hook

`02_physics_ontology.md` §11's `Wave` nodes (frequency, amplitude, propagation_speed, ...) map to a
future `gvpe-wave` crate operating on the same body/contact events the MVP solver already produces
(`Collision GENERATES MechanicalWave`, per the ontology's causal chain) — event-sourced, not a
change to the core solve loop.

## 12.4 Field hook

`02_physics_ontology.md` §12's `Field` types (Gravitational/Pressure/Velocity/...) generalize the
MVP's single hardcoded `Gravity` force (`02_physics_ontology.md` §7) into a queryable spatial
function. MVP's `Gravity` implementation should already be written as "sample a (currently
constant) field at a position" rather than "add a constant vector" — same runtime cost, but the
abstraction is future-compatible with real spatially-varying fields without a rewrite.

## 12.5 Process hook

`02_physics_ontology.md` §13's `Process` types (Melting, Fracture, PhaseTransition, ...) are
inherently multi-frame state machines over an `Entity`. The reserved extension point is a generic
`ProcessState` slot on entities (currently unused in MVP) that a future process-simulation crate
can attach to, without touching `gvpe-dynamics`'s core per-body state layout.

## 12.6 Explicit non-goal for this pass

No numerical solver for any Energy/Wave/Field/Process phenomenon is designed here — only the
*seams* where such a solver would attach. Designing the actual numerics (e.g. a wave equation
solver) before there's a driving use case would itself be the kind of premature complexity
`00_vision.md` §0.2's prohibitions exist to prevent.
