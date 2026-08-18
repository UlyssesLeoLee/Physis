# GVPE — Physics Ontology（物理オントロジー）

Input baseline: `01_requirements.md` GVPE-ONT-*, GVPE-GPH-001/002. This is the schema for the
**Physics Knowledge Graph only** (`03_graph_schema.md` §1.A) — it does not describe the Runtime
Constraint Graph or Execution Graph, which are separate documents/sections entirely (§1.B/§1.C
there). Confusing this ontology with runtime data is exactly the failure mode `04_architecture.md`
§4.3 and this document's own Ontology Review (§Review) exist to catch.

## 1. Top-level concepts (must all exist in schema; MVP instance population is a strict subset, §MVP)

```
Physics
├── Entity            ├── Field              ├── SolverModel
├── Matter             ├── Energy             ├── BoundaryCondition
├── Material           ├── Wave               ├── Observation
├── Phase               ├── Process            ├── Experiment
├── Property            ├── PhysicalLaw        ├── Hypothesis
├── State               ├── ConstitutiveModel  ├── Simulation
├── Force               ├── ApproximationModel ├── SimulationState
├── Interaction         ├── PhysicsProfile     ├── Constraint
└── VectorDescriptor    └── Result
```

## 2. Matter — "what something is"

```
Matter
├── SolidMatter    ├── PlasmaMatter     ├── PorousMatter
├── LiquidMatter    ├── GranularMatter   ├── CompositeMatter
├── GasMatter                            └── MultiphaseMatter
```

**Ontology rule (binding)**: Solid/Liquid/Gas are **Phase**, not Matter subtypes. `Water HAS_PHASE
Liquid` is correct; `Water IS_A LiquidMatter` conflates the object with its current state and is
rejected by the Ontology Review (`ONT-ISS` category: Matter/Phase confusion).

## 3. Phase — "what state it's currently in", plus PhaseTransition

```
Phase: Solid | Liquid | Gas | Plasma | Supercritical | GranularState | MultiphaseState | MixedPhase
```

`PhaseTransition` relations: `MELTS_TO`, `EVAPORATES_TO`, `IONIZES_TO`, `FREEZES_TO`,
`CONDENSES_TO`, `SUBLIMATES_TO` — each transition may associate `Temperature`, `Pressure`,
`Energy`, `Rate`, `BoundaryCondition`, `Material` as qualifying context (conditional relations,
§10).

## 4. MechanicalBehavior — "how it behaves mechanically", independent of Matter/Phase

```
MechanicalBehavior: Rigid | Elastic | Plastic | ElastoPlastic | Viscoelastic | Hyperelastic |
                     Brittle | Ductile | SoftBody | Cloth | Membrane | Rope | Rod | Shell |
                     Continuum | GranularBehavior
```

An entity `CAN_BE_MODELED_AS` more than one behavior (`Steel CAN_BE_MODELED_AS RigidBody` *and*
`Steel CAN_BE_MODELED_AS ElastoPlasticSolid`) — the choice is a `PhysicalModel`/`ApproximationModel`
decision (§13/§14), not an ontology fact about Steel itself.

## 5. Property

Categories (leaf enumeration kept, per-property unit/range/confidence table deferred to next pass
per `00_vision.md` §0.6 depth policy — every `Property` node type below carries the same attribute
shape: `value, unit, range, confidence, source, measurement_method, estimation_method, timestamp,
validity, uncertainty`):

- **Mass**: Mass, Density, CenterOfMass, InertiaTensor
- **Mechanical**: YoungModulus, PoissonRatio, ShearModulus, BulkModulus, Hardness, YieldStrength,
  Toughness, Stiffness, Compliance, Damping, Friction, Restitution, Viscosity, SurfaceTension
- **Thermal**: Temperature, HeatCapacity, ThermalConductivity, ThermalExpansion
- **Fluid**: Pressure, Viscosity, Compressibility, FlowRate, Density, SurfaceTension
- **Electromagnetic**: Charge, Conductivity, Permittivity, Permeability

Node/edge decision (`03_graph_schema.md` §2) applies per-property, not per-category: a property
that is measured, sourced, and reused across entities (e.g. a specific `YoungModulus` measurement
from an `Experiment`) is a node; a property value baked directly into a `PhysicsProfile` for one
simulation run is not separately node-ified.

## 6. State — time-indexed snapshot

`Position, Rotation, LinearVelocity, AngularVelocity, Acceleration, ForceState, Momentum,
AngularMomentum, Temperature, Pressure, Density, Stress, Strain, Deformation, PhaseState,
EnergyState, ChargeState` — each `State` node is time-indexed (`State@t0`, `State@t1`, ...).

**Ontology rule**: `State` is "what it currently is"; `Property` is "what it characteristically
has". Confusing a per-frame `Position` with a durable `Property` is an `ONT-ISS` (State/Property
confusion) — and is exactly why raw per-frame `State` must never be bulk-persisted to the graph
(`03_graph_schema.md` §4, GVPE-PROHIBIT-03/04's ontological grounding).

## 7. Force

```
Force: Gravity | ContactForce | FrictionForce | ElasticForce | DragForce | BuoyancyForce |
       PressureForce | ElectromagneticForce | SpringForce | UserAppliedForce
```
Relation: `Force ACTS_ON Entity`.

## 8. Interaction — what happens *between* entities

```
Interaction: Collision | Contact | Friction | Adhesion | Cohesion | Drag | Buoyancy |
             HeatTransfer | Radiation | ElectromagneticInteraction | FluidStructureInteraction |
             ParticleInteraction
```

**Ontology rule**: `Interaction` is inherently binary/multi-entity ("how objects act on each
other"); `Process` (§16) is what happens *to* a single entity over time. Conflating the two is an
`ONT-ISS` (Process/Interaction confusion) — e.g. `Melting` is a `Process` an `Entity` undergoes,
not an `Interaction` between two entities.

## 9. Constraint (ontology layer — semantic, not the runtime row)

```
Constraint: ContactConstraint | DistanceConstraint | JointConstraint | FixedConstraint |
            HingeConstraint | SliderConstraint | VolumeConstraint | StretchConstraint |
            BendingConstraint | AttachmentConstraint | BoundaryConstraint
```

**Binding rule**: a graph `Constraint` node describes *type and semantics*; the *runtime*
constraint row (a numeric solver entry) lives exclusively in the Runtime Constraint Graph
(`03_graph_schema.md` §1.B). No code path may treat a graph `Constraint` node as directly solvable
— it must go through the Compiler (`GVPE-FR-003`).

## 10. Energy — first-class node, with conversion relations

```
Energy: KineticEnergy | GravitationalPotentialEnergy | ElasticPotentialEnergy | ThermalEnergy |
        InternalEnergy | ElectromagneticEnergy | ChemicalEnergy | AcousticEnergy
```
Relations: `CONVERTS_TO`, `TRANSFERS_TO`, `DISSIPATES_TO`, `STORES`, `RELEASES`, `ABSORBS`.

Example causal chain (also see §25):
`GravitationalPotentialEnergy --CONVERTS_TO--> KineticEnergy --(via Collision)--> ElasticEnergy
--DISSIPATES_TO--> ThermalEnergy`

**Ontology rule**: Energy is never a `Matter` subtype and never an `Entity` — it is a first-class
node type of its own (`ONT-ISS` category: Energy/Matter confusion, explicitly checked in Review).

## 11. Wave — independent of Matter, propagates through it

```
Wave: MechanicalWave | AcousticWave | ElectromagneticWave | SurfaceWave | PressureWave |
      ShockWave | ElasticWave
```
Attributes: frequency, wavelength, amplitude, phase, direction, propagation_speed, attenuation,
energy_flux, polarization.
Relations: `PROPAGATES_THROUGH`, `GENERATED_BY`, `REFLECTED_BY`, `REFRACTED_BY`, `ABSORBED_BY`,
`SCATTERED_BY`, `CARRIES_ENERGY`.

Example: `Collision --GENERATES--> MechanicalWave --PROPAGATES_THROUGH--> Solid`.

**Ontology rule**: Wave is not an `Entity` (`ONT-ISS` category: Wave/Entity confusion) — it is a
propagating disturbance associated with, not identical to, the medium it travels through.

## 12. Field — continuous physical quantity over space

```
Field: GravitationalField | ElectromagneticField | PressureField | VelocityField |
       TemperatureField | DensityField | AcousticField | StressField | StrainField
```
Kind: `ScalarField | VectorField | TensorField`. Relations: `Entity EXISTS_IN Field`,
`Field ACTS_ON Entity`.

**Ontology rule**: Field and Force are distinct — a `Field` is the continuous spatial quantity; a
`Force` is what a specific `Entity` experiences as a consequence of existing in that field
(`ONT-ISS` category: Field/Force confusion).

## 13. Process — what happens to an entity over time (single-entity temporal change)

```
PhysicalProcess: Motion | Collision | Deformation | Flow | Oscillation | Vibration | Diffusion |
                 Fracture | Compression | Expansion | Melting | Freezing | Evaporation |
                 Condensation | HeatTransfer | PhaseTransition | WavePropagation |
                 EnergyTransfer | Dissipation
```
Relation: `Entity UNDERGOES Process`. Example: `Ice UNDERGOES Melting PRODUCES LiquidWater`.

## 14. PhysicalLaw — knowledge layer, not an implementation obligation

```
PhysicalLaw: NewtonLaw | ConservationOfMomentum | ConservationOfEnergy | HookeLaw |
             CoulombFriction | NavierStokes | HeatEquation | WaveEquation | MaxwellEquation |
             ConstitutiveLaw
```

**Ontology rule**: a `PhysicalLaw` node existing in the graph does **not** imply the Runtime
implements it — `04_architecture.md` §4.5 tracks which laws have a corresponding `SolverModel`.
Conflating "known law" with "implemented law" is an `ONT-ISS` (Law/Model confusion).

## 15. Model — how an entity is approximated for computation

```
PhysicalModel: RigidBodyModel | ParticleModel | SpringMassModel | PBDModel | XPBDModel |
               ElasticSolidModel | PlasticModel | FEMModel | FluidModel |
               IncompressibleFluidModel | CompressibleFluidModel | ShellModel |
               ReducedOrderModel
```
Relation: `Entity MODELED_BY PhysicalModel`. This is what makes `PhysicsLOD` (§19) possible.

**Ontology rule**: `Model` (a chosen mathematical approximation) is distinct from `Solver` (the
algorithm that numerically solves that model) — e.g. `XPBDModel SOLVED_BY XpbdSolver` is correct;
collapsing Model and Solver into one node type is an `ONT-ISS` (Model/Solver confusion).

## 16. ApproximationModel — accuracy/performance trade-off, LOD-facing

`FullModel | ReducedModel | SimplifiedModel | ProxyModel | LODModel`. Example: `Water` near camera
→ `FullFluidModel`; middle distance → `ParticleApproximation`; far → `SurfaceApproximation`.
Simulation accuracy vs. performance budget is a graph-visible decision (feeds `PhysicsLOD`, §19).

## 17. BoundaryCondition

`FixedBoundary | FreeBoundary | PeriodicBoundary | PressureBoundary | TemperatureBoundary |
VelocityBoundary | CollisionBoundary` — relevant to Fluid/FEM/Wave/Heat/Field, reserved now for
extension without migration later.

## 18. Observation, Experiment, Hypothesis, Simulation

- **Observation**: `CameraObservation | VideoObservation | 3DGSObservation | SensorObservation |
  SimulationObservation | ManualObservation | MeasurementObservation`, each recording `source,
  timestamp, coordinate_system, confidence, noise, resolution, sampling_rate`.
- **Experiment**: relates `Material`, `Property`, `Observation`, `BoundaryCondition`, `Result`.
  Example: `Experiment MEASURES YoungModulus`.
- **Hypothesis**: `Observation SUPPORTS Hypothesis`; `Hypothesis ASSUMES Material`; `Hypothesis
  ASSUMES PhysicsProfile`; `Hypothesis TESTED_BY Simulation`.
- **Simulation / SimulationState / SimulationResult**: `Simulation USES PhysicalModel`; `Simulation
  USES PhysicsProfile`; `Simulation PRODUCES SimulationState`.

**Ontology rule**: `Observation` is evidence *about* reality, never reality itself, and never
directly writable into `State`/`Property` without going through `Hypothesis`/`Experiment`
provenance (`ONT-ISS` category: Observation/Reality confusion).

## 19. Physics LOD (consumer of §15/§16)

`PhysicsLOD: LOD0 Full Simulation | LOD1 Reduced Simulation | LOD2 Approximation | LOD3 Cached
Behavior | LOD4 Static` — selection inputs: distance, screen importance, interaction importance,
simulation budget, observation confidence, gameplay importance. MVP implements LOD0 only; the
descriptor slot for the rest is reserved (`GVPE-FR-007`).

## 20. Physics Signature (Vector Space schema, cross-referenced from `11_vector_design.md`)

```
PhysicsSignature
├── MaterialSignature   ├── ContactSignature      ├── EnvironmentSignature
├── MotionSignature      ├── EnergySignature        └── SolverSignature
├── DeformationSignature ├── WaveSignature
└── InteractionSignature └── FieldSignature
```
Instances: `ObservedPhysicsSignature | SimulatedPhysicsSignature | KnownPhysicsSignature`
(GVPE-VEC-002 requires these be type-distinct, not merely tagged).

## 21. Physics Profile (the only Graph→Runtime handoff shape, GVPE-FR-003)

```
PhysicsProfile { mass, density, inertia, friction, restitution, damping, stiffness, compliance,
                 viscosity, solver_type, solver_iterations, collision_profile,
                 approximation_level }
```
Pipeline: `Physics Knowledge Graph → Physics Compiler → PhysicsProfile → RuntimeDescriptor →
Rust Runtime`. No step may be skipped (`03_graph_schema.md` §3 forbids Runtime→Cypher).

## 22. Causality relation vocabulary (must be conditional-relation-capable, not a bare taxonomy)

```
IS_A, INSTANCE_OF, HAS_MATERIAL, HAS_PHASE, HAS_PROPERTY, HAS_STATE, HAS_ENERGY, EXISTS_IN,
ACTS_ON, INTERACTS_WITH, INTERACTS_VIA, PARTICIPATES_IN, UNDERGOES, GENERATES,
PROPAGATES_THROUGH, TRANSFERS_TO, CONVERTS_TO, DISSIPATES_TO, MODELED_BY, GOVERNED_BY,
SOLVED_BY, APPROXIMATED_BY, OBSERVED_BY, MEASURED_BY, ESTIMATED_BY, INFERRED_FROM,
VALIDATED_BY, DEPENDS_ON, CAUSES, RESULTS_IN, REQUIRES, ENABLES, SUPPRESSES,
INCREASES, DECREASES, AFFECTS
```
Conditional example: `HigherTemperature DECREASES Viscosity` (relation qualified by a monotonic
dependency, not an unconditional edge).

## 23. Spatial relations (3DGS-facing, §13)

`INSIDE, OUTSIDE, INTERSECTS, CONTACTS, ABOVE, BELOW, NEAR, FAR, ATTACHED_TO, CONTAINS,
CONNECTED_TO`

## 24. Temporal relations

`BEFORE, AFTER, DURING, STARTS_AT, ENDS_AT, PERSISTS_UNTIL` — apply to `Observation`,
`SimulationState`, `Process`, `Collision`, `Wave`, `PhaseTransition`.

## 25. Physical Causality — the chain the whole ontology exists to support

```
Cause → Process → StateChange → EnergyTransfer → ObservableEffect
```
Worked example:
```
ExternalForce --CAUSES--> Acceleration --CAUSES--> VelocityChange --CAUSES--> Collision
  --GENERATES--> Deformation --STORES--> ElasticEnergy --RELEASES--> KineticEnergy
  --GENERATES--> SoundWave
```
This chain is the concrete test of §22's relation vocabulary — if a real physical scenario cannot
be expressed as an unbroken chain using only §22's relations, the vocabulary is incomplete and
that's an `ONT-ISS`.

## 26. MVP ontology instance scope (schema above is NOT scoped down — only instance population is)

Populate: `Entity, Material, Phase, Property (subset), PhysicalModel (RigidBodyModel only),
Solver, PhysicsProfile, Simulation, Observation (SimulationObservation only)`.

Schema-present-but-unpopulated in MVP: `Energy, Wave, Field, Process, PhysicalLaw,
BoundaryCondition, Experiment, Hypothesis` and most of `MechanicalBehavior`. These must round-trip
through the schema validator with zero instances without any schema change being required later —
that is the concrete, testable meaning of "must not require a breaking migration" (verified in
`15_testing_strategy.md`).

## Review — Ontology Review (mandatory before this baseline is accepted)

Checked against the eleven confusion categories named throughout this document:

| # | Check | Result | Registered as |
|---|---|---|---|
| 1 | Solid/Liquid/Gas mistaken for object classification | Rejected explicitly in §2 | — |
| 2 | Phase vs Material confused | Distinguished in §2/§3 | — |
| 3 | Energy mistaken for Matter | Distinguished in §10 | — |
| 4 | Wave mistaken for Entity | Distinguished in §11 | — |
| 5 | Field vs Force confused | Distinguished in §12 | — |
| 6 | State vs Property confused | Distinguished in §6 | — |
| 7 | Process vs Interaction confused | Distinguished in §8/§13 | — |
| 8 | Law vs Model confused | Distinguished in §14 | — |
| 9 | Model vs Solver confused | Distinguished in §15 | — |
| 10 | Observation vs Reality confused | Distinguished in §18 | — |
| 11 | Graph Node vs Runtime State confused | Distinguished in §6, §9; enforced in `03_graph_schema.md` §2/§4 | **ONT-ISS-001** (open, see below) |

**ONT-ISS-001**
- Severity: Medium
- Finding: Rule 11 is stated correctly here, but this document alone cannot *enforce* it — enforcement
  is a `gvpe-graph` implementation concern (a write-path guard rejecting bulk per-frame `State`
  writes). Until that guard exists in code, the rule is documentation-only.
- Recommendation: `03_graph_schema.md` §4 must specify the guard's mechanism; `15_testing_strategy.md`
  must include a test that attempts a bulk per-frame write and asserts rejection. Close this issue
  when both exist.
- Affected: `gvpe-graph`, `03_graph_schema.md`, `15_testing_strategy.md`.
