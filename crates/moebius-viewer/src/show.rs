//! Rule slate, the committed-run picture, and the open hall.
//! These systems do not call `World::tick`. The hall worker does.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use bevy::audio::Volume;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::mesh::skinning::{SkinnedMesh, SkinnedMeshInverseBindposes};
use bevy::prelude::*;
use bevy::text::FontSize;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use super::{
    AudioKey, BODY_SCALE, Boot, Creature, Dims, Drawn, DriftLight, FluidOrb, Frame, FrameClock,
    Gallery, Hud, PLAY_TICKS_PER_SEC, Reel, Score, Site, Stage, Threads, ViewState, load_reel,
};
use crate::body::{
    BIND_START, BIND_STEP, bone_cage, creature_mesh, food_clot_mesh, fused_flesh_mesh,
    mandible_mesh, nutrient_mesh, rig_bind_positions, write_slime, write_threads,
};
use crate::layout::{self, CAMERA_FOV, FILAMENT_WINDOW, path_segments, position_at, site_to_view};
use crate::live::{self, Hall};
use crate::rules::{self, Draft};
use moebius_core::farm::{DIG_START, JOINTS, SPINE_JOINTS, action_dir};

const FLUID_HIDE: f32 = 0.008;
const HALL_FOV: f32 = 64.0 * std::f32::consts::PI / 180.0;

#[derive(Resource, Default)]
pub struct FlyCamera {
    captured: bool,
    selected: Option<u64>,
    inspect: bool,
    page: usize,
}
impl FlyCamera {
    pub fn with_inspector(inspect: bool) -> Self {
        Self {
            inspect,
            ..default()
        }
    }
}
#[derive(Component)]
pub struct MotorJoint {
    id: u64,
    index: usize,
}
#[derive(Component)]
pub struct FeedingMaw {
    id: u64,
}

#[derive(Component)]
pub struct FleshSurface;

pub fn flesh_surfaces(
    mut commands: Commands,
    hall: Res<Hall>,
    gallery: Res<Gallery>,
    mut meshes: ResMut<Assets<Mesh>>,
    surface: Query<Entity, With<FleshSurface>>,
    mut previous: Local<Vec<moebius_core::farm::FleshDeposit>>,
) {
    let Some(picture) = &hall.latest else {
        return;
    };
    if *previous == picture.flesh {
        return;
    }
    if let Some(mut mesh) = meshes.get_mut(&gallery.flesh_mesh) {
        *mesh = fused_flesh_mesh(&picture.flesh, hall.dims);
    }
    previous.clone_from(&picture.flesh);
    if surface.is_empty() {
        commands.spawn((
            Mesh3d(gallery.flesh_mesh.clone()),
            MeshMaterial3d(gallery.flesh_mat.clone()),
            Transform::default(),
            FleshSurface,
        ));
    }
}

fn rig_assets(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    bindposes: &mut Assets<SkinnedMeshInverseBindposes>,
) -> (
    Handle<SkinnedMeshInverseBindposes>,
    Handle<Mesh>,
    Handle<StandardMaterial>,
) {
    let poses = rig_bind_positions()
        .into_iter()
        .map(|position| Mat4::from_translation(position).inverse())
        .collect::<Vec<_>>();
    (
        bindposes.add(poses),
        meshes.add(Sphere::new(0.08).mesh().ico(2).expect("mouth")),
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.27, 0.035, 0.045),
            metallic: 0.18,
            perceptual_roughness: 0.22,
            clearcoat: 0.8,
            ..default()
        }),
    )
}

#[derive(Component)]
pub struct NeuralPanel;
#[derive(Component)]
pub struct JetVisual(usize);

pub fn neural_inspector(
    keyboard: Res<ButtonInput<KeyCode>>,
    hall: Res<Hall>,
    view: Res<ViewState>,
    mut fly: ResMut<FlyCamera>,
    mut panels: Query<(&mut Text, &mut Node), With<NeuralPanel>>,
    mut history: Local<(u64, Option<u64>, Vec<[i32; 8]>)>,
) {
    if keyboard.just_pressed(KeyCode::KeyN) {
        fly.inspect = !fly.inspect;
    }
    if fly.inspect && keyboard.just_pressed(KeyCode::Tab) {
        fly.page = (fly.page + 1) % 6;
    }
    let Ok((mut text, mut node)) = panels.single_mut() else {
        return;
    };
    node.display = if fly.inspect {
        Display::Flex
    } else {
        Display::None
    };
    if !fly.inspect {
        return;
    }
    let Some(picture) = &hall.latest else {
        return;
    };
    if fly
        .selected
        .is_none_or(|id| picture.cells.binary_search_by_key(&id, |c| c.id).is_err())
    {
        fly.selected = picture.cells.first().map(|c| c.id);
    }
    let Some(id) = fly.selected else {
        text.0 = "No living organism".into();
        return;
    };
    let cell = &picture.cells[picture
        .cells
        .binary_search_by_key(&id, |c| c.id)
        .expect("selected")];
    let Some(rig) = &cell.rig else {
        return;
    };
    let Some(state) = &rig.neural else {
        text.0 = "Legacy FARM2 discrete motor controller; no neural activations".into();
        return;
    };
    if history.1 != Some(id) {
        history.1 = Some(id);
        history.2.clear();
    }
    if history.0 != picture.tick {
        history.0 = picture.tick;
        history.2.push(
            state
                .activations
                .as_slice()
                .try_into()
                .expect("eight neurons"),
        );
        if history.2.len() > 48 {
            history.2.remove(0);
        }
    }
    let mut lines = format!(
        "NEURAL INSPECTOR  id {id}  parent {}  gen {}\ncommitted {}  rendered {:.2}  page {}/6  Tab: next  N: close\ngenotype {}\n",
        rig.parent,
        rig.generation,
        picture.tick,
        view.tick,
        fly.page + 1,
        &state.genome.id()[..20]
    );
    match fly.page {
        0 => {
            lines.push_str(&format!("energy {}   last intake {}   physiological signal {}\nneural cost {}   actuator cost {}   jets {}\nvelocity {:?}   acceleration {:?}\norientation {:?}   angular rate {:?}\nactivations {:?}\ntime constants {:?}\nplastic changed updates {}   delta L1 {}\n\nSignals below are committed simulation state.\nBody display blends observations; it does not tick or choose motors.\nFull body/soft-tissue physics and learned locomotion are unproven.\n",cell.energy,state.last_intake,state.reward,state.spent_neural,state.spent_actuator,picture.jets.len(),state.body.velocity,state.body.acceleration,state.body.orientation,state.body.angular_velocity,state.activations,state.genome.tau,state.changed_connections,state.plastic.iter().map(|v|v.unsigned_abs() as u64).sum::<u64>()));
            lines.push_str("\nObserved activation history (render sampling):\n");
            let shades = b" .:-=+*#%@";
            for neuron in 0..8 {
                let spark: String = history
                    .2
                    .iter()
                    .map(|frame| {
                        let firing = moebius_core::neural::Q as i64 * i64::from(frame[neuron])
                            / (1024 + i64::from(frame[neuron]).abs());
                        let bucket = ((firing + 1024) * 9 / 2048).clamp(0, 9) as usize;
                        shades[bucket] as char
                    })
                    .collect();
                lines.push_str(&format!("n{neuron} |{spark}\n"));
            }
        }
        1 => {
            lines.push_str("axis: command / delivered torque / angle / rate / active ticks\n");
            for (i, binding) in state.genome.anatomy.axes.iter().enumerate() {
                lines.push_str(&format!(
                    "j{:02} {:?}: {:+5} / {:+5} / {:+4} / {:+3} / {}\n",
                    binding.joint,
                    binding.axis,
                    state.commands[i],
                    state.realized_efforts[i],
                    state.body.angles[i],
                    state.body.rates[i],
                    state.participation[i]
                ));
            }
            let n = state.genome.anatomy.axes.len();
            lines.push_str(&format!(
                "valve/nozzle commands {:?}\nrealized {:?}\n",
                &state.commands[n..],
                &state.realized_efforts[n..]
            ));
        }
        2..=4 => {
            let names = state.genome.anatomy.sensor_names();
            let start = (fly.page - 2) * 36;
            lines.push_str("sensor ABI 1: bounded fixed-point normalized [-1024,1024]\n");
            for (i, name) in names.iter().enumerate().skip(start).take(36) {
                lines.push_str(&format!("{i:03} {name}: {:+5}\n", state.sensors[i]));
            }
        }
        _ => {
            lines.push_str("inherited recurrent weights (rows):\n");
            for row in state.genome.recurrent.chunks(8) {
                lines.push_str(&format!("{row:?}\n"));
            }
            lines.push_str("\nlifetime plastic deltas (half is written into the child genes):\n");
            for row in state.plastic.chunks(8) {
                lines.push_str(&format!("{row:?}\n"));
            }
            lines.push_str(&format!(
                "\nplasticity parameters {:?}\n",
                state.genome.plastic_rates
            ));
        }
    }
    text.0 = lines;
}

pub fn jet_picture(
    mut commands: Commands,
    hall: Res<Hall>,
    gallery: Res<Gallery>,
    mut pool: Local<usize>,
    mut jets: Query<(&JetVisual, &mut Transform, &mut Visibility)>,
) {
    let Some(picture) = &hall.latest else {
        return;
    };
    while *pool < picture.jets.len() {
        commands.spawn((
            Mesh3d(gallery.mouth.clone()),
            MeshMaterial3d(gallery.flesh_mat.clone()),
            Transform::default(),
            Visibility::Hidden,
            JetVisual(*pool),
        ));
        *pool += 1;
    }
    let half = Vec3::new(
        (hall.dims.width - 1) as f32,
        (hall.dims.height - 1) as f32,
        (hall.dims.depth - 1) as f32,
    ) * 0.5;
    for (JetVisual(index), mut transform, mut visibility) in &mut jets {
        if let Some(jet) = picture.jets.get(*index) {
            *visibility = Visibility::Visible;
            let p = Vec3::new(
                jet.position[0] as f32,
                jet.position[1] as f32,
                jet.position[2] as f32,
            ) / 1024.0;
            transform.translation = (p - half) * layout::SPACING;
            // A spent droplet, much smaller than the mouth. Spinal work is not a spit.
            transform.scale = Vec3::splat(0.18);
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

pub fn boot(mut commands: Commands, boot: Res<Boot>, mut next: ResMut<NextState<Stage>>) {
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(12.0),
            ..default()
        },
        TextColor(Color::srgb(0.85, 0.84, 0.75)),
        Node {
            position_type: PositionType::Absolute,
            right: px(12),
            bottom: px(12),
            width: px(580),
            padding: UiRect::all(px(10)),
            display: Display::None,
            ..default()
        },
        BackgroundColor(Color::srgba(0.015, 0.02, 0.018, 0.94)),
        NeuralPanel,
    ));
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
                .map(Box::new)
                .map(Started::Reel)
        } else if open_run {
            live::open(&draft.config, draft.seed)
                .map(Box::new)
                .map(Started::Hall)
        } else {
            commit_new(&draft)
                .and_then(|path| load_reel(&path))
                .map(Box::new)
                .map(Started::Reel)
        };
        draft.pending = false;
        match outcome {
            Ok(Started::Reel(reel)) => {
                install_playback(&mut commands, *reel);
                next.set(Stage::Playing);
            }
            Ok(Started::Hall(hall)) => {
                install_hall(&mut commands, *hall);
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
    mut bindpose_assets: ResMut<Assets<SkinnedMeshInverseBindposes>>,
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
    let creature_mats = std::array::from_fn(|_| materials.add(carapace_material()));
    diffuse_lights(&mut commands);
    let orb = meshes.add(nutrient_mesh());
    let (bindposes, mouth, mouth_mat) =
        rig_assets(&mut meshes, &mut materials, &mut bindpose_assets);
    let orb_mat = materials.add(bile_material());
    let flesh_mesh = meshes.add(nutrient_mesh());
    let mandibles = [
        meshes.add(mandible_mesh(-1.0)),
        meshes.add(mandible_mesh(1.0)),
    ];
    let flesh_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.26, 0.075, 0.085),
        metallic: 0.10,
        perceptual_roughness: 0.28,
        clearcoat: 0.85,
        ..default()
    });
    let empty = meshes.add(creature_mesh(0));
    if let Some(mut mesh) = meshes.get_mut(&empty) {
        write_threads(&mut mesh, &[], 0.01);
    }
    let threads = empty;
    let thread_mat = materials.add(filament_material());
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
        bindposes,
        mouth,
        mouth_mat,
        flesh_mat,
        flesh_mesh,
        mandibles,
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

#[allow(clippy::too_many_arguments)] // Bevy system inputs.
pub fn orbit(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    mut fly: ResMut<FlyCamera>,
    hall: Option<Res<Hall>>,
    mut cursors: Query<(&Window, &mut CursorOptions), With<PrimaryWindow>>,
    mut view: ResMut<ViewState>,
    mut camera: Query<(&mut Transform, &mut Projection), With<Camera3d>>,
) {
    let Ok((window, mut cursor)) = cursors.single_mut() else {
        return;
    };
    if keyboard.just_pressed(KeyCode::Escape) || !window.focused {
        fly.captured = false;
    } else if mouse.just_pressed(MouseButton::Left) {
        fly.captured = true;
    }
    cursor.visible = !fly.captured;
    cursor.grab_mode = if fly.captured {
        CursorGrabMode::Locked
    } else {
        CursorGrabMode::None
    };
    if fly.captured {
        view.yaw -= motion.delta.x * 0.0025;
        view.pitch = (view.pitch - motion.delta.y * 0.0025).clamp(-1.52, 1.52);
    }
    let rotation = Quat::from_rotation_y(view.yaw) * Quat::from_rotation_x(view.pitch);
    let forward = rotation * Vec3::NEG_Z;
    let right = rotation * Vec3::X;
    let mut eye = view.eye.unwrap_or(Vec3::new(0.0, 0.4, view.distance));
    if window.focused {
        if keyboard.just_pressed(KeyCode::KeyF)
            && let Some(hall) = &hall
            && let Some(picture) = &hall.latest
            && let Some(target) = picture
                .cells
                .iter()
                .filter_map(|cell| {
                    hall.poses
                        .get(&cell.id)
                        .map(|(_, p)| (cell.id, Vec3::from_array(*p)))
                })
                .min_by(|a, b| {
                    a.1.distance_squared(eye)
                        .total_cmp(&b.1.distance_squared(eye))
                })
        {
            fly.selected = Some(target.0);
            eye = target.1 - forward * 2.2;
        }
        let mut direction = Vec3::ZERO;
        if keyboard.pressed(KeyCode::KeyW) {
            direction += forward;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            direction -= forward;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            direction -= right;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            direction += right;
        }
        if keyboard.pressed(KeyCode::KeyE) {
            direction += Vec3::Y;
        }
        if keyboard.pressed(KeyCode::KeyQ) {
            direction -= Vec3::Y;
        }
        let speed = if keyboard.pressed(KeyCode::ShiftLeft) {
            12.0
        } else {
            3.0
        };
        eye += direction.normalize_or_zero() * speed * time.delta_secs().min(0.05);
        let wheel = scroll.delta.y
            * if scroll.unit == MouseScrollUnit::Pixel {
                0.025
            } else {
                1.0
            };
        view.fov = (view.fov - wheel * 0.045).clamp(0.30, 1.55);
    }
    view.eye = Some(eye);
    if let Ok((mut transform, mut projection)) = camera.single_mut() {
        transform.translation = eye;
        transform.rotation = rotation;
        if let Projection::Perspective(p) = projection.as_mut() {
            p.fov = view.fov;
        }
    }
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
    if keyboard.just_pressed(KeyCode::BracketLeft) {
        view.audio_level = view.audio_level.saturating_sub(1);
    }
    if keyboard.just_pressed(KeyCode::BracketRight) {
        view.audio_level = (view.audio_level + 1).min(10);
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
                write_threads(&mut mesh, &segments, layout::SPACING * 0.09);
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
    time: Res<Time>,
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
        bucket: (time.elapsed_secs_f64() / 8.0) as u64,
        muted: view.muted || !view.playing || view.audio_level == 0,
        level: view.audio_level,
    };
    if view.audio_for == Some(key) {
        return;
    }
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    if !key.muted {
        let bytes =
            crate::audio::synth_wav(frame.tick, frame.live, frame.dead, frame.field, frame.waste);
        let handle = sources.add(AudioSource {
            bytes: Arc::<[u8]>::from(bytes),
        });
        commands.spawn((
            AudioPlayer::new(handle),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.022 * view.audio_level as f32)),
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
    let sound = if view.muted {
        "muted".into()
    } else {
        format!("bursts {}/10", view.audio_level)
    };
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
         1 bodies {}   2 fluid {}   3 slime {}   4 hypothesis {} ({hypothesis_source}, off until pressed)\n\
         {}\n\
         audio {sound}: viewer pcm, not in the state hash. WASD move  Q/E rise/fall  click: mouse look  Esc: release  wheel: zoom  F inspect nearest  M mute  [/] volume",
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
        audio_level: 3,
        yaw: 0.55,
        pitch: 0.28,
        distance,
        eye: None,
        fov: CAMERA_FOV,
        body_scale: BODY_SCALE,
        lamp_reach: layout::SPACING * reel.dims.width.max(1) as f32 * 0.28,
        fps: 0.0,
        audio_for: None,
        threads_for: None,
        note: opening_note(reel),
    }
}

pub fn view_for_hall(hall: &Hall) -> ViewState {
    let _ = hall.dims;
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
        audio_level: 3,
        yaw: 0.0,
        pitch: 0.0,
        distance: 9.0,
        eye: None,
        fov: HALL_FOV,
        body_scale: BODY_SCALE,
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
    mut bindpose_assets: ResMut<Assets<SkinnedMeshInverseBindposes>>,
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
                start: 12.0,
                end: 42.0,
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
    let creature_mats = std::array::from_fn(|_| materials.add(carapace_material()));
    diffuse_lights(&mut commands);
    let orb = meshes.add(if hall.nursery {
        food_clot_mesh()
    } else {
        nutrient_mesh()
    });
    let (bindposes, mouth, mouth_mat) =
        rig_assets(&mut meshes, &mut materials, &mut bindpose_assets);
    let orb_mat = materials.add(if hall.nursery {
        StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 0.23,
            clearcoat: 0.9,
            metallic: 0.08,
            ..default()
        }
    } else {
        bile_material()
    });
    let flesh_mesh = meshes.add(nutrient_mesh());
    let mandibles = [
        meshes.add(mandible_mesh(-1.0)),
        meshes.add(mandible_mesh(1.0)),
    ];
    let flesh_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.26, 0.075, 0.085),
        metallic: 0.10,
        perceptual_roughness: 0.28,
        clearcoat: 0.85,
        ..default()
    });
    let empty = meshes.add(creature_mesh(0));
    if let Some(mut mesh) = meshes.get_mut(&empty) {
        write_threads(&mut mesh, &[], 0.01);
    }
    let threads = empty;
    let thread_mat = materials.add(filament_material());
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
        bindposes,
        mouth,
        mouth_mat,
        flesh_mat,
        flesh_mesh,
        mandibles,
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
        view.tick = hall.interpolated_tick();
    }
}

pub fn embody(
    mut commands: Commands,
    gallery: Res<Gallery>,
    mut hall: ResMut<Hall>,
    mut view: ResMut<ViewState>,
) {
    let Some(picture) = hall.latest.clone() else {
        return;
    };
    for cell in &picture.cells {
        if !hall.spawned_ids.insert(cell.id) {
            continue;
        }
        let costume = costume(cell.id, cell.genome);
        let root = commands
            .spawn((
                Mesh3d(gallery.creatures[costume.variant as usize].clone()),
                MeshMaterial3d(gallery.creature_mats[costume.variant as usize].clone()),
                Transform::default(),
                Visibility::Hidden,
                costume.body,
            ))
            .id();
        if cell.rig.is_some() {
            let mut joints = Vec::new();
            let mut parent = root;
            for index in 0..SPINE_JOINTS {
                let joint = commands
                    .spawn((
                        Transform::from_xyz(
                            0.0,
                            if index == 0 { BIND_START } else { BIND_STEP },
                            0.0,
                        ),
                        MotorJoint { id: cell.id, index },
                    ))
                    .id();
                commands.entity(parent).add_child(joint);
                joints.push(joint);
                parent = joint;
            }
            let head_bone = joints[SPINE_JOINTS - 3];
            for jaw in 0..2 {
                let sign = if jaw == 0 { -1.0 } else { 1.0 };
                let joint = commands
                    .spawn((
                        Transform::from_xyz(sign * 0.10, 0.15, 0.08),
                        MotorJoint {
                            id: cell.id,
                            index: SPINE_JOINTS + jaw,
                        },
                        Mesh3d(gallery.mandibles[jaw].clone()),
                        MeshMaterial3d(gallery.creature_mats[costume.variant as usize].clone()),
                        Visibility::Inherited,
                    ))
                    .id();
                commands.entity(head_bone).add_child(joint);
                joints.push(joint);
            }
            debug_assert_eq!(joints.len(), JOINTS);
            commands.entity(root).insert(SkinnedMesh {
                inverse_bindposes: gallery.bindposes.clone(),
                joints,
            });
            let mouth = commands
                .spawn((
                    Mesh3d(gallery.mouth.clone()),
                    MeshMaterial3d(gallery.mouth_mat.clone()),
                    Transform::from_xyz(0.0, 0.85, 0.1),
                    FeedingMaw { id: cell.id },
                ))
                .id();
            commands.entity(head_bone).add_child(mouth);
            // Feeding organ follows the articulated head.
            commands
                .entity(mouth)
                .insert(Transform::from_xyz(0.0, 0.15, 0.1));
        }
    }
    if !hall.spawned {
        if let Some(cell) = picture.cells.first() {
            let position = hall
                .poses
                .get(&cell.id)
                .map_or(Vec3::ZERO, |(_, p)| Vec3::from_array(*p));
            view.eye = Some(position + Vec3::new(0.0, 0.20, 2.2));
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
            "VIEW_READY creatures {} food colonies {}",
            picture.cells.len(),
            hall.fluid_origins.len()
        );
    }
}

pub fn hall_bodies(
    time: Res<Time>,
    hall: Res<Hall>,
    view: Res<ViewState>,
    mut commands: Commands,
    mut bodies: Query<(Entity, &Creature, &mut Transform, &mut Visibility)>,
) {
    let lead = hall.motion_blend();
    let Some(picture) = &hall.latest else {
        return;
    };
    for (entity, body, mut transform, mut visibility) in &mut bodies {
        let Ok(index) = picture.cells.binary_search_by_key(&body.id, |c| c.id) else {
            commands.entity(entity).despawn();
            continue;
        };
        let cell = &picture.cells[index];
        *visibility = if view.cells {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if let Some(span) = hall.spans.get(&body.id) {
            transform.translation = Vec3::from_array(span.translation(lead));
        } else if let Some((before, after)) = hall.poses.get(&body.id) {
            transform.translation =
                Vec3::from_array(*before).lerp(Vec3::from_array(*after), lead.min(1.0));
        }
        if let Some(rig) = &cell.rig {
            if let Some(state) = &rig.neural {
                let (rotation, center) = if let Some(span) = hall.spans.get(&body.id) {
                    let rotation = neural_orientation(span.orientation(lead));
                    let center = Vec3::from_array(span.centroid(lead)) / 1024.0 * layout::SPACING;
                    (rotation, center)
                } else {
                    let rotation = neural_orientation(state.body.orientation.map(|v| v as f32));
                    let center = Vec3::new(
                        state.body.centroid[0] as f32,
                        state.body.centroid[1] as f32,
                        state.body.centroid[2] as f32,
                    ) / 1024.0
                        * layout::SPACING;
                    (rotation, center)
                };
                transform.rotation = rotation;
                transform.translation -= rotation * center;
                transform.scale = Vec3::splat(BODY_SCALE);
                continue;
            }
            let velocity = Vec3::new(
                rig.velocity[0] as f32,
                rig.velocity[1] as f32,
                rig.velocity[2] as f32,
            );
            let direction = if usize::from(rig.last_action) >= DIG_START {
                let (x, y, z) = action_dir(usize::from(rig.last_action))
                    .expect("valid motor action")
                    .delta();
                Vec3::new(x as f32, y as f32, z as f32)
            } else {
                velocity
            };
            if direction.length_squared() > 0.5 {
                let facing = Quat::from_rotation_arc(Vec3::Y, direction.normalize());
                transform.rotation = transform
                    .rotation
                    .slerp(facing, 1.0 - (-time.delta_secs() * 4.0).exp());
            }
            let size = if rig.parent > 0 {
                (0.65 + rig.updates as f32 / 128.0).min(1.0)
            } else {
                1.0
            };
            let nutrition = 0.88 + 0.12 * (cell.energy as f32 / 40.0).min(1.0);
            transform.scale = body.stretch * (view.body_scale * size * nutrition);
        } else {
            transform.scale = body.stretch * view.body_scale;
        }
    }
}

fn neural_orientation(angles: [f32; 3]) -> Quat {
    let o = angles.map(|angle| angle / 1024.0);
    Quat::from_rotation_z(o[2]) * Quat::from_rotation_y(o[1]) * Quat::from_rotation_x(o[0])
}

pub fn rig_joints(
    time: Res<Time>,
    hall: Res<Hall>,
    mut joints: Query<(&MotorJoint, &mut Transform), Without<Creature>>,
) {
    let lead = hall.motion_blend();
    let Some(picture) = &hall.latest else {
        return;
    };
    for (joint, mut transform) in &mut joints {
        let Ok(index) = picture.cells.binary_search_by_key(&joint.id, |c| c.id) else {
            continue;
        };
        let Some(rig) = &picture.cells[index].rig else {
            continue;
        };
        if let Some(state) = &rig.neural {
            let mut bend = 0.0;
            let mut twist = 0.0;
            for (index, binding) in state
                .genome
                .anatomy
                .axes
                .iter()
                .enumerate()
                .filter(|(_, b)| b.joint == joint.index)
            {
                let angle = hall
                    .spans
                    .get(&joint.id)
                    .filter(|span| span.angles_to.len() == state.body.angles.len())
                    .map(|span| span.angle(index, lead) / 1024.0)
                    .unwrap_or(state.body.angles[index] as f32 / 1024.0);
                match binding.axis {
                    moebius_core::neural::schema::Axis::Twist => twist = angle,
                    _ => bend = angle,
                }
            }
            let target = if joint.index < SPINE_JOINTS {
                Quat::from_rotation_z(bend) * Quat::from_rotation_y(twist)
            } else {
                Quat::from_rotation_z(if joint.index == SPINE_JOINTS {
                    -bend
                } else {
                    bend
                })
            };
            // The span already moves at a steady rate across the committed step.
            // An extra ease would arrive early and then wait, which is the tick.
            transform.rotation = target;
            continue;
        }
        let angle = rig.joints[joint.index] as f32 / 256.0;
        let target = if joint.index < SPINE_JOINTS {
            let twist = rig.twists[joint.index] as f32 / 256.0;
            Quat::from_rotation_z(angle)
                * Quat::from_rotation_x(angle * 0.55)
                * Quat::from_rotation_y(twist)
        } else {
            let sign = if joint.index == SPINE_JOINTS {
                -1.0
            } else {
                1.0
            };
            Quat::from_rotation_z(sign * angle) * Quat::from_rotation_x(angle * 0.35)
        };
        transform.rotation = transform
            .rotation
            .slerp(target, 1.0 - (-time.delta_secs() * 3.2).exp());
    }
}

pub fn feeding(
    time: Res<Time>,
    hall: Res<Hall>,
    mut hold: Local<BTreeMap<u64, f32>>,
    mut mouths: Query<(&FeedingMaw, &mut Transform), Without<Creature>>,
) {
    let Some(picture) = &hall.latest else {
        return;
    };
    let now = time.elapsed_secs();
    for (maw, mut transform) in &mut mouths {
        let Ok(index) = picture.cells.binary_search_by_key(&maw.id, |c| c.id) else {
            continue;
        };
        let Some(rig) = &picture.cells[index].rig else {
            continue;
        };
        if rig.last_meal > 0 {
            // One committed bite is a single tick. Hold the gulp so it reads.
            hold.insert(maw.id, now + 0.32);
        }
        let target = if hold.get(&maw.id).is_some_and(|until| now < *until) {
            let chew = (now * 22.0).sin();
            let open = chew.max(0.0);
            let shut = (-chew).max(0.0);
            Vec3::new(1.35 + open * 0.7, 0.28 + shut * 0.22, 1.35 + open * 0.7)
        } else {
            Vec3::new(0.72, 0.62, 0.72)
        };
        transform.scale = transform
            .scale
            .lerp(target, 1.0 - (-time.delta_secs() * 14.0).exp());
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
        if hall.nursery {
            *visibility = Visibility::Visible;
            transform.translation = Vec3::from_array(site_to_view(origin, hall.dims));
            // The clot is one exact stock site. Volume follows its committed amount.
            transform.scale = Vec3::splat(layout::SPACING * 1.6 * fraction.cbrt());
            transform.rotation = Quat::from_rotation_z((*index as f32 * 2.399).sin() * 0.55);
            continue;
        }
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
            if let Some(mut mesh) = meshes.get_mut(&gallery.threads) {
                write_slime(&mut mesh, &hall.motion_paths, shown);
            }
            view.threads_for = Some(shown);
        }
        *visibility = Visibility::Visible;
    } else {
        *visibility = Visibility::Hidden;
    }
}

pub fn hall_sound(
    time: Res<Time>,
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
        bucket: (time.elapsed_secs_f64() / 8.0) as u64,
        muted: view.muted || !view.playing || view.audio_level == 0,
        level: view.audio_level,
    };
    if view.audio_for == Some(key) {
        return;
    }
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    if !key.muted {
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
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.022 * view.audio_level as f32)),
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
    let sound = if view.muted {
        "muted".into()
    } else {
        format!("bursts {}/10", view.audio_level)
    };
    let hash: String = picture.state_hash.chars().take(16).collect();
    let flesh = picture.material_flesh;
    let generation = picture
        .cells
        .iter()
        .filter_map(|c| c.rig.as_ref().map(|r| r.generation))
        .max()
        .unwrap_or(0);
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
        "Möbius Automata — Farm M1   tick {}   shown {:.1}   {fps}   {play}\n\
         farm. founding cells {}. mode is identified in the neural inspector (N).\n\
         seed {}   config {config}   code {code}   energy cap {}\n\
         live {}   dead {}   field {}   reservoir {}   waste {}\n\
         material residual {}   action-energy residual {}   state {hash} at tick {}\n\
         generation {generation}   deposited flesh {flesh}   N: neural inspector / Tab: pages\n\
         food colonies: block samples, stride {}. mouths respond to actual nutrient uptake.\n\
         1 bodies {}   2 fluid {}   3 slime {}   4 hypothesis {} (empty scaffold, not membership)\n\
         {extra}\
         space pauses the scientific clock. closing the window stops it.\n\
         audio {sound}: viewer pcm, not in the state hash. WASD move  Q/E rise/fall  click: mouse look  Esc: release  wheel: zoom  F inspect nearest  M mute  [/] volume",
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
    Reel(Box<Reel>),
    Hall(Box<Hall>),
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
        0.78 + (genome[2] as f32 / 255.0) * 0.25 + ((id >> 3) % 5) as f32 * 0.02,
        0.9 + (genome[3] as f32 / 255.0) * 0.25 + ((id >> 5) % 7) as f32 * 0.02,
        0.78 + (genome[4] as f32 / 255.0) * 0.25 + ((id >> 7) % 5) as f32 * 0.02,
    );
    let spin_mag = 0.06 + ((mix >> 8) % 180) as f32 / 180.0 * 0.16;
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

fn secretion_scale(size: Vec3, fraction: f32, pulse: f32) -> Vec3 {
    // Volume, rather than radius, tracks the normalized block nutrient amount.
    Vec3::splat(size.max_element() * 0.12 * fraction.cbrt() * pulse)
}

fn bile_material() -> StandardMaterial {
    StandardMaterial {
        base_color: Color::srgba(0.30, 0.52, 0.13, 0.24),
        emissive: LinearRgba::new(0.10, 0.22, 0.025, 1.0),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.65,
        metallic: 0.0,
        reflectance: 0.25,
        ..default()
    }
}

fn carapace_material() -> StandardMaterial {
    StandardMaterial {
        base_color: Color::WHITE,
        metallic: 0.38,
        perceptual_roughness: 0.34,
        clearcoat: 0.65,
        clearcoat_perceptual_roughness: 0.2,
        ..default()
    }
}

fn filament_material() -> StandardMaterial {
    StandardMaterial {
        base_color: Color::srgba(0.75, 0.88, 0.52, 0.65),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.16,
        metallic: 0.12,
        clearcoat: 0.85,
        emissive: LinearRgba::new(0.025, 0.04, 0.012, 1.0),
        ..default()
    }
}

fn diffuse_lights(commands: &mut Commands) {
    // Broad fill reaches the whole hall instead of falling off around a lamp.
    for (color, illuminance, at) in [
        (
            Color::srgb(0.82, 0.90, 1.0),
            2_000.0,
            Vec3::new(1.0, 2.0, 1.0),
        ),
        (
            Color::srgb(1.0, 0.83, 0.64),
            900.0,
            Vec3::new(-2.0, 0.5, -1.0),
        ),
    ] {
        commands.spawn((
            DirectionalLight {
                color,
                illuminance,
                shadow_maps_enabled: false,
                ..default()
            },
            Transform::from_translation(at).looking_at(Vec3::ZERO, Vec3::Y),
        ));
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

#[cfg(test)]
mod camera_tests {
    use super::*;

    #[test]
    fn fly_camera_responds_to_keyboard_mouse_zoom_and_escape() {
        let mut config = moebius_core::SimConfig::m0_64();
        config.width = 4;
        config.height = 4;
        config.depth = 4;
        config.initial_population = 1;
        config.inject_per_tick = 1;
        let hall = live::open(&config, 7).unwrap();
        let mut app = App::new();
        let mut clock = Time::<()>::default();
        clock.advance_by(std::time::Duration::from_millis(50));
        app.insert_resource(clock)
            .insert_resource(view_for_hall(&hall))
            .init_resource::<FlyCamera>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<AccumulatedMouseMotion>()
            .init_resource::<AccumulatedMouseScroll>()
            .add_systems(Update, orbit);
        app.world_mut().spawn((
            Window {
                focused: true,
                ..default()
            },
            CursorOptions::default(),
            PrimaryWindow,
        ));
        app.world_mut().spawn((
            Camera3d::default(),
            Transform::default(),
            Projection::default(),
        ));
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyW);
        app.update();
        assert!(app.world().resource::<ViewState>().eye.unwrap().z < 9.0);
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.world_mut()
            .resource_mut::<AccumulatedMouseMotion>()
            .delta = Vec2::new(40.0, -20.0);
        app.world_mut()
            .resource_mut::<AccumulatedMouseScroll>()
            .delta = Vec2::new(0.0, 2.0);
        app.update();
        let view = app.world().resource::<ViewState>();
        assert!(view.yaw < 0.0 && view.pitch > 0.0);
        assert!(view.fov < HALL_FOV);
        assert!(app.world().resource::<FlyCamera>().captured);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();
        assert!(!app.world().resource::<FlyCamera>().captured);
    }
}
