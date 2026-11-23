# MORPH-001 existing-system audit

Audited before implementation on 2026-11-23. HEAD:8e351e212d1f979a32b1bb3a862719a06ea1b70d. The worktree already contains substantial uncommitted M6, cathedral and Worm_A work; the baseline hash inventory is preserved externally with this slice. Do not reset or overwrite it.

## Executing paths and boundaries

- Cargo workspace: core, CLI, viewer. Viewer Cargo.toml pins Bevy=0.20.0; Cargo.lock resolves bevy_mesh0.20.1. MIT workspace; Rust edition2024/minimum1.97.1. HEAD history:8e351e2 M5,fd837e7 M4,ecfd933 neural nursery,32a6b03 articulated Farm M1. Read README.md, docs/BLUEPRINT.md (historical M0), docs/FARM_M1.md, docs/neural/ARCHITECTURE.md and HANDOFF.md/current Worm_A reports separately.
- Core producer: World::tick / tick_observed / tick_controlled -> atomic audited transition. neural::schema::Anatomy::reference calls chain(12,2):12 spinal joints with bend/twist plus2 jaw axes. This is implemented, not assumed. NeuralState/controller/body own authoritative motors, fixed-point kinematics, material/energy state. Arbitrary visible appendages have no automatic simulation role.
- Viewer producer/consumer: main.rs registers live::Hall workers and show::hall_pump, embody, hall_bodies, rig_joints. live::pump owns World and emits Picture; render systems interpolate PoseSpan. Renderer does not advance physics. Existing diagnostic jaw_exercise is explicit/labelled, not a production gait.
- Legacy procedural producer: body.rs private Sculpt::tube/conduit/mesh, creature_mesh -> Gallery -> show::embody. It sweeps indexed rings and adds colours/normals/UVs, then assigns skin weights against hard-coded12-spine bind positions. It is useful local implementation experience, not a reusable versioned anatomical compiler or general editor. It lacks independent manifold/property validation.
- Worm_A path: scripts/rig_worm_a.py + worm_morph.py + worm_teeth.py derive a GLB/.blend/rig.json from the preserved sculpt. worm_a::WormAssets::load -> show::embody -> SkinnedMesh and shared MorphWeights; worm_a::sculpt_mouth maps actual jaw angles to shape keys. This depends on the supplied asset and offline Python/Blender authoring. It does not procedurally generate arbitrary anatomy. Current metallic crown work and its accepted lip motion remain untouched by MORPH-001.
- UI: rules.rs Draft + show::rules_keys/rules_text and FlyCamera are coupled to simulation modes. The lab can reuse the keyboard/Text interaction approach but should not couple a new anatomy definition to SimConfig or the recurrent motor ABI. Existing snapshot playback is a separate historical path.

## Reuse decision

Build a separate headless moebius-morph crate and an isolated moebius-morph-lab Bevy binary. Share the workspace/toolchain and Bevy version, not simulation types. The compiler exports indexed surfaces, attachment frames and explicit preview deformation controls. Existing fixed joint count, law/versioned snapshots, appearance-vs-capability distinction, material conservation and controller behaviour stay intact. No compulsory Blender stage and no Zeitgeist integration.

Relevant existing checks: core invariant/farm/neural/M4/M5/M6/terrain tests; viewer actual-sculpt morphology, articulated mouth, capture/pause and camera tests. Baseline acceptance checks will run unchanged alongside new tests; scientific reports are historical evidence and must not be rewritten as new results.
