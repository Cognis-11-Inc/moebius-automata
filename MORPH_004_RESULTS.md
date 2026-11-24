# MORPH-004 — The unobserved world

2026-11-23. **PARTIAL pending clean-commit broad-suite certification.** The
operator selected option2: promote MORPH-003's free-space substrate into the
canonical ship runtime, preserving the lattice World as a separate nursery
experiment. The focused runtime, neutrality, persistence, intervention,
coordinate and rendered comparisons pass. No push is authorized.

Base main20f0febb0cdc8e613ae36dfc6ed1abe529994d52. This implementation is
currently uncommitted. The earlier blocked investigation remains in
[INVESTIGATION_RESULTS.md](docs/morph_004/INVESTIGATION_RESULTS.md); its original
Mermogon archive is unchanged. The explicit ruling and authored metric mapping
are in [CONVERGENCE_DECISION.json](docs/morph_004/CONVERGENCE_DECISION.json).

## Implementation and authority

`moebius-morph/src/runtime.rs`: ShipRuntime privately owns one WorldSnapshot,
one timeline, shared AnatomySpec definitions, compact instance records, port
exposure inputs, intervention queue/causal events and material substates.
Every parcel admission and integration delegates to the byte-unchanged
MORPH-003 MaterialBook. Each book tick must equal the ship tick. Candidate
transitions commit atomically; overflow or invalid state leaves the prior
world intact. The core nursery source/laws/snapshots and anatomy compiler are
unchanged. No nursery nutrient/energy/physiology adapter is invented.

`witness.rs`: immutable snapshot inspection, separate presentation identity
cache, and an independently owned worker. Its latest-snapshot slot can replace
observation packets without blocking or deleting world steps. The worker takes
an explicit run length/publication interval; no camera or Bevy Time input.
It reaches its requested target with zero readers or a stalled renderer.

`examples/ship_run.rs`: general headless execution from a recorded initial
snapshot and optional typed intervention stream; saves initial/final state and
manifest. `morph_004_bench.rs`: population/replay/performance comparison.
`morph-lab/src/bin/moebius-world-witness.rs`: native free-space witness; only
visible geometry is created, exits remove visual entities/mesh assets. Entry
rebuilds from existing fixed-rest anatomy, world identity and position. Particle
points represent conserved parcels. There is no floor or fake propulsion.

The ship fixture is deliberately limited: prescribed constant instance drift,
rest-bound nozzle frames, environmental exposure, emission and actual integer
parcel transport. No per-organism detailed mesh is compiled during headless
ticks. It does not compute neural control, metabolism, reproduction, feeding,
interactions, ecological evolution or physically derived organism locomotion.
Rest-bound nozzles do not claim deforming-emitter physiology. Visible bodies use
the same rest pose. MORPH-001/002/003 editors and their animation remain intact.

## Tests and mandatory verdicts

| Gate | Verdict | Evidence and boundary |
|---|---|---|
| G1 runtime independence | PASS, reference fixture | General ship_run and benchmark have no Bevy/window/GPU/audio/input dependency; actual material/position/exposure transitions |
| G2 observer neutrality | PASS in tested build | All16 per-tick authoritative digests equal in A/headless,B/fixed sphere,C/moving/resizing sphere at100/1000/10000; independent repeat matches |
| G3 boundary continuity | PASS in sampled crossings | Unit cache crossings preserve world/parcel identities and quantities; native240 frames/six PNG captures show exits/re-entry, final state equals headless |
| G4 temporal integrity | PASS in bounded runs | Zero-reader/stalled-reader worker reaches exact target; worker lifetime survives app consumption; native120 ticks equal headless; no backlog deletion in ship runtime |
| G5 interventions | PASS, minimal contract | Target tick, typed operation, unique sequence, origin, accepted/applied events; deterministic sorting, duplicate/late/bad-origin rejection; invalid admission and failed tick leave state intact |
| G6 snapshots/replay | PASS in this build | Full state/config/future interventions/events persisted; continuation equality; invalid clocks, stocks, identity, local coordinates and duplicated emission events rejected |
| G7 ship coordinates | PASS in sampled envelope | Normalized1000m regions and integer micro-units; exact1µm differences, region crossings, instance/parcel translations at30,000km/1,500km; checked bounds; camera-relative f32 after subtraction |
| G8 performance | PASS for stated measurements |100/1000/10000 actual fixture instances,16 ticks/mode; snapshot/restore/replay/selection/crossing timings; peak benchmark RSS; rendered CPU/frame timings. Isolated GPU and volumetric cost NOT_MEASURED |
| G9 regression | FOCUSED PASS; broad gate PENDING | Existing morph/lab regressions plus10 new tests; strict workspace/all-target Clippy/format; baseline source hashes prove nursery/MaterialBook/compiler unchanged |
| G10 provenance | PASS for implemented boundary | Operator option2 recorded; one canonical ship owner reuses existing numerical substrate; nursery explicitly separate; no imported donor code, competing ship runtime or speculative IPC |

Nine honest-red/green cases are in [HONEST_RED.json](docs/morph_004/HONEST_RED.json):
observation-triggered command admission, tick loss, presentation cache identity,
invalid intervention admission, snapshot validation bypass, metric precision
loss, duplicate emission history, illumination-budget overflow and failed-tick
atomicity. Every deliberate mutation produced focused exit101; every restored
source passed. The observation fault injects a command from the test inspection
caller; the production inspector accepts an immutable snapshot and has no owner
mutation reference. Guard-specific mutations are recorded exactly; this is not
an assertion that arbitrary bugs or ecology have been exhaustively certified.

## Measured baseline

One release run in the recorded environment,16 ticks per population and mode.
Times are measurements, not confidence intervals or long-horizon claims.

| Instances | Headless world ticks/s | Final serialized state |
|---|---:|---:|
|100|20128.57|485926 bytes|
|1000|1792.03|4896870 bytes|
|10000|91.01|49482155 bytes|

Peak RSS of the entire benchmark child:311432KiB (~304MiB), including transient
world copies, replay and serialization. It is not isolated per-organism memory.
At10000, stationary/moving selection costs and cache costs are separately
recorded in [PERFORMANCE.json](docs/morph_004/PERFORMANCE.json), along with
snapshot construction, serialization, validated restore and replay costs.

The final native witness ran240 frames/six captures and120 world ticks.
Its complete final state and the separate general headless runner have digest
`4f34d8daad96fd1fc84860a53168ed0e6580a68fae095bbdf3ed7bf4b3154552`.
CPU observation/mesh/digest cost median0.561315ms/frame in that recorded run;
frame intervals were approximately16.67ms under vsync. This is end-to-end/CPU
evidence, not an isolated GPU timing. Boundary counts, tick samples and timings
are retained in RENDER_COMPARISON.json and the full raw frame report.

Full-state transactional copies, complete event history and linear witness
scans are explicit implementation costs. No spatial index, bounded decades-long
history, million-organism ecology or100000-instance prototype budget support is
claimed. Snapshot size increases with material/event history. Detailed geometry
is bounded by the chosen sphere in the native16-instance demonstration, not
benchmarked for arbitrary large visible populations.

## Illumination, coordinates and computational instruments

Metric mapping is an authored004 convention:1 anatomical unit=1m,1e6 integer
micro-units/m,1000m regions, absolute support +/-1e15micro-units. This does not
convert nursery sites to metres. Proposed vessel dimensions remain30,000km
length/3,000km diameter; no dense grid, hull, docking or atmosphere simulated.

LightPort exposure is an authored integer inverse-distance-square input plus
ambient, sampled independently of witness position. The port can be outside
the sphere. No radiative transport, occlusion, heat solver or fluid atmosphere.
The native witness uses simple studio lighting, separate from authoritative
exposure. Locked Bevy0.20 volumetric capability was inspected; the optional
bounded fog prototype was not implemented and GPU volumetric cost is unverified.

Rust owns ship transitions and validation. Bevy observes immutable snapshots.
Python retains offline scientific analysis; Lisp policy exchange remains a
proposed future typed/batched contract with no executable consumer or IPC.
[MORPH_004_ARCHITECTURE.md](MORPH_004_ARCHITECTURE.md) preserves the archaeology,
convergence ruling and producer/consumer map; [donor assessment](MORPH_004_DONOR_ASSESSMENT.md)
links primary sources and separates consultation from local validation.

## Reproduction, failures and storage

```
cargo test -p moebius-morph --test runtime --offline
cargo test -p moebius-morph -p moebius-morph-lab --offline -- --test-threads=8
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo fmt --all -- --check
cargo run --release -p moebius-morph --example ship_run --offline -- NEW_OUT 120
cargo run --release -p moebius-morph --example morph_004_bench --offline -- NEW_BENCH
cargo run --release -p moebius-morph-lab --bin moebius-world-witness --offline -- --ticks 120 --population 16 --record NEW_CAPTURE
```

rustc1.99.1, local offline dependencies, same-build f32 anatomical compilation;
integer transitions/digests compared exactly. No clean portable build on a fresh
machine or cross-platform f32 bit identity is claimed. Native witness hardware
is the available NVIDIA/Vulkan machine; GPU timestamp cost not measured.

The initial native witness failed finalization because App::run consumed its
resources. The fixed owner/telemetry survive outside App, with a new lifetime
regression. Original six failed-run captures and panic log are retained; its exact original
failing source/binary digest was not captured before repair. Benchmarkv1 also
had a varying radius in its stationary-labelled mode; it is superseded as a
protocol defect, not used for final fixed-sphere claims. See TRIAL_CHRONOLOGY.json. All
Clippy/compile failures, all benchmark versions, and all red/green runs remain
preserved. None is relabelled PASS. Earlier BLOCKED investigation remains intact.

The Stable Diffusion Atlas is an independent knowledge layer, with location and
hashes in docs/morph_002/ATLAS_REFERENCE.json. Material evidence stays on Mermogon
UUIDD0A02EEBA02ED82C. Runtime raw artifacts are classified and added as a new
immutable archive version, preserving the original investigation archive. Exact
archive/member hashes and restoration receipts are in docs/morph_004/.

```
python3 python/morph_004_artifacts.py verify
python3 python/morph_004_artifacts.py unpack NEW_RESTORE
python3 python/morph_002_atlas.py
```

Raw logs, screenshots, full states/frame reports, measured trials, selected
source and Atlas snapshots stay external; reports/configuration/source identity,
checksums and reproducibility metadata stay in Git. Clean local commit/full
workspace certification is the final remaining gate. No push or MORPH-005.
