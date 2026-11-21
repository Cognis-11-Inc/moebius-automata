//! Rule slate, the committed-run picture, and the open hall.
//! These systems do not call `World::tick`. The hall worker does.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use bevy::audio::Volume;
use bevy::prelude::*;
use bevy::text::FontSize;

use super::{
    AudioKey, BODY_SCALE, Boot, Creature, Dims, Drawn, DriftLight, FluidOrb, Frame, FrameClock,
    Gallery, Hud, PLAY_TICKS_PER_SEC, Reel, Score, Site, Stage, Threads, ViewState, load_reel,
};
use crate::body::{bone_cage, creature_mesh, write_threads};
use crate::layout::{self, CAMERA_FOV, FILAMENT_WINDOW, path_segments, position_at, site_to_view};
use crate::live::{self, Founder, Hall, LIVE_TICKS_PER_SEC};
use crate::rules::{self, Draft};

const FLUID_HIDE: f32 = 0.08;
const HALL_FOV: f32 = 64.0 * std::f32::consts::PI / 180.0;

pub fn boot(mut commands: Commands, boot: Res<Boot>, mut next: ResMut<NextState<Stage>>) {
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: CAMERA_FOV,
            ..default()
        }),
        Msaa::Sample2,
        Transform::from_xyz(0.0, 0.4, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(15.0),
            ..default()
        },
        TextColor(Color::srgb(0.78, 0.74, 0.64)),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(14),
            ..default()
        },
        Hud,
    ));
    if boot.auto_start {
        next.set(Stage::Playing);
    }
}

pub fn rules_keys(
    mut commands: Commands,
    mut draft: ResMut<Draft>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next: ResMut<NextState<Stage>>,
) {
    if draft.pending {
        let unchanged = draft.unchanged();
        let open_run = draft.until_stopped && !unchanged;
        let outcome = if unchanged {
            draft
                .recording
                .clone()
                .ok_or_else(|| "recording path is missing".to_string())
                .and_then(|path| load_reel(&path))
                .map(Started::Reel)
        } else if open_run {
            live::open(&draft.config, draft.seed).map(Started::Hall)
        } else {
            commit_new(&draft)
                .and_then(|path| load_reel(&path))
                .map(Started::Reel)
        };
        draft.pending = false;
        match outcome {
            Ok(Started::Reel(reel)) => {
                install_playback(&mut commands, reel);
                next.set(Stage::Playing);
            }
            Ok(Started::Hall(hall)) => {
                install_hall(&mut commands, hall);
                next.set(Stage::Playing);
            }
            Err(err) => {
                draft.status = format!("refused: {err}");
            }
        }
        return;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        rules::move_selection(&mut draft, true);
    }
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        rules::move_selection(&mut draft, false);
    }
    let large = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    if keyboard.just_pressed(KeyCode::ArrowRight) {
        rules::adjust(&mut draft, large, true);
    }
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        rules::adjust(&mut draft, large, false);
    }
    let start =
        keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::NumpadEnter);
    if start {
        draft.pending = true;
        draft.status = if draft.unchanged() {
            "Opening the committed recording.".into()
        } else if draft.until_stopped {
            "Locking the founding rules. The open run starts at tick 0.".into()
        } else {
            "Committing these rules from tick 0. The picture waits.".into()
        };
    }
}

pub fn rules_text(draft: Res<Draft>, mut text: Query<&mut Text, With<Hud>>) {
    let Ok(mut text) = text.single_mut() else {
        return;
    };
    text.0 = rules::slate(&draft);
}

pub fn assemble(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    reel: Res<Reel>,
) {
    let span = (reel
        .dims
        .width
        .max(reel.dims.height)
        .max(reel.dims.depth)
        .saturating_sub(1) as f32)
        * layout::SPACING;
    let extent = Vec3::splat(span) + Vec3::splat(layout::SPACING * 2.2);
    let cage = meshes.add(bone_cage(extent, extent.max_element() * 0.012));
    let cage_mat = materials.add(oily(
        Color::srgb(0.16, 0.17, 0.15),
        LinearRgba::new(0.0, 0.01, 0.008, 1.0),
        0.88,
        0.34,
        0.4,
    ));
    commands.spawn((Mesh3d(cage), MeshMaterial3d(cage_mat), Transform::default()));
    let creatures = [
        meshes.add(creature_mesh(0)),
        meshes.add(creature_mesh(1)),
        meshes.add(creature_mesh(2)),
    ];
    let creature_mats = [
        materials.add(oily(
            Color::srgb(0.52, 0.48, 0.4),
            LinearRgba::new(0.02, 0.015, 0.008, 1.0),
            0.58,
            0.26,
            0.74,
        )),
        materials.add(oily(
            Color::srgb(0.06, 0.07, 0.055),
            LinearRgba::new(0.004, 0.014, 0.009, 1.0),
            0.94,
            0.08,
            1.0,
        )),
        materials.add(oily(
            Color::srgb(0.24, 0.16, 0.13),
            LinearRgba::new(0.035, 0.008, 0.004, 1.0),
            0.78,
            0.18,
            0.58,
        )),
    ];
    let orb = meshes.add(Sphere::new(1.0).mesh().ico(3).expect("icosphere"));
    let orb_mat = materials.add(bile_material());
    let empty = meshes.add(creature_mesh(0));
    if let Some(mut mesh) = meshes.get_mut(&empty) {
        write_threads(&mut mesh, &[], 0.01);
    }
    let threads = empty;
    let thread_mat = materials.add(oily(
        Color::srgb(0.45, 0.42, 0.32),
        LinearRgba::new(0.02, 0.016, 0.006, 1.0),
        0.7,
        0.22,
        0.65,
    ));
    let mark = meshes.add(Sphere::new(0.08).mesh().ico(1).expect("mark sphere"));
    let mark_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.95, 0.22, 0.55),
        unlit: true,
        ..default()
    });
    let gallery = Gallery {
        creatures,
        creature_mats,
        threads: threads.clone(),
        orbs: orb.clone(),
        orb_mat: orb_mat.clone(),
        mark: mark.clone(),
        mark_mat: mark_mat.clone(),
    };
    for (id, genome) in &reel.seeds {
        let costume = costume(*id, *genome);
        commands.spawn((
            Mesh3d(gallery.creatures[costume.variant as usize].clone()),
            MeshMaterial3d(gallery.creature_mats[costume.variant as usize].clone()),
            Transform::default(),
            Visibility::Hidden,
            costume.body,
        ));
    }
    for index in 0..reel.fluid_origins.len() {
        commands.spawn((
            Mesh3d(orb.clone()),
            MeshMaterial3d(orb_mat.clone()),
            Transform::default(),
            Visibility::Hidden,
            FluidOrb(index),
        ));
    }
    commands.spawn((
        Mesh3d(threads),
        MeshMaterial3d(thread_mat),
        Transform::default(),
        Visibility::Hidden,
        Threads,
    ));
    commands.insert_resource(gallery);
    let reach = span.max(layout::SPACING);
    commands.spawn(lamp(
        Color::srgb(1.0, 0.78, 0.55),
        2_200_000.0,
        Vec3::new(reach * 0.42, reach * 0.62, reach * 0.38),
    ));
    commands.spawn(lamp(
        Color::srgb(0.22, 0.62, 0.55),
        1_700_000.0,
        Vec3::new(-reach * 0.48, reach * 0.12, -reach * 0.36),
    ));
    commands.spawn((
        lamp(
            Color::srgb(0.86, 0.28, 0.16),
            900_000.0,
            Vec3::new(0.0, -reach * 0.2, reach * 0.2),
        ),
        DriftLight,
    ));
    eprintln!(
        "VIEW_READY creatures {} orbs {}",
        reel.seeds.len(),
        reel.fluid_origins.len()
    );
}

pub fn orbit(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut view: ResMut<ViewState>,
    mut camera: Query<&mut Transform, With<Camera3d>>,
) {
    let dt = time.delta_secs();
    view.yaw += dt * 0.08;
    if keyboard.pressed(KeyCode::KeyJ) {
        view.yaw -= dt * 1.3;
    }
    if keyboard.pressed(KeyCode::KeyL) {
        view.yaw += dt * 1.3;
    }
    if keyboard.pressed(KeyCode::KeyI) {
        view.pitch += dt * 1.0;
    }
    if keyboard.pressed(KeyCode::KeyK) {
        view.pitch -= dt * 1.0;
    }
    if keyboard.pressed(KeyCode::Minus) {
        view.distance += dt * view.distance * 0.45;
    }
    if keyboard.pressed(KeyCode::Equal) {
        view.distance -= dt * view.distance * 0.45;
    }
    view.pitch = view.pitch.clamp(-1.05, 1.05);
    view.distance = view.distance.clamp(view.distance_min, view.distance_max);
    let Ok(mut transform) = camera.single_mut() else {
        return;
    };
    let (sy, cy) = view.yaw.sin_cos();
    let (sp, cp) = view.pitch.sin_cos();
    let eye = Vec3::new(sy * cp, sp, cy * cp) * view.distance;
    transform.translation = eye;
    transform.look_at(Vec3::ZERO, Vec3::Y);
}

pub fn picture_keys(keyboard: Res<ButtonInput<KeyCode>>, mut view: ResMut<ViewState>) {
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
}

pub fn playback(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    reel: Res<Reel>,
    mut view: ResMut<ViewState>,
) {
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        view.playing = false;
        view.tick = neighbor_tick(&reel.frames, view.tick, false);
    }
    if keyboard.just_pressed(KeyCode::ArrowRight) {
        view.playing = false;
        view.tick = neighbor_tick(&reel.frames, view.tick, true);
    }
    if keyboard.just_pressed(KeyCode::Home) {
        view.playing = false;
        view.tick = 0.0;
    }
    if keyboard.just_pressed(KeyCode::End) {
        view.playing = false;
        view.tick = view.tick_end;
    }
    if keyboard.just_pressed(KeyCode::Space) {
        if view.tick >= view.tick_end - 0.001 {
            view.tick = 0.0;
            view.playing = true;
        } else {
            view.playing = !view.playing;
        }
    }
    if view.playing {
        let next = view.tick + time.delta_secs() * PLAY_TICKS_PER_SEC;
        if next >= view.tick_end {
            view.tick = view.tick_end;
            view.playing = false;
        } else {
            view.tick = next;
        }
    }
}

pub fn glide_bodies(
    time: Res<Time>,
    reel: Res<Reel>,
    view: Res<ViewState>,
    mut bodies: Query<(&Creature, &mut Transform, &mut Visibility)>,
) {
    let seconds = time.elapsed_secs();
    let tick = view.tick;
    let (lo, hi, blend) = bracket(&reel.frames, tick);
    for (body, mut transform, mut visibility) in &mut bodies {
        let Some(path) = reel.paths.get(&body.id) else {
            *visibility = Visibility::Hidden;
            continue;
        };
        let alive = path
            .last()
            .is_some_and(|(last, _)| tick <= *last as f32 + 0.02);
        let Some(place) = position_at(path, tick, reel.dims) else {
            *visibility = Visibility::Hidden;
            continue;
        };
        if !view.cells || !alive {
            *visibility = Visibility::Hidden;
            continue;
        }
        let energy = lerped_energy(&reel.frames, lo, hi, blend, body.id);
        *visibility = Visibility::Visible;
        pose_creature(
            body,
            Vec3::from_array(place),
            energy,
            seconds,
            view.body_scale,
            &mut transform,
        );
    }
}

pub fn glide_fluid(
    time: Res<Time>,
    reel: Res<Reel>,
    view: Res<ViewState>,
    mut orbs: Query<(&FluidOrb, &mut Transform, &mut Visibility)>,
) {
    let seconds = time.elapsed_secs();
    let tick = view.tick;
    let (lo, hi, blend) = bracket(&reel.frames, tick);
    for (FluidOrb(index), mut transform, mut visibility) in &mut orbs {
        let amount = reel.fluid_amounts[*index][lo]
            + (reel.fluid_amounts[*index][hi] - reel.fluid_amounts[*index][lo]) * blend;
        let sites = reel.fluid_sites[*index].max(1) as f32;
        let denom = reel.site_cap as f32 * sites;
        let fraction = if denom > 0.0 {
            (amount / denom).clamp(0.0, 1.0)
        } else {
            0.0
        };
        if !view.fluid || fraction <= FLUID_HIDE {
            *visibility = Visibility::Hidden;
            continue;
        }
        let (center, size) = block_pose(reel.fluid_origins[*index], reel.stride, reel.dims);
        let pulse = 1.0 + (seconds * 0.55 + *index as f32).sin() * 0.04;
        let drift = Vec3::new(
            (seconds * 0.17 + *index as f32).sin() * 0.28,
            (seconds * 0.13 + *index as f32 * 0.7).cos() * 0.22,
            (seconds * 0.11 + *index as f32 * 0.4).sin() * 0.18,
        );
        *visibility = Visibility::Visible;
        transform.translation = center + drift;
        transform.scale = secretion_scale(size, fraction, pulse);
    }
}

pub fn glide_threads(
    reel: Res<Reel>,
    gallery: Res<Gallery>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut view: ResMut<ViewState>,
    mut threads: Query<&mut Visibility, With<Threads>>,
) {
    let Ok(mut visibility) = threads.single_mut() else {
        return;
    };
    if view.filaments && reel.trails_present {
        let shown = view.tick.floor() as u64;
        if view.threads_for != Some(shown) {
            let segments = path_segments(&reel.paths, shown, FILAMENT_WINDOW, reel.dims);
            if let Some(mut mesh) = meshes.get_mut(&gallery.threads) {
                write_threads(&mut mesh, &segments, layout::SPACING * 0.055);
            }
            view.threads_for = Some(shown);
        }
        *visibility = Visibility::Visible;
    } else {
        *visibility = Visibility::Hidden;
    }
}

pub fn drift_lamp(
    time: Res<Time>,
    view: Res<ViewState>,
    mut drift: Query<&mut Transform, With<DriftLight>>,
) {
    let Ok(mut transform) = drift.single_mut() else {
        return;
    };
    let angle = time.elapsed_secs() * 0.17;
    let radius = view.lamp_reach;
    transform.translation = Vec3::new(
        angle.cos() * radius,
        angle.sin() * radius * 0.35,
        angle.sin() * radius,
    );
}

pub fn marks(
    mut commands: Commands,
    gallery: Res<Gallery>,
    reel: Res<Reel>,
    mut view: ResMut<ViewState>,
    drawn: Query<Entity, With<Drawn>>,
) {
    if view.hypothesis == view.hypothesis_drawn {
        return;
    }
    for entity in &drawn {
        commands.entity(entity).despawn();
    }
    if view.hypothesis {
        for mark in &reel.hypothesis.marks {
            let place = Vec3::from_array(site_to_view(mark.site, reel.dims));
            commands.spawn((
                Mesh3d(gallery.mark.clone()),
                MeshMaterial3d(gallery.mark_mat.clone()),
                Transform::from_translation(place),
                Drawn,
            ));
        }
    }
    view.hypothesis_drawn = view.hypothesis;
}

pub fn soundtrack(
    mut commands: Commands,
    mut sources: ResMut<Assets<AudioSource>>,
    reel: Res<Reel>,
    mut view: ResMut<ViewState>,
    existing: Query<Entity, With<Score>>,
) {
    let (index, _, _) = bracket(&reel.frames, view.tick);
    let Some(frame) = reel.frames.get(index) else {
        return;
    };
    let key = AudioKey {
        bucket: index as u64 / 2,
        muted: view.muted,
    };
    if view.audio_for == Some(key) {
        return;
    }
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    if !view.muted {
        let bytes =
            crate::audio::synth_wav(frame.tick, frame.live, frame.dead, frame.field, frame.waste);
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

pub fn pace(time: Res<Time>, mut clock: ResMut<FrameClock>, mut view: ResMut<ViewState>) {
    if !clock.reported && clock.seconds < 1.5 && clock.frames == 0 {
        clock.seconds += time.delta_secs();
        if clock.seconds >= 1.5 {
            clock.seconds = 0.0;
            clock.frames = 1;
        }
        return;
    }
    clock.frames += 1;
    clock.seconds += time.delta_secs();
    if clock.seconds >= 0.25 {
        view.fps = clock.frames as f32 / clock.seconds;
    }
    if !clock.reported && clock.seconds >= 2.0 {
        eprintln!("VIEW_FPS {:.1}", view.fps);
        clock.reported = true;
    }
    if clock.seconds >= 2.0 {
        clock.frames = 0;
        clock.seconds = 0.0;
    }
}

pub fn hud(reel: Res<Reel>, view: Res<ViewState>, mut text: Query<&mut Text, With<Hud>>) {
    let Ok(mut text) = text.single_mut() else {
        return;
    };
    let (index, _, _) = bracket(&reel.frames, view.tick);
    let Some(frame) = reel.frames.get(index) else {
        text.0 = "no committed frame".into();
        return;
    };
    let play = if view.playing {
        "playing"
    } else if view.tick >= view.tick_end - 0.001 {
        "held at end"
    } else {
        "paused"
    };
    let sound = if view.muted { "muted" } else { "on" };
    let hypothesis_source = if reel.hypothesis_from_file {
        "file"
    } else {
        "scaffold"
    };
    let hash: String = frame.state_hash.chars().take(16).collect();
    let config: String = reel.config_hash.chars().take(12).collect();
    let code: String = reel.code_version.chars().take(12).collect();
    let fps = if view.fps > 0.0 {
        format!("{:.0} fps", view.fps)
    } else {
        "fps …".into()
    };
    text.0 = format!(
        "Möbius Automata M0   tick {:.1} / {:.0}   {fps}   {play}\n\
         seed {}   config {config}   code {code}   energy cap {}\n\
         live {}   dead {}   field {}   reservoir {}   waste {}\n\
         material residual {}   action-energy residual {}   state {hash}\n\
         bodies are a bone-and-oil costume on committed cells, not organism membership, not in the state hash\n\
         rules locked at the start. space at the end replays this run. it does not retick the world\n\
         1 bodies {}   2 fluid {}   3 filaments {}   4 hypothesis {} ({hypothesis_source}, off until pressed)\n\
         {}\n\
         audio {sound}: viewer pcm, not in the state hash. J/L yaw  I/K pitch  -/= zoom  M mute",
        view.tick,
        view.tick_end,
        reel.seed,
        reel.energy_cap,
        frame.live,
        frame.dead,
        frame.field,
        frame.reservoir,
        frame.waste,
        super::material_residual(frame),
        super::energy_residual(frame),
        on_off(view.cells),
        on_off(view.fluid),
        on_off(view.filaments),
        on_off(view.hypothesis),
        view.note,
    );
}

pub fn view_for(reel: &Reel) -> ViewState {
    let distance = crate::layout::stage_distance(reel.dims);
    ViewState {
        tick: 0.0,
        tick_end: reel
            .frames
            .last()
            .map(|frame| frame.tick as f32)
            .unwrap_or(0.0),
        playing: true,
        cells: true,
        fluid: true,
        filaments: true,
        hypothesis: false,
        hypothesis_drawn: false,
        muted: false,
        yaw: 0.55,
        pitch: 0.28,
        distance,
        distance_min: distance * 0.28,
        distance_max: distance * 2.6,
        body_scale: BODY_SCALE,
        lamp_reach: layout::SPACING * reel.dims.width.max(1) as f32 * 0.28,
        fps: 0.0,
        audio_for: None,
        threads_for: None,
        note: opening_note(reel),
    }
}

pub fn view_for_hall(hall: &Hall) -> ViewState {
    let far = crate::layout::stage_distance(hall.dims);
    ViewState {
        tick: 0.0,
        tick_end: 0.0,
        playing: true,
        cells: true,
        fluid: true,
        filaments: true,
        hypothesis: false,
        hypothesis_drawn: false,
        muted: false,
        yaw: 0.35,
        pitch: 0.16,
        distance: 9.0,
        distance_min: 3.5,
        distance_max: far,
        body_scale: 1.7,
        lamp_reach: 3.2,
        fps: 0.0,
        audio_for: None,
        threads_for: None,
        note: String::new(),
    }
}

pub fn assemble_hall(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    hall: Res<Hall>,
    mut cameras: Query<(Entity, &mut Projection), With<Camera3d>>,
) {
    if let Ok((entity, mut projection)) = cameras.single_mut() {
        if let Projection::Perspective(perspective) = projection.as_mut() {
            perspective.fov = HALL_FOV;
        }
        commands.entity(entity).insert(DistanceFog {
            color: Color::srgb(0.01, 0.016, 0.012),
            directional_light_color: Color::NONE,
            directional_light_exponent: 8.0,
            falloff: FogFalloff::Linear {
                start: 5.0,
                end: 16.0,
            },
        });
    }
    let span = (hall
        .dims
        .width
        .max(hall.dims.height)
        .max(hall.dims.depth)
        .saturating_sub(1) as f32)
        * layout::SPACING;
    let extent = Vec3::splat(span) + Vec3::splat(layout::SPACING * 2.2);
    let cage = meshes.add(bone_cage(extent, 0.055));
    let cage_mat = materials.add(oily(
        Color::srgb(0.04, 0.045, 0.04),
        LinearRgba::new(0.0, 0.004, 0.002, 1.0),
        0.72,
        0.48,
        0.22,
    ));
    commands.spawn((Mesh3d(cage), MeshMaterial3d(cage_mat), Transform::default()));
    let creatures = [
        meshes.add(creature_mesh(0)),
        meshes.add(creature_mesh(1)),
        meshes.add(creature_mesh(2)),
    ];
    let creature_mats = [
        materials.add(oily(
            Color::srgb(0.46, 0.4, 0.32),
            LinearRgba::new(0.03, 0.016, 0.006, 1.0),
            0.42,
            0.34,
            0.86,
        )),
        materials.add(oily(
            Color::srgb(0.03, 0.035, 0.028),
            LinearRgba::new(0.002, 0.01, 0.004, 1.0),
            0.96,
            0.08,
            1.0,
        )),
        materials.add(oily(
            Color::srgb(0.32, 0.08, 0.06),
            LinearRgba::new(0.05, 0.006, 0.004, 1.0),
            0.7,
            0.22,
            0.7,
        )),
    ];
    let orb = meshes.add(Sphere::new(1.0).mesh().ico(3).expect("icosphere"));
    let orb_mat = materials.add(bile_material());
    let empty = meshes.add(creature_mesh(0));
    if let Some(mut mesh) = meshes.get_mut(&empty) {
        write_threads(&mut mesh, &[], 0.01);
    }
    let threads = empty;
    let thread_mat = materials.add(oily(
        Color::srgb(0.38, 0.3, 0.22),
        LinearRgba::new(0.03, 0.01, 0.004, 1.0),
        0.64,
        0.28,
        0.55,
    ));
    let mark = meshes.add(Sphere::new(0.08).mesh().ico(1).expect("mark sphere"));
    let mark_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.95, 0.22, 0.55),
        unlit: true,
        ..default()
    });
    commands.insert_resource(Gallery {
        creatures,
        creature_mats,
        threads: threads.clone(),
        orbs: orb,
        orb_mat,
        mark,
        mark_mat,
    });
    commands.spawn((
        Mesh3d(threads),
        MeshMaterial3d(thread_mat),
        Transform::default(),
        Visibility::Hidden,
        Threads,
    ));
    commands.spawn(lamp(
        Color::srgb(1.0, 0.72, 0.48),
        650_000.0,
        Vec3::new(2.2, 1.4, 1.6),
    ));
    commands.spawn(lamp(
        Color::srgb(0.22, 0.42, 0.18),
        160_000.0,
        Vec3::new(-2.4, 0.3, -1.4),
    ));
    commands.spawn((
        lamp(
            Color::srgb(0.62, 0.08, 0.05),
            380_000.0,
            Vec3::new(0.6, -1.2, 1.5),
        ),
        DriftLight,
    ));
}

pub fn hall_pump(mut hall: ResMut<Hall>) {
    hall.drain();
}

pub fn hall_clock(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    hall: Res<Hall>,
    mut view: ResMut<ViewState>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        if hall.halted.is_some() {
            view.playing = false;
            view.note = "That tick was refused. Space does not invent another one.".into();
        } else {
            let paused = !hall.is_paused();
            hall.set_paused(paused);
            view.playing = !paused;
        }
    }
    if view.playing && hall.halted.is_none() {
        let target = hall.committed_tick() as f32;
        let next = view.tick + time.delta_secs() * LIVE_TICKS_PER_SEC;
        view.tick = next.min(target);
    }
}

pub fn embody(mut commands: Commands, gallery: Res<Gallery>, mut hall: ResMut<Hall>) {
    if hall.spawned {
        return;
    }
    let Some(picture) = hall.latest.clone() else {
        return;
    };
    if picture.error.is_some() && picture.cells.is_empty() {
        hall.spawned = true;
        eprintln!("VIEW_READY creatures 0 orbs 0");
        return;
    }
    for cell in &picture.cells {
        let costume = costume(cell.id, cell.genome);
        commands.spawn((
            Mesh3d(gallery.creatures[costume.variant as usize].clone()),
            MeshMaterial3d(gallery.creature_mats[costume.variant as usize].clone()),
            Transform::default(),
            Visibility::Hidden,
            costume.body,
        ));
    }
    for index in 0..hall.fluid_origins.len() {
        commands.spawn((
            Mesh3d(gallery.orbs.clone()),
            MeshMaterial3d(gallery.orb_mat.clone()),
            Transform::default(),
            Visibility::Hidden,
            FluidOrb(index),
        ));
    }
    hall.spawned = true;
    eprintln!(
        "VIEW_READY creatures {} orbs {}",
        picture.cells.len(),
        hall.fluid_origins.len()
    );
}

pub fn hall_bodies(
    time: Res<Time>,
    hall: Res<Hall>,
    view: Res<ViewState>,
    mut bodies: Query<(&Creature, &mut Transform, &mut Visibility)>,
) {
    let Some(picture) = &hall.latest else {
        return;
    };
    let seconds = time.elapsed_secs();
    let tick = view.tick;
    for (body, mut transform, mut visibility) in &mut bodies {
        let Some(path) = hall.paths.get(&body.id) else {
            *visibility = Visibility::Hidden;
            continue;
        };
        let alive = path
            .last()
            .is_some_and(|(last, _)| tick <= *last as f32 + 0.02);
        let Some(place) = position_at(path, tick, hall.dims) else {
            *visibility = Visibility::Hidden;
            continue;
        };
        if !view.cells || !alive {
            *visibility = Visibility::Hidden;
            continue;
        }
        *visibility = Visibility::Visible;
        pose_creature(
            body,
            Vec3::from_array(place),
            founder_energy(&picture.cells, body.id),
            seconds,
            view.body_scale,
            &mut transform,
        );
    }
}

pub fn hall_fluid(
    time: Res<Time>,
    hall: Res<Hall>,
    view: Res<ViewState>,
    mut orbs: Query<(&FluidOrb, &mut Transform, &mut Visibility)>,
) {
    let seconds = time.elapsed_secs();
    let floor_tick = view.tick.floor() as u64;
    let blend = if hall.committed_tick() == floor_tick.saturating_add(1) {
        view.tick.fract()
    } else {
        1.0
    };
    for (FluidOrb(index), mut transform, mut visibility) in &mut orbs {
        let Some(curr) = hall.fluid_amounts.get(*index).copied() else {
            *visibility = Visibility::Hidden;
            continue;
        };
        let prev = hall.fluid_before.get(*index).copied().unwrap_or(curr);
        let amount = prev + (curr - prev) * blend;
        let sites = hall.fluid_sites.get(*index).copied().unwrap_or(1).max(1) as f32;
        let denom = hall.site_cap as f32 * sites;
        let fraction = if denom > 0.0 {
            (amount / denom).clamp(0.0, 1.0)
        } else {
            0.0
        };
        if !view.fluid || fraction <= FLUID_HIDE {
            *visibility = Visibility::Hidden;
            continue;
        }
        let Some(origin) = hall.fluid_origins.get(*index).copied() else {
            *visibility = Visibility::Hidden;
            continue;
        };
        let (center, size) = block_pose(origin, hall.stride, hall.dims);
        let pulse = 1.0 + (seconds * 0.55 + *index as f32).sin() * 0.04;
        *visibility = Visibility::Visible;
        transform.translation = center;
        transform.scale = secretion_scale(size, fraction, pulse);
    }
}

pub fn hall_threads(
    hall: Res<Hall>,
    gallery: Res<Gallery>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut view: ResMut<ViewState>,
    mut threads: Query<&mut Visibility, With<Threads>>,
) {
    let Ok(mut visibility) = threads.single_mut() else {
        return;
    };
    if view.filaments && hall.latest.is_some() {
        let shown = view.tick.floor() as u64;
        if view.threads_for != Some(shown) {
            let segments = path_segments(&hall.paths, shown, FILAMENT_WINDOW, hall.dims);
            if let Some(mut mesh) = meshes.get_mut(&gallery.threads) {
                write_threads(&mut mesh, &segments, layout::SPACING * 0.055);
            }
            view.threads_for = Some(shown);
        }
        *visibility = Visibility::Visible;
    } else {
        *visibility = Visibility::Hidden;
    }
}

pub fn hall_sound(
    mut commands: Commands,
    mut sources: ResMut<Assets<AudioSource>>,
    hall: Res<Hall>,
    mut view: ResMut<ViewState>,
    existing: Query<Entity, With<Score>>,
) {
    let Some(picture) = &hall.latest else {
        return;
    };
    let key = AudioKey {
        bucket: picture.tick / 16,
        muted: view.muted,
    };
    if view.audio_for == Some(key) {
        return;
    }
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    if !view.muted {
        let bytes = crate::audio::synth_wav(
            picture.tick,
            picture.live,
            picture.dead,
            picture.field,
            picture.waste,
        );
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

pub fn hall_hud(hall: Res<Hall>, view: Res<ViewState>, mut text: Query<&mut Text, With<Hud>>) {
    let Ok(mut text) = text.single_mut() else {
        return;
    };
    let Some(picture) = &hall.latest else {
        text.0 = format!(
            "Möbius Automata M0\n{}\nUp/Down and Left/Right were on the slate. This run is locked.",
            hall.note
        );
        return;
    };
    let play = if hall.halted.is_some() {
        "refused"
    } else if view.playing {
        "ticking"
    } else {
        "paused"
    };
    let sound = if view.muted { "muted" } else { "on" };
    let hash: String = picture.state_hash.chars().take(16).collect();
    let config: String = hall.config_hash.chars().take(12).collect();
    let code: String = hall.code_version.chars().take(12).collect();
    let fps = if view.fps > 0.0 {
        format!("{:.0} fps", view.fps)
    } else {
        "fps …".into()
    };
    let extra = if !hall.note.is_empty() {
        format!("{}\n", hall.note)
    } else if !view.note.is_empty() {
        format!("{}\n", view.note)
    } else {
        String::new()
    };
    text.0 = format!(
        "Möbius Automata M0   tick {}   shown {:.1}   {fps}   {play}\n\
         open run. founding cells {}. no division. no mutation. no new organism.\n\
         seed {}   config {config}   code {code}   energy cap {}\n\
         live {}   dead {}   field {}   reservoir {}   waste {}\n\
         material residual {}   action-energy residual {}   state {hash} at tick {}\n\
         bone, oil, fangs, and bile are a Giger costume. not membership. not in the state hash\n\
         fluid stride {} hides thin blocks. the field line above is the stock.\n\
         1 bodies {}   2 fluid {}   3 filaments {}   4 hypothesis {} (empty scaffold, not membership)\n\
         {extra}\
         space pauses the scientific clock. closing the window stops it.\n\
         audio {sound}: viewer pcm, not in the state hash. J/L yaw  I/K pitch  -/= zoom  M mute",
        picture.tick,
        view.tick,
        hall.founding,
        hall.seed,
        hall.energy_cap,
        picture.live,
        picture.dead,
        picture.field,
        picture.reservoir,
        picture.waste,
        picture.material_residual(),
        picture.energy_residual(),
        picture.tick,
        hall.stride,
        on_off(view.cells),
        on_off(view.fluid),
        on_off(view.filaments),
        on_off(view.hypothesis),
    );
}

fn install_hall(commands: &mut Commands, hall: Hall) {
    let view = view_for_hall(&hall);
    commands.insert_resource(view);
    commands.insert_resource(FrameClock {
        frames: 0,
        seconds: 0.0,
        reported: false,
    });
    commands.insert_resource(hall);
}

enum Started {
    Reel(Reel),
    Hall(Hall),
}

fn install_playback(commands: &mut Commands, reel: Reel) {
    let view = view_for(&reel);
    commands.insert_resource(view);
    commands.insert_resource(FrameClock {
        frames: 0,
        seconds: 0.0,
        reported: false,
    });
    commands.insert_resource(reel);
}

fn commit_new(draft: &Draft) -> Result<PathBuf, String> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("moebius-play-{nanos}"));
    fs::create_dir_all(&dir).map_err(|err| format!("cannot create {}: {err}", dir.display()))?;
    let config_path = dir.join("operator.json");
    let text = serde_json::to_string_pretty(&draft.config)
        .map_err(|err| format!("cannot encode rules: {err}"))?;
    fs::write(&config_path, format!("{text}\n"))
        .map_err(|err| format!("cannot write {}: {err}", config_path.display()))?;
    let out = dir.join("run");
    moebius_cli::run_batch(&moebius_cli::RunOpts {
        seed: draft.seed,
        ticks: draft.ticks,
        config: config_path,
        out: out.clone(),
        snapshot_interval: draft.snapshot_interval,
    })
    .map_err(|err| err.to_string())?;
    Ok(out)
}

fn opening_note(reel: &Reel) -> String {
    let mut note = format!(
        "fluid stride {} draws {} block-sum orbs. The sums are a downsample, not a new stock.",
        reel.stride,
        reel.fluid_origins.len()
    );
    if !reel.trails_present {
        note.push_str(" trails.csv is absent, so filaments have no samples.");
    } else {
        note.push_str(" filaments are the last committed steps, split on a wrap. Not bonds.");
    }
    note
}

fn on_off(enabled: bool) -> &'static str {
    if enabled { "on" } else { "off" }
}

fn bracket(frames: &[Frame], tick: f32) -> (usize, usize, f32) {
    if frames.is_empty() {
        return (0, 0, 0.0);
    }
    if tick <= frames[0].tick as f32 {
        return (0, 0, 0.0);
    }
    let last = frames.len() - 1;
    if tick >= frames[last].tick as f32 {
        return (last, last, 0.0);
    }
    let mut lo = 0usize;
    let mut hi = last;
    while hi - lo > 1 {
        let mid = (lo + hi) / 2;
        if (frames[mid].tick as f32) <= tick {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let span = (frames[hi].tick - frames[lo].tick) as f32;
    let blend = if span > 0.0 {
        ((tick - frames[lo].tick as f32) / span).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (lo, hi, blend)
}

fn neighbor_tick(frames: &[Frame], tick: f32, forward: bool) -> f32 {
    if frames.is_empty() {
        return 0.0;
    }
    if forward {
        frames
            .iter()
            .find(|frame| frame.tick as f32 > tick + 0.001)
            .map(|frame| frame.tick as f32)
            .unwrap_or_else(|| frames.last().map(|frame| frame.tick as f32).unwrap_or(0.0))
    } else {
        frames
            .iter()
            .rev()
            .find(|frame| (frame.tick as f32) < tick - 0.001)
            .map(|frame| frame.tick as f32)
            .unwrap_or(0.0)
    }
}

fn energy_of(frame: &Frame, id: u64) -> Option<u64> {
    frame
        .cells
        .binary_search_by_key(&id, |cell| cell.id)
        .ok()
        .map(|index| frame.cells[index].energy)
}

fn lerped_energy(frames: &[Frame], lo: usize, hi: usize, blend: f32, id: u64) -> f32 {
    match (energy_of(&frames[lo], id), energy_of(&frames[hi], id)) {
        (Some(left), Some(right)) => left as f32 + (right as f32 - left as f32) * blend,
        (Some(value), None) | (None, Some(value)) => value as f32,
        (None, None) => 0.0,
    }
}

struct Costume {
    variant: u8,
    body: Creature,
}

fn costume(id: u64, genome: [u8; 8]) -> Costume {
    let mix = id
        ^ u64::from(genome[0])
        ^ (u64::from(genome[1]) << 8)
        ^ (u64::from(genome[2]) << 16)
        ^ (u64::from(genome[5]) << 24);
    let stretch = Vec3::new(
        0.78 + (genome[2] as f32 / 255.0) * 0.55 + ((id >> 3) % 5) as f32 * 0.04,
        0.9 + (genome[3] as f32 / 255.0) * 0.7 + ((id >> 5) % 7) as f32 * 0.03,
        0.78 + (genome[4] as f32 / 255.0) * 0.55 + ((id >> 7) % 5) as f32 * 0.04,
    );
    let spin_mag = 0.35 + ((mix >> 8) % 180) as f32 / 180.0;
    Costume {
        variant: (mix % 3) as u8,
        body: Creature {
            id,
            phase: ((mix % 10_000) as f32) * 0.001 * std::f32::consts::TAU,
            spin: if mix & 1 == 0 { spin_mag } else { -spin_mag },
            stretch,
            facing: Quat::from_euler(
                EulerRot::YXZ,
                (mix % 360) as f32 * std::f32::consts::TAU / 360.0,
                (((mix >> 10) % 100) as f32 / 100.0 - 0.5) * 0.9,
                (((mix >> 16) % 100) as f32 / 100.0 - 0.5) * 0.6,
            ),
        },
    }
}

fn pose_creature(
    body: &Creature,
    place: Vec3,
    energy: f32,
    seconds: f32,
    body_scale: f32,
    transform: &mut Transform,
) {
    let bulk = 0.88 + 0.28 * (1.0 - (-energy / 48.0).exp());
    let breathe = (seconds * 1.7 + body.phase).sin() * 0.045;
    transform.translation = place;
    transform.rotation = Quat::from_rotation_y(seconds * body.spin + body.phase)
        * Quat::from_rotation_x((seconds * 0.7 + body.phase).sin() * 0.18)
        * body.facing;
    transform.scale = body.stretch * (body_scale * bulk * (1.0 + breathe));
}

fn founder_energy(cells: &[Founder], id: u64) -> f32 {
    cells
        .binary_search_by_key(&id, |cell| cell.id)
        .ok()
        .map(|index| cells[index].energy as f32)
        .unwrap_or(0.0)
}

fn secretion_scale(size: Vec3, fraction: f32, pulse: f32) -> Vec3 {
    Vec3::splat(size.max_element() * 0.45 * fraction * pulse)
}

fn bile_material() -> StandardMaterial {
    StandardMaterial {
        base_color: Color::srgba(0.05, 0.07, 0.032, 0.32),
        emissive: LinearRgba::new(0.016, 0.028, 0.006, 1.0),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.28,
        metallic: 0.62,
        reflectance: 0.46,
        ..default()
    }
}

fn oily(
    base: Color,
    emissive: LinearRgba,
    metallic: f32,
    rough: f32,
    coat: f32,
) -> StandardMaterial {
    StandardMaterial {
        base_color: base,
        emissive,
        metallic,
        perceptual_roughness: rough,
        reflectance: 0.5,
        clearcoat: coat,
        clearcoat_perceptual_roughness: 0.12,
        ..default()
    }
}

fn lamp(color: Color, intensity: f32, at: Vec3) -> impl Bundle {
    (
        PointLight {
            color,
            intensity,
            range: 90.0,
            radius: 1.8,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_translation(at),
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
    let size = (far - near).abs() + Vec3::splat(layout::SPACING * 0.85);
    (center, size)
}
