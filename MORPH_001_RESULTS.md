# MORPH-001 — The Anatomical Forge

2026-11-23. **SUCCESS for the defined first vertical slice**, with the limits below. The application independently generates a configurable worm, renders its actual surfaces, bends its body, operates a geometric mouth, moves two attached flagella and reproduces the structure from a saved definition. This is an authored anatomical preview, not a living/autonomous simulation result.

## What was delivered

- [Existing-system audit](MORPH_001_EXISTING_SYSTEM_AUDIT.md), completed before implementation.
- [Donor assessment and licensing history](MORPH_001_DONOR_ASSESSMENT.md), with exact commits/files/symbols and [file provenance](docs/morph_001/DONOR_PROVENANCE.json). No donor code or assets were imported. Donor A's historical MIT file is identical at the fork/common ancestor; upstream's later GPL change is recorded. Bundled third-party assets were not cleared or reused. Donor B targets Bevy0.19, so it was not added to this Bevy0.20 workspace.
- [Architecture/interface](MORPH_001_ARCHITECTURE.md): a headless Rust engine and a separate native laboratory. No LLM, service, proprietary runtime or Blender step is needed.
- `crates/moebius-morph`: strict version1 `WormSpec`, seed expansion, body/profile/curvature/resolution, mouth controls, two configurable flagella with surface coordinates, optional ridges, appearance, bounded validation, JSON persistence, `CompiledWorm::pose`, indexed `Surface` data, attachment frames and explicitly absent simulation binding.
- `crates/moebius-morph-lab`: a Bevy0.20 consumer with native parameter editing, orbit/zoom, authored body/flagellar motion, mouth cycling/manual aperture, seed selection, save/reload and an actual-input walkthrough. The engine is also consumed by headless population validation and capture replay, rather than being owned by the GUI.
- Portable [example definition](experiments/morph_001_worm.json). It is independent of Worm_A assets and existing organism definitions.

The body is an axial ring sweep with a closed tail, annular lip and recessed interior ending at a throat floor. A central ray test reaches that floor rather than a face-colour patch. At minimum aperture a positive slot remains to prevent degenerate triangles. Ridges modulate the actual surface. Each flagellum is a closed tapered tube; its root is evaluated from a body triangle's barycentric coordinates in every pose. Local attachment frames and capability=`decorative_unbound` are exported. CPU deformation preserves topology and recomputes normals. There is no claimed general skeletal rig.

## Mandatory gates

| Gate | Verdict | Evidence |
|---|---|---|
| G1 deterministic generation | PASS in recorded environment | 100 seeded definitions reproduced,100 distinct body-position digests; identical geometry for repeated definitions at each tested pose. Same Rust/x86_64 environment only. |
| G2 geometric integrity | PASS for sampled configurations/poses | 100 configurations ×8 poses=800; finite attributes, legal indices, positive triangle area, face/normal alignment, consistent winding, closed edge incidence and positive enclosed signed volume for each surface. Minimum double triangle area0.00002161930024. |
| G3 morphological variation | PASS | 100/100 configurations pass,100 distinct body-position digests. Length/radius/profile/aspect/mouth width/depth vary. Three visible seed variants are recorded. This is authored geometry variation, not independent biological functionality. |
| G4 animation | PASS for demonstrated motion | Body and flagellar positions change in23 early-motion captures; mouth open/near-closed and continued attachment demonstrated. All31 captured poses pass geometry checks and rebuild exactly headlessly. Attachment error0 across800 population poses. No collapse/separation or unintended clipping observed in inspected witnesses. Root interpenetration is intentional overlap mounting. Global self/inter-component intersections are not certified. |
| G5 persistence | PASS | All100 definitions saved/reloaded and reproduced. Live keyboard handlers saved an edited organism, generated other seeds, then reloaded the exact definition and posed-geometry digest. Invalid definition loads/saves preserve the last valid state. |
| G6 integration safety | PASS | 117 final workspace tests; strict all-target Clippy and formatting pass offline. All pre-slice simulation, viewer, models, existing experiments and evidence retain their baseline hashes. Root Cargo files add two internal packages only; no third-party version changes. README/handoff receive bounded additions. |

The population dataset uses seeds0–99 and explicit near-closed/open, rest and fully deformed poses. Cases with0/1/2 flagella and absent/present ridges are included. Full trial rows,100 definitions, six full geometry witnesses, all live captures and failures are retained externally. [Validation metadata](docs/morph_001/VALIDATION.json) provides machine-readable results and preservation checks.

## Run and use

```sh
cargo run --release -p moebius-morph-lab --offline
```

1–7 selects a parameter; Left/Right edits it; Shift changes faster. R selects the next seed. F cycles0/1/2 flagella. G toggles ridges. Space pauses/runs authored motion. M toggles mouth cycling. S saves and L reloads `runs/morph_001/organism.json`. An existing definition there is loaded at startup. Right-drag or Alt+arrows orbit; the wheel zooms. `--definition PATH` selects another local definition/save path; `--seed N` selects the initial generator seed when that path does not exist.

The save/reload walkthrough drives the same keyboard handlers used by the interactive application, not alternate geometry code. The mouse-orbit/wheel handlers have a direct Bevy input test; the recorded orbit uses Alt+arrow. The figures are internal rendered captures of generated meshes, not prebuilt models or mockups. UI text clipping/missing glyphs found in the earlier witnesses were fixed in walkthrough-v3. Final clock review found that a one-turn phase wrap was discontinuous for fractional-frequency channels. Source revision2 uses a ten-turn common period; a regression checks body, mouth and flagellar continuity across the wrap. This editor-clock fix leaves the headless engine and all captured poses unchanged. Both source versions are preserved.

## Exact validation commands and outcomes

Paths below must be new directories. The recorded run names were `population-v1` and `walkthrough-v3`; earlier lab attempts remain preserved.

```sh
cargo test --workspace --offline
# PASS:117 tests,0 failures
cargo clippy --workspace --all-targets --offline -- -D warnings
# PASS
cargo fmt --all -- --check
# PASS
cargo run --release -p moebius-morph --example validate --offline -- NEW_POPULATION_DIR
# PASS:100/100 definitions,800 poses,100 save/reload repetitions
cargo run --release -p moebius-morph-lab --offline -- --walkthrough NEW_WALKTHROUGH_DIR
# PASS:31 internal PNG captures plus pose/definition metadata and real editor commands
cargo run --release -p moebius-morph --example replay_lab --offline -- NEW_WALKTHROUGH_DIR
# PASS:31/31 exact capture digests; body/flagellar motion and3 visible seeds
python3 python/morph_artifacts.py verify
python3 python/morph_artifacts.py unpack NEW_RESTORE_DIRECTORY
```

Recorded compiler:rustc1.99.1, commit58cf8f9c35b355199727627d07ae34a4e2846d67, LLVM23.1.1, x86_64-unknown-linux-gnu. Exact environment and build/test logs are external. The minimum declared workspace Rust version remains1.97.1. Cross-platform/toolchain bit-identical floating-point trigonometry is **NOT DEMONSTRATED**.

Donor validation boundaries: Unity was not run. Donor B's attempted offline unit-test command failed package resolution for uncached optional `bevy-inspector-egui`; donor tests are **NOT RUN**, not passed. Initial Bevy0.20 API/test/lint errors are preserved with their corrected outcomes. A fresh standalone offline lockfile resolution also failed on a yanked Windows-only clipboard-win version. The reproduction package retains existing pins and prunes workspace membership through offline Cargo using that lock; no third-party versions are updated. The failed fresh-resolution and pre-prune --locked attempts remain preserved. These limitations do not replace validation of our independent implementation.

## Limitations and next priorities

1. Physical locomotion, feeding, metabolism, learning, adaptation and reproduction are outside this engine. Mesh shape or appendage presence grants no capability. The current neural controller's fixed joint ABI is not a compatible arbitrary-anatomy adapter; one must be designed explicitly later.
2. Roots are overlap-mounted closed tubes, not a welded body/flagellar manifold union. Add explicit attachment patch topology and collision checks before claiming seamless unified anatomy. No global self-intersection detector or exhaustive proof over every accepted parameter combination is included.
3. CPU parametric deformation is implemented; general automatic skeletal rigging, volume/arc-length preservation, skin sliding and arbitrary branching anatomy are not. Add a rig/deformation backend behind the headless API when justified.
4. The mouth has a recessed cavity but retains a tiny slot at its closed limit. Airtight closure, bite collision and ingestion are not demonstrated. The fixed angular UV seam is adequate for vertex-coloured preview; production texture unwrapping remains future work.
5. This is a minimal keyboard-driven laboratory. Component picking, drag/add/remove/transform gizmos, arbitrary anatomy families, anatomical sculpt quality and complete creator UX remain future slices. Current flags can be added/removed and parameterized, not freely dragged.
6. No Zeitgeist files were changed. Future consumers can use the headless crate without exposing this editor. No pretrained model or runtime network access is required.

## Source, storage and final Git state

Base HEAD and current HEAD:`8e351e212d1f979a32b1bb3a862719a06ea1b70d` on `main`. MORPH changes are **uncommitted; no push**. Pre-existing M6/cathedral/Worm_A work remains uncommitted and is preserved. Newly authorized scope is the two crates, root Cargo membership/lock additions, README/handoff additions, this slice's audits/results, fixture and evidence metadata. Full final status is in `docs/morph_001/GIT_STATE.json`.

[Source identity](docs/morph_001/SOURCE_IDENTITY.json), [artifact classification](docs/morph_001/ARTIFACT_CLASSIFICATION.json) and [storage/restore metadata](docs/morph_001/ARTIFACT_STORAGE.json) distinguish Git-worthy source/reports/metadata from raw geometry, donor snapshots, logs and rendered evidence. Raw evidence lives in immutable Mermogon MORPH-001/v1 archives plus a source-only v2 revision. Archive/member hashes, restoration and a rebuild of the restored standalone headless source are verified. Previous scientific and Worm_A archives are unchanged. The standalone archive manifest contains only the two new crates for reproduction; the original workspace manifest is also preserved explicitly.
