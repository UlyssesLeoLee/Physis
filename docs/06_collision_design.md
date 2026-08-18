# GVPE — Collision Design（衝突検出詳細設計）

Input baseline: `04_architecture.md` (`gvpe-collision`), `01_requirements.md` GVPE-FR-002.
Self-developed per `00_vision.md` §0.1/§0.2 — no third-party collision library vendored as the core.

## 6.1 Broad phase (candidates, MVP picks one — decision deferred to implementation spike)

| Algorithm | Fit | MVP choice rationale |
|---|---|---|
| Sweep and Prune (SAP) | good for mostly-coherent motion frame-to-frame | simplest to self-implement correctly first |
| Dynamic AABB Tree | good general-purpose, moderate complexity | strong second candidate |
| BVH | similar to AABB tree, more complex build | deferred |
| Spatial Hash | good for uniform density scenes | deferred, revisit if MVP scenes are grid-like |

MVP implements SAP first (lowest implementation risk for a from-scratch solver), with the
interface shaped so Dynamic AABB Tree can replace it without touching narrow phase.

## 6.2 Narrow phase — shapes (MVP subset bolded)

**Sphere**, **Box**, **Plane**, Capsule, Convex Hull, Triangle Mesh, Heightfield, Compound.

## 6.3 Narrow phase — algorithms

- **SAT** (Separating Axis Theorem): primary for Box-Box, Box-Plane, Sphere-Box in MVP.
- **GJK**: reserved for convex hull support (post-MVP).
- **EPA**: reserved, pairs with GJK for penetration depth once convex hulls land.

## 6.4 Contact manifold

Output of narrow phase feeds `gvpe-constraint`'s `ContactConstraint` rows (`07_solver_design.md`
§2) — never a graph `Constraint` node (`02_physics_ontology.md` §9's binding rule).

```rust
struct ContactManifold {
    body_a: BodyHandle,
    body_b: BodyHandle,
    points: SmallVec<[ContactPoint; 4]>,
}
struct ContactPoint { position: Vec3, normal: Vec3, penetration: f32 }
```

## 6.5 Interface stability for future broad-phase swap

`gvpe-collision` exposes only `fn broad_phase(bodies: &[Aabb]) -> Vec<(BodyIndex, BodyIndex)>` to
the rest of the engine — algorithm choice is fully internal, so §6.1's "MVP picks SAP" decision is
reversible without touching `gvpe-constraint`/`gvpe-solver`.
