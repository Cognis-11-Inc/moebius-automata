# Möbius Automata

An artificial-life vivarium with small biomechanical inhabitants. The default
live habitat is the **nursery**: a 256³ periodic cube with fixed edible clots.
Eight recurrent neurons command 26 rotational axes and secretion. Feeding is
authored mouth contact with a nearby clot, not a learned search. Cells can
reproduce, and paid reaction mass can produce recoil and persistent flesh.
Lifetime plasticity is separate from inherited controller genes. Useful
learning and adaptive evolution have not been demonstrated.

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
independent command. Internal articulation does not create COM thrust. Body
orientation follows simulation state; movement has no authored gait. The mouth
also displays actual nutrient uptake. The camera starts
near a founding cell and flies freely; it has no simulation body. Broad fill
lighting and depth fog keep the habitat readable.

In the nursery, each food site is one clot. A bite is one unit and credits 4
energy. Maintenance is 1, so feeding pays for life and can fund a birth. The
live mesh is folded tissue and its size follows the remaining stock. A short
filament is drawn only when a committed tick records uptake at that site.
Movement leaves a lasting flesh thread on the vacated site; the green ribbon
is only recent gloss. Older views without nursery feeding
still draw green lobed **block sums of the nutrient field**. At 256³ that
aggregate stride is 64, and at most 64 clusters are drawn. Wet translucent
movement traces fade over 192 ticks; they are viewer samples. Brown-red flesh deposits are separate,
material-bearing state: secretion creates them and they persist after death.
Neural-mode droplets carry actual material and opposite reaction momentum;
they cure on contact or under the configured medium law.
Face-adjacent deposits weld into connected tissue, displayed as a single smooth
surface. A closing mandible stroke can excavate adjacent tissue for an energy cost.

The simulation runs on a worker thread. Render frames interpolate committed
positions and joints; they never advance the world. The full articulated farm
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
limits, vacuum COM invariance, bounded plasticity, germline separation,
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
