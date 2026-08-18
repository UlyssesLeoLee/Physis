# GVPE — Parallel Design（並列設計）

Input baseline: `04_architecture.md` §4.1 (`gvpe-island`, `gvpe-scheduler`), `07_solver_design.md`.

## 9.1 Physics Islands (Runtime Constraint Graph → parallel unit)

```
Runtime Constraint Graph → Connected Components → Physics Islands
```
Each island is solved independently — no cross-island data dependency within a frame. Islands are
the unit of: sleeping (a fully-sleeping island is skipped entirely), parallelism (islands run on
separate scheduler jobs), work partition, and load balancing.

## 9.2 Job DAG (Execution Graph, `03_graph_schema.md` §1.C, made concrete)

```
BroadPhase → NarrowPhase[] → IslandBuild → SolveIsland[] → Integrate[]
```
`NarrowPhase[]`, `SolveIsland[]`, `Integrate[]` are per-island/per-pair fan-out points — the `[]`
marks where the scheduler distributes work across threads.

## 9.3 Scheduler mechanism (candidates, MVP: simple work-stealing pool)

Research directions: work stealing, thread pool with dependency counters (job B runs only after
job A's counter hits zero), per-thread scratch allocators (ties to `08_memory_design.md` §8.2 —
each thread's `FrameScratch` is independent, no cross-thread arena sharing).

**Explicit goal**: avoid a global mutex on any hot-path structure. Island-level parallelism is
chosen specifically because islands are provably independent (no shared constraint rows across
islands within a frame) — this avoids needing fine-grained locking altogether, rather than trying
to make fine-grained locking fast.

## 9.4 What MVP actually implements

"Basic Multithreading" per `01_requirements.md` §11 MVP scope — parallel `SolveIsland[]` across a
simple thread pool, no work-stealing sophistication required for MVP acceptance. Work-stealing and
per-thread scratch refinement are listed here as the target shape, not an MVP gate.
