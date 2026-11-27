//! Experimental environmental observer. The only numerical owner is Chamber.
use bevy::{
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
};
use clap::Parser;
use moebius_morph::primordial::{Chamber, Config, StepProfile, experiments, norm};
use moebius_morph::primordial::{
    Mass,
    scalable::{self, Lane, Workspace},
};
use std::{path::PathBuf, time::Instant};
#[derive(Parser)]
struct Args {
    #[arg(long)]
    record: Option<PathBuf>,
    #[arg(long, default_value_t = 120)]
    ticks: usize,
    #[arg(long, default_value_t = 16)]
    cells: usize,
    #[arg(long, default_value = "Reference")]
    lane: String,
    #[arg(long, default_value_t = 4)]
    block_edge: usize,
    #[arg(long, default_value = "demo")]
    scenario: String,
}
#[derive(Resource)]
struct Authority {
    world: Chamber,
    lane: Lane,
    workspace: Workspace,
    block_edge: usize,
    scenario: String,
    playing: bool,
    single: bool,
    accumulator: f64,
    error: Option<String>,
    profiles: Vec<StepProfile>,
    frames_ms: Vec<f64>,
    record: Option<PathBuf>,
    target: usize,
    completed_frames: usize,
    initial_digest: String,
}
#[derive(Resource, Default)]
struct RenderTimings {
    draw_ns: Vec<u128>,
}
#[derive(Clone, Copy)]
enum Action {
    Toggle,
    Step,
    Reset,
    Density,
    Viscosity,
    Gravity,
    GravityStrength,
}
#[derive(Resource, Default)]
struct Pending(Vec<Action>);
#[derive(Resource)]
struct View {
    yaw: f32,
    pitch: f32,
    distance: f32,
    flow: bool,
    species: usize,
}
#[derive(Component)]
struct Hud;
#[derive(Component)]
struct ObserverCamera;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().nth(1).as_deref() == Some("--build-identity") {
        println!("{}", moebius_morph::BENCHMARK_BUILD_IDENTITY);
        return Ok(());
    }
    let args = Args::parse();
    if args.ticks == 0 || args.ticks > 16384 {
        return Err("ticks outside recording budget".into());
    }
    if let Some(path) = &args.record {
        std::fs::create_dir(path)?;
    }
    let lane: Lane = serde_json::from_str(&format!("\"{}\"", args.lane))?;
    let world = new_world(
        Config {
            cells: args.cells,
            ..Default::default()
        },
        &args.scenario,
        lane,
        args.block_edge,
    )?;
    let initial_digest = world.digest()?;
    if let Some(path) = &args.record {
        world.save_candidate(&path.join("initial.json"), lane, 1)?;
    }
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.007, 0.012, 0.021)))
        .insert_resource(Authority {
            world,
            lane,
            workspace: Workspace::default(),
            block_edge: args.block_edge,
            scenario: args.scenario,
            playing: true,
            single: false,
            accumulator: 0.,
            error: None,
            profiles: Vec::new(),
            frames_ms: Vec::new(),
            record: args.record,
            target: args.ticks,
            completed_frames: 0,
            initial_digest,
        })
        .insert_resource(Pending::default())
        .insert_resource(RenderTimings::default())
        .insert_resource(View {
            yaw: 0.68,
            pitch: 0.38,
            distance: 2.1,
            flow: true,
            species: 0,
        })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "HABITAT-002 | Experimental computational lanes".into(),
                resolution: (1280, 900).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, (controls, advance, draw, record).chain())
        .run();
    Ok(())
}
fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            near: 0.005,
            far: 20.,
            ..default()
        }),
        Transform::from_xyz(1.3, 1.1, 1.4).looking_at(Vec3::splat(0.4), Vec3::Y),
        ObserverCamera,
    ));
    commands.spawn((
        Text::new("HABITAT-002"),
        TextFont {
            font_size: FontSize::Px(14.),
            ..default()
        },
        TextColor(Color::srgb(0.82, 0.91, 0.94)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(16.),
            top: Val::Px(16.),
            width: Val::Px(330.),
            ..default()
        },
        Hud,
    ));
}
fn controls(
    keys: Res<ButtonInput<KeyCode>>,
    authority: Res<Authority>,
    mut pending: ResMut<Pending>,
    mut view: ResMut<View>,
    time: Res<Time>,
    mut camera: Query<&mut Transform, With<ObserverCamera>>,
) {
    if authority.record.is_none() {
        for (key, action) in [
            (KeyCode::Space, Action::Toggle),
            (KeyCode::KeyN, Action::Step),
            (KeyCode::KeyR, Action::Reset),
            (KeyCode::KeyD, Action::Density),
            (KeyCode::KeyV, Action::Viscosity),
            (KeyCode::KeyG, Action::Gravity),
            (KeyCode::KeyB, Action::GravityStrength),
        ] {
            if keys.just_pressed(key) {
                pending.0.push(action);
            }
        }
    }
    if keys.just_pressed(KeyCode::KeyF) {
        view.flow = !view.flow;
    }
    if keys.just_pressed(KeyCode::KeyC) {
        view.species = (view.species + 1) % 2;
    }
    let dt = time.delta_secs();
    if keys.pressed(KeyCode::ArrowLeft) {
        view.yaw -= dt;
    }
    if keys.pressed(KeyCode::ArrowRight) {
        view.yaw += dt;
    }
    if keys.pressed(KeyCode::ArrowUp) {
        view.pitch = (view.pitch + dt).min(1.3);
    }
    if keys.pressed(KeyCode::ArrowDown) {
        view.pitch = (view.pitch - dt).max(-1.3);
    }
    if keys.pressed(KeyCode::PageUp) {
        view.distance = (view.distance - dt).max(0.6);
    }
    if keys.pressed(KeyCode::PageDown) {
        view.distance = (view.distance + dt).min(4.);
    }
    for mut camera in &mut camera {
        let target = Vec3::splat(authority.world.state().config.length_m as f32 * 0.5);
        camera.translation = target
            + view.distance
                * Vec3::new(
                    view.yaw.sin() * view.pitch.cos(),
                    view.pitch.sin(),
                    view.yaw.cos() * view.pitch.cos(),
                );
        let right = Vec3::new(view.yaw.cos(), 0., -view.yaw.sin());
        camera.look_at(target - right * 0.24, Vec3::Y);
    }
}
fn advance(mut authority: ResMut<Authority>, mut pending: ResMut<Pending>, time: Res<Time>) {
    let authority = &mut *authority;
    for action in pending.0.drain(..) {
        match action {
            Action::Toggle => authority.playing = !authority.playing,
            Action::Step => authority.single = true,
            change => {
                let mut config = authority.world.state().config.clone();
                match change {
                    Action::Density => {
                        config.density_kg_m3 = if config.density_kg_m3 < 30. {
                            config.density_kg_m3 * 2.
                        } else {
                            10.
                        }
                    }
                    Action::Viscosity => {
                        config.viscosity_pa_s = if config.viscosity_pa_s < 0.15 {
                            config.viscosity_pa_s * 2.
                        } else {
                            0.05
                        }
                    }
                    Action::GravityStrength => {
                        let old = norm(config.gravity_m_s2);
                        let next = if old < 0.015 {
                            (old * 2.).max(0.001)
                        } else {
                            0.001
                        };
                        config.gravity_m_s2 = if old > 0. {
                            config.gravity_m_s2.map(|v| v / old * next)
                        } else {
                            [0., -next, 0.]
                        };
                    }
                    Action::Gravity => {
                        config.gravity_m_s2 = if config.gravity_m_s2 == [0.; 3] {
                            [0., -0.0025, 0.]
                        } else if config.gravity_m_s2[1] < 0. {
                            [0.0025, 0., 0.]
                        } else {
                            [0.; 3]
                        }
                    }
                    _ => {}
                }
                match new_world(
                    config,
                    &authority.scenario,
                    authority.lane,
                    authority.block_edge,
                ) {
                    Ok(world) => {
                        authority.world = world;
                        authority.workspace = Workspace::default();
                        authority.error = None;
                        authority.accumulator = 0.;
                        authority.profiles.clear();
                    }
                    Err(e) => authority.error = Some(format!("New run refused: {e}")),
                }
            }
        }
    }
    if authority.error.is_some() {
        return;
    }
    let recording = authority.record.is_some();
    if recording && authority.world.state().tick >= authority.target as u64 {
        return;
    }
    if authority.playing || recording {
        authority.accumulator += time.delta_secs_f64();
    }
    let mut steps = 0;
    while (recording && steps == 0)
        || authority.single
        || (!recording
            && authority.playing
            && authority.accumulator >= moebius_morph::runtime::TICK_NANOS as f64 / 1e9
            && steps < 4)
    {
        match authority
            .world
            .step_candidate(authority.lane, &mut authority.workspace)
        {
            Ok(profile) => {
                if authority.profiles.len() < 16384 {
                    authority.profiles.push(profile);
                } else {
                    authority.profiles.remove(0);
                    authority.profiles.push(profile);
                }
            }
            Err(e) => {
                authority.error = Some(e.to_string());
                break;
            }
        }
        steps += 1;
        authority.single = false;
        if !recording {
            authority.accumulator -= moebius_morph::runtime::TICK_NANOS as f64 / 1e9;
        }
    }
    if !authority.playing && !authority.single {
        authority.accumulator = authority.accumulator.min(0.1);
    }
}
fn draw(
    authority: Res<Authority>,
    view: Res<View>,
    mut gizmos: Gizmos,
    mut hud: Query<&mut Text, With<Hud>>,
    mut timings: ResMut<RenderTimings>,
) {
    let started = Instant::now();
    let state = authority.world.state();
    let length = state.config.length_m as f32;
    for axis in 0..3 {
        for a in [0., length] {
            for b in [0., length] {
                let mut p = [0.; 3];
                p[(axis + 1) % 3] = a;
                p[(axis + 2) % 3] = b;
                let mut q = p;
                q[axis] = length;
                gizmos.line(
                    Vec3::from_array(p),
                    Vec3::from_array(q),
                    Color::srgb(0.26, 0.47, 0.54),
                );
            }
        }
    }
    let n = state.config.cells;
    let mut block_keys = std::collections::BTreeSet::new();
    if authority.lane == Lane::Blocks {
        for f in &state.fields {
            if let Mass::Blocks(b) = &f.mass {
                for &k in b.blocks.keys() {
                    block_keys.insert(k);
                }
            }
        }
    } else if authority.lane == Lane::ImplicitCarrier {
        block_keys.extend(authority.workspace.carrier_blocks());
    }
    let edge = if authority.lane == Lane::Blocks {
        authority.block_edge
    } else {
        4
    };
    let nb = n.div_ceil(edge);
    for &key in block_keys.iter().take(512) {
        let base = [
            key % nb * edge,
            key / nb % nb * edge,
            key / (nb * nb) * edge,
        ];
        let h = state.config.h() as f32;
        for axis in 0..3 {
            for a in [0, edge] {
                for b in [0, edge] {
                    let mut p = base;
                    p[(axis + 1) % 3] += a;
                    p[(axis + 2) % 3] += b;
                    let mut q = p;
                    q[axis] += edge;
                    gizmos.line(
                        Vec3::from_array(p.map(|v| v.min(n) as f32 * h)),
                        Vec3::from_array(q.map(|v| v.min(n) as f32 * h)),
                        Color::srgba(0.15, 0.55, 0.45, 0.25),
                    );
                }
            }
        }
    }
    let field = &state.fields[view.species % state.fields.len()];
    let peak = field
        .mass
        .nonzero()
        .iter()
        .map(|(_, v)| *v)
        .fold(0f64, f64::max);
    for i in 0..state.config.count() {
        let p = state.config.coordinates(i);
        let at = Vec3::from_array(state.config.centre(i).map(|v| v as f32));
        let mass = field.mass.get(i);
        if peak > 0. && mass / peak > 0.015 {
            let fraction = (mass / peak) as f32;
            let color = if view.species == 0 {
                Color::srgba(0.22, 0.85, 0.87, 0.35 + 0.6 * fraction)
            } else {
                Color::srgba(0.78, 0.46, 0.98, 0.35 + 0.6 * fraction)
            };
            gizmos.sphere(
                Isometry3d::from_translation(at),
                0.003 + fraction * 0.008,
                color,
            );
        }
        if view.flow && p.iter().all(|i| i % 3 == 0) && i < n.pow(3) {
            let velocity = Vec3::from_array(state.sample_index(i).velocity_m_s.map(|v| v as f32));
            if velocity.length() > 0.00001 {
                gizmos.arrow(at, at + velocity * 5., Color::srgba(0.38, 0.79, 0.56, 0.7));
            }
        }
    }
    for body in &state.spheres {
        let at = Vec3::from_array(body.position_m.map(|v| v as f32));
        let color = if body.id == 1 {
            Color::WHITE
        } else {
            Color::srgb(1., 0.65, 0.26)
        };
        gizmos.sphere(
            Isometry3d::from_translation(at),
            body.radius_m as f32,
            color,
        );
    }
    let g = state.config.gravity_m_s2;
    let gravity_at = Vec3::new(length + 0.1, length * 0.75, length * 0.5);
    gizmos.arrow(
        gravity_at,
        gravity_at + Vec3::from_array(g.map(|v| v as f32)) * 40.,
        Color::srgb(1., 0.48, 0.36),
    );
    let diag = state.diagnostics();
    let step = authority
        .profiles
        .last()
        .map_or(0., |p| p.total_ns as f64 / 1e6);
    let activity = &authority.workspace.activity;
    let text = format!(
        "HABITAT-002\nCOMPUTATIONAL EXPERIMENT\nLane: {:?} / {}\nUniform resolution; CPU owner\nGPU timing/memory: N/A\n\nChamber {:.2} m / {}^3 cells\nTick {} / dt {:.6} s\nDensity {:.3} kg/m^3\nViscosity {:.3} Pa*s\nGravity [{:.4}, {:.4}, {:.4}] m/s^2\n\n{}: suspended {:.3e} kg\nWall stock {:.3e} kg\nCarrier residual {:.3e} kg\nMomentum residual {:.3e} kg*m/s\nLattice Mach {:.4}\nUpdate {:.3} ms\n\nScalar/carrier blocks: {} / {}\nImplicit carrier cells: {} (D kernel)\nScalar occupied {} / {} slots\nScratch {} KiB; block events +{} -{}\nDecoded authoritative State stays dense\n\nSpace pause; N step; R reset\nD density; V viscosity\nG gravity direction; B strength\nC species; F flow arrows\nArrows orbit; PgUp/PgDn zoom\nParameter change starts NEW RUN\n\nCyan/violet: concentration samples\nWhite/orange: physical spheres\nGreen grid: allocated blocks\nFlow gain 5s; gravity gain 40s^2\nNo canonical ship integration\n{}",
        authority.lane,
        authority.scenario,
        length,
        n,
        state.tick,
        state.config.dt(),
        state.config.density_kg_m3,
        state.config.viscosity_pa_s,
        g[0],
        g[1],
        g[2],
        field.species.material,
        field.mass.total(),
        field.wall_kg.iter().sum::<f64>(),
        diag.carrier_residual_kg,
        norm(diag.momentum_residual),
        diag.max_mach,
        step,
        activity.scalar_blocks,
        activity.active_carrier_blocks,
        activity.implicit_carrier_cells,
        state
            .fields
            .iter()
            .map(|f| f.mass.nonzero().len())
            .sum::<usize>(),
        state.fields.iter().map(|f| f.mass.stored()).sum::<usize>(),
        activity.scratch_bytes / 1024,
        activity.activated,
        activity.retired,
        authority.error.as_deref().unwrap_or("")
    );
    for mut hud in &mut hud {
        hud.0 = text.clone();
    }
    // Drawing never mutates Authority; CPU submission timing is collected separately.
    if timings.draw_ns.len() < 16384 {
        timings.draw_ns.push(started.elapsed().as_nanos());
    }
}
fn record(
    mut authority: ResMut<Authority>,
    time: Res<Time>,
    timings: Res<RenderTimings>,
    mut commands: Commands,
    mut exit: MessageWriter<AppExit>,
) {
    if authority.record.is_none() {
        return;
    }
    authority.frames_ms.push(time.delta_secs_f64() * 1000.);
    if let Some(error) = &authority.error {
        eprintln!("{error}");
        exit.write(AppExit::error());
        return;
    }
    if authority.world.state().tick < authority.target as u64 {
        return;
    }
    authority.completed_frames += 1;
    let path = authority.record.as_ref().unwrap().clone();
    if authority.completed_frames == 1 {
        if let Err(e) = authority
            .world
            .save_candidate(&path.join("final.json"), authority.lane, 1)
        {
            eprintln!("{e}");
            exit.write(AppExit::error());
            return;
        }
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path.join("chamber.png")));
    }
    if authority.completed_frames == 20 {
        let report = serde_json::json!({"status":"RECORDED","initial_digest":authority.initial_digest,"final_digest":authority.world.digest().unwrap(),"ticks":authority.world.state().tick,"lane":authority.lane,"scenario":authority.scenario,"block_edge":authority.block_edge,"activity":authority.workspace.activity,"physical_digest":authority.world.physical_digest().unwrap(),"step_profiles":authority.profiles,"frame_ms":authority.frames_ms,"draw_cpu_ns":timings.draw_ns,"diagnostics":authority.world.state().diagnostics(),"build_identity":serde_json::from_str::<serde_json::Value>(moebius_morph::BENCHMARK_BUILD_IDENTITY).unwrap(),"scope":"ordinary native observer; frame times include update/render/wait, no GPU timestamps or human mouse acceptance"});
        if let Err(e) = std::fs::write(
            path.join("report.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        ) {
            eprintln!("{e}");
            exit.write(AppExit::error());
            return;
        }
        exit.write(AppExit::Success);
    }
}

fn new_world(
    config: Config,
    scenario: &str,
    lane: Lane,
    edge: usize,
) -> Result<Chamber, moebius_morph::MorphError> {
    let mut world = match scenario {
        "demo" => experiments::demo(config)?,
        "cloud" => scalable::cloud(config, 0.01, 1, 0, 0.0001, 1, 0.002)?,
        "two-clouds" => scalable::cloud(config, 0.01, 2, 0, 0.0001, 1, 0.002)?,
        _ => {
            return Err(moebius_morph::MorphError(
                "Unknown experimental scenario".into(),
            ));
        }
    };
    if lane == Lane::Blocks {
        world.use_blocks(edge)?;
    }
    Ok(world)
}
