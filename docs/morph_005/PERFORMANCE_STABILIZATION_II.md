# Performance stabilization II — active, not certified

2026-11-25. Operator authorizes autonomous investigation, implementation, validation,
archival, commit and push. Biology remains suspended. Baseline0a63909 is clean;
its crates/Cargo sources match clean-tested cdc3acd exactly.228 tests are a prior
receipt, not certification of the present work. Original directive is retained
beside this record. Targets include mean<=5ms/preferred2ms, P99<=8ms,95% less
congested allocation traffic,3600ticks within180s. No fidelity or outcome changes.

Containment: no new physics representation, learning, biology, skipped collisions,
search limits or erased history. No optimization claim before source-identified
matched evidence. New code below is uncommitted and under investigation.

| Concern | Existing path / status | Reuse ruling / missing seam |
|---|---|---|
| Recorder orchestration | python/simulation_0001_experiments.py, active supporting observer | adapt it to isolated owned builds; shared target copying was unsafe |
| Old one-off matrix scripts | prior immutable performance archive, historical evidence | retain; not a second canonical driver |
| Build identity | existing example source hashes, supporting | extend with build.rs source/dependency/compiler/features identity and --build-identity endpoint |
| Allocation recorder | coordinate_domain_bench.rs GlobalAlloc, executable-only observer | extend realloc/dealloc tracking; do not turn instrumentation into game authority |
| Call-path counters | CollisionProfile, existing partial timing | insufficient for allocation attribution; feature-gated profiling.rs adds scoped observer counters only |
| Contact workspace | collision.rs recovery -> Parry contact, canonical query path | sampled allocation stacks identify fresh EPA storage; reuse only if exact contact/state equivalence holds |

Read AGENTS/HANDOFF, MORPH005 authority, current report/storage/validation,
source/call sites and primary cached Parry0.27 contact/GJK/EPA implementation.
No exhaustive branch audit or external scientific publication is claimed.

Initial attribution: three congested measured ticks attribute about184MB requested
allocation bytes to recovery, about58MB to other resolve work, and14.7MB to geometry
construction. These are instrumentation observations, not ordinary performance
results.47 recovery attempts dominate exclusive scoped time. Stack samples reach
EPA::closest_points through support-map contact and nested compound/trimesh
traversal. Samples include warmup and measurement; they are a separate diagnostic
pass, not quantitative allocation totals. The final snapshot matches the prior
canonical snapshot byte-for-byte.

The old wrapper lacked realloc override, potentially forcing copies. Revised
instrumentation delegates to System::realloc and separately counts allocation,
reallocation and deallocation requests. Do not compare revised totals directly
to old instrumentation as an optimization gain. Per-scope frees are attributed
to execution location, not ownership/lifetime. Uncovered/zero scope counts are
not evidence of zero work. Complete attribution and worst-tick investigations
remain pending.

Candidate: persistent EPA capacity in CollisionCache for recovery contacts.
Original relative transforms, GJK arithmetic, composite traversal and strict
contact tie rule are retained. EPA resets logical storage before every solve;
its iteration limit bounds workspace growth. Focused tests alone do not establish
state/contact equivalence. No accepted speed or95% allocation claim yet.

Isolated quantitative screens (3 measured congested ticks after identical warmup):

| Candidate | Requested bytes | Instrumented mean ms | Exact final and tick digests |
|---|---:|---:|---|
| Same observer baseline |263041278|137.708|reference|
| Recovery EPA reuse only |79252446|134.401|PASS|
| All contact EPA reuse |23393966|132.347|PASS|
| Identical-query memo |23520342|82.016|PASS|
| Hull support tree, first implementation |23505870|67.647|PASS|

These short instrumentation samples are screening results, not ordinary timings,
full acceptance or95% allocation success. Frozen build/run receipts and binaries
are in the current raw workspace. Later support-count observer adds overhead and
is distinct. Its baseline counted329292/227592/311038 support queries and
97147413/65581828/82378522 potential convex vertex scans in ticks722–724.
GJK contact leaf calls26560/16858/23445 and EPA calls7726/5641/8275. These are
calls, not inaccessible internal solver loop iteration counts.

Hull support tree indexes existing immutable convex points, preserving original
f32 dot arithmetic and earliest-index tie ordering. Outward f64 bounds include
f32 roundoff allowance; unresolved nodes are searched. No hull/pipe geometry or
contact ordering changes. Persistent indices retain owned original hulls and
compounds, bounded8192entries; overflow runs the original linear support scan.
Exact-query memo is cleared per resolve, bounded8192entries and retains original
shapes while answers survive. Overflow computes the query, never skips it.

Rejected lifetime candidate: initial support-owner registry also retained changing
unindexed meshes. Ordinary long-run process reached537768KiB RSS before deliberate
SIGTERM143; incomplete outputs retained, no timing acceptance. Corrected registry
retains only convex hulls/compounds. Mesh/capsule nonretention regression passes.
Matrix2 uses separately captured ordinary-index-fixed. Initial ordinary420 result
shows almost no improvement (~33.39ms); unchanged spikes remain. This falsifies
any inference that the congested screen alone resolves colony performance.

Unchanged oriented triangle normals must remain: overlaps() calls contains_point
for volumetric containment. Removing ORIENTED to avoid construction costs would
remove a live collision rule; rejected without implementation.

## Ordinary separating planes — measured 2026-11-25, targets unmet

Authoritative tick is `TICK_NANOS = 16_666_667` (16.667 ms). rustc 1.99.1.
Release, offline, locked, isolated `habitat_probe` builds. Exact means initial,
final, and tick-row bytes match, and each run reports replay equality with
material gap 0. Raw outputs are on tmpfs at
`/tmp/moebius-stabilization-ii-20261125T232446Z/` and are not archived.

Indexed support without face planes left the colony spikes in place. Matched
ordinary runs, index-fixed versus distance-plane candidate:

| Case | Ticks | Index mean / p99 / max ms | Plane mean / p99 / max ms | Exact |
|---|---:|---|---|---|
| Eight founders `scaling-f8-s11` | 120 | 77.715 / 758.852 / 1016.212 | 20.924 / 83.541 / 188.101 | PASS |
| Fresh colony `matrix-fresh420` | 420 | 32.976 / 284.342 / 401.823 | 18.197 / 70.591 / 189.168 | PASS |

Eight-founder distance time fell from 62.272 ms/tick to 5.553 ms/tick. Pipe
remesh stayed about 5.2 ms/tick. Query counts did not change.

A second isolated pair added the same plane test to contact GJK, using
prediction as the limit and a 0.001 roundoff margin. Uncertain pairs still run
GJK/EPA. Bit-identical contact fixtures passed, then:

| Case | Distance-plane mean / p50 / p95 / p99 / max | Contact-plane mean / p50 / p95 / p99 / max | Exact |
|---|---|---|---|
| Eight founders, 120 | 21.149 / 15.843 / 56.898 / 84.984 / 188.110 | 19.399 / 14.714 / 46.346 / 76.638 / 191.076 | PASS |
| Fresh colony, 420 | 18.318 / 11.821 / 52.235 / 71.480 / 193.740 | 16.463 / 11.110 / 47.495 / 64.578 / 187.695 | PASS |

Contact time fell from 2.639 ms/tick to 0.754 ms/tick on eight founders.
Steady ticks after the cold first tick average 17.955 ms (eight founders) and
16.055 ms (fresh colony). Worst later ticks are 76.638 ms at eight-founder
tick 32 and 102.072 ms at fresh tick 34. The first tick is about 191 ms and
188 ms.

Instrumented 39-tick eight-founder screens, not ordinary timings: distance
profile summed 5832.236 ms (max 813.989 ms); active face planes summed
1066.032 ms (max 87.037 ms) with the same tick digests. A node-plane variant
summed 1389.794 ms and was not kept.

Library tests: `cargo test -p moebius-morph --offline --lib -- --test-threads=8`
— 21 passed, debug profile, 21.92 s. This does not certify the workspace suite.

Still open: mean <= 5 ms, p99 <= 8 ms, native 60 FPS, 95% congested allocation
reduction on an ordinary rerun, and 3600 ticks within 180 s. Population 1/2/4/16,
congested, native 720, and long 3600 have no contact-plane pair yet. Biological
expansion stays suspended.

## Ordinary spike composition — 2026-11-25

`habitat_probe` now writes `collision-ticks.json`, one row per tick, from the
existing collision counters. Counter-only builds kept the 45-tick final
snapshots byte-identical. Release, offline, locked.

Eight founders, tick 32: wall 77.73 ms, resolve 76.80 ms, distance 66.56 ms.
873 distance searches, 442 inside the Lipschitz limit and 431 certified clear.
Interval AABB rejected 3. Mesh-BVH interval rejected 9. Linear casts 0.
Pipe builds 15 cost 6.55 ms. The cold first tick is body geometry: wall
187.01 ms, body build 188.03 ms, distance 0.03 ms.

Fresh colony, tick 34: wall 104.16 ms, distance 98.41 ms, 524 searches
(264 within, 260 clear), 3 pipe builds, 2 linear casts.

A per-triangle support along the hull-center to triangle-centroid direction
preserved both final snapshots and made the spikes slower: eight-founder tick
32 distance 66.56 -> 147.51 ms, fresh tick 34 distance 98.41 -> 140.40 ms.
That check was removed. The repeated mesh-distance search remains the spike.
