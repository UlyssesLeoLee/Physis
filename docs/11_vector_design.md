# GVPE — Vector Design（ベクトル空間設計）

Input baseline: `02_physics_ontology.md` §20, GVPE-VEC-001/002.

## 11.1 Physics Signature — multi-vector, not a single embedding

```
PhysicsSignature
├── MaterialSignature    ├── ContactSignature      ├── EnvironmentSignature
├── MotionSignature       ├── EnergySignature        └── SolverSignature
├── DeformationSignature  ├── WaveSignature
└── InteractionSignature  └── FieldSignature
```
Each sub-signature is independently computable and independently comparable — fusing them into one
score, if ever done, happens at the retrieval layer, not by concatenating into one undifferentiated
vector at extraction time (same reasoning the archived PRE work's Multi-Vector ADR already
established; restated here because it's correct independent of project).

## 11.2 Three signature instances (GVPE-VEC-002, type-distinct)

```rust
struct ObservedPhysicsSignature(PhysicsSignature);   // from Observation
struct SimulatedPhysicsSignature(PhysicsSignature);  // from a Simulation run
struct KnownPhysicsSignature(PhysicsSignature);      // from an already-Validated graph entry
```
Newtype wrappers, not a shared struct with a tag field — prevents accidentally comparing an
`Observed` signature against another `Observed` signature when a `Simulated` one was intended, at
compile time rather than by runtime discipline.

## 11.3 Extraction boundary (never in the hot path — GVPE-VEC-001)

```
gvpe-runtime produces SimulationState (per-frame, hot path)
        │  (offline / 1~30Hz / event-triggered — NOT every step)
        ▼
gvpe-vector extracts PhysicsSignature from a SimulationState snapshot
```
`gvpe-vector` never calls into `gvpe-solver`/`gvpe-collision` synchronously from within `step(dt)`
— it consumes already-produced state snapshots asynchronously or on a slower cadence.

## 11.4 Retrieval

ANN-style similarity search over `KnownPhysicsSignature` entries, returning candidates for
`gvpe-inference`'s hypothesis generation (`13_3dgs_future_design.md`). Retrieval always produces
*candidates*, never a final answer — the closed loop's Comparison/Optimization step
(`13_3dgs_future_design.md` §13.1) is the actual arbiter, mirroring the "retrieval proposes, physics
verifies" principle the archived PRE work already established as correct and worth keeping.

## 11.5 What's explicitly not decided here

Embedding dimensionality, encoder architecture (deterministic feature vector vs. learned), and the
concrete ANN index technology are all deferred to an implementation spike once `gvpe-graph`'s
schema (§03) has enough real instance data to design against — speculating on encoder architecture
before there's data to encode would be premature.
