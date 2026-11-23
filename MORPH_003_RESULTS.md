# MORPH-003 — The first secretion

2026-11-23. **DONE** for this bounded slice. Focused mechanism gates passed, and the clean-tree workspace suite passed on implementation commit `a8cb705661f3dc19cdba22b97e97f63b5795665f`. The documentation commit that records this paragraph must itself be retested. This is a conserving material book attached to an anatomical nozzle. It is not metabolism, locomotion, learning, pipe construction, or a living organism.

Parent revision `1296ce4d338c5863eb023863e39a45554447f47f`. No external literature search. The motion rule is authored and recorded in `docs/morph_003/SUBSTRATE.md`.

## Decision

Nursery `Jet` / `advance_jets` / `SECRETE_ACTION` stay untouched. They are lattice reaction mass: life 4, nutrient and energy debit, body impulse, then flesh or waste. `FluidSample` and `TrailPoint` are not a free-space stock.

`MaterialBook` in `moebius-morph` owns quanta after `try_emit`. `moebius-core` was not edited. The anatomical flag `operational` stays false. Runtime operational becomes true only after an admitted parcel.

## Gates

| Gate | Status | Evidence |
|---|---|---|
| G1 conservation | PASS, focused | Experiments keep residual 0. Honest paths: duplicate residual +4, unrecorded deletion residual -4. Source stub of `residual` to `Ok(0)` failed `controlled_experiments_conserve_and_reproduce` at `duplicate_residual > 0`, exit 101. Restored SHA-256 `56efe0b5ae3916b3a46c179314c5263c627e307f9ce852e5fc2a938f30f4bbe7`. The deletion assert is the next check and calls the same function. The restored test passed. |
| G2 nozzle transform | PASS, focused | Translation, quarter-turn yaw (+X to -Z), body bend, and attachment station 0.22 all change or preserve the placed frame as specified. `nozzle.translated/rotated/bent/reattached` are true. Anatomical operational false. Runtime operational false before admission and true after. |
| G3 determinism | PASS in this build | Repeat and JSON reload hash `cff18b1d0174646fd6b852f6d7cff059894420420fe5d2809cb6641c77e90cfb`. Integer steps match under Rust toward-zero division. f32 pose quantization is same-build only. |
| G4 environment | PASS, focused | Same launch, 12 steps. Drag 0, drag 1000, and +X acceleration 4000 produce three different positions, residuals 0. Cardinal check: 6 undragged steps displace +Z by 120000; full drag stays at the origin; 10 steps of acceleration 3000 from rest displace +X by 165000. |
| G5 persistence | PASS, focused | JSON and filesystem round trip match. A forged quantity 9 is rejected. Preview save/load matches the state hash. |
| G6 bound | PASS, focused | Capacity 2 rejects the third parcel with residual 0 and reservoir 6. Capacity 4096 holds 4096 and rejects the next. Parcel is 96 bytes; the full list is 393216 bytes. Release fill of 4096 admissions plus 100 steps: 138027 µs on this machine. |
| G7 regression | PASS on `a8cb705` | Clean tree. `cargo test --workspace --offline -- --test-threads=8` exit 0. 136 tests, 0 failures. rustc 1.99.1 (`58cf8f9c3` 2026-10-13). Focused morph and lab tests and clippy `-D warnings` had already passed. The documentation commit that records this row must be retested. |
| G8 observability | PASS, focused | Ledger reports initial, reservoir, in-world, sinks, and residual without reading the renderer. The release report is `docs/morph_003/REPORT.json`. |

## Experiments

Showcase seed `20261123`, one nozzle, channel `future_conserved_substrate`.

| Run | Result |
|---|---|
| A, emission off, 20 ticks | Reservoir 100, parcels 0, sinks 0, residual 0, tick 20. |
| B, stationary launch 50000, 10 steps, then a 10-unit body shift | Integrated position `[-13077, -1134568, 255321]` matches the model. After the shift the parcel is unchanged and the nozzle is at x `9987353`. |
| C, body velocity 1 world unit per tick, launch 0, 8 emits | First parcel stays at its birth point. Separation from the nozzle at the end is `[8000000, 0, 0]`. |
| D | Drag-free, full-drag, and accelerated positions differ. Residuals `[0, 0, 0]`. |
| E, reservoir 5, request 3 | First parcel quantity 3, reservoir 2, second status `ReservoirExhausted`, residual 0. |
| F | Two scripted runs and a reload compare equal. |

A same-tick second emit is `DuplicateTick` and does not debit. `sink_parcel` of 4 leaves sinks 4 and residual 0.

## Laboratory

The Bevy lab observes the book. E toggles emission and arms the fixed clock. C toggles the clock without emission. U/I change rate. J/K stage the reset reservoir. F/V change drag. G toggles a weak +X acceleration. O toggles a uniform +Z flow. Q toggles an authored trajectory. X resets. F5/F9 save and load the book. Space pauses whichever clock is active. The old anatomy keys are unchanged.

Trail segments connect consecutive parcels of one nozzle. They are visualization. There is no floor. The preview spends at most four ticks per frame and says when it drops stalled time. The benchmark does not use that cap.

## Commands

rustc 1.99.1 (`58cf8f9c3` 2026-10-13). Release report command:

```sh
cargo run -p moebius-morph --release --offline --example morph_003_bench -- runs/morph_003
```

Report SHA-256 `ed97e76ce073a481abe17d222648af076a42e2c0f040dc0550ff4c479e0fbe67` (3694 bytes).

Focused:

```sh
cargo test -p moebius-morph --offline -- --test-threads=8
cargo test -p moebius-morph-lab --offline -- --test-threads=8
cargo clippy -p moebius-morph -p moebius-morph-lab --all-targets --offline -- -D warnings
```

Clean-tree certificate, HEAD `a8cb705661f3dc19cdba22b97e97f63b5795665f`, `git status --short` empty:

```sh
cargo test --workspace --offline -- --test-threads=8
```

Exit 0. 136 tests, 0 failures. Counted from the per-target lines: batch 1, core 6, farm 11, invariants 13, m4 7, m5 9, m6 4, neural 16, nursery 5, terrain 6, morph unit 2, composition 5, emission 4, forge 6, lab library 10, lab binary 2, view 29.

## Archive

Mermogon re-verified UUID `D0A02EEBA02ED82C` at `/run/media/cognis/Mermogon`.

New archive, not an overwrite:

`/run/media/cognis/Mermogon/Moebius-Automata/experiments/MORPH-003/2026-11-23`

Index SHA-256 `b0b96e92aab4aa91cf33551509d99389be29c08b1e548a044799ab14a4f69674`.

`v1/report-v1.tar` SHA-256 `2a91c824648ab272aa1520d6687483d1930447d3bb648fa5f4dacbb56f65cc9e`, 10240 bytes, one member `report.json`.

`python3 python/morph_003_artifacts.py verify` PASS. Unpack to `/tmp/morph003-restore-check` matched the git report byte for byte. The MORPH-002 archive was not modified.

There is no bulky tick trace. The benchmark state is the JSON book. The external copy is the immutable report, not a second authority.

## Limitations

Drag is not an atmosphere. Acceleration is not gravity and there is no floor. Flagella do not propel the body. The trajectory is a test input. The organism is not physiologically functional. No neural control, metabolism, reproduction, evolution, pipe, or hull transport was added. The preview can drop stalled ticks. Cross-platform f32 identity of the anatomical pose is not claimed. G7 passed on `a8cb705`. The documentation commit must be retested. No push. Do not start MORPH-004.
