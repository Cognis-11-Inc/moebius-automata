# MORPH-004 — Authorized free-space world convergence

Operator ruling received: **option2**. Promote and extend the MORPH-003
free-space substrate as the canonical MŒBIUS ship runtime. Preserve core World
as the separate nursery experiment with its existing laws, tests and formats.
This resolves the ownership gate; no adapter to nursery physiology is implied.

The runtime lives beside the existing MaterialBook in moebius-morph and delegates
all parcel admission/integration/accounting to that unchanged implementation.
One ShipRuntime owns a complete versioned state and tick. Per-instance books
are substates constrained to that same tick, not independently scheduled clocks.
The laboratory's old preview remains a scoped editor/demo, not a ship authority.

## Sealed construction contract

Containment: do not change MaterialBook laws, nursery source/snapshots, or
anatomical definitions. No biology/controller/swimming/learning/new atmosphere.
Compact instances compute prescribed constant drift, environmental exposure and
unchanged emission/parcel integration. They are test organisms, not full ecology.

Metric convention authored for004:1 anatomical unit=1metre;1e6 integer micro-units
per metre. Region width1000metres; normalized region/local coordinates preserve
micro-unit differences. Explicit supported absolute bound1e15micro-units.
No dense grid. Legacy lattice units are not converted or silently redefined.

A definition has a fixed rest-pose nozzle binding computed once at admission.
No headless per-organism detailed mesh generation per tick. Visible geometry
uses that same pose; animate only through explicitly recorded instance state.
This slice does not add deforming-emitter physiology.

Tick transaction: apply accepted interventions for current tick in sequence
order; for each organism in ascending stable ID emit at its existing nozzle,
step the unchanged book, apply prescribed drift, sample environmental light;
advance global tick once. Failure commits none of the candidate. Checked integer
arithmetic and a fixed event order; f32 anatomy compilation remains same-build.

Intervention submission validates target tick, unique stable sequence, origin
and typed environment/illumination operation. Admission/applied causal events
and future commands belong to the persisted digest. Camera/witness/quality do
not occur in this state or command API.

Immutable snapshots feed a pure Witness(center,radius) selection and renderer
cache. Independent worker owns runtime and advances to a fixed target without
reading camera or Bevy Time; a latest-snapshot slot never blocks or drives ticks.
Dropping observation packets is allowed; world ticks and causal logs are kept.
Zero listeners, slow frames and cache entry/exit cannot mutate ship state.

Light ports provide an explicitly authored integer distance-falloff exposure
input independent of visual illumination. No transport, occlusion or thermal
solver. Local fog capability is assessed in donor report; implementing it is
optional and no GPU-cost claim is made without actual measurement.

Existing and proposed hybrid boundaries in the audit below remain valid, except
that the choice of canonical owner is now settled in favor of the free-space
substrate. Lisp/Python have no new live consumer or mutation rights.

---

## Preserved pre-ruling investigation

Architecture convergence is unresolved.

# MORPH-004 authority audit and proposed contracts — 2026-11-23

Status: BLOCKED for world-runtime implementation; archaeology/research only.
Entry main20f0febb0cdc8e613ae36dfc6ed1abe529994d52; only pre-existing edit
was the Codex rehydration note in HANDOFF.md. MORPH-004 authorizes local
in-scope commits, no push. Its new directive supersedes the old instruction
not to begin004, but explicitly retains the convergence gate.

## Containment and decision boundary

AGENTS.md section4: "stop implementation in that contested domain" when
systems overlap or a bridge lacks a proven interface. Section4 reserves the
architecture decision for the operator. MORPH-004 section0 forbids a competing
engine or speculative third runtime. Therefore this audit does not promote
MaterialBook, add a scheduler beside World, or invent a combined material law.

These are distinct lawful authorities for their original scopes, not proof
that prior work is defective. Combining them into one world is the new seam.
MORPH_003_IMPLEMENTATION_PLAN.md explicitly limited its convergence ruling
to an experiment-local book, not a replacement World or shared snapshot.

## Archaeology matrix

| Concern | Path/symbol | Status | Input -> output | Consumer | Authority | Existing tests | Reuse ruling / missing seam |
|---|---|---|---|---|---|---|---|
| Simulation state | moebius-core/src/world.rs World::{genesis,tick,tick_observed,tick_controlled} | active canonical nursery | SimConfig/seed or Snapshot -> audited World/TickReceipt | CLI batch, viewer worker, scientific examples | World owns cells, BrainRig, field, flesh, jets, stocks, tick | core/tests/invariants.rs, neural.rs, nursery.rs, terrain.rs, m4/m5/m6 | reuse canonical transitions; no AnatomySpec or MaterialBook consumer |
| Headless execution | moebius-cli/src/lib.rs run_batch | active canonical batch | RunOpts -> ticks, metrics, manifest, snapshots | CLI main Run | delegates every transition to World::tick | CLI batch tests | already headless, not missing a renderer-free engine |
| Production live clock | moebius-viewer/src/live.rs {start,pump,wait,shoot} | active presentation worker | SimConfig -> World::tick -> Picture | Bevy live viewer | worker owns World | viewer/live tests | sync-channel send can block; disconnect stops worker; observe pacing needs an explicit lifetime/backpressure contract |
| Nursery observation | moebius-core/src/report.rs observe; snapshot.rs Snapshot | active read-only reporting/persistence | &World -> metrics/canonical bytes/hash | CLI, analysis, viewer | observational; includes tick in canonical bytes | core invariant/snapshot tests | reuse; does not capture MORPH-003 parcels or definitions |
| Core anatomy | moebius-core/src/neural/{schema,body,controller}.rs Anatomy/Body/NeuralGenome | active nursery motor rig | chain joint schema/motor vector -> integer articulation | BrainRig and World | core physiology/momentum laws | neural/m4/m5 tests | not the composable geometry schema; no tested identity/unit/motor mapping to AnatomySpec |
| Composable anatomy | moebius-morph/src/{spec,anatomy,compiler}.rs AnatomySpec/CompiledAnatomy | active headless anatomy compiler | v1/v2 definition + Pose -> surfaces/output frames | lab, population/replay consumers, M3 emission sampling | geometry only, no physiology | forge/composition tests | reuse definitions, not interpret animated pose as physical motion |
| Conserving secretion | moebius-morph/src/emission.rs MaterialBook::{try_emit,step,residual,to_json,from_json} | active experiment-local numerical substrate | quantized output frame/request + Environment -> parcels, book tick/ledger | M3 bench and lab | owns only its own initial/reservoir/parcels/sinks | morph/tests/emission.rs and module tests | preserve exactly; separate tick, identity, stock and JSON; no core account transfer |
| M3 GUI clock and controls | morph-lab/src/{main.rs simulate,emission_preview.rs PreviewControls} | active experimental preview | Bevy Time/buttons -> book step/emit and authored trajectory | lab renderer | preview directly changes book; never claimed independent world | lab clock/preview tests | cannot promote to world runtime: drops backlog after4 steps, same-frame mutation and observation |
| Reference world coordinates | core/geom.rs {Site,Dims,step_site}; core/neural/body.rs | active periodic lattice | u32 sites + 1/1024 site local offsets -> wrap | World | discrete lattice, no metre/ship-region contract | geom/neural tests | not a sparse30,000km world representation |
| M3 coordinates | emission.rs {SCALE,RigidPose,place_output} | active experiment micro-units | f32 anatomy -> i64 micro-units/quantized yaw | book and lab | 1e6 =1 anatomical unit; no stated metres per unit | nozzle transforms/persistence tests | i64 extent alone does not define ship units or region ownership |
| Existing cross-language use | python/ audits/analysis | supporting offline instruments | saved evidence -> checks/reports | human/research consumers | cannot mutate World | slice-specific audits | no live Python or Lisp world-control consumer found in inspected scope |

Source paths in the matrix are relative to crates/ except python/. No branches
besides main and origin/main, no stash, one worktree. Inspected Cargo dependencies:
core and morph do not depend on each other; both use serde/blake3. No existing
producer-to-consumer bridge was found in the inspected active source and reports.
This is not a claim that every historical file or external project was audited.

## Exact incompatibilities to decide

1. Owner and lifetime: preserve World as sole owner/clock and embed/delegate the
   unchanged M3 book, or explicitly authorize promotion of the free-space
   substrate while classifying the existing nursery as a separate historical
   experimental model. A third unclassified owner is excluded.
2. Organism state: core Cell/BrainRig/NeuralGenome and morph AnatomySpec are
   different contracts. Geometry records cannot be counted as physiologically
   simulated organisms. Need explicit per-instance identity, state ownership,
   position/velocity meaning and which existing transitions actually run.
3. Material: nursery nutrient/reservoir/flesh/waste/jets and M3 private initial/
   reservoir/parcels/sinks have no transfer operation or shared budget. Summing
   them without a declared partition risks invented or double-counted stock.
4. Time/order: book and World own ticks. Existing M3 benchmark emits then steps;
   preview steps then emits. A single runtime must choose one recorded order,
   admission boundary, failed-tick atomicity and continuation/snapshot contract.
5. Coordinates: map anatomical units and lattice sites to ship/local frames
   explicitly, rather than silently declaring a site or anatomical unit a metre.
6. Environmental intervention: core SimConfig is private World-owned state;
   M3 Environment is book-owned. A typed request needs a decided owner and
   replay format. UI callbacks currently mutate only their experimental book.

## Reviewable convergence choices (PROPOSED, not implemented)

A. Extend existing core World as canonical authority. Add a versioned suspended
world mode and explicit ownership of unchanged MaterialBook substates and
AnatomySpec instances. Retain legacy periodic transitions/restore formats.
Define the six seams above before implementation; do not convert geometry to
feeding/locomotion. This prioritizes continuity of the present canonical owner
and requires an expressly authorized schema/dependency/convergence change.

B. Promote and extend the M3 free-space substrate as the canonical MŒBIUS ship
runtime, explicitly classifying the existing lattice World as a preserved
nursery experiment. Reuse its scientific tests; do not silently delete or
repurpose its laws. This is a change of canonical authority, not a default
implementation convenience, and needs an operator ruling.

C. Narrow004 to observer-neutral presentation of existing lattice World only;
leave M3 and ship-scale organism coordinates separate. This can test an existing
world without new authority, but does not meet004's combined reference-world
requirements. It needs an explicit scope reduction before being called success.

## Witness and intervention contracts after a ruling (PROPOSED)

An owner publishes immutable tick-labelled state with stable organism/material
IDs and complete replay digest. Witness(center,radius,quality) selects only
presentation data. Mesh/particle caches belong to the renderer and may be
removed on exit. No owner takes camera/witness visibility as tick input.
Backpressure may coalesce observation packets but never erase authoritative
steps/events. Lifecycle must allow zero subscribers. No viewport entry reset.

Intervention: target_tick + origin + stable_sequence + typed operation. Admission
validates before application; deterministic order is recorded and replayed.
The minimal candidate changes explicit environmental input only. Observation
and visual-quality messages never enter that command stream.

Illumination: owner environmental exposure/thermal inputs are distinct from
render light/fog settings. A port outside the sphere may contribute an explicit
analytical exposure input without its full mesh. No radiative-transfer law is
proposed as implemented. Bevy0.20 local sources have VolumetricFogPlugin,
FogVolume extraction and a separate render sub-app. Availability is verified;
no GPU cost or bounded-volume implementation measured in this audit.

Coordinates candidate: hierarchical integer region plus bounded local fixed
point; convert differences to camera-relative f32 only after subtracting
origins. Vessel dimensions are proposed30,000km by3,000km; no dense ship grid.
Metre/quantum/region-width choices and overflow/roundtrip tests remain pending.

## Hybrid interface map

| Producer | Consumer | Representation | Validation and owner | Frequency | Status / persistence |
|---|---|---|---|---|---|
| core World | existing CLI/viewer | TickReceipt, Snapshot/Picture | World audits, viewer observes | per committed tick, sampling possible | EXISTING, core snapshots and manifests |
| CompiledAnatomy | M3 book sampling / lab | Pose -> OutputFrame/surfaces | anatomy validates geometry; book admits quantity | explicit sample/preview tick | EXISTING, v1/v2 JSON and M3 book JSON separately |
| chosen world owner | Witness/Bevy | immutable tick-labelled compact snapshot | owner validates state, Witness has no mutation rights | bounded publication cadence | PROPOSED; no transport or second owner created |
| operator | chosen world owner | typed future-tick intervention with provenance | owner admits/applies/logs | sparse explicit actions | PROPOSED; ordering/replay gate required |
| persisted experiment artifacts | Python | JSON/CSV/binary + hashes | analysis is read-only, runtime remains Rust | offline batches | EXISTING supporting audits |
| Lisp policy experiment | chosen world owner | bounded typed proposed actions | owner would validate/apply; Lisp has no direct writes | future batched tick schedule | PROPOSED; no executable consumer, no IPC added |

No new runtime, observation neutrality, intervention integrity, ship-coordinate
precision, volumetric cost, or100/1000/10000-organism support is demonstrated
by this document. The donor assessment and preserved tests are evidence for
consultation and existing contracts only.
