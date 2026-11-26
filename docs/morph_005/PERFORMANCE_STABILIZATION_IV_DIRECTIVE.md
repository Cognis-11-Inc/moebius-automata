# MŒBIUS AUTOMATA
## MORPH-005 — Performance Stabilization IV
### Local Collision Architecture, Continuous Casting and Historical Continuity

## 0. Mission

Resume MORPH-005 from Grok's current dirty working tree.

MORPH-005 remains PARTIAL.

The spatial horizon has demonstrated useful exclusion of distant geometry, but local hollow-mesh collision queries, continuous shape casting and authoritative history retention remain substantial architectural constraints.

The objective of this phase is to isolate, understand and resolve these constraints without reducing physical fidelity, biological potential or deterministic continuity.

**MORPH-006 and biological expansion remain suspended.**

Do not restart the stabilization investigation from zero.

## 1. Mandatory Handoff and Evidence Recovery

Read:

- AGENTS.md
- HANDOFF.md
- docs/morph_005/PERFORMANCE_STABILIZATION_II.md
- docs/morph_005/PERFORMANCE_STABILIZATION_III.md
- docs/morph_005/SPATIAL_LOCALITY_AUDIT.md
- docs/morph_005/QUERY_MULTIPLICITY.md
- docs/morph_005/SENSORY_ARCHITECTURE.md

Inspect the current working tree, Git diff, benchmark infrastructure and Mermogon evidence archives.

Grok reported:

- HEAD: `0a6390950ecb75f5b866e17f77ce54b5d7115075`
- Dirty working tree.
- 239 workspace tests passed.
- Clippy passed.
- Completed benchmark rows replayed exactly.
- Material gap: zero.
- MORPH-006 not started.
- No commit or push.

Stabilization III archive:

- Name: `performance-stabilization-iii-v1`
- SHA-256: `db2ae3d3fe36b82e8f9afe228bff7d005de299881b4f7fa23b49c8287f576368`

Independently reconcile these claims against the actual artifacts.

Preserve all successful and unsuccessful experiments.

Do not overwrite sealed evidence, discard Grok's changes or resurrect previously rejected optimizations without new evidence.

## 2. Architectural Priorities

Execute the following workstreams in priority order:

- P0-A: Isolated hollow-pipe collision investigation.
- P0-B: Continuous shape-cast attribution and optimization.
- P1-A: Procedural geometry acceleration.
- P1-B: Historical continuity and bounded resident storage.
- P2-A: Incremental pipe remeshing.
- P2-B: Cold-start body geometry construction.

These priorities may be reordered only when measured causal evidence justifies it.

Document any reordering.

## 3. P0-A — Isolate the Pathological Physical Interaction

The previous investigation identified:

### Eight-founder tick 32

- Tick duration: 77.4 ms.
- Distance searches: 873.
- 402 searches attributed to one cast involving body 12 and pipe 6, chunk 0.

### Fresh-colony tick 34

- Tick duration: 102.8 ms.
- Distance searches: 524.
- 513 searches involving body 8 and one pipe chunk.

These are inherited measurements requiring independent reconciliation.

Extract reproducible fixtures representing the expensive interactions from the preserved authoritative states.

Construct a minimal benchmark containing:

- One authentic body collision representation.
- One authentic hollow pipe chunk.
- Exact initial body pose.
- Exact movement and rotation.
- Exact pipe geometry and revision.
- Applicable physical tolerances.
- Original collision query parameters.

The isolated benchmark must preserve the actual query semantics.

Do not replace the pathological geometry with a convenient synthetic approximation.

### Required attribution

Determine where each expensive search originates.

Distinguish:

- Candidate generation.
- Compound-shape traversal.
- Triangle traversal.
- Continuous collision advancement.
- Distance-kernel invocation.
- Repeated support mapping.
- GJK convergence.
- EPA, where applicable.
- Contact recovery.
- Numerical refinement.
- Distinct intermediate poses.
- Repeated evaluations of identical inputs.

Produce a trace explaining the complete cost of the interaction.

Repeated calls are not automatically redundant.

Identify precisely which operations are mathematically necessary, implementation-dependent or potentially avoidable.

## 4. P0-B — Continuous Shape-Cast Investigation

The endurance run exposed a separate bottleneck:

- Steady mean: 61.89 ms.
- Distance time: approximately 3.6 ms per tick.
- Horizon maintenance: approximately 0.10 ms per tick.
- Cast timer: approximately 38 ms per tick.
- 140 ticks exceeded 100 ms while spending less than 1 ms on recorded distance computation.

The current distance counter does not capture all linear `cast_shapes` work.

This is now a primary investigation target.

Instrument the complete authoritative casting path.

Establish:

- Exact invocation sites and consumers.
- Number of casts per tick.
- Number of distinct geometric pairs.
- Geometry types involved.
- Shape complexity.
- Broad-phase candidate counts.
- Narrow-phase invocation counts.
- Time spent inside each relevant collision-library operation.
- Whether casts repeat because of recovery, iteration, movement subdivision or another mechanism.
- Whether identical or conservatively equivalent work is repeated.

Produce isolated fixtures for representative expensive casts from the endurance run.

Investigate whether existing movement bounds, geometric revisions and previous contact information can lawfully eliminate unnecessary casting work.

Do not optimize only the separately instrumented distance kernel while ignoring the larger casting pipeline.

## 5. P1-A — Exploit Procedural Pipe Geometry

M.A. constructs its own hollow structures.

Investigate whether authoritative pipe construction data can accelerate interaction queries more efficiently than treating every pipe as an arbitrary triangle mesh.

Perform repository archaeology before proposing any representation change.

Identify whether the following information exists, persists and has lawful consumers:

- Centreline geometry.
- Section orientation.
- Inner radius.
- Outer radius.
- Wall thickness.
- Segment connectivity.
- Local curvature.
- Junction geometry.
- Construction and deformation revisions.

Do not assume these fields exist.

### Candidate investigations

Evaluate, where supported by actual geometry:

- Conservative segment bounds.
- Hierarchical subdivision of hollow pipe chunks.
- Annular or tubular exclusion bounds.
- Localized triangle candidate sets.
- Persistent per-segment acceleration structures.
- Conservative closest-feature reuse.
- Geometry-aware continuous casting.
- Validated temporal separation certificates.

These are investigation candidates, not mandatory implementations.

The preferred solution should reduce the amount of geometry examined rather than repeatedly accelerate the same exhaustive work.

### Authority restriction

The existing authoritative collision geometry remains authoritative.

Do not replace it with capsules, solid cylinders, simplified tubes or approximate meshes merely to improve benchmark results.

In particular, preserve:

- Hollow interiors.
- Openings.
- Inner-wall contacts.
- Outer-wall contacts.
- Junctions.
- Grazing contacts.
- Rotational interactions.
- Deformed geometry.
- Relevant material and structural boundaries.

Conservative acceleration may exclude impossible contacts.

It must never exclude admissible contacts.

Any proposed alternative authoritative representation requires a separate operator decision and a dedicated equivalence study.

## 6. Adversarial Collision Validation

Every candidate optimization must be challenged against the existing reference implementation.

Include cases involving:

- Entry into hollow pipes.
- Motion through pipe interiors.
- Approach to inner and outer walls.
- Grazing contact.
- Narrow clearance.
- Combined translation and rotation.
- Curved and irregular sections.
- Pipe ends and junctions.
- High relative movement.
- Contact recovery.
- Geometry changes during successive ticks.
- Newly introduced geometry.
- Previously cached geometric relationships becoming invalid.

Require exact authoritative state, contact and replay equivalence under the established MORPH-005 contract.

Do not accept matching final snapshots alone when intermediate contacts or state transitions differ.

Preserve negative results.

## 7. P1-B — Historical Continuity Architecture

The endurance execution stopped at tick 1863 because of `HistoryBudget`.

Reported state:

- 65,505 stored events.
- `max_events = 65,536`.
- Next authoritative tick refused.
- No authoritative ticks discarded.
- Event cap not raised.

The refusal is preferable to silent history destruction, but an indefinitely persistent vivarium requires a lawful strategy for historical growth.

Investigate the actual event architecture.

Identify:

- Event producers.
- Event consumers.
- Event ordering.
- Replay dependencies.
- Snapshot dependencies.
- Checkpoint semantics.
- Historical query requirements.
- Material-ledger dependencies.
- Whether complete history is required for the next authoritative tick.
- Resident-memory and persistent-storage ownership.

### Design objective

Separate, where lawful:

1. Active authoritative simulation state.
2. Immediately required historical context.
3. Immutable historical records.
4. Persistent checkpoints.
5. Replay reconstruction.

Investigate append-only archival segments, checkpoint-bound historical storage and verified restoration.

Prefer existing infrastructure over creating a competing persistence mechanism.

### Mandatory restrictions

- Do not silently discard events.
- Do not arbitrarily raise `max_events`.
- Do not redefine replay to exclude inconvenient history.
- Do not weaken identity or material continuity.
- Do not introduce an unbounded resident-memory structure.
- Do not make rendering or Witness observation responsible for persistence.

Before implementing a substantial history-authority redesign, produce an explicit proposal identifying changed contracts and required migration semantics.

Obtain operator approval if the proposal changes the existing authoritative history model.

A bounded in-memory buffer and lossless historical storage may be appropriate, but their feasibility must be demonstrated rather than assumed.

## 8. P2 — Geometry Construction and Remeshing

Investigate the remaining secondary costs:

- Pipe remeshing.
- Cold-start body-compound construction, reported at approximately 187–207 ms.

Determine actual geometry ownership and invalidation semantics.

Evaluate:

- Incremental reconstruction.
- Dirty-region maintenance.
- Reuse of unchanged segments.
- Persistent acceleration structures.
- Reuse of genuinely identical body geometry.
- Separation of render-only data from authoritative physical geometry.

Do not introduce unbounded caches.

Do not share mutable geometry between organisms without proving ownership and invalidation correctness.

Cold-start and steady-state costs must remain separately reported.

## 9. Performance Investigation Discipline

Stop running the complete benchmark matrix after every speculative change.

Use an escalating evidence sequence:

### Gate A — Isolated reproduction

Reproduce the expensive interaction using authenticated fixtures.

### Gate B — Causal improvement

Demonstrate that the candidate removes or reduces the identified expensive operation.

Measure both execution time and operation counts.

### Gate C — Exact equivalence

Pass focused collision, conservation and replay tests.

### Gate D — Matched colony continuation

Demonstrate improvement on the original pathological colony cases.

### Gate E — Full stabilization matrix

Only after the preceding gates succeed, execute the complete matrix.

Reject modifications that merely relocate cost into another timer, increase memory retention or create regressions elsewhere.

Preserve before-and-after measurements and executable identities.

Do not claim an optimization solely from instrumented timing.

Confirm accepted improvements using ordinary release builds.

## 10. Spatial Locality Completion

Preserve Grok's successful distant-pipe horizon mechanism.

Previous experiments translated the scene and introduced distant geometry.

Translation is not equivalent to enlarging the habitat itself.

Complete the outstanding actual habitat-dimension experiments where the current implementation supports them:

- Original habitat dimensions.
- 10× linear dimensions.
- 100× linear dimensions.

Maintain equivalent relevant local physical conditions.

Distinguish:

- Empty volume.
- Distant occupied volume.
- Local structural density.
- Coordinate magnitude effects.

If the architecture cannot represent these experiments without changing physical conditions, report that limitation explicitly.

Do not manufacture equivalent conditions through hidden changes.

## 11. Population Scaling and Endurance

Investigate the reported discontinuity between one and two founders.

Previous fresh population measurements:

| Founders | Mean tick |
|---|---:|
| 1 | 0.45 ms |
| 2 | 7.32 ms |
| 4 | 8.45 ms |
| 8 | 17.29 ms |
| 16 | 49.60 ms |

Determine whether this growth originates from:

- Population itself.
- Structural geometry.
- Interaction density.
- Collision recovery.
- Shape-casting complexity.
- Different colony trajectories.
- Another measured causal mechanism.

Do not infer asymptotic complexity from these five observations alone.

The current founding limit is reportedly 16.

Do not arbitrarily raise that limit to obtain a larger benchmark.

Establish why it exists and report what would be required to extend it lawfully.

## 12. Sensory Architecture Reservation

Preserve the existing sensory architecture contract.

Do not implement sensory organs during this stabilization phase.

Ensure that spatial infrastructure developed here could eventually serve bounded sensory consumers without conflating perception with authoritative physical interaction.

Future sensory systems must remain:

- Local.
- Modality-specific.
- Range-bounded.
- Cadence-aware.
- Non-omniscient.
- Compatible with shared spatial infrastructure.

Do not allow speculative future requirements to expand the present stabilization work unnecessarily.

## 13. Final Acceptance Matrix

Retain all previous MORPH-005 requirements.

Complete:

- One-founder case.
- Two-founder case.
- Four-founder case.
- Eight-founder case.
- Sixteen-founder case.
- Ordinary 420-tick colony.
- Eight-founder resume.
- Congested save.
- Native 720-tick execution.
- Complete 3,600-tick endurance execution.
- Spatial locality experiments.
- Adversarial collision equivalence tests.
- History restoration and replay validation, if modified.

Report:

- Mean.
- Median.
- P95.
- P99.
- Maximum.
- Steady-state mean.
- Cold-start cost.
- Collision-query multiplicity.
- Shape-cast multiplicity.
- Geometry maintenance cost.
- Requested allocations.
- Resident memory.
- Historical storage growth.

Retain the established targets:

- Authoritative tick duration: 16.667 ms.
- Provisional mean target: ≤ 5 ms.
- Preferred mean target: ≤ 2 ms.
- P99 target: ≤ 8 ms.
- Previously established allocation-reduction and memory-stability requirements.

Do not weaken the gates to declare success.

## 14. Deliverables

Produce:

- `PERFORMANCE_STABILIZATION_IV.md`
- Isolated collision fixture suite.
- Hollow-pipe collision investigation.
- Continuous shape-cast attribution report.
- Accepted optimization evidence.
- Rejected candidate evidence.
- Historical continuity architecture assessment.
- Updated spatial locality results.
- Full matched benchmark matrix when qualified.
- Updated HANDOFF.md.
- Updated Atlas documentation.
- Full workspace test results.
- Clippy and formatting verification.
- Restore-verified Mermogon evidence archive.

Maintain exact artifact identities, source revisions and executable provenance.

Do not rely exclusively on tmpfs for evidence.

## 15. Completion Authority

If every applicable stabilization gate passes:

- Verify the final source tree and executable identity.
- Complete the evidence archive.
- Commit and push according to repository policy.
- Report MORPH-005 stabilization completion.

If any gate remains unmet:

- Report PARTIAL or FAIL.
- Preserve all improvements and evidence.
- Identify the remaining causal bottleneck.
- Do not claim completion.
- Do not start MORPH-006.

## 16. Governing Principle

MŒBIUS AUTOMATA is not being developed to demonstrate how much computation can be consumed by a handful of creatures.

It is being developed to investigate what emerges when rich automata inhabit a rich, persistent, three-dimensional physical environment.

The engine must make that richness economically possible.

**Optimize the cost of causality, not its richness.**

Proceed autonomously within the established authority boundaries.