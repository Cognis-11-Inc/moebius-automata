# First simulation: feeding constructor

2026-11-24 (America/Toronto), authorized by the operator: the first simulation needs functional
food and Arachnitech. This is a follow-up within canonical ShipRuntime.

## Containment and exclusions

Risk: a scripted body or a decorative mesh can be mistaken for ecological
function. Preserve all old fixtures/snapshots through an opt-in construction
rule. No nursery bridge, second world owner, fabricated mouth, new learning
claim, unlimited food credit, or renderer authority. Do not certify the whole
simulation as learned locomotion or living structural tissue.

## Existing authority and reuse

| Concern | Source | Status | Input -> output / consumer | Authority and tests | Reuse / missing seam |
|---|---|---|---|---|---|
| World | runtime.rs ShipRuntime::tick | canonical | commands + state -> next snapshot / worker | sole mutation owner; runtime tests | extend existing transaction |
| Food | runtime.rs blood_tick, Condensate | active uncommitted | finite stock -> sac -> eater | book transfers; blood tests | enable builder and actual generated mouth in opt-in mode |
| Stock | emission.rs MaterialBook | canonical | intake -> reservoir -> parcel/bound/sink | integer residual; emission tests | reuse unchanged |
| Structure | filament.rs Assembly::deposit | canonical | paid parcel -> persistent graph | filament tests | same chosen structural nozzle; persistent scaffold supports existing food fixation |
| Movement | ConstructionRules route, runtime tick | authored fixture | prerecorded poses -> body position | route validation | preserve old route; explicit bounded food steering for new mode |
| Mouth | CompiledAnatomy::pose | canonical anatomy | spec + pose -> mouth_center | anatomy tests | place that point with the same rigid body pose used by renderer |
| Startup | witness begin_ship | active uncommitted | player anatomy -> worker | focused witness tests | use one shared first-simulation factory; avoid inactive decorative copies |
| Nursery controller | moebius-core | separate experiment | lattice observations -> motor output | separate laws/tests | no unproved adapter; not used here |

Paths are under crates/moebius-morph/src except witness under
crates/moebius-morph-lab/src/bin. Inspected latest handoffs, architecture,
source/call sites, food/blood/filament tests and current Git state. No external
scientific source, branch/stash audit, or new scientific model adopted.

## Construction target

One player-created constructor and finite-stock Blood-Sacs. Explicit opt-in
food steering operates in ShipRuntime and is a kinematic authored policy,
not a neural controller or physical propulsion. At stock <=12 it seeks food
within 12m; at stock >=48 it leaves the meal and builds, moving at most 0.025m
per tick. Between thresholds the explicit saved seeking/building state supplies
hysteresis. A meal transfers at most4 quanta within0.25m of the generated open
mouth. Movement itself has no metabolic/energy cost in this prototype.
Generated mouth contact
transfers actual sac quantity into the same builder's material book. The
builder's admitted nozzle secretion pays for bound persistent structure.
Existing light-and-secretion-gated sac fixation supplies the ecological
feedback; it is not structural tissue metabolism. No generic regrowth ledger
is mixed into this finite-stock mode.

Show real intake and construction counters and short uptake markers from
committed feeding events. Do not append decorative anatomy. Keep renderer
and headless body pose identical. Save/restore the consequential steering
state explicitly. Old authored constructions retain their route semantics.

## Proof obligations

Focused tests: first startup has active builder and paid food; actual mouth
contact, distant/closed mouth refusal; food consumption credits the builder;
construction exceeds starting stock only after paid intake; no-food and
emission-disabled interventions; exact condensate/book balances; structural
food-fixation dependence; save/restore continuation; witness startup uses this
same factory. Run honest-red faults for the new feeding guard and loop.
Relevant crate tests, format/whitespace, and strict Clippy; preserve pre-existing
failures. No full clean-commit certification without authorized clean commit.
Native inspection is a separate gate, not inferred from unit tests.

Status at creation: planned. Results and limitations follow below.


## Implemented and validated result

DONE for the bounded feeding/constructor loop. The larger proposed simulation,
learned locomotion and living structural tissue remain PARTIAL/unimplemented.
`ShipRuntime::first_simulation` is shared by player new-game startup and the
`--first-simulation` inspection flag. It creates one active constructor and
six paid Blood-Sacs, not twelve emission-disabled copies. The current cap of36
includes subsequent food bodies. The original explicitly bounded run duration
remains in effect; it is not an endless ecology service.

`ConstructorState` saves both the rigid pose and the seeking/building phase.
`ForagingRules` makes sensing radius, contact radius, step, bite and refill/build
thresholds explicit. `WorldSnapshot::{body_pose,anatomical_pose,mouth_position}`
provides the shared numerical/presentation mapping. `Event::Fed` records eater,
source, quantity, tick and actual mouth landmark. Event totals must match the
constructor book's intake. The same fixed-point rigid placement is used for
nozzle and mouth landmarks. Feeding is a0.25m proximity test at the mouth to
the food origin, not a mesh collision/swallowing simulation.

`MaterialBook::ingest` promotes paid input to explicit book version3 and checks
both additions before mutation. Binding preserves that version. Older book
versions retain their zero-external-input restriction. No observer creates
material; finite food transfers and book residuals are checked by the runtime.
The new mode also checks exact condensate balance and rejects independently
regrowing generic food. Older unconfigured snapshots omit the new optional
state/rules and retain their original fixture behavior.

The selected structural nozzle binds paid secretion into the existing Assembly.
Additional nozzles emit free parcels through the same ledger; only the configured
structural nozzle binds them. The scaffold permits existing nearby,
light-gated Blood-Sac fixation. It remains a static graph rendered as capped
rods: no demonstrated hollow lumen, mechanical support, tissue metabolism,
physical propulsion, collision response, adaptive learning or multi-constructor
population. Motion and the refill/build rule are explicitly authored; motion
has no metabolic cost. These limits are not paid by the tests below.

### Focused numerical receipt

Bare anatomy seed20261123, same world seed, first-simulation rules,300ticks:

| Measurement | Food present | Food absent control |
|---|---:|---:|
| Founding constructor stock |24|24|
| Intake |242|0|
| Final reservoir |13|0|
| Bound material |252|24|
| Free material |1|0|
| Book residual |0|0|

Finite condensate stock remaining in the food-present world:3566; global
condensate gap0. Tests check each tick's stock/book balances, actual contact,
closed-mouth and distant-food refusal, opposite-side food approach, refill/build
cycling, no-secretion/no-fixation controls, exact serialization continuation,
and rejection of pose, stock, legacy-book, unpaid-quantum and intake-history
corruption. Changing food position changes the approach despite identical
recorded fixture paths.

Commands on rustc1.99.1, existing offline cache (all final runs PASS):

```
cargo test --release --offline -p moebius-morph --lib --tests -- --test-threads=8
cargo test --release --offline -p moebius-morph-lab --lib --bin moebius-world-witness -- --test-threads=8
cargo clippy --offline -p moebius-morph -p moebius-morph-lab --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
cargo build --release --offline -p moebius-morph-lab --bin moebius-world-witness
```

51 morph tests plus16 lab/witness tests:67 passed,0 failed. Eight are the new
first_simulation integration tests. Old serialized-state, material, structural,
clock/observer and replay regressions pass within the relevant crate coverage.
This is focused validation of a dirty tree, not clean-commit full-workspace or
release certification. No commit, push, hosted CI, archived old-data replay,
new-machine restoration or generalization across arbitrary player anatomy is
claimed. The player-body startup test additionally uses anatomy seed77.

Honest-red faults were applied separately to the final implementation and
restored: bypass closed-mouth guard; bypass finite-condensate restore guard;
exclude the builder from feeding; disable transition to building. Each focused
test failed (exit101), then all67 relevant tests passed after restoration.
The tests were not weakened, disabled or replaced to make those failures green.

### Failure and native observation chronology

Initial new integration tests:3 passed/3 failed because the old loader rejected
all external food credit. Explicit fed-book version3 repaired that seam. Initial
viewer check failed on ambiguous Event import; it was qualified. Strict Clippy
initially exposed existing food/runtime/gore warnings; straightforward iterator,
checked-option and integer syntax fixes resolved them. An initial gore iterator
edit failed to compile before its assignment was corrected. No failure is
claimed as a successful run.

Native `--first-simulation --ticks600` was launched and inspected. First view:
oversized food bodies obscured the constructor, and empty presentation meshes
produced repeated Bevy warnings. New-mode food now has a small stock-dependent
display scale; empty mesh slots are removed rather than uploaded. Unrelated
perpetual drips are omitted in this mode; uptake lines use committed Fed events.
A further live observation showed that the initial nearest-food policy parked
on a refilling meal and made a tight coil. Refill/build hysteresis was added and
its independent cycle test and honest-red fault now pass.

The final native capture at tick291 shows a seeking constructor at
[1131197,79880,745228] micro-units,226 intake,250 bound material,4 filaments,
book balance0. This is one inspected frame, not a frame-by-frame render audit or
human playtest. In the final cycle, the300tick numerical receipt above
supersedes the interim parked-policy measurement of1198 intake/299 bound.
Food sizing is presentation, not inferred material volume. Physical mouse
usability, broader player morphology and human visual acceptance remain open.

Only source/tests/protocol/report text are intended for Git. Native PNGs remain
local working diagnostics and are not authoritative scientific raw evidence.
During implementation no existing evidence was deleted or moved, and no external
publication was performed. The subsequently authorized commit/push applies the
archive gate below before publication.

Local diagnostic inventory:
- `/tmp/moebius-first-simulation.png`: 489182 bytes; SHA-256 `d9de9a94672e55e4295a38ab37e46cf72478c24b0b9d2e29008452750ada8ef5`.
- `/tmp/moebius-first-simulation-clear.png`: 382320 bytes; SHA-256 `9f44edc381d2b8fa6d16ef5afed3d23a20a5ef0e8575d7c7b3ef6a3b25e3dbca`.
- `/tmp/moebius-first-simulation-cycle.png`: 378612 bytes; SHA-256 `08cfb0ece4c9b2e6d34ff0a0d5be08dbe54f09e22b54c2bb3add3d65407b205a`.

### Tested source identities

Base main HEAD17989572fecf9874747c77a28286cae05ac9af3b; worktree dirty before
and after, with earlier food/audio/anatomy/camera work preserved. These hashes
identify the tested files, not an immutable or clean whole-repository snapshot.

```
ac9920ce07684bcf95065cc99fac81148b443692fc2c047db77432c0b65b153f  crates/moebius-morph/src/runtime.rs
eae7fbf9782fa522f48d679b076d14e515c910c9c53968ab36aa67c87d200ac1  crates/moebius-morph/src/filament.rs
030050819c7a64d4048e6406aa0b0959b7f6681ff91186dfacb4e45ff65df5c2  crates/moebius-morph/src/emission.rs
acc4f4864670e4876b4623249b905da70d6a7b93324882ffb882a61a179f40be  crates/moebius-morph/src/food.rs
4f35187209539eb6c5cbf19fc95297cf49a1018f37c3388faca46d8e66cc7857  crates/moebius-morph/tests/first_simulation.rs
066058c801fb12c75f95c48541b3f27a58eabfe82932368d64ee484636deec9b  crates/moebius-morph-lab/src/bin/moebius-world-witness.rs
f8674bb715aaf2d0522fe6aa01553433484272c40c916c45b46f677b0d90c683  crates/moebius-morph-lab/src/gore.rs
```

## Publication archive receipt — 2026-11-24

Before the authorized commit, all six retained camera/simulation diagnostic PNGs
were classified and copied to immutable Mermogon v6. The verified mounted UUID is
`D0A02EEBA02ED82C`. Archive/member hashes and restoration against every original
pass. All original working files and previous immutable versions remain.
See `ADDENDUM_V6_CLASSIFICATION.json`, `ARTIFACT_STORAGE.json` and
`RESTORE_VERIFICATION.json`. Restore with
`python3 python/morph_005_artifacts.py unpack NEW_DIRECTORY`; only the new
`first-simulation-camera-v6` subdirectory contains these six images. The local
`/tmp` paths above are working-copy provenance, not authoritative locations.

The raw archive is `v6/first-simulation-camera-v6.tar` under the manifest's
archive root. Git retains reproducibility source, reports and this verified
receipt; raw PNGs are excluded from the commit. Clean-commit full-workspace
validation is recorded separately in `refs/notes/first-simulation-validation`
for its tested commit; if that note is absent the full-suite gate is pending.


## Clean-commit publication validation

Implementation commit `d333fd4ff9746e7e92ca0ab684fb4887996836f5`, clean before
and after `cargo test --workspace --release --offline -- --test-threads=8`:
177 passed,0 failed,0 ignored. Workspace strict all-target Clippy and formatting
also pass. The exact177-test transcript, input/environment identity and sealing
procedure were classified, externally archived and restored with all hashes
verified; `git notes --ref=first-simulation-validation show d333fd4` contains
that receipt. This is local validation, not hosted CI or a new physiology claim.

The following documentation receipt commit retains the same source/tests and
is also checked clean. Its final clean-suite receipt is attached to its HEAD in
`refs/notes/first-simulation-validation`. The post-push remote-ref/clean-tree
receipt is in `refs/notes/first-simulation-publication`. Fetch these notes refs
explicitly; an absent note means the corresponding verification is pending.
