# MŒBIUS AUTOMATA
## MORPH-005 — Performance Stabilization V
### Deterministic Parallelism, Physical Geometry Authority and Computational Economy

## 0. Mission

Continue from the published Stabilization IV checkpoint:

`4e3eea6bae6b344351d4fbf5527e1b9b4435b087`

MORPH-005 remains PARTIAL.

Stabilization IV established substantial improvements in collision traversal, spatial locality, geometry invalidation, cold-start construction and historical continuity.

However, the remaining computational cost is incompatible with the intended population scale and biological richness of MŒBIUS AUTOMATA.

This phase must investigate deeper architectural opportunities rather than indefinitely accumulating narrow sequential optimizations.

**MORPH-006 and biological expansion remain suspended.**

## 1. Mandatory Entry Verification

Read and reconcile:

- AGENTS.md
- HANDOFF.md
- PERFORMANCE_STABILIZATION_IV.md
- PERFORMANCE_STABILIZATION_IV_RESULTS.json
- PERFORMANCE_STABILIZATION_IV_VALIDATION.json
- HISTORICAL_CONTINUITY_PROPOSAL.md
- SPATIAL_LOCALITY_AUDIT.md
- QUERY_MULTIPLICITY.md
- SENSORY_ARCHITECTURE.md

Locate these documents through the actual repository structure.

Verify:

- Current branch, HEAD and working-tree status.
- Published checkpoint and runtime source identity.
- Clean runtime validation at `ecdc8625b5d605997a6e3a559b36903f78231d33`.
- Mermogon archive integrity and accessibility.
- Existing isolated collision fixtures.
- Benchmark executable provenance.
- History v1 and opt-in v2 authority boundaries.

Preserve previous evidence, rejected candidates, historical snapshots and sealed archives.

Do not restart completed archaeology without a specific reason.

## 2. Established Performance Baseline

The published IV report records:

| Scenario | Mean ms | P99 ms |
|---|---:|---:|
| One founder | 0.484 | 4.020 |
| Two founders | 5.342 | 30.156 |
| Four founders | 7.339 | 19.389 |
| Eight founders | 13.516 | 44.421 |
| Sixteen founders | 38.459 | 80.706 |
| Fresh colony | 11.083 | 33.823 |
| Eight-founder resume | 13.635 | 44.968 |
| Congested save | 43.190 | 98.328 |
| Native 720 | 21.520 | 70.085 |
| Endurance v2 | 53.730 | 108.044 |

Endurance accumulated timings:

- Continuous linear casting: 59.922 seconds.
- Contact recovery: 22.566 seconds.
- Wet remeshing: 14.658 seconds.
- Horizon maintenance: 0.284 seconds.

Timers may overlap. Do not add them as independent costs.

Congested allocation requests fell from 263,041,278 to 23,606,126 bytes over matched ticks 722–724.

The reported 91.026% reduction remains below the 95% target.

These are inherited published measurements. Reconcile them against the actual retained evidence before using them as experimental baselines.

## 3. P0-A — Deterministic Parallel Execution

### Objective

Determine how effectively the current authoritative runtime uses the Forge's available CPU resources.

The Forge has a Ryzen 9 7950X processor with 16 physical cores and 32 hardware threads. Verify actual hardware topology, runtime environment and resource availability before benchmarking.

Do not assume that adding threads will produce meaningful acceleration.

### Required dependency archaeology

Construct an execution dependency map of the authoritative tick.

Identify exact producer, consumer and mutation relationships for:

- Biological movement proposals.
- Geometry construction.
- Spatial candidate generation.
- Body-body collision queries.
- Body-pipe collision queries.
- Continuous shape casting.
- Contact recovery.
- Collision admission.
- Material exchange.
- Structural deposition.
- History production.
- Snapshot generation.

Classify operations as:

1. Independently executable.
2. Read-only over an immutable authoritative snapshot.
3. Parallelizable with deterministic reduction.
4. Sequential due to causal dependencies.
5. Currently unresolved.

Establish actual CPU occupancy, thread utilization, synchronization costs and serial bottlenecks.

### Candidate architecture

Investigate whether expensive geometric queries can be evaluated concurrently against immutable geometry and authoritative input snapshots.

Where lawful:

- Construct ordered query descriptors.
- Evaluate independent queries in parallel.
- Collect results without granting workers mutation authority.
- Restore canonical result ordering.
- Admit results through the existing authoritative mutation boundary.

Preserve the current numerical operation order wherever it affects exact results.

Do not introduce nondeterministic floating-point reductions.

Do not parallelize causally dependent contact resolution merely because its operations appear similar.

A result produced against an obsolete pose or geometry revision must never be admitted.

### Implementation discipline

Inspect existing concurrency mechanisms and dependencies before introducing a worker pool.

Avoid:

- Creating threads per tick.
- Nested uncontrolled parallelism.
- Unbounded queues.
- Excessive synchronization.
- Global mutable collision caches.
- Lock contention that merely replaces computational cost.
- Oversubscription of rendering and simulation threads.

### Scaling assay

Compare controlled worker configurations, where supported:

- 1 worker.
- 2 workers.
- 4 workers.
- 8 workers.
- 16 workers.
- 32 workers.

Measure:

- Mean and tail latency.
- Worker utilization.
- Synchronization overhead.
- Allocation cost.
- Memory growth.
- Exact replay.
- Contact equivalence.
- Native rendering responsiveness.

Use the same authenticated input states.

Require exact results across worker counts and repeated executions.

If meaningful parallelization is not possible without changing authoritative semantics, report the actual causal limitation.

Do not manufacture concurrency by weakening determinism.

## 4. P0-B — Physical Geometry Authority

### Objective

Investigate whether all currently generated morphological detail genuinely requires authoritative physical collision representation.

The existing pipe geometry reportedly includes:

- Cubic interpolation.
- Elliptical sections.
- Ridges.
- Nozzle wobble.
- Age-dependent radii.
- Hollow interiors.
- End caps and seals.
- Structural junctions.

Verify the exact implementation and consumers of these properties.

The purpose is to distinguish physical necessities from potentially expressive geometric details.

### Required classification

For each morphological property, identify whether it affects:

- Material conservation.
- Physical occupancy.
- Inner or outer surface contact.
- Traversable interior clearance.
- Adhesion.
- Structural connectivity.
- Mechanical forces.
- Future biological perception.
- Presentation only.

Do not classify a property as presentation-only merely because a physical consumer has not yet been found.

Record uncertainty explicitly.

### Representation candidates

Investigate:

- Canonical physical geometry with independent presentation tessellation.
- Procedural hollow-section collision representations.
- Hierarchical curved segment representations.
- Locally adaptive collision tessellation.
- Conservative analytical bounds.
- Persistent segment-level acceleration.
- Exact local mesh generation on demand.

These are design candidates, not mandates.

### Two distinct experimental lanes

**Lane A — Exact acceleration**

Preserve existing authoritative geometry and contact semantics.

Candidates in this lane must satisfy the existing exact-equivalence contract.

**Lane B — Alternative physical representation**

Investigate whether another representation provides acceptable physical fidelity at substantially lower computational cost.

Lane B is authorized for isolated research prototypes only.

It is not authorized to replace the existing collision authority.

For each alternative, report:

- Which physical properties change.
- Which properties remain invariant.
- Maximum and distribution of geometric deviations.
- Contact-point and contact-normal deviations.
- Time-of-impact deviations.
- Interior clearance preservation.
- Material-ledger compatibility.
- Effects on adhesion and structural interaction.
- Performance and scaling implications.

Do not claim exact equivalence when the physical model has changed.

Do not silently modify legacy snapshots or replay expectations.

A proposed replacement of authoritative geometry requires explicit operator approval, a separate versioned contract and migration plan.

## 5. P1-A — Contact Recovery

The endurance report identifies contact recovery as a substantial remaining cost.

Investigate the complete recovery path.

Determine:

- Why recovery is invoked.
- Which geometry and state it consumes.
- Number of recovery operations per body and tick.
- Number of distinct recovery poses.
- Frequency of repeated recovery against the same structure.
- Whether previous contacts remain useful.
- Whether recovery repeatedly evaluates geometry that cannot influence the outcome.

Distinguish necessary physical correction from implementation-dependent work.

Investigate conservative reuse of contact witnesses, closest features and local geometric information.

Preserve original recovery semantics and tolerances.

Do not cap iterations, skip difficult contacts or change physical admission merely to reduce timing.

## 6. P1-B — Incremental Wet Geometry Maintenance

Wet pipe remeshing remains a measurable cost.

Investigate the actual relationship between:

- Paid filament nodes.
- Deposit age.
- Radius evolution.
- Section interpolation.
- Geometry revision.
- Mesh generation.
- Acceleration-structure construction.
- Presentation updates.
- Collision-cache invalidation.

Determine the smallest region whose geometry must change following each authoritative mutation.

Evaluate:

- Local reconstruction.
- Immutable segment reuse.
- Chunk-level revision ownership.
- Incremental interpolation.
- Reusable topology.
- Partial acceleration-structure rebuilding.
- Independent presentation updates.

The current collision-library API may not support safe vertex refitting. Verify available capabilities before assuming that it does.

Do not introduce unsafe internal mutation or approximate colliders through an undocumented dependency modification.

Preserve openings, junctions, hollow interiors and material conservation.

Demonstrate that unchanged geometry remains reusable while every relevant mutation invalidates affected acceleration structures.

## 7. P2-A — Remaining Allocation Economy

The original 95% allocation-reduction gate remains unmet.

The IV candidate retains approximately 8.974% of the baseline requested allocation volume.

Investigate the remaining allocation sources using the authenticated congested fixture.

Attribute requested bytes and allocation counts to:

- Collision workspaces.
- Shape casting.
- Contact recovery.
- Geometry construction.
- Temporary collections.
- History processing.
- Snapshot operations.
- Diagnostic infrastructure.
- Other measured consumers.

Distinguish cumulative allocation requests from retained memory and peak RSS.

Investigate:

- Bounded workspace reuse.
- Persistent scratch storage.
- Capacity reservation.
- Borrowed immutable geometry.
- Elimination of unnecessary cloning.
- Deterministic worker-local workspaces.

Do not obtain allocation reductions through unbounded retention.

The existing process-wide presentation cache reportedly lacks eviction. Audit its ownership and determine whether it creates meaningful retention under changing anatomy definitions.

Require measured memory stability for the tested workloads.

Do not claim whole-engine lifetime stability from a short allocation probe.

## 8. P2-B — Historical Continuity Compatibility

Preserve history v1 unchanged.

Preserve the opt-in v2 prototype and its demonstrated 3,600-tick continuity.

The IV report identifies the following limitations:

- Source-bound checkpoint fingerprints.
- No automatic v2-to-v2 source migration.
- Explicit provenance ceilings.
- Unproven unrestricted lifetime continuity.
- Unproven physical power-loss durability.
- Unproven whole-engine memory stability.

Develop a scoped compatibility and migration proposal addressing future engine revisions.

Distinguish:

- Serialization compatibility.
- Physical-law compatibility.
- Historical-event compatibility.
- Replay-executable identity.
- Snapshot migration identity.
- Explicit provenance transitions.

Investigate whether compatible source changes can be admitted through a verified migration procedure without weakening fail-closed restoration.

Do not permit arbitrary source-fingerprint bypass.

Do not rewrite immutable historical segments.

No migration becomes canonical merely because its prototype passes.

History compatibility must not distract from the primary collision-performance investigation.

## 9. Experimental Qualification Sequence

Retain the escalating evidence discipline established in IV.

For each candidate:

### Gate A — Attribution

Identify the exact source of computational cost.

### Gate B — Isolated improvement

Demonstrate causal improvement using authenticated fixtures.

### Gate C — Correctness

Pass applicable geometry, contact, conservation, mutation-invalidation and replay tests.

### Gate D — Matched continuation

Reproduce the original pathological colony cases and compare every authoritative intermediate state required by the established contract.

### Gate E — Ordinary release qualification

Measure uninstrumented performance with verified executable identity.

### Gate F — Full matrix

Run the complete benchmark matrix only after focused evidence establishes meaningful benefit.

Rejected candidates must remain documented and recoverable.

Do not repeatedly execute the entire matrix after inconclusive micro-optimizations.

## 10. Spatial Locality Preservation

Preserve the established spatial horizon and corrected mutation-token invalidation.

No coordinate-system redesign is authorized by the current evidence.

The runtime reportedly has no configurable world extent or dense empty-volume allocation.

Do not repeat translation experiments and label them actual habitat enlargement.

Future spatial work must be motivated by measured shortcomings in occupied-space processing, not the theoretical size of empty space.

## 11. Population and Interaction Scaling

Retain the current founder and agent limits until their physical and resource contracts have been audited.

Investigate scaling through controlled factors:

- Fixed population with increasing local structural density.
- Fixed geometry with increasing interacting bodies.
- Physically separated bodies.
- Shared congested structures.
- Different movement conditions.
- Increasing geometry revisions.
- Different worker counts.

Determine whether observed cost is primarily governed by:

- Population.
- Local interaction multiplicity.
- Geometry complexity.
- Sequential admission dependencies.
- Structural growth.
- Contact recovery.
- Another measured variable.

Do not derive asymptotic complexity from unrelated colony trajectories.

## 12. Future Biological Compatibility

Preserve the sensory architecture contract.

Do not implement sensory organs or neural systems during Stabilization V.

The spatial and execution infrastructure should remain compatible with future bounded sensory consumers.

Perception and physical authority must remain separate.

No organism should require an omniscient environmental query merely to exist.

Do not reduce future biological richness to accommodate inefficient engine architecture.

## 13. Final Acceptance Requirements

Retain all established MORPH-005 correctness and performance gates.

Targets:

- Authoritative tick: approximately 16.667 ms.
- Provisional mean: ≤ 5 ms.
- Preferred mean: ≤ 2 ms.
- P99: ≤ 8 ms.
- Congested requested allocation reduction: ≥ 95%.
- Complete 3,600-tick endurance within the established 180-second gate.
- Exact applicable replay and state equivalence.
- Material conservation.
- No missed admissible contact.
- Reliable geometry invalidation.
- Bounded memory under tested workloads.

Complete the full existing benchmark matrix for qualified candidates.

Report cold-start and steady-state performance independently.

Report simulation timing separately from rendering timing.

If an alternative physical representation is studied, report it in a separate experimental matrix. Do not mix its results into exact-acceleration certification.

## 14. Deliverables

Produce:

- PERFORMANCE_STABILIZATION_V.md.
- Authoritative tick dependency map.
- Deterministic parallelization feasibility report.
- Worker-scaling evidence.
- Physical geometry authority audit.
- Alternative geometry research report, if applicable.
- Contact-recovery attribution.
- Incremental remeshing assessment.
- Remaining allocation attribution.
- History compatibility proposal.
- Accepted and rejected optimization evidence.
- Final matched benchmark matrix, when qualified.
- Updated HANDOFF and Atlas.
- Full clean-source validation receipts.
- Restore-verified Mermogon evidence archive.

All results must identify exact sources, executable versions, configurations and relevant test evidence.

Maintain immutable previous archives.

## 15. Publication and Authority

A useful, fully validated checkpoint may be committed and pushed while MORPH-005 remains PARTIAL.

Do not confuse publication with phase completion.

Explicitly identify unmet gates.

Do not promote alternative collision authority or history migration without operator approval.

Do not commence MORPH-006 until stabilization completion is established or the operator explicitly authorizes a revised development boundary.

## 16. Governing Principle

The objective of MŒBIUS AUTOMATA is to investigate what emerges when rich automata inhabit a rich, persistent, three-dimensional physical environment.

The engine must make this richness economically possible.

We are not optimizing creatures into simplicity.

We are optimizing the computational machinery that permits their complexity.

**Optimize the cost of causality, not its richness.**

Proceed autonomously within these authority boundaries.