# Restore the sculpt; close vertically — 2026-11-23

The user rejected v3: its replacement lip plates and locally reshaped head broke the sculpt. They then corrected the first v4 attempt, which closed sideways. Numerical closure checks in v3 did not establish artistic fidelity or the intended articulation. Those results and assets remain preserved as rejected historical evidence.

The current derived mesh restores exactly the accepted pre-v3 positions, triangle order/count and UVs:17325 sculpt vertices,25168 triangles, plus the existing1560-vertex tissue underlayer. There are only two material primitives again. The source root GLB is unchanged. The added plates, fabricated teeth, funnel and separate tongue are absent from the current asset. Silver tooth painting and throat/tongue redesign remain deferred; arbitrary convex-face material patches were rejected in v4 too.

The existing rim now closes **vertically** through two localized shape keys: upper lip down, lower lip up. Transverse X displacement is exactly zero. Only1547 sculpt vertices move;15778(91.07%) stay fixed. Cheeks/forehead outside the elliptical mouth region and the rear/body are fixed by construction and verified against actual target data. Maximum vertical displacement is0.182285 mesh units. Oral skin is anchored to the head, rather than rotating left/right jaw lobes. Existing motor24/25 angles drive the upper/lower keys through clamped(768-angle)/1024. This is authored visual rig calibration, not a new simulation actuator or a learned gait.

The closed shape leaves a narrow seam: on49 frontal sample rays,22 reach behind Y=0.52 when open versus7 when closed. An airtight seal is NOT DEMONSTRATED. The full close/open diagnostic is visible in both Blender and the live renderer. Normal native control is unchanged and need not synchronize the two halves; full autonomous chewing remains NOT DEMONSTRATED. Feeding geometry and coefficients remain unchanged.

Failed iterations are preserved: sideways contraction with bad silver patches; an intermediate vertical squeeze; a posterior-hinge trial that pulled the cheek into a horn; the rejected native-render sequence from that hinge trial. The final elliptical rim is preserved as studio-rim and vertical-render. The latter contains16 internal screenshots and metadata identifying imposed commands. These match the unchanged controlled physics replay; capture metadata also stores the two morph weights.

107 workspace tests, strict Clippy and formatting pass. The actual-asset regression now checks morph vertices, zero horizontal deformation and variation driven by native neural angles. The former bone-only native-lip audit is historical; check_worm_jaws.py now audits actual shape-key motion. No new physical reachability, learning, food uptake or finished anatomical quality is claimed.

```sh
cargo run --release -p moebius-viewer -- --motor-nursery experiments/motor_land_demo.json --motor-control selected --mouth-view --jaw-exercise
python3 scripts/check_worm_sculpt.py RESTORED/mouth-v4-baseline-pre-v3/Worm_A_rigged.glb
python3 scripts/check_worm_jaws.py RESTORED/mouth-v4-validation/moebius-jaw-native-final.jsonl
python3 python/worm_a_artifacts.py verify
python3 python/worm_a_artifacts.py unpack NEW_DIRECTORY
```

Omit --jaw-exercise for native control. Space pauses; V shows the body. Finite-energy, food-free diagnostics eventually die. Current v4 archive preserves prior archives, rejected assets/renders, source identities, tests and restore metadata. Work remains local and uncommitted.
