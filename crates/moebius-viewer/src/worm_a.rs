//! User-authored Worm_A skin. Skeleton follows authoritative reference joints.
use super::Gallery;
use bevy::{gltf::GltfAssetLabel, mesh::skinning::SkinnedMeshInverseBindposes, prelude::*};
const ASSET: &str = "creatures/worm_a/Worm_A_rigged.glb";
pub struct WormAssets {
    pub teeth: Handle<Mesh>,
    pub teeth_material: Handle<StandardMaterial>,
    pub mesh: Handle<Mesh>,
    pub flesh: Handle<Mesh>,
    pub flesh_material: Handle<StandardMaterial>,
    pub material: Handle<StandardMaterial>,
    pub bindposes: Handle<SkinnedMeshInverseBindposes>,
    pub positions: [Vec3; 14],
}
pub fn bind_positions() -> [Vec3; 14] {
    let json: serde_json::Value =
        serde_json::from_str(include_str!("../../../assets/creatures/worm_a/rig.json"))
            .expect("checked Worm_A rig");
    std::array::from_fn(|i| {
        Vec3::from_array(std::array::from_fn(|a| {
            json["bind_positions"][i][a].as_f64().unwrap() as f32
        }))
    })
}
impl WormAssets {
    pub fn load(server: &AssetServer) -> Self {
        Self {
            teeth: server.load(
                GltfAssetLabel::Primitive {
                    mesh: 0,
                    primitive: 2,
                }
                .from_asset(ASSET),
            ),
            teeth_material: server.load(format!("{ASSET}#Material2/std")),
            mesh: server.load(
                GltfAssetLabel::Primitive {
                    mesh: 0,
                    primitive: 0,
                }
                .from_asset(ASSET),
            ),
            material: server.load(format!("{ASSET}#Material0/std")),
            flesh: server.load(
                GltfAssetLabel::Primitive {
                    mesh: 0,
                    primitive: 1,
                }
                .from_asset(ASSET),
            ),
            flesh_material: server.load(format!("{ASSET}#Material1/std")),
            bindposes: server.load(GltfAssetLabel::InverseBindMatrices(0).from_asset(ASSET)),
            positions: bind_positions(),
        }
    }
}
pub fn ready(
    gallery: Option<Res<Gallery>>,
    meshes: Res<Assets<Mesh>>,
    materials: Res<Assets<StandardMaterial>>,
    poses: Res<Assets<SkinnedMeshInverseBindposes>>,
    mut reported: Local<bool>,
) {
    if *reported {
        return;
    }
    if let Some(worm) = gallery.as_ref().and_then(|g| g.worm.as_ref())
        && let Some(mesh) = meshes.get(&worm.mesh)
        && meshes.get(&worm.flesh).is_some()
        && meshes.get(&worm.teeth).is_some()
        && materials.get(&worm.teeth_material).is_some()
        && materials.get(&worm.flesh_material).is_some()
        && materials.get(&worm.material).is_some()
        && poses.get(&worm.bindposes).is_some()
    {
        println!(
            "WORM_A_READY vertices {} joints14 mouth_joint9 texture+skin loaded",
            mesh.count_vertices()
        );
        *reported = true;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mouth_anchor_tracks_physical_head_under_bending_and_twist() {
        use moebius_core::neural::{body::Body, schema::Anatomy};
        let anatomy = Anatomy::reference();
        let bind = bind_positions();
        for seed in [0i32, 1, 2, 3] {
            let mut body = Body::new(&anatomy);
            for (i, a) in body.angles.iter_mut().enumerate() {
                *a = if seed == 0 {
                    0
                } else {
                    ((i as i32 * 97 + seed * 121) % 500) - 250
                };
            }
            let (points, center) = body.local_geometry(&anatomy);
            let mut position = Vec3::ZERO;
            let mut rotation = Quat::IDENTITY;
            for i in 0..=9 {
                let delta = bind[i] - if i == 0 { Vec3::ZERO } else { bind[i - 1] };
                position += rotation * delta;
                rotation *= Quat::from_rotation_z(body.angles[i * 2] as f32 / 1024.)
                    * Quat::from_rotation_y(body.angles[i * 2 + 1] as f32 / 1024.);
            }
            let actual = position * super::super::BODY_SCALE
                - Vec3::from_array(center.map(|v| v as f32)) / 1024.
                    * super::super::layout::SPACING;
            let expected = Vec3::from_array(points[9].map(|v| v as f32)) / 1024.
                * super::super::layout::SPACING;
            assert!(
                actual.distance(expected) < 0.004,
                "mouth error {:?} {:?}",
                actual,
                expected
            );
        }
    }
    #[test]
    fn feeding_uses_the_articulated_mouth_not_body_center_or_remote_food() {
        use moebius_core::{SimConfig, Site, World};
        use std::collections::BTreeMap;
        for aligned in [true, false] {
            let mut config = SimConfig::nursery_farm_256();
            config.width = 16;
            config.height = 16;
            config.depth = 16;
            config.initial_population = 1;
            config.initial_energy = 128;
            config.initial_reservoir = 0;
            config.inflow_per_tick = 0;
            config.inject_per_tick = 0;
            config.feeding.as_mut().unwrap().spacing = 8;
            config.feeding.as_mut().unwrap().initial_stock = 0;
            let mut snap = World::genesis(&config, 20261122).unwrap().snapshot();
            snap.cells[0].site = Site { x: 8, y: 8, z: 8 };
            let state = snap.brains[0].neural.as_mut().unwrap();
            state.body.angles[0] = 96;
            state.body.angles[2] = -64;
            state.body.centroid = state.body.local_geometry(&state.genome.anatomy).1;
            let mouth = state.body.head_site(
                &state.genome.anatomy,
                snap.cells[0].site,
                [0; 3],
                moebius_core::Dims {
                    width: 16,
                    height: 16,
                    depth: 16,
                },
            );
            assert_ne!(mouth, snap.cells[0].site);
            snap.brains[0].sync_neural_pose();
            let food = if aligned {
                mouth
            } else {
                Site {
                    x: (mouth.x + 4) % 16,
                    ..mouth
                }
            };
            snap.nutrient.fill(0);
            snap.nutrient[(food.x + 16 * (food.y + 16 * food.z)) as usize] = 2;
            snap.material_initial = 2;
            let mut world = World::restore(&config, &snap).unwrap();
            let (receipt, rows) = world
                .tick_controlled(&BTreeMap::from([(1, vec![0; 30])]))
                .unwrap();
            assert_eq!(receipt.nutrient_uptaken, if aligned { 1 } else { 0 });
            assert_eq!(rows[0].intake, receipt.nutrient_uptaken);
            assert_eq!(world.accounts().unwrap().material_residual(), 0);
            assert_eq!(world.accounts().unwrap().energy_residual(), 0);
        }
    }
    #[test]
    fn actual_sculpt_morph_moves_vertically_under_native_jaw_signals() {
        let bytes = include_bytes!("../../../assets/creatures/worm_a/Worm_A_rigged.glb");
        let len = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
        let doc: serde_json::Value = serde_json::from_slice(&bytes[20..20 + len]).unwrap();
        assert_eq!(doc["meshes"][0]["primitives"].as_array().unwrap().len(), 3);
        let targets = &doc["meshes"][0]["primitives"][0]["targets"];
        let mut peak_z = 0f32;
        for (target_i, target) in targets.as_array().unwrap().iter().enumerate() {
            let a = &doc["accessors"][target["POSITION"].as_u64().unwrap() as usize];
            let view = &doc["bufferViews"][a["bufferView"].as_u64().unwrap() as usize];
            let offset = 28 + len + view["byteOffset"].as_u64().unwrap() as usize;
            let count = a["count"].as_u64().unwrap() as usize;
            let mut moving = 0;
            for i in 0..count {
                let f = |axis: usize| {
                    let o = offset + i * 12 + axis * 4;
                    f32::from_le_bytes(bytes[o..o + 4].try_into().unwrap())
                };
                assert_eq!(f(0), 0., "closure must preserve lateral width");
                if f(2).abs() > 0.001 {
                    moving += 1;
                }
                peak_z = peak_z.max(f(2).abs());
            }
            assert!(moving > 100, "deform real sculpt vertices {target_i}");
        }
        assert!(peak_z > 0.05);
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../experiments/motor_land_demo.json");
        let mut world = super::super::motor_demo::load(&path, "selected").unwrap();
        let mut range = [1f32, 0f32];
        for _ in 0..256 {
            world.tick().unwrap();
            let angle = world.brains()[&1].neural.as_ref().unwrap().body.angles[25] as f32;
            let weight = close_weight(angle);
            range[0] = range[0].min(weight);
            range[1] = range[1].max(weight);
        }
        assert!((range[1] - range[0]) * peak_z > 0.03);
    }
    #[test]
    fn entering_mouth_view_does_not_pause_simulation() {
        use super::super::{live, motor_demo, show};
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../experiments/motor_land_demo.json");
        let mut world = motor_demo::load(&path, "selected").unwrap();
        for _ in 0..20 {
            world.tick().unwrap();
        }
        let mut hall = live::open_initial(world).unwrap();
        for _ in 0..100 {
            hall.drain();
            if hall.latest.is_some() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(hall.committed_tick() >= 20);
        let view = show::view_for_hall(&hall);
        let mut app = App::new();
        app.insert_resource(hall)
            .insert_resource(view)
            .insert_resource(MouthView {
                enabled: true,
                ..default()
            })
            .init_resource::<ButtonInput<KeyCode>>()
            .add_systems(Update, mouth_view);
        app.world_mut().spawn((
            Camera3d::default(),
            Transform::default(),
            Projection::default(),
        ));
        app.update();
        assert!(
            !app.world().resource::<live::Hall>().is_paused(),
            "mouth inspection must not freeze jaw movement"
        );
        assert!(app.world().resource::<super::super::ViewState>().playing);
    }
    #[test]
    fn legacy_jaw_rotation_has_consistent_open_close_sign() {
        let bind = bind_positions();
        let mouth = bind[9];
        let left = mouth + Vec3::new(-0.16, 0., 0.);
        let right = mouth + Vec3::new(0.16, 0., 0.);
        let aperture = |angle: f32| {
            let a = bind[12] + Quat::from_rotation_z(-angle) * (left - bind[12]);
            let b = bind[13] + Quat::from_rotation_z(angle) * (right - bind[13]);
            (b.x - a.x).abs()
        };
        assert!(aperture(-0.25) < aperture(0.));
        assert!(aperture(0.75) > aperture(0.));
    }
}

/// A moving, head-following mouth close-up. Only the user pauses native ticking.
#[derive(Resource, Default)]
pub struct MouthView {
    pub enabled: bool,
    pub view_initialized: bool,
    pub saved_view: Option<(Vec3, f32, f32, f32)>,
}
pub fn mouth_view(
    request: Option<ResMut<MouthView>>,
    keys: Res<ButtonInput<KeyCode>>,
    capture: Option<Res<super::FrameCapture>>,
    hall: Option<Res<super::live::Hall>>,
    mut view: ResMut<super::ViewState>,
    mut cameras: Query<(&mut Transform, &mut Projection), With<Camera3d>>,
) {
    let Some(mut request) = request else {
        return;
    };
    if keys.just_pressed(KeyCode::KeyV) && capture.is_none() {
        request.enabled = !request.enabled;
        if request.enabled {
            request.view_initialized = false;
        } else if let Some((eye, yaw, pitch, fov)) = request.saved_view.take() {
            view.eye = Some(eye);
            view.yaw = yaw;
            view.pitch = pitch;
            view.fov = fov;
            view.note.clear();
        }
    }
    if !request.enabled {
        return;
    }
    if request.saved_view.is_none() {
        request.saved_view = Some((
            view.eye.unwrap_or(Vec3::new(0., 0.4, 9.)),
            view.yaw,
            view.pitch,
            view.fov,
        ));
    }
    let Some(hall) = hall else {
        return;
    };
    let Some(picture) = &hall.latest else {
        return;
    };
    let Some(cell) = picture.cells.first() else {
        return;
    };
    let Some(state) = cell.rig.as_ref().and_then(|r| r.neural.as_ref()) else {
        return;
    };
    let lead = hall.motion_blend();
    let span = hall.spans.get(&cell.id);
    let angles = |i: usize| span.map_or(state.body.angles[i] as f32, |s| s.angle(i, lead)) / 1024.;
    let bind = bind_positions();
    let mut point = Vec3::ZERO;
    let mut rot = Quat::IDENTITY;
    for i in 0..=9 {
        point += rot * (bind[i] - if i == 0 { Vec3::ZERO } else { bind[i - 1] });
        rot *= Quat::from_rotation_z(angles(i * 2)) * Quat::from_rotation_y(angles(i * 2 + 1));
    }
    let orient = span.map_or(state.body.orientation.map(|v| v as f32), |s| {
        s.orientation(lead)
    });
    let orient = orient.map(|a| a / 1024.);
    let rotation = Quat::from_rotation_z(orient[2])
        * Quat::from_rotation_y(orient[1])
        * Quat::from_rotation_x(orient[0]);
    let centroid = span.map_or(state.body.centroid.map(|v| v as f32), |s| s.centroid(lead));
    let base = span.map_or_else(
        || super::layout::site_to_view(cell.site, hall.dims),
        |s| s.translation(lead),
    );
    point += rot * Vec3::new(0., 0., 0.28);
    let target = Vec3::from_array(base)
        + rotation
            * (point * super::BODY_SCALE
                - Vec3::from_array(centroid) / 1024. * super::layout::SPACING);
    let mouth_rotation = rotation * rot;
    let eye = target + mouth_rotation * Vec3::new(0.35, 1.15, 0.3);
    for (mut camera, mut projection) in &mut cameras {
        *camera = Transform::from_translation(eye).looking_at(target, mouth_rotation * Vec3::Z);
        if let Projection::Perspective(p) = projection.as_mut() {
            p.fov = 0.72;
        }
    }
    view.eye = Some(eye);
    if !request.view_initialized {
        request.view_initialized = true;
        view.note =
            "Worm_A moving mouth view. Space pauses/resumes; V returns to body view.".into();
    }
}

/// Records real rendered/native jaw motion; never supplies motor commands.
#[derive(Resource)]
pub struct MouthRecording {
    pub directory: std::path::PathBuf,
    pub next_tick: u64,
    pub frames: u64,
}
pub fn record_mouth(
    request: Option<ResMut<MouthRecording>>,
    hall: Option<Res<super::live::Hall>>,
    gallery: Option<Res<Gallery>>,
    meshes: Res<Assets<Mesh>>,
    mut commands: Commands,
) {
    use bevy::render::view::screenshot::{Screenshot, save_to_disk};
    let Some(mut request) = request else {
        return;
    };
    if request.frames >= 16 {
        return;
    }
    let Some(hall) = hall else {
        return;
    };
    if hall.committed_tick() < request.next_tick {
        return;
    }
    let Some(worm) = gallery.as_ref().and_then(|g| g.worm.as_ref()) else {
        return;
    };
    if meshes.get(&worm.mesh).is_none()
        || meshes.get(&worm.flesh).is_none()
        || meshes.get(&worm.teeth).is_none()
    {
        return;
    }
    let Some(picture) = &hall.latest else {
        return;
    };
    let Some(cell) = picture.cells.first() else {
        return;
    };
    let Some(neural) = cell.rig.as_ref().and_then(|r| r.neural.as_ref()) else {
        return;
    };
    let lead = hall.motion_blend();
    let pose: [f32; 2] = std::array::from_fn(|i| {
        hall.spans
            .get(&cell.id)
            .map_or(neural.body.angles[24 + i] as f32, |s| s.angle(24 + i, lead))
    });
    let file = request
        .directory
        .join(format!("frame_{:06}", request.frames));
    let metadata = serde_json::json!({"tick":picture.tick,"state_hash":picture.state_hash,"config_hash":hall.config_hash,"genome":neural.genome.id(),"jaw_angles":&neural.body.angles[24..26],"render_jaw_angles":pose,"sculpt_close_weights":pose.map(close_weight),"jaw_drive":&neural.actuator_drive()[24..26],"energy":cell.energy,"intake":neural.last_intake,"externally_imposed_motor_commands":hall.jaw_exercise});
    std::fs::write(
        file.with_extension("json"),
        serde_json::to_vec_pretty(&metadata).unwrap(),
    )
    .expect("mouth recording metadata");
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(file.with_extension("png")));
    request.frames += 1;
    request.next_tick = picture.tick + 16;
    if request.frames == 16 {
        println!(
            "WORM_MOUTH_RECORD_COMPLETE frames16 last_tick{} external_motor_commands={}",
            picture.tick, hall.jaw_exercise
        );
    }
}

/// Calibrated shape keys supplement skinning; the original sculpt is the open basis.
/// Only existing jaw angles supply these weights, never a clock or idle loop.
pub fn close_weight(angle: f32) -> f32 {
    ((768. - angle) / 1024.).clamp(0., 1.)
}
pub fn sculpt_mouth(
    hall: Res<super::live::Hall>,
    mut mouths: Query<(&super::Creature, &mut bevy::mesh::morph::MorphWeights)>,
) {
    let Some(picture) = &hall.latest else {
        return;
    };
    let lead = hall.motion_blend();
    for (creature, mut morph) in &mut mouths {
        let Ok(i) = picture.cells.binary_search_by_key(&creature.id, |c| c.id) else {
            continue;
        };
        let Some(n) = picture.cells[i]
            .rig
            .as_ref()
            .and_then(|r| r.neural.as_ref())
        else {
            continue;
        };
        for (jaw, weight) in morph.weights_mut().iter_mut().enumerate() {
            let angle = hall
                .spans
                .get(&creature.id)
                .map_or(n.body.angles[24 + jaw] as f32, |s| s.angle(24 + jaw, lead));
            *weight = close_weight(angle);
        }
    }
}
