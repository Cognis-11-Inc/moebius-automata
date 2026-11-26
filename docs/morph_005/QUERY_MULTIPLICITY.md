# Query multiplicity and ownership

IV correction, 2026-11-26 UTC: the III trace/replay doubling described below is
preserved as historical evidence. The current recorder extracts query-trace before
replay and labels its scope forward-only. Do not halve current IV traces. The
complete continuous path now includes separate linear_cast_ns and original-tree
node/reject/leaf counters. Authenticated fixtures and exact repeated-sample traces
are described in PERFORMANCE_STABILIZATION_IV.md. No unique intermediate sample
is treated as a redundant repeated cast.


MORPH-005 stabilization III. 2026-11-26. Feature-off timing identity is
`habitat_probe` sha256
`d317ccfb5093070ad6eb932216ec85abbaf3f0374ff7b3226629ed557cc80f85`,
schema 1, release, `features: {}`, rustc 1.99.1. Pair rows come from the
`query-trace` build, which is not a timing oracle. Its per-tick counters match
the feature-off `exact3` collision rows on every one of the 45 ticks in both
scenes.

## How the trace was read

`habitat_probe` writes `query-trace.json` after `replay.advance`. The trace
lives in a thread-local map, so the replay merges into the same labels and
doubles every counter. Raw files are exactly twice the profile totals
(eight 21138 versus 10569 distance searches; fresh 3954 versus 1977). Every
stored counter is even.

The probe's `"tick"` is the world tick after `tick()`. `set_pair` labels the
tick the step started from. Profile row N is label prefix `t(N-1)`.

Halving the trace and using that prefix matches `distance_queries`,
`distance_within`, `distance_clear`, `triangles_examined`, `distance_bounded`,
and `distance_exact` on all 45 rows of both scenes against
`/home/cognis/moebius-iii-20261126/exact3/*/collision-ticks.json`.
Mismatched ticks: 0. Odd fields: 0.

A distance search here is one `within_distance` sample inside `cast`. The
consumer of that sample is a boolean: the swept interval is still possible, or
it is certified clear. `distance_exact` counts leaves that reached the exact
distance kernel because a separating plane did not certify them. It is not a
count of terminal contact points. Terminal contact is `cached_world_contact`,
timed in `contact_ns`.

`note_cast` fingerprints the four endpoint poses of one cast. `sample_keys`
then mix that fingerprint with the sample limit and midpoint. Many sample keys
under `casts: 1` are distinct interval samples of one swept pair, not distinct
body poses and not repeated identical queries. The cast cache already returns
an identical endpoint cast. These spikes are not that case: cache hits on the
dominant pairs are 0.

## Eight-founder tick 32

Feature-off row, resume of `scaling-f8-s11`, world tick 32, step that began at
tick 31.

| Measure | Value |
|---|---:|
| Wall | 77.367 ms |
| Distance | 66.754 ms |
| Pipe mesh builds | 15, 6.450 ms |
| Body geometry | 0 ms |
| Distance searches | 873 |
| Within / clear | 442 / 431 |
| Triangles examined | 785660 |
| Bounded / exact leaves | 784005 / 1904 |
| Horizon indexed / candidates / rejects | 15 / 174 / 111 |
| Horizon radius bounds recomputed | 0 |

Archived split2, before this horizon, recorded the same 873 searches, wall
77.73 ms, distance 66.56 ms. The search count is unchanged.

36 labels.  The searches are not 873 geometric pairs.

| Pair (before tick 31) | Searches | Casts | Cache hits | Sample keys | Revisions | Triangles | Bounded | Exact |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| body 12 × pipe 6 chunk 0 | 402 | 1 | 0 | 402 | 1 | 442848 | 441855 | 993 |
| body 8 × pipe 2 chunk 0 | 92 | 1 | 0 | 92 | 1 | 77534 | 77310 | 224 |
| body 12 part 0 × pipe 6 chunk 0 | 87 | 1 | 0 | 87 | 1 | 71052 | 70771 | 281 |
| body 14 × pipe 8 chunk 0 | 83 | 1 | 0 | 83 | 1 | 143632 | 143282 | 350 |

The first pair is 46% of the searches and one geometry revision. One cast, 402
distinct interval samples, 18 distinct limits. The part row is the graft check
against the same pipe, a second cast with its own samples. Body-body pairs in
the same tick contribute searches and no triangles; they are compounds, not
the mesh spike.

Almost every admitted triangle is a bounded decision. The exact kernel is a
small leaf count inside the boolean. The time is the repeated mesh walk.

Horizon rejects 111 chunk visits and still leaves this local pair in the
candidate set. Indexing distant pipes cannot remove the spike.

## Fresh-colony tick 34

Feature-off row, resume of `matrix-fresh420`, world tick 34, step that began
at tick 33.

| Measure | Value |
|---|---:|
| Wall | 102.812 ms |
| Distance | 97.082 ms |
| Pipe mesh builds | 3, 1.073 ms |
| Body geometry | 0 ms |
| Distance searches | 524 |
| Within / clear | 264 / 260 |
| Triangles examined | 270174 |
| Bounded / exact leaves | 268378 / 1796 |
| Horizon indexed / candidates / rejects | 3 / 27 / 0 |
| Horizon radius bounds recomputed | 2 |

Archived split2 recorded the same 524 searches, wall 104.16 ms, distance
98.41 ms.

7 labels. One pair dominates.

| Pair (before tick 33) | Searches | Casts | Cache hits | Sample keys | Revisions | Triangles | Bounded | Exact |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| body 8 × pipe 2 chunk 1 | 513 | 1 | 0 | 513 | 1 | 269816 | 268021 | 1795 |
| body 8 × pipe 2 chunk 0 | 7 | 1 | 0 | 7 | 1 | 358 | 357 | 1 |

513 of 524 searches are one body against one pipe chunk, one revision, one
cast, 513 distinct interval samples, 17 limits, at most 692 triangles in a
single sample. Horizon rejects are 0 because the scene has three chunks and
this chunk is inside the reach. The operator figure "524 distance searches
against three pipe chunks" is the whole tick; the work is one of those chunks.

## Cold start and pipe remesh

Eight-founder tick 1: wall 207.659 ms, body geometry 206.818 ms, 12 distance
searches, 0 triangles. Fresh tick 1: wall 187.174 ms, body geometry 186.341 ms,
4 distance searches, 0 triangles. Cold start is compound construction of body
hulls. It is not a distance search.

Pipe remesh on the eight-founder spike tick is 6.450 ms beside 66.754 ms of
distance. On the fresh spike it is 1.073 ms beside 97.082 ms. Wet chunks still
rebuild their hollow trimesh when `pipe_key` ages, including ticks whose
horizon stamp did not change (eight tick 32 recomputed 0 radius bounds and
still built 15 meshes). That rebuild is real and smaller than the spike.

## What was checked and is not available

Identical endpoint casts are already cached. The spike pairs miss the cache
because the expensive work is inside one cast.

A parent Lipschitz bound that fails to clear an interval does not clear either
child: the child's farthest point is the parent endpoint. Propagating a failed
parent bound cannot prune this bisection. Clears already return. The within
half has to be split or the contact time can move.

Sample keys equal the search count on the dominant pairs, so those samples are
not repeated identical queries.

A cross-tick separation certificate would apply to a pair that stays outside
the reachable interval. These pairs are inside it: within and clear are nearly
equal, and the cast ends on a contact. Skipping them would be a missed
admissible contact. The part cast is a separate physical question (whether the
secreting outlet is the only overlap) and is not a duplicate of the parent cast.

No simplified collision mesh was substituted. The triangles above are the
authoritative hollow chunks.
