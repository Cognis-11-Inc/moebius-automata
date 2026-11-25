# SIMULATION-0001 coordinate-domain audit

2026-11-24 (America/Toronto). Scope: source audit and matched-scene measurements;
no coordinate redesign or collision-response edit during this audit. Entry main
`abdb664738a23c1aaf37c46810d26728f88c7d2f` plus preserved uncommitted continuation.
The operator subsequently authorized collision repair **after** this comparison.

## Containment, authority and reuse

Do not infer a simulated hull/grid from the 30,000 km × 3,000 km design dimensions.
No extent setting exists in `runtime.rs::WorldConfig`. `MAX_POSITION` is an
absolute-coordinate validity guard, not a physical boundary or allocated grid.
Keep REGION, relative positions, geometry, stock, controllers and collision rules
fixed. Do not change the production guard or fabricate a new runtime setting.

| Concern | Inspected path/symbol | Status / input -> output | Consumer / reuse ruling / missing seam |
|---|---|---|---|
| Ship authority | runtime.rs WorldConfig/ShipRuntime/Position | active saved instances -> canonical ticks | reuse exactly; no ship extent field |
| Habitat recorder | examples/habitat_probe.rs | supporting run -> traces/replay | existing records supply saved scenes; does not isolate allocation phases or compiled bounds |
| Historical benchmark | examples/morph_004_bench.rs | supporting fixed-rest instances -> timing | scope predates habitat collision; reuse methodology, not its different scene |
| New measurement harness | examples/coordinate_domain_bench.rs | supporting saved scene -> tick/allocation/query counters | delegates to ShipRuntime; executable-only allocator instrumentation; no new world owner |
| Presentation index | filament.rs StructuralIndex | occupied chunks -> sparse region bins | inspect/build/query independently; not the collision candidate index |
| Nursery | core/world.rs World::genesis, geom.rs Dims | separate lattice -> dense fields | not called by ship witness or morph crate; preserve separate laws |

Paths relative to crates/moebius-morph unless qualified. Inspected current
AGENTS/handoff, Cargo, MORPH-004/005 authority records, source and existing tests.
No exhaustive branch archaeology or external scientific study is claimed.

## Registered comparison

Since physical ship extent is not configurable, directly resizing the ship is
**NOT APPLICABLE**. Closest executable intervention: isolated source copies
changing only MAX_POSITION to ±1 km, ±30,000 km, and the current ±1,000,000 km.
The production tree keeps its original constant. These are axis-wise supported
coordinate bounds, not cylinder dimensions.

Use the prior seed-11 opening and trapped tick-720 snapshots. Rehome the single
remote founding Blood-Sac once at (100 m, id m, 0), recompute its world-owned
exposure, then use the same serialized local scene in every condition. All
inter-condition relative coordinates, geometry and rules remain identical.
The scene preparation and original/final hashes are recorded. This is an authored
local test scene, not an unmodified replay of the earlier whole snapshot.

Three bounds × two scenes × six blocks. Each block uses a different permutation
of the three bounds, sequential fresh processes, one warmup tick, then 10 opening
or 3 congested ticks. Record every digest, full final snapshot, query/build counts,
requested allocation counts/bytes/peak live bytes, process peak RSS, restore time
and tick time. Require byte-identical final snapshots and per-tick digests across
bounds. Additional observational measurements: occupied-index construction,
visibility at several radii, coordinate operations at increasing absolute offsets.
Distinguish coordinate *magnitude*, witness radius and supported extent.

CONSULTED: local implementation and saved evidence. IMPLEMENTED: measurement
harness only. VALIDATED/results: pending below. No runtime redesign authorized.


## Measured outcome

**VERIFIED:** no material cost from the supported coordinate envelope in these
matched local scenes. Six fresh-process repetitions per cell, all six order
permutations, release/offline Rust 1.99.1. All full final snapshots and every
measured tick digest match exactly across limits; collision query and allocation
counts/bytes are identical. Absolute differences below do not scale with extent.

| Supported bound on each axis | Opening mean ms/tick | Congested mean ms/tick |
|---|---:|---:|
| ±1,000 m | 29.502 ± 0.286 | 486.655 ± 1.998 |
| ±30,000,000 m | 29.065 ± 0.239 | 485.468 ± 2.568 |
| ±1,000,000,000 m | 29.264 ± 0.149 | 490.015 ± 3.996 |

± is sample SD of the six run means, not a confidence interval. The largest
versus smallest bound changes opening time by -0.81%, congested time by +0.69%.
Reducing the bound cannot explain or cure a roughly 0.49-second tick.

Opening ten measured ticks: 893 interval/query visits, 185,424 allocations,
174,034,735 requested allocation bytes in every condition. Congested three ticks:
3340 visits, 295,757 allocations, 229,648,195 requested bytes in every condition.
These bytes are cumulative allocation traffic, not resident memory. Peak live
requested heap is about 2.43 MB / 17.21 MB. Six-byte differences across bounds
match the varying output-path argument length retained by this executable.
Peak RSS ranges overlap: opening about 8.3–8.5 MiB; congested about 34.7–35.6 MiB.
System allocator metadata, renderer/GPU memory and long-run heap behavior are
outside this measurement. Allocator instrumentation adds equal overhead to all
conditions; these are controlled comparisons, not uninstrumented release timings.

## Source audit

- `runtime.rs::WorldConfig` stores definitions, ports, seed, medium, timestep and
  optional construction rules. It has no dimensions/extent/hull field. The
  confirmed cylinder dimensions are Atlas design metadata, not live configuration.
- `Position::{absolute,from_absolute,offset,relative}` operate on three fixed
  integer axes with checked i128 multiplication/addition, quotient/remainder by
  constant REGION, and constant-count MAX_POSITION comparisons. They neither
  allocate nor enumerate regions. Raising the range does not enlarge the values'
  fixed-width storage. Relative integer subtraction precedes f32 collision geometry.
- `ShipRuntime::tick` clones actual state and loops over actual commands, instances,
  nozzles, material records and events. Those allocations can be large, but no
  allocation size or loop bound is the total empty ship volume.
- `habitat.rs::prepare` scans existing food, agents and stored pipe nodes. Costs
  grow with occupied state; it does not traverse unoccupied ship coordinates.
- `collision.rs::resolve` builds shapes for existing instances, scans organism
  pairs, existing solids, actual pipe chunks, roots and free parcels. Its sparse
  cache is keyed by definitions/chunks, not a dense world grid. **There is no
  active global occupied-region collision index**: the existing region index is
  presentation-only. Local sphere tests reject candidates after scanning actual
  chunks. Growth in object/chunk counts can therefore hurt even in a tiny world.
- `filament.rs::StructuralIndex::build` inserts bins for actual chunks, at most
  eight bins per admitted chunk (each axis spans at most two bins). `visible`
  iterates existing bins, not all bins within the witness radius. A larger radius
  can reveal more occupied chunks and cause more rendering work; it is not a
  world-extent iteration.
- `witness.rs::WitnessSphere::inspect` scans actual instances and their parcels.
  Camera navigation and the 4 km witness radius affect presentation; the authority
  thread receives no camera/physical-extent input. No native renderer benchmark
  was repeated for this coordinate-envelope question.
- **Dense arrays do exist in the separate nursery**: `moebius-core/src/world.rs`
  `World::genesis` allocates nutrient/occupancy by `Dims::volume` and loops over
  width × height × depth. This is a real dimension-dependent cost for that
  separate lattice. The current ship witness uses moebius-morph::ShipRuntime;
  neither morph crate nor its collision path invokes nursery World::genesis.

Meaningful omissions: no full-hull/atmosphere simulation exists to time; no physical
extent setting was invented; no nursery-volume experiment, GPU draw comparison,
large-population extrapolation or alternate coordinate representation was tested.
The measured null concerns fixed occupied ship scenes and the coordinate guard.

No coordinate redesign or production extent change was made. The subsequent
collision-response repair has different source/behavior and must not be mixed
into these measurements. Raw snapshots, isolated sources, executables, orders,
per-tick/allocation records and hashes are retained in the task's external archive.
