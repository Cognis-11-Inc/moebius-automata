# Dense local scalar blocks: lane C

`Mass::Blocks(BlockMass)` extends the existing scalar representation. Global block
keys are ordered integers; each key owns one contiguous edge³ f64 Vec. Local and
global cell addressing derive from existing Config index/coordinates, not a new
world coordinate owner. Edge4/8/16/32 are measured candidates. Partial edge tiles
have zero padded slots; padding is validated. No tree node per occupied cell.

Exact faces are built from all nonzero cells plus predecessors along each axis,
then Vec sort/dedup restores canonical (cell,axis) order. The same authoritative
flux expression and sorted delta application apply; unallocated scalar cells
mean exactly zero scalar mass. Active destination blocks allocate before receiving
nonzero mass. A block can retire only if every scalar slot equals zero; subnormal
tails remain. Species wall stocks remain separate and cannot be retired.

This bounded candidate retains dense carrier State, dense delta/outgoing/velocity
scratch arrays, a dense destination table and staging copies. It is not a globally
sparse atmosphere or proven enormous-domain architecture. Block serialization is
source-bound within Chamber's checkpoint adapter. No material ledger was added.

Report occupancy, padding waste, support expansion, allocation/retirement and
crossover. Diffusion can populate exact nonzero tails throughout a finite chamber;
there is no threshold. The fastest block size is an outcome, not a preset choice.
