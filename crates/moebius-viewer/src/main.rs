//! Recorded snapshot inspection and the live Farm M1 vivarium.
//!
//! The opening slate can change the founding-cell rules before tick 0.
//! Enter on an open run hands the world to a worker thread. Enter on a finite
//! run commits through the headless batch runner, or watches an unchanged
//! recording. Render frames do not call `World::tick` and do not change the
//! rules. Bone, oil, and bile are a viewing costume. Filaments are movement
//! samples. Fluid orbs are sums of the nutrient field. Audio is local PCM.
//! M0 does not divide, mutate, or detect an organism.

#![forbid(unsafe_code)]

mod audio;
mod body;
mod jaw_exercise;
mod layout;
mod live;
mod motor_demo;
mod rules;
mod show;
mod worm_a;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use clap::Parser;
use moebius_core::layers::validate_hypothesis_grid;
use moebius_core::{
    Dims, HypothesisLayer, SimConfig, Site, SnapshotFile, TrailPoint, audit_with_config,
    config_hash, fluid_samples, parse_hypothesis, parse_trails,
};

use layout::display_stride;
use live::Hall;
use rules::Draft;

const PLAY_TICKS_PER_SEC: f32 = 30.0;
const BODY_SCALE: f32 = 0.48;

#[derive(Parser)]
#[command(
    name = "moebius-view",
    version,
    about = "Choose founding rules, then watch. An open run keeps ticking off the render thread."
)]
struct Cli {
    /// Default first creature is the user sculpt. Procedural preserves the old visual.
    #[arg(long, default_value="worm-a", value_parser=["worm-a","procedural"])]
    creature_skin: String,
    /// Follow the moving mouth; Space pauses/resumes.
    #[arg(long)]
    mouth_view: bool,
    /// Diagnostic jaw close/open motor exercise. All other actuators rest; not autonomous.
    #[arg(long, requires = "motor_nursery")]
    jaw_exercise: bool,
    /// Record16 mouth frames and motor-control provenance without pausing.
    #[arg(long)]
    record_mouth: Option<PathBuf>,
    /// Run directory that contains manifest.json and snapshots/.
    /// Omit it to open the 256³ farm.
    #[arg(long)]
    snapshots: Option<PathBuf>,
    /// Skip the rule slate and watch --snapshots immediately.
    #[arg(long)]
    start: bool,
    /// Skip the rule slate and tick the open 256³ farm.
    #[arg(long)]
    live: bool,
    /// Open the read-only neural inspector (N toggles it; Tab changes page).
    #[arg(long)]
    inspect: bool,
    /// Explicit M6 checkpoint bundle; never changes the ordinary nursery default.
    #[arg(long)]
    motor_nursery: Option<PathBuf>,
    /// Native recurrent demo controller: selected, untrained, or compare (IDs1/2).
    #[arg(long,default_value="compare",value_parser=["selected","untrained","compare"])]
    motor_control: String,
    /// Save an actual rendered frame and exit after capture completes.
    #[arg(long)]
    capture_frame: Option<PathBuf>,
    /// Leave the captured viewer open for inspection.
    #[arg(long)]
    capture_keep_open: bool,
    #[arg(long, default_value_t = 128)]
    capture_tick: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, States)]
enum Stage {
    #[default]
    Rules,
    Playing,
}

#[derive(Resource)]
struct Boot {
    auto_start: bool,
}

#[derive(Resource)]
struct Reel {
    seed: u64,
    config_hash: String,
    code_version: String,
    energy_cap: u64,
    site_cap: u64,
    dims: Dims,
    stride: u32,
    frames: Vec<Frame>,
    paths: BTreeMap<u64, Vec<(u64, Site)>>,
    fluid_origins: Vec<Site>,
    fluid_sites: Vec<u32>,
    fluid_amounts: Vec<Vec<f32>>,
    seeds: Vec<(u64, [u8; 8])>,
    trails_present: bool,
    hypothesis: HypothesisLayer,
    hypothesis_from_file: bool,
}

struct Frame {
    tick: u64,
    state_hash: String,
    cells: Vec<LiveCell>,
    nutrient: Vec<u64>,
    live: u32,
    dead: u32,
    field: u64,
    material_flesh: u64,
    in_flight: u64,
    reservoir: u64,
    waste: u64,
    external_input: u64,
    external_output: u64,
    energy_live: u64,
    energy_credited: u64,
    energy_maintenance: u64,
    energy_disposed: u64,
    material_initial: u64,
}

struct LiveCell {
    id: u64,
    site: Site,
    energy: u64,
    genome: [u8; 8],
}

#[derive(Resource)]
struct ViewState {
    tick: f32,
    tick_end: f32,
    playing: bool,
    cells: bool,
    fluid: bool,
    filaments: bool,
    hypothesis: bool,
    hypothesis_drawn: bool,
    muted: bool,
    audio_level: u8,
    yaw: f32,
    pitch: f32,
    distance: f32,
    eye: Option<Vec3>,
    fov: f32,
    body_scale: f32,
    lamp_reach: f32,
    fps: f32,
    audio_for: Option<AudioKey>,
    threads_for: Option<u64>,
    note: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct AudioKey {
    bucket: u64,
    muted: bool,
    level: u8,
}

#[derive(Resource)]
struct Gallery {
    worm: Option<worm_a::WormAssets>,
    creatures: [Handle<Mesh>; 3],
    creature_mats: [Handle<StandardMaterial>; 3],
    threads: Handle<Mesh>,
    bindposes: Handle<bevy::mesh::skinning::SkinnedMeshInverseBindposes>,
    mouth: Handle<Mesh>,
    mouth_mat: Handle<StandardMaterial>,
    flesh_mat: Handle<StandardMaterial>,
    flesh_mesh: Handle<Mesh>,
    mandibles: [Handle<Mesh>; 2],
    orbs: Handle<Mesh>,
    orb_mat: Handle<StandardMaterial>,
    mark: Handle<Mesh>,
    mark_mat: Handle<StandardMaterial>,
}

#[derive(Resource)]
struct CreatureSkin {
    worm_a: bool,
}

#[derive(Resource)]
struct FrameClock {
    frames: u32,
    seconds: f32,
    reported: bool,
}

#[derive(Component)]
struct Creature {
    id: u64,
    phase: f32,
    spin: f32,
    stretch: Vec3,
    facing: Quat,
}

#[derive(Component)]
struct FluidOrb(usize);

#[derive(Component)]
struct Threads;

#[derive(Component)]
struct DriftLight;

#[derive(Component)]
struct Drawn;

#[derive(Component)]
struct Score;

#[derive(Component)]
struct Hud;

fn main() {
    let cli = Cli::parse();
    if cli.start && cli.live {
        eprintln!("moebius-view: pass only one of --start and --live");
        std::process::exit(1);
    }
    if cli.start && cli.snapshots.is_none() {
        eprintln!("moebius-view: --start needs --snapshots");
        std::process::exit(1);
    }
    if cli.live && cli.snapshots.is_some() {
        eprintln!(
            "moebius-view: --live starts the open hall. Omit --snapshots, or pass --start to watch that recording."
        );
        std::process::exit(1);
    }
    if cli.motor_nursery.is_some() && (cli.start || cli.live || cli.snapshots.is_some()) {
        eprintln!("motor demo is separate from --live/--start/--snapshots");
        std::process::exit(1);
    }
    let motor_world = cli
        .motor_nursery
        .as_ref()
        .map(|path| motor_demo::load(path, &cli.motor_control));
    let motor_world = match motor_world {
        Some(Ok(w)) => Some(w),
        Some(Err(e)) => {
            eprintln!("motor demo: {e}");
            std::process::exit(1);
        }
        None => None,
    };
    let motor_enabled = motor_world.is_some();
    let draft = if let Some(world) = &motor_world {
        Draft::from_parts(world.config().clone(), world.seed(), 1000, 50, true)
    } else {
        match &cli.snapshots {
            Some(path) => Draft::from_run_dir(path),
            None => Ok(Draft::hall()),
        }
    };
    let draft = match draft {
        Ok(draft) => draft,
        Err(err) => {
            eprintln!("moebius-view: {err}");
            std::process::exit(1);
        }
    };
    let prepared_hall = if let Some(world) = motor_world {
        match if cli.jaw_exercise {
            live::open_jaw_exercise(world)
        } else {
            live::open_initial(world)
        } {
            Ok(hall) => Some(hall),
            Err(e) => {
                eprintln!("motor demo: {e}");
                std::process::exit(1);
            }
        }
    } else if cli.live {
        match live::open(&draft.config, draft.seed) {
            Ok(hall) => Some(hall),
            Err(err) => {
                eprintln!("moebius-view: {err}");
                std::process::exit(1);
            }
        }
    } else {
        None
    };
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(AssetPlugin { file_path: concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets").into(), ..default() }).set(WindowPlugin {
        primary_window: Some(Window {
            title: if motor_enabled {
                format!(
                    "M6 native recurrent demo — {} — evolved candidate, not lifetime learning",
                    cli.motor_control
                )
            } else {
                "Möbius Automata — Vivarium".into()
            },
            ..default()
        }),
        ..default()
    }))
    .insert_resource(ClearColor(Color::srgb(0.006, 0.008, 0.007)))
    .insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.68, 0.74, 0.70),
        brightness: 450.0,
        ..default()
    })
    .insert_resource(CreatureSkin { worm_a: cli.creature_skin=="worm-a" })
    .insert_resource(worm_a::MouthView { enabled: cli.mouth_view, ..default() })
    .insert_resource(show::FlyCamera::with_inspector(cli.inspect))
    .insert_resource(draft)
    .insert_resource(Boot {
        auto_start: cli.start || cli.live || motor_enabled,
    })
    .init_state::<Stage>()
    .add_systems(Startup, show::boot)
    .add_systems(
        Update,
        (show::rules_keys, show::rules_text)
            .chain()
            .run_if(in_state(Stage::Rules)),
    )
    .add_systems(
        OnEnter(Stage::Playing),
        (
            show::assemble.run_if(resource_exists::<Reel>),
            show::assemble_hall.run_if(resource_exists::<Hall>),
        ),
    )
    .add_systems(
        Update,
        (
            show::orbit,
            show::picture_keys,
            show::playback.run_if(resource_exists::<Reel>),
            show::hall_clock.run_if(resource_exists::<Hall>),
            show::glide_bodies.run_if(resource_exists::<Reel>),
            show::glide_fluid.run_if(resource_exists::<Reel>),
            show::glide_threads.run_if(resource_exists::<Reel>),
            show::hall_pump.run_if(resource_exists::<Hall>),
            show::embody.run_if(resource_exists::<Hall>),
            show::hall_bodies.run_if(resource_exists::<Hall>),
            (
                show::rig_joints,
                worm_a::ready,
                worm_a::sculpt_mouth.after(show::rig_joints),
                show::feeding,
                show::flesh_surfaces,
                show::neural_inspector,
                show::jet_picture,
            )
                .chain()
                .run_if(resource_exists::<Hall>),
            show::hall_fluid.run_if(resource_exists::<Hall>),
            show::drift_lamp,
            show::marks.run_if(resource_exists::<Reel>),
            show::soundtrack.run_if(resource_exists::<Reel>),
            show::hall_sound.run_if(resource_exists::<Hall>),
            show::pace,
            show::hud.run_if(resource_exists::<Reel>),
            show::hall_hud.run_if(resource_exists::<Hall>),
        )
            .chain()
            .run_if(in_state(Stage::Playing)),
    );
    if cli.start {
        let path = cli.snapshots.expect("--start needs --snapshots");
        match load_reel(&path) {
            Ok(reel) => {
                let view = show::view_for(&reel);
                app.insert_resource(reel)
                    .insert_resource(view)
                    .insert_resource(FrameClock {
                        frames: 0,
                        seconds: 0.0,
                        reported: false,
                    });
            }
            Err(err) => {
                eprintln!("moebius-view: {err}");
                std::process::exit(1);
            }
        }
    }
    if let Some(hall) = prepared_hall {
        let view = show::view_for_hall(&hall);
        app.insert_resource(hall)
            .insert_resource(view)
            .insert_resource(FrameClock {
                frames: 0,
                seconds: 0.0,
                reported: false,
            });
    }
    app.add_systems(
        Update,
        worm_a::mouth_view
            .after(show::orbit)
            .after(show::hall_clock)
            .run_if(in_state(Stage::Playing)),
    );
    if let Some(directory) = cli.record_mouth {
        std::fs::create_dir(&directory).expect("mouth recording needs a new directory");
        app.insert_resource(worm_a::MouthRecording {
            directory,
            next_tick: 16,
            frames: 0,
        })
        .add_systems(
            Update,
            worm_a::record_mouth
                .after(worm_a::mouth_view)
                .after(show::rig_joints)
                .after(worm_a::sculpt_mouth)
                .run_if(in_state(Stage::Playing)),
        );
    }
    if let Some(path) = cli.capture_frame {
        if path.exists() {
            eprintln!("capture path exists; refusing overwrite");
            std::process::exit(1);
        }
        app.insert_resource(FrameCapture {
            path,
            tick: cli.capture_tick,
            requested: false,
            exit_after: !cli.capture_keep_open,
        })
        .add_systems(Update, capture_frame.run_if(in_state(Stage::Playing)));
    }
    app.run();
}

fn material_residual(frame: &Frame) -> i128 {
    i128::from(frame.field)
        + i128::from(frame.material_flesh)
        + i128::from(frame.in_flight)
        + i128::from(frame.reservoir)
        + i128::from(frame.waste)
        + i128::from(frame.external_output)
        - i128::from(frame.external_input)
        - i128::from(frame.material_initial)
}

fn energy_residual(frame: &Frame) -> i128 {
    i128::from(frame.energy_live)
        + i128::from(frame.energy_maintenance)
        + i128::from(frame.energy_disposed)
        - i128::from(frame.energy_credited)
}

fn load_reel(dir: &Path) -> Result<Reel, String> {
    let manifest_text = fs::read_to_string(dir.join("manifest.json"))
        .map_err(|err| format!("cannot read manifest.json: {err}"))?;
    let manifest: serde_json::Value = serde_json::from_str(&manifest_text)
        .map_err(|err| format!("manifest JSON rejected: {err}"))?;
    let config: SimConfig = serde_json::from_value(
        manifest
            .get("config")
            .cloned()
            .ok_or_else(|| "manifest is missing config".to_string())?,
    )
    .map_err(|err| format!("manifest config rejected: {err}"))?;
    let seed = manifest
        .get("seed")
        .and_then(|value| value.as_u64())
        .ok_or_else(|| "manifest is missing seed".to_string())?;
    let stated_hash = manifest
        .get("config_hash")
        .and_then(|value| value.as_str())
        .ok_or_else(|| "manifest is missing config_hash".to_string())?
        .to_string();
    let actual_hash = config_hash(&config).map_err(|err| err.to_string())?;
    if stated_hash != actual_hash {
        return Err(format!(
            "manifest config_hash {stated_hash} != computed {actual_hash}"
        ));
    }
    let code_version = manifest
        .get("code_version")
        .and_then(|value| value.as_str())
        .unwrap_or("unknown")
        .to_string();
    let dims = Dims {
        width: config.width,
        height: config.height,
        depth: config.depth,
    };
    let mut paths = Vec::new();
    let snapshot_dir = dir.join("snapshots");
    let entries = fs::read_dir(&snapshot_dir)
        .map_err(|err| format!("cannot read {}: {err}", snapshot_dir.display()))?;
    for entry in entries {
        let path = entry
            .map_err(|err| format!("cannot read snapshot entry: {err}"))?
            .path();
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if name.starts_with("snap_") && name.ends_with(".bin") {
            paths.push(path);
        }
    }
    paths.sort();
    if paths.is_empty() {
        return Err(format!("no snap_*.bin files in {}", snapshot_dir.display()));
    }
    let mut frames = Vec::with_capacity(paths.len());
    for path in &paths {
        let bytes =
            fs::read(path).map_err(|err| format!("cannot read {}: {err}", path.display()))?;
        let file =
            SnapshotFile::decode(&bytes).map_err(|err| format!("{}: {err}", path.display()))?;
        audit_with_config(&config, &file).map_err(|err| format!("{}: {err}", path.display()))?;
        let snapshot = file.snapshot;
        if snapshot.seed != seed {
            return Err(format!(
                "{} seed {} != manifest seed {seed}",
                path.display(),
                snapshot.seed
            ));
        }
        if snapshot.width != dims.width
            || snapshot.height != dims.height
            || snapshot.depth != dims.depth
        {
            return Err(format!(
                "{} grid does not match the manifest config",
                path.display()
            ));
        }
        let field = snapshot
            .nutrient
            .iter()
            .try_fold(0u64, |acc, value| acc.checked_add(*value))
            .ok_or_else(|| format!("{} nutrient sum overflowed", path.display()))?;
        let energy_live = snapshot
            .cells
            .iter()
            .try_fold(0u64, |acc, cell| acc.checked_add(cell.energy))
            .ok_or_else(|| format!("{} energy sum overflowed", path.display()))?;
        frames.push(Frame {
            tick: snapshot.tick,
            state_hash: snapshot.state_hash(),
            cells: {
                let mut cells: Vec<LiveCell> = snapshot
                    .cells
                    .iter()
                    .map(|cell| LiveCell {
                        id: cell.id,
                        site: cell.site,
                        energy: cell.energy,
                        genome: cell.genome,
                    })
                    .collect();
                cells.sort_by_key(|cell| cell.id);
                cells
            },
            nutrient: snapshot.nutrient,
            live: u32::try_from(snapshot.cells.len())
                .map_err(|_| format!("{} live count exceeds u32", path.display()))?,
            dead: u32::try_from(snapshot.deaths.len())
                .map_err(|_| format!("{} death count exceeds u32", path.display()))?,
            field,
            material_flesh: snapshot.flesh.iter().map(|deposit| deposit.amount).sum(),
            in_flight: snapshot.jets.len() as u64,
            reservoir: snapshot.reservoir,
            waste: snapshot.waste,
            external_input: snapshot.external_input,
            external_output: snapshot.external_output,
            energy_live,
            energy_credited: snapshot.energy_credited,
            energy_maintenance: snapshot.energy_maintenance,
            energy_disposed: snapshot.energy_disposed,
            material_initial: snapshot.material_initial,
        });
    }
    frames.sort_by_key(|frame| frame.tick);
    for pair in frames.windows(2) {
        if pair[0].tick == pair[1].tick {
            return Err(format!("two snapshots share tick {}", pair[0].tick));
        }
    }
    let trail_path = dir.join("trails.csv");
    let (trails, trails_present) = if trail_path.exists() {
        let text = fs::read_to_string(&trail_path)
            .map_err(|err| format!("cannot read trails.csv: {err}"))?;
        let trails = parse_trails(&text).map_err(|err| err.to_string())?;
        (trails, true)
    } else {
        (Vec::new(), false)
    };
    let hypothesis_path = dir.join("hypothesis.json");
    let (hypothesis, hypothesis_from_file) = if hypothesis_path.exists() {
        let text = fs::read_to_string(&hypothesis_path)
            .map_err(|err| format!("cannot read hypothesis.json: {err}"))?;
        let layer = parse_hypothesis(&text).map_err(|err| err.to_string())?;
        validate_hypothesis_grid(&layer, dims).map_err(|err| err.to_string())?;
        (layer, true)
    } else {
        (
            HypothesisLayer::scaffold("organism").map_err(|err| err.to_string())?,
            false,
        )
    };
    let stride = display_stride(dims);
    let fluid = cache_fluid(&frames, dims, stride)?;
    let motion = index_paths(&frames, &trails);
    let seeds = costume_seeds(&frames);
    Ok(Reel {
        seed,
        config_hash: actual_hash,
        code_version,
        energy_cap: config.energy_cap,
        site_cap: config.site_cap,
        dims,
        stride,
        frames,
        paths: motion,
        fluid_origins: fluid.origins,
        fluid_sites: fluid.sites,
        fluid_amounts: fluid.amounts,
        seeds,
        trails_present,
        hypothesis,
        hypothesis_from_file,
    })
}

struct FluidCache {
    origins: Vec<Site>,
    sites: Vec<u32>,
    amounts: Vec<Vec<f32>>,
}

fn cache_fluid(frames: &[Frame], dims: Dims, stride: u32) -> Result<FluidCache, String> {
    let mut origins = Vec::new();
    let mut sites = Vec::new();
    let mut amounts: Vec<Vec<f32>> = Vec::new();
    for (frame_index, frame) in frames.iter().enumerate() {
        let samples = fluid_samples(&frame.nutrient, dims, stride)
            .map_err(|err| format!("tick {} fluid downsample rejected: {err}", frame.tick))?;
        if frame_index == 0 {
            origins = samples.iter().map(|sample| sample.origin).collect();
            sites = samples.iter().map(|sample| sample.sites).collect();
            amounts = vec![Vec::with_capacity(frames.len()); samples.len()];
        } else if samples.len() != origins.len() {
            return Err(format!(
                "tick {} fluid block count {} != {}",
                frame.tick,
                samples.len(),
                origins.len()
            ));
        }
        for (index, sample) in samples.iter().enumerate() {
            if sample.origin != origins[index] || sample.sites != sites[index] {
                return Err(format!(
                    "tick {} fluid block {index} does not match the first snapshot",
                    frame.tick
                ));
            }
            amounts[index].push(sample.amount as f32);
        }
    }
    Ok(FluidCache {
        origins,
        sites,
        amounts,
    })
}

fn index_paths(frames: &[Frame], trails: &[TrailPoint]) -> BTreeMap<u64, Vec<(u64, Site)>> {
    let mut paths: BTreeMap<u64, Vec<(u64, Site)>> = BTreeMap::new();
    if trails.is_empty() {
        for frame in frames {
            for cell in &frame.cells {
                paths
                    .entry(cell.id)
                    .or_default()
                    .push((frame.tick, cell.site));
            }
        }
    } else {
        for point in trails {
            paths
                .entry(point.id)
                .or_default()
                .push((point.tick, point.site()));
        }
    }
    for points in paths.values_mut() {
        points.sort_by_key(|(tick, site)| (*tick, *site));
        points.dedup_by_key(|(tick, _)| *tick);
    }
    paths
}

fn costume_seeds(frames: &[Frame]) -> Vec<(u64, [u8; 8])> {
    let mut seeds = BTreeMap::new();
    for frame in frames {
        for cell in &frame.cells {
            seeds.entry(cell.id).or_insert(cell.genome);
        }
    }
    seeds.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use moebius_core::{
        GenomeFill, SnapshotFile, World, build_manifest, format_trails, manifest_json,
    };

    fn tiny() -> SimConfig {
        SimConfig {
            width: 4,
            height: 4,
            depth: 4,
            initial_population: 4,
            initial_energy: 12,
            initial_nutrient_per_site: 3,
            nutrient_wave_amplitude: 0,
            initial_reservoir: 40,
            initial_waste: 0,
            inflow_per_tick: 1,
            inject_per_tick: 1,
            uptake_per_tick: 1,
            energy_per_nutrient: 2,
            maintenance_cost: 1,
            export_per_tick: 1,
            site_cap: 20,
            reservoir_cap: 10_000,
            energy_cap: 1_000,
            genome_fill: GenomeFill::Zero,
            farm: None,
            neural: None,
            feeding: None,
        }
    }

    #[test]
    fn reel_reads_committed_files_and_does_not_need_a_window() {
        let dir = std::env::temp_dir().join(format!("moebius-view-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("snapshots")).unwrap();
        let config = tiny();
        let mut world = World::genesis(&config, 9).unwrap();
        let mut trails = world.trail_points();
        fs::write(
            dir.join("snapshots/snap_0000000000.bin"),
            SnapshotFile::encode(&config, &world).unwrap(),
        )
        .unwrap();
        for _ in 0..3 {
            world.tick().unwrap();
            trails.extend(world.trail_points());
        }
        fs::write(
            dir.join(format!("snapshots/snap_{:010}.bin", world.tick_index())),
            SnapshotFile::encode(&config, &world).unwrap(),
        )
        .unwrap();
        fs::write(dir.join("trails.csv"), format_trails(&trails)).unwrap();
        let manifest = build_manifest("test", &config, 9, 3, 3, &[0, 3], &world).unwrap();
        fs::write(dir.join("manifest.json"), manifest_json(&manifest).unwrap()).unwrap();
        fs::write(
            dir.join("hypothesis.json"),
            r#"{"name":"organism","note":"hand annotation for the viewer test","sites":[[0,0,0,1]]}"#,
        )
        .unwrap();

        let reel = load_reel(&dir).unwrap();
        assert_eq!(reel.frames.len(), 2);
        assert_eq!(reel.frames[0].tick, 0);
        assert_eq!(reel.frames[1].tick, 3);
        assert_eq!(reel.frames[1].state_hash, manifest.final_state_hash);
        assert_eq!(material_residual(&reel.frames[1]), 0);
        assert_eq!(energy_residual(&reel.frames[1]), 0);
        assert!(reel.trails_present);
        assert!(reel.hypothesis_from_file);
        assert_eq!(reel.hypothesis.marks.len(), 1);
        assert!(reel.hypothesis.note.contains("annotation"));
        let _ = fs::remove_dir_all(&dir);
    }
    #[test]
    fn keep_open_capture_does_not_pause_live_motion() {
        let config = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../experiments/motor_land_demo.json");
        let mut world = motor_demo::load(&config, "selected").unwrap();
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
        let dir = std::env::temp_dir().join(format!("moebius-keep-open-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("capture.png");
        let mut app = App::new();
        app.insert_resource(hall)
            .insert_resource(FrameCapture {
                path: path.clone(),
                tick: 16,
                requested: false,
                exit_after: false,
            })
            .add_systems(Update, capture_frame);
        app.update();
        assert!(app.world().resource::<FrameCapture>().requested);
        assert!(
            !app.world().resource::<Hall>().is_paused(),
            "keeping a capture open must leave live jaw movement running"
        );
        assert!(path.with_extension("capture.json").exists());
        std::fs::remove_dir_all(dir).unwrap();
    }
}

#[derive(Resource)]
struct FrameCapture {
    exit_after: bool,
    path: PathBuf,
    tick: u64,
    requested: bool,
}
fn capture_frame(
    mut request: ResMut<FrameCapture>,
    hall: Option<Res<Hall>>,
    mut commands: Commands,
) {
    use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk};
    let Some(hall) = hall else {
        return;
    };
    if request.requested || hall.committed_tick() < request.tick {
        return;
    }
    request.requested = true;
    if request.exit_after {
        hall.set_paused(true);
    }
    if let Some(picture) = &hall.latest {
        let record = serde_json::json!({"committed_tick_at_request":picture.tick,"mode":"native recurrent motor demo","state_hash":picture.state_hash,"land_units":picture.material_flesh,"reservoir":picture.reservoir,"config_hash":hall.config_hash,"states":picture.cells.iter().map(|cell|serde_json::json!({"id":cell.id,"site":cell.site,"genome_id":cell.rig.as_ref().and_then(|b|b.neural.as_ref()).map(|n|n.genome.id())})).collect::<Vec<_>>()});
        let _ = std::fs::write(
            request.path.with_extension("capture.json"),
            serde_json::to_vec_pretty(&record).unwrap(),
        );
    }
    let mut shot = commands.spawn(Screenshot::primary_window());
    shot.observe(save_to_disk(request.path.clone()));
    if request.exit_after {
        shot.observe(
            |_: On<ScreenshotCaptured>, mut exit: MessageWriter<bevy::app::AppExit>| {
                exit.write(bevy::app::AppExit::Success);
            },
        );
    }
}
