# GVPE — Dependency & License Review（依存関係・ライセンス審査）

Input baseline: `01_requirements.md` GVPE-LIC-001, `03_graph_schema.md` §1.A (backing store choice
still open).

## 16.1 Scope

Applies to any candidate **Graph Database** or **Vector Database** considered for `gvpe-graph` /
`gvpe-vector`'s backing store. Does not apply to `gvpe-core`/`gvpe-solver`/etc.'s ordinary Rust
crate dependencies (math, SIMD, threading libraries), which follow normal OSS license hygiene but
not this heavyweight review — the review exists specifically because embedding a DB engine has
commercial-redistribution stakes that a math crate doesn't.

## 16.2 Review matrix (every candidate must clear every row before selection)

| Check | Question | Pass criterion |
|---|---|---|
| License | What license? | OSI-approved, permissive or weak-copyleft compatible with commercial redistribution |
| Commercial Use | Explicitly allowed? | Yes, unambiguously, in the license text itself |
| OEM | Can GVPE be embedded and resold inside a larger product? | Yes |
| Redistribution | Can the compiled artifact be redistributed? | Yes, without per-copy fee or registration requirement |
| Modification | Can the source be modified and the modified version shipped? | Yes |
| Static Linking | Permitted? | Yes, without triggering copyleft obligations on GVPE's own code |
| Dynamic Linking | Permitted? | Yes |
| SaaS | If offered as a hosted service, any AGPL-style network-use clause triggered? | No such clause, or clause is acceptable to the project |
| Embedded Use | Explicitly supports embedded/library use (not just standalone server deployment)? | Yes |

A candidate failing **any** row is rejected outright — this is a hard gate, not a scored trade-off
(mirrors the same non-negotiable-gate pattern `01_requirements.md`'s GVPE-PROHIBIT list uses for
architectural constraints; license risk gets the same treatment as architectural risk here).

## 16.3 Candidate tracking (populate as the spike proceeds — none are pre-selected)

| Candidate | Kind | License | Status |
|---|---|---|---|
| (embedded graph store, hand-rolled) | Graph | N/A (project's own code) | Always passes §16.2 trivially; fallback if no external candidate clears review |
| (embedded vector index, hand-rolled or `usearch`-class library) | Vector | TBD | Not yet reviewed |
| (any full Graph DB engine, e.g. an embeddable graph library) | Graph | TBD | Not yet reviewed |

No row above is a commitment — this table exists to make the review's *output* auditable once a
real spike happens, not to pre-announce a decision.

## 16.4 Fallback position

If no external candidate clears §16.2 for either Graph or Vector storage, `gvpe-graph`/`gvpe-vector`
default to a hand-rolled embedded store (own license, trivially compliant). This is an acceptable
outcome, not a failure state — `03_graph_schema.md`'s query patterns (§6 there) were deliberately
kept simple (bounded traversal, no unbounded multi-hop) specifically so a hand-rolled store remains
viable if no third-party candidate survives review.

## 16.5 Re-review trigger

Any change to a selected dependency's license (new major version, relicensing) triggers immediate
re-review against §16.2, not a "grandfather it in" exception.
