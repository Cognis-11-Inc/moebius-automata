# SIMULATION-0001 repair checkpoint — 2026-11-25

This is the initial3573e1a checkpoint record. See the subsequent
[performance stabilization measurements](PERFORMANCE_STABILIZATION.md) for current
implementation, validation and failed performance gates. History below is retained.

**PARTIAL.** Operator authorizes commit/push of the current work, then focused
regression checks and matched collision benchmarks. Smooth, lightweight simulation
remains the acceptance target; a completed recorder is not playable acceptance.

## Containment and existing authority

Keep ShipRuntime as the sole state owner, paid finite material, hollow pipe
geometry, actual collision admission, witness neutrality and replay. No coordinate
redesign, disabled product collisions, material deletion, new species, physiology,
or learning system. Extend the existing collision cache/query and habitat policy;
do not create another runtime. MORPH-004/005 and the original SIMULATION-0001
directive establish these boundaries. The later explicit exploration request
authorizes authored local travel goals within the habitat policy.

| Concern | Existing path/symbol | Status; input -> output | Consumer and reuse ruling | Proof / missing gate |
|---|---|---|---|---|
| Response | collision.rs resolve/along_surface/retreat | canonical poses + contacts -> admitted poses | ShipRuntime; narrow corrections | focused regressions exist; final run pending |
| Geometry | collision.rs CollisionCache/pipe_key | derived specs + mesh time -> shapes | existing admission/query paths; reuse | old-key honest-red and restored-cache replay recorded |
| Feeding | compiler.rs mouth_landmark; runtime.rs mouth_position_at | same spec/pose -> same mouth point | canonical intake; remove redundant full mesh generation | matched continuation snapshot/contact equality recorded |
| Deposit queue | runtime.rs automatic_deposits | paid parcels + capture radius -> eligible IDs | existing assembly admission; retain every parcel | unreachable-parcel regression exists |
| Travel | habitat.rs TravelMemory/prepare/turn_heading | local observations + bounded memory -> authored heading | existing policy; no learning claim | fed departure/reload and original pouch assertions recorded |
| Measurement | examples/habitat_probe.rs, coordinate_domain_bench.rs | existing ShipRuntime -> traces/counters | supporting observers only | final-source focused and clean suite pending |

Inspected source/call sites, tests, Cargo, recent history and preserved reports.
No exhaustive branch audit or external scientific research is claimed.

## Implemented, not yet certified on the committed checkpoint

Collision response corrects inward normal projection, composes candidate changes
and verifies them, uses actual stopping normals, and permits sampled nondeepening
translation out of existing overlap. This is bounded kinematic recovery, not a
continuous penetration theorem. Self-deposit exemptions cover the emitting nozzle
alone; the trunk remains physical. Linear motion uses Parry linear casts while
rotation retains the conservative interval detector. Shared body geometry,
value-validated spec identities, exact-input cast memoization and AABB exclusions
remove repeated work without replacing the hollow mesh.

Wet-pipe cache keys now include relevant geometry content and actual wet mesh time.
The former youngest-node-age key aliased times t and t+1 around future-node
admission, making warm/cold physics disagree. The retained old-key regression fails;
the replacement and warm/restored exploration test pass in the recorded focused run.

Travel stores at most 256 visited 2-metre cells, pursues local unexplored targets,
and uses refill hysteresis. Offspring start with empty travel memory. Bounded
spherical steering can turn toward an exactly-behind target, where the old
normalized blend could not turn. No learned competence is claimed.

## Retained measurements and failures

See [coordinate audit](COORDINATE_DOMAIN_AUDIT.md): matched scenes under supported
bounds from ±1 km to ±1,000,000 km have identical states, allocations and queries;
tick means change less than 1%. Empty coordinate extent is not the bottleneck.

Latest pre-checkpoint completed headless run: two founders, seed11, finite abundance
4000, 420 ticks, 3 agents/1 paid birth, replay exact, material gap0. Mean88.219ms,
median18.219ms, p95497.879ms, p99856.500ms, max1858.943ms. Founder maximum net
displacements3.724/3.953m; total paths16.691/16.541m. These are distinct measures.
Source identities are recorded per run. No final-source native playtest is claimed.

Matched 40-tick continuation before/after AABB containment correction:
mean64.710/39.328ms; max504.264/217.673ms. Outcomes differ slightly, so this is
corrective behavior, not exact-equivalence optimization. A longer earlier-source
run hit its budget at796/1800ticks, max3702.312ms; no smoothness claim follows.

Preserved rejected experiments: padded linear angular shortcut missed the thin
rotational barrier; nonlinear casts were slower; rotation unlocking failed active
offspring behavior. They are absent from production. The finite barrier regression
now spans ±20m across its transverse axes, preventing legal travel around its old
finite edge from being mistaken for crossing its wall. Original no-penetration
assertions remain. The seal fixture seals all old streams and verifies their
preservation. Original pouch threshold/window remain unchanged.

Final focused checks, full clean-checkpoint suite, matched pre-build rejection,
fresh distance/tail-latency measurements and final Atlas reconciliation are pending.
See the [repair archive receipt](SIMULATION_0001_REPAIR_STORAGE.json) for retained
raw versions, failures and source overlay. Checkpoint publication precedes the new
focused checks at the Operator's explicit request; it is not a validation receipt.
