# SIMULATION-0001 stabilization continuation

Entry: main `abdb664738a23c1aaf37c46810d26728f88c7d2f`, clean, 2026-11-24
(America/Toronto). Operator supplied the First Colony directive, authorizing
in-scope local commits and explicitly excluding push and new biology.

## Containment

Collision acceleration must preserve actual contact records, not only body
trajectories. Keep the hollow meshes, cache-fill order, query tolerances,
canonical ShipRuntime transaction and observation neutrality. No new species,
metabolism, mortality, learned motion, external authority or collision proxy.
Preserve all prior experiments, including rejected filters. No success claim
for a short run, founding count, or forced birth fixture.

## Authority and reuse

| Concern | Path/symbol | Status / input -> output | Consumer / authority / tests | Reuse ruling / missing seam |
|---|---|---|---|---|
| Swept detection | collision.rs `cast`, `CollisionCache` | active shapes + poses -> contact | `resolve`, canonical ShipRuntime; collision tests | retain mesh and query semantics; investigate conservative interval bounds |
| Policy and births | habitat.rs `prepare`, `finish` | active finite stock + local observations -> candidate motion/paid child | canonical tick; habitat tests | preserve rules; natural birth eligibility needs longitudinal measurement |
| Experiment | examples/habitat_probe.rs, colony_scale.rs | supporting initial state -> tick logs and reports | offline observer | extend existing recorder; short scale runs cannot establish survival/growth |
| Native operator interface | morph-lab witness `begin_ship`, Args | design/config -> existing Worker | sole runtime remains ShipRuntime; witness bin tests | reuse controls; creator currently ignores CLI founding configuration |
| Publication | Mermogon SIMULATION-0001 archive, existing Atlas | reports/evidence -> reproducibility/navigation | UUID-verified external storage | new immutable version, retain all earlier versions |

Paths relative to crates/moebius-morph unless stated. Inspected current source,
tests, Cargo, MORPH-004/005 architecture, habitat/collision contracts, handoff,
Git history and prior raw receipts. No exhaustive branch/history audit claimed.

## Target and proof obligations

First investigate an AABB lower bound *inside* the existing swept interval
query. It may reject only an interval whose entire motion remains separated;
it must not skip mesh construction or reorder contacts. Compare current-source
420-tick baseline, exact full snapshots and every tick/contact row. Measure
mean/p95/p99/max, query counts and memory. Reject the optimization if equality
fails or it brings no useful gain. Run collision, habitat and emission tests,
honest-red at canonical resolve, strict Clippy and formatting. Continue with
longitudinal finite-resource runs and native controls where practical. Report
missing biology as a limitation; do not manufacture it. Clean-commit broad
suite and immutable external evidence remain final gates.

CONSULTED: local implementation and cached Parry 0.27 source. No scientific
paper or external study consulted. IMPLEMENTED / VALIDATED: pending below.


## Observed outcome before response repair

IMPLEMENTED: conservative interval bounds, exact-input bounded cast memo, query
phase counters, creator founding controls, compact default telemetry and bounded
longitudinal recorder/registered driver. No collision representation or response
law changed. Final 420-tick trace and snapshot remain byte-identical to the fresh
baseline. Final instrumented mean 24.976 ms versus baseline 24.839 ms: **no robust
speed improvement established**. Query visits fell 15645 -> 14896, but that does
not meet the performance target.

VALIDATED focused final source: morph lib 5, collision 8, emission 5, habitat 11,
lab lib 14, witness bin 9 passed (52 total); strict workspace Clippy passed. The
canonical resolve-bypass honest-red failed with presented-anatomy penetration and
source was restored byte-identically. A distinct expected-red slide regression
proved an existing sign defect; source and log are archived, not silently greened.
Native 720-tick seed-11 colony reproduced the operator's stuck-body complaint and
replayed exactly. See SIMULATION_0001_COLLISION_DIAGNOSIS.md for counterfactuals,
limits and the pending response repair. Full clean-commit certification NOT RUN;
this continuation is uncommitted and is not a playable-release result.
