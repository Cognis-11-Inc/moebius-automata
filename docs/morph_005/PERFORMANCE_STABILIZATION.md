# Performance stabilization — 2026-11-25

**ACTIVE / PARTIAL.** Operator designates this entire development phase as
performance stabilization. Additional biological complexity is held back. The
whole simulation must leave room for future metabolism, neural activity, chemistry,
gradients, reproduction, structural growth and evolution. Twenty milliseconds per
subsystem is not an acceptable budget. No smoothness or large-population claim.

Working engineering target for the present small colony: total authority mean
2ms, p99 8.3ms, maximum16.7ms, leaving game-frame headroom. These are proposed
engineering targets, not achieved measurements or an agreed universal capacity.

Acceptance includes congested saved worlds, longer execution with growing state,
population scaling, memory/allocation cost, native frame and authoritative tick
spikes. A recorder PASS, exact replay, smooth rendering or lower average alone
cannot close the performance gate. The current ceiling16 is not evidence for
enormous populations. New biology is not a way to mask or postpone this debt.

## Containment and reuse

Keep canonical ShipRuntime, finite paid material, actual collisions, hollow pipe
walls/bores, persisted provenance, observation neutrality and exact deterministic
outcomes for query optimizations. No collider substitution, coordinate redesign,
offscreen physics suppression or discarded ticks/events. The empty coordinate
domain was already measured and is not a material cost in matched local scenes.

| Concern | Path / symbol | Status / producer -> consumer | Reuse ruling and proof |
|---|---|---|---|
| Distance predicate | collision.rs within_distance | canonical shapes/relative poses -> interval-clearance decision | extend existing Parry BVHs and kernels, retain contact query/tolerances; original-distance comparison and complete history equality |
| GJK failure handling | collision.rs within_distance | library status + geometric support bound -> early decision or exact fallback | NoIntersection also denotes iteration exhaustion; status alone never establishes separation |
| Mesh preparation | collision.rs pipe_separated/pipe_key | sampled bound/content/time -> existing mesh cache | reject distant construction after fixing the age-zero alias; same state/contact evidence |
| Spawn placement | collision.rs spawn_pose; habitat.rs finish | canonical world + runtime cache -> legal pose | reuse the existing cache across searches; no new placement owner or debit rule |
| Causal history | runtime.rs tick | previous events + candidate events -> same ordered history | append only after every fallible gate, retaining atomic HistoryBudget refusal; no trimming |
| Numerical outlets | anatomy.rs CompiledAnatomy::output_frames; compiler.rs attachment_landmarks | validated anatomy/pose -> OutputFrame | existing attachment triangles, normal summation order and component-frame math; 72-pose bit equality against full meshes |
| Outlet consumer | runtime.rs tick / place_output | OutputFrame -> paid emission request | same existing book and assembly authority, no new physiology |
| Measurement | habitat_probe, witness, coordinate_domain_bench | canonical ticks -> reports/traces | reuse existing observers; immutable external raw archive before results commit |

Inspected source, call sites, current and historical contracts, existing tests,
recent Git history and retained experiment versions. Consulted local primary
Parry0.27 source for BVH traversal and GJK status semantics. No external scientific
study, exhaustive branch audit or alternate physics authority is claimed.

## Implemented candidates under final validation

The interval solver needs a distance predicate, not a globally nearest triangle.
Its recursive BVH search stops at the first confirmed near component, preserving
the original leaf calculations. GJK proximity mode may supply a separating support
plane or an upper distance bound; a roundoff band and the original exact fallback
cover uncertain cases. Rotational interval bounds, 0.1mm skin, query limit and
actual contact construction are retained. No physical proxy was introduced.

Tick candidates keep new events separately and append the retained history only
after successful admission. The history budget counts both. Runtime frames use
only the trunk triangles/normals needed by nozzles, instead of allocating and
auditing full posed component meshes on every numerical tick. Visible meshes
continue through their existing generator.

Rejected and retained: the direct closest-point replacement changed contact times
and normals at tick723 despite initially matching agent rows; it is absent from
production. A duplicate bounded-distance preflight offered little benefit. A
separating-plane prefilter did not improve the saved-scene timing. One recorder
build failed and copied a stale binary; its runs are explicitly excluded in the
raw INVALID_RUNS.json. No failed/null case was removed.

Protocol: [SIMULATION_0001_PERFORMANCE_PROTOCOL.json](SIMULATION_0001_PERFORMANCE_PROTOCOL.json).

## Measured acceptance matrix

Release/offline Rust1.99.1, AMD Ryzen9 7950X, powersave governor. Serial jobs; no CPU pinning or controlled ambient load. Raw source/binary identities are retained. Baseline is pushed3573e1a; optimized source is the recorded overlay. Percentiles use ceil((n-1)*p).

| Matched scene | Baseline mean ms | Optimized mean ms | Optimized p99 ms | Optimized max ms |
|---|---:|---:|---:|---:|
| Two founders,420ticks | 87.641 | 33.860 | 299.564 | 408.400 |
| continuation,3 paired runs | 37.808 | 18.890 | 203.756 | 204.981 |
| congested,3 paired runs | 203.507 | 183.746 | 409.641 | 409.641 |

The420-tick mean reduction is 61.4%. All matched initial/final snapshots and every tick/contact row are byte-identical, including additional20-tick eight- and sixteen-founder comparisons and the complete720-tick native world. Saved-scene repeats alternate execution order. Full histories are single matched runs; no broad capacity extrapolation.

| Initial founders | Completed ticks | Mean ms | p99 ms | Max ms |
|---:|---:|---:|---:|---:|
| 1 | 120 | 0.604 | 4.304 | 6.370 |
| 2 | 120 | 23.426 | 379.126 | 411.010 |
| 4 | 120 | 12.488 | 50.374 | 58.025 |
| 8 | 120 | 77.192 | 757.505 | 1043.214 |
| 16 | 120 | 105.140 | 691.168 | 759.219 |

All five scaling cases replay exactly with material gap0. They use the same anatomy, seed11 and free condensate4000, but placement/interactions differ with founder count. Four being cheaper than two is not a capacity conclusion. Counts30/100/300 are refused by the current ceiling16, not measured.

Long execution: **PARTIAL**, 2818/3600ticks (46.97 simulated seconds), soft180-second measurement budget. Eight bodies,7 paid births,1007nodes; exact replay and gap0. Mean63.161ms, p95199.405, p99322.880, max596.209. Founder1 max net displacement8.664m versus80.091m travelled; no large-distance competence claim.

| Long-run ticks | Mean ms | Bodies at window end | Nodes at window end |
|---|---:|---:|---:|
| 1–360 | 3.001 | 1 | 39 |
| 361–720 | 11.218 | 2 | 109 |
| 721–1080 | 19.369 | 4 | 363 |
| 1081–1440 | 50.618 | 5 | 582 |
| 1441–1800 | 80.633 | 6 | 798 |
| 1801–2160 | 131.146 | 7 | 876 |
| 2161–2520 | 121.000 | 8 | 956 |
| 2521–2818 | 93.535 | 8 | 1007 |

Native bare-anatomy run:720ticks,4 bodies/2 paid births/653nodes, exact headless replay,3016 retained frames, no frame-log cap. Authority mean52.901ms, p99398.966, max635.394. Frame interval mean16.669ms, p9917.138, max44.842. Draw CPU mean4.753ms/frame; publishing mean0.084ms/tick. Recorded frame intervals include pace/capture/startup; they are not simulation tick times. Closing normal-rendered PNG inspected, without claiming Operator visual acceptance.
A matched headless replay of that same native world averages51.822ms/tick. Its initial/final states and all tick/contact rows match the baseline. The extra baseline timing control was externally terminated during replay after completing720ticks; its timing receipt is unavailable and is not inferred.

## Memory and allocation traffic

Three measured congested ticks after one identical warmup, with executable-only allocation instrumentation: allocations399,876 → 364,084; cumulative requested bytes295,451,183 → 263,048,702; peak live requested heap16.839 → 9.306MB. Per-tick digests and complete final state match. Process peak RSS37.86 → 31.92MiB. Distinct embedded source hashes and independent build outputs verify the conditions.

The optimized case still requests about87.7MB per tick cumulatively. Allocation traffic is not resident memory, but this is substantial transient work. Long-run recorder VmHWM reaches74.45MiB; that includes final snapshot/serialization and recorder bookkeeping. A late-start proc-status trace distinguishes measurement from replay by final.json existence; it does not prove absence of a leak or isolate engine-only heap.

The first allocation pair accidentally used identical executables from a shared target directory and is EXCLUDED. It remains archived, together with the corrected observer-only harness patch and distinct-source v2 results.

## Validation and remaining gates

Focused restored source:81 tests pass (17 morph library,8 collision,7 composition,5 emission,9 filament,12 habitat,14 lab library,9 witness). Three intentionally injected normal/history-budget/resolve faults fail their focused assertions and restore byte-identically; the actual tetrahedron panic also has honest-red/restored evidence. The initial8-tick probe did not reach that panic and is not counted as honest-red. Strict Clippy/format checks pass. The full workspace suite passed228 tests, zero failures/ignored, on clean commit `cdc3acdc865ae48011e2c65f370428f1f312e915`. See the [exact source and archive receipt](SIMULATION_0001_PERFORMANCE_VALIDATION.json); this follow-up changes documentation only.

**Performance remains unacceptable.** The congested save, population spikes, growing-run slowdown and native tick stalls all fail the working total-authority budget. No new biology is admitted. No sustainable ecology, mortality, metabolism, learned motion or enormous-population capacity is established.

Next performance dependencies: attribute the currently uncounted admission/recovery work; remove occupied-state allocation/copy amplification; index nearby physical candidates without changing selection/order/outcomes; retain exact replay/contact gates and repeat this acceptance matrix. Do not replace geometry or relax physical rules merely to pass a timing target.

Machine-readable measurements: [summary](SIMULATION_0001_PERFORMANCE_SUMMARY.json).
Immutable archive/member hashes and restore instructions:
[storage receipt](SIMULATION_0001_PERFORMANCE_STORAGE.json) and
[manifest](SIMULATION_0001_PERFORMANCE_MANIFEST.json). All772 retained files were
hashed and restored successfully. Earlier rejected, failed and interrupted
experiments are retained, not overwritten.
