# MORPH-005 — Performance Stabilization V

2026-11-26 America/Toronto, Codex. **PARTIAL — investigation checkpoint.**
The isolated query executor and attribution are implemented and measured. No
parallel executor is installed in ShipRuntime, no alternative collision model
is adopted, and no engine performance gate is newly passed. MORPH-006 and
biological expansion remain suspended.

## Containment and authority

Risk: parallel evaluation can conceal stale poses, changed geometry, changed
error ordering or duplicated memory. Preserve all canonical numerical queries,
ordered contact admission, tolerances, material/history guards, current population
limits and spatial invalidation. No approximate geometry, source-fingerprint
bypass, changed cap, per-tick thread creation, global collision workspace,
unordered numerical reduction or renderer mutation is authorized by this work.

Target: authenticate retained query descriptors, evaluate them concurrently with
bounded worker-local scratch, restore result ordering, and measure the difference
between throughput feasibility and causal tick integration. Add explicit recovery
observation using the same runtime, then attribute the remaining allocation
requests. Proof: exact captured contacts across all worker counts/repetitions,
tamper rejection and honest-red, matched full/intermediate world continuations,
source-identified builds, focused/static checks, clean-commit certification and
restore-verified evidence. The full timing matrix remains gated on a qualified
runtime optimization; this research executor is not such a candidate.

## Entry and retained baseline

VERIFIED: clean `main`, HEAD and remote main
`4e3eea6bae6b344351d4fbf5527e1b9b4435b087`. Its delta from clean runtime/test
certificate `ecdc8625b5d605997a6e3a559b36903f78231d33` is documentation/receipts.
The certificate records 253 tests, zero failed/ignored, empty before/after status
and 112.449 seconds; that is an inherited run, not a new V test execution.

Read the required IV report/results/certificate, history proposal, locality,
query-multiplicity and sensory contracts and the current handoff/architecture.
Rechecked Mermogon mount UUID `D0A02EEBA02ED82C`, all three IV archive/manifest
hashes and all 1532 members (1499 + 22 + 11), including failed-attempt evidence.
No sealed prior archive was altered. New input copies retain the same hashes.

Reconciled IV cases from retained RESULTS/report/tick evidence: founder 1/2/4/8/16
means 0.484/5.342/7.339/13.516/38.459 ms; fresh 11.083, eight-founder resume
13.635, congested 43.190, headless native-input 720 21.520, opt-in endurance
53.730 ms. Endurance forward 193.428 seconds exceeds 180; P99 108.044 ms.
These remain IV measurements. V does not relabel them as current benchmark runs.
Endurance linear 59.922 s, recovery 22.566 s, remesh 14.658 s and horizon 0.284 s
are retained overlapping timers, not additive independent costs.

## Dependency and CPU findings

[Authoritative dependency map](AUTHORITATIVE_TICK_DEPENDENCIES_V.md) lists exact
producers, consumers, mutation rights, classifications, reuse rulings and omissions.
Runtime contact responses update current targets inside ordered body/body and
body/static loops. Recovery and later passes consume those updates. Consequently
a whole captured tick is not one lawful parallel admission batch.

VERIFIED hardware: Ryzen 9 7950X, one socket, 16 cores, two threads/core, 32
online/allowed logical CPUs, one NUMA node. Inspected every current cgroup ancestor:
CPU quota is `max`, memory limits are `max` where exposed, effective CPUs 0..31.
This does not reserve those CPUs against other desktop work. Entry load was low;
clock/boost and OS scheduling were not pinned.

A retained ordinary IV executable continued the authenticated congested input
for 30 ticks and replayed it. Every intermediate row and full final world match
the retained matrix. Process CPU/wall was 0.990 and sampling observed one thread:
this recorder uses approximately one core, not the available 32. Its process
measurement includes forward execution, replay and observer work; it is not native
render-thread utilization. Source confirms the separate Witness numerical worker
and Bevy renderer. No current native responsiveness measurement is claimed.

## Deterministic parallel feasibility

IMPLEMENTED behind existing `collision-fixture`: `collision_fixtures::batch::Batch`
and `collision_fixture --batch`. Canonical body/pipe constructors authenticate
each input before dispatch. Fixed immutable geometry and four poses are the query
inputs. Each worker retains its own EPA/support workspace, clears contact receipts
between descriptors, and calls the existing private `cast`. Bounded channels return
results to their original indices. Exact TOI bits, integer normal and contact point
must equal the captured reference; no worker can admit or mutate world state.

Measured with one captured release executable/source archive, 1/2/4/8/16/32 workers,
two processes per cell in ascending then descending worker order, 30 rounds each.
First round is separated; warm means use 58 rounds. The empirical P99 below has
only 58 observations per cell and is not a stable-tail guarantee.
The reported quantile uses the lower order statistic at floor(0.99*(n-1)).
Pool construction is outside round timers and included in process CPU/wall.

| Workers | Eight single query mean ms | Fresh single query mean ms | Endurance 370 queries mean ms | Endurance empirical P99 ms | Endurance relative throughput |
|---:|---:|---:|---:|---:|---:|
| 1 | 17.876 | 60.186 | 73.636 | 75.067 | 1.00x |
| 2 | 16.894 | 60.512 | 38.638 | 40.551 | 1.91x |
| 4 | 16.963 | 61.419 | 20.110 | 21.678 | 3.66x |
| 8 | 16.788 | 60.149 | 13.768 | 17.477 | 5.35x |
| 16 | 16.881 | 60.696 | 14.030 | 16.697 | 5.25x |
| 32 | 16.786 | 60.558 | 13.845 | 18.725 | 5.32x |

VALIDATED: all 133920 query results across these 36 processes match their authentic
reference exactly; each scene has one ordered-result hash across counts/rounds.
Single-query cells cannot occupy multiple workers. Their small timing variation
does not establish scaling. Endurance saturates around eight workers: process
CPU/wall about 3.7, query elapsed occupancy 0.753 of eight workers, falling to
0.220 at 32. Query elapsed occupancy includes scheduling effects and is not CPU
occupancy. Wall minus maximum worker query elapsed is about 0.04–0.05 ms/round;
it combines dispatch, waiting, result collection and imbalance, rather than
isolating lock costs. CPU/RSS/first-round/worker rows and executable identities
are retained in the raw archive. Per-worker allocation request counts and native
rendering responsiveness across worker counts are NOT MEASURED by this assay.

The 370 descriptors span 23 distinct body/target recipes and 42 consecutive pose
groups, including bodies with changed targets. Their concurrent evaluation proves
query throughput, not admissibility as one runtime wave, replay of a parallel
tick, or a 5.35x simulation gain. A bounded frontier/prefetch design with exact
pose/revision receipt checks and canonical fallback remains PROPOSED. Lazy error
visibility, unused speculative work, allocation overhead and native contention
must be measured before adopting it. No full engine matrix is justified yet.

## Recovery attribution and conservative reuse

IMPLEMENTED explicit `habitat_probe --capture-recovery` observer under
`collision-fixture`. It records pre-step tick, body/target, full geometry hashes,
four poses, sample time/depth bits and elapsed times. Geometry fingerprinting is
outside recorded recovery time; the instrumented executable is not a timing
oracle. Recording stops before replay. Diagnostic count is bounded at 8192 with
explicit failure, never silent truncation.

| Authenticated continuation | Ticks | Recovery calls | Contact samples | Distinct exact geometry/pose recipes |
|---|---:|---:|---:|---:|
| Eight from pre-step 31 | 8 | 0 | 0 | 0 |
| Fresh from pre-step 33 | 8 | 0 | 0 | 0 |
| Endurance from pre-step 1667 | 8 | 34 | 113 | 24 |
| Congested from pre-step 720 | 4 | 69 | 303 | 39 |

All 28 forward intermediate rows and all four full final worlds equal independent
retained ordinary-runtime continuations. Each recorder also restores/replays
exactly with zero material gap. Initial unlabelled three-tick probes are retained;
the labelled congested probe includes an extra tick to expose the same warmup and
722–724 range as allocation attribution. They are distinct runs, not conflicting
counts. Allocation probe's warmup is tick 721 and is excluded from its totals.

Recovery is invoked only after an initial sweep hit at TOI <=0.00001. Stationary
or changing-orientation proposals refuse recovery early. Otherwise the initial
contact and inward-normal check precede samples 0.25, 0.5, 0.75, 1; any deepening
refuses. The endurance histogram is 1 empty, 13 one-sample, 20 five-sample calls;
congested is 9 one-sample, 3 three-sample, 57 five-sample calls. Full per-body/tick
and target multiplicities are in RECOVERY_ATTRIBUTION JSON. For example endurance
body54/filament16/chunk2 has 13 calls but nine distinct recipes.

Repeated recipes exist, but the existing contact cache already memoizes exact
relative transforms and prediction within a resolve; it clears at resolve entry.
Recipe repetition is not evidence that all repeated calls recompute every contact.
Normals/closest-feature witnesses can seed rejection checks only after current
geometry/pose validation, with fallback to all original tests. Cross-tick reuse
must also account for wet revision changes. No recovery iteration cap, tolerance,
sample, or difficult contact was removed. Witness-reuse optimization is NOT
IMPLEMENTED or qualified by this observer.

## Allocation, remeshing and physical authority

[Physical geometry audit](PHYSICAL_GEOMETRY_AUTHORITY_V.md) separates occupancy,
inner/outer contact, discrete material accounting, connectivity, attachments and
future perception. Ridges/wobble/ellipse/age/caps are current physical mesh inputs.
No alternative representation was prototyped or mixed into exact measurements.
It also gives local node dependencies, wet/dry revision ownership and the verified
limits of Parry's public API. Arbitrary safe vertex refitting is unavailable in
the inspected dependency; no internal mutation or flag removal was substituted.

On the authenticated congested fixture, new allocation-attribution execution
matches every measured digest at ticks 722–724. Requested bytes 23,606,814,
91.0254% below original 263,041,278: **95% target still fails**. The new diagnostic
total is 688 bytes above IV's ordinary-counted 23,606,126; no claim of byte-exact
allocation equivalence or a new allocation reduction is made. Instrumentation
and build identities differ; the small difference is retained, not rounded away.

| Exclusive allocation scope | Requested bytes / three measured ticks |
|---|---:|
| Geometry construction | 14,682,192 |
| Enclosing tick work | 4,966,776 |
| Resolve, excluding child scopes | 2,314,506 |
| Deposit admission, excluding geometry | 1,044,081 |
| State clone | 418,959 |
| Recovery | 18,240 |

All scope totals reconcile to the requested-byte counter. Geometry has 18 calls;
both cached pipe construction and deposition preflight call `geometry`, so
`pipe_builds` alone understates mesh construction. Allocation by individual Parry
container, individual shape cast, temporary type, snapshot clone, or diagnostic
stack is not separately resolved by these exclusive scopes. The enclosing tick
includes `physical_next` cloning and temporary collections; their individual byte
shares are not established. V1 event commit requests zero bytes in these ticks;
this does not establish zero history/snapshot cost in other workloads. Recovery's
small allocation cost does not contradict its large measured compute time.

Prioritization: exact local remesh/topology/collider-construction economy has a
larger measured allocation opportunity here than reducing recovery allocations.
Persistent immutable geometry should be reused only with complete revision keys;
speculative deposition geometry cannot acquire paid-node or admission authority.

`presentation_slack` owns a process-global mutex/BTreeMap keyed by serialized
anatomy hash, storing a hex-string key and scalar. It has no eviction. Per-body
Geometry's lazy OnceLock does not bound the process map across changing anatomy
definitions. Current fixed-definition probes cannot establish whether a long
editing/evolution session's retention is materially costly. The map is not a
cache of all rendered meshes; no unsupported large-mesh leak is claimed. Eviction
and changing-definition stress remain unimplemented/unmeasured.

The isolated 8-worker executor also evaluated 370000 exact queries over 1000
rounds in 13.588 s process wall time. Sampled RSS between seconds 2 and 12 was
42892–43012 KiB; the final observed high-water spike reached 58800 KiB. Output
construction/serialization follows the timed batch; its phase was not separately
instrumented, so that attribution is INFERRED. Retained round records and worker
workspace growth are finite in this assay. This is neither a flat whole-process
memory result nor proof of whole-engine/changing-anatomy lifetime stability.
Requested allocation volume, sampled RSS and OS peak RSS remain distinct metrics.

## Scaling and history boundaries

Fixed authenticated geometry/query counts isolate the worker-count factor. The
single-query and 370-query scenes establish different available parallel work;
they do not establish population asymptotics. Independent factorial experiments
for fixed geometry/increasing bodies, separated bodies, density and revision
growth are NOT RUN. Existing founder 1..16 and max_agents<=16 guards are unchanged.
No extent redesign, fake habitat enlargement or new sensory consumer was added.

[History compatibility proposal](HISTORY_COMPATIBILITY_V_PROPOSAL.md) distinguishes
serialization, laws, events, executable identity, converter identity and explicit
provenance transitions. It proposes immutable eras and strict verified transition
receipts, not arbitrary fingerprint bypass. No migration is implemented/adopted.
IV's opt-in v2 3600-tick evidence and its finite ceilings remain preserved under
their own executable/source identities; legacy v1 is unchanged.

## Candidate rulings and qualification

| Candidate | Ruling / evidence gates |
|---|---|
| Immutable authenticated query executor | IMPLEMENTED/VALIDATED as isolated research; exact count scaling and honest-red; no runtime admission path, therefore no Gate D–F runtime qualification |
| Unordered parallel body/static admission | REJECTED as proposed semantics: inspected target mutation invalidates the premise; no runtime implementation or timing claim |
| 32 workers as automatic performance policy | NOT SELECTED: no throughput gain over eight in this batch, lower occupancy and more duplicated memory; native policy unqualified |
| Whole-tick batch as claimed 5.35x engine speedup | REJECTED inference: captured descriptors span changed targets and dependent passes |
| Remove ridges/caps/hollow geometry or change contact limits | EXCLUDED from exact lane; no prototype or replacement adopted |
| Recovery witness reuse / incremental remesh cache | PROPOSED following attribution, not implemented or benchmarked improvement |
| Source-compatible history migration | PROPOSED only, no adoption/bypass |

Gate A attribution and isolated query feasibility are established within the
recorded limits. No runtime candidate qualifies for the final matched full matrix.
It was deliberately NOT RUN. No new native/GPU qualification, migrated-history
endurance, population-factorial result or whole-engine memory certificate exists.
All engine acceptance targets remain open; publication does not complete MORPH-005.

## Validation, evidence and reproduction

Focused collision library: 25 passed; new batch test compares 1/2/4 workers over
repeated real captures and rejects stale snapshot/revision and altered expected
hits. Honest-red removes the exact-hit comparison: the test fails (exit 101);
restoring it passes (exit 0). No test was disabled. Strict all-target/all-feature
workspace Clippy, formatting and nine Python integrity tests pass. Initial Rust
scope-lifetime compile failure and missing `/usr/bin/time` runner failure are
preserved; the final runner uses `wait4` and `/proc`, with exact CPU/RSS scope.

Clean-source broad validation is recorded separately in
PERFORMANCE_STABILIZATION_V_VALIDATION.json. The source checkpoint,
before/after status, command, result and archive receipt are the authority; this
paragraph does not substitute for that gate.

Final certificate: **PASS** at clean
`eea57b210be520ddb2d25a160dc9252e6a475bd0`, empty status before/after,
`cargo test --offline --workspace --all-targets --all-features`: 254 passed,
zero failed/ignored, 120.058 seconds. Subsequent publication changes are
documentation/receipts only. No hosted CI or performance certification is implied.
The two V archives retain 709 files / 320583884 uncompressed bytes with every
restored size/hash checked. The restored main-archive executable additionally
reproduced 1110 exact query results. Main archive SHA256
`acb2200ddee07e68568f641cf514b9415e036a6f22b44300ae5770716d02dedc`;
validation supplement SHA256
`2703bc48f2029bc60aa967a3267f973509c9edb753ad19a141106982a67870bb`.
All previous archives and working copies remain retained. Atlas navigation passes
45-note/197-link validation on its verified separate volume.

Working evidence: `/home/cognis/moebius-v-20261126`, a working copy. Authoritative
external evidence and restore instructions: PERFORMANCE_STABILIZATION_V_STORAGE.json
and the archive's REPRODUCE.md. Git retains this report, compact attribution/scaling
JSON, source, directive, provenance/validation/storage receipts. Raw worlds,
per-worker rounds, logs, executables, source archives and memory samples belong
on verified Mermogon, not in Git. Prior IV archives remain immutable.

To reproduce isolated queries, restore the evidence, verify its manifest and
captured build via `python/benchmark_integrity.py`, then execute:

```
collision_fixture.binary --batch RESTORED/inputs/endurance NEW_OUTPUT 8 30
```

For a rebuilt assay use `cargo build --offline --locked --release -p moebius-morph
--example collision_fixture --features collision-fixture`, capture source/binary
identity using the existing integrity helper, and keep that new identity with
its new results. Do not relabel the retained measurements as a rebuilt run.

Next authorized dependency: investigate an exact bounded query frontier with
pose/revision receipts and canonical fallback, or exact local wet-mesh allocation
reuse. Start from the dependency map and measured inputs, falsify unused/stale
result admission and lazy-error changes, then require matched every-tick state
and contacts before any ordinary release timing or full matrix. No change to
collision authority, history encoding or biological scope follows from this report.
