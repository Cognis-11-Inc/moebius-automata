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
    filament::{Assembly, CHUNK_EDGES, StructuralIndex, construction_fixture, filament_chunk_at},
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
    /// Use a saved player anatomy for direct first-simulation startup.
    #[arg(long, requires = "first_simulation")]
    anatomy: Option<PathBuf>,
    #[arg(long)]
    record: Option<PathBuf>,
    #[arg(long, default_value_t = 2)]
    mesh_budget: usize,
    #[arg(long, default_value_t = 0.003)]
    sensitivity: f32,
    /// Measurement world shared with `habitat_probe`: showcase anatomy, seed 77.
    /// The default first-simulation receipt stays bare anatomy, seed 20261123.
    #[arg(long, requires = "first_simulation", conflicts_with = "anatomy")]
    probe_world: bool,
    /// Founding Arachnitechs. One is the existing habitat. Each extra body is
    /// paid from condensate. The habitat budget stops at 16.
    #[arg(long, default_value_t = 1)]
    founders: usize,
    /// World seed for a direct habitat launch. Probe default 77. Bare default 20261123.
    #[arg(long)]
    seed: Option<u64>,
    /// Free condensate after the founding Blood-Sacs have been paid.
    #[arg(long)]
    abundance: Option<u64>,
}
#[derive(Resource)]
struct Authority(Arc<Mutex<Option<Worker>>>);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Selected {
    Organism(u64),
    Filament(u64),
}
type ChunkMesh = (Entity, Handle<Mesh>, Position, u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Gate {
    Title,
    Design,
    Running,
}
#[derive(Resource)]
struct Display {
    nav: Navigation,
    founders: usize,
    world_seed: u64,
    abundance: Option<u64>,
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
    wet_pipe: Handle<StandardMaterial>,
    hard_pipe: Handle<StandardMaterial>,
    sac_roots: Option<(Entity, Handle<Mesh>)>,
    frame: u64,
    out: Option<PathBuf>,
    ticks: u64,
    rows: Vec<serde_json::Value>,
    frame_log_capped: bool,
    telemetry: Arc<Mutex<Option<serde_json::Value>>>,
    show_filaments: bool,
    show_parcels: bool,
    diagnostics: bool,
    adapt: bool,
    mesh_budget: usize,
    generation: u64,
    captures: usize,
    final_capture_frame: Option<u32>,
    camera_cpu_ns: u128,
    mesh_cpu_ns: u128,
    pipe_mesh_cpu_ns: u128,
    gore_cpu_ns: u128,
    body_rebuilds: u64,
    body_reused: u64,
    last_row_tick: u64,
    shown_tick: u64,
    peak_population: usize,
    /// Wall-clock window for the authoritative tick rate. A single frame
    /// often contains no tick, so the overlay keeps the last 0.25 s sample.
    rate_mark_tick: u64,
    rate_mark: Instant,
    sim_hz: f64,
    /// tick, eating, reservoir. Unchanged pose keeps the mesh.
    body_stamp: BTreeMap<u64, (u64, u8, u64)>,
    started: Instant,
    last_frame: Instant,
}
#[derive(Component)]
struct SceneCamera;
#[derive(Component)]
struct Status;
#[derive(Component)]
struct StartSimulation;
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
    if std::env::args().nth(1).as_deref() == Some("--build-identity") {
        println!("{}", moebius_morph::BENCHMARK_BUILD_IDENTITY);
        return;
    }
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
    assert!(
        (1..=16).contains(&args.founders),
        "founding population is 1..=16"
    );
    let steps = args.ticks.min(16384) as usize;
    let mut world = if args.probe_world {
        let seed = args.seed.unwrap_or(77);
        ShipRuntime::habitat_colony(
            AnatomySpec::showcase(seed),
            seed,
            steps,
            args.founders,
            args.abundance,
        )
        .expect("habitat colony")
    } else if args.first_simulation {
        let seed = args.seed.unwrap_or(20261123);
        let spec = args
            .anatomy
            .as_ref()
            .map_or_else(|| Ok(AnatomySpec::bare(seed)), |path| load_anatomy(path))
            .expect("valid saved player anatomy");
        ShipRuntime::habitat_colony(spec, seed, steps, args.founders, args.abundance)
            .expect("habitat colony")
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
    .insert_resource(new_display(&args, snapshot, telemetry.clone()))
    .add_observer(buttons)
    .add_systems(Startup, setup)
    .add_systems(
        Update,
        (inject_input, controls, start_button_state, draw).chain(),
    );
    moebius_morph_lab::register_ambient(&mut app, args.record.is_none());
    app.run();
    let Some(worker) = authority.lock().unwrap().take() else {
        return;
    };
    let playback = worker.playback().unwrap();
    let _ = worker.control(Playback::Stop);
    let (final_state, timing) = worker.finish_measured().expect("authoritative run");
    if let Some(out) = &args.record {
        initial.save(&out.join("world-initial.json")).unwrap();
        final_state.save(&out.join("world-final.json")).unwrap();
        let vm_hwm_kb_before_replay = proc_kib("VmHWM:");
        let mut reference = ShipRuntime::restore(initial).unwrap();
        reference.advance(final_state.tick).unwrap();
        let vm_hwm_kb_after_replay = proc_kib("VmHWM:");
        let vm_rss_kb = proc_kib("VmRSS:");
        assert_eq!(reference.snapshot(), final_state);
        let mut report = telemetry.lock().unwrap().take().unwrap_or_else(|| {
            serde_json::json!({
                "status": "EARLY_CLOSE",
                "reason": "witness closed before recording telemetry was stored; saved world state is the retained evidence",
            })
        });
        let complete = final_state.tick >= args.ticks
            && report.get("status").and_then(|value| value.as_str()) != Some("EARLY_CLOSE");
        report["status"] = serde_json::json!(if complete { "PASS" } else { "EARLY_CLOSE" });
        report["ticks"] = serde_json::json!(final_state.tick);
        report["requested_ticks"] = serde_json::json!(args.ticks);
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
        report["authority"] = authority_report(&timing);
        report["vm_hwm_kb_before_replay"] = serde_json::json!(vm_hwm_kb_before_replay);
        report["vm_hwm_kb_after_replay"] = serde_json::json!(vm_hwm_kb_after_replay);
        report["vm_rss_kb"] = serde_json::json!(vm_rss_kb);
        report["percentile"] = serde_json::json!("nearest-rank, index = ceil((n-1)*p)");
        if let Some(rows) = report.get("rows").and_then(|value| value.as_array()) {
            report["presentation"] = presentation_report(rows);
        }
        report["witness_cache"] = serde_json::json!(
            "The Witness Sphere decides visibility. It does not rebuild a mesh whose stamp is unchanged. A builder mesh stamp is tick, eating, and reservoir. Blood-Sacs update transforms only. A pipe chunk stamp is its presentation age bucket and lookahead span, and at most mesh_budget chunks are rebuilt per frame. The traveling body wave changes with the tick, so one rebuild per tick is the pose. Frame interval includes the authority pace sleep and is not the tick cost."
        );
        std::fs::write(
            out.join("render-report.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
        std::fs::write(
            out.join("authority-ticks.json"),
            serde_json::to_vec(&serde_json::json!({
                "tick_ns": timing.tick_ns,
                "publish_ns": timing.publish_ns,
            }))
            .unwrap(),
        )
        .unwrap();
        println!(
            "MORPH005_WITNESS_COMPLETE {}",
            final_state.digest().unwrap()
        );
    }
}

fn authority_report(timing: &RunTiming) -> serde_json::Value {
    let profile = &timing.collision;
    let ticks = timing.tick_ns.len().max(1) as f64;
    serde_json::json!({
        "tick_ms": time_summary(&timing.tick_ns),
        "publish_ms": time_summary(&timing.publish_ns),
        "note": "tick_ms is ShipRuntime::tick on the authority thread. publish_ms is the snapshot copy handed to the witness. The pace sleep is in neither sample. Frame time is not tick time. Collision counters are not world state.",
        "collision": {
            "resolve_calls": profile.resolve_calls,
            "resolve_ns": profile.resolve_ns,
            "mean_resolve_ms": (profile.resolve_ns as f64 / ticks) / 1e6,
            "queries": profile.queries,
            "query_ns": profile.query_ns,
            "distance_queries": profile.distance_queries,
        "distance_ns": profile.distance_ns,
        "contact_ns": profile.contact_ns,
            "interval_rejects": profile.interval_rejects,
            "cast_cache_hits": profile.cast_cache_hits,
            "body_builds": profile.body_builds,
            "body_hits": profile.body_hits,
            "body_geometry_ns": profile.body_geometry_ns,
            "pipe_builds": profile.pipe_builds,
            "pipe_hits": profile.pipe_hits,
            "pipe_candidates": profile.pipe_candidates,
            "pipe_rejects": profile.pipe_rejects,
            "pipe_geometry_ns": profile.pipe_geometry_ns,
        }
    })
}

fn presentation_report(rows: &[serde_json::Value]) -> serde_json::Value {
    let intervals = json_u128(rows, "frame_interval_ns");
    serde_json::json!({
        "frames": rows.len(),
        "frame_interval_ms": time_summary(&intervals),
        "draw_cpu_ms": time_summary(&json_u128(rows, "cpu_ns")),
        "body_mesh_cpu_ms": time_summary(&json_u128(rows, "mesh_cpu_ns")),
        "pipe_mesh_cpu_ms": time_summary(&json_u128(rows, "pipe_mesh_cpu_ns")),
        "gore_cpu_ms": time_summary(&json_u128(rows, "gore_cpu_ns")),
        "camera_cpu_ms": time_summary(&json_u128(rows, "camera_cpu_ns")),
        "body_rebuilds": json_u128(rows, "body_rebuilds").iter().sum::<u128>(),
        "body_reused": json_u128(rows, "body_reused").iter().sum::<u128>(),
        "pipe_meshes_generated": json_u128(rows, "meshes_generated").iter().sum::<u128>(),
    })
}

fn json_u128(rows: &[serde_json::Value], name: &str) -> Vec<u128> {
    rows.iter()
        .filter_map(|row| row.get(name).and_then(serde_json::Value::as_u64))
        .map(u128::from)
        .collect()
}

fn time_summary(samples: &[u128]) -> serde_json::Value {
    let sum = samples.iter().copied().sum::<u128>();
    let n = samples.len().max(1) as f64;
    serde_json::json!({
        "count": samples.len(),
        "mean_ms": (sum as f64 / n) / 1e6,
        "p50_ms": nearest_rank(samples, 0.50) as f64 / 1e6,
        "p95_ms": nearest_rank(samples, 0.95) as f64 / 1e6,
        "p99_ms": nearest_rank(samples, 0.99) as f64 / 1e6,
        "max_ms": nearest_rank(samples, 1.0) as f64 / 1e6,
    })
}

fn nearest_rank(samples: &[u128], fraction: f64) -> u128 {
    if samples.is_empty() {
        return 0;
    }
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let index = ((sorted.len() - 1) as f64 * fraction).ceil() as usize;
    sorted[index.min(sorted.len() - 1)]
}

fn proc_kib(prefix: &str) -> Option<u64> {
    let text = std::fs::read_to_string("/proc/self/status").ok()?;
    text.lines().find_map(|line| {
        let rest = line.strip_prefix(prefix)?;
        rest.split_whitespace().next()?.parse().ok()
    })
}

fn new_display(
    args: &Args,
    snapshot: Arc<WorldSnapshot>,
    telemetry: Arc<Mutex<Option<serde_json::Value>>>,
) -> Display {
    let recording = args.record.is_some();
    let mut nav = Navigation::new();
    nav.sensitivity = args.sensitivity;
    let first = snapshot.instances.values().next().unwrap().id;
    // A habitat receipt follows the founding builder. Construction keeps the
    // first instance and its own scripted camera.
    let tracked = if recording && snapshot.habitat.is_some() && snapshot.instances.contains_key(&1)
    {
        1
    } else {
        first
    };
    let tracked_at = snapshot.instances[&tracked].position;
    let peak_population = snapshot
        .habitat
        .as_ref()
        .map(|habitat| habitat.agents.len())
        .unwrap_or(0);
    if let Some((center, span)) = colony_focus(&snapshot) {
        nav.focus(center, span);
    } else {
        nav.focus(tracked_at, 2.2);
    }
    if recording && snapshot.habitat.is_some() {
        nav.mode = NavigationMode::Orbit;
    }
    let opening_tick = snapshot.tick;
    Display {
        nav,
        founders: args.founders,
        world_seed: args
            .seed
            .unwrap_or(if args.probe_world { 77 } else { 20261123 }),
        abundance: args.abundance,
        sphere: WitnessSphere {
            center: tracked_at,
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
        selected: Selected::Organism(tracked),
        follow: args.construction || args.first_simulation,
        last_target: tracked_at,
        snapshot,
        bodies: BTreeMap::new(),
        chunks: BTreeMap::new(),
        index: StructuralIndex::default(),
        index_nodes: 0,
        material: default(),
        wet_pipe: default(),
        hard_pipe: default(),
        sac_roots: None,
        frame: 0,
        out: args.record.clone(),
        ticks: args.ticks,
        rows: vec![],
        frame_log_capped: false,
        telemetry: telemetry.clone(),
        show_filaments: true,
        show_parcels: true,
        diagnostics: false,
        adapt: true,
        mesh_budget: args.mesh_budget,
        generation: 0,
        captures: 0,
        final_capture_frame: None,
        started: Instant::now(),
        last_frame: Instant::now(),
        camera_cpu_ns: 0,
        mesh_cpu_ns: 0,
        pipe_mesh_cpu_ns: 0,
        gore_cpu_ns: 0,
        body_rebuilds: 0,
        body_reused: 0,
        shown_tick: 0,
        peak_population,
        rate_mark_tick: opening_tick,
        rate_mark: Instant::now(),
        sim_hz: 0.,
        last_row_tick: 0,
        body_stamp: BTreeMap::new(),
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
    display.wet_pipe = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.16,
        clearcoat: 0.85,
        clearcoat_perceptual_roughness: 0.10,
        ..default()
    });
    display.hard_pipe = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.52,
        clearcoat: 0.2,
        ..default()
    });
    if display.panel {
        spawn_panel(&mut commands);
    }
    spawn_start_button(&mut commands);
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
    start: Query<(), With<StartSimulation>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut d: ResMut<Display>,
    a: Res<Authority>,
) {
    if start.contains(event.entity) {
        start_design(&mut d, &a);
        return;
    }
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
fn spawn_start_button(commands: &mut Commands) {
    commands
        .spawn((
            Button,
            StartSimulation,
            bevy::picking::hover::Hovered::default(),
            Node {
                display: bevy::ui::Display::None,
                position_type: PositionType::Absolute,
                right: Val::Px(24.),
                bottom: Val::Px(24.),
                padding: UiRect::axes(Val::Px(28.), Val::Px(18.)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.34, 0.30)),
        ))
        .with_children(|button| {
            button.spawn((
                Text::new("Start Simulation  [Enter]"),
                TextFont {
                    font_size: FontSize::Px(22.),
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.94, 0.85)),
            ));
        });
}
fn start_button_state(
    d: Res<Display>,
    mut buttons: Query<
        (
            &mut Node,
            &mut BackgroundColor,
            &bevy::picking::hover::Hovered,
            Has<bevy::ui::Pressed>,
        ),
        With<StartSimulation>,
    >,
) {
    for (mut node, mut color, hovered, pressed) in &mut buttons {
        node.display = if d.gate == Gate::Design {
            bevy::ui::Display::Flex
        } else {
            bevy::ui::Display::None
        };
        color.0 = match (pressed, hovered.get()) {
            (true, _) => Color::srgb(0.10, 0.26, 0.23),
            (_, true) => Color::srgb(0.18, 0.46, 0.39),
            _ => Color::srgb(0.12, 0.34, 0.30),
        };
    }
}
/// Construction recordings exercise flight, orbit, and selection.
/// A habitat receipt must not inherit that script: it steals the builder.
fn construction_camera_script(d: &Display) -> bool {
    d.out.is_some() && d.snapshot.habitat.is_none()
}
/// Centre and view distance for a founding group. One body keeps the old frame.
fn colony_focus(snapshot: &WorldSnapshot) -> Option<(Position, f64)> {
    let habitat = snapshot.habitat.as_ref()?;
    if habitat.agents.len() < 2 {
        return None;
    }
    let points = habitat
        .agents
        .keys()
        .map(|id| snapshot.instances[id].position.absolute())
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    let count = points.len() as i64;
    let mean = [0, 1, 2].map(|axis| points.iter().map(|point| point[axis]).sum::<i64>() / count);
    let center = Position::from_absolute(mean).ok()?;
    let reach = points
        .iter()
        .map(|point| {
            (0..3)
                .map(|axis| (point[axis] - mean[axis]) as f64)
                .map(|value| value * value)
                .sum::<f64>()
                .sqrt()
        })
        .fold(0f64, f64::max);
    Some((center, (reach / SCALE as f64 + 4.).clamp(6., 18.)))
}
fn generation_depth(snapshot: &WorldSnapshot) -> u32 {
    let Some(habitat) = &snapshot.habitat else {
        return 0;
    };
    fn walk(
        habitat: &moebius_morph::habitat::Habitat,
        id: u64,
        seen: &mut std::collections::BTreeSet<u64>,
    ) -> u32 {
        if !seen.insert(id) {
            return 0;
        }
        match habitat.agents.get(&id).and_then(|agent| agent.parent) {
            Some(parent) => 1 + walk(habitat, parent, seen),
            None => 0,
        }
    }
    habitat
        .agents
        .keys()
        .map(|id| walk(habitat, *id, &mut std::collections::BTreeSet::new()))
        .max()
        .unwrap_or(0)
}
fn inject_input(
    d: Res<Display>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut motion: ResMut<AccumulatedMouseMotion>,
    mut wheel: MessageWriter<MouseWheel>,
    windows: Query<Entity, With<Window>>,
) {
    if !construction_camera_script(&d) {
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
        d.peak_population = d.snapshot.habitat.as_ref().map_or(0, |h| h.agents.len());
        d.rate_mark_tick = d.snapshot.tick;
        d.rate_mark = Instant::now();
        d.sim_hz = 0.;
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
        if d.snapshot.habitat.is_some() {
            let framed = colony_focus(&d.snapshot);
            let body = target(&d);
            d.follow = true;
            d.adapt = true;
            d.nav.mode = NavigationMode::Orbit;
            if let Some((center, span)) = framed {
                d.nav.focus(center, span);
            } else {
                d.nav.focus(body, 8.);
            }
            d.last_target = body;
        } else {
            d.nav = Navigation::new();
            d.follow = false;
            d.adapt = true;
        }
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
    // Interactive orbit keeps the existing fit. A habitat receipt stays outside
    // the trunk and close enough that the builder and its fresh pipes remain readable.
    if d.adapt && d.follow && d.nav.mode == NavigationMode::Orbit {
        let span = local_span(&d.snapshot, at);
        if d.out.is_none() {
            d.nav.fit(span);
        } else if d.snapshot.habitat.is_some() {
            d.nav.fit(span.clamp(6., 14.));
        }
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
    let mut spec = AnatomySpec::bare(20261123);
    spec.add_food_pouch();
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
    let world =
        match ShipRuntime::habitat_colony(spec, d.world_seed, steps, d.founders, d.abundance) {
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
            d.peak_population = d.founders;
            d.rate_mark_tick = 0;
            d.rate_mark = Instant::now();
            d.sim_hz = 0.;
            d.notice = "Finite food. Authored movement. No mortality rule.".into();
        }
        Err(error) => d.notice = format!("Worker refused: {error}"),
    }
}
/// Mouse and keyboard share validation and the canonical worker startup path.
fn start_design(d: &mut Display, authority: &Authority) {
    if d.gate != Gate::Design {
        return;
    }
    let Some(spec) = d.design.as_ref().map(|session| session.spec.clone()) else {
        return;
    };
    if !spec
        .components
        .iter()
        .any(|part| matches!(part.shape, Shape::SecretionNozzle { .. }))
    {
        d.notice = "Add a secretion nozzle before starting. Press N to add one.".into();
        return;
    }
    begin_ship(spec, d, authority);
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
        start_design(d, authority);
        return;
    }
    if keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight) {
        return;
    }
    if keys.just_pressed(KeyCode::F2) {
        d.founders = d.founders.saturating_sub(1).max(1);
    }
    if keys.just_pressed(KeyCode::F3) {
        d.founders = (d.founders + 1).min(16);
    }
    if keys.just_pressed(KeyCode::F4) {
        d.world_seed = d.world_seed.wrapping_add(1);
    }
    if keys.just_pressed(KeyCode::F7) {
        d.abundance = Some(d.abundance.unwrap_or(4000).saturating_sub(250));
    }
    if keys.just_pressed(KeyCode::F8) {
        d.abundance = Some((d.abundance.unwrap_or(4000) + 250).min(1_000_000));
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
        edits.push(Edit::Field(
            (index + ComponentField::ALL.len() - 1) % ComponentField::ALL.len(),
        ));
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
        (KeyCode::KeyB, Edit::AddFoodPouch),
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
        (KeyCode::KeyL, Edit::Reload),
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
                "DESIGN THE ARACHNITECH\nSelected {part}   {}\nA flagellum   P spine   N nozzle   B food pouch   D duplicate   Delete remove\nTab select   arrows adjust   [ ] length   - = thickness   , . mouth\nH full showcase   R another seed   S save anatomy   L reload anatomy\nDrag: orbit   Wheel: zoom   Middle drag: pan   Home: frame body\nFounders {} [F2/F3]   World seed {} [F4 +1]\nFree condensate {:?} [F7/F8 -/+250; None = default]\nClick Start Simulation or press Enter. Escape returns.\n{}\n{}",
                session.field.label(),
                d.founders,
                d.world_seed,
                d.abundance,
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
const RECORDING_FRAME_BUDGET: usize = 4096;
const RECORDING_FRAME_SAFETY: u64 = 65536;
fn admit_recording_row(rows: &mut Vec<serde_json::Value>, row: serde_json::Value) -> bool {
    if rows.len() >= RECORDING_FRAME_BUDGET {
        return false;
    }
    rows.push(row);
    true
}
fn recording_observation(d: &Display) -> serde_json::Value {
    serde_json::json!({
        "frames": d.frame,
        "captures": d.captures,
        "rows": d.rows,
        "elapsed_ns": d.started.elapsed().as_nanos(),
        "mesh_budget": d.mesh_budget,
        "frame_log_capped": d.frame_log_capped,
        "frame_rows": d.rows.len(),
        "tick": d.snapshot.tick,
        "requested_ticks": d.ticks,
        "claims": "PARTIAL. Authored habitat motion and paid snailpipe. Writhe is presentation. Propulsion is the rigid step. No learned control.",
        "gpu_timestamps": "unavailable; CPU/end-to-end frame timings only",
        "UI_test": "programmatic Activate on actual production widgets; no physical mouse playtest claimed",
    })
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
    d.mesh_cpu_ns = 0;
    d.pipe_mesh_cpu_ns = 0;
    d.gore_cpu_ns = 0;
    d.body_rebuilds = 0;
    d.body_reused = 0;
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
    // The construction recording looks away and resets. A habitat receipt must
    // keep hunting, feeding, and laying pipe without that scripted interruption.
    if construction_camera_script(&d) {
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
        let mut filled_spec = d.snapshot.config.definitions[&instance.definition].clone();
        let capacity = filled_spec.food_capacity();
        if capacity > 0 {
            for c in &mut filled_spec.components {
                if let Shape::FoodPouch { radius, .. } = &mut c.shape {
                    *radius *= 0.65
                        + 0.35 * (instance.book.reservoir as f32 / capacity as f32).clamp(0., 1.);
                    *radius = radius.max(0.08);
                }
            }
        }
        let spec = &filled_spec;
        let pose = d.snapshot.anatomical_pose(*id).unwrap();
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
            .with_rotation(
                Quat::from_rotation_y(yaw)
                    * Quat::from_rotation_z(rigid.axial_roll as f32 / 1_000_000.),
            )
            .with_scale(Vec3::splat(scale));
        // Blood-Sac anatomy is rest-pose geometry; growth and evasion change
        // only its transform. Do not regenerate hundreds of unchanged surfaces.
        if instance.definition == BLOOD_SAC
            && let Some(parts) = d.bodies.get(id)
        {
            for (e, _) in parts {
                if let Ok(mut t) = transforms.get_mut(*e) {
                    *t = transform;
                }
            }
            continue;
        }
        let eating = d
            .snapshot
            .habitat
            .as_ref()
            .and_then(|habitat| habitat.agents.get(id))
            .is_some_and(|agent| agent.eating);
        // Sphere visibility does not rebuild an unchanged stamp. The wave's
        // phase advances with the tick, so a new tick is a real pose change.
        let stamp = (d.snapshot.tick, u8::from(eating), instance.book.reservoir);
        if d.bodies.contains_key(id) && d.body_stamp.get(id) == Some(&stamp) {
            for (e, _) in &d.bodies[id] {
                if let Ok(mut t) = transforms.get_mut(*e) {
                    *t = transform;
                }
            }
            d.body_reused += 1;
            continue;
        }
        let mesh_started = Instant::now();
        let g = CompiledAnatomy::new(spec.clone())
            .unwrap()
            .pose(pose)
            .unwrap();
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
        d.body_stamp.insert(*id, stamp);
        d.body_rebuilds += 1;
        d.mesh_cpu_ns += mesh_started.elapsed().as_nanos();
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
            && d.sphere.contains(*mouth).unwrap_or(false)
            && d.sphere.contains(sac.position).unwrap_or(false)
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
    let mut roots = Vec::new();
    if let Some(h) = &d.snapshot.habitat {
        for (id, a) in &h.sacs {
            if !visible.contains(id) {
                continue;
            }
            if let Some(root) = moebius_morph::habitat::anchor(&d.snapshot, a) {
                if !d.sphere.contains(root).unwrap_or(false) {
                    continue;
                }
                let local = |p: Position| {
                    Vec3::from_array(
                        p.relative(d.nav.position)
                            .unwrap()
                            .map(|v| v as f32 / SCALE as f32),
                    )
                };
                roots.push((local(root), local(d.snapshot.instances[id].position)));
            }
        }
    }
    let material = d.material.clone();
    paint(
        &mut commands,
        &mut meshes,
        &material,
        &mut d.sac_roots,
        moebius_morph_lab::gore::substrate_roots(&roots),
    );
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
    let pipe_started = Instant::now();
    for key in desired {
        let f = &d.snapshot.assembly.as_ref().unwrap().filaments[&key.0];

        let end = (key.1 * CHUNK_EDGES + CHUNK_EDGES).min(f.nodes.len() - 1);
        let age = d
            .snapshot
            .tick
            .saturating_sub(f.nodes[end].deposited_tick.max(f.created_tick));
        let phase = (age / 24).min(8);
        // Presentation bucket only. Collision keeps the finer wet-to-dry age.
        // One-neighbour lookahead invalidates a sealed chunk's shared end tangent once.
        // An unchanged stamp is not rebuilt. Off-sphere chunks are despawned.
        let stamp =
            (((f.nodes.len() - key.1 * CHUNK_EDGES).min(CHUNK_EDGES + 2) as u64) << 8) | phase;
        let changed = d.chunks.get(&key).is_none_or(|v| v.3 != stamp);
        if changed && generated < d.mesh_budget {
            let (origin, surface) = filament_chunk_at(f, key.1, d.snapshot.tick).unwrap();
            let h = meshes.add(mesh(&surface));
            let at = origin
                .relative(d.nav.position)
                .unwrap()
                .map(|v| v as f32 / SCALE as f32);
            let e = commands
                .spawn((
                    Mesh3d(h.clone()),
                    MeshMaterial3d(if phase < 4 {
                        d.wet_pipe.clone()
                    } else {
                        d.hard_pipe.clone()
                    }),
                    Transform::from_translation(Vec3::from_array(at)),
                ))
                .id();
            if let Some((old, old_h, _, _)) = d.chunks.insert(key, (e, h, origin, stamp)) {
                commands.entity(old).despawn();
                meshes.remove(old_h.id());
            }
            generated += 1;
        }
    }
    d.pipe_mesh_cpu_ns += pipe_started.elapsed().as_nanos();
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
    let gore_started = Instant::now();
    paint(
        &mut commands,
        &mut meshes,
        &material,
        &mut d.food_gore,
        moebius_morph_lab::gore::food_gore(&sites, frame),
    );
    d.gore_cpu_ns += gore_started.elapsed().as_nanos();
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
    let pouch_capacity =
        d.snapshot.config.definitions[&d.snapshot.instances[&1].definition].food_capacity();
    let activity = d
        .snapshot
        .steering
        .map_or("recorded demonstration", |state| {
            if state.seeking_food {
                "seeking food"
            } else {
                if d.snapshot
                    .config
                    .construction
                    .as_ref()
                    .and_then(|r| r.foraging.as_ref())
                    .is_some_and(|r| r.axial_spin != 0)
                {
                    "stocked / braiding"
                } else {
                    "stocked / building"
                }
            }
        });
    let activity = if let Some(h) = &d.snapshot.habitat {
        if h.agents[&1].eating {
            "feeding / secretion closed"
        } else if !book.emission_enabled {
            "exploring / secretion disabled"
        } else if book.external_input > 0
            && book.bound.len() >= book.registered.len() * 8
            && book.reservoir <= h.rules.reserve_food.min(pouch_capacity / 4)
        {
            "exploring / conserving pouch"
        } else {
            "exploring / laying trails"
        }
    } else {
        activity
    };
    let cage = if let Some(h) = &d.snapshot.habitat {
        format!(
            "{cage}Arachnitech {} | rooted sacs {} | wall contacts {} (closed)\n",
            h.agents.len(),
            h.sacs.len(),
            d.snapshot
                .assembly
                .as_ref()
                .map_or(0, |a| a.junctions.len())
        )
    } else {
        cage
    };
    let (colony_line, next_peak, next_shown, next_hz, refresh_rate) = if let Some(habitat) =
        d.snapshot.habitat.as_ref()
    {
        let agents = habitat.agents.len();
        let peak = d.peak_population.max(agents);
        let births = d.snapshot.total_births();
        let stored: u64 = d
            .snapshot
            .instances
            .values()
            .filter(|organism| organism.definition != BLOOD_SAC)
            .map(|organism| organism.book.reservoir)
            .sum();
        let sac_food: u64 = d
            .snapshot
            .instances
            .values()
            .filter(|organism| organism.definition == BLOOD_SAC)
            .map(|organism| organism.book.reservoir)
            .sum();
        let shown = d.snapshot.tick;
        let sim_s = shown as f64 * (TICK_NANOS as f64 / 1e9);
        let frame_s = (frame_interval as f64 / 1e9).max(1e-6);
        let depth = generation_depth(&d.snapshot);
        let window_s = d.rate_mark.elapsed().as_secs_f64();
        let (tick_hz, refresh_rate) = if window_s >= 0.25 {
            (
                shown.saturating_sub(d.rate_mark_tick) as f64 / window_s,
                true,
            )
        } else {
            (d.sim_hz, false)
        };
        let frame_hz = 1. / frame_s;
        (
            format!(
                "Sim {sim_s:.2}s | colony {agents} peak {peak}\nBirths {births} | deaths N/A | generation {depth}\nFood {sac_food} | stored {stored}\nTick {tick_hz:.1} Hz | frame {frame_hz:.1} Hz\n"
            ),
            peak,
            shown,
            tick_hz,
            refresh_rate,
        )
    } else {
        (
            String::new(),
            d.peak_population,
            d.shown_tick,
            d.sim_hz,
            false,
        )
    };
    for mut t in &mut text {
        let details = format!(
            "MŒBIUS | tick {} / {} | {}x {}\n{colony_line}Selected {:?} | {:?} | follow {}\nCoordinates [um] {:?}\nArachnitech: {}\nFood pouch {}/{} | eaten {} | free {} | bound {} | balance {}\nFilaments {} | nodes {} | visible chunks {}\nBlood-Sacs {} | condensate {} | generic sites {}\n{}{}Flight speed {:.2}m/s | sensitivity {:.4}\nPale snailpipe trails, amber secretion, red Blood-Sacs. Uptake lines mark meals.\n{}\nDrone {}   M mute   [ ] volume\nG adapts the orbit. Wheel dollies in. F5 saves. Home frames the body.\nLocal pursuit/evasion (authored). Blood-Sacs root on paid snailpipe substrate.",
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
            pouch_capacity,
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
        t.0 = if d.diagnostics || d.snapshot.habitat.is_none() {
            details
        } else {
            format!(
                "MŒBIUS  {} / {}  {}x {}\n{colony_line}Blood-Sacs {} | nodes {nodes} | material gap {}\nPouch {}/{} | eaten {} | {}\n{:?} | {:?} | follow {}\n[B] details   [Home] frame   [F5] save\nMortality / metabolism: unimplemented",
                d.snapshot.tick,
                d.ticks,
                playback.rate,
                if playback.playing { "PLAY" } else { "PAUSE" },
                d.snapshot
                    .instances
                    .values()
                    .filter(|i| i.definition == BLOOD_SAC)
                    .count(),
                d.snapshot.condensate_gap(),
                book.reservoir,
                pouch_capacity,
                book.external_input,
                activity,
                d.selected,
                d.nav.mode,
                d.follow,
            )
        };
    }
    d.peak_population = next_peak;
    d.shown_tick = next_shown;
    if refresh_rate {
        d.rate_mark_tick = next_shown;
        d.rate_mark = Instant::now();
        d.sim_hz = next_hz;
    }
    assert_eq!(before, d.snapshot.digest().unwrap());
    if d.out.is_some() {
        let tick = d.snapshot.tick;
        let chunks = d.chunks.len();
        let cpu = started.elapsed().as_nanos();
        let camera = d.nav.position;
        let generation = d.generation;
        let camera_cpu_ns = d.camera_cpu_ns;
        let camera_mode = format!("{:?}", d.nav.mode);
        let camera_rotation = d.nav.rotation.to_array();
        let camera_speed = d.nav.speed;
        let tick_delta = tick.saturating_sub(d.last_row_tick);
        d.last_row_tick = tick;
        let row = serde_json::json!({"frame":frame,"tick":tick,"tick_delta":tick_delta,"generation":generation,"digest":before,"nodes":nodes,"bound":bound,"free":free,"filaments":filaments,"chunks":chunks,"meshes_generated":generated,"cpu_ns":cpu,"frame_interval_ns":frame_interval,"camera_cpu_ns":camera_cpu_ns,"mesh_cpu_ns":d.mesh_cpu_ns,"pipe_mesh_cpu_ns":d.pipe_mesh_cpu_ns,"gore_cpu_ns":d.gore_cpu_ns,"body_rebuilds":d.body_rebuilds,"body_reused":d.body_reused,"mesh_assets":meshes.len(),"camera":camera,"camera_mode":camera_mode,"camera_rotation":camera_rotation,"camera_speed":camera_speed});
        if !admit_recording_row(&mut d.rows, row) && !d.frame_log_capped {
            d.frame_log_capped = true;
            eprintln!(
                "Recording frame budget exceeded; frame log stopped at {RECORDING_FRAME_BUDGET}; world ticks continue"
            );
        }
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
        if frame >= 320 && d.snapshot.tick >= d.ticks && d.final_capture_frame.is_none() {
            if d.snapshot.habitat.is_some() && d.snapshot.instances.contains_key(&1) {
                d.selected = Selected::Organism(1);
                d.follow = true;
                d.adapt = true;
                // last_target must match the new selection before the next follow step.
                // A stale Blood-Sac selection would throw the closing camera off the builder.
                d.last_target = target(&d);
                if d.nav.mode != NavigationMode::Orbit {
                    d.nav.switch();
                }
                let at = d.last_target;
                let span = local_span(&d.snapshot, at).clamp(6., 14.);
                d.nav.focus(at, span);
            } else {
                d.selected = Selected::Organism(1);
                d.follow = true;
                let at = target(&d);
                let span = local_span(&d.snapshot, at).clamp(5., 10.);
                d.nav.focus(at, span);
            }
            d.final_capture_frame = Some(frame);
        }
        if let Some(last) = d.final_capture_frame
            && frame == last + 8
        {
            d.captures += 1;
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(
                    d.out.as_ref().unwrap().join("final_habitat.png"),
                ));
        }
        if d.frame >= RECORDING_FRAME_SAFETY && d.snapshot.tick < d.ticks {
            let mut report = recording_observation(&d);
            report["status"] = serde_json::json!("EARLY_CLOSE");
            report["reason"] = serde_json::json!(
                "presentation safety ceiling before the requested tick; world history to this tick retained"
            );
            *d.telemetry.lock().unwrap() = Some(report);
            eprintln!(
                "Recording presentation ceiling {RECORDING_FRAME_SAFETY} reached at tick {}; requested {}",
                d.snapshot.tick, d.ticks
            );
            exit.write(AppExit::error());
            return;
        }
        if d.final_capture_frame.is_some_and(|last| frame >= last + 12) {
            *d.telemetry.lock().unwrap() = Some(recording_observation(&d));
            exit.write(AppExit::Success);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn creator_app() -> App {
        let args = Args::parse_from(["witness", "--ticks", "90"]);
        let snapshot = Arc::new(fixture(1, 77).unwrap().snapshot());
        let mut display = new_display(&args, snapshot, Arc::new(Mutex::new(None)));
        open_design(&mut display);
        let mut app = App::new();
        app.insert_resource(display)
            .insert_resource(Authority(Arc::new(Mutex::new(None))))
            .insert_resource(ButtonInput::<KeyCode>::default())
            .add_observer(buttons)
            .add_systems(Startup, |mut commands: Commands| {
                spawn_start_button(&mut commands)
            })
            .add_systems(Update, start_button_state);
        app.update();
        app
    }
    fn start_entity(app: &mut App) -> Entity {
        app.world_mut()
            .query_filtered::<Entity, With<StartSimulation>>()
            .single(app.world())
            .unwrap()
    }
    fn stop_and_check_run(app: &mut App) {
        assert_eq!(app.world().resource::<Display>().gate, Gate::Running);
        assert!(app.world().resource::<Display>().design.is_none());
        let lock = app.world().resource::<Authority>().0.clone();
        let worker = lock.lock().unwrap().take().unwrap();
        assert!(worker.playback().unwrap().playing);
        let deadline = Instant::now() + Duration::from_secs(3);
        while worker.latest().unwrap().tick < 2 && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(2));
        }
        worker.control(Playback::Stop).unwrap();
        let snapshot = worker.finish().unwrap();
        assert!(snapshot.tick >= 2, "startup must advance committed ticks");
        assert_eq!(snapshot.config.definitions[&1].body.seed, 20261123);
        ShipRuntime::restore(snapshot).unwrap().validate().unwrap();
    }
    #[test]
    fn recording_frame_log_caps_at_the_budget_and_rejects_another_row() {
        let mut rows = vec![serde_json::json!(0); RECORDING_FRAME_BUDGET - 1];
        assert!(admit_recording_row(&mut rows, serde_json::json!("last")));
        assert_eq!(rows.len(), RECORDING_FRAME_BUDGET);
        assert!(!admit_recording_row(
            &mut rows,
            serde_json::json!("overflow")
        ));
        assert_eq!(rows.len(), RECORDING_FRAME_BUDGET);
    }
    #[test]
    fn creator_start_button_runs_the_player_body_and_hides_after_start() {
        let mut app = creator_app();
        // Exercise the real editor actions that originally triggered startup refusal.
        for _ in 0..3 {
            app.world_mut()
                .resource_mut::<Display>()
                .design
                .as_mut()
                .unwrap()
                .edit(Edit::AddNozzle)
                .unwrap();
        }
        let button = start_entity(&mut app);
        assert_eq!(
            app.world().get::<Node>(button).unwrap().display,
            bevy::ui::Display::Flex
        );
        app.world_mut().trigger(Activate { entity: button });
        app.update();
        assert_eq!(
            app.world()
                .resource::<Authority>()
                .0
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .latest()
                .unwrap()
                .instances[&1]
                .book
                .registered
                .len(),
            4
        );
        assert_eq!(
            app.world().get::<Node>(button).unwrap().display,
            bevy::ui::Display::None
        );
        stop_and_check_run(&mut app);
    }
    #[test]
    fn creator_launch_uses_founders_seed_and_finite_abundance() {
        let mut app = creator_app();
        {
            let mut d = app.world_mut().resource_mut::<Display>();
            d.founders = 2;
            d.world_seed = 23;
            d.abundance = Some(1500);
        }
        let spec = app
            .world()
            .resource::<Display>()
            .design
            .as_ref()
            .unwrap()
            .spec
            .clone();
        let button = start_entity(&mut app);
        app.world_mut().trigger(Activate { entity: button });
        let lock = app.world().resource::<Authority>().0.clone();
        let worker = lock.lock().unwrap().take().unwrap();
        worker.control(Playback::Stop).unwrap();
        let snapshot = worker.finish().unwrap();
        assert_eq!(snapshot.config.seed, 23);
        assert_eq!(snapshot.habitat.as_ref().unwrap().agents.len(), 2);
        assert!(
            snapshot
                .habitat
                .as_ref()
                .unwrap()
                .agents
                .values()
                .all(|a| a.parent.is_none())
        );
        assert_eq!(snapshot.condensate_gap(), 0);
        let mut control = ShipRuntime::habitat_colony(spec, 23, 90, 2, Some(1500)).unwrap();
        control.advance(snapshot.tick).unwrap();
        assert_eq!(snapshot, control.snapshot());
        assert_eq!(app.world().resource::<Display>().peak_population, 2);
    }
    #[test]
    fn creator_enter_still_starts_the_same_worker() {
        let mut app = creator_app();
        let mut keys = ButtonInput::default();
        keys.press(KeyCode::Enter);
        let authority = Authority(app.world().resource::<Authority>().0.clone());
        gate_keys(
            &keys,
            &Time::default(),
            &mut app.world_mut().resource_mut::<Display>(),
            &authority,
        );
        stop_and_check_run(&mut app);
    }
    #[test]
    fn creator_start_button_preserves_design_and_explains_missing_nozzle() {
        let mut app = creator_app();
        app.world_mut()
            .resource_mut::<Display>()
            .design
            .as_mut()
            .unwrap()
            .spec
            .components
            .retain(|part| !matches!(part.shape, Shape::SecretionNozzle { .. }));
        let button = start_entity(&mut app);
        app.world_mut().trigger(Activate { entity: button });
        app.update();
        let d = app.world().resource::<Display>();
        assert_eq!(d.gate, Gate::Design);
        assert!(d.design.is_some());
        assert!(d.notice.contains("nozzle"));
        assert!(
            app.world()
                .resource::<Authority>()
                .0
                .lock()
                .unwrap()
                .is_none()
        );
        assert_eq!(
            app.world().get::<Node>(button).unwrap().display,
            bevy::ui::Display::Flex
        );
    }
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
    fn habitat_recording_orbits_the_founder_and_skips_the_construction_camera_script() {
        let habitat =
            ShipRuntime::habitat_simulation(AnatomySpec::bare(20261123), 20261123, 8).unwrap();
        let snapshot = Arc::new(habitat.snapshot());
        assert!(snapshot.habitat.is_some());
        let args = Args::parse_from([
            "witness",
            "--first-simulation",
            "--ticks",
            "8",
            "--record",
            "/tmp/moebius-habitat-camera-gate",
        ]);
        let display = new_display(&args, snapshot, Arc::new(Mutex::new(None)));
        assert!(!construction_camera_script(&display));
        assert_eq!(display.nav.mode, NavigationMode::Orbit);
        assert!(display.follow);
        assert_eq!(display.selected, Selected::Organism(1));
        let construction =
            ShipRuntime::first_simulation(AnatomySpec::bare(77), 20261123, 8).unwrap();
        let plain = Arc::new(construction.snapshot());
        assert!(plain.habitat.is_none());
        let args = Args::parse_from([
            "witness",
            "--construction",
            "--ticks",
            "8",
            "--record",
            "/tmp/moebius-construction-camera-gate",
        ]);
        let scripted = new_display(&args, plain, Arc::new(Mutex::new(None)));
        assert!(construction_camera_script(&scripted));
        assert_eq!(scripted.nav.mode, NavigationMode::Flight);
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
