# Dense reference optimization: independent lane B

Original `Chamber::step` remains lane A. Explicit `step_candidate(Buffered,w)` uses
one reusable tentative State, copied from committed State before every attempt;
reuses scalar delta/outgoing/velocity/face arrays and a carrier streaming buffer.
A compact u32 destination/wall table is prepared only when Config changes. The
state's population Vec and streaming Vec swap; after validation committed and
staged State swap. No partial state is published on a fallible operation.

Same collision expression, cell/direction loop, face sort/accumulation, wall
impulse order, source/data validation, low-Mach/Reynolds/CFL and finite-inventory
admission remain. Failure reuses the scratch only after recopying committed data.
Capacity is checked before preparation; late-backend refusal tests staging. Real
process OOM in Rust's infallible allocator is NOT claimed recoverable.

Dense field/population Vec capacities are reused; sparse/block clone allocations,
nonzero iteration collections and validation costs still exist. Working data is
AoS; SoA not implemented. Optional2/4/8-worker collision fills a separate post array;
ordered streaming/wall reduction stays serial and exact. Threads are scoped per
step, not a persistent pool; overhead must be measured and may reject this lane.

Full-state clone was replaced by staged copying/swapping, not removed from the
transaction contract: two physical-state-sized owners still exist. No memory
saving is claimed until measured allocator/retained-state results support it.

## Final measured status

Final B demo mean0.894063ms/P990.921569ms,82.538percent fewer requested bytes than current A.16³/32³ worker controls show no multicore speedup; scoped-thread candidate remains available for reproduction. Accepted bounded single-worker candidate only.
