# Spatial locality audit

## IV correction and current boundary

2026-11-26 UTC. The III measurements below are retained as historical results.
The quick-stamp certificate described below had a same-tick middle-node edit hole.
IV fixes it using `FilamentMap` mutation revision receipts and full BLAKE3 changed-
filament stamps. Serialization remains a transparent BTreeMap. Identical unedited
clones share the receipt; writes detach it; later ticks need no global scan when
inputs are unchanged. An intentional stale-certificate test failed before the
repair and passes after it. Failed maintenance still clears the horizon.

Actual 1x/10x/100x configured habitat dimensions remain **NOT SUPPORTED**: neither
WorldConfig nor HabitatRules has an extent parameter. MAX_POSITION is a fixed
coordinate-validity bound, not an allocated world or boundary condition. The
translation experiments measure coordinate magnitude, not empty-world extent.
There is no evidence warranting a coordinate-system redesign. All region/local
position transformations are per active entity/query; no dense empty-space grid
was found in the inspected runtime/collision/filament paths. Whole-tick cost can
still scale with occupied assembly clones and local biological/physical work.

Current IV spatial rerun and exact-source results are recorded in
PERFORMANCE_STABILIZATION_IV.md and its immutable storage receipt when available.
No III translation is relabeled as a true larger-habitat experiment.


MORPH-005 stabilization III. 2026-11-26. The law under test:

**The cost of local physical interactions must not scale materially with
unreachable distant geometry or empty habitat volume.**

Status of the law for the distance kernel and for horizon bound maintenance:
**VALIDATED** on the measurements below. Status of whole-tick cost:
**PARTIAL**. A residual walk outside `resolve` still grows with total filament
nodes. Close local mesh bisection is independent of habitat size and remains
the spike.

## Archaeology

Collision authority is `collision::resolve` in
`crates/moebius-morph/src/collision.rs`. It is called from `ShipRuntime::tick`
on the pre-step assembly. The same cache serves five chunk walks, all through
`CollisionCache::interacting_chunks`:

| Walk | Symbol | Question |
|---|---|---|
| Movement | `resolve` | Swept body against bodies and statics |
| Placement | `clear_shape`, used by `spawn_pose` | Is a candidate pose empty |
| Secretion | `deposit_allowed` | May this deposit occupy its box |
| Emission | `emission_obstacle` | First solid in front of a nozzle |
| Roots | `admit_roots` | May a stalk advance |

`pipe_separated` remains the sample-sphere test for chunks the horizon keeps.
`within_distance` is the boolean interval test. `cast` bisects when either
body has angular velocity. Colony motion is angular, so the spikes take
bisection rather than `cast_shapes`.

`filament::StructuralIndex` is presentation-only. `visible` walks every stored
bin and then tests chunk AABBs. It is not serialized as world state. Collision
does not call it. It was not extended into a second physical index.

`WorldConfig` has no habitat-extent field. `MAX_POSITION` is a validity guard.
`resolve` does not read it. Empty volume is not allocated and is not iterated.
The earlier coordinate-domain audit already measured that guard at three
magnitudes and found no material cost. Production `MAX_POSITION` was not
changed.

Body geometry is a compound of convex hulls of the posed anatomy. Pipe chunks
are hollow trimeshes, `CHUNK_EDGES = 16`. Support indices are retained only for
convex hulls and compounds. The centroid support precheck and the extra
node-plane experiment stay removed.

## Horizon that was reused and narrowed

`CollisionCache` already owned pipe meshes and the cast memo. The horizon is a
field of that cache, not a new subsystem. It is not serialized, not restored as
world state, and not consulted by the Witness. `restore` builds a fresh cache.

Cells are 8 m (`PIPE_CELL = 8 * SCALE`). Each chunk sphere is padded by 0.5 mm
(`HORIZON_PAD_M`), wider than the 0.3 mm `pipe_separated` margin. A sphere that
would cover more than 64 cells on an axis, or more than 512 cells, is kept in
`always_chunks` and is never rejected. An unready horizon or an enormous query
returns every live chunk. Order stays filament id, then chunk index.

Cell membership is a prefilter. Overlapping padded spheres have overlapping
AABBs and therefore share a Euclidean cell, so a sphere-near chunk is not
dropped. `pipe_beyond_horizon` remains the test oracle that requires both the
cell miss and an absolute sphere miss.

Maintenance is per filament. `pipe_filament_stamp` covers id, creation, nozzle,
quantity, section, and every node id, deposit tick, and absolute position.
`sync_pipe_horizon` returns immediately when the quick stamp and
`confirmed_tick` already match this tick. Otherwise only filaments whose stamp
changed are reindexed. A younger radius bound is kept while a pipe is wet,
because `radii_at` does not grow with age (`drying_does_not_increase_the_chunk_radius_bound`).
On any update error the horizon is cleared and the next query sees every chunk.

Measured on the two 45-tick spike resumes, distance searches stayed 10569
(eight) and 1977 (fresh) after this change. Radius-bound recomputations fell
from 1691 to 225, and from 70 to 39. Final snapshots stayed byte-identical to
the contact-plane runs through 120 and 420 ticks (digests below).

## Certificate limit

The quick stamp does not see an in-place edit of a middle node. Production
deposit and chew change the stamp. A middle edit that does not is invisible
until the next tick if this tick was already confirmed. That behaviour is
covered by `interacting_chunks_match_the_per_chunk_horizon_and_omit_distant_pipes`.
It was not "fixed" by restamping every filament on every query. That would
restore the full-geometry walk this change removed. Uncertain or failed updates
fall back to every live chunk.

`casts` is cleared at the start of each `resolve` and capped at 8192. The
horizon keeps one sphere per live chunk and drops filaments that are gone.
`pipes` retains only keys still in the horizon. No empty-volume grid is stored.

## Scaling measurements

Executable: `spatial_scale` built with `--features spatial-scale` only.
Eight measured ticks after one warmup. Not the timing oracle. Raw rows:
`/home/cognis/moebius-iii-20261126/spatial/spatial-scale.json`.
Spliced filaments use organism 9000001, after `restore`, and do not enter the
material books. `habitat::validate` would reject them on a second restore, so
the splice stays inside the measured process.

### A. Empty habitat

There is no extent to multiply. The saved fresh scene, including the region-8
blood sac, spans 8002.5 m. The run translated that whole scene by 0, 10×, and
100× of the span (0 m, 80024 m, 800250 m) and recomputed exposure. Local
relative poses are unchanged. Production `MAX_POSITION` is unchanged.

| Scale | Mean tick | Distance searches | Triangles | Pipe bounds |
|---|---:|---:|---:|---:|
| 1× | 1.324 ms | 328 | 6827 | 13 |
| 10× | 1.324 ms | 328 | 6827 | 13 |
| 100× | 1.312 ms | 328 | 6827 | 13 |

Search counts match. At 100× the horizon's split between cell rejects and
sample-sphere candidates moves (rejects 23 → 37, candidates 79 → 65) because
cell alignment changes. The same chunks stay out of the distance kernel.
This is a coordinate-magnitude test of one scene, not a larger empty world.
The prior coordinate-domain audit remains the empty-extent result.

### B. Distant pipe geometry

0, 8, 32, and 96 extra chunks were spliced at +800 m. Each chunk is 17 nodes
of real hollow section.

| Extra chunks | Indexed | Horizon rejects | Bounds computed | Distance searches | Triangles | Mean tick | Resolve / tick | Horizon / tick |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0 | 2 | 23 | 13 | 328 | 6827 | 1.316 ms | 0.938 ms | 0.005 ms |
| 8 | 10 | 431 | 13 | 328 | 6827 | 1.446 ms | 0.985 ms | 0.011 ms |
| 32 | 34 | 1655 | 13 | 328 | 6827 | 1.438 ms | 0.955 ms | 0.023 ms |
| 96 | 98 | 4919 | 13 | 328 | 6827 | 1.660 ms | 0.982 ms | 0.059 ms |

Distance work does not change. Bound recomputation does not change: unchanged
filaments keep their stamp. Horizon time grows to 0.059 ms/tick at 1648 nodes.
Resolve stays near 0.98 ms. Mean tick rises by about 0.34 ms. That rise is
outside `resolve`. **INFERRED** cause: the tick clones the assembly and food
walks nodes. It was not given its own counter. RSS stays about 8.4 MiB.
Before per-filament stamps, the same 96 chunks recomputed on the order of a
thousand bounds per window. That maintenance leak is closed.

### C. Local density

32 distant chunks were held constant. 0, 4, 16, and 48 chunks were added on a
line starting 2 m from the organism.

Distance searches stay 328 and triangles stay 6827 at every local density.
The added pipes do enter the horizon (candidates 79 → 204 at 4 local chunks)
and they are meshed (pipe builds 14 → 30). Sample-sphere candidates rise from
16 to 59 and sphere rejects from 35 to 117. They do not reach `within_distance`.
Mean tick rises from 1.432 ms to about 2.85 ms at the first 4 local chunks and
then stays near 3.0 ms while indexed chunks grow from 38 to 82. The cost that
moved is nearby wet mesh construction, not the distance kernel.

This is not a distance-versus-local-density curve. One was not invented.
The eight-founder and fresh spikes remain the evidence that a pipe which is
actually inside the swept interval dominates the distance kernel.

### D. Temporal coherence

| Case | Mean tick | Searches | Triangles | Notes |
|---|---:|---:|---:|---|
| Grown contact, 4 setup ticks then measured | 2.037 ms | 305 | 9681 | Pipes that exist |
| Those pipes then shifted +800 m | 1.153 ms | 280 | 2016 | Same nodes, out of reach |
| One spliced chunk about 2 m ahead | 1.805 ms | 328 | 6827 | Searches unchanged |
| Unmodified fresh window | 1.177 ms | 328 | 6827 | Same scene as A at 1× |
| Eight-founder snapshot, first 8 ticks | 4.858 ms | 529 | 47281 | Not the tick-32 spike |

Shifting the pipes that the colony had already grown cuts triangle work from
9681 to 2016. That is isolation, not reuse. A fresh snapshot has no filaments,
so shifting pipes at tick 0 was a null experiment and was replaced by this
grown pair.

Cast-cache hits on the spike pairs are 0. The short windows do not show a
close pair whose samples can be reused across ticks. A certificate that is
only valid while the gap exceeds translation, rotation, deformation, pipe
edits, and a new neighbour would not cover the spike pairs: they are in
contact by the end of the cast.

## What was not added

No second spatial authority. No coarser stand-in for the hollow mesh. No
change to `MAX_POSITION`. No cross-tick certificate on close pairs. Sensory
sampling is specified in `SENSORY_ARCHITECTURE.md` and is not implemented.
The horizon is not a sensory organ.
