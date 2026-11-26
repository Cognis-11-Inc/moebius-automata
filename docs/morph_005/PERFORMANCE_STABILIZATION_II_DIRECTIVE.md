# MŒBIUS AUTOMATA
## MORPH-005 — Performance Stabilization II
### Allocation, Locality and Collision Scalability

**Directive:** Proceed autonomously with investigation, implementation, validation, benchmarking, documentation, evidence archival, commit and push.

Biological expansion remains suspended until the simulation substrate achieves acceptable performance. The objective is not merely to improve the current benchmark but to establish an architecture capable of supporting substantially greater biological and environmental complexity.

## 1. Establish the verified baseline

Begin from the latest pushed revision, reported as `0a63909`.

Read and reconcile:

- `AGENTS.md`
- `HANDOFF.md`
- The relevant Atlas entries.
- `docs/morph_005/PERFORMANCE_STABILIZATION.md`
- Existing benchmark evidence, rejected experiments and archive receipts.
- Relevant implementation and Git history.

Verify the relationship between `cdc3acd`, on which 228 tests reportedly passed, and `0a63909`.

Previously reported performance:

| Benchmark | Mean tick | Worst tick |
|---|---:|---:|
| Matched 420 ticks | 33.9 ms | 408 ms |
| Congested save | 183.7 ms | 410 ms |
| 16 founders | 105.1 ms | 759 ms |
| Native 720 ticks | 52.9 ms | 635 ms |
| Longer execution | 63.2 ms | 596 ms |

Additional reported failures:

- Eight-founder execution produced a 1.04-second tick.
- The longer execution completed only 2,818 of 3,600 requested ticks within its measurement budget.
- Congested allocation instrumentation reported 87.7 MB of cumulative allocation requests per tick.

Treat these as reported historical measurements until reconciled with their underlying evidence.

## 2. Mandatory benchmark integrity

Permanently harden the benchmark infrastructure before undertaking new optimization comparisons.

Require:

- Isolated baseline and candidate build directories.
- Verified executable SHA-256 hashes.
- Embedded source and build identities.
- Identical compiler settings, dependencies and instrumentation.
- Identical initial states, seeds, simulation policies and requested tick counts.
- Automatic rejection of stale, mismatched or unidentified executables.
- Immediate termination following compilation failure.
- Explicit identification of incomplete and timed-out runs.
- Separation of allocation-instrumented runs from ordinary performance measurements.

Add regression coverage for the benchmark integrity mechanisms where practical.

No invalid comparison may contribute to an optimization claim.

## 3. P0 — Attribute computational amplification

Before implementing another optimization, establish precisely where the execution time and allocation activity originate.

Instrument representative ordinary ticks and pathological ticks.

Measure:

- Execution time by authoritative subsystem.
- Allocation count and cumulative requested bytes by call path.
- Reallocation and deallocation activity.
- Peak live memory and process memory.
- Collision candidate counts.
- Narrow-phase query counts.
- Solver iterations.
- Recovery attempts.
- Admission and placement attempts.
- Geometry construction and reconstruction.
- Spatial-index construction and update costs.
- State cloning and temporary collection construction.
- Event-history processing.
- Cache hits, misses and invalidations.

Produce a ranked attribution of the most expensive execution paths.

Investigate the previously observed hundreds-of-milliseconds and one-second spikes individually.

Do not infer causality from aggregate timings alone.

## 4. P0 — Eliminate allocation amplification

Investigate the reported 87.7 MB of cumulative allocation requests per congested tick.

Identify the exact functions, call sites, allocation lifetimes and consumers responsible.

Where justified by evidence, implement:

- Reusable working buffers.
- Persistent scratch storage.
- Collection capacity reuse.
- Borrowed access instead of unnecessary cloning.
- Incremental state updates.
- Elimination of redundant temporary representations.
- Reuse of collision query storage.
- Transactional event accumulation without historical copying.

Ensure reused storage cannot introduce stale state, cross-organism contamination or nondeterministic execution.

Measure each accepted change independently.

Do not confuse cumulative allocated bytes with retained or resident memory.

## 5. P0 — Recovery and admission efficiency

Investigate the recovery, admission, birth-placement and feeding-placement paths.

Determine whether pathological execution originates from:

- Repeated unsuccessful searches.
- Redundant collision queries.
- Repeated geometry preparation.
- Repeated examination of identical candidates.
- Solver iteration amplification.
- Repeated evaluation of unchanged conditions.

Eliminate provably redundant work.

Preserve existing authoritative outcomes, candidate ordering, tie-breaking, recovery semantics and admission rules.

Do not silently impose arbitrary search limits, suppress difficult cases or defer authoritative work merely to improve benchmark results.

Any proposed change to simulation semantics must remain a separate, explicitly documented experiment and must not enter the accepted implementation without authorization.

## 6. P1 — Persistent spatial locality

Audit the existing spatial-indexing and broad-phase mechanisms before selecting an implementation.

Determine whether collision and placement queries scale with the relevant local environment or unnecessarily depend on global organism and structural populations.

Investigate suitable approaches, including:

- Spatial hashing.
- Sparse uniform grids.
- Hierarchical spatial partitioning.
- Persistent or incrementally maintained BVHs.
- Separate indexing strategies for mobile organisms and persistent secretion structures.

Select the approach supported by measurements and existing geometry characteristics, not architectural preference alone.

Requirements:

- Conservative candidate selection.
- Coverage of swept movement bounds.
- Correct handling of elongated, articulated and hollow geometry.
- Correct handling of cell and partition boundaries.
- Incremental insertion, removal and movement updates where applicable.
- Deterministic candidate ordering.
- No missed authoritative collision candidates.
- No dependency on camera position or Witness radius.

The Witness is an observation and presentation mechanism. It must not determine whether a physically relevant collision exists.

Test the indexing mechanism against exhaustive reference searches, including adversarial spatial arrangements.

## 7. P1 — Persistent geometry and cache ownership

Audit the lifetime and ownership of:

- Anatomy collision geometry.
- Secretion structures.
- Pipe segments.
- Spatial indices.
- Acceleration structures.
- Placement-query caches.
- Collision-query caches.

Ensure unchanged geometry is not unnecessarily reconstructed.

Prefer incremental updates where the underlying geometry permits them.

Measure invalidation costs and identify any remaining cache bypasses.

Do not introduce unbounded cache growth as a substitute for computation.

## 8. P2 — Parallelism feasibility

Investigate parallel execution only after the dominant serial inefficiencies have been characterized.

Identify independent operations that can be executed concurrently without changing authoritative outcomes.

Potential candidates include read-only geometry queries, independent candidate preparation and other demonstrably independent computations.

Preserve deterministic ordering and commit semantics.

Do not introduce GPU compute, a replacement physics engine or additional heavyweight dependencies without evidence that the existing architecture requires them.

Document rejected architectural alternatives and their reasons.

## 9. Mandatory correctness gates

Every accepted optimization must preserve:

- Authoritative simulation states.
- Contact logs.
- Collision semantics.
- Material conservation.
- Feeding outcomes.
- Birth and admission outcomes.
- Secretion continuity.
- Save/reload behaviour.
- Deterministic replay.
- Headless and rendered Witness equivalence.

Require exact equality for existing matched deterministic benchmarks.

Add focused regression tests for newly introduced mechanisms.

Reject faster candidates that fail equivalence.

## 10. Benchmark matrix

Execute matched baseline and candidate comparisons covering:

- One founder.
- Two interacting founders.
- Four founders.
- Eight founders.
- Sixteen founders.
- Thirty-two founders, if computationally feasible.
- Existing matched 420-tick history.
- Existing congested save.
- Native 720-tick execution.
- Long-duration 3,600-tick execution.
- Sparse and dense structural environments.
- Increasing structural complexity at fixed population.
- Increasing population at approximately fixed structural complexity.
- Increasing history length without equivalent increases in active simulation complexity.

Separate population scaling from structural scaling and historical accumulation.

For each benchmark, report:

- Requested and completed ticks.
- Mean, median, P95, P99 and maximum tick duration.
- Tick throughput.
- Real-time factor relative to the configured simulation frequency.
- Allocation requests per tick.
- Peak live and resident memory.
- Candidate counts and solver iterations.
- Geometry rebuild counts.
- Population and structural-node counts.
- Replay and equivalence verdicts.

Where practical, repeat performance measurements to characterize run-to-run variability.

Native rendering benchmarks must report frame-time distributions separately from authoritative simulation tick distributions.

## 11. Provisional engineering targets

These are performance objectives, not permission to alter simulation fidelity.

| Metric | Target |
|---|---:|
| Current small-colony mean tick | ≤ 5 ms |
| Preferred small-colony mean tick | ≤ 2 ms |
| Current small-colony P99 tick | ≤ 8 ms |
| Unchanged geometry rebuilds | Zero |
| Full historical event copies per tick | Zero |
| Deterministic correctness regressions | Zero |
| Native rendering | Stable 60 FPS or better |
| Congested cumulative allocation requests | At least 95% reduction |
| Existing 3,600-tick benchmark | Complete within 180 seconds |

The long-duration completion criterion is an interim regression gate, not proof of acceptable real-time performance.

Verify the intended authoritative simulation frequency and derive its actual real-time tick budget.

Establish additional population and structural scaling budgets from measured evidence.

Do not declare scalability solved merely because a small population meets its target.

## 12. Delivery and evidence

Produce:

- Updated performance stabilization report.
- Ranked bottleneck attribution.
- Before/after allocation profiles.
- Worst-tick causal breakdowns.
- Population and structural scaling curves.
- Native and headless performance comparisons.
- Exact replay and correctness evidence.
- Accepted and rejected optimization inventory.
- Remaining limitations.
- Updated Atlas and `HANDOFF.md`.
- Verified evidence archive on Mermogon, following repository requirements.

Run focused and full regression suites, strict Clippy, formatting checks and applicable Atlas validation.

Commit and push validated improvements.

Record the exact tested revision and final pushed revision. Reconcile any differences.

Partial improvements may be committed and pushed, provided all unresolved performance failures remain explicitly visible.

## 13. Final directive

MŒBIUS AUTOMATA will eventually contain substantially more biological, chemical, neurological and structural complexity than its present implementation.

The current organisms must become computationally inexpensive enough to leave considerable processing headroom for future systems.

Do not optimize away the simulation we intend to create.

Do not reduce biological fidelity to compensate for architectural inefficiency.

Do not fake scalability through rendering tricks, hidden simulation suspension, suppressed collisions or weakened authoritative behaviour.

**Make inactivity cheap. Make locality fundamental. Make persistent structures inexpensive. Make computational cost proportional to relevant activity wherever possible.**

Proceed until the stabilization objectives are achieved or a demonstrable architectural limitation requires a new design decision.

Keep biological expansion suspended while the performance gate remains unresolved.
