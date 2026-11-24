# MORPH-005 — The first filament

2026-11-23. **Engineering/scientific gates PASS; visual acceptance PARTIAL (Operator review pending).** The
finite-material construction and mandatory navigation addendum are implemented.
Focused tests, seven red/green demonstrations, A–H experiments and rendered
replay pass. Human/physical-mouse visual acceptance is not claimed. No push.

Base `d6fa1ac41ec73f7c6ab0372032d32fe6faf0c047`, main. Fetch verified0/0
against origin/main before implementation. Existing146-test M4 attestation is
on78f1f44; that publication history was verified, not assumed.

## Authority, material and topology

`MaterialBook::bind` is the sole new free-to-bound transfer: a whole admitted
parcel is debited exactly once from free material and credited to a bound
record. It rejects zero/mismatched quantities, unavailable identities and
duplicate transfers before committing. The book owns quantity; a filament's
cached quantity must equal its referenced bound records and is never counted
as another stock. The invariant is:

`initial + authorized input = reservoir + free + bound + recorded sinks`.

No input is credited by these experiments. Old JSON omits empty new fields;
old v1 free-only states retain their original digests. Structural book/world
states use version2. M3 free-parcel transport/admission coefficients remain;
bound positions are explicitly static rather than being integrated as free
particles. This is an authored structural phase, not equilibrium or hardening.

`Assembly` stays inside canonical `ShipRuntime::WorldSnapshot`. Ordered nodes
form implicit consecutive edges. Filament identity, organism/nozzle, experiment,
creation tick, material and parcel provenance persist. World events record
emission, binding and rejection; restoration validates graph/book/event agreement.
A mesh owns no stock and cannot modify the graph.

Default law: material age>=2ticks; capture<=0.75m from the actual animated
nozzle; whole-parcel admission; edge length0.001–0.5m. A longer gap starts a new
filament, never a teleporting edge. Too-short/out-of-capture material remains
free. Up to32 eligible candidates are examined and one admitted each tick.
There is no branching, arbitrary merging, rod/elastic solver, tension, collision,
load-bearing, curing, anchoring, biological construction decision or pipe system.

The recorded3D path uses existing RigidPose/AuthoredTrajectory validation;
CompiledAnatomy supplies the actual posed mouth/body/flagella/nozzle geometry.
Numerical nozzle and visible anatomy use the same tick-derived authored pose.
Motion is prescribed, not propulsion or autonomous locomotion. The headless
construction tick currently samples full posed anatomy for one nozzle; this is
measured, not an optimized million-organism output-only sampler.

## Experiments and gates

| Experiment | Measured outcome |
|---|---|
| A no reserve |0 nodes,0 filaments; explicit exhausted/no-eligible results; balance0 |
| B straight |64 bound nodes,1 filament; balance0 |
| C curved |64 bound nodes,1 filament; balance0 |
| D interrupt/restart |5tick emission gap;64 nodes, existing filament extended; balance0 |
| E exhaustion |5 admitted/bound quanta; later requests rejected; balance0 |
| Vertical |64 nodes,1 suspended filament, no floor; balance0 |
| Disconnected path |79 bound nodes across2 filaments plus1 free parcel from80 initial; no unsupported long edge or lost material |
| F witness neutrality |120ticks xheadless/stationary/moving; every authoritative digest identical |
| G midway restore |Save at60, restore and continue60; final state equals uninterrupted120 |
| H growth scale |32/128/512/1024nodes exercised; no extrapolation beyond measured cases |

| Gate | Status and evidence |
|---|---|
| G1 conservation |FOCUSED PASS: independent free/bound ledger, duplicate/loss rejection and signed residual faults |
| G2 connectivity |FOCUSED PASS: stable ordered graph, bounded nonzero edges; binding/node/event provenance; local mesh audit. No global self-intersection certificate |
| G3 persistence |FOCUSED PASS: graph unchanged after emitter moves/exhausts reserve; camera exits/re-enters from existing data |
| G4 free space |FOCUSED PASS: horizontal/vertical/curved/rotating/gap cases; authored static binding only |
| G5 determinism |FOCUSED PASS: repeated state/digest equality in recorded build; f32 anatomy/f64 initial-path generation not cross-platform certified |
| G6 observation |FOCUSED PASS: A/B/C numerical equality; all320 final native snapshots and pre-reset digest replay headlessly |
| G7 serialization |FOCUSED PASS: complete graph/binding/events/config and continuation; malformed quantities/topology/cached stocks rejected |
| G8 resources |FOCUSED PASS for measured bounds/costs below; growth/history/input/capture budgets explicit |
| G9 regression |FOCUSED PASS:49 morph/lab tests; strict workspace Clippy/format; clean implementation full suite156/156 PASS |
| G10 provenance |FOCUSED PASS: every node matches an admitted parcel/binding/creation event and source experiment; failures retained with explicit source-capture limits |

Seven source/consumer fault demonstrations in HONEST_RED.json cover transfer
copying, transfer loss, graph-stock mismatch, observation-dependent command
admission, history overflow, tick loss and unsupported long-edge growth.
Every deliberately defective source fails its focused test (exit101); restored
source passes. Duplication/loss are caught by the transactional balance guard,
so no corrupted material state is committed. Direct signed residual corruption
checks also distinguish duplication from deletion.

The original M3 repeated-state digest remains
`cff18b1d0174646fd6b852f6d7cff059894420420fe5d2809cb6641c77e90cfb`;
the original M4 120tick digest remains
`4f34d8daad96fd1fc84860a53168ed0e6580a68fae095bbdf3ed7bf4b3154552`.
No historical result/report, nursery implementation or anatomical compiler law
was rewritten. New schema/input capabilities are explicit, scoped extensions.

## Navigation and visible construction

Same native Witness binary; `--construction` selects the one-organism/nozzle
experiment. Right-mouse look captures only while looking; flight uses local
quaternion axes without forced terrestrial up. WASD/QE translate, Z/X roll,
Shift accelerates, Ctrl is precise, wheel changes speed. Analytic damped velocity
integration retains sub-micro-unit camera rounding residue. Fixed-heading30Hz
and120Hz tests agree within2micro-units. Changing-input trajectories are not
claimed mathematically identical at arbitrary sampling rates.

Tab toggles flight/orbit while preserving orientation; left drag orbits the
selected pivot, middle drag pans, wheel dollies, F frames selected anatomy or
filament bounds, Home resets camera. Left click selects organism/filament.
Camera and witness are separate; witness follows camera by default, with visible
radius control and optional diagnostics. No camera coordinate becomes an
organism position. The older anatomical editor's controls remain in its separate
binary/default launch path.

Production widgets expose play/pause, step, reset,1x/4x/16x numerical batch
rates, follow/free camera, visibility, diagnostics, witness radius/follow and
mesh budget. Reset explicitly starts a new experiment generation from recorded
initial state; it is not erosion of the previous world's structures. The prior
run's final tick/digest and clock actions are retained and replay-checked.
Play/pause/step/reset/rate controls go through the existing owner worker's bounded
channel. Speed selects actual tick executions, never dropped history; compute
or rendering may fall behind requested wall-clock pace.

Final native run:320frames,6captures,360ticks,359 bound quanta and1 free from
512 initial (reserve152). Every frame and pre-reset state independently rebuilds
headlessly. Final digest:
`d8de2311d21ac10492d6cee73755e955eaf61dba912d8d6f65e58027da016fd4`.
Body bending/flagellar motion, amber free material and blue persistent geometry
are visible.0.025m tube radius is a display convention, not a validated volume,
material density or load-bearing property.

Input testing injects keyboard/mouse resources and actual UI Activate events
through production handlers, including orbit/look/pan/dolly/focus/roll/precision.
An explicit diagnostic cursorpoint replaces a denied Wayland cursor warp.
This is actual GPU presentation on RTX5060Ti/driver615.78.08, not physical mouse
or human acceptance. Earlier Wayland errors remain in raw logs.

## Bounds and measured performance

Maximums in the new construction schema:4096 combined free/bound material
records per book,4096 nodes,4096 filaments,16384 route samples,65536events,
256 pending interventions; controller queue64, clock-command history512,
recorded frames4096. Over-budget state/requests reject explicitly; history cap
fails a tick atomically instead of deleting prior events. These are prototype
budgets, not support claims for every simultaneous maximum.

Derived sparse occupied-region bins select16-edge chunks. There is no dense
vessel grid. Only visible/new/changed chunks mesh, under a default2 builds/frame;
sealed chunks are reused. Re-entry reconstructs presentation only. Chunk AABBs
are conservative; up to one chunk's local geometry can overdraw the sphere.
Index rebuild currently scans the bounded graph when its node count changes;
full-state transactions/history and per-tick anatomical sampling remain costs.

| Nodes | Construction time | Final JSON bytes | Index build | Local query |
|---:|---:|---:|---:|---:|
|32|17.73ms|24194|1.24µs|1.11µs|
|128|57.81ms|78383|2.02µs|1.78µs|
|512|225.11ms|300995|6.82µs|2.77µs|
|1024|473.43ms|599452|12.57µs|4.46µs|

Each run advances node-count+8ticks; actual final counts equal requested counts.
Peak RSS14252KiB is for the whole headless benchmark, including replay,
serialization and mesh audits. One release measurement, no confidence interval.
Final native v7 median draw CPU2.438712ms/frame; camera/control CPU0.003475ms/frame;
frame intervals ~16.67ms under vsync; maximum38 mesh assets and2 filament mesh
builds/frame in the final sample. No isolated GPU timestamp or pure GPU cost.

Future candidates: incremental chunk/index updates, partitioned histories with
checkpointed replay, and output-only anatomical sampling. Each requires paired
state/digest/geometry and boundary-query tests against this exact reference
before adoption. Mechanical rods, anchoring, hardening and multiscale ecological
approximations remain future experiments, not consequences of a persistent mesh.

## Reproduction and evidence

```
cargo run --release -p moebius-morph-lab --bin moebius-world-witness --offline -- --construction --ticks 3600
cargo run --release -p moebius-morph --example morph_005_bench --offline -- NEW_RUN
cargo run --release -p moebius-morph-lab --bin moebius-world-witness --offline -- --construction --ticks 360 --record NEW_CAPTURE
cargo run --release -p moebius-morph --example morph_005_verify_render --offline -- CAPTURE
cargo test -p moebius-morph -p moebius-morph-lab --offline -- --test-threads=8
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo fmt --all -- --check
```

Mermogon stores immutable raw trials/logs/captures/states/source overlays/Atlas
versions with verified index/archive/member/restore hashes. Git stores reports,
protocol/source identities, small summaries and reproducibility references.
Atlas remains on the Stable Diffusion Drive, outside this repository.
Exact paths/versions/hashes and restoration receipts are in docs/morph_005/.
No fresh-machine dependency resolution or cross-platform reproduction claimed.

All earlier nulls and failed test/compiler/input trials remain classified in
TRIAL_CHRONOLOGY.json. Full source images were not captured before early test
fixture repairs; recorded diagnostics/parameter corrections are retained and
that limitation is explicit. No external scientific papers were consulted;
static binding, capture and growth laws are authored assumptions. No Lisp or
Python live service was added. Clean implementation4c44c63 passed all156 workspace tests,0failures, with a clean
tree before/after. The documentation receipt is retested on its final clean HEAD;
its exact SHA/transcript/archive receipt are in the local Git note:
`git notes --ref=morph-005-validation show HEAD`. Absent note means that final
gate is pending. Physical mouse/human visual acceptance remains pending.
No push or MORPH-006 is authorized.
