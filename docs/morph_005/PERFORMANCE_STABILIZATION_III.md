# MORPH-005 performance stabilization III

Spatial locality, interaction horizons, and the sensory contract.
2026-11-26. **PARTIAL.** Biological expansion stays suspended. MORPH-006 was
not started. No commit and no push: the timing gates are not met.

The cost that moved is unreachable pipe maintenance. The cost that did not
move is a close body against one hollow chunk, bisected for the whole tick.

## Identity

| Item | Value |
|---|---|
| HEAD | `0a6390950ecb75f5b866e17f77ce54b5d7115075` |
| Branch | `main`, dirty, no new commit |
| Timing binary | `target/release/examples/habitat_probe` |
| sha256 | `d317ccfb5093070ad6eb932216ec85abbaf3f0374ff7b3226629ed557cc80f85` |
| Build | schema 1, release, `opt-level = 3`, `features: {}` |
| rustc | 1.99.1 (`58cf8f9c3` 2026-10-13) |
| Tick budget | `TICK_NANOS = 16_666_667` (16.667 ms) |

`query-trace` and `spatial-scale` builds are measurement tools. Their timings
are not mixed into the table below.

After that binary was built, two non-physical edits landed. `Cargo.toml` sets
`required-features = ["spatial-scale"]` on the `spatial_scale` example, so a
feature-off workspace build does not compile it. `pipe_beyond_horizon`, the
test oracle production does not call, now enumerates the center components
instead of indexing `0..3`, which is what `cargo clippy --workspace --offline
--all-targets -- -D warnings` required. The measured binary was not rebuilt.
Its embedded `collision.rs` blake3 remains
`e67d2545def7b61a505aa9f1a5124c9d2e900d6f4480b13004367c0ab81279ce`.

Stabilization III measurements are archived separately and were restore-verified
on 2026-11-26T03:42:52Z. Receipt:
`PERFORMANCE_STABILIZATION_III_STORAGE.json`. Archive sha256
`db2ae3d3fe36b82e8f9afe228bff7d005de299881b4f7fa23b49c8287f576368`,
6463840 bytes, 104 members, volume UUID `D0A02EEBA02ED82C`, path
`.../2026-11-25/performance-stabilization-iii-v1/evidence.tar.gz`.
The II archive was not repacked.

Stabilization II evidence remains:

`/run/media/cognis/Mermogon/Moebius-Automata/experiments/SIMULATION-0001/2026-11-25/performance-stabilization-ii-v1/evidence.tar.gz`

sha256 `62e46ca7e8976fa0246fa9b628545abd6dd13ac7254815a67bf6f740db7ba8b0`,
40626097 bytes, volume UUID `D0A02EEBA02ED82C`. Receipt:
`PERFORMANCE_STABILIZATION_II_STORAGE.json`. The handoff lines that still say
the II evidence is only on tmpfs describe that earlier moment.

The separating-plane path and the reverted centroid support precheck are
unchanged. The negative support-axis result was not rerun.

## What the code does now

`CollisionCache` keeps a pipe horizon and refreshes it per filament. The five
physical chunk walks (`resolve`, `clear_shape`, `deposit_allowed`,
`emission_obstacle`, `admit_roots`) ask `interacting_chunks` before they touch
a mesh. Distant chunks are omitted. An unready index, a huge query, or a
failed update returns every live chunk. `pipe_separated` and the hollow
trimesh stay authoritative for whatever remains. Details, the certificate
limit, and the scaling tables are in `SPATIAL_LOCALITY_AUDIT.md`. Pair
identity for the two spikes is in `QUERY_MULTIPLICITY.md`. The sensory
contract is `SENSORY_ARCHITECTURE.md`. It adds no organs and no runtime path.

## Spike reconciliation

Operator figures, archived split2, and this binary agree on the search counts.

| Scene | Tick | Archived wall / distance | This binary wall / distance | Searches |
|---|---:|---|---|---:|
| Eight founders | 32 | 77.73 / 66.56 ms | 77.367 / 66.754 ms | 873 |
| Fresh colony | 34 | 104.16 / 98.41 ms | 102.812 / 97.082 ms | 524 |

Eight tick 32 examines 785660 triangles (784005 bounded, 1904 exact) across
36 labels. Body 12 against pipe 6 chunk 0 is 402 of the 873 searches: one
cast, one geometry revision, 402 distinct interval samples, 0 cache hits.
Fresh tick 34 examines 270174 triangles. Body 8 against pipe 2 chunk 1 is 513
of the 524 searches, and the horizon rejects nothing because only three chunks
exist. Pipe remesh on those ticks is 6.450 ms and 1.073 ms. Cold tick 1 is
body-compound construction (eight 206.818 ms, fresh 186.341 ms) and almost no
distance work.

Those repeated samples are not redundant calls of one query. The cast cache
already covers identical endpoints. A failed parent separation bound cannot
certify either child. The pairs are inside contact range, so a standing
separation certificate would be false.

## Locality result

On the spatial-scale window, translating the scene by 10× and 100× of its
8002.5 m span leaves distance searches at 328 and triangles at 6827. Adding
96 hollow chunks at +800 m leaves the same 328 searches and the same 13 radius
bounds. Horizon time at that load is 0.059 ms/tick. Resolve stays under 1 ms.
Mean tick rises about 0.34 ms outside resolve as node count goes from 16 to
1648. **INFERRED**, not separately counted: state clone and the food walk.

Local pipes at +2 m increase mesh builds and sample-sphere tests. They do not
increase distance searches. The distance kernel grows when a chunk is inside
the swept interval, which is the spike, not the distant-geometry series.

Per-filament stamps cut radius-bound recomputes on the real 45-tick resumes
from 1691 to 225 (eight) and from 70 to 39 (fresh) without changing search
counts or final bytes.

## Exactness

Feature-off resumes match the contact-plane finals:

| Run | Ticks | Digest | Prior run with the same digest |
|---|---:|---|---|
| `scaling-f8-s11` | 120 | `74c7be43784e5e7deed4152d9e8172cdb622ba44b93bacc75efc85089b6d21fa` | `contactplane-population-8-contact` |
| `matrix-fresh420` | 420 | `b6ec37d812a7242661edd1a3e6c30e85a539155798ce48b788e695cace09cddf` | `contactplane-fresh420-contact` |
| Founders 1, 2, 4, 8, 16 | 120 | equal to each `matrix2-population-N-candidate` | same scenes |

The first 45 tick-rows of the new eight and fresh resumes match the `exact3`
rows byte for byte. Every matrix row below reports `replay_equal` and material
gap 0. Thirty-two founders were not run. `habitat_colony` rejects a founding
population outside 1..=16. That cap was not raised.

## Matched matrix

Forward tick times only. Replay is extra wall time and is not added into the
percentiles. Cold is the first tick. Steady mean drops that tick. Nearest-rank
percentiles, the probe's own definition. Binary as above. Commands and raw
reports: `/home/cognis/moebius-iii-20261126/matrix-iii/`.

| Case | Ticks | Mean | Median | P95 | P99 | Max | Cold | Steady mean | Agents | Nodes |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Eight-founder resume | 120 | 19.039 | 13.883 | 45.628 | 79.023 | 207.330 | 207.330 | 17.457 | 8 | 176 |
| Fresh resume | 420 | 16.313 | 10.979 | 47.639 | 64.468 | 187.696 | 187.696 | 15.904 | 3 | 224 |
| 1 founder, seed 11, abundance 4000 | 120 | 0.452 | 0.276 | 1.145 | 4.414 | 6.861 | 6.861 | 0.398 | 1 | 8 |
| 2 founders | 120 | 7.321 | 2.525 | 26.434 | 48.293 | 103.936 | 6.698 | 7.326 | 2 | 32 |
| 4 founders | 120 | 8.445 | 7.184 | 18.326 | 22.128 | 22.697 | 9.601 | 8.436 | 4 | 80 |
| 8 founders, fresh | 120 | 17.286 | 13.545 | 43.073 | 61.890 | 81.074 | 7.285 | 17.370 | 8 | 176 |
| 16 founders | 120 | 49.595 | 48.305 | 98.220 | 145.565 | 147.835 | 8.895 | 49.937 | 16 | 368 |
| Congested save, 30 ticks | 30 | 58.149 | 49.657 | 77.137 | 295.854 | 295.854 | 295.854 | 49.952 | 3 | 292 |
| Native, 2 founders, seed 11, abundance 4000 | 720 | 27.060 | 25.135 | 64.610 | 82.854 | 112.504 | 7.013 | 27.088 | 3 | 521 |
| Same protocol, 3600 requested, 180 s forward budget | 1863 | 61.864 | 59.073 | 132.677 | 166.466 | 353.287 | 7.324 | 61.893 | 8 | 1678 |

The endurance row stopped with `HistoryBudget: no authoritative ticks discarded`.
`construction.max_events` is 65536. The saved world is tick 1863 and holds
65505 events. The next tick's new events would have carried the log past the
cap, so that tick was refused.
Forward time to that refusal was 115.3 s, inside the 180 s budget. The budget
was not the stopper. Replay of the 1863 committed ticks matches, and the
material gap is 0. The cap was not raised. Six births, eight agents, 1678 nodes.

Targets, unchanged:

| Target | Result on this matrix |
|---|---|
| Mean ≤ 5 ms | Met by the single founder only |
| Preferred mean ≤ 2 ms | Met by the single founder only |
| P99 ≤ 8 ms | Met by the single founder only (4.414 ms) |
| Tick budget 16.667 ms on the mean | Missed by the eight-founder resume (19.0 ms), fresh eight founders (17.3 ms), sixteen founders (49.6 ms), the congested save (58.1 ms), and native 720 (27.1 ms). Fresh 420 is under the budget on the mean (16.3 ms) and over it on p99 (64.5 ms). Two and four founders are under on the mean and over on p99 |
| 3600 ticks inside 180 s | Not met. 1863 ticks were committed in 115.3 s, then `HistoryBudget` refused the next tick. Mean 61.9 ms, p99 166.5 ms |

One founder is a quiet scene: 8 nodes, no second body. It does not certify
the colony. Sixteen founders average 49.6 ms. The congested save averages
50.0 ms after a 295.9 ms cold rebuild and holds 292 nodes with 3 agents.
Native 720 averages 27.1 ms with 521 nodes. Population and accumulated local
structure both move the mean. Empty volume and distant chunks do not.

## Gates

| Gate | Result |
|---|---|
| Authoritative state versus the contact-plane finals | Held for the 120- and 420-tick resumes and the five population rows |
| Replay equality and material gap 0 | Held on every finished matrix row |
| Contact equality | Held where the full snapshot digest matches. Digests cover the saved contacts |
| No missed admissible collision on the tested scenes | Held by those digests. The horizon tests also keep a distant chunk out and an enormous query complete |
| Stale horizon after a failed update | Cleared. Middle-node same-tick hole remains, and is tested |
| Cache bounds | Cast memo capped at 8192 and cleared each resolve. Horizon stores live chunks only |
| Witness or frame cadence | Collision does not read the Witness. `StructuralIndex` stays presentation-only |
| Benchmark identity | sha256 above, features empty |
| Mean ≤ 5 ms, p99 ≤ 8 ms | Not met |
| Workspace tests | `cargo test --workspace --offline -- --test-threads=8`: 239 passed, 0 failed, 0 ignored, 32 suites, debug, after the Clippy edit |
| Clippy | `cargo clippy --workspace --offline --all-targets -- -D warnings` passed. The spatial-scale example was linted separately with that feature and passed. `rustfmt --edition 2024 --check` on `collision.rs` passed |

Allocation counters were not remeasured. The timing binary has no
allocation-attribution feature. Prior instrumented figures stay in the II
report and are not this executable's counts.

## Bottleneck

The remaining causal cost is local.

On the short spike scenes, one close hollow chunk walked once per bisection
sample is the fresh tick and most of the eight-founder tick. A spatial index
cannot drop that chunk without dropping a real contact. Coarsening the
bisection would change the contact time and was not done. The same shape of
spike is still the worst endurance tick: tick 1793 is 353.3 ms, of which
215.5 ms is distance across 637 searches and 638448 triangles.

On the 1863-tick run the mean is a different mix. Steady distance is 3.6 ms
per tick. Pipe meshes are 6.0 ms. Terminal contact is 7.4 ms. The cast timer
averages 38.1 ms and resolve averages 57.0 ms. Horizon maintenance is 0.10 ms.
140 ticks cost more than 100 ms with under 1 ms of distance time. Those ticks
record several hundred linear casts and almost no distance searches. The
linear path is `DefaultQueryDispatcher.cast_shapes`, taken when both angular
velocities are zero, and it is not included in `distance_ns`. **INFERRED:**
that call is the bulk of those timers. The chunks involved are ones the
horizon kept. At the end of the run the horizon indexes 114 chunks. Over the
whole run it rejected 2.79 million candidate visits and kept 3.36 million.

Wet pipes still rebuild their trimesh when the age key changes, including
ticks that recompute no horizon bound.

A small whole-tick rise outside `resolve` remains when distant node count
grows and distance work does not. It is about 0.34 ms from 16 nodes to 1648
in experiment B. **INFERRED:** state clone and the food walk. It is not the
endurance mean.

Population and the pipes founders leave behind move the mean: sixteen founders
49.6 ms, congested steady 50.0 ms, native 720 27.1 ms, endurance steady 61.9 ms.
Empty volume and distant chunks do not. The history cap then stops the run
before 3600 ticks. That cap is the existing refusal to discard events. It was
not introduced by the horizon, and it was not relaxed.

## Non-claims

Lag is not resolved. The single-founder timing is not a colony result.
Experiment A did not create a larger habitat. Experiment C did not produce a
distance-versus-density slope. No sensory organ exists. No collective
behaviour was scripted. No organism is ranked by neural size. The 0.34 ms
rise outside resolve is an inference, as is the attribution of the
low-distance endurance ticks to `cast_shapes`. The debug suite is not a
clean-commit release certification. The tree stays dirty, and nothing was
committed.
