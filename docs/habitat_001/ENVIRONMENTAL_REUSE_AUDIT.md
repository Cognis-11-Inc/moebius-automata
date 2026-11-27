# HABITAT-001 environmental reuse audit

2026-11-27. Entry966948a; main/remote equal, clean. VII263-test certificate at
847b700 is historical regression evidence; canonical-source differences to that
publication are metadata only. Existing instructions and handoff were read.

| Concern | Path/symbol | Status | Input -> output / consumer | Authority / tests | Reuse ruling / missing seam |
|---|---|---|---|---|---|
| Uniform environment | morph/emission.rs Environment, integrate, MaterialBook::step | canonical experiment-local parcel law | per-mille blend/uniform acceleration/flow -> integer parcel motion | book owns admitted secretion quanta; MORPH-003/substrate/tests | preserve; no spatial density, SI mass, viscosity, pressure or carrier momentum state |
| Paid material | MaterialBook::try_emit, bound, Ledger; filament::Assembly | canonical ship stock/provenance | integer generic secretion -> free/bound/sinks | finite ledger, source/version tests | do not duplicate/debit it; atmospheric species start as separate explicit experimental SI inventories, no transfer mapping authorized |
| Ship collision mode | habitat::validate; collision::resolve | canonical | suspended Environment -> admitted kinematic contacts | existing force-free/drift guards | keep guards; no physical atmosphere adapter here |
| Nursery rig | core/neural/body.rs Body::{step,step_with_law}, shape_impulse,lunge_impulse | separate active nursery approximation | quantized motor/drag -> body impulse + absorbed reaction | World::step adds to environment_momentum; neural/rig tests | reuse action-reaction discipline, not coefficients or authored lunge; no spatial fluid state or SI drag law |
| Nursery material/cells | core/world.rs jets/nutrient, geom::Dims/Site, layers::fluid_samples | canonical separate periodic lattice | integer material field -> lattice transitions/visual aggregate | nursery invariant tests | visual samples are not fluid dynamics; cannot serve sealed ship metric chamber |
| Global coordinates | morph/runtime.rs Position::{from_absolute,absolute,relative,offset}, SCALE, TICK_NANOS | active canonical metric infrastructure | region/local micro-units -> local differences and clock | coordinate/translation tests | reuse Position origin, conversions and tick; local SI arrays are discretization coordinates, not a new global coordinate system |
| Local geometry index | CollisionCache::interacting_chunks/pipe horizon; StructuralIndex | active collision derived state | paid geometry/revisions -> conservative candidates | IV mutation/horizon tests | not an atmospheric field/index; preserve responsibility |
| Persistence | morph::write_atomic; WorldSnapshot JSON/digest; history::Store | active atomic write/v1 and opt-in source-bound ship history | validated ship Event/WorldSnapshot -> history | atomic/history faults/replay | reuse atomic write, BLAKE3 and source identity, initial+advance pattern; no second event/replay service. A bounded chamber-state bit checkpoint is a typed adapter; ship Store accepts incompatible Event/WorldSnapshot only |
| Witness | morph-lab Bevy app, Worker/Navigation, native recorder | active observer infrastructure | immutable authoritative state -> display/explicit clock controls | native/source neutrality tests | reuse package/Bevy/gizmos/recording pattern; separate explicit HABITAT binary calls the same headless Chamber; never touches ShipRuntime |

Inspected current source above, Cargo/lock, root architecture and VII/V physical
and sensory contracts; source/history commits cdc3acd,8748f66,847b700/966948a and
current main/origin/main. No stash/other local branch. No physical atmospheric
solver found by viscosity/pressure/advection/diffusion searches; core fluid_samples
is observation of nutrient. No unpublished branch, entire old prototype corpus,
external atmosphere package or large-scale hull mechanism is assumed audited.

Decision: narrow experimental module in existing moebius-morph, gated
habitat-experiment; extend existing lab with a separate binary. No new crate,
shared environmental authority, organism controller, generic material service or
coordinate/replay redesign. Independent ownership is explicitly authorized by
HABITAT-001; future canonical integration remains unapproved.
