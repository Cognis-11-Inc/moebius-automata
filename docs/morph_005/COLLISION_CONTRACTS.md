# Collision contracts — 2026-11-24

2026-11-25: [performance stabilization](PERFORMANCE_STABILIZATION.md) is the
current phase. Bounded distance predicates retain the hollow geometry and tested
state/contact evidence. Congested-save and population spikes remain unacceptable;
no lag-resolution claim. Earlier query receipts below describe their own versions.

Current 2026-11-25 continuation: **PARTIAL, repairs implemented; final checks pending**.
See [current repair status](SIMULATION_0001_REPAIR_STATUS.md) for response changes,
linear/rotational query paths, wet-cache correction and retained failed experiments.
The following receipts describe the preceding investigation.
See [the causal diagnosis](SIMULATION_0001_COLLISION_DIAGNOSIS.md) and the
[root results](../../SIMULATION_0001_PLAYABLE_COLONY_RESULTS.md). Historical
receipts below do not certify absence of trapping or current-source performance.

IN PROGRESS. Operator explicitly expanded the habitat slice to collision and
three separate contracts. This supersedes the earlier no-collision scope;
transport, fluid pressure, elastic mechanics and learned contact policies remain
outside this slice. Canonical ShipRuntime alone admits physical state changes.

1. Detection: physical anatomy (including pouches/appendages), food and hollow
   pipe walls, with swept translation/rotation queries. Numerical region-local
   geometry, not Witness visibility, supplies the candidates. A pipe's bore is
   empty; membranes seal only declared sealed ends. Explicit static obstacles
   can represent future physical walls; no ship hull is inferred from darkness.
2. Response: bounded kinematic obstruction/avoidance, legal oral feeding and
   declared structural/source attachment. No fake force, mass or elasticity
   claim. Initial/spawn placement must be checked, not passed through solids.
3. Biological consequences: typed contact observations, including partner,
   location, normal and response, available from immutable snapshots. Authored
   avoidance may use them; recording contact does not establish learning.

Reuse audit: no active ship collision implementation found in runtime.rs,
filament.rs, anatomy/compiler or tests. Core's periodic neural body explicitly
calls complete capsule collision a prerequisite and is a separate runtime.
Parry3d0.27.0 is locally cached (Apache-2.0); its primary source math,
DefaultQueryDispatcher, cast_shapes/cast_shapes_nonlinear, TriMesh flags and
PointQuery interfaces were inspected. Pin exact dependency and enable enhanced
Determinism; preserve existing dependency versions. This is an adopted query
library, not another world/physics authority. No external paper was consulted.

The first contact pass uses rigid numerical anatomy, including every generated
part, and a conservative full-pouch/full-grown-sac envelope. It does not claim
articulated/self-collision physics. One `CompiledAnatomy` generator supplies
both samples. The cached hit volume is the tight pose: bend 0.12, phase 0,
mouth at the aperture, flagella extended. The witness pose is bend 0.72, the
traveling construction phase, flagella extended, and the aperture plus 0.25
while eating, capped at 1. The same rigid body pose places both. Propulsion is
that rigid step. The writhe is authored presentation, not learned control.
`presentation_slack` measures presentation vertices outside the same-index
convex hull across eight phases (ticks 0, 20, … 140) and both mouth states,
then adds 3 cm. That clearance expands solid cuboids only. Pipes, prey,
parcels, and other builders stay on the tight hull. Measured on rustc 1.99.1,
2026-11-24: bare seed 77 is 0.11743747 m; the showcase-plus-pouch probe body
is 0.26036128401756287 m. Blood-Sacs are prey, and so are their substrate grafts: contact is recorded
and does not cancel the builder's step. The graft still holds the sac.
Snailpipe contact deflects a builder along the surface instead of cancelling
its step. Solids and other bodies still block. A gulp suppresses secretion
for that tick and does not park locomotion. Feeding is the first drive,
reproduction the second, and snailpipe is the trail of that motion.


## Witness authority and executable checks

The Witness sphere has no input to `ShipRuntime::tick`, `collision::resolve`,
material admission or habitat policy. Canonical geometry is built independently
of visibility. `tests/collision.rs::witness_visibility_and_reload_cannot_change_collision_or_biology`
compares exact snapshots every tick for unobserved, near-sphere/cache and distant
zero-radius-sphere runs, including an offscreen barrier response and mid-run
serialization/reload. Existing runtime/filament observation-neutrality and worker
no-listener checks remain separate regression coverage. Mesh budgets and visibility
control presentation work only; numerical ticks must not be discarded.

The world-handler honest-red bypasses the actual `collision::resolve` invocation
inside `ShipRuntime::tick`; the presented body then crosses the barrier and
`canonical_world_blocks_motion_and_exposes_biological_contacts` fails with
`presented anatomy penetrated the barrier`. Repeated on this restored tree
2026-11-24: fault exit 101, then the same test passed inside collision 7/7
after `resolve` was put back. An earlier fault run only failed missing contact
observations and was insufficient evidence for penetration detection; it is
retained separately.
No renderer collision or render-side state correction is used.

## Query implementation and current limits

Parry's nonlinear cast reported a hit for a stationary, separated mesh/box case.
That failed diagnostic is retained. The current swept detector instead subdivides
time intervals using static Parry distance and a conservative relative linear plus
angular-speed separation bound. A 0.1mm skin, 1e-5 interval and 512-query budget
refuse unresolved motion. These are authored numerical tolerances, not a force law.
Bodies use a union of conservative convex envelopes for generated anatomical
parts; numerical hull failure falls back to the component's bounding box.
This includes appendages/full pouches and conservative full-grown food envelopes,
but can obstruct empty concavities, including the oral cavity. Pipes retain the
canonical hollow triangle surfaces; they are not replaced by solid capsules.
Root stalks have canonical capsule envelopes with an explicit substrate graft.

Contact-driven avoidance can translate away while holding body orientation;
it must not reverse heading instantly or continue steering into a known obstacle.
Source nozzle grafts do not exempt other anatomical components. Secretion and
full candidate pipe binding are checked before material admission/binding.
Failed births/food placement have no material debit. Unsupported force fields,
instance drift and ballistic droplets are rejected in collision mode; free droplets
are stationary physical obstacles. This first pass does not establish articulated
self-collision, contact learning, sliding/friction, pressure or elastic mechanics.
Historical modes without habitat retain their original rules.

Measured on rustc 1.99.1, this machine, current dirty source, 2026-11-24.
The scheduled tick is 16.7 ms. Headless throughput and the native window were
measured separately. They are different worlds: the probe is showcase seed 77;
the native receipt is bare seed 20261123 plus a pouch.

Headless `habitat_probe` 420 ticks, `/tmp/moebius-habitat-throughput-20261124`:
PASS, replay equal, material gap 0, digest
`7b68517ba0f82ed979469ef059db5f151243eb6999fe8d37f2e58d69940c2afe`.
Elapsed 21.138 s. Simulation time 10.456 s, mean tick 24.896 ms.
Mean resolve 19.600 ms (`resolve_ns` 8231860968). Parry queries are 15645
and 7.940 s, about 96% of resolve. Body hull builds 2, hits 10169, geometry
6.057 ms total. Pipe builds 623, hits 1837, geometry 0.259 s, about 3% of
resolve. One agent moved on all 420 ticks (19.65 m), 42 meal ticks, max stock
171, 0 feeding or stationary emissions, 58 nodes, 0 junctions, 14 Blood-Sacs,
0 rooted. An older 8.14 s / 24-node probe (digest
`a2568647e4d693c86c4ef319e0631332e864af317d830b5f50bc066e30251c89`) predates
these behavior edits and is not the current cost. The receipt above also
predates the cache-key narrowing and the sample-sphere gate. The same seed
measured again, with those changes, is the comparison after the native receipt.

Native receipt `/tmp/moebius-habitat-native-v7`, release witness rebuilt
2026-11-24 19:13:11 -0500, `--first-simulation --ticks 420`: PASS, headless
replay equal, digest
`76309b40cee405ef51e80107c360737a2803513d2ad4ef753953bf90d5333fc5`
(same world as native-v5 and native-v6; the camera change did not enter the
digest). 1641 frames, log not capped, tick 420/420, elapsed 27.354 s.
Mean frame interval 16.660 ms. Mean draw CPU 0.921 ms, body mesh 0.365 ms,
pipe mesh 0.0025 ms, gore 0.00019 ms, camera 0.0063 ms. Body mesh rebuilds
434 and reuses 1226: an unchanged stamp skips the rebuild. Rendering is not
the lag. The window holds about 60 Hz while the simulation tick does not.
Fed 55 events, quantity 212; Emitted 108; Deposited 108; AssemblyRejected 312,
all `NoEligibleMaterial`; Contact 1253. Condensate gap 0. Pouch 128/1024 from
an initial 24, free 0, bound 108, filaments 5, nodes 108, Blood-Sacs 20,
condensate 3115, rooted sacs 6, agents 1, junctions 0. Frame 120 (tick 95)
shows feeding with secretion closed. Frame 280 (tick 176) shows trail laying.
`final_habitat.png` (tick 420) keeps organism 1 in Orbit with follow on, pipes
and Blood-Sacs still in the shot. Native-v3 and native-v4 aborted early.
Native-v6 completed the same world, but its closing frame lost the body
because the construction camera script still ran. Age remains in the pipe
cache key. Pipe geometry is not the budget. Pipe-age quantization was not
applied. The cast subdivision was not rewritten.

### Same-seed filter comparison — 2026-11-25

Showcase seed 77, 420 ticks, rustc 1.99.1, offline, this machine, dirty tree.
Contract digest
`7b68517ba0f82ed979469ef059db5f151243eb6999fe8d37f2e58d69940c2afe`,
path 19.64609043675258 m, 420 movements, 42 meal ticks, 58 nodes, 14
Blood-Sacs, 1 agent. Percentiles are nearest-rank, index = ceil((n-1)*p).
Times below are rounded to 0.001 ms; the JSON reports hold the full figures.
The hollow trimesh stays the contact surface. An exterior primitive was not
used. Collision counters stay out of the world digest.

The kept cache key is (filament id, chunk, before-node id or 0, after-node
id or 0, (span << 16) | age.min(192)). Age is the newest node in the chunk.
`nodes.len()` is not in the key. The wet-to-dry radius change is real
collision geometry, so age stays. The witness age/24 stamp is presentation
only and is not copied into this key.

`/tmp/moebius-perf-a-noreject` fetched every chunk (pipe rejects 0, pipe hits
1837, pipe builds 623, queries 15645) and matched the contract digest, with
replay equal and material gap 0. Mean tick 24.844 ms, p50 8.313, p95 87.053,
p99 115.607, max 578.249. VmHWM 10928 kB before replay, 13576 kB after. The
bisect that forced every fetch is not in the current source. That directory
is the receipt.

The centerline chord test is not a cast gate. `/tmp/moebius-perf-a-full`
digest `8ca37da92b57e11ff4b2f0846788fc338140965ff75aeab6e21e05107542dc43`,
pipe builds 612, queries 15416, pipe hits 625, pipe candidates 1244, pipe
rejects 1243, mean tick 24.687 ms. The chord test is gone from the source.

A sphere used as a pre-build skip produced the same wrong digest and the
same 612 builds. Control-polygon sphere:
`/tmp/moebius-perf-a-sphere`, queries 15639, pipe hits 916, mean tick
24.898 ms. Sample-exact sphere:
`/tmp/moebius-perf-a-sample`, queries 15639, pipe hits 897, pipe candidates
1516, pipe rejects 971, mean tick 24.800 ms. Against the baseline tick log,
agent rows differ on 0 of 420 ticks and contact rows differ on ticks 288,
294, 300, and 301. Each drifted sample is organism 1 against pipe filament 2
chunk 0. Responses and time-of-impact stay the same. The point moves by tens
of micrometres, and tick 300 also moves one normal component by 1. The three
rejected logs agree with each other on all 420 contact rows. Path metres
matched the contract. The digest is the contract. Those skips are not in
the source.

The validated gate skips a chunk only when the sample sphere says the chunk
is far and the cache already holds that key. A miss is still built, then
`near` and the hollow-trimesh cast run. `pipe_rejects` counts “sphere says
far,” including misses that were filled. The five sites are the mover sweep,
`clear_shape`, `deposit_allowed`, `emission_obstacle`, and `admit_roots`.

The 2026-11-25 colony scale ran this same gate. Founders 1, seed 77, 420
ticks, digest exact, queries 15645, pipe builds 623, hits 905, rejects 971,
mean tick 24.358 ms. `collision.rs` in that pass only aliases the pipe cache
key and changes a test to borrow the surface. The contact rule is the one
measured above. Population scaling is in
`SIMULATION_0001_PLAYABLE_COLONY_RESULTS.md`.

**A. Full, witness disabled, validated gate.**
`/tmp/moebius-perf-a-fill`. PASS, collisions on, replay equal, material gap
0, error null, digest exact. Tick log matches the baseline on every agent
row and every contact row. Elapsed 21.134 s, simulation time 10.456 s, mean
tick 24.895 ms, p50 8.168, p95 88.049, p99 115.362, max 578.153. Mean resolve
19.638 ms (`resolve_ns` 8248161462). Queries 15645 and 7.946 s, about 96% of
resolve. Body builds 2, hits 10169. Pipe builds 623, hits 905, candidates
1516, rejects 971, geometry 0.260 s. VmHWM 10416 kB before replay, 13468 kB
after. Against the pre-filter baseline, pipe hits fell from 1837 to 905.
The query count and the mean tick stayed where they were. On this 58-node
world the gate skips a clone of a far cached chunk. The expensive pairs are
the nearby hollow trimeshes that `near` already keeps.

**B. Collision mode off, witness disabled, diagnostic only.**
`/tmp/moebius-perf-b-collision-off`. Status DIAGNOSTIC. Digest
`e3346b4b41bb932d0b7103e215560915cfe74512b61d083c7e3652f6ecb82e65`. Replay
equal, material gap 0. Elapsed 4.910 s, mean tick 5.570 ms, p50 4.717, p95
8.657, p99 8.691, max 8.770. Resolve calls 0, queries 0. Agents 2, movements
479, path 22.88400551621157 m, meal ticks 122, max stock 344, nodes 272,
Blood-Sacs 30, rooted 7. VmHWM 8428 kB before replay, 8688 kB after.
`HabitatRules.collisions = false` also changes birth offsets, sac crawl
height, spawn pose, and other habitat branches. This is a different world.
The product default stays on. B is not an acceptable final simulation.

**C. Witness enabled, mesh budget 1.**
`/tmp/moebius-perf-c-minimal`,
`--first-simulation --probe-world --ticks 420 --mesh-budget 1`. PASS,
headless equal, digest exact. 1075 frames, log not capped, elapsed 17.908 s
including the pace sleep. Authority tick mean 25.633 ms, p50 9.164, p95
88.113, p99 116.193, max 585.605. Publish mean 0.007 ms (max 0.032).
Collision counters match A. Mean resolve 19.834 ms. Body rebuilds 430,
reused 658, pipe meshes generated 80. Frame interval mean 16.653 ms. That
interval includes the authority pace sleep and is not the tick cost. Draw
CPU mean 1.219 ms, p95 2.834, max 13.553. Body-mesh CPU mean 0.851 ms, with
p50 0 on reused frames. Pipe-mesh CPU mean 0.002 ms. VmHWM 299512 kB, VmRSS
251920 kB. GPU timestamps were unavailable. This is a Bevy/Vulkan process;
its resident size is not the headless simulation heap.

**D. Witness enabled, mesh budget 16.**
`/tmp/moebius-perf-d-detailed`, same binary, `--mesh-budget 16`. PASS,
headless equal, digest exact. 1071 frames, elapsed 17.847 s. Authority tick
mean 25.469 ms, p50 8.995, p95 88.605, p99 116.297, max 579.153. Publish
mean 0.007 ms. Collision counters match A and C. Body rebuilds 430, reused
654, pipe meshes generated 80. Frame interval mean 16.659 ms. Draw CPU mean
1.265 ms. VmHWM 295220 kB, VmRSS 252580 kB. Budget 16 generated the same 80
pipe meshes as budget 1. The presentation stamp limited pipe rebuilds on
this world. One body rebuild per changed tick is the traveling pose.

The Witness Sphere decides what to render. It must not reconstruct unchanged
geometry every frame. That rule is implemented and printed in the C and D
`witness_cache` field. A builder stamp is tick, eating, and reservoir.
Blood-Sacs update transforms only. A pipe chunk rebuilds when its age/24
presentation stamp or lookahead span changes, and at most `mesh_budget`
chunks are rebuilt per frame.

Cost on this 58-node world: the lag is the authority tick's nearby
hollow-trimesh casts. Snapshot publish, draw CPU, and the mesh budget are
small beside that tick. The validated sphere gate does not remove the 15645
queries and does not bring the mean tick under 16.7 ms. Far cached chunks
are now cheaper to reject. Costs that stay bounded as the network grows are
not established by this gate. Writhe clearance still covers solids only.
An inexpensive exterior remains a candidate; the open lumen keeps the
hollow trimesh as the hit.

Long-run habitat abundance and arbitrary imported-state physical admissibility
must not be inferred from the barrier or Witness tests.
