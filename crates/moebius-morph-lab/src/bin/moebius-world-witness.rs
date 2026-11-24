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
    emission::SCALE,
    filament::{Assembly, CHUNK_EDGES, StructuralIndex, construction_fixture, filament_chunk},
    runtime::*,
    witness::*,
    *,
};
use moebius_morph_lab::{
    ComponentField, CompositionSession, Edit,
    navigation::{Navigation, NavigationMode},
};
/// Far enough that the bound pipe is not deleted at a visible wall.
/// The chamber stays oppressive through darkness, not through a small cage.
const SHIP_WITNESS_RADIUS: i64 = 4_000 * SCALE;
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
    /// Start the functional food/constructor loop directly, using the bare anatomy.
    #[arg(long)]
    first_simulation: bool,
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Gate {
    Title,
    Design,
    Running,
}
#[derive(Resource)]
struct Display {
    nav: Navigation,
    sphere: WitnessSphere,
    witness_follow: bool,
    gate: Gate,
    panel: bool,
    notice: String,
    design: Option<CompositionSession>,
    design_parts: Vec<(Entity, Handle<Mesh>)>,
    gore: Option<(Entity, Handle<Mesh>)>,
    drops: Option<(Entity, Handle<Mesh>)>,
    food_gore: Option<(Entity, Handle<Mesh>)>,
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
    adapt: bool,
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
    let recording = args.record.is_some();
    let mut world = if args.first_simulation {
        ShipRuntime::first_simulation(
            AnatomySpec::bare(20261123),
            20261123,
            args.ticks.min(16384) as usize,
        )
        .unwrap()
    } else if args.construction || !recording {
        construction_fixture(20261123, args.ticks.min(16384) as usize, 512, "head-first").unwrap()
    } else {
        fixture(args.population, 20261123).unwrap()
    };
    if recording && !args.first_simulation {
        world.sow_generic_food().unwrap();
    }
    let initial = world.snapshot();
    let snapshot = Arc::new(initial.clone());
    let worker = if recording || args.first_simulation {
        Some(
            Worker::interactive(world, args.ticks, Duration::from_nanos(TICK_NANOS), false)
                .unwrap(),
        )
    } else {
        None
    };
    let authority = Arc::new(Mutex::new(worker));
    let telemetry = Arc::new(Mutex::new(None));
    let mut nav = Navigation::new();
    nav.sensitivity = args.sensitivity;
    let first = snapshot.instances.values().next().unwrap();
    nav.focus(first.position, 2.2);
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "MOEBIUS / FOOD AND ARACHNITECH".into(),
            resolution: (1420, 900).into(),
            ..default()
        }),
        ..default()
    }))
    .insert_resource(ClearColor(Color::srgb(0.008, 0.013, 0.025)))
    .insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 320.,
        ..default()
    })
    .insert_resource(Authority(authority.clone()))
    .insert_resource(Display {
        nav,
        sphere: WitnessSphere {
            center: first.position,
            radius: SHIP_WITNESS_RADIUS,
        },
        witness_follow: true,
        gate: if recording || args.first_simulation {
            Gate::Running
        } else {
            Gate::Title
        },
        panel: recording || args.first_simulation,
        notice: String::new(),
        design: None,
        design_parts: Vec::new(),
        gore: None,
        drops: None,
        food_gore: None,
        selected: Selected::Organism(first.id),
        follow: args.construction || args.first_simulation,
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
        adapt: true,
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
    moebius_morph_lab::register_ambient(&mut app, args.record.is_none());
    app.run();
    let Some(worker) = authority.lock().unwrap().take() else {
        return;
    };
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
        Projection::Perspective(PerspectiveProjection {
            near: 0.012,
            far: 8_000.,
            ..default()
        }),
        DistanceFog {
            color: Color::srgb(0.008, 0.013, 0.025),
            directional_light_color: Color::NONE,
            directional_light_exponent: 8.,
            falloff: FogFalloff::from_visibility_squared(78.),
        },
        Transform::from_rotation(display.nav.rotation),
        SceneCamera,
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 1600.,
            ..default()
        },
        Transform::from_xyz(5., 8., 4.).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    display.material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.48,
        ..default()
    });
    if display.panel {
        spawn_panel(&mut commands);
    }
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
fn spawn_panel(commands: &mut Commands) {
    commands.spawn((Node{position_type:PositionType::Absolute,left:Val::Px(15.),top:Val::Px(15.),width:Val::Px(245.),padding:UiRect::all(Val::Px(10.)),flex_direction:FlexDirection::Column,row_gap:Val::Px(2.),..default()},BackgroundColor(Color::srgba(0.015,0.025,0.035,0.93)))).with_children(|p|{
        p.spawn((Text::new("THE SHIP\nFood / Arachnitech / secretion"),TextFont{font_size:FontSize::Px(16.),..default()}));
        for(label,command)in[("Play / Pause [Space]",Command::PlayPause),("Single step [.]",Command::Step),("Reset experiment [R]",Command::Reset),("Speed 1x [1]",Command::Rate(1)),("Speed 4x [4]",Command::Rate(4)),("Speed 16x [6]",Command::Rate(16)),("Follow selected [C]",Command::Follow),("Free camera",Command::Free),("Flight / Orbit [Tab]",Command::Mode),("Focus selected [F]",Command::Focus),("Show / hide filaments",Command::Filaments),("Show / hide free material",Command::Parcels),("Diagnostics [B]",Command::Diagnostics),("Witness radius -",Command::Radius(-SCALE)),("Witness radius +",Command::Radius(SCALE)),("Witness follows camera",Command::WitnessFollow),("Mesh budget +",Command::Budget(1)),("Mesh budget -",Command::Budget(-1))]{
            p.spawn((Button,Control(command),Node{padding:UiRect::all(Val::Px(3.)),..default()},BackgroundColor(Color::srgb(0.08,0.14,0.17)))).with_children(|b|{b.spawn((Text::new(label),TextFont{font_size:FontSize::Px(12.),..default()}));});
        }
        p.spawn((Text::new("WASD / QE: flight\nRight mouse: look\nZ / X: local roll\nShift: fast  Ctrl: creep\nWheel: speed, or dolly in close\nTab: orbit  G: adapt to the local shape\nOrbit: left drag / middle pan\nClick: select  Home: reset view\nF5: save the ship"),TextFont{font_size:FontSize::Px(12.),..default()}));
    });
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
        let Some(worker) = lock.as_ref() else {
            return;
        };
        command(control.0, &mut d, worker);
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
    mut mix: ResMut<moebius_morph_lab::AmbientMix>,
) {
    let started = Instant::now();
    moebius_morph_lab::ambient_keys(&keys, &mut mix, KeyCode::KeyM, true);
    if d.gate != Gate::Running {
        gate_keys(&keys, &time, &mut d, &a);
    }
    let lock = a.0.lock().unwrap();
    let Some(w) = lock.as_ref() else {
        preview_camera(
            &time,
            &keys,
            &mouse,
            &motion,
            &mut wheel,
            &mut window,
            &mut d,
        );
        return;
    };
    if keys.just_pressed(KeyCode::F5) && d.out.is_none() {
        d.notice = match d.snapshot.save(&save_path()) {
            Ok(()) => format!("Saved {}", save_path().display()),
            Err(error) => format!("Save refused: {error}"),
        };
    }
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
        d.adapt = true;
    }
    if keys.just_pressed(KeyCode::KeyG) && d.out.is_none() {
        d.adapt = !d.adapt;
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
        if d.nav.mode == NavigationMode::Orbit {
            d.adapt = false;
        }
        d.nav.wheel(event.y);
    }
    if d.adapt && d.follow && d.out.is_none() && d.nav.mode == NavigationMode::Orbit {
        let span = local_span(&d.snapshot, at);
        d.nav.fit(span);
    }
    if d.witness_follow {
        d.sphere.center = d.nav.position;
    }
    d.camera_cpu_ns = started.elapsed().as_nanos();
}
fn save_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../saves/latest.json")
}
fn open_design(d: &mut Display) {
    let spec = AnatomySpec::bare(20261123);
    d.design = Some(
        CompositionSession::new(spec, save_path().with_file_name("arachnitech.json")).unwrap(),
    );
    d.gate = Gate::Design;
    d.notice.clear();
    frame_design(d);
}
fn frame_design(d: &mut Display) {
    let Some(session) = d.design.as_ref() else {
        return;
    };
    let Ok(geometry) = session.geometry() else {
        return;
    };
    frame_preview(&mut d.nav, &geometry);
}
fn frame_preview(nav: &mut Navigation, geometry: &Organism) {
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    for point in geometry.surfaces.iter().flat_map(|s| &s.positions) {
        min = min.min(Vec3::from_array(*point));
        max = max.max(Vec3::from_array(*point));
    }
    let center = (min + max) * 0.5;
    let radius = (max - min).length() * 0.5;
    nav.mode = NavigationMode::Orbit;
    nav.focus(
        Position::from_absolute(
            (center.as_dvec3() * SCALE as f64)
                .round()
                .to_array()
                .map(|v| v as i64),
        )
        .unwrap(),
        f64::from(radius).max(2.4),
    );
}
/// Preview meshes use anatomy coordinates; running meshes are camera-relative.
fn preview_transform(nav: &Navigation) -> Transform {
    let origin = Position::from_absolute([0; 3]).unwrap();
    Transform::from_translation(Vec3::from_array(
        nav.position
            .relative(origin)
            .unwrap()
            .map(|v| v as f32 / SCALE as f32),
    ))
    .with_rotation(nav.rotation)
}
fn begin_ship(spec: AnatomySpec, d: &mut Display, authority: &Authority) {
    let steps = d.ticks.min(16384) as usize;
    let world = match ShipRuntime::first_simulation(spec, 20261123, steps) {
        Ok(world) => world,
        Err(error) => {
            d.notice = format!("This body cannot start the ship: {error}");
            return;
        }
    };
    match Worker::interactive(world, d.ticks, Duration::from_nanos(TICK_NANOS), false) {
        Ok(worker) => {
            *authority.0.lock().unwrap() = Some(worker);
            d.gate = Gate::Running;
            d.design = None;
            d.follow = true;
            d.notice =
                "Blood-Sacs feed this Arachnitech. Its secretion builds the scaffold. Steering is authored."
                    .into();
        }
        Err(error) => d.notice = format!("Worker refused: {error}"),
    }
}
fn load_ship(d: &mut Display, authority: &Authority) {
    let snapshot = match WorldSnapshot::load(&save_path()) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            d.notice = format!("Load refused: {error}");
            return;
        }
    };
    let world = match ShipRuntime::restore(snapshot) {
        Ok(world) => world,
        Err(error) => {
            d.notice = format!("Restore refused: {error}");
            return;
        }
    };
    match Worker::interactive(world, d.ticks, Duration::from_nanos(TICK_NANOS), false) {
        Ok(worker) => {
            *authority.0.lock().unwrap() = Some(worker);
            d.gate = Gate::Running;
            d.design = None;
            d.follow = true;
            d.notice = format!("Loaded {}", save_path().display());
        }
        Err(error) => d.notice = format!("Worker refused: {error}"),
    }
}
fn gate_keys(keys: &ButtonInput<KeyCode>, time: &Time, d: &mut Display, authority: &Authority) {
    if let Some(session) = d.design.as_mut() {
        session.phase = (session.phase + time.delta_secs()) % 1_000.;
    }
    if d.gate == Gate::Title {
        if keys.just_pressed(KeyCode::KeyN) {
            open_design(d);
        }
        if keys.just_pressed(KeyCode::KeyL) || keys.just_pressed(KeyCode::Enter) {
            if save_path().exists() {
                load_ship(d, authority);
            } else if keys.just_pressed(KeyCode::KeyL) {
                d.notice = "No save at saves/latest.json".into();
            } else {
                d.notice = "No save yet. Design the Arachnitech.".into();
                open_design(d);
            }
        }
        return;
    }
    if d.gate != Gate::Design {
        return;
    }
    if keys.just_pressed(KeyCode::Escape) {
        d.gate = Gate::Title;
        d.design = None;
        return;
    }
    if keys.just_pressed(KeyCode::Enter) {
        let Some(spec) = d.design.as_ref().map(|session| session.spec.clone()) else {
            return;
        };
        if !spec
            .components
            .iter()
            .any(|part| matches!(part.shape, Shape::SecretionNozzle { .. }))
        {
            d.notice = "Keep a nozzle. That mouth lays the pipe. Press N.".into();
            return;
        }
        begin_ship(spec, d, authority);
        return;
    }
    if keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight) {
        return;
    }
    let Some(session) = d.design.as_mut() else {
        return;
    };
    let mut edits = vec![];
    let index = ComponentField::ALL
        .iter()
        .position(|field| *field == session.field)
        .unwrap_or(0);
    if keys.just_pressed(KeyCode::ArrowDown) {
        edits.push(Edit::Field(index + 1));
    }
    if keys.just_pressed(KeyCode::ArrowUp) {
        edits.push(Edit::Field((index + 11) % 12));
    }
    if keys.just_pressed(KeyCode::Tab) {
        edits.push(if keys.pressed(KeyCode::ShiftLeft) {
            Edit::SelectPrevious
        } else {
            Edit::SelectNext
        });
    }
    let step = if keys.pressed(KeyCode::ShiftLeft) {
        3.
    } else {
        1.
    };
    for (key, edit) in [
        (KeyCode::ArrowLeft, Edit::Adjust(-step)),
        (KeyCode::ArrowRight, Edit::Adjust(step)),
        (KeyCode::KeyA, Edit::AddFlagellum),
        (KeyCode::KeyP, Edit::AddSpine),
        (KeyCode::KeyN, Edit::AddNozzle),
        (KeyCode::KeyD, Edit::Duplicate),
        (KeyCode::Delete, Edit::Remove),
        (KeyCode::BracketLeft, Edit::BodyLength(-step)),
        (KeyCode::BracketRight, Edit::BodyLength(step)),
        (KeyCode::Minus, Edit::BodyRadius(-step)),
        (KeyCode::Equal, Edit::BodyRadius(step)),
        (KeyCode::Comma, Edit::Mouth(-step)),
        (KeyCode::Period, Edit::Mouth(step)),
        (KeyCode::KeyH, Edit::Showcase),
        (KeyCode::KeyR, Edit::Seed),
        (KeyCode::KeyS, Edit::Save),
    ] {
        if keys.just_pressed(key) {
            edits.push(edit);
        }
    }
    for edit in edits {
        if let Err(error) = session.edit(edit) {
            session.status = format!("REFUSED: {error}");
        }
    }
}
fn preview_camera(
    time: &Time,
    keys: &ButtonInput<KeyCode>,
    mouse: &ButtonInput<MouseButton>,
    motion: &AccumulatedMouseMotion,
    wheel: &mut MessageReader<MouseWheel>,
    window: &mut Query<(&Window, &mut CursorOptions), With<PrimaryWindow>>,
    d: &mut Display,
) {
    if keys.just_pressed(KeyCode::Home) && d.gate == Gate::Design {
        frame_design(d);
    }
    if let Ok((window, mut cursor)) = window.single_mut() {
        let looking = window.focused
            && (mouse.pressed(MouseButton::Right)
                || (d.gate == Gate::Design && mouse.pressed(MouseButton::Left)));
        cursor.grab_mode = if looking {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        };
        cursor.visible = !looking;
        if looking {
            d.nav.look(motion.delta);
        }
        if d.nav.mode == NavigationMode::Orbit && mouse.pressed(MouseButton::Middle) {
            d.nav.pan(motion.delta);
        }
    }
    if d.gate == Gate::Title {
        let local = Vec3::new(
            (keys.pressed(KeyCode::KeyD) as i32 - keys.pressed(KeyCode::KeyA) as i32) as f32,
            (keys.pressed(KeyCode::KeyE) as i32 - keys.pressed(KeyCode::KeyQ) as i32) as f32,
            (keys.pressed(KeyCode::KeyS) as i32 - keys.pressed(KeyCode::KeyW) as i32) as f32,
        );
        d.nav.fly(local, time.delta_secs_f64(), 1., 0.);
    }
    for event in wheel.read() {
        d.nav.wheel(event.y);
    }
}
fn paint(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: &Handle<StandardMaterial>,
    slot: &mut Option<(Entity, Handle<Mesh>)>,
    mesh: Mesh,
) {
    if mesh.count_vertices() == 0 {
        if let Some((entity, handle)) = slot.take() {
            commands.entity(entity).despawn();
            meshes.remove(handle.id());
        }
        return;
    }
    if let Some((_, handle)) = slot {
        if let Some(mut existing) = meshes.get_mut(handle) {
            *existing = mesh;
        }
    } else {
        let handle = meshes.add(mesh);
        let entity = commands
            .spawn((
                Mesh3d(handle.clone()),
                MeshMaterial3d(material.clone()),
                Transform::IDENTITY,
            ))
            .id();
        *slot = Some((entity, handle));
    }
}
fn draw_front(
    commands: &mut Commands,
    d: &mut Display,
    meshes: &mut Assets<Mesh>,
    cameras: &mut Query<&mut Transform, With<SceneCamera>>,
    text: &mut Query<&mut Text, With<Status>>,
) {
    if d.gate != Gate::Design {
        for (entity, handle) in d.design_parts.drain(..) {
            commands.entity(entity).despawn();
            meshes.remove(handle.id());
        }
    } else if let Some(session) = d.design.as_ref() {
        let Ok(organism) = session.geometry() else {
            return;
        };
        if d.design_parts.len() != organism.surfaces.len() {
            for (entity, handle) in d.design_parts.drain(..) {
                commands.entity(entity).despawn();
                meshes.remove(handle.id());
            }
            for surface in &organism.surfaces {
                let handle = meshes.add(mesh(surface));
                let entity = commands
                    .spawn((
                        Mesh3d(handle.clone()),
                        MeshMaterial3d(d.material.clone()),
                        Transform::IDENTITY,
                    ))
                    .id();
                d.design_parts.push((entity, handle));
            }
        } else {
            for ((_, handle), surface) in d.design_parts.iter().zip(&organism.surfaces) {
                if let Some(mut existing) = meshes.get_mut(handle) {
                    *existing = mesh(surface);
                }
            }
        }
    }
    let line = match d.gate {
        Gate::Title => format!(
            "MŒBIUS\nNew game [N] designs your Arachnitech from the basic parts\nSimulation [Enter] continues saves/latest.json, or opens the bench\nLoad [L]\n{}\nParts: trunk, mouth, flagellum, spine, nozzle.\nThe neuron grid is not in this body yet.",
            d.notice
        ),
        Gate::Design => {
            let session = d.design.as_ref().unwrap();
            let part = session.component().map(|c| c.kind()).unwrap_or("trunk");
            format!(
                "DESIGN THE ARACHNITECH\nSelected {part}   {}\nA flagellum   P spine   N nozzle   D duplicate   Delete remove\nTab select   arrows adjust   [ ] length   - = thickness   , . mouth\nH full showcase   R another seed   S save this anatomy\nDrag: orbit   Wheel: zoom   Middle drag: pan   Home: frame body\nEnter begins the ship. Escape returns.\n{}\n{}",
                session.field.label(),
                session.status,
                d.notice
            )
        }
        Gate::Running => String::new(),
    };
    for mut status in text {
        status.0 = line.clone();
    }
    if let Ok(mut camera) = cameras.single_mut() {
        *camera = preview_transform(&d.nav);
    }
}
fn local_span(snapshot: &WorldSnapshot, target: Position) -> f64 {
    let mut span = 1.6_f64;
    let within = |position: Position, reach: f64| -> Option<f64> {
        let rel = position.relative(target).ok()?;
        let metres = rel
            .iter()
            .map(|v| (*v as f64 / SCALE as f64).powi(2))
            .sum::<f64>()
            .sqrt();
        (metres < reach).then_some(metres)
    };
    if let Some(structure) = &snapshot.assembly {
        for filament in structure.filaments.values() {
            for node in &filament.nodes {
                if let Some(metres) = within(node.position, 60.) {
                    span = span.max(metres);
                }
            }
        }
    }
    if let Some(food) = &snapshot.food {
        for site in food.sites.values() {
            if let Some(metres) = within(site.position, 40.) {
                span = span.max(metres);
            }
        }
    }
    span
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
    mix: Res<moebius_morph_lab::AmbientMix>,
) {
    let started = Instant::now();
    d.frame += 1;
    let frame = d.frame;
    let frame_interval = d.last_frame.elapsed().as_nanos();
    d.last_frame = Instant::now();
    if d.gate != Gate::Running {
        draw_front(&mut commands, &mut d, &mut meshes, &mut cameras, &mut text);
        return;
    }
    for (entity, handle) in d.design_parts.drain(..) {
        commands.entity(entity).despawn();
        meshes.remove(handle.id());
    }
    if !d.panel {
        spawn_panel(&mut commands);
        d.panel = true;
    }
    let lock = a.0.lock().unwrap();
    let Some(w) = lock.as_ref() else {
        return;
    };
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
                d.sphere.radius = 16 * SCALE;
            }
            190 => {
                d.witness_follow = true;
                d.sphere.radius = SHIP_WITNESS_RADIUS;
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
        let pose = d.snapshot.anatomical_pose(*id).unwrap();
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
        let rigid = d.snapshot.body_pose(*id).unwrap();
        let yaw = (rigid.yaw_sin as f64).atan2(rigid.yaw_cos as f64) as f32;
        // Food is shown as a small stock-dependent sac; it must not engulf the
        // constructor. This scale is presentation, not a collision/volume law.
        let first = d
            .snapshot
            .config
            .construction
            .as_ref()
            .is_some_and(|r| r.foraging.is_some());
        let scale = if first && instance.definition == BLOOD_SAC {
            0.1 + 0.035 * (instance.book.reservoir as f32).cbrt()
        } else {
            1.
        };
        let transform = Transform::from_translation(at)
            .with_rotation(Quat::from_rotation_y(yaw))
            .with_scale(Vec3::splat(scale));
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
            let rigid = d.snapshot.body_pose(*id).unwrap();
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
    for event in d.snapshot.events.iter().rev().take(256) {
        if let moebius_morph::runtime::Event::Fed {
            at, source, mouth, ..
        } = event
            && d.snapshot.tick.saturating_sub(*at) <= 12
            && let Some(sac) = d.snapshot.instances.get(source)
        {
            let local = |p: Position| {
                Vec3::from_array(
                    p.relative(d.nav.position)
                        .unwrap()
                        .map(|v| v as f32 / SCALE as f32),
                )
            };
            gizmos.line(
                local(sac.position),
                local(*mouth),
                Color::srgb(0.8, 0.35, 0.18),
            );
        }
    }
    let mut drip_sites = Vec::new();
    for id in &view.organisms {
        let instance = &d.snapshot.instances[id];
        if instance.definition != BLOOD_SAC
            || d.snapshot
                .config
                .construction
                .as_ref()
                .is_some_and(|r| r.foraging.is_some())
        {
            continue;
        }
        let Ok(rel) = instance.position.relative(d.nav.position) else {
            continue;
        };
        drip_sites.push((
            Vec3::from_array(rel.map(|v| v as f32 / SCALE as f32)),
            instance.book.reservoir,
        ));
    }
    let material = d.material.clone();
    let frame = d.frame as u32;
    paint(
        &mut commands,
        &mut meshes,
        &material,
        &mut d.gore,
        moebius_morph_lab::gore::hemorrhage_field(&drip_sites, frame),
    );
    if d.show_parcels {
        let mut points = Vec::new();
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
            points.push(Vec3::from_array(at));
        }
        paint(
            &mut commands,
            &mut meshes,
            &material,
            &mut d.drops,
            moebius_morph_lab::gore::droplets(&points),
        );
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
    let mut sites = Vec::new();
    if let Some(food) = &d.snapshot.food {
        for site in food.sites.values() {
            let Ok(rel) = site.position.relative(d.nav.position) else {
                continue;
            };
            if rel.iter().any(|v| v.abs() > 80 * SCALE) {
                continue;
            }
            let radius = 0.18 + site.quantity as f32 * 0.03;
            sites.push((
                Vec3::from_array(rel.map(|v| v as f32 / SCALE as f32)),
                radius,
                site.id,
            ));
        }
    }
    paint(
        &mut commands,
        &mut meshes,
        &material,
        &mut d.food_gore,
        moebius_morph_lab::gore::food_gore(&sites, frame),
    );
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
    let cage = if d.diagnostics {
        format!(
            "Witness {:.0}m {} | mesh budget {} / frame\n",
            d.sphere.radius as f64 / SCALE as f64,
            if d.witness_follow {
                "camera-follow"
            } else {
                "independent"
            },
            d.mesh_budget,
        )
    } else {
        String::new()
    };
    let activity = d
        .snapshot
        .steering
        .map_or("recorded demonstration", |state| {
            if state.seeking_food {
                "seeking food"
            } else {
                "stocked / building"
            }
        });
    for mut t in &mut text {
        t.0 = format!(
            "MŒBIUS | tick {} / {} | {}x {}\nSelected {:?} | {:?} | follow {}\nCoordinates [um] {:?}\nArachnitech: {}\nConstructor stock {} | eaten {} | free {} | bound {} | balance {}\nFilaments {} | nodes {} | visible chunks {}\nBlood-Sacs {} | condensate {} | generic sites {}\n{}{}Flight speed {:.2}m/s | sensitivity {:.4}\nOchre scaffold, amber secretion, red food. Uptake lines mark actual meals.\n{}\nDrone {}   M mute   [ ] volume\nG adapts the orbit. Wheel dollies in. F5 saves.\nNearest-food steering (authored). Scaffold supports food growth.",
            d.snapshot.tick,
            d.ticks,
            playback.rate,
            if playback.playing { "PLAY" } else { "PAUSE" },
            d.selected,
            d.nav.mode,
            d.follow,
            position,
            activity,
            book.reservoir,
            book.external_input,
            free,
            bound,
            book.residual().unwrap(),
            filaments,
            nodes,
            d.chunks.len(),
            d.snapshot
                .instances
                .values()
                .filter(|organism| organism.definition == BLOOD_SAC)
                .count(),
            d.snapshot
                .condensate
                .as_ref()
                .map(|stock| stock.biomass)
                .unwrap_or(0),
            d.snapshot.food.as_ref().map_or(0, |food| food.sites.len()),
            cage,
            if d.adapt {
                "Frame: following the local shape\n"
            } else {
                "Frame: manual close-up\n"
            },
            d.nav.speed,
            d.nav.sensitivity,
            d.notice,
            if !mix.play {
                "off (capture)".to_string()
            } else if mix.muted || mix.level == 0 {
                "muted".to_string()
            } else {
                format!("{}/10", mix.level)
            }
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
    fn new_game_starts_the_same_functional_runtime_with_the_player_body() {
        let mut ship = ShipRuntime::first_simulation(AnatomySpec::bare(77), 20261123, 90).unwrap();
        assert_eq!(ship.state().config.definitions[&1].body.seed, 77);
        assert!(
            ship.state()
                .config
                .construction
                .as_ref()
                .unwrap()
                .foraging
                .is_some()
        );
        ship.advance(60).unwrap();
        assert!(ship.state().instances[&1].book.external_input > 0);
        assert!(ship.state().assembly.as_ref().unwrap().nodes() > 0);
        ship.validate().unwrap();
    }
    #[test]
    fn preview_camera_frames_the_body_and_orbit_zoom_pan_move_the_view() {
        for spec in [AnatomySpec::bare(20261123), AnatomySpec::showcase(20261123)] {
            let geometry = CompiledAnatomy::new(spec)
                .unwrap()
                .pose(Pose {
                    phase: 0.,
                    bend: 0.8,
                    mouth: 0.5,
                    flagella: 1.,
                })
                .unwrap()
                .geometry;
            let mut nav = Navigation::new();
            frame_preview(&mut nav, &geometry);
            assert_eq!(nav.mode, NavigationMode::Orbit);
            let camera = preview_transform(&nav);
            // Default 45 degree vertical field of view, with space around every surface.
            let inverse = camera.to_matrix().inverse();
            for point in geometry.surfaces.iter().flat_map(|s| &s.positions) {
                let view = inverse.transform_point3(Vec3::from_array(*point));
                assert!(
                    view.z < -0.012,
                    "camera inside/behind the anatomy: {view:?}"
                );
                assert!(
                    view.x.abs().max(view.y.abs()) < -view.z * (std::f32::consts::PI / 8.).tan(),
                    "body clipped: {view:?}"
                );
            }
            nav.look(Vec2::new(100., -40.));
            let orbited = preview_transform(&nav);
            assert_ne!(camera.translation, orbited.translation);
            assert!(
                (orbited.translation.distance(Vec3::from_array(
                    nav.pivot
                        .relative(Position::from_absolute([0; 3]).unwrap())
                        .unwrap()
                        .map(|v| v as f32 / SCALE as f32)
                )) - nav.distance as f32)
                    .abs()
                    < 0.001
            );
            nav.wheel(2.);
            assert_ne!(orbited.translation, preview_transform(&nav).translation);
            let zoomed = preview_transform(&nav);
            nav.pan(Vec2::new(15., 10.));
            assert_ne!(zoomed.translation, preview_transform(&nav).translation);
            frame_preview(&mut nav, &geometry);
            assert!(preview_transform(&nav).translation.is_finite());
        }
    }
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
