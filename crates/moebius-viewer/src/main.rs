//! Read-only 3D playback of committed M0 snapshots.
//!
//! Render frames select a snapshot. They do not call `World::tick` and they
//! do not write hypothesis marks back into the world. Filaments are movement
//! samples. Fluid blocks are sums of the nutrient field. Audio is local PCM.

#![forbid(unsafe_code)]

mod audio;
mod layout;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use bevy::audio::Volume;
use bevy::prelude::*;
use bevy::text::FontSize;
use clap::Parser;
use moebius_core::layers::validate_hypothesis_grid;
use moebius_core::{
    Dims, HypothesisLayer, SimConfig, Site, SnapshotFile, TrailPoint, audit_with_config,
    config_hash, fluid_samples, parse_hypothesis, parse_trails,
};

use layout::{FILAMENT_WINDOW, display_stride, filament_segments, site_to_view};

#[derive(Parser)]
#[command(
    name = "moebius-view",
    version,
    about = "Play committed Möbius Automata M0 snapshots. This program does not advance the simulation."
)]
struct Cli {
    /// Run directory that contains manifest.json and snapshots/.
    #[arg(long)]
    snapshots: PathBuf,
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
    trails: Vec<TrailPoint>,
    trails_present: bool,
    hypothesis: HypothesisLayer,
    hypothesis_from_file: bool,
}

struct Frame {
    tick: u64,
    state_hash: String,
    cells: Vec<(u64, Site, u64)>,
    nutrient: Vec<u64>,
    live: u32,
    dead: u32,
    field: u64,
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

#[derive(Resource)]
struct ViewState {
    index: usize,
    playing: bool,
    accumulator: f32,
    cells: bool,
    fluid: bool,
    filaments: bool,
    hypothesis: bool,
    muted: bool,
    yaw: f32,
    pitch: f32,
    distance: f32,
    drawn_for: Option<DrawKey>,
    audio_for: Option<AudioKey>,
    note: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct DrawKey {
    index: usize,
    cells: bool,
    fluid: bool,
    filaments: bool,
    hypothesis: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct AudioKey {
    index: usize,
    muted: bool,
}

#[derive(Resource)]
struct Shapes {
    cell: Handle<Mesh>,
    block: Handle<Mesh>,
    filament: Handle<Mesh>,
    mark: Handle<Mesh>,
}

#[derive(Component)]
struct Drawn;

#[derive(Component)]
struct Score;

#[derive(Component)]
struct Hud;

fn main() {
    let cli = Cli::parse();
    let reel = match load_reel(&cli.snapshots) {
        Ok(reel) => reel,
        Err(err) => {
            eprintln!("moebius-view: {err}");
            std::process::exit(1);
        }
    };
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Möbius Automata M0".into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.012, 0.015, 0.02)))
        .insert_resource(reel)
        .insert_resource(ViewState {
            index: 0,
            playing: false,
            accumulator: 0.0,
            cells: true,
            fluid: true,
            filaments: true,
            hypothesis: false,
            muted: false,
            yaw: 0.7,
            pitch: 0.45,
            distance: 14.0,
            drawn_for: None,
            audio_for: None,
            note: String::new(),
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (orbit, playback, redraw, soundtrack, hud).chain())
        .run();
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.insert_resource(Shapes {
        cell: meshes.add(Sphere::new(1.0).mesh().ico(2).unwrap()),
        block: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        filament: meshes.add(Cuboid::new(0.012, 1.0, 0.012)),
        mark: meshes.add(Sphere::new(0.05).mesh().ico(2).unwrap()),
    });
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 4.0, 14.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        PointLight {
            intensity: 250_000.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(6.0, 10.0, 4.0),
    ));
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(15.0),
            ..default()
        },
        TextColor(Color::srgb(0.78, 0.9, 0.78)),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        },
        Hud,
    ));
}

fn orbit(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut view: ResMut<ViewState>,
    mut camera: Query<&mut Transform, With<Camera3d>>,
) {
    let dt = time.delta_secs();
    view.yaw += dt * 0.12;
    if keyboard.pressed(KeyCode::KeyJ) {
        view.yaw -= dt * 1.4;
    }
    if keyboard.pressed(KeyCode::KeyL) {
        view.yaw += dt * 1.4;
    }
    if keyboard.pressed(KeyCode::KeyI) {
        view.pitch += dt * 1.1;
    }
    if keyboard.pressed(KeyCode::KeyK) {
        view.pitch -= dt * 1.1;
    }
    if keyboard.pressed(KeyCode::Minus) {
        view.distance += dt * 6.0;
    }
    if keyboard.pressed(KeyCode::Equal) {
        view.distance -= dt * 6.0;
    }
    view.pitch = view.pitch.clamp(-1.15, 1.15);
    view.distance = view.distance.clamp(2.5, 48.0);
    let Ok(mut transform) = camera.single_mut() else {
        return;
    };
    let (sy, cy) = view.yaw.sin_cos();
    let (sp, cp) = view.pitch.sin_cos();
    let eye = Vec3::new(sy * cp, sp, cy * cp) * view.distance;
    transform.translation = eye;
    transform.look_at(Vec3::ZERO, Vec3::Y);
}

fn playback(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    reel: Res<Reel>,
    mut view: ResMut<ViewState>,
) {
    if keyboard.just_pressed(KeyCode::Digit1) {
        view.cells = !view.cells;
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        view.fluid = !view.fluid;
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        view.filaments = !view.filaments;
    }
    if keyboard.just_pressed(KeyCode::Digit4) {
        view.hypothesis = !view.hypothesis;
    }
    if keyboard.just_pressed(KeyCode::KeyM) {
        view.muted = !view.muted;
    }
    if keyboard.just_pressed(KeyCode::Space) {
        view.playing = !view.playing;
    }
    let last = reel.frames.len().saturating_sub(1);
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        view.playing = false;
        view.index = view.index.saturating_sub(1);
    }
    if keyboard.just_pressed(KeyCode::ArrowRight) {
        view.playing = false;
        view.index = (view.index + 1).min(last);
    }
    if keyboard.just_pressed(KeyCode::Home) {
        view.playing = false;
        view.index = 0;
    }
    if keyboard.just_pressed(KeyCode::End) {
        view.playing = false;
        view.index = last;
    }
    if view.playing {
        view.accumulator += time.delta_secs();
        if view.accumulator >= 0.45 {
            view.accumulator = 0.0;
            if view.index >= last {
                view.playing = false;
            } else {
                view.index += 1;
            }
        }
    }
}

fn redraw(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    reel: Res<Reel>,
    shapes: Res<Shapes>,
    mut view: ResMut<ViewState>,
    drawn: Query<Entity, With<Drawn>>,
) {
    let key = DrawKey {
        index: view.index,
        cells: view.cells,
        fluid: view.fluid,
        filaments: view.filaments,
        hypothesis: view.hypothesis,
    };
    if view.drawn_for == Some(key) {
        return;
    }
    for entity in &drawn {
        commands.entity(entity).despawn();
    }
    view.note.clear();
    let Some(frame) = reel.frames.get(view.index) else {
        view.drawn_for = Some(key);
        return;
    };
    if view.cells {
        for (_id, site, energy) in &frame.cells {
            let place = Vec3::from_array(site_to_view(*site, reel.dims));
            let fraction = energy_fraction(*energy, reel.energy_cap);
            let radius = 0.035 + 0.07 * fraction;
            let material = materials.add(StandardMaterial {
                base_color: energy_color(fraction),
                unlit: true,
                ..default()
            });
            commands.spawn((
                Mesh3d(shapes.cell.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(place).with_scale(Vec3::splat(radius)),
                Drawn,
            ));
        }
    }
    if view.fluid {
        match fluid_samples(&frame.nutrient, reel.dims, reel.stride) {
            Ok(samples) => {
                let mut drawn_blocks = 0u32;
                for sample in &samples {
                    if sample.amount == 0 {
                        continue;
                    }
                    let (center, size) = block_pose(sample.origin, reel.stride, reel.dims);
                    let denom = (reel.site_cap as f32) * (sample.sites.max(1) as f32);
                    let fraction = if denom > 0.0 {
                        (sample.amount as f32 / denom).clamp(0.0, 1.0)
                    } else {
                        0.0
                    };
                    let material = materials.add(StandardMaterial {
                        base_color: Color::srgba(
                            0.15,
                            0.55 + 0.35 * fraction,
                            0.62,
                            0.08 + 0.45 * fraction,
                        ),
                        alpha_mode: AlphaMode::Add,
                        unlit: true,
                        ..default()
                    });
                    commands.spawn((
                        Mesh3d(shapes.block.clone()),
                        MeshMaterial3d(material),
                        Transform::from_translation(center).with_scale(size),
                        Drawn,
                    ));
                    drawn_blocks += 1;
                }
                view.note = format!(
                    "fluid stride {} draws {drawn_blocks} nonempty block sums; the sums are a downsample, not a new stock",
                    reel.stride
                );
            }
            Err(err) => {
                view.note = format!("fluid layer withheld: {err}");
            }
        }
    }
    if view.filaments {
        if reel.trails_present {
            let segments = filament_segments(&reel.trails, frame.tick, FILAMENT_WINDOW, reel.dims);
            let material = materials.add(StandardMaterial {
                base_color: Color::srgb(0.62, 0.7, 0.5),
                unlit: true,
                ..default()
            });
            for (start, end) in segments {
                let a = Vec3::from_array(start);
                let b = Vec3::from_array(end);
                let delta = b - a;
                let length = delta.length();
                if length < 1.0e-4 {
                    continue;
                }
                let rotation = Quat::from_rotation_arc(Vec3::Y, delta / length);
                commands.spawn((
                    Mesh3d(shapes.filament.clone()),
                    MeshMaterial3d(material.clone()),
                    Transform {
                        translation: (a + b) * 0.5,
                        rotation,
                        scale: Vec3::new(1.0, length, 1.0),
                    },
                    Drawn,
                ));
            }
        } else if !view.note.is_empty() {
            view.note
                .push_str(" | trails.csv is absent, so filaments have no samples");
        } else {
            view.note = "trails.csv is absent, so filaments have no samples".into();
        }
    }
    if view.hypothesis {
        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.95, 0.22, 0.55),
            unlit: true,
            ..default()
        });
        for mark in &reel.hypothesis.marks {
            let place = Vec3::from_array(site_to_view(mark.site, reel.dims));
            commands.spawn((
                Mesh3d(shapes.mark.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(place),
                Drawn,
            ));
        }
    }
    view.drawn_for = Some(key);
}

fn soundtrack(
    mut commands: Commands,
    mut sources: ResMut<Assets<AudioSource>>,
    reel: Res<Reel>,
    mut view: ResMut<ViewState>,
    existing: Query<Entity, With<Score>>,
) {
    let key = AudioKey {
        index: view.index,
        muted: view.muted,
    };
    if view.audio_for == Some(key) {
        return;
    }
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    if !view.muted
        && let Some(frame) = reel.frames.get(view.index)
    {
        let bytes = audio::synth_wav(frame.tick, frame.live, frame.dead, frame.field, frame.waste);
        let handle = sources.add(AudioSource {
            bytes: Arc::<[u8]>::from(bytes),
        });
        commands.spawn((
            AudioPlayer::new(handle),
            PlaybackSettings::LOOP.with_volume(Volume::Linear(0.22)),
            Score,
        ));
    }
    view.audio_for = Some(key);
}

fn hud(reel: Res<Reel>, view: Res<ViewState>, mut text: Query<&mut Text, With<Hud>>) {
    let Ok(mut text) = text.single_mut() else {
        return;
    };
    let Some(frame) = reel.frames.get(view.index) else {
        text.0 = "no committed frame".into();
        return;
    };
    let play = if view.playing { "playing" } else { "paused" };
    let sound = if view.muted { "muted" } else { "on" };
    let hypothesis_source = if reel.hypothesis_from_file {
        "file"
    } else {
        "scaffold"
    };
    let hash: String = frame.state_hash.chars().take(16).collect();
    let config: String = reel.config_hash.chars().take(16).collect();
    text.0 = format!(
        "Möbius Automata M0 — committed snapshots only\n\
         tick {}   frame {}/{}   {play}\n\
         seed {}   config {config}   code {}\n\
         live {}   dead {}   field {}   reservoir {}   waste {}\n\
         material residual {}   action-energy residual {}\n\
         state {hash}\n\
         layers  1 cells {}   2 fluid {}   3 filaments {}   4 hypothesis {} ({hypothesis_source}, off until pressed)\n\
         hypothesis \"{}\": {} Marks are not organism membership and are not in the state hash.\n\
         filaments: last {FILAMENT_WINDOW} committed steps, split on wraps. Not bonds.\n\
         {}\n\
         audio {sound}: viewer f32 PCM, not in the state hash, does not advance ticks\n\
         space play/pause   arrows step   home/end   J/L yaw   I/K pitch   -/= zoom   M mute",
        frame.tick,
        view.index + 1,
        reel.frames.len(),
        reel.seed,
        reel.code_version,
        frame.live,
        frame.dead,
        frame.field,
        frame.reservoir,
        frame.waste,
        material_residual(frame),
        energy_residual(frame),
        on_off(view.cells),
        on_off(view.fluid),
        on_off(view.filaments),
        on_off(view.hypothesis),
        reel.hypothesis.name,
        reel.hypothesis.note,
        if view.note.is_empty() {
            "fluid hidden".to_string()
        } else {
            view.note.clone()
        },
    );
}

fn on_off(enabled: bool) -> &'static str {
    if enabled { "on" } else { "off" }
}

fn energy_fraction(energy: u64, cap: u64) -> f32 {
    if cap == 0 {
        0.0
    } else {
        (energy as f32 / cap as f32).clamp(0.0, 1.0)
    }
}

fn energy_color(fraction: f32) -> Color {
    Color::srgb(
        0.05 + 0.9 * fraction,
        0.16 + 0.7 * fraction,
        0.1 + 0.18 * fraction,
    )
}

fn block_pose(origin: Site, stride: u32, dims: Dims) -> (Vec3, Vec3) {
    let sx = stride.min(dims.width - origin.x).max(1);
    let sy = stride.min(dims.height - origin.y).max(1);
    let sz = stride.min(dims.depth - origin.z).max(1);
    let near = Vec3::from_array(site_to_view(origin, dims));
    let far = Vec3::from_array(site_to_view(
        Site {
            x: origin.x + sx - 1,
            y: origin.y + sy - 1,
            z: origin.z + sz - 1,
        },
        dims,
    ));
    let center = (near + far) * 0.5;
    let size = (far - near).abs() + Vec3::splat(layout::SPACING * 0.72);
    (center, size)
}

fn material_residual(frame: &Frame) -> i128 {
    i128::from(frame.field)
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
            cells: snapshot
                .cells
                .iter()
                .map(|cell| (cell.id, cell.site, cell.energy))
                .collect(),
            nutrient: snapshot.nutrient,
            live: u32::try_from(snapshot.cells.len())
                .map_err(|_| format!("{} live count exceeds u32", path.display()))?,
            dead: u32::try_from(snapshot.deaths.len())
                .map_err(|_| format!("{} death count exceeds u32", path.display()))?,
            field,
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
    Ok(Reel {
        seed,
        config_hash: actual_hash,
        code_version,
        energy_cap: config.energy_cap,
        site_cap: config.site_cap,
        dims,
        stride: display_stride(dims),
        frames,
        trails,
        trails_present,
        hypothesis,
        hypothesis_from_file,
    })
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
}
