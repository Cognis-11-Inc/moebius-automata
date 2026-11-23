# M6 — the motor nursery (2026-11-22)

The existing recurrent controller can be genetically optimized to sustain COM translation in this simulator. The selected genome travels a mean 1.718 sites in 256 ticks versus 0.159 for the matched factory genome, and passes the registered sustained-motion gate in 8/8 held-out starts. This is evolutionary optimization across candidate genomes. Useful individual lifetime motor learning, cue-driven steering and ecological transfer are **not demonstrated**.

## Acceptance gates

| Gate | Verdict | Evidence boundary |
|---|---|---|
| M6-G1 physical capability | PASS | Paid periodic mechanical reference sustains travel in 6/6 orientation/medium conditions; not a learned policy |
| M6-G2 exploratory activity | PASS | 120 babble trials, actual joint responses; 2 randomly generated simultaneous trials pass movement gate |
| M6-G3 optimization | PASS |8/8 paired wins; median selected advantage 1.484 sites, well above registered 0.25 site threshold |
| M6-G4 sustained COM locomotion | PASS |8/8 selected frozen trials, last-quarter progress and advantage over matched zero-drive continuation; no jets |
| M6-G5 directional control | FAIL |Corrected cue pilot 0/8 held-out direction-change successes; not a physical impossibility result |
| M6-G6 held-out robustness | PASS |8 fixed unfamiliar orientation/pose starts, balanced drag 8/32; one optimization seed only |
| M6-G7 individual lifetime learning | FAIL — NOT DEMONSTRATED |Plastic weights change; neither genome shows useful retained competence gain beyond controls |
| M6-G8 ecological transfer | NOT RUN |Curriculum blocked at direction changes; no claim about second meals or survival |
| M6-G9 replay/lawful accounting | PASS |Full 1724 episodes / 253440 primary ticks replay exactly, including invalid cue trials; independent arithmetic and verified external restore |

Stages A/B pass. C fails its bounded criterion. D integrated braking/reorientation/recovery, E exploration, and F ecological transfer were not run. Unfamiliar-pose movement probes are not a substitute for the integrated recovery task.

## Baseline, physical mechanism and budget

Read [integration audit](AUDIT.md), [preregistered protocol and chronological amendments](PROTOCOL.json), [measurements/statistics](ANALYSIS.json), [validation](VALIDATION.json), and [storage/restore metadata](ARTIFACT_STORAGE.json). Base is `8e351e2`, with M4/M5 committed evidence preserved. No external scientific publications were consulted. All claims here concern an authored integer simulation, not validated animal biomechanics.

No anatomy, body mechanics, energy/resource coefficients or production nursery defaults changed. One original 14-joint organism receives 4096 initial energy in an otherwise empty 16³ periodic environment, population cap 1, no uptake objective/reproduction. Resets are provisioned assays, not biological survival. Optimization uses native eight-neuron recurrent ticks and 107 ordinary scalar inputs, without global target coordinates, clock-indexed gait or evaluator fitness in its inputs. External actuator patterns occur only in diagnostic/babble/mechanical/coast controls.

Capability: the prior M5 periodic reference travels 4.400–4.433 sites at drag 8 and 5.644–5.672 at drag 32 over 256 ticks across three starting orientations/poses. All 6 pass the sustained criterion. A single initial pulse travels roughly 2 sites but has zero advantage over its matched last-half zero-drive continuation, so it fails. Factory activity is mostly joint flicker, spring relaxation and insufficient effective drive. Rotation is measured, but independent rotational control, reversal and active stopping are not established by this envelope.

Motor babbling comprises 120 episodes × 128 ticks: individual contractions, positive/negative reversals, simultaneous random contractions, seeded sequential permutations and independent random phases/amplitudes/durations, across three conditions. Exactly 2 pass the motion gate: `babble_402_103_BabbleSimultaneous` (net 0.757 site) and `babble_407_103_BabbleSimultaneous` (0.834). Their exact per-tick submitted commands and outcomes survive in the archive. They were not distilled into NN weights. Plasticity was enabled during babbling, but this alone does not demonstrate acquisition.

The bounded movement optimizer evaluates 16 candidates × 12 generations × 3 conditions × 128 ticks, 576 episodes; seed 60620261122, three training starts 101–103, frozen plastic weights. Each generation retains 4 elites, introduces 4 independently randomized immigrants and mutates 8 selected offspring. All 16 candidates per generation have distinct IDs; full parents, mutation loci, genomes and pairwise parameter diversity are preserved. Only supported NN parameters vary. Body genes and valve/nozzle rows remain fixed. Best training score rises from 0.832 in generation 0 to 1.652 in generation 11; selection is not based on held-out scores.

Fitness records net/path, quarter projections, rotation, actual joint participation, fees and pending fractional debt separately. Its movement scalar is `net +4*positive_min_last2_quarters +2*positive_active_gain +.25*net/(1+effective_motor_cost) -.001*effective_motor_cost -.002*rotation`. Near-zero travel receives no efficiency credit and energy/distance is null. Active gain compares the actual last half with a cloned midpoint continued using zero raw commands through normal muscle relaxation/fees/physics. Sustained requires net ≥ 0.5 site, both final quarters ≥ 0.10 site projected progress, active gain ≥ 0.05 site, late active drive and zero jets. Mouth extension receives no COM score.

Locked movement genome: `204546b9001fcfb7a0b1397abe47c91ee1aadf61603c8ae1dd97a9178be3a05c`. Factory genome: `dcc63a2efe8458e55be40460ce4ba9b70f4454bea48a878869ef95a3189ee362`. Small exact copies are under `checkpoints/`. The directional checkpoint is retained externally and is not substituted into the movement demo.

## Locked held-out comparison

Eight seeds 601–608 were opened after selection, 256 ticks each, unfamiliar bounded poses/orientations, drag 8/32. Distances are sites; cost is actuator energy only, with pending fractional cost retained in raw measurements. All controls have identical starts/energy within seed.

| Controller | Mean net [range] | Mean path | Mean active gain | Paid motor energy | Sustained |
|---|---:|---:|---:|---:|---:|
| selected_frozen | 1.718 [1.322, 2.264] | 1.902 | 0.538 | 95.5 | 8/8 |
| selected_live | 1.637 [1.290, 2.257] | 1.840 | 0.445 | 96.0 | 8/8 |
| original_frozen | 0.159 [0.000, 0.361] | 0.193 | 0.037 | 32.6 | 0/8 |
| original_live | 0.184 [0.000, 0.361] | 0.218 | 0.040 | 32.6 | 0/8 |
| random | 0.306 [0.091, 0.568] | 0.420 | 0.063 | 57.1 | 2/8 |
| rest | 0.000 [0.000, 0.000] | 0.000 | 0.000 | 0.0 | 0/8 |
| pulse | 1.911 [1.514, 2.321] | 2.043 | 0.000 | 12.0 | 0/8 |
| periodic | 5.038 [4.377, 5.660] | 5.082 | 1.761 | 36.0 | 8/8 |
| selected_no_afferent | 0.360 [0.000, 0.615] | 0.364 | 0.150 | 92.0 | 3/8 |
| selected_feedback_off | 1.564 [1.129, 2.169] | 1.770 | 0.399 | 96.0 | 8/8 |
| selected_recurrence_off | 0.299 [0.000, 0.468] | 0.335 | 0.032 | 104.8 | 0/8 |

Selected minus original frozen: mean +1.559 sites, median +1.484, range +1.319 to +1.902, wins 8/8. Paired bootstrap 95% interval for the mean is [1.407, 1.720] sites (10000 deterministic resamples). This summarizes these eight starts conditional on **one** optimizer seed; it is not uncertainty over independent optimization campaigns. Selected motor energy/distance ranges 42.53–73.16, and total 256 tick expenditure is 383–384 including maintenance and neural fees. The provisioned budget is substantially above ecological 128 energy. The fixed mechanical reference remains much faster: this does not establish optimal or efficient swimming.

### What causes the improvement, and what remains uncertain

All 26 joint axes actually change in selected trials. Raw command ranges, per-axis changes, rates, efforts, complete neural states and fractional motion are recorded. Eliminating afferent connections reduces mean net to 0.360 and removing recurrent connections to 0.299; the latter retains leaky neural integration and is not the same as disabling all temporal state. Live plasticity does not explain the frozen selected advantage. Feedback-off also removes the physiological signal from an ordinary sensory channel; it must not be described as a pure weight-update intervention.

Post-selection mechanistic probes were registered separately and were never used for retuning: vacuum motion is exactly 0 in 8/8; bias-only joint outputs average 0.127 site and pass 0/8; a fixed shuffle of joint output rows/biases averages 1.085 sites and still passes 8/8. Thus dynamic, medium-coupled actuation is necessary in these probes, but the particular motor mapping is not uniquely necessary. This is weaker evidence than a discovered anatomical gait with indispensable joint-specific phases.

The body law contains both anisotropic shape drag and an authored collective-bend lunge threshold 280. Corrected analysis counts 19–30 final-half threshold crossings per selected trial. A separate compiled Rust decomposition, [MECHANISM.json](MECHANISM.json), reproduces the integer lunge formula with the public rotation function: 1–24 late ticks actually deliver nonzero lunge impulses (rounding suppresses others). Inferred shape impulses occur on 111–128 late ticks. Both mechanisms contribute; shape accounts for most summed absolute component impulse in these records, which is not a causal allocation of displacement or energy. This is authored coarse coupling, not a fluid simulation.

**Analysis correction:** the initial Python filter used `bend` while the saved schema says `Bend`; it falsely reported zero late threshold crossings and led to a false shape-only explanation. The report and independent auditor are corrected, a schema assertion prevents silent recurrence, and Rust/Python threshold counts are cross-checked. Superseded analysis, report and source remain in the immutable v1 evidence and v2 correction addendum. Native COM, fitness, accounting and replay results are unaffected.

Integer drag truncation leaves small residual momentum; at drag 8, absolute momentum below 128 can escape damping, with velocity quantized by inertia 16. Simple pulse/coast motion is a real confound and is explicitly rejected by the active-gain gate. Selected advantage over zero-drive continuation is 0.187–0.890 site, zero vacuum travel, paid ongoing actuation and matched ablations rule out **only** an initial free impulse/coast explanation. These tests do not prove resolution-independent locomotion: no finer fixed-point/continuum law was tested. Rounding may influence the efficiency and stability of shape coupling. Total momentum remains balanced, not created for free.

## Lifetime plasticity

Each genome has a 512 tick continuous experience episode under plasticity-on, plasticity-off and feedback-off. A matched untrained arm receives no experience. Pre and post probes clone matched physical starts and 4096 energy. Frozen post probes reset fast activation, eligibility, muscle and body state while retaining the experienced plastic weight arrays; further updates and physiological feedback are disabled. Separate carry-fast probes retain neural/muscle state, and fresh inherited probes use the existing half-assimilation offspring routine with mutation off. Neither reset assay nor inherited-parameter probe represents actual birth or survival.

The table uses eight unfamiliar 128 tick frozen probes, not the longer primary 256 tick benchmark.

| Genome | Nonzero changed synapses | Pre mean net | Post-minus-pre | Paired mean 95% interval | Positive starts |
|---|---:|---:|---:|---:|---:|
| original | 38 | 0.074837 | +0.011510 | [-0.001540, +0.028472] | 4/8 |
| selected | 113 | 0.618827 | -0.003608 | [-0.018563, +0.010590] | 4/8 |

All plasticity-off, feedback-off and untrained frozen post-minus-pre differences are exactly 0. Original plasticity yields a small retained numerical change, with 4/8 positive starts and no sustained locomotion. Selected plasticity yields a small negative mean; it has the same 4/8 sustained 128 tick probes as its no-learning controls. Both intervals include 0. Full carry-state and inherited results are preserved in ANALYSIS.json; carry-state can change short transients and carries incoming fee debt, so its absolute final debt must not be interpreted as newly incurred cost. Neither synaptic change nor assimilation demonstrates useful individual motor learning.

The existing feedback is clipped energy change, dominated by maintenance and actuation costs in a food-free world. It contains no task-aligned locomotion consequence beyond those costs; eligibility is not enough to guarantee better motor coordination. The smallest next **proposed, unimplemented** test is a separately versioned, opt-in plasticity experiment using a bounded scalar derived from ordinary local velocity/rotation/effort observations, with matched delayed/shuffled-feedback controls. Keep the physiological sensory channel constant across update ablations, preregister retained held-out competence criteria, and compare against energy-only plasticity. That would test a new authored learning signal; it would not validate biological learning.

## Directional null and honest chronology

After sustained-motion gates passed, a second pilot used 12 candidates × 6 generations × 3 conditions × 128 ticks, optimizer seed 60620261123 and the movement-selected warm start. Evaluator goals remain outside the NN. Direction is represented by an explicit authoritative nutrient gradient seen through existing radius 8 LocalLobesV1; uptake is 0. At 64 training / 128 test ticks, the field is rearranged to reverse its gradient while conserving its current material sum, with no body position, velocity, momentum or energy edit. This is an imposed environmental cue, not simulated chemical transport. Fitness rewards signed COM progress in both phases, with cost/rotation penalties and a -1000 score for jets.

**The first cue pilot was run before interface validation.** We then caught all six channels saturated at 1024: field weights 64+8*coordinate, total 507904. Both 216 optimization and 48 held-out v1 episodes are preserved as INVALID CUE, not evidence that a meaningful directional cue could not be learned. No data was erased. The error was disclosed before the rerun.

V2 changes only cue concentration to 4+coordinate, total 47104. The sensor/material/no-impulse regression passed **before** repeating the same bounded campaign. Aligned positive-Y lobes initially read 538 versus 283 and reverse order after cue reversal. This validates the interface in that fixture, not adequate observability at every arbitrary pose and periodic-gradient seam.

The corrected candidate passes 0/8 fresh 701–708 held-out cases (both phases must advance ≥ 0.10 site and no jets). Mean signed first/second-phase progress is -0.0227/+0.0142 site. The movement-selected checkpoint also passes 0/8; so do the factory, random, rest and periodic controls. The random directional control secretes reaction-mass jets and gets no joint-locomotion credit. Its waste exports/digging are lawful recorded material flows, not extra free energy. Cue coasts retain the old field on the diagnostic clone, so their active-gain values are not matched-environment evidence for steering; the directional gate uses direct phase progress instead.

This is a bounded training null. It cannot distinguish insufficient search budget/capacity from a poor directional objective or cue geometry, and does not establish physical steering impossibility. The direction fitness can prefer reduced motion over strongly wrong-way motion; the selected directional checkpoint is slower than the movement checkpoint. We stop rather than promote that into competence. Integrated braking, exploration and ecological feeding transfer remain NOT RUN. M5 second-meal conclusions are unchanged.

## Visual witness and preservation

[Demo instructions](README.md) launch the optional selected/untrained native viewer. IDs 1/2 are identified in the HUD; solo modes expose their neural states. Actual saved PNG pixels were inspected for both solo modes and the comparison. Selected activations vary and velocity is nonzero while factory activations mostly settle and velocity is zero in the inspected 128 tick frames. Body articulation and orientation are visible. Comparison frames use a common fixed camera; no procedural gait was added. Stills are supporting witnesses, not a substitute for COM telemetry. The requested tick 1 capture is blank because render assets had not become ready; it is retained as an unsuccessful witness, and later captures were inspected instead. Capture ticks are lower bounds and render interpolation may lag the committed state slightly.

All raw full-state traces, candidate lineages, failed candidates, invalid cue trials, experienced states, screenshots, logs and protocol copies live in immutable versioned Mermogon archives, with SHA256 member/archive/restore verification. Reports, small selected/original checkpoint metadata, source/config identities, manifests and reproducibility tools remain in Git. Working caches were retained. See ARTIFACT_STORAGE.json for exact counts/bytes/paths and VALIDATION.json for test and restore evidence. No commit or push was performed for M6.
