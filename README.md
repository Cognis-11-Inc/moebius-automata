# Möbius Automata

An artificial-life vivarium with small biomechanical inhabitants. The default
**Farm M1** runs in a 256³ periodic cube: adaptive motor controllers drive
articulated bodies, strokes produce thrust and coasting, cells feed and reproduce,
and paid secretion leaves persistent flesh in the environment.

The original deterministic **M0** experiment remains available as
`experiments/m0_64.json`, with its inert genomes and random lattice movement.
Farm rules are opt-in in the core; the live viewer selects them by default.
See [the farm laws and limits](docs/FARM_M1.md), [M0 laws](docs/LAWS.md),
and [snapshot format](docs/SNAPSHOT.md). Hypothesis marks remain annotations.

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
| Mouse wheel | Zoom the lens in / out |
| M | Mute |
| [ / ] | Lower / raise audio volume |

Small biomechanical bodies have rounded head shells, curved ribs, vertebral
plates, soft cores, and looped conduits. Twelve spinal joints and two mandible joints follow committed motor state.
Front and rear sections have separate stroke choices and wide bending/twisting ranges. The mouth contracts when nutrient uptake occurs. The camera starts
near a founding cell and flies freely; it has no simulation body. Broad fill
lighting and depth fog keep the habitat readable.

Green lobed food colonies are **block sums of the nutrient field**, not discrete
physical food balls. At 256³ the display stride is 64; at most 64 clusters are
drawn. Cells eat local field nutrient. Wet translucent movement traces fade over
192 ticks; they are viewer samples. Brown-red flesh deposits are separate,
material-bearing state: secretion creates them and they persist after death.
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
