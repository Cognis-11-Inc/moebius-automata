//! An independent witness of the canonical ship runtime; all mutations stay on its worker.
use bevy::{
    asset::RenderAssetUsages,
    mesh::Indices,
    prelude::*,
    render::{
        render_resource::PrimitiveTopology,
        view::screenshot::{Screenshot, save_to_disk},
    },
};
use clap::Parser;
use moebius_morph::{CompiledAnatomy, Pose, Surface, emission::SCALE, runtime::*, witness::*};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
#[derive(Parser)]
struct Args {
    #[arg(long, default_value_t = 3600)]
    ticks: u64,
    #[arg(long, default_value_t = 16)]
    population: usize,
    #[arg(long)]
    record: Option<PathBuf>,
}
#[derive(Resource)]
struct Authority(Arc<Mutex<Option<Worker>>>);
#[derive(Resource)]
struct Display {
    sphere: WitnessSphere,
    snapshot: Arc<WorldSnapshot>,
    cache: PresentationCache,
    entities: BTreeMap<u64, Vec<(Entity, Handle<Mesh>)>>,
    material: Handle<StandardMaterial>,
    frame: u64,
    out: Option<PathBuf>,
    ticks: u64,
    rows: Vec<serde_json::Value>,
    cpu_ns: u128,
    started: Instant,
    last_frame: Instant,
    captured: usize,
    telemetry: Arc<Mutex<Option<serde_json::Value>>>,
}
#[derive(Component)]
struct Status;
fn main() {
    let args = Args::parse();
    if let Some(out) = &args.record {
        std::fs::create_dir(out).expect("new capture directory required");
    }
    let world = fixture(args.population, 20261123).expect("supported fixture");
    let snapshot = Arc::new(world.snapshot());
    let worker =
        Worker::spawn(world, args.ticks, 1, Duration::from_millis(16)).expect("independent worker");
    let authority = Arc::new(Mutex::new(Some(worker)));
    let telemetry = Arc::new(Mutex::new(None));
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "MOEBIUS / THE UNOBSERVED WORLD / MORPH-004".into(),
            resolution: (1280, 800).into(),
            ..default()
        }),
        ..default()
    }))
    .insert_resource(ClearColor(Color::srgb(0.008, 0.013, 0.025)))
    .insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 180.,
        ..default()
    })
    .insert_resource(Authority(authority.clone()))
    .insert_resource(Display {
        sphere: WitnessSphere {
            center: Position::from_absolute([0, 0, 0]).unwrap(),
            radius: 10 * SCALE,
        },
        snapshot,
        cache: PresentationCache::default(),
        entities: BTreeMap::new(),
        material: default(),
        frame: 0,
        out: args.record.clone(),
        ticks: args.ticks,
        rows: vec![],
        cpu_ns: 0,
        started: Instant::now(),
        last_frame: Instant::now(),
        captured: 0,
        telemetry: telemetry.clone(),
    })
    .add_systems(Startup, setup)
    .add_systems(Update, observe);
    app.run();
    // Closing the window does not cancel the recorded world run. Finish its exact tick target.
    let worker = authority.lock().unwrap().take().unwrap();
    let final_state = worker.finish().expect("authoritative run completes");
    if let Some(out) = &args.record {
        final_state
            .save(&out.join("world-final.json"))
            .expect("persist world");
        let mut reference = fixture(args.population, 20261123).unwrap();
        reference.advance(args.ticks).unwrap();
        assert_eq!(reference.snapshot(), final_state);
        let mut report = telemetry.lock().unwrap().take().expect(
            "recorded witness reached target; partial captures preserved if window closed early",
        );
        report["status"] = serde_json::json!("PASS");
        report["ticks"] = serde_json::json!(final_state.tick);
        report["final_digest"] = serde_json::json!(final_state.digest().unwrap());
        report["headless_final_state_exactly_equal"] = serde_json::json!(true);
        std::fs::write(
            out.join("render-report.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
        println!(
            "MORPH004_WITNESS_COMPLETE {}",
            final_state.digest().unwrap()
        );
    }
}
fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut display: ResMut<Display>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(7., 5., 9.).looking_at(Vec3::new(1., 0., 0.), Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 4000.,
            ..default()
        },
        Transform::from_xyz(5., 8., 4.).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    display.material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.5,
        ..default()
    });
    commands.spawn((
        Text::new("Independent ship authority"),
        TextFont {
            font_size: FontSize::Px(18.),
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.),
            top: Val::Px(20.),
            ..default()
        },
        Status,
    ));
}
fn mesh(s: &Surface) -> Mesh {
    let mut m = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    m.insert_attribute(Mesh::ATTRIBUTE_POSITION, s.positions.clone());
    m.insert_attribute(Mesh::ATTRIBUTE_NORMAL, s.normals.clone());
    m.insert_attribute(Mesh::ATTRIBUTE_COLOR, s.colors.clone());
    m.insert_indices(Indices::U32(s.indices.clone()));
    m
}
#[allow(clippy::too_many_arguments)]
fn observe(
    mut commands: Commands,
    authority: Res<Authority>,
    mut display: ResMut<Display>,
    keys: Res<ButtonInput<KeyCode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut transforms: Query<&mut Transform, Without<Camera>>,
    mut text: Query<&mut Text, With<Status>>,
    mut gizmos: Gizmos,
    mut exit: MessageWriter<AppExit>,
) {
    let started = Instant::now();
    display.frame += 1;
    let frame = display.frame;
    let frame_dt = display.last_frame.elapsed();
    display.last_frame = Instant::now();
    display.snapshot = authority
        .0
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .latest()
        .expect("snapshot feed");
    if display.out.is_some() {
        display.sphere.center = Position::from_absolute([
            if (frame / 40) % 2 == 1 {
                1000 * SCALE
            } else {
                0
            },
            0,
            0,
        ])
        .unwrap();
        display.sphere.radius = if (frame / 25).is_multiple_of(2) {
            10 * SCALE
        } else {
            5 * SCALE
        };
    } else {
        let step = 200_000;
        let delta = [
            (keys.pressed(KeyCode::KeyD) as i64 - keys.pressed(KeyCode::KeyA) as i64) * step,
            (keys.pressed(KeyCode::KeyE) as i64 - keys.pressed(KeyCode::KeyQ) as i64) * step,
            (keys.pressed(KeyCode::KeyS) as i64 - keys.pressed(KeyCode::KeyW) as i64) * step,
        ];
        display.sphere.center = display.sphere.center.offset(delta).unwrap();
        if keys.just_pressed(KeyCode::Equal) {
            display.sphere.radius = (display.sphere.radius + SCALE).min(100 * SCALE);
        }
        if keys.just_pressed(KeyCode::Minus) {
            display.sphere.radius = (display.sphere.radius - SCALE).max(SCALE);
        }
    }
    let before = display.snapshot.digest().unwrap();
    let view = display.sphere.inspect(&display.snapshot).unwrap();
    let change = display.cache.update(&view);
    for id in &change.exited {
        if let Some(entities) = display.entities.remove(id) {
            for (e, h) in entities {
                commands.entity(e).despawn();
                meshes.remove(h.id());
            }
        }
    }
    for id in &change.entered {
        let i = &display.snapshot.instances[id];
        let s = &display.snapshot.config.definitions[&i.definition];
        let translation = Vec3::from_array(
            i.position
                .relative(display.sphere.center)
                .unwrap()
                .map(|v| v as f32 / SCALE as f32),
        );
        let geometry = CompiledAnatomy::new(s.clone())
            .unwrap()
            .pose(Pose::rest(&s.body))
            .unwrap();
        let mut entities = vec![];
        for surface in geometry.geometry.surfaces {
            let handle = meshes.add(mesh(&surface));
            let e = commands
                .spawn((
                    Mesh3d(handle.clone()),
                    MeshMaterial3d(display.material.clone()),
                    Transform::from_translation(translation),
                ))
                .id();
            entities.push((e, handle));
        }
        display.entities.insert(*id, entities);
    }
    for (id, entities) in &display.entities {
        let at = display.snapshot.instances[id]
            .position
            .relative(display.sphere.center)
            .unwrap()
            .map(|v| v as f32 / SCALE as f32);
        for (e, _) in entities {
            if let Ok(mut t) = transforms.get_mut(*e) {
                t.translation = Vec3::from_array(at);
            }
        }
    }
    for (organism, parcel) in &view.parcels {
        let p = display.snapshot.instances[organism]
            .book
            .parcels
            .iter()
            .find(|p| p.id == *parcel)
            .unwrap();
        let at = Position::from_absolute(p.position)
            .unwrap()
            .relative(display.sphere.center)
            .unwrap()
            .map(|v| v as f32 / SCALE as f32);
        gizmos.sphere(
            Isometry3d::from_translation(Vec3::from_array(at)),
            0.035,
            Color::srgb(0.25, 0.9, 0.65),
        );
    }
    gizmos.sphere(
        Isometry3d::IDENTITY,
        display.sphere.radius as f32 / SCALE as f32,
        Color::srgb(0.1, 0.24, 0.35),
    );
    assert_eq!(before, display.snapshot.digest().unwrap());
    let cpu_ns = started.elapsed().as_nanos();
    display.cpu_ns += cpu_ns;
    if display.out.is_some() {
        let tick = display.snapshot.tick;
        display.rows.push(serde_json::json!({"frame":frame,"tick":tick,"visible":view.organisms,"parcels":view.parcels.len(),"entered":change.entered,"exited":change.exited,"digest":before,"cpu_ns":cpu_ns,"frame_interval_ns":frame_dt.as_nanos()}));
        if [30, 65, 90, 130, 170, 210].contains(&frame) {
            display.captured += 1;
            let path = display
                .out
                .as_ref()
                .unwrap()
                .join(format!("frame_{frame:03}.png"));
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(path));
        }
        if frame >= 240 && display.snapshot.tick == display.ticks {
            *display.telemetry.lock().unwrap() = Some(
                serde_json::json!({"frames":display.frame,"captured":display.captured,"observation_and_mesh_cpu_ns":display.cpu_ns,"elapsed_ns":display.started.elapsed().as_nanos(),"frame_samples":display.rows,"gpu_timestamp_cost":"NOT_MEASURED: frame intervals are CPU/end-to-end, not isolated GPU cost","volumetrics":"NOT_IMPLEMENTED; optional prototype deferred","semantics":"read-only fixed-rest anatomy and material points; no floor, physiology or camera-dependent simulation"}),
            );
            exit.write(AppExit::Success);
        }
    }
    for mut t in &mut text {
        t.0 = format!(
            "MORPH-004 / THE UNOBSERVED WORLD\nCommitted tick {}/{} | visible {} | material {} | radius {:.0}m\nWASD/QE: move witness   -/=: radius\nPrescribed drifting fixtures. No physiology or propulsion.\nWorld worker retains every tick with or without this window.",
            display.snapshot.tick,
            display.ticks,
            display.cache.organisms.len(),
            display.cache.parcels.len(),
            display.sphere.radius as f32 / SCALE as f32
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn closing_or_consuming_the_app_retains_the_authority_until_its_exact_target() {
        let initial = fixture(2, 44).unwrap();
        let mut reference = fixture(2, 44).unwrap();
        reference.advance(20).unwrap();
        let authority = Arc::new(Mutex::new(Some(
            Worker::spawn(initial, 20, 1, Duration::from_millis(1)).unwrap(),
        )));
        let mut app = App::new();
        app.insert_resource(Authority(authority.clone()));
        app.set_runner(|_consumed| AppExit::Success);
        app.run();
        let snapshot = authority.lock().unwrap().take().unwrap().finish().unwrap();
        assert_eq!(snapshot, reference.snapshot());
    }
}
