# Runtime query frontiers — VI contract

2026-11-26. Implemented in `collision_execution.rs` and canonical collision call
sites. No worker is a world, material, history, geometry-admission or collision
response authority. No external scientific sources were consulted.

The caller owns a persistent Executor through CollisionCache's ContactWorkspace.
Policy is explicit runtime machinery and is not serialized as a physical law.
Workers are 2..32, command/result channels have capacity one per worker, and a
synchronous exclusive owner allows one active batch. A frontier has at most 64
descriptors/results. All tasks finish before consumption; no work accumulates
between ticks. One-worker policy uses the existing sequential path without a
pool. Worker creation failure leaves the previous configuration intact; shutdown
closes ports before joining threads. Numerical panic resets that worker's scratch
and invalidates its receipt; closed ports/missing receipts fall back canonically.

Each descriptor binds context (pre-step tick, solver pass, body ID, target kind
and ID), canonical ordinal, component ID, two immutable shape incarnations,
radius bits, all four original integer endpoint poses, and query parameters.
Contact parameters additionally bind both sampled floating-point poses bitwise,
sample time and the unchanged 0.001 prediction. Cast parameters reference the
unchanged canonical cast kernel and its compiled constants.

Geometry revisions use allocation identities of immutable Arc-owned shapes,
held alive by the frontier through discard. They are capabilities, not persisted
hashes. A changed canonical pipe key constructs a fresh shape, including same-tick
node edits; a body definition/quantity change likewise replaces geometry. Shape
ownership prevents address reuse while a receipt exists. This relies on the
existing tested geometry-key/invalidation boundary; workers cannot edit meshes.
The diagnostic geometry recorder separately retains complete revision keys and
collider fingerprints. No approximate pose matching or source/history bypass is
used for receipt admission.

Receipts add batch generation, worker identity and health. Results are placed at
their declared ordinal independently of completion order. At each original
consumption point the caller constructs the expected key from current inputs.
Every field, generation, worker bound, health and result kind must match. Otherwise
the value is discarded and the original canonical query runs at current inputs.
Stale values cannot set targets, contacts, events or material records. Generation
exhaustion and oversized batches bypass dispatch instead of reusing a generation.

The original sequential comparison, short-circuit and error order are retained.
A healthy numerical error becomes visible only if its matching descriptor is
actually reached. Unreached errors are discarded with other unnecessary results.
Broken-worker errors are never substituted for the canonical numerical answer.
Transaction rollback remains ShipRuntime's original provisional-state commit.

Source graft: independently evaluate non-outlet component casts, consume in the
original component order and stop at the first obstruction/error. This lane is
unprofitable in the measured endurance continuation and requires the explicit
`experimental-collision-frontiers` feature to select.

Recovery: keep stationary/rotation refusal, initial contact and inward-normal
check sequential. Compute the original 0.25/0.5/0.75/1 samples at fixed poses;
consume depths and nondeepening comparisons in original order. Existing exact
contact memo hits bypass worker computation. Validated raw relative contacts are
memoized at the caller, then transformed exactly once as in the canonical query.
This prevents speculative work from replacing cheap memo hits with recomputation.
Four-worker recovery is the candidate advancing to full qualification; it is
explicit opt-in, with a sequential reference still available.

Body-static: bounded per-body windows bind the current target, with exact stale
fallback after slides. Existing cast memo hits bypass dispatch; only consumed,
authenticated successful results enter the existing memo. This lane is retained
under `experimental-collision-frontiers` after measured regression, including the
first version that missed canonical memo reuse. It is not a production default.
No body-body frontier or parallel geometry preparation was forced into the
runtime after the measured earlier seams failed to show sufficient benefit.

Diagnostics report eligible, acknowledged dispatched, consumed, stale, unnecessary,
recalculated, failure and capacity-bypass counts; frontier-size histograms; worker
query elapsed time; support-entry high water; dispatch/wait and canonical fallback
time. The synchronization upper estimate subtracts the busiest worker's query
elapsed time from total dispatch/wait. It includes collection and scheduling;
it is not an isolated lock-cost or OS-CPU measurement. Support entries are not
workspace bytes. Existing parent collision counters do not include all worker
kernel counters; use executor counters rather than silently adding them together.

Falsification includes changed poses, geometry incarnations, target/context,
parameters, radii/components, incorrect ordering/generation/worker/result kind;
real same-tick pipe mutation; unreachable speculative error; worker panic/closed
ports; actual bounded command-queue saturation; capacity overflow; exact runtime
events and HistoryBudget rollback. Honest-red removes key equality and the paid
geometry revision check independently, producing failures before restored green.

Geometry handoff is separate. Deposit preflight may hold at most two chunk
recipes. Only successful Assembly::deposit, a matching paid binding/node and the
complete actual pipe key may promote a candidate. Rejections clear pending state;
later mutations miss the ordinary cache key. Promoted tracking is capped at 128
entries, and existing horizon ownership still prunes pipe caches. The congested
attribution found no matching preflight/pipe construction pairs and no consumed
handoffs there. This unprofitable lane requires `experimental-geometry-reuse`;
it does not authorize an alternative mesh, ledger mutation or private-library
refit. Canonical hollow geometry, orientation and pseudo-normal construction stay.
