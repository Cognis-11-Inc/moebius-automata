# Closing mouth and silver teeth — 2026-11-23

The previous fix removed two viewer pauses and demonstrated lip displacement. It did **not** demonstrate a closed mouth. The user's live screenshot correctly exposed that gap: the source sculpt retained an open cavity, its visual aperture was offset from the authored landmark, and the selected controller holds the left jaw near its open limit. Previous records remain historical; their displacement checks must not be read as full closure.

The current render asset adds thick wrinkled bilateral lips calibrated to meet at the existing jaw minimum(-256),20 silver recurved teeth, a recessed dark throat and a separately modelled tongue. The original central filling is removed from the derived mesh; the supplied root sculpt remains unchanged. The front sculpt is aligned by an authored0.25 mesh-unit vertical adjustment. Visual jaw pivots are fitted to the new lip geometry. Six material primitives share the same14-bone rig; the tongue follows the head and has no invented motion. Silver is a material appearance, not newly simulated metal.

## What actually closes

`--jaw-exercise` is an explicit diagnostic. It imposes-1024 jaw motor commands for32 ticks and+1024 for32 ticks, repeating; all other actuators rest. It calls the existing controlled-actuation interface and pays ordinary metabolic/actuator costs. It never sets a pose, refuels the creature or changes the production controller. Both the HUD and capture metadata identify imposed control. Normal runs still call native ticking. This feature requires an explicit motor-nursery configuration.

Over256 controlled ticks both jaws reach[-256,-256] at each closing hold and[768,768] at each opening hold. Independent projection tests on the **actual lip triangles** show all9 central frontal rays blocked at closure and none blocked at full opening. The closed seam overlaps; the mean open seam gap exceeds0.30 mesh units. Each tick has zero material and energy residuals. Spending from tick1 to256 was301 energy units. This establishes render closure under controlled motors, not triangle-level feeding physics.

The native selected controller still does not fully close: in256 native ticks its mean seam gap ranges0.09658–0.48969 mesh units. Full native closure and autonomous chewing remain **NOT DEMONSTRATED**. All512 selected/untrained native telemetry rows remain byte-identical to the preceding slice. Core laws, genes, food fixture and supplied source/collision assets retain their prior hashes. The existing physical bite stencil remains unchanged; the deeper visual throat does not grant larger food uptake.

## Validation and evidence

107 workspace tests pass; the added test verifies paid controlled closure/opening and exact replay across two worlds. Final viewer tests, strict Clippy and formatting pass. Sixteen internal live-renderer frames at ticks16–256 show four close/open cycles with16/16 exact matches to controlled replay state hashes and jaw angles. The GIF uses those frames at their0.64-second sampling interval. Blender front close/open views and actual live pixels were inspected. The first modelling pass sat on the chin; its renders are retained as a rejected alignment. Later fitting/contour witnesses and compilation/audit failures are retained too.

```sh
cargo run --release -p moebius-viewer -- --motor-nursery experiments/motor_land_demo.json --motor-control selected --mouth-view --jaw-exercise
cargo run --release -p moebius-viewer --example mouth_exercise > exercise.jsonl
cargo run --release -p moebius-viewer --example jaw_probe > native.jsonl
python3 scripts/check_worm_mouth.py exercise.jsonl native.jsonl
```

Omit `--jaw-exercise` for the native controller. Space pauses; V shows the body. This food-free demonstration has finite energy. `--record-mouth NEW_DIRECTORY` records16 rendered frames and mode metadata without pausing.

Authoritative reports, source identities and reproducibility metadata remain in Git scope; raw evidence belongs in the immutable Mermogon v3 addendum. See ARTIFACT_STORAGE.json; `python3 python/worm_a_artifacts.py verify` checks destination hashes and `unpack NEW_DIRECTORY` verifies restored members. Previous v1/v2 archives are preserved. No commit/push made.
