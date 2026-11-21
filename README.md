# Möbius Automata

M0 is a deterministic 3D lattice simulation of digital cells and one material
nutrient field, with a headless runner and a read-only viewer.

A cell is an id, a site, an action-energy balance, an age, and eight inert
genome bytes. The grid is not the organism. M0 does not implement binding,
division, collective heredity, or multicellularity. Hypothesis marks are
annotations. They are not membership. See [docs/BLUEPRINT.md](docs/BLUEPRINT.md),
[docs/LAWS.md](docs/LAWS.md), and [docs/SNAPSHOT.md](docs/SNAPSHOT.md).

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

With no arguments the window opens on the 128³ hall. Up and down select a
founding rule. Left and right step it. Shift steps further. A step that fails
validation is refused and the previous value stays. `span` sets width, height,
and depth together. `open_run` keeps ticking until a tick is refused or the
window closes. Enter locks the rules and starts from tick 0.

M0 does not divide, mutate, or detect an organism. A longer run does not create
one. Founding cells are the initial population. They are not a membership.

The acceptance experiment is still 64³ and 1000 ticks:

```bash
cargo run -p moebius-viewer --release -- --snapshots runs/m0_64
```

Enter on an unchanged recording watches it. A changed finite rule commits a new
run, then watches it. `--start` with `--snapshots` skips the slate. `--live`
skips the slate and ticks the open hall.

| Key | Effect |
| --- | --- |
| Up / Down | Select a rule, before Enter |
| Left / Right | Step that rule. Shift steps further |
| Enter | Lock the rules and start |
| Space | Pause the open clock, or play / pause a recording |
| Left / Right | During a recording, step one snapshot and pause |
| Home / End | First or last tick of a recording |
| 1 2 3 4 | Toggle bodies, nutrient fluid, filaments, hypothesis |
| J / L | Yaw |
| I / K | Pitch |
| - / = | Zoom |
| M | Mute |

Bodies are a Giger bone-and-oil costume on committed cells, with ribs, pipes,
and fangs. They are not organism membership and they are not in the state hash.
The hall camera starts inside the tank. Fog hides the far cage. Both are
costume. The picture glides between committed positions and does not stretch a
periodic wrap. Fluid orbs are sums of the nutrient field. Thin blocks stay
hidden. On 64³ the stride is 16. On 128³ it is 32. At most 64 orbs.
Filaments are the last committed steps and break on a wrap. Hypothesis marks
stay off until `4`. Render frames do not advance the simulation. The open hall
ticks on a worker thread.

Audio is viewer PCM. It is not part of the state hash.

## Python

See [python/README.md](python/README.md). The checker recomputes the ledgers in
a finished run directory. It does not simulate.

## Tests

`cargo test --workspace` covers unique legal occupancy, closed material and
action-energy ledgers, contested movement, reproducibility on a small grid
through 10,000 ticks, rejection of invalid configuration, explicit edge cases,
snapshot restore, and observer purity. Extinction is recorded. It is not by
itself a test failure.
