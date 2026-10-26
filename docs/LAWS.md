# M0 laws

These are the rules the Rust core commits. A passing test is evidence that a
build followed them. This page is not itself that evidence.

A digital cell is an id, a site, an action-energy balance, an age, and eight
inert genome bytes. It is not a lattice site. The lattice is a finite
64-axis-capped 3D grid with one material nutrient field. The default experiment
is 64×64×64. Axes are `1..=64`. Volume is `1..=262144`.

Neighborhood: 6 face neighbors plus stay (`von-neumann-6-periodic`). Coordinates
wrap with `rem_euclid`. Only periodic boundaries are implemented.

Index: `x + width * (y + height * z)`, x fastest.

## Stocks

Material is a `u64` ledger:

`material_initial + external_input = field + reservoir + waste + external_output`

Action-energy is a separate integer ledger. Uptake removes 1 nutrient and adds
1 waste, and may credit `energy_per_nutrient` action-energy. Death does not
return action-energy to material. The manifest field
`action_energy_is_conserved_physical_quantity` is false.

`energy_credited = energy_live + energy_maintenance + energy_disposed`

Seed energy is credited at genesis, before tick receipts. Both residuals must
be 0 or the tick is not committed.

## One tick

The tick clones the world, advances the clone, audits it, and only then swaps
it in. A rejected tick leaves the caller unchanged.

1. Add `inflow_per_tick` to the reservoir and to `external_input`. If the
   reservoir would exceed `reservoir_cap`, the tick errors and does not commit.
2. Inject `min(inject_per_tick, reservoir)` by walking at most one full sweep
   from `inject_cursor`. A visited site gains at most one unit, and only when
   it is below `site_cap`. Unplaced budget stays in the reservoir. The receipt
   records the amount that actually moved.
3. Each live cell draws a movement direction from its own stream.
4. Resolve movement on the pre-move occupancy. See below.
5. Uptake, in ascending cell id, at the post-move site. A unit is taken only
   when the whole energy credit fits under `energy_cap`. No partial credit.
   If `energy_per_nutrient` is 0, nutrient still becomes waste and no energy is
   credited.
6. Maintenance is all or nothing. A cell with energy greater than or equal to
   `maintenance_cost` pays it and its age increases by 1. Otherwise it is
   removed, its remaining action-energy is disposed, and a death record stores
   the pre-increment tick. Cell id 0 is illegal. Dead ids are not reused.
7. Export `min(export_per_tick, waste)` from waste to `external_output`.
8. Increment the tick index.

`next_cell_id` does not increase in M0. There is no division.

## Movement

Directions are stay, +x, -x, +y, -y, +z, -z.

A cell may enter a site only when that site is empty in the pre-move snapshot.
An occupied destination is refused even if its occupant is leaving, so there
are no swaps and no displacement chains. If several cells propose the same
empty site, the lowest cell id wins. Losers stay. Origins must be unique.
Outcomes are applied in id order.

## Random streams

Generator: ChaCha12 from `rand_chacha` 0.9. Each stream seed is blake3 of the
label `moebius-automata/prng/v1`, the master seed, the stream label, the tick,
and the cell id.

Placement uses one stream labeled `placement` at tick 0 and cell id 0. Sites
are shuffled with Fisher-Yates. `GenomeFill::Derived` then draws eight genome
bytes. `GenomeFill::Zero` writes eight zero bytes and does not draw them. The
shuffle still runs. Dynamics do not read the genome. The state hash includes it.

Movement uses an independent stream per `(seed, tick, cell_id)` labeled
`movement`. The direction is the first unbiased residue modulo 7, by rejection
sampling. One cell's draw does not depend on another cell having been visited.

## What fails instead of clamping

Configuration rejects an axis or volume outside the caps, a population above
the volume, a nutrient peak above `site_cap`, an initial energy above
`energy_cap`, an initial reservoir above `reservoir_cap`, `inject_per_tick`
above the volume, unknown JSON fields, and negative integers. A run whose
worst-case reservoir (`initial_reservoir + inflow_per_tick * ticks`, assuming
injection removes nothing), or whose material or energy counters, cannot fit
is rejected before the first tick. Snapshot interval and trail-row budgets are
rejected rather than subsampled. Snapshot files whose counts exceed the volume
are rejected before those counts are allocated.

Tick horizon for a run is at most 100,000. Snapshot count is at most 48.
Trail rows are at most 200,000.

## Layers

Trails, fluid block sums, and hypothesis marks are observations. They are not
written by the tick and they are not part of the state hash. A hypothesis file
may name sites. M0 has no organism detector and does not store membership in
the world. See `docs/BLUEPRINT.md`.
