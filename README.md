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
cargo run -p moebius-viewer --release -- --snapshots runs/m0_64
```

Playback shows committed snapshots. Render frames do not advance the simulation.

| Key | Effect |
| --- | --- |
| Space | Play or pause |
| Left / Right | Step one snapshot and pause |
| Home / End | First or last snapshot |
| 1 2 3 4 | Toggle cells, nutrient fluid, filaments, hypothesis |
| J / L | Yaw |
| I / K | Pitch |
| - / = | Zoom |
| M | Mute |

Cells are brighter when their action-energy is a larger fraction of `energy_cap`.
Fluid blocks are sums of the nutrient field at a stride chosen so at most 512
blocks are drawn. On 64³ the stride is 8. The HUD says so. Filaments use
`trails.csv`, not the gaps between snapshots, and they break on a periodic wrap.
Hypothesis marks stay off until `4` is pressed. If `hypothesis.json` is absent,
the layer is the empty M0 scaffold.

Audio is a one-second WAV synthesized from the displayed tick, live count, dead
count, field sum, and waste. It uses `f32` and is not part of the state hash.

## Python

See [python/README.md](python/README.md). The checker recomputes the ledgers in
a finished run directory. It does not simulate.

## Tests

`cargo test --workspace` covers unique legal occupancy, closed material and
action-energy ledgers, contested movement, reproducibility on a small grid
through 10,000 ticks, rejection of invalid configuration, explicit edge cases,
snapshot restore, and observer purity. Extinction is recorded. It is not by
itself a test failure.
