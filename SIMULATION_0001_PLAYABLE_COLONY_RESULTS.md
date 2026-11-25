# SIMULATION-0001 current repair checkpoint

**PARTIAL — 2026-11-25.** Collision, cache, feeding-cost and authored exploration
repairs are implemented; final checkpoint checks and matched performance work are
pending. Latest completed run still peaks at1.86seconds/tick. See
[current repair status](docs/morph_005/SIMULATION_0001_REPAIR_STATUS.md) and
[coordinate audit](docs/morph_005/COORDINATE_DOMAIN_AUDIT.md).

The following investigation is preserved as its historical pre-repair receipt.

# SIMULATION-0001 previous investigation

**PARTIAL — 2026-11-24 (America/Toronto).** The requested playable colony is
not delivered. The operator observed the new native run trapped in secretion.
The causal investigation is [recorded here](docs/morph_005/SIMULATION_0001_COLLISION_DIAGNOSIS.md).
Wrong slide-normal handling is reproduced by a failing focused test. Candidate
corrections can overwrite earlier ones without a final re-sweep. Own hollow
pipes dominate the saved state's cost, while stationary free secretion parcels
also stop bodies. **Collision-response repair is not implemented.**

The earlier report below is preserved as history. Its green barrier and workspace
checks do not establish absence of these newly reproduced defects. Current
changes are uncommitted on main over `abdb664738a23c1aaf37c46810d26728f88c7d2f`.
No push or clean-commit certification of this continuation.

## What this continuation established

- Current-source baseline reproduced reference digest `7b68517ba0f82ed979469ef059db5f151243eb6999fe8d37f2e58d69940c2afe`.
- An interval AABB bound and exact-input cast memo preserved every 420-tick row
  and the complete final snapshot. Preliminary mean tick changed from 24.839
  to 24.097 ms, not enough to solve the authority lag. No proxy replaced the
  hollow mesh. With final distance/contact instrumentation, the final exact
  compatibility run averaged 24.976 ms (p95 89.427, p99 108.712, max 597.942).
  A robust speed improvement is not established.
- Creator startup now accepts founding count, world seed and finite abundance.
  F2/F3 adjust founders, F4 increments world seed, F7/F8 adjust free condensate;
  CLI values also initialize the creator. Existing saved-design startup remains.
  The default overlay is compact; B retains full details. These are implemented
  changes, not proof of a playable simulation.
- Native two-founder seed 11, free condensate 4000, reached three bodies with one
  paid birth at event tick 360. It completed 720 ticks and matched initial-state
  replay exactly (`b509ab9d302d3a6abe888558b582a8219562d895bdcc953cce7ce23a64b332bc`).
  Mean authority tick 189.610 ms, p95 654.742 ms, p99 697.509 ms, maximum
  2061.339 ms. Mean retained draw CPU 2.458 ms. The 4096-frame log cap was
  reached; authoritative ticks continued. The trap is a failed playability
  outcome even though the recorder's completion status is PASS.

## Registered finite-resource observations

Protocol: [SIMULATION_0001_EXPERIMENT.json](docs/morph_005/SIMULATION_0001_EXPERIMENT.json).
Three predetermined seeds, unchanged biology, explicit free condensate 4000.
Longitudinal target 1200 ticks, soft 60-second measurement budget. Scaling target
120 ticks, soft 20-second budget; those short cases cannot assess reproduction.
Replay runs outside the measurement budget. All completed reports below have
exact replay and material gap zero. Runs overlapped native capture and focused
checks; timing is observed throughput under that load, not isolated capacity.

| Case | Final tick | Bodies | Births | Generation | Mean ms/tick | Completion |
|---|---:|---:|---:|---:|---:|---|
| longitudinal-f1-s77 | 819 | 2 | 1 | 1 | 73.281 | PARTIAL |
| longitudinal-f1-s11 | 1178 | 3 | 2 | 1 | 50.623 | PARTIAL |
| longitudinal-f1-s23 | 494 | 2 | 1 | 1 | 121.137 | PARTIAL |
| scaling-f5-s77 | 120 | 5 | 0 | 0 | 100.387 | PASS |
| scaling-f5-s11 | 120 | 5 | 0 | 0 | 102.115 | PASS |
| scaling-f5-s23 | 120 | 5 | 0 | 0 | 83.249 | PASS |
| scaling-f10-s77 | 63 | 10 | 0 | 0 | 318.093 | PARTIAL |
| scaling-f10-s11 | 72 | 10 | 0 | 0 | 275.349 | PARTIAL |
| scaling-f10-s23 | 76 | 10 | 0 | 0 | 260.280 | PARTIAL |

The three one-founder windows each produced paid offspring without lowering
birth thresholds or moving food to the mouth. This establishes reachability,
not continued competence: the observed offspring and founders can become trapped.
All three longitudinal cases ended on their wall budget. Ten-founder cases also
ended on their wall budget. Targets 30, 100 and 300 were refused by the existing
rule ceiling, not empirically measured. Highest population measured in this
continuation: 10 placed founders, not 10 descendants. The historical sixteen-
founder result remains below. No sustainable colony size was established.

Mortality and metabolic consumption remain unimplemented. Deaths/extinction and
colony survival duration therefore are **not measurable under current rules**.
Finite stock, light gating and rooted growth do not establish the proposed full
spore ecology. No biology was added to manufacture a successful experiment.

## Launch and evidence

```sh
cargo run --release --offline -p moebius-morph-lab --bin moebius-world-witness -- --first-simulation --probe-world --founders 2 --seed 11 --abundance 4000 --ticks 1200
```

This reproduces a world with known collision/playability defects. In the creator,
use the same binary without `--first-simulation`; colony CLI settings are retained.
For a saved design, add `--anatomy PATH` to a direct first-simulation launch and
omit `--probe-world`. Existing play/pause/step/reset/rate/camera/F5 controls remain.

New raw evidence, diagnostic source, hashes and restore receipt:
[SIMULATION_0001_CAUSAL_STORAGE.json](docs/morph_005/SIMULATION_0001_CAUSAL_STORAGE.json).
Focused tests and static checks are recorded there. A clean-commit full suite is
pending; it must not be used to conceal the expected-red slide regression.
Next dependency is the bounded collision-response repair described in the causal
report, followed by this saved-state case and a fresh complete-history replay.

---

## Preserved earlier Grok receipt


Status: **PARTIAL**. Date: 2026-11-25. Measurements were taken on dirty `main` at `5a83c21fac6b6fb3032e40ce608f097a0a86356b`. That source is local commit `e863824f13422f5bf83a03c604a79eb85152db62`. The clean suite passed there. Nothing was pushed. Rustc 1.99.1, Cargo offline. Collision behavior is the validated fill-on-miss sample sphere. This pass did not replace the hollow trimesh.

The first colony can be launched, watched, and measured. It was not made to flourish. No natural window in this experiment produced an Arachnitech birth. No Arachnitech died, because the runtime has no removal rule. Food is a finite condensate. Light does not create biomass.

## Launch

Default one-founder receipt, unchanged:

```sh
cargo run --release --offline -p moebius-morph-lab --bin moebius-world-witness -- --first-simulation --ticks 3600
```

Founding group. `--founders` is 1..=16. `--seed` defaults to 77 with `--probe-world` and to 20261123 for the bare first simulation. `--abundance` is the free condensate left after the founding Blood-Sacs are paid. Home frames one body, or the group when more than one Arachnitech is present. Play, pause, single-step, and 1x/4x/16x are the existing playback: a rate of N runs N authoritative ticks, then waits one tick interval. The creator's Start Simulation button still opens one founder.

```sh
cargo run --release --offline -p moebius-morph-lab --bin moebius-world-witness -- --first-simulation --probe-world --founders 2 --seed 77 --ticks 180 --mesh-budget 8
```

Headless scale record, new directory only:

```sh
cargo run --release --offline -p moebius-morph --example colony_scale -- /tmp/moebius-colony-scale-20261125
```

## Highest verified population

**16 placed founders, 0 births.** That is the habitat rule ceiling (`max_agents` 1..=16). Targets of 30, 100, and 300 were refused with `founding population is outside the habitat budget of 16` and were not ticked. Sixteen founders on showcase seed 77 completed 60 ticks at a mean of **790.071 ms/tick** (p95 1619.274, max 1653.925), 97,429 narrow-phase queries, 341 pipe nodes, material gap 0, 2 meal events. That is about 1.3 authoritative ticks per second. It is a measured population, and it is the limit of this runtime.

The population above is the founding count. Reproduction did not increase it.

## Sustainable speed

Contract world, founders 1, seed 77, 420 ticks. Digest `7b68517ba0f82ed979469ef059db5f151243eb6999fe8d37f2e58d69940c2afe`, the same digest as `/tmp/moebius-perf-a-fill`. Material gap 0. Queries 15,645. Pipe builds 623, hits 905, rejects 971.

| | This run | Prior fill-on-miss run |
|---|---:|---:|
| Mean tick | 24.358 ms | 24.895 ms |
| p95 | 85.554 ms | 88.049 ms |
| p99 | 112.557 ms | 115.362 ms |
| Max | 569.275 ms | 578.153 ms |
| Resolve | 8,061,816,195 ns | 8,248,161,462 ns |

Mean resolve on the prior run was 19.638 ms, about 96% of it in swept queries. This run did not reduce that query count. Headless VmHWM for the one-founder process was 10,108 kB. The witness process for the two-founder recording reached VmHWM 291,972 kB and VmRSS 221,320 kB.

Showcase seed 77, collisions on, construction horizon equal to the requested tick count so emission stays enabled for every measured tick:

| Founders | Seed | Ticks | Stop | Mean ms | p95 ms | Max ms | Agents | Births | Meals | Nodes | Queries | Gap |
|---:|---:|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 77 | 420 | completed | 24.358 | 85.554 | 569.275 | 1 | 0 | 42 | 58 | 15645 | 0 |
| 1 | 11 | 180 | completed | 12.783 | 24.192 | 291.098 | 1 | 0 | 14 | 8 | 6703 | 0 |
| 1 | 23 | 180 | completed | 22.858 | 102.966 | 179.134 | 1 | 0 | 31 | 13 | 10943 | 0 |
| 2 | 77 | 180 | completed | 42.321 | 165.760 | 301.908 | 2 | 0 | 22 | 32 | 11492 | 0 |
| 4 | 77 | 77 of 120 | one tick over 2 s | 157.384 | 269.603 | 4574.479 | 4 | 0 | 18 | 80 | 8632 | 0 |
| 8 | 77 | 90 | completed | 269.686 | 863.975 | 1620.475 | 8 | 0 | 7 | 176 | 24154 | 0 |
| 16 | 77 | 60 | completed | 790.071 | 1619.274 | 1653.925 | 16 | 0 | 2 | 341 | 97429 | 0 |

Deaths are 0 in every row. Peak agents equal the founders. Condensate at the end of the contract run was 3606. Stored pouch food at tick 420 was 128. Exact rows are in the archive `summary.json`.

Pipe density and population were not fully separated. One founder grows from a handful of nodes to 58 over 420 ticks, and the same world's earlier percentile split (p50 about 8.2 ms, p95 about 88 ms) is the pipe-growth cost of a single body. Sixteen founders already have about 340 nodes by tick 30, because every body deposits. Far chunks are rejected (971 rejects on the contract run, 84,769 on the 16-founder run). The queries that remain are nearby hollow trimeshes. Headless memory stays under 18 MB at 16 founders. The tick cost is collision, then presentation is a separate witness cost.

A two-founder native session, mesh budget 8, measured the split directly. Authority mean tick 43.532 ms, p95 169.861, max 305.544. Publish mean 0.007132 ms. Draw CPU mean 1.442 ms. Frame interval mean 16.651 ms, which includes the pace sleep. Body rebuilds 365, reused 960. Pipe meshes generated 48. The HUD tick rate is a 0.25 s wall-clock sample of completed ticks. The frame rate is the presentation rate. They are different clocks.

## Reproduction and colony lifetime

No `Event::Reproduced` occurred in any scale row or in the native 180-tick session. Arachnitech birth requires `external_input > 0`, reservoir at least `child_stock + 96` (160), `birth_interval` of 360 ticks, and room under `max_agents`. On the contract run the pouch at tick 420 was 128. The eligible window opened with the pouch still below 160, because construction had spent the surplus. That is the measured outcome.

`paid_offspring_are_active_and_resume_exactly_after_reload` passed in this session's habitat suite (11/11). That test sets `birth_interval` to 60 and does produce a paid, active child that survives reload. The mechanism is present. The natural showcase windows did not meet it.

There is no Arachnitech mortality or removal rule. Deaths in the overlay are 0 for that reason. Every measured colony was still at its founding count when the window ended. Simulated durations were 7.00 s (420 ticks), 3.00 s (180), 1.50 s (90), and 1.00 s (60), at `TICK_NANOS` 16,666,667. Those windows show feeding, pipe deposition, and survival of the founders. They are not a longitudinal stability result, and they are not an extinction.

## Blood-Sac ecology that actually runs

In habitat mode the condensate is finite. A rooted sac with exposure at least 30, reservoir below 64, and a position beside flesh can take up to 2 units per tick from that condensate. Germination every 60 ticks pays 16 onto the newest pipe node when biomass is at least 16 and the sac count is under `max_sacs`. Rooted sacs can endow 12 to an offspring on a pipe node every 90 ticks. Feeding, when the eater is in range and has room, moves sac stock into the pouch and leaves at least 1 in the sac. Unrooted sacs do not grow. Habitat mode returns before sac hemorrhage, so the spore burst does not run. Light does not mint biomass. The playable experiment uses the recorded opening stock.

## Native visual session

`/tmp/moebius-colony-native-20261125b`. Release witness, `--first-simulation --probe-world --founders 2 --seed 77 --ticks 180 --mesh-budget 8`. Status PASS. Tick 180/180. Frames 658. Frame log not capped. Headless replay equal. Final digest `91ddb63588132a8125f11279361c2143d6791ea3c637fb8602e73af86568ba19`, the same digest as the headless two-founder row.

Opening frames show both Arachnitechs, flagella, and Blood-Sacs in frame. By tick 180 the followed body has moved, the pouch reads 84/1024 with 68 eaten, stored food across both bodies is 99, sac food is 173, nodes are 32, rooted sacs are 2, births are 0, and pale pipe segments remain in view. Movement is the authored habitat step. The rendered writhe is bend 0.72. The collision hull is bend 0.12. Writhe clearance still applies to solids only. An earlier capture of the same world, `/tmp/moebius-colony-native-20261125`, has the same digest. Its overlay reported a per-frame tick rate that read 0 or about 60 while the authority tick was slower. The cited session uses the 0.25 s sample.

The recording restored `world-initial.json` and advanced 180 ticks to the same final snapshot. Interactive F5 was not pressed during the capture. Playback controls in the report are empty because the recording drives the clock.

## Gates

| Gate | Result |
|---|---|
| G1 World integrity | ShipRuntime remains the authority. The witness replayed the saved snapshot. |
| G2 Collision | Contract digest reproduced. Honest-red: `resolve` returned immediately, `canonical_world_blocks_motion_and_exposes_biological_contacts` panicked `presented anatomy penetrated the barrier`, source restored byte-identical, the same test passed. |
| G3 Conservation | `material_gap` 0 on every measured row. Emission suite includes `controlled_experiments_conserve_and_reproduce`. |
| G4 Population | Founders are paid initial conditions with `parent: None`. Births stayed 0. The forced-interval offspring test passed. |
| G5 Structure | Nodes remained after the bodies moved. Lumen and seal tests passed in the collision and habitat suites. |
| G6 Observation | Two-founder headless digest equals the witnessed digest. Headless equal is true. |
| G7 Reproducibility | Seeds and founder counts are in `summary.json`. The native initial snapshot replayed exactly. |
| G8 Performance | Measured above. Sixteen founders are the ceiling. They are not interactive. |
| G9 Playability | Release witness launches. Two founders stay in a usable opening view and run 180 ticks. Four founders produced a 4.574 s tick. |
| G10 Regression | Habitat 11/11, collision 8/8, emission 5/5, and witness bin 8/8 passed on the dirty tree before the commit. On clean `e863824f13422f5bf83a03c604a79eb85152db62`, `cargo test --workspace --offline -- --test-threads=8` passed 212 tests and 0 failures. `cargo clippy --workspace --all-targets --offline -- -D warnings` and `cargo fmt --all -- --check` exited 0. rustc 1.99.1. Log `/tmp/moebius-clean-suite-e863824.log`. |

The clean-commit suite ran on that commit with a clean tree before and after. A later documentation commit records this paragraph and was not itself the suite subject.

## Collision change in this pass

`collision.rs` gained a `PipeCacheKey` alias and the sphere-gate test borrows the surface with `from_ref` instead of cloning it into a one-element slice. Resolve, the cache key, and the fill-on-miss rule are the same. The contract digest above is the check.

## Known limits

- One showcase Arachnitech still averages about 24 ms per tick on a 58-node network. The p99 is above 110 ms. A steady 60 Hz authority tick was not measured.
- Founding populations above 16 are rejected. Eight and sixteen founders are collision-bound.
- Four founders, seed 77, stopped at tick 77 because one tick took 4.574 s. The cause of that single stall was not isolated.
- Natural reproduction did not occur in these windows.
- Arachnitechs do not die, so a survival curve cannot show extinction.
- Habitat mode does not run the spore burst. Condensate is not refilled by light.
- Writhe clearance covers solids only. Visual anatomy and the tight hull still differ.
- An exterior collision primitive was not added.
- Nothing was pushed. The clean suite above does not make the colony a 60 Hz or self-sustaining result.
- The live atlas inventory for the two ship-canon notes lagged the 2026-11-23 22:28 operator-size text. Those inventory hashes now match the vault. `python3 python/morph_002_atlas.py` passed at 2026-11-25T02:52:07Z (45 notes, 197 links). The sealed MORPH-005 `v1/atlas-after.tar` still stores the earlier wording.

Archive: Mermogon `/run/media/cognis/Mermogon/Moebius-Automata/experiments/SIMULATION-0001/2026-11-25`, volume UUID `D0A02EEBA02ED82C`, verified mounted on this run. Hashes are in that directory's `MANIFEST.sha256`.
