# GVPE — 3DGS Physics Inverse Future Design

Input baseline: `11_vector_design.md`, `02_physics_ontology.md` §18. Explicitly not an MVP blocker
(`01_requirements.md` §11, NG2) — this document exists so the closed loop's shape is designed once
and doesn't force a rework of §02/§03/§04 when it's eventually built.

## 13.1 The closed loop

```
Dynamic 3DGS → Temporal Feature → Motion Feature → Deformation Feature
    → ObservedPhysicsSignature → Vector Retrieval → Graph Hypothesis
    → Candidate PhysicsProfile → Rust Simulation → SimulatedPhysicsSignature
    → Error → Parameter Optimization
```

## 13.2 Module boundary

`gvpe-3dgs` ingests 3DGS reconstruction output (external to GVPE — no 3DGS reconstruction is
implemented by this project) and produces `Observation` graph nodes plus raw feature data feeding
`11_vector_design.md`'s extraction boundary. It never touches `gvpe-runtime`/`gvpe-solver`
directly — same Compiler-mediated boundary as every other Graph/Vector-space consumer
(`04_architecture.md` §4.3/§4.4).

## 13.3 Hypothesis-driven simulation loop

```
Observation --SUPPORTS--> Hypothesis --ASSUMES--> PhysicsProfile
Hypothesis --TESTED_BY--> Simulation --PRODUCES--> SimulationState
  --(extract)--> SimulatedPhysicsSignature
  --(compare against ObservedPhysicsSignature)--> Error
  --(feeds)--> gvpe-inference parameter optimization --> refined PhysicsProfile
```
This is a direct instantiation of `02_physics_ontology.md` §18's `Hypothesis`/`Simulation` schema —
no new graph node types are required for this loop, which is itself evidence the ontology's §26
"no breaking migration" claim holds for this specific future extension.

## 13.4 What's deliberately unspecified

Which optimization algorithm (gradient-based, CMA-ES, Bayesian) drives the refinement step; how
many retrieval candidates get simulated per iteration; how convergence is judged. These are
`gvpe-inference` implementation details that should be decided against real Observation data, not
speculated here.

## 13.5 Non-blocking guarantee

Every module this document names (`gvpe-3dgs`, and the `gvpe-inference` optimization loop) sits
strictly above `gvpe-compiler` in the dependency graph (`04_architecture.md` §4.3) — their absence
or incompleteness cannot affect `01_requirements.md` AC-01/AC-02/AC-03, which are Simulation-Space-
only acceptance criteria.
