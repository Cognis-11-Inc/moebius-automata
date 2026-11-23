# MORPH-001 donor code and licensing assessment

Read-only source/history audit on 2026-11-23, before implementation. Selective local checkouts and exact audit logs are archived externally. No donor code or assets are copied into the engine; implementation is independent.

## A: SPORE Creature Creator

Fork pin:[e7044c2c503ac5b0cb0bc7fa0c49f7f29d68310a](https://github.com/el-zach/SPORE-Creature-Creator/tree/e7044c2c503ac5b0cb0bc7fa0c49f7f29d68310a). Upstream inspected pin:[1e7745c50b701c5444eab3863892824c89c8b432](https://github.com/daniellochner/creature/tree/1e7745c50b701c5444eab3863892824c89c8b432). Common ancestor:713f940c49c8c0b250f693914d32ef1cc8f2115d.

Exact paths below are under Assets/Creature Creator/Scripts/Runtime/ at the fork pin:

| Mechanism | Source and executing relationship | Assessment |
|---|---|---|
| Ring body, cap triangles, UVs, skinning and local radius blend shapes | Core/Controllers/CreatureController.cs::Setup, called from creator startup/rebuild; UpdateBoneConfiguration and UpdateMeshCollider maintain the edited body | Implemented Unity mesh/SkinnedMeshRenderer path. Not implicit-surface/marching-cubes evidence. |
| Serialization | Core/Data/Creature/CreatureData.cs stores bones, attached parts, pattern/colours; CreatureController::Save/Load use JsonUtility and Demo/Utilities/SaveUtility.cs; Core/Managers/CreatureCreator::SaveCreature/LoadCreature invoke them | Concrete persistence and restoration, Unity-specific scene/prefab dependencies. |
| Component placement | Core/Data/AttachedBodyPart.cs boneIndex + SerializableTransform; CreatureController::SetupBodyPart, AttachBodyPart and drag callbacks raycast/parent/pair-mirror objects | Reusable architectural concept: stable attachment coordinates + local transforms. Prefabs/plugins not imported. |
| Mouth component | Core/Objects/Body Parts/Features/Mouth.cs::Mouth | Stores Diet metadata on a Feature; it does not generate an opening/interior or operate jaws. The new procedural mouth is independently authored. |
| Editable limbs | Core/Controllers/BodyPartControllers/LimbController.cs::Start binds drag/scroll; LateUpdate orients bone chain; UpdateMeshCollider bakes the skin | Actual rig infrastructure, dependent on authored skinned meshes; not a demonstrated general automatic rig or locomotion system. |
| Editor | Core/Managers/CreatureCreator.cs::Build/Test/Paint switches menus and interaction; Core/Controllers/CameraOrbit.cs and Interact/ callbacks supply direct editing | Source-wired controls. Test() is a UI mode change, not evidence of autonomous animation or learning. |

The fork README explicitly says its testing section was not implemented; its MIT link points to LICENSE.md although the actual pinned license file is LICENSE. Source history, rather than that stale link, was used below. Unity was not run here; source call-site evidence is distinct from live validation. The tree contains IK/example/plugin infrastructure, but mere presence does not prove execution in the audited creature path. No claim of general automatic appendage anatomy, robust watertight biological attachment, paid locomotion or learning follows from this donor.

### Licensing provenance gate

The fork HEAD LICENSE, common-ancestor LICENSE and original MIT-introduction LICENSE all have Git blob c2b519fd46c0d022f539e5132f699800c37b2027. MIT introduction:[63fbf1189a1aff648e0f31b2f34884cb9045fcc5](https://github.com/daniellochner/creature/commit/63fbf1189a1aff648e0f31b2f34884cb9045fcc5), dated2020-09-26, copyright Daniel Lochner. Upstream later deleted LICENSE at[56e4e20dc4e7efe22873855cb6ef0966ec5e2637](https://github.com/daniellochner/creature/commit/56e4e20dc4e7efe22873855cb6ef0966ec5e2637) and introduced GPL-3.0 LICENSE.md at[25aac626c5c6789b51c828ba63994fa88601404e](https://github.com/daniellochner/creature/commit/25aac626c5c6789b51c828ba63994fa88601404e), both2022-12-21. The historical MIT file explains the fork/upstream discrepancy; a current badge alone would not have established that history. This is provenance evidence, not a blanket clearance for every bundled prefab/plugin/asset. Those were not individually cleared. Gate decision: architectural reference only; zero copied donor-A source/assets.

## B: Bevy Procedural Meshes

Pin:[38032008e7790d99b462bb355c7a1fc20f77746c](https://github.com/bevy-procedural/meshes/tree/38032008e7790d99b462bb355c7a1fc20f77746c). Cargo.toml:bevy_procedural_meshes0.19.0, Bevy^0.19.0; MIT OR Apache-2.0 with LICENSE-MIT/LICENSE-APACHE. It is not type-compatible with our Bevy0.20 Mesh without an adapter/port or duplicate engine dependency.

| Mechanism | Exact source | Maturity / limits |
|---|---|---|
| Typed vertex/index/attribute construction | src/mesh/mod.rs::PMesh::{build,build_ex,extend}; src/mesh/vertices.rs::PVertices; src/mesh/indices.rs | Useful buffers, index types, transforms; no creature spec or attachment model. |
| Normals/UV | src/mesh/normals.rs::{flat_normals,smooth_normals}; shapes.rs::generate_uv_for_fan | Smooth normals carry a bug warning; no automatic robustness guarantee. |
| Bevy conversion/update | src/mesh/backend_bevy/mod.rs::{to_bevy,bevy_set} | Validates bounds/attribute lengths; clears attributes, falls back to duplicated flat normals. Does not preserve skin/morph controls. |
| 3D capability | vertices.rs::extrude, shapes.rs::rect/fan/polygon/triangle; examples/3d.rs::update | Extrudes a vertex loop to side walls. The advertised3D example actually draws planar Lyon geometry and leaves whole-shape extrusion TODO; not a general organic surface generator. |
| Validation | src/mesh/geometry/triangle.rs::Triangle::{area,is_degenerate,normal}; examples/2d.rs,3d.rs | Geometry helpers, not an anatomical test suite. No #[test] functions were found in src. Donor cargo test --no-default-features --lib --offline could not resolve uncached optional bevy-inspector-egui; tests NOT RUN, not a passing build. |

Decision: no new dependency and no selective code copy. A small independent indexed-surface implementation keeps headless data free of Bevy and supports exact topology, attachment and deformation validation. Donor concepts consulted: structured buffers, explicit normals/UVs, transform separation and mesh update. No transfer is claimed as validated until the new implementation's own gates pass.
