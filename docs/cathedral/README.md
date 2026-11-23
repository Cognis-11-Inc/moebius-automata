# Cathedral foundation — permanent matter, editable form

User direction,2026-11-22: slime is the first land; the organisms' role is to build the cathedral. Deposits must not fade or disappear, but creatures may sculpt by reshaping and relocating conserved material. This is a construction objective, not evidence of construction intelligence.

Run the new native demonstration:

```sh
cargo run --release -p moebius-viewer -- --motor-nursery experiments/motor_land_demo.json --motor-control compare
```

ID1 carries the genetically selected movement controller; ID2 is the stationary untrained comparison. Both are alive at tick1024. This is food-free and provisioned with4096 energy per organism and1024 material units. They will eventually exhaust energy; this change does not demonstrate ecological survival or cathedral planning.

## Material rules

`farm.terrain_law` is an explicit authored material-rule variant, independent of unchanged law2 body dynamics:

- Omitted / `recyclable_v1`: exact historical behavior and serialization; composting and destructive digging remain available for historical experiments.
- `permanent_v1`: deposits accumulate, never compost or excavate; immovable material.
- `conserved_sculpt_v1`: permanent material with editable form. No composting or destructive excavation. A successful sculpting action transfers at most2 units from a reachable mouth/action site to the creature's current site, costs1 energy in addition to ordinary motor fees, and produces no waste, nutrient or energy. Total deposited material cannot fall. Capacity limits refuse transfers instead of losing matter.

The interactive viewer's new-world default and the new demo use `conserved_sculpt_v1`. Historical config constructors, files and M6 `DEMO.json` remain unchanged for replay. A sculpt command does not create a structure target or a reward signal. Legacy rigs use an explicit directional digging action. Neural rigs require loaded, moving, actively closing jaws with actual closing drive below-256; passive spring relaxation cannot sculpt. Material placement is a local authored interaction, not a full mechanics model for transporting chunks.

Existing deposition draws1 material unit from the accounted reservoir on a successful site crossing. Source material is explicit; when the reservoir/capacity is exhausted, deposition stops without erasing prior deposits. Existing spatial fusion/collision laws remain. Current bounds are8192 deposit sites and64 units/site. Raw motion-history samples are still bounded for pose interpolation, but they no longer render as slime: live land geometry is derived solely from authoritative `World::flesh()` deposits, with no time attenuation or history-ring truncation. Key3 toggles that material surface. Solid tissue uses the existing tissue material.

## Evidence and limits

The1024tick native demo records5 units relocated by sculpting and has14 deposited units and1010 reservoir units, total1024, with zero material/energy residuals. Both creatures remain alive. The actual fixed-camera PNG was inspected; its HUD state hash matches the independently recorded/replayed native state. An earlier strict immovable variant produced17 units; that capture and its reconstructed configuration/reproduction are retained separately. Different trajectories under different material rules are not M6 benchmark improvements.

Six focused terrain tests cover historical encoding, persistence beside food through512ticks and snapshot restart, strict refusal of destruction, paid conserved relocation, neural active closure versus passive relaxation, and receiving-capacity/unfunded refusals. All100 workspace tests pass; strict Clippy/format checks pass. All1724 historical M6 episodes and400 M4 primary trials replay with exact hashes after the change. The new1024tick run also replays exactly from its saved initial snapshot through every receipt, land map and state hash.

This establishes persistent material and a tested local sculpting affordance. It does **not** establish purposeful architecture, spatial logic, learned sculpting, ecological transfer, or a controller that knows when/where to build. No template cathedral, privileged construction target or scripted building behavior was installed.

See [VALIDATION.json](VALIDATION.json), [SOURCE_IDENTITY.json](SOURCE_IDENTITY.json) and [ARTIFACT_STORAGE.json](ARTIFACT_STORAGE.json). Original M6 reports/identities describe the frozen M6 slice; current code additionally implements this post-M6 material rule. Nothing was committed or pushed.

Restore and replay:

```sh
python3 python/cathedral_artifacts.py verify
python3 python/cathedral_artifacts.py unpack NEW_DIRECTORY
cargo build --release -p moebius-viewer --example land_probe
target/release/examples/land_probe replay NEW_DIRECTORY/native-run
```
