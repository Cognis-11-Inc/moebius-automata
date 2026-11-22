# M5 — prove reachability

**Some second meals are reachable under the existing laws and 128-energy budget. The demonstrated mechanism is chiefly mouth articulation and the existing capture stencil, not swimming to food.** A target-aware diagnostic oracle reaches 24 of 96 designed placements; the current production recurrent controller reaches none. No anatomy, food-conversion coefficient, maintenance cost, actuator strength, material law or momentum law was changed.

The forward three-site fixture remains **PHYSICAL REACHABILITY NOT DEMONSTRATED** at the default budget/medium. Its failed controls do not prove that every possible motor trajectory is impossible. This distinction is central to the result.

## Eight required verdicts

| Question | Verdict and boundary |
|---|---|
| **PHYSICAL REACHABILITY** | **DEMONSTRATED for selected second-site placements under the authored laws.** Oracle 24/96, with actual founding ingestion followed by ingestion at a different site. **NOT DEMONSTRATED for the forward three-site fixture at default budget/drag.** Continuous mesh contact is not established. |
| **ACTIVE STEERING** | **DEMONSTRATED in a limited mechanical sense:** paid input changes COM heading and can aim the mouth. A matched counter-stroke changes heading by 30.3° at tick 31. Reliable arbitrary 3D steering/navigation is **NOT DEMONSTRATED**. |
| **ACTIVE BRAKING** | **DEMONSTRATED at fixed-point resolution:** after the same paid launch, the brake oracle holds zero COM velocity during ticks 27–94. Remaining momentum is sub-velocity-resolution, not exactly zero. This controller is diagnostic, and it dies at tick 95; sustainable stopping at arbitrary food is **NOT DEMONSTRATED**. |
| **FOOD OBSERVABILITY** | Original rays see aligned food early but miss reachable off-axis sites. Optional bounded lobes fix this measured coverage defect. Long-range discovery beyond the eight-site radius is **NOT DEMONSTRATED**. |
| **ORACLE TARGET REACHING** | **DEMONSTRATED, limited:** three of twelve designed placements succeed in each of eight seeds, both from a hungry start and after a founding bite. Nine placements fail. Privileged target information stays in the diagnostic example. |
| **LOCAL-SENSOR TARGET REACHING** | **DEMONSTRATED, limited:** sensor-only motor model reaches the rear target in 8/8 seeds, or 8/96 total placements, using original rays. Optional lobes give 9/96, including one oblique case. Literal second-meal tests reproduce those counts. |
| **AUTONOMOUS FORAGING** | **NOT DEMONSTRATED for the production organism or a sparse habitat.** Production recurrent, matched random, rest and simple local-pulse controls each reach 0/96 distant targets. The authored local model is an isolated fixture controller, not a production policy or general ecological forager. |
| **LEARNED IMPROVEMENT** | **NOT DEMONSTRATED.** No trained-vs-naive, retention or generalization experiment. Oracle search and the local model are authored algorithms; neither is learning discovered by the organism. |

## Evidence, provenance and design

Base: [`fd837e7`](https://github.com/Cognis-11-Inc/moebius-automata/commit/fd837e7e19c670311ee17ad600259f1c086c6d63). Before edits, AGENTS.md, HANDOFF.md and the complete M4 record were read. All 1,732 M4 artifact hashes were checked and every saved JSON/organism trace read. All 427 saved M4 trials were replayed, including production and supplementary controls; after implementation, all 400 primary M4 trials replayed again to their original hashes. See [M4 audit](M4_EVIDENCE_AUDIT.json).

The [protocol](PROTOCOL.json) precedes executed M5 trials. It preserves later amendments for the independent local motor model and optional sensor. No external scientific papers were searched or consulted. The [research record](../research/2026-11-21-m5-reachability.json) separates source consultation, authored assumptions, implementation, mechanism tests, observations and nulls.

There are **2,509 recorded dynamic trials**, including exploration and repeated conditions, plus **480 static sensor probes**. This is not a random sample of habitats. Exploration uses seed 1; eight-seed confirmation includes that seed again. Twelve target placements are deliberately chosen, not representative resource draws. Consequently 24/96 is a benchmark count, not an estimated ecological success probability. All target failures remain saved; execution `PASS` means invariants/recording succeeded, not that a target was reached. There were zero execution failures.

The single existing founder retains its seeded genome, fourteen-joint rig and initial pose. Controlled conditions explicitly use a 32³ periodic world, COM `(16,16,16)`, no initial momentum, no reservoir/inflow/injection, and population capped at one. Standard reserve is 128, maintenance 1/tick, neural fee 1/eight ticks, uptake 1, conversion 4. Distant stock is 512. Exploratory high-reserve runs use an explicitly labelled 2,048 energy; medium conditions are drag 0/8/32/128. None changes the production default.

Targets lie three lattice sites from the **initial rounded articulated mouth site**, matching M4. Axial offsets are `(±3,0,0)`, `(0,±3,0)`, `(0,0,±3)`. The six oblique offsets are `(2,2,1)`, `(-2,2,1)`, `(2,-2,1)`, `(2,1,2)`, `(1,2,2)`, `(-1,-2,-2)`, each squared length nine. Continuous mouth-center distances differ slightly because the mouth is rounded for capture. Literal second-meal conditions add exactly one nutrient at the founding mouth; tick zero consumes it through the normal feeding law. That earned four energy is recorded. The old site is then empty; the remote target remains three lattice sites away. This is an explicit depleted-founding-food condition, not a production resource-density change.

## Stage 1: what the rig can do

[Analysis](ANALYSIS.json) contains the entire control bank, signed single-axis results, paired axis ablations, heading and braking comparisons. Raw records preserve COM/site/offset, velocity/momentum, orientation, continuous mouth location, exact mouth capture, sensors, submitted commands, smoothed drive, angles/rates/effort, action spending and death.

The screen uses seven patterns—coordinated bend, alternating opposing bends, front-only, back-only, twist, combined bend/twist and opposing jaws—at signed amplitudes 512/1024, pulse lengths 4/8/16, four media, and two explicit reserves: 672 trials. The periodic bank adds 288 default-medium/default-reserve conditions: signed bend/front/back/combined controls, pulse lengths 4/8/12, periods 16/24/32, with passive or weak opposed recovery. These are externally specified diagnostic controls; none was installed as a production gait. They do not establish global motor optimality.

| Condition | Tested controls | Best net COM displacement | Forward target successes |
|---|---:|---:|---:|
| Vacuum, energy 128 | 84 | 0 sites | 0 |
| Default drag 8, energy 128, single pulse | 84 | 0.991 sites | 0 |
| Default drag 8, energy 128, periodic | 288 | 1.424 sites | 0 |
| Default drag 8, explicit energy 2048 | 84 | 2.138 sites | 9 |
| Drag 32, energy 128 | 84 | 1.602 sites | 1 |
| Drag 32, explicit energy 2048 | 84 | 1.750 sites | 3 |
| Drag 128, energy 128 | 84 | 1.519 sites | 0 |
| Drag 128, explicit energy 2048 | 84 | 1.519 sites | 0 |

Best net displacement and target success may belong to different controls. Higher drag is not monotonically better. The best default-budget periodic control travels 1.428 sites of path, 1.424 net, and pays 15 actuator energy: **10.50 paid actuator energy/site of path**, plus 0.351 units of pending fractional actuator debt. Pending debt is retained in the model and audited separately; small integer-paid costs are not labelled free mechanical work. Maintenance and neural expenditure are additional.

The initially listed 256-amplitude and 32-tick single-pulse cells were **NOT RUN**; the executed bank is exactly the one listed above. No 512-tick high-energy capability trial was run. These omissions are disclosed rather than counted as tested failures, and further limit any global reachability conclusion.

The model admits COM propulsion, but the forward geometry and budget are unforgiving. The forward target is `(16,20,16)`, four sites ahead of COM because the founding mouth starts at lattice y=17. Capture requires the rounded mouth to get to at least y=19. Across **all 372 default-budget screen/periodic controls, the highest mouth site was y=18**; minimum mouth-to-target lattice distance squared was four, while capture requires at most two. This is the precise measured miss for that bank. The nine default-medium high-reserve successes reach this forward target at ticks 180–252 after spending 215–287 action energy before/through first ingestion—beyond the original 128 budget. One medium-change condition succeeds within 128; that establishes coupling sensitivity, not permission to silently raise drag in production.

The unsuccessful forward oracle instead settles into a locally attractive pose with almost no translation. Its finite, short-horizon search does not find the longer propulsion/preparation sequence. That is an identified controller/search limitation, not proof that every possible forward control is physically impossible.

The source explains several bottlenecks: the authored lunge needs absolute signed summed bend rate above 280; opposed bends cancel that channel. Shape coupling is coarse and quantized. Joint stops prevent further useful stroke, mouth orientation can undo COM progress, and action/maintenance costs expire the reserve. These are source-derived mechanisms tested within a finite bank, not a theorem about the complete reachable set.

### Joint contribution and braking

The 79 attribution conditions include the full combined pulse, each of 26 axes individually disabled, and positive/negative single-axis pulses. These are coupled trajectory interventions: a lower fee can extend lifetime, so differences are not additive force decompositions. Disabling a bend axis reduces the reference net displacement by about 0.092–0.155 sites. Twist effects vary in sign; removing some twists improves displacement. Disabling jaws changes nothing in that reference control because jaws were uncommanded. Single-axis/jaw motion is recorded separately rather than equated with propulsion.

The terminal spine axes have no downstream points to deform in `local_geometry`. They can still affect the authored lunge/counter-rotation, so a measured contribution is not proof of realistic segment mechanics. The fourteen-joint anatomy is unchanged.

Six matched suffix tests begin with the same paid 16-tick launch and identical committed prefix hashes. At tick 31, release gives velocity `[1,12,2]/1024` sites/tick; an eight-tick reverse pulse gives `[2,3,0]/1024`, a 30.3° heading change. A back-bend turn also changes heading, more weakly. These establish actuator influence on direction, not reliable chosen-heading 3D navigation.

Release alone never stops within 96 ticks. The brake oracle reduces speed to zero from ticks 27–94, then the cell dies at tick 95. It pays 11 actuator units after the switch versus one for release, in addition to the shared launch and neural/maintenance costs. Its final living momentum is `[0,1,-1]`, below velocity resolution. At default drag, integer damping removes no momentum when an axis has `abs(momentum)<128`, while velocity truncates to zero only below 16. This explains the coast residual and why active counter-actuation can stop motion that passive drag does not eliminate. This is stopping under the fixed-point model, not exact zero mechanical momentum or indefinite sustainable braking.

## Stages 2–4: what permits a second meal

The isolated oracle searches a finite library of signed actuator primitives using cloned **real world transitions**, including smoothing, neural fees, actuator debt and actual ingestion. The score uses privileged target location/capture geometry only inside the example. It cannot inject force, set pose, credit energy or move food. Production controllers never receive that target information. The same submitted-command stream is sufficient to reproduce every committed tick; external controller memory is not silently added to the snapshot format.

The existing neural network still integrates and can update its pre-existing plastic traces during externally controlled trials. Its motor output is overridden before the ordinary muscle/cost path, so those traces do not establish the demonstrated policy or learned competence. Every trial starts fresh; no oracle-exposed final brain/genome is adopted by production.

Oracle search and local-model prediction are experimental harness computation. Their CPU/planning cost is not separately charged beyond the existing flat neural fee; the demonstrated action-energy balances are not biological computational-efficiency results. The local model is not an implementation in the existing eight recurrent neurons, and a globally minimal controller change is not established.

The scalar-only model controller has a separate API accepting only ordinary sensors and its own memory. It reconstructs an approximate body-local motor model from joint angles/rates/effort and velocity, aims using directional nutrient differences, and attempts to hold mouth contact. It has fixed reference anatomy/tissue/drag assumptions. It is independently authored, approximate and untrained; no oracle action sequence was copied into the production controller. A simpler sensor-driven pulse comparator remains preserved as a failed controller.

| Eight-seed comparison, 12 placements/seed | Hungry-start target reaches | Actual second-meal reaches |
|---|---:|---:|
| Target-aware oracle | 24/96 | 24/96 |
| Current production recurrent controller | 0/96 | 0/96 |
| Native deterministic random policy | 0/96 | 0/96 |
| Rest control | 0/96 | 0/96 |
| Simple sensor pulse comparator | 0/96 | 0/96 |
| Sensor-only motor model, original rays | 8/96 | 8/96 |
| Same model, optional local lobes | 9/96 | 9/96 |

The oracle succeeds at offsets `(0,-3,0)`, `(2,-2,1)` and `(-1,-2,-2)` in each seed. In the hungry-start benchmark, first target ingestion is at ticks 6, 3 and 7 respectively. At those bites, COM displacement is only **0.003–0.019 sites**, while continuous mouth displacement is **1.03–1.61 sites**. In literal second-meal tests it is **0.002–0.023 sites COM** and **1.03–1.61 sites mouth**, with 6–14 total action energy spent through target ingestion. Most of the reserve therefore remains. These are not journeys across three COM sites.

**Capture geometry makes that result possible.** Food uptake checks `Body::head_site` at spine point `spine-3`, rounds continuous position, then accepts neighboring sites with squared lattice distance ≤2. No learned jaw closure or continuous collision/contact is required. For the successful `(2,-2,1)` hungry-start cases the continuous mouth center is still about **2.02 sites from the food center** at ingestion. This is valid under the current authored feeding law; it does not demonstrate literal continuous mouth-food contact, swimming, or realistic full embodiment. All eligibility calculations were independently checked against recorded head and food sites.

In the successful rear-target local-model trials, ingestion continues on every tick from tick seven through 127: 121 bites, final energy 465. This demonstrates maintained capture and a positive action-energy balance over that window. COM still drifts (sample final velocity `[4,-2,0]/1024` sites/tick); integrated arrival-and-stopping is not demonstrated. The separate brake oracle establishes a braking mechanism, not a completed food-arrival policy.

### Observability and the optional sensor

The original six scent channels rotate body directions, then snap each to its dominant world axis and inspect an eight-site line. A food site differing from COM in multiple coordinates can therefore produce **no chemical signal even when nearby and physically capturable**. At a ~45° test orientation, snapping also loses coverage of two of six aligned targets. Rotating the body does not turn these into diagonal world-space search rays.

In the initial orientation at equal **COM distance three**, original rays observe all 6 aligned targets and none of 24 oblique targets. Measured from the initial mouth site, only 2/6 aligned targets are observed from the COM-based rays, and 0/24 oblique targets. The rear successful target supplies a signal at tick zero, early enough for the scalar-only motor model to consume at tick seven (tick eight after the founding bite). A regression compares the blind oblique fixture with food-absent state and proves identical sensor streams/commands for that controller during the tested early window.

After that evidence, an optional `ChemicalSensing::LocalLobesV1` was introduced. It sums authoritative nutrient stock within an eight-site minimum-image radius, once per periodic site, weighting by distance and positive projection onto each **actual rotated body direction**. Normalization and bounds are explicit in the protocol/source. It outputs the same six scalar slots; no food coordinates, IDs, target rules or global map enter the controller. Even-dimension half-axis ties use the documented negative direction. This is an authored bounded field measurement, **not chemical diffusion or a published biological receptor model**. Existing fixed neural fees remain; no biological sensing-cost claim follows.

The new sensor observes all 6 aligned and all 24 oblique equal-radius targets in both anchor conditions and all four tested orientations. Range, rotation, stock removal, chemistry ablation, unique periodic samples, serialization compatibility and identical paid physics under identical commands have regression tests.

Its motor benefit is small: the same local model adds one successful oblique case, offset `(-1,-2,-2)` in seed 6, with a first signal at tick zero and ingestion at tick nine (tick ten after the founding bite). The other off-axis failures remain. Thus sensory coverage was a genuine defect, but repairing it does not establish a competent forager. Mouth/COM control and state estimation remain substantial limitations.

The **production default remains law 2 with the wave off and original rays**. The optional sensor is explicitly selected by `chemical_sensing: "local_lobes_v1"`; [the example config](../../experiments/nursery_m5_local_lobes_v1.json) changes that setting only. It still runs the production recurrent controller, not the experimental local model. No diagnostic policy was installed into the viewer.

## What changed, checks and portable artifacts

Changes are confined to a validated raw-command diagnostic interface, a read-only ordinary-sensor accessor, receipt serialization, isolated experiment/controller/replay code, the optional versioned sensor, tests and evidence. `Body` mechanics and all production energy/material/momentum coefficients are unchanged. Original config encoding omits the new default-valued option; original snapshots/configs remain compatible. No species, anatomy, reproduction system, viewer animation or cosmetic movement was added.

**89 workspace tests pass**, including nine M5 regressions. Strict workspace Clippy, formatting and diff checks pass. All **2,509 trials, 5,018 snapshots, 325,653 ticks and 480 sensor probes** replay exactly, including every per-tick receipt, sensor vector, organism telemetry record and state hash. Independent Python auditing recomputes individual energy, actuator fees/debt, material balance, momentum closure, motion, actual joint rates and capture eligibility. The separate M4 post-change replay verifies original-law compatibility. See [validation](VALIDATION.json), [replay](REPLAY_AUDIT.json), [analysis](ANALYSIS.json) and [bite events](GOAL_EVENTS.json).

The user correctly flagged the initial 10,280 generated Git artifact paths. They were bundled into **12 per-experiment raw archives**, then externalized to **Mermogon** on 2026-11-22 under the new storage gate. [Storage classification and locations](ARTIFACT_STORAGE.json) records their sizes, purposes, version and hashes; authoritative protocol/results and reproducibility manifests stay in Git. [CONFIGURATIONS.json](CONFIGURATIONS.json) retains all 2,509 per-trial configuration identities and deduplicated complete configuration values in Git. Every raw file was verified before its expanded copy moved to local cache. Complete unpacking was tested, followed by replay from unpacked evidence. No trials or failed outcomes were deleted. The eight-seed result counts use the designated confirmation/model datasets, not exploration duplicates.

```sh
# Verify mounted Mermogon, then restore all raw evidence into a new directory:
python3 python/m5_artifacts.py verify
python3 python/m5_artifacts.py unpack NEW_EXPANDED_DIRECTORY
cargo run -p moebius-core --release --example m5_replay -- NEW_EXPANDED_DIRECTORY/confirmation_v1

# New experiments always require new output directories:
cargo run -p moebius-core --release --example m5_probe -- screen NEW_SCREEN_DIRECTORY
cargo run -p moebius-core --release --example m5_probe -- benchmark NEW_BENCHMARK_DIRECTORY 8
cargo run -p moebius-core --release --example m5_probe -- second_meal NEW_SECOND_MEAL_DIRECTORY 8
cargo run -p moebius-core --release --example m5_probe -- local_model NEW_LOCAL_DIRECTORY 8
cargo run -p moebius-core --release --example m5_probe -- local_lobes NEW_LOBE_DIRECTORY 8

# Repeat the independent analysis using expanded evidence and saved replay JSONL:
python3 python/analyze_m5.py NEW_EXPANDED_DIRECTORY REPLAY_OUTPUT_JSONL
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

The default restore requires the verified Mermogon volume UUID and reads its versioned archive root. On another machine, pass `--archive-root VERSIONED_DIRECTORY` for an explicit archive copy; SHA256 checks remain mandatory. Unpack outside the checkout to keep the Git file list compact. For raw-directory replay the loader accepts plain or gzip-compressed `.bin` and `.jsonl`; the saved archive layout uses gzip. Snapshot replay alone cannot resume an external controller without its command stream. No live viewer witness or hosted CI check is claimed. The scientific validation record predates publication; consult Git for commit/push status.

The remaining problem is specific: develop reliable COM propulsion, mouth alignment and local closed-loop control for placements that folding/capture cannot reach, while paying the existing budget. Some second meals are physically possible now; a viable autonomous route through sparse food is not established. Increasing energy or changing the medium would conceal that boundary if done silently, so neither became a production fix.
