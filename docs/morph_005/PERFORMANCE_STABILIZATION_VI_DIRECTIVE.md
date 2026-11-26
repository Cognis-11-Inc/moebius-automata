# MŒBIUS AUTOMATA
## MORPH-005 — Performance Stabilization VI
### Deterministic Runtime Parallelism and Exact Geometry Reuse

## 0. Mission

Continue from published checkpoint `9a5b85d`.

Stabilization V established:

- Exact isolated collision-query execution across 1–32 workers.
- 5.35× isolated endurance-query throughput at eight workers.
- Approximately single-core utilization in the measured headless runtime.
- Geometry construction accounting for 62.19% of remaining requested allocations.
- Verified causal dependencies preventing unrestricted parallel collision admission.

Stabilization VI must exploit these findings through qualified runtime implementation.

This is primarily an implementation phase, not another broad architectural investigation.

**MORPH-005 remains PARTIAL. MORPH-006 and biological expansion remain suspended.**

## 1. Entry Verification and Containment

Reconcile the current repository with:

- AGENTS.md
- HANDOFF.md
- PERFORMANCE_STABILIZATION_IV.md
- PERFORMANCE_STABILIZATION_V.md
- AUTHORITATIVE_TICK_DEPENDENCIES_V.md
- PHYSICAL_GEOMETRY_AUTHORITY_V.md
- HISTORY_COMPATIBILITY_V_PROPOSAL.md
- Relevant validation, storage and benchmark receipts.

Verify:

- Local and remote commit identity.
- Working-tree cleanliness.
- Runtime validation checkpoint `eea57b210be520ddb2d25a160dc9252e6a475bd0`.
- Mermogon archive integrity.
- Authenticated collision fixtures and benchmark executables.
- Existing query executor implementation and exact-result tests.

Reuse completed archaeology. Reopen source investigations only where necessary for implementation.

Preserve all previous evidence and rejected candidates.

Do not alter collision authority, history encoding, biological rules, material accounting, population limits, contact tolerances or coordinate representation.

## 2. P0 — Bounded Runtime Parallel Query Execution

### 2.1 Objective

Integrate a deterministic, bounded parallel query mechanism into the authoritative collision pipeline.

Reuse the validated isolated executor's numerical kernels, authentication principles and worker-local workspace architecture wherever appropriate.

Do not simply move the entire experimental fixture executor into production.

The authoritative runtime must retain its existing sequential mutation and admission semantics.

### 2.2 Initial integration seams

Evaluate the following seams in order of causal simplicity:

1. Independent non-outlet component queries within source-graft checks.
2. Fixed-pose contact samples within nondeepening recovery.
3. Independent immutable geometry preparation.
4. Bounded body-static query frontiers.
5. Bounded body-body query frontiers.

This ordering is a proposed implementation strategy. Actual measured benefit and verified dependencies determine which candidates advance.

Do not force an unprofitable seam into production merely because it is parallelizable.

### 2.3 Query frontier contract

A parallel frontier must originate from an explicitly identified authoritative solver state.

Each descriptor must carry or reference all inputs necessary to authenticate its result, including:

- Canonical query identity and ordering.
- Body and target identities.
- Immutable geometry ownership.
- Geometry revisions.
- All four endpoint poses.
- Query parameters and tolerances.
- Relevant solver context.

Workers evaluate numerical queries only.

Workers must not:

- Modify authoritative world state.
- Admit contacts.
- Update movement targets.
- Emit authoritative events.
- Change material balances.
- Mutate structural assemblies.
- Grant geometry or history authority.

The existing solver remains the sole consumer responsible for admission.

### 2.4 Result receipt and canonical fallback

Consume results in the original canonical order.

Before consuming a result, verify that its complete authoritative query inputs still match.

If any relevant input has changed:

1. Reject the stale result.
2. Record the rejection in diagnostic builds.
3. Execute the canonical query using current authoritative inputs.
4. Continue through the original admission path.

A stale result must never influence collision decisions.

Do not introduce approximate pose matching or incomplete revision checks.

### 2.5 Speculative computation boundaries

Speculative queries may be evaluated ahead of their canonical consumption point.

Their existence must not alter:

- Early termination.
- Short-circuit evaluation.
- Error visibility.
- Error precedence.
- Contact ordering.
- Solver mutation ordering.
- Transaction rollback.

An error from a speculative query that would never have been reached by the sequential solver must not become an authoritative runtime error.

Test this explicitly.

### 2.6 Worker ownership

Investigate a persistent bounded executor suitable for actual runtime integration.

Require:

- Explicit executor ownership.
- Bounded worker count.
- Bounded descriptor and result capacity.
- Worker-local collision scratch storage.
- Immutable geometry sharing.
- No threads created per tick.
- No unbounded task accumulation.
- No global mutable collision workspace.
- Clean shutdown and failure handling.
- A canonical sequential fallback.

Evaluate whether small frontiers should bypass parallel dispatch.

Derive dispatch thresholds from measured costs, not intuition alone.

### 2.7 Runtime configuration

Support a controlled sequential reference configuration and configurable worker counts for qualification.

Evaluate 1, 2, 4, 8, 16 and 32 workers where useful.

Eight workers is the initial experimental candidate because V measured its best mean throughput.

It is not an established production default.

The selected runtime policy must account for actual frontier size, useful work, memory duplication and rendering contention.

## 3. P0 — Parallelism Qualification

### 3.1 Required measurements

For every runtime candidate, record:

- Number of eligible queries.
- Number dispatched.
- Number consumed.
- Number discarded as stale.
- Number discarded as unnecessary.
- Number recalculated sequentially.
- Actual frontier-size distributions.
- Worker utilization.
- Queue and synchronization costs.
- Canonical fallback cost.
- Requested allocation volume.
- Worker workspace memory.
- Mean, median, P95, P99 and maximum tick durations.

Distinguish useful parallel computation from speculative waste.

An isolated throughput improvement is not sufficient.

### 3.2 Mandatory falsification tests

Demonstrate rejection of:

- Stale poses.
- Stale geometry revisions.
- Changed target identities.
- Same-tick geometry mutations.
- Altered descriptor inputs.
- Incorrect result ordering.
- Invalid worker receipts.
- Speculative errors from unreachable branches.

Exercise worker failure, bounded-queue saturation and sequential fallback.

Require honest-red evidence for the critical admission protections.

### 3.3 Determinism

Compare sequential and parallel execution using identical authenticated initial worlds.

Require:

- Exact applicable intermediate states.
- Exact contact results and ordering.
- Exact movement admission.
- Exact material balances.
- Exact event ordering and payloads.
- Exact final snapshots.
- Exact replay.

Repeat across qualified worker counts.

Do not accept statistical equivalence in place of the existing exact contract.

### 3.4 Runtime acceptance

A parallel runtime candidate qualifies only if ordinary release measurements demonstrate meaningful improvement after accounting for dispatch, stale work, fallback and memory overhead.

Reject or redesign candidates that:

- Accelerate isolated queries but regress authoritative ticks.
- Increase tail latency unacceptably.
- Introduce excessive speculative waste.
- Cause uncontrolled memory growth.
- Harm native rendering responsiveness.
- Weaken causal correctness.

Preserve rejected implementations and measurements.

## 4. P1 — Exact Geometry Construction Reuse

### 4.1 Objective

Reduce the largest measured remaining allocation scope without changing physical geometry.

V measured:

- Original baseline: 263,041,278 requested bytes.
- Current diagnostic execution: 23,606,814 requested bytes.
- Geometry construction: 14,682,192 requested bytes.
- Geometry construction calls: 18 across three measured ticks.

Geometry construction represents approximately 62.19% of remaining requested bytes.

If other allocation scopes remain constant, reducing geometry allocations by approximately 71.2% would satisfy the original overall 95% reduction target.

This is an arithmetic opportunity estimate, not a demonstrated achievable reduction.

### 4.2 Duplicate construction attribution

Investigate the exact relationship between:

- `collision::deposit_allowed`
- `Assembly::deposit`
- `CollisionCache::pipe_in`
- `pipe_key`
- `filament_chunk_at`
- `geometry`

Determine which geometry is constructed during deposition preflight and which is subsequently reconstructed for authoritative collision use.

Record:

- Exact input fingerprints.
- Candidate versus admitted assembly identity.
- Geometry revision.
- Mesh time.
- Paid-node state.
- Vertex/index fingerprints.
- Number of duplicate constructions.
- Requested bytes.
- Construction time.

Do not assume preflight geometry and admitted geometry are equivalent.

Establish that equivalence for each proposed reuse case.

### 4.3 Transactional geometry handoff

Where exact equivalence is established, investigate reusing immutable geometry generated during deposition preflight after successful authoritative deposition.

Potential architecture:

- Build candidate geometry.
- Evaluate the existing deposition predicates.
- Admit deposition through existing authoritative paths.
- Verify the committed assembly inputs against the candidate geometry receipt.
- Reuse the immutable geometry only when its complete inputs match.
- Otherwise discard it and reconstruct canonically.

Candidate geometry must not acquire authority over:

- Material payment.
- Node admission.
- Structural junctions.
- Book mutation.
- Collision admission.
- History.

Rejected deposits must never contaminate the committed geometry cache.

Test rollback and same-tick subsequent mutations explicitly.

### 4.4 Wet-mesh intermediate reuse

Investigate reuse of unchanged computational intermediates:

- Centreline interpolation.
- Tangent calculations.
- Local frames.
- Ring topology.
- Index arrays.
- Static phase-dependent calculations.
- Valid reusable construction buffers.

Preserve the original arithmetic order wherever it affects authoritative vertex bits.

Wet radii evolve independently. Reuse must not suppress required age-dependent geometry changes.

Account for:

- Neighbouring-node dependencies.
- Chunk origin changes.
- Append operations.
- Chewing and removal.
- Endpoint status.
- Cap and seal changes.
- Nozzle inputs.
- Same-tick mutation tokens.
- Future-node age handling.
- Wet-to-dry transitions.

Retain exact physical mesh positions, indices, ordering, orientation and collision pseudo-normal semantics.

### 4.5 Acceleration structures

Continue using supported collision-library APIs.

Do not assume safe arbitrary vertex refitting exists.

Do not introduce unsafe private-field mutation, unsupported dependency modifications or approximate collision geometry.

If exact acceleration-structure reuse cannot be established, retain canonical reconstruction and optimize lawful preceding work.

## 5. Geometry Reuse Qualification

For every accepted reuse candidate, demonstrate:

- Exact geometry fingerprints.
- Exact vertex and index data.
- Exact collision queries.
- Correct invalidation.
- Preserved hollow interiors.
- Preserved openings and seals.
- Preserved structural connectivity.
- Preserved material accounting.
- Exact matched continuations.

Test relevant mutation classes individually.

Include negative tests proving that incomplete keys, obsolete geometry and rejected deposition candidates cannot be reused.

Measure requested allocation bytes and retained memory independently.

No unbounded geometry retention is authorized.

## 6. Independent Optimization Lanes

Qualify parallel execution and geometry reuse separately before combining them.

Use controlled configurations:

- A: published IV/V-equivalent sequential reference.
- B: parallel runtime candidate only.
- C: geometry reuse candidate only.
- D: combined qualified candidate.

Where implementation permits, isolate the relevant configuration difference without changing unrelated code paths.

Compare the same authenticated inputs.

Record source and executable identities for every variant.

This separation is mandatory to establish causal attribution.

An improvement in D does not automatically establish that both B and C are beneficial.

## 7. Contact Recovery

V established that recovery is computationally significant but accounts for relatively little remaining requested allocation volume.

Retain recovery as a performance target, not an allocation priority.

Investigate fixed-pose sample parallelism and conservative witness reuse only where warranted by measured cost.

Preserve:

- Initial inward-normal checks.
- Original sample locations.
- Nondeepening comparisons.
- Original ordered predicates.
- Early-exit semantics.
- Current tolerances.
- Canonical fallback.

Do not reduce sample counts or weaken difficult-contact handling.

## 8. Benchmark Qualification Sequence

### Gate A — Implemented integration

Establish an actual authoritative runtime call site.

An isolated executor or registry entry is not sufficient.

### Gate B — Focused correctness

Pass authentication, deterministic ordering, stale-result rejection, rollback and geometry invalidation tests.

### Gate C — Authenticated continuation

Compare against retained pathological fixtures.

Require every applicable intermediate state and full final snapshot to match.

### Gate D — Ordinary release performance

Demonstrate measurable runtime improvement with diagnostics disabled.

Separate cold-start, steady-state and tail costs.

### Gate E — Combined qualification

Combine independently successful candidates.

Establish that they do not interfere with each other's correctness, memory ownership or performance.

### Gate F — Full matrix

Run the complete established matrix only after candidates demonstrate meaningful integrated benefit.

Do not spend resources repeatedly rerunning the full matrix after inconclusive isolated changes.

## 9. Full Performance Matrix

Retain the existing cases:

- One founder, 120 ticks.
- Two founders, 120 ticks.
- Four founders, 120 ticks.
- Eight founders, 120 ticks.
- Sixteen founders, 120 ticks.
- Fresh colony, 420 ticks.
- Eight-founder resume, 120 ticks.
- Congested save, 30 ticks.
- Native validation, 720 ticks.
- Opt-in v2 endurance, 3,600 ticks.

Retain original authenticated input configurations.

Do not alter population limits or manufacture unsupported habitat scaling.

Report actual simulation timing separately from rendering timing.

For native execution, assess competition between collision workers and the renderer, including frame intervals and authoritative tick latency.

## 10. Acceptance Targets

Retain the established MORPH-005 targets:

- Provisional mean tick duration ≤ 5 ms.
- Preferred mean tick duration ≤ 2 ms.
- P99 ≤ 8 ms.
- Real-time authoritative tick budget approximately 16.667 ms.
- Requested allocation reduction ≥ 95%.
- Complete 3,600-tick endurance within the 180-second gate.
- Exact applicable replay.
- Exact authoritative state equivalence.
- Material conservation.
- No missed admissible contacts.
- Correct geometry invalidation.
- Bounded memory under qualified workloads.

Do not reinterpret these gates after observing the results.

Report each independently as PASS, FAIL or INDETERMINATE.

## 11. Deferred Investigations

Do not pursue during VI unless required by a demonstrated blocker:

- Alternative physical collision representations.
- Coordinate-system redesign.
- Historical source-version migration.
- New biological systems.
- Neural architectures.
- Sensory organs.
- Population-limit expansion.
- GPU collision offloading.

Preserve the existing proposals and architectural reservations.

If exact CPU-side implementation proves insufficient, document the measured limiting mechanism and propose the next architectural investigation separately.

## 12. Historical Continuity

Preserve history v1 and the opt-in v2 prototype.

Do not bypass source-fingerprint verification to run new binaries against incompatible checkpoints.

Use the established lawful reproduction procedure, compatible retained executables and authenticated initial worlds.

Preserve immutable historical segments and checkpoint identity.

Any new history-authority changes require separate operator approval.

## 13. Deliverables

Produce:

- PERFORMANCE_STABILIZATION_VI.md.
- Runtime parallel executor implementation and integration evidence.
- Query-frontier authority and receipt specification.
- Useful versus discarded speculative-work attribution.
- Worker-scaling results from actual runtime execution.
- Exact geometry reuse implementation and evidence, where qualified.
- Duplicate construction attribution.
- Independent and combined optimization comparisons.
- Determinism, rollback and invalidation tests.
- Honest-red validation receipts.
- Full benchmark matrix for qualified candidates.
- Clean-source validation receipts.
- Updated HANDOFF and Atlas.
- Restore-verified immutable Mermogon evidence archive.

Distinguish implemented, proposed, rejected, validated and untested capabilities.

Preserve complete provenance for failed experiments.

## 14. Publication

Commit and push qualified work and evidence.

A meaningful validated checkpoint may be published while MORPH-005 remains PARTIAL.

Publication does not waive failed performance gates.

Do not commence MORPH-006 without established stabilization completion or explicit operator authorization.

## 15. Governing Principle

Parallelize computation, not authority.

Reuse information only while its causal inputs remain valid.

Preserve physical richness while reducing its computational cost.

**Optimize the cost of causality, not its richness.**

Proceed autonomously within these boundaries.