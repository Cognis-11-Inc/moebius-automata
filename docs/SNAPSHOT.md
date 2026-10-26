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
