//! Independent ship witness. Camera/meshes have no numerical mutation rights.
use bevy::{
    asset::RenderAssetUsages,
    input::mouse::{AccumulatedMouseMotion, MouseWheel},
    mesh::Indices,
    prelude::*,
    render::{
        render_resource::PrimitiveTopology,
        view::screenshot::{Screenshot, save_to_disk},
    },
    ui_widgets::{Activate, Button},
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use clap::Parser;
use moebius_morph::{
    emission::{RigidPose, SCALE},
    filament::{
        Assembly, CHUNK_EDGES, StructuralIndex, construction_fixture, construction_pose,
        filament_chunk,
    },
    runtime::*,
    witness::*,
    *,
};
use moebius_morph_lab::navigation::{Navigation, NavigationMode};
use std::{
    collections::{BTreeMap, BTreeSet},
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
    construction: bool,
    #[arg(long)]
    record: Option<PathBuf>,
    #[arg(long, default_value_t = 2)]
    mesh_budget: usize,
    #[arg(long, default_value_t = 0.003)]
    sensitivity: f32,
}
#[derive(Resource)]
struct Authority(Arc<Mutex<Option<Worker>>>);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Selected {
    Organism(u64),
    Filament(u64),
}
type ChunkMesh = (Entity, Handle<Mesh>, Position, usize);
#[derive(Resource)]
struct Display {
    nav: Navigation,
    sphere: WitnessSphere,
    witness_follow: bool,
    snapshot: Arc<WorldSnapshot>,
    selected: Selected,
    follow: bool,
    last_target: Position,
    bodies: BTreeMap<u64, Vec<(Entity, Handle<Mesh>)>>,
    chunks: BTreeMap<(u64, usize), ChunkMesh>,
    index: StructuralIndex,
    index_nodes: usize,
    material: Handle<StandardMaterial>,
    frame: u64,
    out: Option<PathBuf>,
    ticks: u64,
    rows: Vec<serde_json::Value>,
    telemetry: Arc<Mutex<Option<serde_json::Value>>>,
    show_filaments: bool,
    show_parcels: bool,
    diagnostics: bool,
    mesh_budget: usize,
    generation: u64,
    captures: usize,
    camera_cpu_ns: u128,
    started: Instant,
    last_frame: Instant,
}
#[derive(Component)]
struct SceneCamera;
#[derive(Component)]
struct Status;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Command {
    PlayPause,
    Step,
    Reset,
    Rate(u32),
    Follow,
    Free,
    Filaments,
    Parcels,
    Diagnostics,
    Mode,
    Focus,
    Radius(i64),
    WitnessFollow,
    Budget(i32),
}
#[derive(Component)]
struct Control(Command);
fn main() {
    let args = Args::parse();
    assert!(
        (1..=16).contains(&args.mesh_budget)
            && args.sensitivity.is_finite()
            && (0.0001..=0.02).contains(&args.sensitivity)
    );
    if let Some(out) = &args.record {
        std::fs::create_dir(out).expect("new capture directory");
    }
    let world = if args.construction {
        construction_fixture(20261123, args.ticks.min(16384) as usize, 512, "curved").unwrap()
    } else {
        fixture(args.population, 20261123).unwrap()
    };
    let initial = world.snapshot();
    let snapshot = Arc::new(initial.clone());
    let worker =
        Worker::interactive(world, args.ticks, Duration::from_nanos(TICK_NANOS), false).unwrap();
    let authority = Arc::new(Mutex::new(Some(worker)));
    let telemetry = Arc::new(Mutex::new(None));
    let mut nav = Navigation::new();
    nav.sensitivity = args.sensitivity;
    let first = snapshot.instances.values().next().unwrap();
    nav.focus(first.position, 2.2);
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "MOEBIUS / THE FIRST FILAMENT / AUTHORED CONSTRUCTION".into(),
            resolution: (1420, 900).into(),
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
        nav,
        sphere: WitnessSphere {
            center: first.position,
            radius: 16 * SCALE,
        },
        witness_follow: true,
        selected: Selected::Organism(first.id),
        follow: args.construction,
        last_target: first.position,
        snapshot,
        bodies: BTreeMap::new(),
        chunks: BTreeMap::new(),
        index: StructuralIndex::default(),
        index_nodes: 0,
        material: default(),
        frame: 0,
        out: args.record.clone(),
        ticks: args.ticks,
        rows: vec![],
        telemetry: telemetry.clone(),
        show_filaments: true,
        show_parcels: true,
        diagnostics: false,
        mesh_budget: args.mesh_budget,
        generation: 0,
        captures: 0,
        started: Instant::now(),
        last_frame: Instant::now(),
        camera_cpu_ns: 0,
    })
    .add_observer(buttons)
    .add_systems(Startup, setup)
    .add_systems(Update, (inject_input, controls, draw).chain());
    app.run();
    let worker = authority.lock().unwrap().take().unwrap();
    let playback = worker.playback().unwrap();
    let _ = worker.control(Playback::Stop);
    let final_state = worker.finish().expect("authoritative run");
    if let Some(out) = &args.record {
        initial.save(&out.join("world-initial.json")).unwrap();
        final_state.save(&out.join("world-final.json")).unwrap();
        let mut reference = ShipRuntime::restore(initial).unwrap();
        reference.advance(final_state.tick).unwrap();
        assert_eq!(reference.snapshot(), final_state);
        let mut report = telemetry
            .lock()
            .unwrap()
            .take()
            .expect("recorded witness completion; early-close evidence retained");
        report["status"] = serde_json::json!("PASS");
        report["ticks"] = serde_json::json!(final_state.tick);
        report["final_digest"] = serde_json::json!(final_state.digest().unwrap());
        report["headless_equal"] = serde_json::json!(true);
        report["reset_digests"] = serde_json::json!(playback.reset_digests);
        report["playback_controls"] = serde_json::json!(
            playback
                .controls
                .iter()
                .map(
                    |(n, t, c)| serde_json::json!({"ordinal":n,"tick":t,"command":format!("{c:?}")})
                )
                .collect::<Vec<_>>()
        );
        std::fs::write(
            out.join("render-report.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
        println!(
            "MORPH005_WITNESS_COMPLETE {}",
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
        Transform::from_rotation(display.nav.rotation),
        SceneCamera,
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
        perceptual_roughness: 0.48,
        ..default()
    });
    commands.spawn((Node{position_type:PositionType::Absolute,left:Val::Px(15.),top:Val::Px(15.),width:Val::Px(245.),padding:UiRect::all(Val::Px(10.)),flex_direction:FlexDirection::Column,row_gap:Val::Px(2.),..default()},BackgroundColor(Color::srgba(0.015,0.025,0.035,0.93)))).with_children(|p|{
        p.spawn((Text::new("THE FIRST FILAMENT\nAuthored trajectory / no locomotion"),TextFont{font_size:FontSize::Px(16.),..default()}));
        for(label,command)in[("Play / Pause [Space]",Command::PlayPause),("Single step [.]",Command::Step),("Reset experiment [R]",Command::Reset),("Speed 1x [1]",Command::Rate(1)),("Speed 4x [4]",Command::Rate(4)),("Speed 16x [6]",Command::Rate(16)),("Follow selected [C]",Command::Follow),("Free camera",Command::Free),("Flight / Orbit [Tab]",Command::Mode),("Focus selected [F]",Command::Focus),("Show / hide filaments",Command::Filaments),("Show / hide free material",Command::Parcels),("Diagnostics [B]",Command::Diagnostics),("Witness radius -",Command::Radius(-SCALE)),("Witness radius +",Command::Radius(SCALE)),("Witness follows camera",Command::WitnessFollow),("Mesh budget +",Command::Budget(1)),("Mesh budget -",Command::Budget(-1))]{
            p.spawn((Button,Control(command),Node{padding:UiRect::all(Val::Px(3.)),..default()},BackgroundColor(Color::srgb(0.08,0.14,0.17)))).with_children(|b|{b.spawn((Text::new(label),TextFont{font_size:FontSize::Px(12.),..default()}));});
        }
        p.spawn((Text::new("WASD / QE: flight\nRight mouse: look\nZ / X: local roll\nShift: fast  Ctrl: precise\nWheel: speed / orbit dolly\nOrbit: left drag / middle pan\nClick: select  Home: reset view"),TextFont{font_size:FontSize::Px(12.),..default()}));
    });
    commands.spawn((
        Text::new("Preparing authoritative snapshot"),
        TextFont {
            font_size: FontSize::Px(15.),
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(20.),
            top: Val::Px(20.),
            ..default()
        },
        Status,
    ));
}
fn target(d: &Display) -> Position {
    match d.selected {
        Selected::Organism(id) => d
            .snapshot
            .instances
            .get(&id)
            .map_or(d.last_target, |i| i.position),
        Selected::Filament(id) => d
            .snapshot
            .assembly
            .as_ref()
            .and_then(|a| a.filaments.get(&id))
            .and_then(|f| f.nodes.last())
            .map_or(d.last_target, |n| n.position),
    }
}
fn command(c: Command, d: &mut Display, w: &Worker) {
    match c {
        Command::PlayPause => {
            let status = w.playback().unwrap();
            let _ = w.control(if status.playing {
                Playback::Pause
            } else {
                Playback::Play
            });
        }
        Command::Step => {
            let _ = w.control(Playback::Step);
        }
        Command::Reset => {
            let _ = w.control(Playback::Reset);
        }
        Command::Rate(n) => {
            let _ = w.control(Playback::Rate(n));
        }
        Command::Follow => {
            d.follow = !d.follow;
            d.last_target = target(d);
        }
        Command::Free => {
            d.follow = false;
            if d.nav.mode == NavigationMode::Orbit {
                d.nav.switch();
            }
        }
        Command::Filaments => d.show_filaments = !d.show_filaments,
        Command::Parcels => d.show_parcels = !d.show_parcels,
        Command::Diagnostics => d.diagnostics = !d.diagnostics,
        Command::Mode => {
            d.nav.switch();
            if d.nav.mode == NavigationMode::Orbit {
                let pivot = target(d);
                d.nav.orbit_target(pivot);
            }
        }
        Command::Focus => {
            let mut p = target(d);
            let mut size = 2.2;
            if let Selected::Filament(id) = d.selected
                && let Some(f) = d
                    .snapshot
                    .assembly
                    .as_ref()
                    .and_then(|a| a.filaments.get(&id))
            {
                let mut low = [i64::MAX; 3];
                let mut high = [i64::MIN; 3];
                for node in &f.nodes {
                    let point = node.position.absolute().unwrap();
                    for k in 0..3 {
                        low[k] = low[k].min(point[k]);
                        high[k] = high[k].max(point[k]);
                    }
                }
                p = Position::from_absolute(std::array::from_fn(|k| {
                    ((i128::from(low[k]) + i128::from(high[k])) / 2) as i64
                }))
                .unwrap();
                size = (0..3)
                    .map(|k| (high[k] - low[k]) as f64 / SCALE as f64)
                    .fold(0.1, f64::max);
            }
            d.nav.focus(p, size);
            if d.witness_follow {
                d.sphere.radius = ((d.nav.distance + size + 2.) * SCALE as f64).ceil() as i64;
            }
            d.last_target = target(d);
        }
        Command::Radius(delta) => {
            d.sphere.radius = (d.sphere.radius + delta).clamp(SCALE, 10000 * SCALE)
        }
        Command::WitnessFollow => d.witness_follow = !d.witness_follow,
        Command::Budget(delta) => {
            d.mesh_budget = (d.mesh_budget as i32 + delta).clamp(1, 16) as usize
        }
    }
}
fn buttons(
    event: On<Activate>,
    buttons: Query<&Control>,
    keys: Res<ButtonInput<KeyCode>>,
    mut d: ResMut<Display>,
    a: Res<Authority>,
) {
    if keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter) {
        return;
    }
    if let Ok(control) = buttons.get(event.entity) {
        let lock = a.0.lock().unwrap();
        command(control.0, &mut d, lock.as_ref().unwrap());
    }
}
fn inject_input(
    d: Res<Display>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut motion: ResMut<AccumulatedMouseMotion>,
    mut wheel: MessageWriter<MouseWheel>,
    windows: Query<Entity, With<Window>>,
) {
    if d.out.is_none() {
        return;
    }
    let frame = d.frame + 1;
    keys.reset_all();
    mouse.reset_all();
    motion.delta = Vec2::ZERO;
    if (82..=88).contains(&frame) {
        keys.press(KeyCode::KeyW);
        keys.press(KeyCode::ShiftLeft);
    }
    if (92..=96).contains(&frame) {
        mouse.press(MouseButton::Right);
        motion.delta = Vec2::new(2., -1.);
    }
    if [100, 125].contains(&frame) {
        keys.press(KeyCode::Tab);
    }
    if (103..=106).contains(&frame) {
        mouse.press(MouseButton::Left);
        motion.delta = Vec2::new(1., 0.5);
    }
    if (109..=111).contains(&frame) {
        mouse.press(MouseButton::Middle);
        motion.delta = Vec2::new(1., -1.);
    }
    if [115, 190].contains(&frame) {
        keys.press(KeyCode::KeyF);
    }
    if (130..=133).contains(&frame) {
        keys.press(KeyCode::KeyZ);
        keys.press(KeyCode::KeyW);
        keys.press(KeyCode::ControlLeft);
    }
    if frame == 140 {
        keys.press(KeyCode::Home);
    }
    if let Some(entity) = windows.iter().next()
        && [112, 135].contains(&frame)
    {
        wheel.write(MouseWheel {
            unit: bevy::input::mouse::MouseScrollUnit::Line,
            x: 0.,
            y: 1.,
            window: entity,
            phase: bevy::input::touch::TouchPhase::Moved,
        });
    }
}
#[allow(clippy::too_many_arguments)]
fn controls(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    mut wheel: MessageReader<MouseWheel>,
    mut window: Query<(&Window, &mut CursorOptions), With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<SceneCamera>>,
    mut d: ResMut<Display>,
    a: Res<Authority>,
) {
    let started = Instant::now();
    let lock = a.0.lock().unwrap();
    let w = lock.as_ref().unwrap();
    d.snapshot = w.latest().unwrap();
    let status = w.playback().unwrap();
    if status.generation != d.generation {
        d.generation = status.generation;
        d.index_nodes = usize::MAX;
        d.last_target = target(&d);
    }
    for (k, c) in [
        (KeyCode::Space, Command::PlayPause),
        (KeyCode::Period, Command::Step),
        (KeyCode::KeyR, Command::Reset),
        (KeyCode::Digit1, Command::Rate(1)),
        (KeyCode::Digit4, Command::Rate(4)),
        (KeyCode::Digit6, Command::Rate(16)),
        (KeyCode::KeyC, Command::Follow),
        (KeyCode::Tab, Command::Mode),
        (KeyCode::KeyF, Command::Focus),
        (KeyCode::KeyB, Command::Diagnostics),
        (KeyCode::Minus, Command::Radius(-SCALE)),
        (KeyCode::Equal, Command::Radius(SCALE)),
    ] {
        if keys.just_pressed(k) {
            command(c, &mut d, w);
        }
    }
    if keys.just_pressed(KeyCode::Home) {
        d.nav = Navigation::new();
        d.follow = false;
    }
    let at = target(&d);
    if d.follow {
        let delta = at.relative(d.last_target).unwrap();
        d.nav.position = d.nav.position.offset(delta).unwrap();
        d.nav.pivot = d.nav.pivot.offset(delta).unwrap();
    }
    d.last_target = at;
    if let Ok((window, mut cursor)) = window.single_mut() {
        let cursor_position = if d.out.is_some() {
            Some(Vec2::new(800., 400.))
        } else {
            window.cursor_position()
        };
        let looking = window.focused && mouse.pressed(MouseButton::Right);
        cursor.grab_mode = if looking {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        };
        cursor.visible = !looking;
        if looking
            || (d.nav.mode == NavigationMode::Orbit
                && mouse.pressed(MouseButton::Left)
                && cursor_position.is_some_and(|p| p.x > 275.))
        {
            d.nav.look(motion.delta);
        }
        if d.nav.mode == NavigationMode::Orbit && mouse.pressed(MouseButton::Middle) {
            d.nav.pan(motion.delta);
        }
        if mouse.just_pressed(MouseButton::Left)
            && cursor_position.is_some_and(|p| p.x > 275.)
            && let (Ok((camera, transform)), Some(pos)) = (camera.single(), cursor_position)
            && let Ok(ray) = camera.viewport_to_world(transform, pos)
        {
            let direction = ray.direction.as_vec3();
            let mut nearest = f32::INFINITY;
            let mut selected = None;
            let mut pick = |point: Position, radius: f32, candidate: Selected| {
                let p = Vec3::from_array(
                    point
                        .relative(d.nav.position)
                        .unwrap()
                        .map(|v| v as f32 / SCALE as f32),
                );
                let t = p.dot(direction);
                let dist = (p - direction * t).length();
                if t > 0. && dist < radius && t < nearest {
                    nearest = t;
                    selected = Some(candidate);
                }
            };
            for i in d.snapshot.instances.values() {
                pick(i.position, 1.2, Selected::Organism(i.id));
            }
            if let Some(assembly) = &d.snapshot.assembly {
                for f in assembly.filaments.values() {
                    for n in &f.nodes {
                        pick(n.position, 0.12, Selected::Filament(f.id));
                    }
                }
            }
            if let Some(selected) = selected {
                d.selected = selected;
                d.last_target = target(&d);
            }
        }
    }
    let local = Vec3::new(
        (keys.pressed(KeyCode::KeyD) as i32 - keys.pressed(KeyCode::KeyA) as i32) as f32,
        (keys.pressed(KeyCode::KeyE) as i32 - keys.pressed(KeyCode::KeyQ) as i32) as f32,
        (keys.pressed(KeyCode::KeyS) as i32 - keys.pressed(KeyCode::KeyW) as i32) as f32,
    );
    let multiplier = if keys.pressed(KeyCode::ShiftLeft) {
        4.
    } else if keys.pressed(KeyCode::ControlLeft) {
        0.1
    } else {
        1.
    };
    let roll = (keys.pressed(KeyCode::KeyZ) as i32 - keys.pressed(KeyCode::KeyX) as i32) as f32;
    d.nav.fly(local, time.delta_secs_f64(), multiplier, roll);
    for event in wheel.read() {
        d.nav.wheel(event.y);
    }
    if d.witness_follow {
        d.sphere.center = d.nav.position;
    }
    d.camera_cpu_ns = started.elapsed().as_nanos();
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
fn draw(
    mut commands: Commands,
    mut d: ResMut<Display>,
    a: Res<Authority>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut transforms: Query<&mut Transform, Without<SceneCamera>>,
    mut cameras: Query<&mut Transform, With<SceneCamera>>,
    mut text: Query<&mut Text, With<Status>>,
    mut gizmos: Gizmos,
    mut exit: MessageWriter<AppExit>,
    ui_controls: Query<(Entity, &Control)>,
) {
    let started = Instant::now();
    d.frame += 1;
    let frame = d.frame;
    let frame_interval = d.last_frame.elapsed().as_nanos();
    d.last_frame = Instant::now();
    let lock = a.0.lock().unwrap();
    let w = lock.as_ref().unwrap();
    let playback = w.playback().unwrap();
    if d.out.is_some() {
        let action = match frame {
            30 | 40 => Some(Command::PlayPause),
            35 => Some(Command::Step),
            60 => Some(Command::Rate(4)),
            80 => Some(Command::Rate(1)),
            145 | 147 => Some(Command::Filaments),
            128 | 129 => Some(Command::Parcels),
            210 => Some(Command::Reset),
            212 if !playback.playing => Some(Command::PlayPause),
            220 => Some(Command::Rate(16)),
            _ => None,
        };
        if let Some(action) = action {
            let (entity, _) = ui_controls
                .iter()
                .find(|(_, c)| c.0 == action)
                .expect("production UI control exists");
            commands.trigger(Activate { entity });
        }
        match frame {
            150 => {
                d.follow = false;
                d.witness_follow = false;
                d.sphere.center = Position::from_absolute([1000 * SCALE, 0, 0]).unwrap();
            }
            190 => {
                d.witness_follow = true;
                command(Command::Focus, &mut d, w);
            }
            _ => {}
        }
    }
    if playback.error.is_some() {
        eprintln!("WORLD_STOPPED {:?}", playback.error);
        exit.write(AppExit::error());
    }
    let before = d.snapshot.digest().unwrap();
    let view = d.sphere.inspect(&d.snapshot).unwrap();
    let visible = view.organisms.iter().copied().collect::<BTreeSet<_>>();
    let gone = d
        .bodies
        .keys()
        .filter(|id| !visible.contains(id))
        .copied()
        .collect::<Vec<_>>();
    for id in gone {
        if let Some(parts) = d.bodies.remove(&id) {
            for (e, h) in parts {
                commands.entity(e).despawn();
                meshes.remove(h.id());
            }
        }
    }
    for id in &view.organisms {
        let instance = &d.snapshot.instances[id];
        let spec = &d.snapshot.config.definitions[&instance.definition];
        let pose = if d.snapshot.config.construction.is_some() {
            construction_pose(d.snapshot.tick)
        } else {
            Pose::rest(&spec.body)
        };
        let g = CompiledAnatomy::new(spec.clone())
            .unwrap()
            .pose(pose)
            .unwrap();
        let at = Vec3::from_array(
            instance
                .position
                .relative(d.nav.position)
                .unwrap()
                .map(|v| v as f32 / SCALE as f32),
        );
        let rigid = d
            .snapshot
            .config
            .construction
            .as_ref()
            .map_or(RigidPose::identity(), |r| r.rigid(d.snapshot.tick));
        let yaw = (rigid.yaw_sin as f64).atan2(rigid.yaw_cos as f64) as f32;
        let transform = Transform::from_translation(at).with_rotation(Quat::from_rotation_y(yaw));
        if !d.bodies.contains_key(id) {
            let mut parts = vec![];
            for s in &g.geometry.surfaces {
                let h = meshes.add(mesh(s));
                let e = commands
                    .spawn((
                        Mesh3d(h.clone()),
                        MeshMaterial3d(d.material.clone()),
                        transform,
                    ))
                    .id();
                parts.push((e, h));
            }
            d.bodies.insert(*id, parts);
        } else {
            for ((e, h), s) in d.bodies[id].iter().zip(&g.geometry.surfaces) {
                if let Some(mut m) = meshes.get_mut(h) {
                    *m = mesh(s);
                }
                if let Ok(mut t) = transforms.get_mut(*e) {
                    *t = transform;
                }
            }
        }
        if d.diagnostics {
            for output in g.outputs {
                let p = moebius_morph::emission::place_output(&output, rigid).unwrap();
                let at = Position::from_absolute(p.origin)
                    .unwrap()
                    .relative(d.nav.position)
                    .unwrap()
                    .map(|v| v as f32 / SCALE as f32);
                gizmos.sphere(
                    Isometry3d::from_translation(Vec3::from_array(at)),
                    0.08,
                    Color::srgb(1., 0.55, 0.1),
                );
            }
        }
    }
    if d.show_parcels {
        for (org, parcel) in &view.parcels {
            let p = d.snapshot.instances[org]
                .book
                .parcels
                .iter()
                .find(|p| p.id == *parcel)
                .unwrap();
            let at = Position::from_absolute(p.position)
                .unwrap()
                .relative(d.nav.position)
                .unwrap()
                .map(|v| v as f32 / SCALE as f32);
            gizmos.sphere(
                Isometry3d::from_translation(Vec3::from_array(at)),
                0.05,
                Color::srgb(1., 0.65, 0.08),
            );
        }
    }
    let nodes = d.snapshot.assembly.as_ref().map_or(0, Assembly::nodes);
    if nodes != d.index_nodes {
        d.index = d
            .snapshot
            .assembly
            .as_ref()
            .map_or_else(StructuralIndex::default, |a| {
                StructuralIndex::build(a).unwrap()
            });
        d.index_nodes = nodes;
    }
    let desired = if d.show_filaments {
        d.index
            .visible(&d.sphere)
            .unwrap()
            .into_iter()
            .collect::<BTreeSet<_>>()
    } else {
        BTreeSet::new()
    };
    let old = d
        .chunks
        .keys()
        .filter(|k| !desired.contains(k))
        .copied()
        .collect::<Vec<_>>();
    for k in old {
        if let Some((e, h, _, _)) = d.chunks.remove(&k) {
            commands.entity(e).despawn();
            meshes.remove(h.id());
        }
    }
    let mut generated = 0;
    for key in desired {
        let f = &d.snapshot.assembly.as_ref().unwrap().filaments[&key.0];
        let count = (f.nodes.len() - key.1 * CHUNK_EDGES).min(CHUNK_EDGES + 1);
        let changed = d.chunks.get(&key).is_none_or(|v| v.3 != count);
        if changed && generated < d.mesh_budget {
            let (origin, surface) = filament_chunk(f, key.1).unwrap();
            let h = meshes.add(mesh(&surface));
            let at = origin
                .relative(d.nav.position)
                .unwrap()
                .map(|v| v as f32 / SCALE as f32);
            let e = commands
                .spawn((
                    Mesh3d(h.clone()),
                    MeshMaterial3d(d.material.clone()),
                    Transform::from_translation(Vec3::from_array(at)),
                ))
                .id();
            if let Some((old, old_h, _, _)) = d.chunks.insert(key, (e, h, origin, count)) {
                commands.entity(old).despawn();
                meshes.remove(old_h.id());
            }
            generated += 1;
        }
    }
    for (e, _, origin, _) in d.chunks.values() {
        if let Ok(mut t) = transforms.get_mut(*e) {
            t.translation = Vec3::from_array(
                origin
                    .relative(d.nav.position)
                    .unwrap()
                    .map(|v| v as f32 / SCALE as f32),
            );
        }
    }
    if let Ok(mut camera) = cameras.single_mut() {
        *camera = Transform::from_rotation(d.nav.rotation);
    }
    if d.diagnostics {
        let at = d
            .sphere
            .center
            .relative(d.nav.position)
            .unwrap()
            .map(|v| v as f32 / SCALE as f32);
        gizmos.sphere(
            Isometry3d::from_translation(Vec3::from_array(at)),
            d.sphere.radius as f32 / SCALE as f32,
            Color::srgb(0.1, 0.24, 0.35),
        );
    }
    let book = &d.snapshot.instances.values().next().unwrap().book;
    let bound = book.bound_quantity().unwrap();
    let free = book.in_world().unwrap();
    let filaments = d
        .snapshot
        .assembly
        .as_ref()
        .map_or(0, |a| a.filaments.len());
    let position = target(&d).absolute().unwrap();
    for mut t in &mut text {
        t.0 = format!(
            "MORPH-005 | tick {} / {} | {}x {}\nSelected {:?} | {:?} | follow {}\nCoordinates [um] {:?}\nReserve {} | free {} | bound {} | balance {}\nFilaments {} | nodes {} | visible chunks {}\nWitness {:.0}m {} | mesh budget {} / frame\nFlight speed {:.2}m/s | sensitivity {:.4}\nAmber: free secretion   Blue: bound filament\nAuthored motion. Static binding, no mechanical support.",
            d.snapshot.tick,
            d.ticks,
            playback.rate,
            if playback.playing { "PLAY" } else { "PAUSE" },
            d.selected,
            d.nav.mode,
            d.follow,
            position,
            book.reservoir,
            free,
            bound,
            book.residual().unwrap(),
            filaments,
            nodes,
            d.chunks.len(),
            d.sphere.radius as f64 / SCALE as f64,
            if d.witness_follow {
                "camera-follow"
            } else {
                "independent"
            },
            d.mesh_budget,
            d.nav.speed,
            d.nav.sensitivity
        );
    }
    assert_eq!(before, d.snapshot.digest().unwrap());
    if d.out.is_some() {
        if d.rows.len() >= 4096 {
            eprintln!("Recording frame budget exceeded; no world history discarded");
            exit.write(AppExit::error());
            return;
        }
        let tick = d.snapshot.tick;
        let chunks = d.chunks.len();
        let cpu = started.elapsed().as_nanos();
        let camera = d.nav.position;
        let generation = d.generation;
        let camera_cpu_ns = d.camera_cpu_ns;
        let camera_mode = format!("{:?}", d.nav.mode);
        let camera_rotation = d.nav.rotation.to_array();
        let camera_speed = d.nav.speed;
        d.rows.push(serde_json::json!({"frame":frame,"tick":tick,"generation":generation,"digest":before,"nodes":nodes,"bound":bound,"free":free,"filaments":filaments,"chunks":chunks,"meshes_generated":generated,"cpu_ns":cpu,"frame_interval_ns":frame_interval,"camera_cpu_ns":camera_cpu_ns,"mesh_assets":meshes.len(),"camera":camera,"camera_mode":camera_mode,"camera_rotation":camera_rotation,"camera_speed":camera_speed}));
        if [25, 55, 120, 175, 200, 280].contains(&frame) {
            d.captures += 1;
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(
                    d.out
                        .as_ref()
                        .unwrap()
                        .join(format!("frame_{frame:03}.png")),
                ));
        }
        if frame >= 320 && d.snapshot.tick >= d.ticks {
            *d.telemetry.lock().unwrap() = Some(
                serde_json::json!({"frames":d.frame,"captures":d.captures,"rows":d.rows,"elapsed_ns":d.started.elapsed().as_nanos(),"mesh_budget":d.mesh_budget,"claims":"authored motion and static material binding; no locomotion or mechanics","gpu_timestamps":"unavailable; CPU/end-to-end frame timings only","UI_test":"programmatic Activate on actual production widgets; no physical mouse playtest claimed"}),
            );
            exit.write(AppExit::Success);
        }
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
