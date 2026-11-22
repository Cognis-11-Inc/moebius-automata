# M4: locomotion failure and ecological viability — 2026-11-21

**Verdict: useful locomotion and learned foraging have not improved.** The controller can articulate, and imposed strong bends can translate the rig, but the tested controllers do not bridge even the three-site food fixture. Nursery survival primarily demonstrates sustained contact with founding food. The smallest corrections fix false joint-rate reporting, omitted actuator cost, and two integer/finite-difference defects. They do not establish an effective survival strategy.

The viewer's default nursery now uses neural law **2**, with `spinal_wave: false`. No energy reserve, food spacing, conversion, actuator strength, or maintenance cost was raised. Explicit historical law-1 JSON configurations retain their original behavior, including its defects. Wave-on remains an experimental condition. Removing the wave is not presented as a propulsion solution; it can reduce nursery intake by moving the mouth away from its founding meal.

## Evidence and chronology

Read [protocol](PROTOCOL.json), [machine-readable analysis](ANALYSIS.json), [mechanical interventions](MECHANICS.json), and [validation](VALIDATION.json). Raw per-organism tick records, exact configurations, and compressed initial/final snapshots are under [artifacts](artifacts/). These are new measurements of the **supplied dirty working tree**, not measurements of the last published commit. Existing changes and historical studies were retained.

1. Read AGENTS.md, HANDOFF.md, and the user's M4 directive. No external scientific publications were searched or consulted. [Research record](../research/2026-11-21-m4-source-diagnosis.json) identifies source-derived assumptions and evidence status.
2. Added observational telemetry and wave ablation without altering law-1 defaults; saved protocol before either primary run.
3. Ran 200 baseline trials: five environments × five controllers × eight seeds, up to 256 ticks. Preserved every trial, including nulls; zero execution failures.
4. Introduced explicit law 2, then ran the same 200 conditions. Zero execution failures. Environmental budgets and seed placement remained matched.
5. Re-ran all 200 law-1 trials using the corrected executable: the entire results JSON matched the saved baseline. Independently decoded/audited 800 primary snapshots and replayed all 400 trials to their exact final hashes.
6. Added three exploratory full-scale spot checks: 256³, original nursery spacing **16**, 192 founders, seed 1, 256 ticks: law-1/current, law-2/current, law-2/no-wave. The first production command was issued before its written amendment during compilation; these are explicitly **not strictly preregistered**. No tuning followed these observations.

A final control audit found nominal `wave_only` ecological descendants developed raw motor commands up to magnitude 31 despite zero founding motor weights. Eight additional law-1 `locked_wave_only` trials with plasticity disabled retained that drift, so lifetime plasticity alone was **not** the explanation; inherited mutation remained active. Those unsuccessful control-isolation runs are preserved. Sixteen additional `fixed_wave_only` trials disable both plasticity and mutation to isolate the authored wave. These are supplementary, declared after the primary results, not substitutions for them. The primary current/no-wave comparisons remain valid. The nominal rest condition also starts with zero output weights but permits inherited changes; its measured zero COM travel and zero billed motor energy are observations, not a guarantee of genetically zero output forever.

The fully fixed wave controls produced **zero raw neural commands** on every recorded tick in both laws. Both had 36,768 total intake, 96 births and 192 final living cells across eight seeds. Law 1 billed zero actuator energy; law 2 billed 4,416. Their total COM paths across all organisms were 0.0176 sites and zero respectively. These supplementary results confirm contact-fed reproduction with an authored drive, without a successful foraging strategy. The plasticity-only control failure remains in `artifacts/locked_wave_v1/`; the clean controls are `fixed_wave_v1/` and `fixed_wave_v2/`.

The primary smoke fixture has spacing **8**, 32³ dimensions, and 12 founders in its ecological condition. It must not be conflated with the live 256³ configuration. High reserve and nearby-food changes in the free/easy/moderate conditions are explicit experimental conditions, never hidden production adjustments. Moderate trials remain energy-provisioned at the observation horizon; their survival does not demonstrate sustainability.

## Exact implementation audit

Short source paths below live under `crates/moebius-core/src/`. Line numbers may drift; symbols are authoritative.

| Mechanism | Source / call chain | Status and consequence |
|---|---|---|
| Traveling drive, delay, threshold | `neural/controller.rs::NeuralState::integrate → slew_muscle → spinal_wave` | Authored, not discovered. It runs from the first tick at amplitude 640/pull 512; amplitude 960/pull 720 when energy <96 **or** `steps >8 && last_intake ==0`. This means previous-tick empty intake, not eight consecutive unsuccessful bites. Phase is `tick %12`; bend sign uses `(axis_index + phase)%6`. |
| Neural output | `integrate`, `ControllerGenome::founder` | Eight recurrent neurons; random initial weights, small motor biases, 26 torque axes + valve/local-nozzle xyz. Random baseline is deterministic by seed/id/tick/axis. Recurrent state, authored physiological plasticity and germline mutation are distinct. |
| Inherited / lifetime changes | `feedback`, `offspring`, `ControllerGenome::mutate` | Plastic recurrent/output traces change during life; half is assimilated into child genes before germline mutation, then traces reset. No useful learned strategy follows from weight changes alone. |
| Commands to body | `world.rs::World::advance` calls `integrate`, `actuator_drive`, then `Body::step_with_law` | The muscle drive can differ substantially from neural output because smoothing/wave is downstream. Telemetry saves both. Passive spring/return-to-rest contributes effort without an active command. |
| Joint dynamics | `neural/body.rs::Body::step_with_law` | Kinematic rates, caps 10/40/64, bounded angles, spring, approximate counter-rotation. Law 1 can report motion at a clamped stop. Law 2 reports actual angle difference. Jaws only oppose closing under local load; no full capsule/body collision mechanics. |
| Translation / medium | `Body::step_with_law → shape_impulse, lunge_impulse`; `World::advance` applies velocity to site/offset | Authored resistive coupling plus an explicit lunge when absolute summed bend rates exceed 280. Counter-rotation alone supplies no COM thrust. Vacuum produces no internal-motion COM thrust. External reaction is exactly booked. Integer momentum/16 gives velocity; one velocity unit is 1/1024 site/tick. Internal actuation is not a full hydrodynamic solver. |
| Movement blocking | `World::advance → fused_surface, resolve_moves` | Fused flesh or occupied-site crossings can block; offsets revert, momentum reverses/damps, reaction goes to environment. Thread material condenses from reservoir on a crossing. No global path planner. |
| Motor / neural cost | `integrate` then `World::advance` | One neural energy every eight steps. Original actuator coefficient: scaled absolute joint drive summed, divided by 8, accumulated against 4096. Law 1 bills **raw neural commands**, missing wave/smoothing changes. Law 2 bills actual requested muscle drive and suppresses unfunded active drive. Passive springs still operate as authored mechanics. This is action-energy accounting, not mechanical energy conservation. |
| Other energy | `World::advance` | Maintenance 1/tick. Each nutrient gives 4 energy. Successful secretion costs 2 and consumes source nutrient; successful excavation costs 1. Exact per-organism breakdown, intake and disposal are in every-tick telemetry. |
| Sensing | `World::observe_neural → ray_scent → sensing::encode` | Six body-facing channels are snapped to the dominant world axis. Each samples an eight-site **line**, weighted 8…1 and normalized by `site_cap*8`; this is not an isotropic smell radius or chemical diffusion. Rays can alias under rotation. Off-ray nearby food can be invisible; regression fixture confirms it. No coordinates enter the controller. |
| Feeding | `World::advance → feeding_site → neural_jaw_target → Body::head_site`; `feeding::mouth_sites` | Automatic uptake at the articulated head's short lattice neighborhood, squared distance ≤2. Does not require learned jaw closure. `head_site` uses spine point `spine-3`, not the distal endpoint. The first available mouth site wins. Food encounter, ingestion, and reward signals are distinct. |
| Distribution/refill | `feeding::sites`, `World::genesis`, `inject`, `compost` | Lattice clots, 512 nutrient each; founders deliberately placed one site below selected clots. Production spacing 16, injection/inflow 1024 total per tick, not per clot. Smoke spacing 8 and 64/tick. Nearby flesh can become one nutrient per clot/tick; distant threads stay. Limited total injection does not guarantee each occupied clot is replenished before depletion. |
| Death | `World::advance` maintenance branch | A cell dies when energy after actuation/feeding/secretion/digging is insufficient for maintenance. Age alone does not kill it. Telemetry preserves death tick, last pose and spending before removal. Zero-based death tick is distinct from receipt tick-after. |
| Reproduction | `World::reproduce` | Reserve ≥256, maturity ≥32, cooldown ≥128, population cap and free adjacent non-solid site. Cost 8 then remaining energy split; momentum transferred to child. Per-organism telemetry records total outgoing birth energy, including transferred child energy, separately from action spending. |

## What actually failed

**Propulsion is much smaller than the food gap.** At 2048 starting energy, 256 ticks without food, the current controller travelled only 0.057 sites on average under law 1. The same controller without the wave travelled 0.125. Neither is a useful journey across spacing 8 or 16. Law 2 removes false motion but does not make the controller effective. Strong imposed bends demonstrate a capacity the controller is not exploiting; they are diagnostic interventions, not a replacement gait.

**Wave movement is not neural discovery, and its energetic benefit was misrepresented.** Under law 1, wave-only produced no billed motor cost. Law 2 bills that drive; wave-only in free locomotion spends 512 actuator energy over eight trials and produces zero COM path in this fixture. The exact integer and symmetric cancellation behavior matters: a large visible oscillation need not generate useful net thrust. Within either law, removing the wave increases mean free travel for these eight seeds. This is not evidence that removing it universally improves ecology: wave/rest controls maintain founding mouth contact better.

**Articulation metrics previously counted fictitious motion.** Law 1 retains nonzero reported rate when angle cannot advance. That rate participates in counter-rotation and propulsion estimates. Across primary trials it occurs hundreds of thousands of times; law 2 reduces this count to zero. In addition, the previous finite-difference probe always looked toward +64, vanishing at the upper bound, and rounded each segment force before summing. Law 2 probes inward at the upper bound and aggregates before division. These are narrow numerical corrections; no fluid equations or optimal gait were added. The behavioral study compares this grouped correction with v1, so it does not isolate each numerical correction's ecological effect.

**Controller sensory opportunity is incomplete.** Ray sensing can see an aligned clot but miss an off-axis one at short Euclidean distance. Food near the mouth can also lie outside the discrete capture stencil. It is inaccurate to diagnose those failures as an inability to learn navigation without first demonstrating a usable local sensing/motor loop. No new isotropic sensor was introduced: choosing and validating one remains unresolved.

**The nursery masks these deficits.** The motionless control obtains food and produces offspring. In all primary trials, every organism's successful meals remain at its own first recorded food site. Offspring may inherit a position already beside the founding clot; those births are habitat-mechanism observations, not independent navigation achievements.

## Matched measurements

Each row covers eight seeds. Path and displacement are means for founders; energy totals cover all organisms in that condition. “Current” means wave on; “no wave” uses the same recurrent controller. Baseline random/wave/rest controls and all joint arrays remain in ANALYSIS.json and results JSON.

| Condition / controller | v1 path, sites | v2 path, sites | v1 / v2 feeding trials | v1 / v2 final living | v1 / v2 actuator cost |
|---|---:|---:|---:|---:|---:|
| Free / current | 0.057 | 0.023 | 0 / 0 | 8 / 8 | 227 / 634 |
| Free / no wave | 0.125 | 0.106 | 0 / 0 | 8 / 8 | 230 / 228 |
| Free / random, no wave | 0.195 | 0.223 | 0 / 0 | 8 / 8 | 831 / 459 |
| Easy / current | 0.003 | <0.001 | 0 / 0 | 0 / 0 | 86 / 213 |
| Easy / no wave | 0.038 | 0.030 | 0 / 0 | 0 / 0 | 85 / 81 |
| Easy / random, no wave | 0.019 | 0.041 | 0 / 0 | 0 / 0 | 271 / 168 |
| Moderate / current | 0.081 | 0.021 | 0 / 0 | 8 / 8 | 226 / 634 |
| Smoke sparse / current | 0.030 | 0.007 | 8 / 8 | 8 / 8 | 232 / 370 |
| Smoke ecology / current | 0.012 | 0.004 | 8 / 8 | 192 / 192 | 4074 / 6590 |
| Smoke ecology / rest | 0 | 0 | 8 / 8 | 192 / 192 | 0 / 0 |

Every easy trial failed: **0/40 baseline and 0/40 corrected**, regardless of controller. Every moderate trial also had zero intake. “Survived moderate” merely means the explicitly larger initial reserve outlasted 256 ticks.

In corrected easy trials, mean zero-based death tick was 90.375 for current, 104.875 without wave, 95 for random, 94 for wave-only, 114 for rest. With 128 initial energy, maintenance plus neural fees alone allow roughly 114 ticks without intake. Actual articulation/turning leaves the COM almost where it started; the tested policies do not cover the food gap before this deadline. Extrapolating speed into a navigation guarantee would be invalid because pose, drag, sensing and energy state all change. Even the three-site setup is **not demonstrated to be physically reachable within budget by these controllers**.

The no-wave and random easy trials spent 23 and 17 organism-ticks respectively within two geometric sites of food without consuming it. “Within two” is deliberately wider than the capture stencil (squared distance ≤2); it does not imply a failed eligible bite. Exact stock at every automatic attempt is recorded. No reachable route through obstacles was computed: nearest-food distance is a periodic Euclidean lower bound, not a reachability certificate.

In the full 256³ checks, each condition ended with 384 live organisms and 192 births. Law-1/current and law-2/current each ate 73,536 nutrient; law-2/no-wave ate 70,909. Mean founding COM path was zero at fixed-point resolution, and **zero meals occurred at another food site**. Law-1/current recorded 446,302 false-motion axis-ticks; both corrected checks recorded zero. This single-seed, 256-tick observation cannot establish long-term viability or explain every reported later extinction. In particular, eventual depletion/replenishment failure needs a longer full-scale run before claiming a specific mortality cause.

## Articulation, phase, direction and braking

The rig still has twelve spinal joints with two axes each and two jaw joints. No joints/species were added. All 26 axes can change angle; activity alone is insufficient evidence of propulsion. In corrected free/current trials, bend axes were active about 256/256 ticks, with mean full excursions 259–339 fixed-point angle units; twist excursions were about 637–826. Yet mean COM path was only 0.023 sites. This is concrete evidence that animation amplitude/activity is not locomotor usefulness.

`ANALYSIS.json` contains axis-wise amplitude, actual activation counts and descriptive lag correlations for founder 1, free condition, selected bend/twist/jaw pairs over lags −12…12. Correlations describe synchronization, not causal joint usefulness or optimal phase. Full actual rates, commanded drive, actuator effort and orientations remain available for independent analysis. Joint-by-joint propulsion ablation was not run. Anatomical interpretation also has a limit: local geometry records each spine point before applying that joint's rotation; the terminal spine axes have no downstream segment to deform. Jaws do not enter the spine thrust calculation. Those axes may turn the approximate body frame but should not all be described as independent propulsors.

Thirty mechanical assays imposed a 16-tick bend, twist or jaw command and then released it, plus a coast condition with an explicitly supplied initial impulse, across both laws and drag 0/8/128. At default drag 8 under v2, positive and negative bend assays travelled 0.992 and 1.017 sites in 128 ticks, with distinct signed displacement; pure twist rotated but did not translate, and jaw commands did not translate. All momentum ledger residuals were zero. Vacuum actuation caused no COM translation. These are kinematic/medium-mechanism checks outside metabolic accounting, not viable controller policies.

The v2 coast assay slowed from 0.1104 sites/tick at tick 15 to 0.04785 at tick 127. This establishes passive damping, not active braking or stopping at food. There is no controller test proving arbitrary three-dimensional steering, chosen-target arrival, or active braking. Integer momentum/velocity thresholds and the coarse authored drag/lunge model remain important physical limitations.

## Separate viability gates

| Gate | Verdict |
|---|---|
| Mechanical locomotion | **PARTIAL**: imposed strong bends translate in medium and obey the momentum ledger; tested autonomous controllers travel negligibly without feeding-derived jets. |
| Directional control | **PARTIAL mechanism only**: opposite imposed bends produce different displacement and twist can rotate. Autonomous target steering, full 3D control and active braking **NOT DEMONSTRATED**. |
| Food encounter away from starting contact | **FAIL in tested fixtures**: easy and moderate intake zero; no second-food-site meals anywhere in primary or full-scale checks. |
| Successful consumption | **PASS for contact mechanics**: founding mouths consume; exact energy/material accounts close. Not learned feeding. |
| Energy sustainability | **PASS only for short provisioned contact windows**. Wave-off saves cost relative to wave-on in easy v2 trials. Unsupported away from founding food. |
| Survival | **PASS at founding food within the horizon; FAIL for all easy trials**. Larger-reserve moderate survival is censored by horizon. |
| Reproduction | **PASS habitat mechanism**, including motionless controls. Not evidence of locomotion or adaptation. |
| Experience-dependent improvement | **NOT ESTABLISHED / not tested here**. No retention, generalization, trained-vs-naive or learned-policy competence experiment. |

## Corrective implementation and next boundary

The new law corrects motion reporting, finite-difference/rounding and billed actuator drive. An explicit `spinal_wave` flag supports ablation; the default nursery disables it. Existing energy coefficients, physiology, food distribution, fixed-point representation, material/momentum accounting and snapshot format remain. Law version is serialized/config-hashed, avoiding silently changing old explicit configs. `Body::step` remains a legacy v1 entry point; worlds route through the validated rules version to `step_with_law`.

`World::tick_observed` collects `OrganismTick` records only when requested. Ordinary ticks and observed ticks produce identical scientific transitions and hashes. Telemetry is not serialized into scientific state. It captures birth energy transfers and the final dying-cell pose, so population changes do not conceal spending.

The immediate unresolved work is a demonstrated local sensing → commanded articulation → useful, controllable propulsion loop that can reach a new meal within the existing budget. This requires resolving sensory coverage and the authored medium model against explicit capability targets, rather than adding another prescribed gait or learning algorithm. No viable learned forager was delivered, and no claim of fixing the user's visual complaint is made without opening and observing the live window. Rendering was not used as evidence of locomotion.

## Reproduce / inspect

```sh
cargo run -p moebius-core --release --example m4_probe -- NEW_OUT 1
cargo run -p moebius-core --release --example m4_probe -- NEW_OUT 2
# Full-scale exploratory check, original resource configuration:
cargo run -p moebius-core --release --example m4_probe -- NEW_OUT 2 experiments/nursery_256.json ecology no_wave 1 256
cargo run -p moebius-core --release --example m4_mechanics
python3 python/analyze_m4.py
cargo run -p moebius-core --release --example m4_replay -- docs/m4/artifacts/baseline_v1
cargo run -p moebius-core --release --example m4_replay -- docs/m4/artifacts/corrected_v2
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Probe output directories must be new. Compress generated `.jsonl` and `.bin` losslessly with gzip for the saved-artifact layout expected by the replay example. Filenames include environment/controller/seed. No failed trial was discarded; an execution PASS indicates instrumentation/invariants succeeded, **not a viability PASS**. Artifacts occupy about 214 MiB including supplementary controls because every organism-tick, full sensor/actuator vector and paired snapshots were retained.


Final checks: **80 workspace tests pass**, including seven M4 regressions; strict workspace Clippy and formatting pass; `git diff --check` passes. Initial lint failures were repaired (two small equivalent viewer refactors plus explicit actuator-ABI lint annotations); they were not ignored. The independent Python audit checks all 427 saved trials, including the unsuccessful control-isolation condition, and asserts zero commands specifically for the fully fixed controls. Main replay checks 400 trials / 800 snapshots / 90,154 ticks. Full-scale and supplementary trials retain configurations/snapshots and pass per-tick world invariants and independent energy audits, but were not all replayed separately. No live viewer witness, commit, push or hosted CI is claimed.
