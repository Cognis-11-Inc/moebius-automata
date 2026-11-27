# Sparse transport attribution

Source `primordial/transport.rs::step_inner` retains the original equations and
face order. `Attribution` optionally times velocity extraction, face discovery,
delta/outgoing allocation, flux, wall exchange, CFL and apply phases. Nested leaf
timers cover ordered-set insertion, neighbor calculation, lookup, delta/outgoing
entry accumulation and insertion/removal. Phase gross and nested leaf time must
NEVER be added. Exclusive remainder is gross minus nested leaves and includes
instrumentation gaps. Individual clock overhead is not subtracted. This intrusive
path is separate from ordinary step timing; it executes on a cloned diagnostic
state, never commits it. Original dense and sparse physical results must agree.

Scalar support/slots, transported faces, requested/live/peak bytes and RSS are
recorded each tick. `allocation-attribution` uses the existing allocator monitor
factored into `examples/support/habitat_alloc.rs`, not a second measurement
service. Enclosing clone/validation/carrier/transport costs come from StepProfile.
Alloc/dealloc API costs are included in phases; standalone allocator latency is
not isolated. `perf` is absent, so cache misses/hits are NOT MEASURED. No cache
hardware-counter inference follows from wall-clock time. BTreeSet and cell BTreeMap
are hypotheses until the paired phase/occupancy records are analyzed.

Final numerical attribution and crossover tables: HABITAT-002_RESULTS.json and
HABITAT-002_SCALING.json after clean-source qualification. Failure/nulls retained.
