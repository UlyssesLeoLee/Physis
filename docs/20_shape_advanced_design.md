# GVPE — Advanced Shapes & GJK/EPA Detailed Design

Input baseline: `06_collision_design.md` §6.2 (MVP subset: Sphere/Box/Plane, rest reserved),
`17_detailed_design.md` §3. A physics engine claiming completeness needs the full shape set a game
or simulation actually uses — Capsule, Convex Hull, Triangle Mesh, Heightfield, Compound — not just
the three MVP primitives.

## 20.1 Shape descriptor extension

```rust
enum ShapeDesc {
    Sphere { radius: f32 },
    Box3 { half_extents: [f32; 3] },
    Plane { normal: [f32; 3], offset: f32 },
    Capsule { radius: f32, half_height: f32 },                       // NEW
    ConvexHull { points: Arc<[[f32; 3]]> },                           // NEW, shared/refcounted (asset-like)
    TriangleMesh { vertices: Arc<[[f32; 3]]>, indices: Arc<[u32]>,    // NEW, static bodies only
                    bvh: Arc<MeshBvh> },
    Heightfield { heights: Arc<[f32]>, width: u32, depth: u32,        // NEW, static bodies only
                   cell_size: f32 },
    Compound { children: Vec<(Transform, Box<ShapeDesc>)> },          // NEW, recursive
}
```
`Arc` on the heavier variants avoids per-body duplication of shared assets (multiple bodies using
the same mesh) — this is an allocation-policy decision consistent with `08_memory_design.md` §8.1's
hot-path discipline: the `Arc` clone itself is not on the hot path (shapes don't change per-frame),
only the collision *queries against* them are.

## 20.2 GJK (Gilbert-Johnson-Keerthi) — convex-convex distance/overlap

```rust
fn gjk_intersect(a: &SupportFn, b: &SupportFn) -> GjkResult {
    let mut simplex = Simplex::new();
    let mut direction = initial_direction(a, b);
    loop {
        let point = minkowski_support(a, b, direction);
        if dot(point, direction) < 0.0 { return GjkResult::NoOverlap; }
        simplex.push(point);
        match simplex.do_simplex(&mut direction) {
            SimplexResult::ContainsOrigin => return GjkResult::Overlap(simplex),
            SimplexResult::Continue => continue,
        }
        if simplex.iterations() > GJK_MAX_ITERATIONS { return GjkResult::NoOverlap; }  // conservative bail-out
    }
}

trait SupportFn { fn support(&self, direction: [f32; 3]) -> [f32; 3]; }
```
Every convex `ShapeDesc` variant (Sphere/Box/Capsule/ConvexHull) implements `SupportFn` — GJK itself
is shape-agnostic, which is why adding a new convex primitive later never touches this function,
only adds one more `SupportFn` implementation (same "algorithm doesn't special-case shape kinds"
discipline `18_joints_ccd_design.md` §18.1 already used for joint rows vs. the solver).

## 20.3 EPA (Expanding Polytope Algorithm) — penetration depth, given GJK's overlap simplex

```rust
fn epa_penetration(simplex: Simplex, a: &SupportFn, b: &SupportFn) -> ContactManifold {
    let mut polytope = Polytope::from_simplex(simplex);
    loop {
        let (closest_face, distance) = polytope.closest_face_to_origin();
        let support_point = minkowski_support(a, b, closest_face.normal);
        let expansion = dot(support_point, closest_face.normal) - distance;
        if expansion < EPA_TOLERANCE { return build_manifold_from_face(closest_face, distance); }
        polytope.expand(support_point);   // re-triangulate, removing faces the new point sees
    }
}
```
GJK+EPA together replace SAT (`17号文書` §3.3) specifically for shape pairs SAT cannot handle
(anything involving Capsule or ConvexHull) — SAT remains the Box-Box/Box-Plane/Sphere-Box path
unchanged (it's cheaper for those cases), per the dispatch table in §20.4.

## 20.4 Narrow-phase dispatch (extends `17号文書` §3.3, does not replace it)

```rust
fn narrow_phase(a: &ShapeDesc, xf_a: &Transform, b: &ShapeDesc, xf_b: &Transform) -> Option<ContactManifold> {
    match (a, b) {
        (Box3{..}|Sphere{..}|Plane{..}, Box3{..}|Sphere{..}|Plane{..}) => narrow_phase_sat(a, xf_a, b, xf_b),
        (Sphere{..}, Sphere{..}) => sphere_sphere_analytic(a, xf_a, b, xf_b),   // cheapest exact case, no SAT needed
        (TriangleMesh{..}, _) | (_, TriangleMesh{..}) => mesh_vs_convex(a, xf_a, b, xf_b),   // BVH-accelerated, per-triangle GJK/EPA
        (Heightfield{..}, _) | (_, Heightfield{..}) => heightfield_vs_convex(a, xf_a, b, xf_b),   // grid-cell lookup, per-cell GJK/EPA
        (Compound{children: ca, ..}, _) => compound_vs_other(ca, xf_a, b, xf_b),   // recurse per child, union manifolds
        (_, Compound{..}) => narrow_phase(b, xf_b, a, xf_a).map(flip_manifold),
        _ => gjk_epa_convex_pair(a, xf_a, b, xf_b),   // Capsule/ConvexHull combinations
    }
}
```
SAT stays the fast path for the MVP shape triple (it's cheaper and already implemented,
`17号文書` §3.3) — GJK/EPA is added, not substituted, exactly as `06_collision_design.md` §6.3
already promised ("GJK/EPA: reserved for convex hull support (post-MVP)").

## 20.5 Broad phase implication

`Arc<[[f32;3]]>`-backed shapes (mesh/heightfield) are almost always **static** bodies
(`06号文書` doesn't require this, but it's the overwhelmingly common case) — the broad phase
(`17号文書` §3.2 SAP) treats static AABBs as never re-sorted, which is already how SAP's insertion
sort behaves for zero-velocity entries (no design change needed, noted here only so the interaction
is explicit rather than accidental).

## 20.6 Test fixtures owed to `15_testing_strategy.md`

- GJK/EPA agreement with SAT on a Box-Box case where both algorithms apply — contact normal and
  penetration depth must match within tolerance (cross-validates the two paths against each other).
- A capsule resting stably on a heightfield (no jitter, no tunneling) over N steps.
- A compound shape (e.g. two boxes forming an L) colliding correctly against a single box —
  verifies per-child manifold union in §20.4's `compound_vs_other`.

Requirements satisfied: `01_requirements.md` GVPE-FR-002 (full shape set), `06_collision_design.md`
§6.2/§6.3 (both "reserved" lines now have a design), `00_vision.md` §0.5.
