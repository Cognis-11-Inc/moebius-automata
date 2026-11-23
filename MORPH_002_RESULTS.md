# MORPH-002 — Composable anatomy

2026-11-23. **PARTIAL.** The MORPH-002 success criterion is met on the dirty working tree: an operator can compose independent structures, inspect and animate them, save and reload them, and reproduce the geometry. Clean-commit broad-suite certification remains blocked because commit and push are not authorized. This is an authored anatomical preview. It does not establish swimming, secretion, learning, or a living organism.

Base revision `9171f0dada9e66c4e92dc548c6ed8a552f9ab1fa` on `main`. The MORPH-002 diff is uncommitted. No external scientific literature search was performed for this slice.

## Archaeology

| Concern | Path / symbol | Status | Input -> output | Consumer | Authority | Tests | Reuse ruling |
|---|---|---|---|---|---|---|---|
| Version1 worm | `WormSpec`, `CompiledWorm`, `generate` | active canonical | seed/JSON -> posed surfaces | MORPH-001 lab path, `examples/validate`, forge tests | headless `moebius-morph` | `tests/forge.rs`; fresh population | kept; attachment math extracted, output unchanged |
| Version2 composition | `AnatomySpec`, `CompiledAnatomy`, `compose` | extension in the same engine | component list -> body plus swept parts | `CompositionSession`, lab, `validate_composition` | headless engine | `tests/composition.rs` | new schema beside v1, migrated on load |
| Interactive editor | `CompositionSession::edit`, `moebius-morph-lab` `main` | active lab | keyboard edits -> validated spec -> meshes | operator preview | lab is a consumer, not the geometry authority | editor unit tests, walkthrough, `replay_lab` | replaced the v1 slider session inside this binary |
| Simulation organisms | `moebius-core`, viewer, nursery | untouched | world tick -> physical state | production simulation | core | workspace tests include them | no bridge added |

Branch inventory on 2026-11-23: only `main` and `origin/main`. No stash entries. One worktree, this checkout, at `9171f0d`. Worm_A remains a separate sculpted viewer asset. The lab does not drive it.

## What the working tree contains

- `AnatomySpec` version 2. Each flagellum, spine, and secretion nozzle has its own id, trunk parent, saved station/azimuth, yaw/pitch, and shape parameters. Triangle bindings are recomputed when the body is posed or regenerated.
- Version1 JSON still loads through `anatomy_from_json`. Migrated bodies keep their parameters. Flagella become independent components.
- The nozzle stores an output frame and `operational: false`. Material emission is absent.
- The primary mouth remains the existing opening. Secondary mouths are absent.
- The Bevy lab selects components from the keyboard, adds, duplicates, deletes, and edits them, then recompiles before replacing accepted state. Attachment markers are drawn. The default scene has no ground plane.
- Atlas notes live on the Stable Diffusion Drive at `/run/media/cognis/Stable Diffusion Drive/MOEBIUS_AUTOMATA_Atlas`. The repository keeps only `docs/morph_002/ATLAS_REFERENCE.json`.

## Gates

| Gate | Status | Evidence |
|---|---|---|
| G1 specification integrity | PASS, focused | Unique ids, reserved trunk 0 and mouth 1, parent must be 0, operational nozzle rejected. `tests/composition.rs`. |
| G2 composition | PASS, focused | Counts 0, 2, 8, and 16 flagella across eight poses. Population seeds 0–99 cover the same counts with independent positions. |
| G3 geometry | PASS for sampled poses | Finite audits inside the population and tests. Maximum root error 0. Showcase near-closure gap 0.010323475. Minimum double area 2.491658779e-05. Global self-intersection is undemonstrated. Roots are overlap-mounted. |
| G4 editing | PASS, focused | Add, duplicate, delete, per-component edits, save/reload, and a refused edit that leaves the accepted spec in place. |
| G5 determinism | PASS in this build | 100/100 version2 configurations, 800 poses, 100 distinct rest digests, save/reload match. Same-build replay. Cross-platform bit identity is undemonstrated. |
| G6 compatibility | PASS | Fresh version1 population: 100/100, 800 poses, attachment error 0. All 108 archive members match the MORPH-001 `population-v1` hashes. |
| G7 regression and cost | focused PASS; clean-commit PENDING | Morph tests, Clippy, and `cargo fmt --all -- --check` passed. `cargo test --workspace --offline` exited 0 on this dirty tree and lists 124 tests. That run is not a clean-commit certificate. |
| G8 atlas | PASS for note links | `python3 python/morph_002_atlas.py` on the Stable Diffusion Drive, UUID `8CA451CDA451BA82`: 28 notes, 94 internal links, zero errors. The atlas is navigation. It does not validate biology. |

Honest-red, same day, after the focused suite and before this report. Both guards were restored and the source hashes match `docs/morph_002/SOURCE_IDENTITY.json`.

- Nozzle `operational` check removed from `AnatomySpec::validate`, then `independent_identity_validation_and_typed_metadata` failed at `tests/composition.rs:23` (`s.validate().is_err()`), exit 101. Restored file SHA-256 `f346e40f5c4699d976b0d9fea6bf016a7a9022d1dcebb68a9ea2d7735a76bd30`. The same test then passed.
- `CompositionSession::edit` was pointed at the already accepted spec instead of the candidate. `refused_geometry_edit_does_not_replace_accepted_state` failed at `composition.rs:374` (`s.edit(Edit::Adjust(-100.)).is_err()`), exit 101. Restored file SHA-256 `222e707c2a5b905f831b515cf9905c3909f1789aec0386e8112a8804a126367f`. The same test then passed.

Duplicate ids, a non-trunk parent, and a bad reload remain covered by the passing rejection tests. Those three were not source-mutated in this cycle.

## Measured cost

Release compile time over the 100 version2 definitions: minimum 533 µs, median 1110 µs, maximum 1979 µs.

Representative rows, including the seed rule that adds a spine on even seeds and a nozzle when `seed % 3 == 0`:

| Seed | Flagella | Components | Vertices | Triangles | Compile |
|---|---|---|---|---|---|
| 0 | 0 | 2 | 1876 | 3740 | 838 µs |
| 1 | 2 | 2 | 2698 | 5384 | 953 µs |
| 2 | 8 | 9 | 4598 | 9156 | 1176 µs |
| 3 | 16 | 17 | 6656 | 13240 | 1667 µs |

The walkthrough status line reported one mouth edit at 3336 µs and one reload at 3804 µs. Those are single on-screen observations.

## Walkthrough

`walkthrough-v1` exited PASS. Nineteen screenshots and matching JSON captures were written. Headless `replay_lab` rebuilt all 19. Body positions changed. Flagellar positions changed. Two seeds appear. Save/reload digest `b167c4c76dd207abff528e00ad8d6f3cae63a7520386eed67ecdf13802050851` matched.

Inspected frames show the organism suspended, with no ground plane. The open-mouth frame shows a tubular opening. Flagella and a short ventral stub are visible. Root markers are drawn. The composed frame selects nozzle 14 and reports aperture 0.600. The recessed cavity is established by the nozzle ray test in `mouth_seam_interior_and_nozzle_are_real_geometry`. The still camera does not by itself show that cavity.

Keyboard controls: Tab / Shift+Tab select; A adds a flagellum; P adds a spine; N adds a nozzle; D duplicates; Delete removes; Up/Down chooses a field; Left/Right edits; `[` `]` change body length; `-` `=` change thickness; `,` `.` set mouth aperture; M cycles the mouth; H loads the short-body showcase; R steps the seed; Space pauses motion; S saves; L reloads; right-drag and the wheel, or Alt+arrows, orbit. There is no click-to-select gizmo.

Preview motion stops at phase 99999. That cap was not reached during the walkthrough.

## Commands and outcomes

Environment: rustc 1.99.1 (`58cf8f9c3` 2026-10-13), x86_64-unknown-linux-gnu, EndeavourOS, kernel 6.18.60-1-lts, Ryzen 9 7950X, RTX 5060 Ti, NVIDIA 615.78.08, Vulkan. Working tree dirty. HEAD `9171f0dada9e66c4e92dc548c6ed8a552f9ab1fa`.

```sh
cargo fmt --all -- --check
cargo clippy -p moebius-morph -p moebius-morph-lab --all-targets --offline -- -D warnings
cargo test -p moebius-morph -p moebius-morph-lab --offline
cargo test --workspace --offline -- --test-threads=8
cargo test --workspace --offline -- --list --format terse
cargo run --release -p moebius-morph --example validate --offline -- NEW_V1_DIR
cargo run --release -p moebius-morph --example validate_composition --offline -- NEW_V2_DIR
cargo run --release -p moebius-morph-lab --offline -- --walkthrough NEW_WALK_DIR
cargo run --release -p moebius-morph --example replay_lab --offline -- NEW_WALK_DIR
```

Recorded directories, all under `/home/cognis/.cache/moebius-automata/morph-002/`: `baseline-v1-reread`, `population-v2`, `walkthrough-v1`. Each of those three runs passed. The workspace command exited 0 on the dirty tree. `--list` reported 124 tests.

```sh
cargo run --release -p moebius-morph-lab --offline
```

Default save path: `runs/morph_002/organism.json`. An existing file there is loaded. `--seed` applies when that file is absent.

## Archive

Mermogon was re-verified on this run: mount `/run/media/cognis/Mermogon`, UUID `D0A02EEBA02ED82C`, label `Mermogon`.

New immutable archive, not an overwrite:

`/run/media/cognis/Mermogon/Moebius-Automata/experiments/MORPH-002/2026-11-23`

| Bundle | Files | Bytes | SHA-256 |
|---|---|---|---|
| `v1/baseline-v1-reread.tar` | 108 | 3532800 | `25343219010d158ad850cc04c5421ec8942b3f91e501d841e293e3ff2729d799` |
| `v1/population-v2.tar` | 104 | 1075200 | `f4cbbcfe506a43ca5a1d96a5980c7918ce8f55b8b5e4df371efad3b076fa5a00` |
| `v1/walkthrough-v1.tar` | 40 | 6266880 | `522febd7146e49a44a66df11276801f8ae8aaff8126f5ecbbe3cd2817da14204` |
| `v1/logs.tar` | 3 | 20480 | `e6c99bdf61c0ee40bc4f4df2e05f9b7c42991cf125f31f5bcd5485e3878452d7` |

Index SHA-256 `558a634320b89c8ad2b545074fc8fceaa02e395a0fda5352edd5fcd76d1e3812`. Raw members: 255 files, 10445516 bytes. `python3 python/morph_002_artifacts.py verify` and `unpack /tmp/morph002-restore-check` both returned PASS, 4 archives, 255 files. The local cache remains a working copy.

Manifest: [docs/morph_002/ARTIFACT_STORAGE.json](docs/morph_002/ARTIFACT_STORAGE.json). Machine summary: [docs/morph_002/VALIDATION.json](docs/morph_002/VALIDATION.json). Source hashes: [docs/morph_002/SOURCE_IDENTITY.json](docs/morph_002/SOURCE_IDENTITY.json).

## Limits

- No neural controller, propulsion, metabolism, reproduction, or secreted material.
- Nozzle `operational` must stay false.
- One primary mouth. The closed pose keeps a positive seam.
- Component budget in validation is 256. Sixteen flagella is a tested count.
- No viewport picking gizmo.
- Long component labels wrap in the side panel.
- The lab binary is now the version2 editor. The old 1–7 / F / G key map is gone from this binary. Version1 definitions still load.
- Clean-commit workspace certification has not been run. `cargo test --workspace` was not repeated in the completion pass. The earlier dirty-tree run, on these same geometry-source hashes, exited 0 and listed 124 tests. That run is not a release certificate.
- No commit and no push were made.

## Next slice

The smallest following experiment is a measured substrate on the nozzle's material channel. `operational` stays false until a conserving emission test exists. Viewport picking and a simulation bridge are separate decisions. Neither is part of this slice.

## This completion pass

2026-11-23T17:20:12-05:00. HEAD `9171f0dada9e66c4e92dc548c6ed8a552f9ab1fa`. Tree dirty. Geometry sources unchanged from the archived population and walkthrough.

```sh
cargo fmt --all -- --check
# exit 0
cargo clippy -p moebius-morph -p moebius-morph-lab --all-targets --offline -- -D warnings
# exit 0
cargo test -p moebius-morph -p moebius-morph-lab --offline -- --test-threads=8
# exit 0: composition 5, forge 6, morph-lab lib 5, morph-lab main 1
python3 python/morph_002_artifacts.py verify
# PASS, 4 archives, 255 files, UUID D0A02EEBA02ED82C
python3 python/morph_002_atlas.py
# PASS, 28 notes, 94 links, UUID 8CA451CDA451BA82
```

The existing Mermogon archive was read and verified. It was not overwritten or replaced.
