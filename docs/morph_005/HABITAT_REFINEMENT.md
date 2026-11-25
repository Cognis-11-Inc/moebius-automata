# Snailpipe habitat refinement — 2026-11-24

Current 2026-11-25 continuation: **PARTIAL, collision and exploration repairs implemented**.
See [current repair status](SIMULATION_0001_REPAIR_STATUS.md). Final checks and
smooth, lightweight operation remain pending. Historical receipts follow.
See [the causal diagnosis](SIMULATION_0001_COLLISION_DIAGNOSIS.md) and the
[root results](../../SIMULATION_0001_PLAYABLE_COLONY_RESULTS.md). Historical
receipts below do not certify absence of trapping or current-source performance.

Status: **PARTIAL**. World rules are implemented and focused-tested. The
native visual receipt of continuous hunting, feeding, movement, and snailpipe
deposition for one bare founder is `/tmp/moebius-habitat-native-v7` (digest
`76309b40cee405ef51e80107c360737a2803513d2ad4ef753953bf90d5333fc5`). The
two-founder showcase receipt is `/tmp/moebius-colony-native-20261125b`
(digest `91ddb63588132a8125f11279361c2143d6791ea3c637fb8602e73af86568ba19`,
180 ticks, headless equal). On the showcase seed, the validated sample-sphere
gate (`/tmp/moebius-perf-a-fill`, digest
`7b68517ba0f82ed979469ef059db5f151243eb6999fe8d37f2e58d69940c2afe`) has mean
tick 24.895 ms. The 2026-11-25 colony scale reproduced that digest at mean
24.358 ms with the same 15645 queries. The gate skips a far chunk only after
that chunk is already cached. It does not remove those queries on this
58-node world. The centerline filter and any pre-build sphere skip were
measured, changed the digest, and were removed. Sixteen founders are the
rule ceiling and averaged 790 ms/tick. Populations of 30, 100, and 300 were
refused. Natural Arachnitech births in these windows were 0. See
`SIMULATION_0001_PLAYABLE_COLONY_RESULTS.md`. Collision-off, minimal-witness,
and detailed-witness receipts are in COLLISION_CONTRACTS.md. The mean tick
stays above 16.7 ms. Clean-commit certification is not done. Base main
5a83c21, dirty, no commit or push.

Operator priorities for SIMULATION-0001, recorded 2026-11-24:

1. Continuous life. Feeding must not permanently freeze the organism.
2. Embodied motion. Generated anatomy twists and turns. Visual pose is not
   propulsion. The current motion is authored, not learned.
3. Physical interaction. Collision must not trap an Arachnitech on its own
   snailpipes, and it must not become the cost of the tick.
4. Persistent consequences. Feeding and movement leave material-backed
   structures that remain after the body leaves.

The first drive is dinner. Reproduction is second. Snailpipe is the trail of
that life, not a plan the creature knows. A gulp suppresses secretion for that
tick and the body keeps a shorter step. Its own fresh pipes may braid. A pipe
or another builder deflects the step instead of cancelling it. Solids still
stop. The hit volume stays the tight cached envelope (bend 0.12, phase 0).
Solids grow by the measured presentation clearance. Pipes and prey do not.
The rendered writhe is the authored pose (traveling phase, bend 0.72) and is
not a worm brain. See COLLISION_CONTRACTS.md for the split cost receipts.

Risk cage: a pretty tube is not a material source or a mechanical simulation.
Preserve canonical ShipRuntime -> MaterialBook -> Assembly -> immutable witness.
No wall species, mechanics/strength, learned pursuit, external runtime, erased
failures or unlimited stock. The operator's current instruction authorizes paid
reproduction and prey evasion, superseding MORPH-005's historical no-reproduction
scope for the explicit new habitat mode only. Nursery/old ship laws stay separate.

Authority/reuse matrix:

| Concern | Inspected path/symbol | Status/input -> output | Consumer/authority/tests | Reuse / missing seam |
|---|---|---|---|---|
| Anatomy | anatomy.rs AnatomySpec/CompiledAnatomy | canonical spec -> geometry/output frames | editor + ship; composition tests | extend Shape with FoodPouch and explicit capacity |
| Storage/intake | emission.rs MaterialBook::ingest/endow | canonical food -> reservoir | ShipRuntime, integer balance | pouch limits intake; never a second stock |
| Trails | filament.rs Assembly::deposit/filament_chunk | paid parcels -> persistent polylines -> mesh | runtime owns graph, witness only mesh | smooth directional policy and interpolated cross-sections; explicit paid-node contact junctions |
| Feeding/pursuit | runtime.rs steer_constructor/blood_tick | active authored single builder + immobile sacs | first_simulation tests | extend with opt-in local-memory policy, pause secretion at meals, move prey |
| Reproduction | runtime.rs birth/hemorrhage | inactive constructor brood / paid food spores | material transfer primitives | new habitat offspring must carry active steering and use same book/assembly |
| Visibility | filament StructuralIndex; witness WitnessSphere | derived sparse chunks -> visible set | render neutrality/budget tests | preserve culling; bounded age refresh and mesh construction |
| Donors | MORPH_001_DONOR_ASSESSMENT; MORPH_004_DONOR_ASSESSMENT | consulted historical architecture | local records, not runtime evidence | no copied donor anatomy or extra world owner |

Small vertical target: a food pouch with capacity and visible fullness; local
pursuit/evasion with stored directional memory; no simultaneous feeding and
emission; bounded paid active offspring; gradual snailpipe sweeps and wet-to-dry
appearance; sacs with world-owned substrate anchors; contact junctions and route
following within the existing assembly. A supporting habitat module may house
policy/state helpers, invoked only by the canonical tick, not another runtime.

Proof obligations: finite stock/capacity, 4x cost, no emission at meals/rest,
prey escapes locally yet is catchable, smoothed heading, active paid offspring,
exact save/replay and witness neutrality, persistent paid junctions, seam/age
mesh checks and bounded generation, native visual comparison. Preserve old tests
and mark changed mode/defaults explicitly. Full clean-commit certification stays
pending without a new authorized commit. No external scientific research needed:
these are authored simulation and presentation rules.
