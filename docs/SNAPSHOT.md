# Snapshot and manifest

## State hash

The state hash is the lowercase hex blake3 of the canonical little-endian
payload. It covers the tick, seed, grid, stocks, cursor, live cells sorted by
id, death records sorted by id, and the nutrient array. It does not cover the
code version, the wall clock, hypothesis layers, trail CSV, viewer meshes, or
audio.

## Snapshot file

Magic `MAS1SNAP`, version `u32 = 1`, 32-byte state hash, 32-byte config hash,
`u32` payload length, then the payload. Decode rejects a bad magic, a bad
version, a length that does not consume the file, a hash that does not match
the payload, and a payload that does not re-encode to the same hash. Cell,
death, and nutrient counts above the volume cap are rejected before allocation.

`audit_with_config` checks the config hash, restores a world, rebuilds
occupancy, and audits caps and both residuals. Restore does not advance a tick.

Canonical field order:

- `u64` tick, seed
- `u32` width, height, depth
- `u64` reservoir, waste, external input, external output
- `u64` energy credited, energy maintenance, energy disposed, material initial
- `u64` next cell id, inject cursor
- `u32` cell count, then cells: id, x, y, z, energy, age, 8 genome bytes
- `u32` death count, then deaths: id, x, y, z, energy disposed, age, during tick, 8 genome bytes
- `u32` nutrient count, then `u64` nutrient values in lattice order

## Manifest

`manifest.json` is format `ma-manifest-1`. The config hash is blake3 of the
compact JSON encoding of `SimConfig` in struct field order. The seed and the
tick count are run parameters. They are in the manifest and in the state, and
they are not inside `SimConfig`.

The batch command writes the manifest only after the run succeeds. A nonzero
residual refuses the manifest. Elapsed time and the wall-clock date are not
fields.

## Viewer observations

`trails.csv` is `tick,id,x,y,z` for every live cell after each committed state,
including tick 0. Filaments drawn by the viewer are a window over those rows.
A step whose axis delta is greater than 1 is a wrap or a gap, and it is not
drawn as a segment. That split is not a bond.

Fluid samples are block sums of the nutrient array at a display stride. Their
sum equals the field and their coverage equals the volume. The HUD states the
stride. The samples are not a second stock.

`hypothesis.json`, when present, is `{ "name", "note", "sites": [[x, y, z, weight], ...] }`
with unknown fields rejected. It is an annotation. The viewer can draw it. The
simulation never writes it.

## Farm M1 extension

M0 payload bytes remain unchanged when brain and flesh lists are empty. Farm
snapshots append `FARM2`, a little-endian `u32` byte length, and compact JSON
with ordered `brains` and `flesh` lists. The extension is covered by the state
hash. All controller weights, eligibility traces, bends, twists, jaw and last-action
state, velocity, subsite
position, lineage, reward memory, and permanent flesh are included. A farm with
no living cells still stores its flesh. Unknown extensions, unordered or duplicate
entries, and payloads above 16 MB are rejected. Restore also audits state bounds,
mode compatibility, occupancy, and both ledgers.

Farm manifests identify `canonical-le-v1+FARM2`. The earlier FARM1 extension encodes five joints and 14 actions. It is
explicitly refused by the new rig loader rather than silently reinterpreted.
Its old run artifacts remain unchanged; replay requires the original implementation.
Older binaries cannot decode the new extension. Existing M0 recordings keep their config hashes and state hashes.
The live viewer uses full rig state; recorded viewer playback currently shows
lattice positions and accounting without the live articulated presentation.

## Neural FARM3 extension

`neural` configurations use `FARM3` in the same length-delimited JSON envelope.
It contains controller genotype and separate lifetime neural state (including
integration/plasticity remainders), authoritative body orientation/rates/momentum,
in-flight secretion jets and environment momentum. An explicit mode marker
preserves FARM3 even after extinction. The config hash includes neural laws.
Restore rejects mode mismatch, unsupported anatomy, invalid neural bounds and
nonzero total linear momentum. Jets count toward material stocks.

Empty optional neural/jet/momentum fields are omitted from legacy FARM2 canonical
JSON; M0 and FARM2 byte/hash contracts remain. FARM1 remains intentionally refused.
The eight-neuron/variable-axis schema is versioned; future topology changes must
introduce explicit compatibility handling. The implementation audit and neural
tests cover save/restore and deterministic continuation.

For an existing batch, audit stored snapshots and replay between checkpoints:

```bash
cargo run -p moebius-core --release --example audit_run -- runs/neural_smoke_a
```
