# GVPE — Testing Strategy（テスト戦略）

Input baseline: `01_requirements.md` Acceptance Criteria, `02_physics_ontology.md` §Review
(ONT-ISS-001), `14_performance_budget.md` §14.5.

## 15.1 Test layers

| Layer | Scope | Example |
|---|---|---|
| Unit | pure functions, math, single solver step | `ConstraintRow` solve for a known two-body case |
| Determinism | Fast Mode repeatability (same build/machine/seed) | AC-01 |
| Regression / benchmark | performance budget (§14) | criterion suite, §14.5 |
| Ontology Review | schema self-consistency | `02_physics_ontology.md` §Review, this document's §15.4 |
| Integration | multi-crate scene simulation | N-body scene reaches expected rest state |
| Dependency isolation | AC-02's compile-time boundary | `cargo tree` check, mirrors the archived PRE project's own lesson about dynamic-vs-hardcoded enumeration |

## 15.2 Determinism testing (AC-01)

Run the same scene twice on the same machine/build; assert bit-identical `SimulationState` output
for every step, with `Graph`/`Vector` features compiled out (GVPE-FR-001). This test is the
concrete falsification target for `05_runtime_design.md` §5.3's Fast Mode claims.

## 15.3 Dependency isolation testing (AC-02)

```
for crate in [gvpe-core, gvpe-collision, gvpe-dynamics, gvpe-constraint,
              gvpe-solver, gvpe-island, gvpe-scheduler, gvpe-runtime]:
    assert "gvpe-graph" not in cargo_tree(crate)
    assert "gvpe-vector" not in cargo_tree(crate)
    assert "gvpe-compiler" not in cargo_tree(crate)
```
Must enumerate crates from `cargo metadata` dynamically, not as a hardcoded list — the archived PRE
project's traceability matrix hit exactly this defect once (a hardcoded enumeration silently missed
new crates); the fix pattern is restated here as a standing test-design rule, not just a one-off
lesson.

## 15.4 Ontology Review as a repeatable check (closes ONT-ISS-001)

A schema validator that: (a) loads `02_physics_ontology.md`'s node/relation types, (b) attempts a
bulk per-frame `State` write directly to `gvpe-graph` and asserts rejection (the concrete
enforcement ONT-ISS-001 says is currently missing), (c) round-trips the MVP-unpopulated ontology
branches (Energy/Wave/Field/Process/PhysicalLaw) through schema validation with zero instances and
asserts no validation error. Passing (b) and (c) is what closes ONT-ISS-001 — until both exist, that
issue stays open per `02_physics_ontology.md`'s own instructions.

## 15.5 Compiler round-trip testing (AC-03)

```
graph_profile = compile(populate_graph(known_material))
manual_profile = PhysicsProfile { ...same values, hand-constructed... }
assert graph_profile == manual_profile
```

## 15.6 Solver correctness fixtures

Known-answer physics cases (two spheres, perfectly elastic collision along one axis; a box resting
on a plane reaching sleep state; etc.) with hand-computable expected outcomes, checked within a
numerical tolerance — these are the self-developed-solver equivalent of the archived PRE project's
`pre-testkit` golden-dataset discipline (generate fixtures from the real solver's known-good runs,
not hand-authored magic numbers), restated here because the reasoning transfers directly: fixtures
must originate from a verified-correct run of the real solver, not be invented.

## 15.7 What's out of scope for this pass

Fuzzing, property-based testing strategy, and CI pipeline configuration are implementation details
appropriate for when `gvpe-core` actually exists as code — listed here only as forward pointers, not
specified.
