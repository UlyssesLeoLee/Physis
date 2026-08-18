# GVPE — Vector Space Detailed Design

Input baseline: `11_vector_design.md`, `17_detailed_design.md` §12 (interface-only).

## 22.1 Signature extraction (deterministic, V1 — mirrors the archived PRE project's own V1
Encoder choice, restated here because the reasoning transfers: interpretable-first, learned-later)

```rust
fn extract_signature(snapshot: &SimulationStateSnapshot) -> PhysicsSignature {
    PhysicsSignature {
        motion: MotionSignature {
            mean_speed: mean(snapshot.bodies.iter().map(|b| length(b.linear_velocity))),
            peak_speed: max(snapshot.bodies.iter().map(|b| length(b.linear_velocity))),
            angular_energy_proxy: sum(snapshot.bodies.iter().map(|b| length_sq(b.angular_velocity))),
        },
        contact: ContactSignature {
            contact_count: snapshot.contact_events.len() as f32,
            mean_restitution: mean(snapshot.contact_events.iter().map(|c| c.restitution_estimate)),
        },
        material: MaterialSignature::from_profiles(&snapshot.profiles_used),   // direct copy, not inferred (02号文書 §5's Property node values)
        // deformation/interaction/energy/wave/field/environment/solver sub-signatures:
        // all-zero / None in MVP (no soft-body, energy-ledger, wave, or field runtime yet --
        // 19/12号文書 respectively), populated once those subsystems produce data to extract from
        ..PhysicsSignature::default()
    }
}
```
Deterministic aggregation (mean/max/sum), no learned parameters — same rationale the archived PRE
project's ADR gave for its V1 encoder: interpretable and testable first, a learned encoder is a
later, evidence-driven upgrade, not a first-pass requirement.

## 22.2 Index structure (fallback: flat scan; ANN only once data volume justifies it)

```rust
struct VectorIndex { entries: Vec<(EntityId, PhysicsSignature)> }

impl VectorIndex {
    fn search(&self, query: &KnownPhysicsSignature, top_n: usize) -> Vec<RetrievalCandidate> {
        // MVP/early-scale path: flat linear scan with per-sub-signature cosine similarity,
        // fused via configurable weights (never a single blended vector -- 11号文書 §11.1)
        let mut scored: Vec<_> = self.entries.iter()
            .map(|(id, sig)| (*id, fused_similarity(&query.0, sig, &DEFAULT_WEIGHTS)))
            .collect();
        scored.sort_by(|a, b| b.1.total_cmp(&a.1));
        scored.truncate(top_n);
        scored.into_iter().map(RetrievalCandidate::from).collect()
    }
}

fn fused_similarity(a: &PhysicsSignature, b: &PhysicsSignature, w: &SignatureWeights) -> f32 {
    w.motion * cosine(&a.motion.as_vec(), &b.motion.as_vec())
        + w.contact * cosine(&a.contact.as_vec(), &b.contact.as_vec())
        + w.material * cosine(&a.material.as_vec(), &b.material.as_vec())
        // remaining sub-signature terms, zero-weighted while their producers are unimplemented
}
```
A flat scan is the deliberate starting point (not an oversight) — `11_vector_design.md` §11.5
explicitly defers ANN technology choice until there's real instance data to design an index
against; this document's job is only to make sure the *interface* (`search`) doesn't change shape
when the *implementation* swaps from flat scan to a real ANN structure later.

## 22.3 Retrieval never decides, only proposes

```rust
struct RetrievalCandidate { entity: EntityId, similarity: f32 /* NOT a final answer, see 13号文書 §13.3 */ }
```
`RetrievalCandidate` is explicitly not consumed directly as ground truth anywhere — it only ever
feeds `gvpe-inference`'s Hypothesis→Simulation→Comparison loop (`13_3dgs_future_design.md` §13.3),
which is where an actual physical answer gets validated. This mirrors the archived PRE project's
"retrieval proposes, physics verifies" principle exactly, restated here because GVPE's Vector Space
plays the identical architectural role that project's retrieval layer did.

## 22.4 Non-hot-path enforcement (GVPE-VEC-001)

`gvpe-vector`'s public API takes `&SimulationStateSnapshot` (an owned/borrowed copy taken *after* a
frame completes, `05_runtime_design.md` §5.5), never a live reference into `BodyStateSoA`
(`17_detailed_design.md` §4.1) — this is a type-level guarantee that `gvpe-vector` cannot be called
mid-step even by accident, since it has no type that grants access to in-progress simulation state.

Requirements satisfied: GVPE-VEC-001/002, `11号文書` §11.1–§11.5 (all now have concrete
implementations behind their previously interface-only traits).
