# MORPH-005 — canonical material assembly and independent navigation

2026-11-23. Preimplementation reuse ruling. Base main d6fa1ac41ec73f7c6ab0372032d32fe6faf0c047,
clean tree; fetch verified HEAD/origin/main0/0. Final M4 runtime certification
78f1f44; d6fa1ac is its published continuity-only follow-up. User's005 directive
and mandatory navigation addendum authorize the following narrow extension,
local clean-commit certification, and no push/006.

## Cage and exclusions

A strand mesh must never create material. A camera must never own construction.
Keep ShipRuntime as sole ship authority. Preserve nursery, old snapshots and old
M3/M4 outcomes. No mechanics, equilibrium, curing, force-bearing, pipe solver,
metabolism, learning, reproduction, live Lisp/Python service, global dense grid
or unbounded histories. All failed/null/protocol-defective runs remain evidence.

## Functional archaeology and reuse

| Concern | Path/symbol | Status / producer -> consumer | Authority/tests | Reuse / missing seam |
|---|---|---|---|---|
| Material admission/integration | morph/emission.rs MaterialBook::{try_emit,step,residual,save,load},Parcel,Ledger | active canonical substrate; finite reservoir -> admitted free parcels | delegated by ShipRuntime; emission tests | extend ledger with bound parcels and explicit atomic transfer; no renderer debit |
| Ship world | morph/runtime.rs ShipRuntime,WorldSnapshot,Operation,Event | canonical owner; typed intervention/clock -> snapshots | runtime tests, M4 exact A/B/C | extend within owner; structural graph and operation currently absent |
| Nozzle geometry | morph/anatomy.rs CompiledAnatomy::pose/OutputFrame; emission.rs place_output/RigidPose | active authored geometry -> world nozzle | geometry/attachment/transform tests | reuse actual animated outputs for005; fixed-rest M4 remains unchanged |
| Authored path | emission.rs AuthoredTrajectory::at | explicit integer trajectory -> rigid pose | M3 transform tests | recorded sample sequence of same RigidPose supports curved3D routes, no autonomous movement |
| Nursery jets/terrain | core/world.rs advance_jets/SECRETE_ACTION, deposited flesh | active separate nursery experiment | M4/M5/neural/terrain tests | not ship filaments; preserve without adapter |
| Historical trails | core/layers.rs TrailPoint/split_trail; morph-lab previews | observation/line presentation, explicitly not bonds | projection tests | reject as material/topology owner |
| Observer/worker | morph/witness.rs WitnessSphere,PresentationCache,Worker | immutable snapshot -> visible IDs; independent owner thread | M4 neutrality/lifetime tests | extend same worker with explicit playback commands and bounded numerical rate; camera stays read-only |
| Native presentation | morph-lab/bin/moebius-world-witness.rs | active M4 fixed-rest witness | native captures/lifetime test | adapt same binary; separate camera module, animation, filament caches/budgets and controls |
| Camera/editor | morph-lab main orbit, M4 witness translation | separate existing UI scopes, inadequate free flight | old orbit test | retain editor keys; new witness navigation owns presentation-only state |
| Spatial querying | witness.rs linear scan, runtime.rs region/local Position | active coordinates, no structural index | coordinate/replay tests | derived sparse region/chunk index only; never topology authority |

Paths under crates/; inspected AGENTS, HANDOFF, M1–4 results/architecture,
relevant source/tests/history. No other branch/stash integration or external
scientific result is assumed. Source identities preserve the entry checkout.

No competing ship construction authority was found. The new seam is explicitly
requested: free secretion -> bound filament material inside the same book/world.
This is extension of existing responsibility, not another canonical runtime.

## Implemented executable contract

Book owns quantities. A bound record retains the admitted parcel, transfer tick,
filament/node identity and provenance. Whole-parcel all-or-none transfer removes
one free record and adds one bound record. Graph quantities are validated cached
projections of those records, never an extra material stock. Empty new fields
are skipped in old JSON, preserving old M3/M4 serialized identities.

Ship owns ordered polylines with stable IDs and source/experiment metadata.
Consecutive nodes are edges. Transfer geometry must have positive bounded edge
length, capture distance and compatible source/nozzle/material. Gaps start a
separate filament; no arbitrary merge or branch. Bound positions are authored
static structural coordinates, not a mechanical equilibrium claim.

Recorded3D RigidPose samples and emission schedule are experimental inputs.
Existing CompiledAnatomy posed geometry supplies moving nozzle origin/direction;
body/flagella presentation uses that same tick-derived authored pose. Numerical
emission/transport and assembly proceed without witness data.

Admission, free-to-bound changes, rejections and causal events are deterministic.
Limits cap material records, nodes, filaments, path samples, commands and events.
Exceeding a hard event/history budget returns an explicit atomic error rather
than silently dropping causal history. Limits apply to005 configurations;
assembly-disabled M4 numerical states remain compatible and reproducible.

Witness uses region-local coordinates. Filament index/cache is derived data;
chunk meshes rebuild only on visible entry or changed chunk, under an explicit
per-frame mesh budget. No global microscopic grid. Frozen geometry and graph
connectivity do not establish tension, elasticity, anchoring or load-bearing.

Free-flight navigation integrates camera velocity analytically with smooth
acceleration/deceleration; quaternion local axes impose no terrestrial up.
Orbit has explicit pivot, focus/pan/dolly and stable transitions. Camera and
sphere are separate, following by default; only clock controls send explicit
Play/Pause/Step/Reset/Rate requests to the existing worker. No UI biological
state writes. Reset is an explicit new run; no silently skipped ticks.

## Acceptance

A–H experiments; horizontal/vertical/curved/orientation/gap cases; exact A/B/C
and restore continuation; finite-budget stress; material duplication/loss and
observer/clock/serialization honest-red; old M3/M4 replay identities; camera
frame-rate and mode/focus tests; real native/GPU inspection with recorded
costs, mesh counts and controls. Then classify/hash/restore raw Mermogon evidence,
update external Stable Diffusion Atlas, authorized local commit, clean full suite.
No external scientific publications consulted; assembly, static binding and
sampled path laws are authored assumptions, not scientifically validated biology.

Focused tests, seven red/green faults, A–H numerical experiments and recorded
native observation/replay establish the bounded contracts described in
MORPH_005_RESULTS.md. Clean-commit workspace certification is recorded separately
in docs/morph_005 and refs/notes/morph-005-validation. Physical mouse usability
and Operator visual acceptance remain pending; no mechanical capability is claimed.
