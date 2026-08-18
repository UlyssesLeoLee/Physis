# GVPE — Energy / Wave / Field / Process Runtime Algorithms

Input baseline: `12_energy_wave_field_design.md` (hooks only), `02_physics_ontology.md` §10–§13.
`12号文書` §12.6 deliberately stopped at "the seams where a solver would attach" — this document is
the numerics that attach to those seams, owed now that the goal is full physics-engine completeness
rather than MVP-minimal scope. These remain **off** by default (feature-gated), consistent with
`01_requirements.md` NG1's scope discipline — "designed and implemented behind a flag" is different
from "in the MVP hot-path budget by default", and this document is careful to keep that distinction.

## 23.1 Energy ledger — concrete computation

```rust
fn compute_energy_ledger(state: &BodyStateSoA, profiles: &[PhysicsProfile], gravity: [f32; 3]) -> EnergyLedger {
    let mut ledger = EnergyLedger::default();
    for i in 0..state.position.len() {
        if state.sleeping[i] { continue; }
        let m = 1.0 / state.inv_mass[i].max(f32::EPSILON);
        ledger.kinetic += 0.5 * m * length_sq(state.linear_velocity[i])
                         + 0.5 * dot(state.angular_velocity[i], apply_inertia(&state.inv_inertia[i], state.angular_velocity[i]));
        ledger.gravitational_potential += -m * dot(gravity, state.position[i]);   // relative to y=0 reference plane
    }
    ledger
}
```
Called as an optional post-`Integrate` diagnostic pass (`05_runtime_design.md` §5.5's step
breakdown gains one conditional stage) — never inside the solver loop itself, so
`GVPE-PROHIBIT-06` (no sacrificing real-time performance) is respected: with the feature disabled
this function is never called and costs nothing.

### 23.1.1 Conservation check (validates `02_physics_ontology.md` §10's conversion relations)

```rust
fn energy_conservation_error(before: &EnergyLedger, after: &EnergyLedger, dissipated_estimate: f32) -> f32 {
    let total_before = before.total();
    let total_after = after.total() + dissipated_estimate;   // dissipated (friction/restitution<1) must be accounted for
    (total_after - total_before).abs() / total_before.max(f32::EPSILON)
}
```
`dissipated_estimate` is accumulated during the solver's friction rows (`17_detailed_design.md`
§6 — friction impulse work is already computable from `applied * relative_velocity` at the point
the impulse is applied) — this is the concrete implementation of the causal chain
`02_physics_ontology.md` §25 describes (`...ElasticEnergy --DISSIPATES_TO--> ThermalEnergy`),
turned into a checkable number rather than only a graph relation.

## 23.2 Wave propagation — event-sourced impulse model (not a full wave-equation PDE solver)

```rust
struct WaveEvent { origin: [f32; 3], t_emitted: f32, kind: WaveKind, initial_amplitude: f32,
                    propagation_speed: f32 }

fn sample_wave_amplitude(events: &[WaveEvent], at: [f32; 3], t_now: f32) -> f32 {
    events.iter().map(|e| {
        let dist = length(sub(at, e.origin));
        let travel_time = dist / e.propagation_speed;
        let arrived_at = e.t_emitted + travel_time;
        if t_now < arrived_at { return 0.0; }
        let age = t_now - arrived_at;
        e.initial_amplitude * attenuation(dist) * decay_envelope(age)   // simple inverse-square + exponential decay
    }).sum()
}
```
`Collision GENERATES MechanicalWave` (`02_physics_ontology.md` §11's worked example) becomes: every
contact event above an impulse threshold emits a `WaveEvent`. This is deliberately **not** a
discretized wave-equation solver (no grid, no FDTD) — it is an analytic point-source approximation,
adequate for gameplay-facing audio/vibration cues, explicitly insufficient for acoustically precise
simulation. That limitation is stated here rather than discovered later, per the same "don't
speculate on numerics without a driving use case" discipline `12号文書` §12.6 already established —
a full wave-equation solver is out of scope for the same reason fluid/FEM are (see `24号文書`).

## 23.3 Field sampling — generalizing the existing gravity hook

```rust
trait FieldSampler { fn sample(&self, at: [f32; 3], t: f32) -> [f32; 3]; }

struct UniformField(pub [f32; 3]);              // MVP's constant-gravity case
impl FieldSampler for UniformField { fn sample(&self, _: [f32; 3], _: f32) -> [f32; 3] { self.0 } }

struct RadialField { center: [f32; 3], strength: f32, falloff: FalloffKind }   // e.g. explosion/vortex fields
impl FieldSampler for RadialField {
    fn sample(&self, at: [f32; 3], _: f32) -> [f32; 3] {
        let d = sub(at, self.center); let r = length(d).max(f32::EPSILON);
        scale(normalize(d), self.strength * apply_falloff(self.falloff, r))
    }
}
```
`17_detailed_design.md` §4.2's `integrate` already calls `scale(gravity, 1.0)` framed as "sample a
field" — this section is that promise fulfilled: `gravity: [f32; 3]` in `RuntimeDescriptor`
(`17号文書` §1.3) becomes `gravity: Box<dyn FieldSampler>` (with `UniformField` as the MVP default,
zero behavioral change), and `integrate` calls `gravity.sample(state.position[i], t)` instead of
using the constant directly. This is the exact "no rewrite required" claim §12.4 made, demonstrated
concretely rather than only asserted.

## 23.4 Process state machines — worked example (Melting)

```rust
enum ProcessState { Idle, InProgress { started_at: f32, energy_accumulated: f32 }, Complete }

struct MeltingProcess { entity: EntityId, energy_required: f32, state: ProcessState }

fn tick_melting(p: &mut MeltingProcess, incoming_thermal_energy: f32, t_now: f32) {
    match &mut p.state {
        ProcessState::Idle if incoming_thermal_energy > 0.0 =>
            p.state = ProcessState::InProgress { started_at: t_now, energy_accumulated: incoming_thermal_energy },
        ProcessState::InProgress { energy_accumulated, .. } => {
            *energy_accumulated += incoming_thermal_energy;
            if *energy_accumulated >= p.energy_required {
                p.state = ProcessState::Complete;
                // emits: Entity UNDERGOES Melting PRODUCES LiquidWater (02号文書 §16's worked example)
                // -- written to gvpe-graph as a semantic State-transition event via 21号文書 §21.3's
                //    guarded write path (a single Process-completion event, not bulk per-frame data,
                //    so it passes the BULK_STATE_WRITE_THRESHOLD check trivially)
            }
        }
        _ => {}
    }
}
```
`incoming_thermal_energy` would be sourced from a future heat-transfer pass (not designed here —
out of scope for the same reason as §23.2's wave equation) or, for a driving MVP-adjacent use case,
directly scripted by a host application. The reserved `ProcessState` slot `12_energy_wave_field_
design.md` §12.5 promised is this enum, attached per-entity without touching `gvpe-dynamics`'s core
`BodyStateSoA` layout (`17号文書` §4.1) — confirmed here by construction: `MeltingProcess` is a
separate side-table keyed by `EntityId`, not a new field on `BodyStateSoA`.

## 23.5 Feature-gating (keeps `GVPE-PROHIBIT-06` intact)

```toml
[features]
energy-ledger = []
wave-propagation = []
field-sampling = []   # UniformField path has zero cost even when enabled; RadialField etc. opt-in per-scene
process-simulation = []
```
None of these are in the `default` feature set — `01_requirements.md` AC-01's determinism test and
`14_performance_budget.md`'s benchmarks both run with these disabled, so their existence cannot
regress the MVP performance/determinism baseline even if their numerics are later found to need
revision.

Requirements satisfied: `02_physics_ontology.md` §10–§13 (schema→runtime bridge from `12号文書` now
has concrete algorithms), `00_vision.md` §0.5 (completeness), GVPE-PROHIBIT-06 (opt-in, zero cost
when disabled).
