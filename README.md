# Möbius Automata

The Ship is in [performance stabilization](docs/morph_005/PERFORMANCE_STABILIZATION.md).
Additional biological complexity is held back; the lag remains unresolved.
See the SIMULATION-0001 [repair history](docs/morph_005/SIMULATION_0001_REPAIR_STATUS.md).
The [coordinate-domain audit](docs/morph_005/COORDINATE_DOMAIN_AUDIT.md) found no
material cost from the enormous empty coordinate envelope.

An artificial-life vivarium with small biomechanical inhabitants. The default
live habitat is the **nursery**: a 256³ periodic cube with fixed edible clots.
Eight recurrent neurons command 26 rotational axes and secretion. Feeding is
authored mouth contact with a nearby clot, not a learned search. Cells can
reproduce, and paid reaction mass can produce recoil and persistent flesh.
At birth, half of a lifetime weight change is written into the child genes.
Useful learning and adaptive evolution have not been demonstrated.

The original deterministic **M0** experiment remains available as
`experiments/m0_64.json`, with its inert genomes and random lattice movement.
Farm rules are opt-in in the core; the live viewer selects them by default.
See [the farm laws and limits](docs/FARM_M1.md), [M0 laws](docs/LAWS.md),
and [snapshot format](docs/SNAPSHOT.md). The new mode's authoritative laws are in
[the neural architecture](docs/neural/ARCHITECTURE.md); see its
[results and limitations](docs/neural/artifacts/main_v2/REPORT.md).
`farm_256.json` retains the legacy stroke-based M1 model;
`neural_farm_256.json` selects the neural mode without nursery clots.
`nursery_256.json` is the 256³ nursery, and `nursery_smoke.json` is the 32³
probe grid. The measured probe is [docs/nursery/PROBE_V1.md](docs/nursery/PROBE_V1.md).
Hypothesis marks remain annotations.

The license is MIT. Copyright 2026 Cognis 11 Inc.

## Build

Rust 1.97.1 or newer. This workspace was developed on 1.99.1.

```bash
cargo test --workspace
cargo run -p moebius-cli --release --bin moebius -- run \
  --seed 20261026 \
  --ticks 1000 \
  --config experiments/m0_64.json \
  --out runs/m0_64
```

`--out` must be missing or empty. The command does not delete an existing directory.

On success the process prints one JSON line and writes:

- `manifest.json` — configuration, seed, PRNG name, tick count, final state hash, residuals
- `metrics.csv` — one row per committed state, including tick 0
- `trails.csv` — live cell sites after each committed state
- `snapshots/snap_TTTTTTTTTT.bin` — tick 0, the final tick, and the interval

The default snapshot interval is 50. The state hash does not include the clock,
the viewer, or a hypothesis file.

## Viewer

```bash
cargo run -p moebius-viewer --release
```

With no arguments the window opens on the 256³ farm. Up and down select a
founding rule. Left and right step it. Shift steps further. A step that fails
validation is refused and the previous value stays. `span` sets width, height,
and depth together. `open_run` keeps ticking until a tick is refused or the
window closes. Enter locks the rules and starts from tick 0.

M1 includes asexual reproduction and inherited controller mutations. M0 recordings
retain their original laws. No collective-organism detector is implemented.

The acceptance experiment is still 64³ and 1000 ticks:

```bash
cargo run -p moebius-viewer --release -- --snapshots runs/m0_64
```

Enter on an unchanged recording watches it. A changed finite rule commits a new
run, then watches it. `--start` with `--snapshots` skips the slate. `--live`
skips the slate and starts the live farm.

| Key | Effect |
| --- | --- |
| Up / Down | Select a rule, before Enter |
| Left / Right | Step that rule. Shift steps further |
| Enter | Lock the rules and start |
| Space | Pause the open clock, or play / pause a recording |
| Left / Right | During a recording, step one snapshot and pause |
| Home / End | First or last tick of a recording |
| 1 2 3 4 | Toggle bodies, nutrient fluid, filaments, hypothesis |
| W A S D | Fly forward, left, backward, right |
| Q / E | Fly down / up |
| Shift | Fly faster |
| Left click / Mouse | Capture the mouse / look around |
| Escape | Release the mouse |
| F | Move close to the nearest living creature |
| N / Tab | Toggle neural inspection / cycle its six pages |
| Mouse wheel | Zoom the lens in / out |
| M | Mute |
| [ / ] | Lower / raise audio volume |

Small biomechanical bodies have rounded head shells, curved ribs, vertebral
plates, soft cores, and looped conduits. Twelve spinal joints and two mandible joints follow committed motor state.
In neural mode, each spine has separate bend/twist effort and each jaw has an
independent command. Internal articulation does not create COM thrust in vacuum.
In a medium, a hard coordinated bend can lunge, and the opposite impulse stays
on the environment ledger. An optional authored traveling spinal wave is retained
for experiments, intensified by hunger; the default M4 nursery disables it.
That drive is not a path to food. Body orientation follows simulation
state. A committed bite gulps the mouth for a short hold. No strand is drawn
from the lips to the clot. The camera starts
near a founding cell and flies freely; it has no simulation body. Broad fill
lighting and depth fog keep the habitat readable.

In the nursery, each food site is one clot. A bite is one unit and credits 4
energy. Maintenance is 1, so feeding pays for life and can fund a birth. The
live mesh is folded tissue and its size follows the remaining stock. A bite
is the mouth gulping, not a strand to the clot.
Movement leaves a lasting flesh thread on the vacated site. Flesh beside a clot
can compost into one nutrient on that clot; other threads stay. The green ribbon
is only recent gloss. Older views without nursery feeding
still draw green lobed **block sums of the nutrient field**. At 256³ that
aggregate stride is 64, and at most 64 clusters are drawn. Wet translucent
movement traces fade over 192 ticks; they are viewer samples. Brown-red flesh deposits are separate,
material-bearing state: secretion creates them and they persist after death.
Neural-mode droplets carry actual material and opposite reaction momentum;
they cure on contact or under the configured medium law.
Face-adjacent deposits weld into connected tissue, displayed as a single smooth
surface. A closing mandible stroke can excavate adjacent tissue for an energy cost.

The simulation runs on a worker thread. Render frames play each committed step
continuously, including the spine, facing, and body centroid. They never advance
the world. The full articulated farm
presentation currently runs in live mode; recorded playback inspects lattice
positions and accounting, without the live joint/feeding/flesh presentation.

Audio is viewer PCM: a softly faded one-second call every eight seconds,
with a quiet default volume of 3/10. Pausing stops sound. Use `[` and `]` to
adjust the volume, or `M` to mute. It is not part of the state hash.

## Python

See [python/README.md](python/README.md). The checker recomputes the ledgers in
a finished run directory. It does not simulate.

## Tests

`cargo test --workspace` covers unique legal occupancy, closed material and
action-energy ledgers, contested movement, reproducibility on a small grid
through 10,000 ticks, rejection of invalid configuration, explicit edge cases,
snapshot restore, observer purity, paid motor impulses, inertial coasting,
reproduction and inherited state, secretion accounting and persistence, contact
with flesh surfaces, and camera input. Extinction is recorded. It is not by
itself a test failure.

Neural tests cover recurrent/oscillatory capability, simultaneous axes, torque
limits, vacuum COM invariance, bounded plasticity, weight assimilation,
mutation, flexible controller mappings, atomic failure and snapshot replay.
They validate mechanisms, not acquired competence.

Reproduce the 112-trial matched neural study in a new directory:

```bash
cargo run -p moebius-cli --release --bin neural-study -- \
  --out runs/my_neural_study --replicates 8 --ticks 256
.venv/bin/python python/analyze_neural_study.py runs/my_neural_study \
  --dest runs/my_neural_analysis
```

The [protocol](docs/neural/STUDY_PROTOCOL.json) preserves the pilot, first
dataset, discovered acceleration-feedback defect and corrected rerun. Random
commands obtained more food than the recurrent controller in the tested foraging
environment. Lineage extinction was 8/8 with mutation and 7/8 without mutation. Mutation produced inherited
differences; adaptive evolution, learned locomotion and learned tunnelling
remain unproven. No failure or null result is hidden.

Scientific searches must be announced beforehand and recorded with provenance,
implementation links and separate consultation/implementation/validation status;
see [AGENTS.md](AGENTS.md) and [the research record](docs/research/2026-11-21-neural-foundation.json).


## M4 diagnostic checkpoint

The default nursery uses neural law 2 with the authored spinal wave disabled.
Historical explicit JSON configs retain law 1; the current explicit config is
[`experiments/nursery_m4_v2.json`](experiments/nursery_m4_v2.json).
Law 2 corrects actual joint-rate reporting, actuator-drive billing, and bounded
finite-difference/force rounding. **Useful foraging is not demonstrated**: all
three-site-food trials starved, while motionless controls ate at founding clots.
See the [M4 report](docs/m4/REPORT.md) for 400 matched trials, three full-scale
spot checks, per-organism traces, independent accounting audits, and exact replay.


## M5 reachability checkpoint

Some second meals are reachable at the existing budget through mouth articulation
and the current capture stencil. This does not establish swimming or production
foraging. The diagnostic oracle reaches 24/96 designed placements; a sensor-only
motor model reaches 8/96 with old rays, 9/96 with the optional local lobes.
The production controller/default sensor remain unchanged. See the
[M5 report](docs/m5/REPORT.md) for all eight verdicts, failed forward-reaching
conditions, paid braking, exact replay, and packed portable evidence.

Raw M5 evidence is archived per experiment on Mermogon; authoritative summaries
and checksum/restore metadata remain in Git. See
[`ARTIFACT_STORAGE.json`](docs/m5/ARTIFACT_STORAGE.json).
`python3 python/m5_artifacts.py unpack NEW_DIRECTORY` restores every file and
verifies its SHA256. Readable protocols/results remain under docs/m5/artifacts/.
Use `--archive-root VERSIONED_DIRECTORY` to restore a verified archive copy on
another machine.

## Anatomical Forge — MORPH-001 and MORPH-002

A separate local Bevy laboratory generates an actual configurable worm from a
versioned JSON definition. Its headless Rust engine produces the body, recessed
mouth, attached flagella and live deformation without Blender. Motion is an
authored preview; simulation binding remains absent.

The laboratory binary now edits MORPH-002 compositions: independent flagella,
spines, and an unbound secretion nozzle. Version1 definitions still load.
The previous 1–7 / F / G key map is no longer in this binary.

```sh
cargo run --release -p moebius-morph-lab --offline
```

Tab selects a component. A, P, and N add a flagellum, spine, or nozzle.
D duplicates. Delete removes. Up/Down chooses a parameter and Left/Right edits it.
`[` `]` change body length. `-` `=` change thickness. `,` `.` set the mouth.
M cycles the mouth. H loads a short showcase. R steps the seed. Space pauses
motion. S saves and L reloads. Right-drag or Alt+arrows orbit; the wheel zooms.
Default save path: `runs/morph_002/organism.json`.

Read [MORPH-002 results](MORPH_002_RESULTS.md) and [MORPH-001 results](MORPH_001_RESULTS.md).
MORPH-001's audit, donor assessment, and architecture notes remain the baseline
record. A portable version1 example is `experiments/morph_001_worm.json`.

## MORPH-004 — the unobserved world

The canonical ship runtime now extends the MORPH-003 free-space substrate,
following the operator's option2 ruling. The lattice `moebius-core::World`
remains a separate preserved nursery experiment. See
[MORPH_004_ARCHITECTURE.md](MORPH_004_ARCHITECTURE.md) and
[MORPH_004_RESULTS.md](MORPH_004_RESULTS.md) for authority, evidence and limits.

```sh
cargo run --release -p moebius-morph --example ship_run --offline -- NEW_OUTPUT 120
cargo run --release -p moebius-morph-lab --bin moebius-world-witness --offline -- --ticks 3600
cargo run --release -p moebius-morph --example morph_004_bench --offline -- NEW_BENCHMARK
```

The headless runner accepts optional initial-snapshot and intervention JSON
paths after the tick count. It records initial/final state and replay inputs.
The witness uses WASD/QE to move its inspection sphere and `-`/`=` for radius.
Its worker advances the explicitly bounded run independently of the window;
closing the window finishes the requested tick target. The default3600 ticks
is about60 simulated seconds. `--record NEW_DIRECTORY` runs repeated boundary
crossings, saves six rendered captures and compares the exact final state to
headless execution. Material persists beyond observation. A completed finite
run stays at its final state; this is not a perpetual background service.

Benchmark organisms prescribe drift, environmental exposure, emission and
conserved material transport. They do not compute metabolism, locomotion,
intelligence or ecological interactions. Geometry is generated only for visible
instances. No volumetric solver, floor or live Lisp/Python authority is added.

## First simulation — food and Arachnitech

Current continuation is **PARTIAL**: own-pipe and stationary secretion contacts
can trap bodies, and authority ticks become very slow. The
[causal diagnosis](docs/morph_005/SIMULATION_0001_COLLISION_DIAGNOSIS.md)
records the reproduced response defects and controlled timing evidence.

Launch the ship witness, press **N**, design the Arachnitech, then click
**Start Simulation** (or press **Enter**). In the creator, **S** saves the
anatomy and **L** reloads it, including after restarting the app.
The new game starts with one Arachnitech and paid Blood-Sacs. A food pouch
stores up to **1024 units** by default; **B** adds a pouch in the creator and its
Capacity parameter changes its limit. Older anatomies gain a pouch when entering
habitat mode; their saved source file is not overwritten. Fullness is visible.

Arachnitech pauses secretion during meals and lays paid snailpipe only while
moving/exploring. Four active nozzles spend four units per emitting tick. A
protected reserve and a short paid founding root prevent immediate emptying.
Blood-Sacs evade nearby mouths, crawl on substrate, and can detach at a tip.
Light and real pipe attachment permit fixation from finite condensate; new
substrate colonies and Arachnitech offspring are paid material transfers.
The authored population limits are eight Arachnitech and 48 Blood-Sacs.

Snailpipes persist as paid graph samples. Their hollow sections store inner and
outer radii and sealed/open end states. Smooth sweeps have wet fresh walls,
subtle ridges and an older hardened appearance. Ordinary segment bores are
continuous; sealed ends remain sealed. Wall contacts encourage route bundles
but **do not open branches**. Open branch bores and gas/liquid transport remain
unimplemented. See the three-property contract and evidence in
[habitat refinement](docs/morph_005/HABITAT_REFINEMENT.md).

To inspect this loop directly with the basic anatomy:

```sh
cargo run --release --offline -p moebius-morph-lab --bin moebius-world-witness -- --first-simulation --ticks 3600
```

A founding group uses the same habitat. Each extra Arachnitech is paid from
the finite condensate. The budget is 16. Home frames the selected body, or
the group when more than one founder is present. Play, pause, single-step,
1x/4x/16x, save (F5) and load (L from the title) do not skip authoritative
ticks. Four times speed runs four ticks, then waits out the same pace.

```sh
cargo run --release --offline -p moebius-morph-lab --bin moebius-world-witness -- --first-simulation --probe-world --founders 4 --seed 77 --ticks 3600
cargo run --release --offline -p moebius-morph --example habitat_probe -- /tmp/NEW_COLONY 420 --seed 77 --founders 1
```

`--abundance` sets the free condensate left after the founding Blood-Sacs are
paid. Movement stays authored. There is no Arachnitech mortality rule, and
Blood-Sac light does not create new biomass.

The HUD shows constructor stock, total eaten, free secretion, bound material,
and balance. Brief uptake lines identify actual committed meals. In the creator,
drag to orbit, wheel to zoom, middle-drag to pan, and Home to frame the body.
Space pauses the simulation, R restarts its initial state, and F5 saves it.
Existing saves retain their configured behavior; use N for a new functional run.

Pursuit, evasion and directional memory are explicit bounded kinematic rules.
Paid reproduction is implemented; learned locomotion, adaptive ecology, mechanical
strength and pipe transport are not established. The run remains explicitly
tick-bounded. Historical first-loop evidence is retained in
[the earlier report](docs/morph_005/FIRST_SIMULATION.md).

Arachnitech's role is snailpipe trails. Wall building is reserved for the second
Automaton species; that species is not implemented here. Winding is geometrical:
strength and load-bearing bonds are not simulated. New habitat runs use canonical
world contact admission, conservative body-part envelopes, and hollow pipe
collision surfaces. The Witness sphere only selects presentation; offscreen
collisions use the same rules. Those contact queries currently miss the 60 Hz
tick. On the measured 420-tick showcase probe the mean tick is 24.9 ms, and
about 96% of collision time is swept distance queries against hollow pipe
meshes. A sample-sphere reject of an already-cached far chunk preserves that
world's digest and leaves the query count at 15645 on a 58-node network.
With the witness open, the same world still spends about 25.5 ms in the
authority tick. Snapshot publish is about 0.007 ms and draw CPU about 1.2 ms.
The witness reuses a mesh whose stamp is unchanged. Mesh budgets 1 and 16
generated the same 80 pipe meshes over 420 ticks. See [collision contracts and limits](docs/morph_005/COLLISION_CONTRACTS.md).
SIMULATION-0001 remains PARTIAL. The clean workspace suite passed on `e863824f13422f5bf83a03c604a79eb85152db62` (212 tests, 0 failures). The colony is still not a steady 60 Hz population.
A founding group of 2 to 16 is measured in
[the playable colony results](SIMULATION_0001_PLAYABLE_COLONY_RESULTS.md).
Sixteen founders averaged 790 ms/tick. Counts of 30, 100, and 300 are refused.
The measured showcase windows produced no Arachnitech birth.

## MORPH-005 — the first filament

```sh
cargo run --release -p moebius-morph-lab --bin moebius-world-witness --offline -- --construction --ticks 3600
```

One prototype follows a recorded3D path, bends its body/flagella, emits finite
material and binds it into persistent filaments. This is authored construction,
not autonomous locomotion or structural mechanics. Blue geometry is backed by
the canonical material ledger; amber points are free secretion.

Use right mouse to look, WASD/QE to fly, Z/X to roll, Shift/Ctrl for speed,
wheel for speed/dolly, Tab for orbit, left drag to orbit and middle drag to pan.
Click to select; F focuses anatomy or filament; Home resets the view. The panel
provides pause, single step, reset,1x/4x/16x, follow/free camera, visibility,
witness radius and a bounded filament mesh budget. Camera controls do not edit
biology. Reset starts a fresh recorded experiment; a completed finite run pauses.

Read [MORPH_005_RESULTS.md](MORPH_005_RESULTS.md) for exact tests, metrics,
physical limits, failed trials and external evidence restoration.

## HABITAT-001 — experimental primordial medium

The HABITAT track establishes a bounded physical environment independently of
organisms. Conventional MORPH-005 stabilization is paused; MORPH-005 remains
PARTIAL and MORPH-006 stays suspended. Canonical ship physics is unchanged.

```sh
cargo run --offline --release -p moebius-morph-lab --features habitat-experiment --bin habitat-witness
cargo run --offline --release -p moebius-morph --features habitat-experiment --example habitat_environment -- suite NEW_OUTPUT
```

This is an isolated isothermal carrier/concentration/inert-sphere experiment,
with explicit approximation and conservation boundaries. Read the
[HABITAT-001 results and reproduction guide](docs/habitat_001/HABITAT-001_RESULTS.md)
before interpreting it as an atmosphere or integrating it into a ship.
