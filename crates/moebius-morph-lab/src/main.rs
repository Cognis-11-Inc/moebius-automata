//! Native laboratory. Generated anatomy is a preview, never a simulation controller.
use bevy::{
    asset::RenderAssetUsages,
    camera::visibility::NoFrustumCulling,
    input::mouse::{AccumulatedMouseMotion, MouseWheel},
    mesh::Indices,
    prelude::*,
    render::{
        render_resource::PrimitiveTopology,
        view::screenshot::{Screenshot, save_to_disk},
    },
};
use clap::Parser;
use moebius_morph::{AnatomySpec, Organism, Shape, Surface};
use moebius_morph_lab::{
    ComponentField, CompositionSession, Edit, PreviewCommand, PreviewControls, world_point,
};
use std::path::PathBuf;
#[derive(Parser)]
struct Cli {
    #[arg(long, default_value_t = 20261123)]
    seed: u64,
    #[arg(long, default_value = "runs/morph_002/organism.json")]
    definition: PathBuf,
    /// New evidence directory; drives real keyboard handlers and records internal screenshots.
    #[arg(long)]
    walkthrough: Option<PathBuf>,
}
#[derive(Resource)]
struct Lab(CompositionSession);
#[derive(Resource)]
struct Emission(PreviewControls);
#[derive(Resource, Default)]
struct Rendered {
    geometry: Option<Organism>,
    handles: Vec<(Entity, Handle<Mesh>)>,
    material: Handle<StandardMaterial>,
}
#[derive(Resource)]
struct Orbit {
    yaw: f32,
    pitch: f32,
    distance: f32,
}
#[derive(Component)]
struct CameraTag;
#[derive(Component)]
struct CreatureMesh;
#[derive(Component)]
enum UiText {
    Controls,
    Status,
    Specimen,
}
#[derive(Resource)]
struct Walk {
    out: PathBuf,
    frame: u64,
    saved_digest: Option<String>,
    saved_spec: Option<AnatomySpec>,
    events: Vec<serde_json::Value>,
    failure: Option<String>,
    captures: usize,
}
fn main() {
    let cli = Cli::parse();
    let path = cli
        .walkthrough
        .as_ref()
        .map_or(cli.definition.clone(), |p| p.join("saved_spec.json"));
    if let Some(p) = &cli.walkthrough {
        std::fs::create_dir(p).expect("walkthrough requires a new directory");
    }
    let spec = if path.exists() {
        moebius_morph::load_anatomy(&path).expect("valid definition file")
    } else {
        AnatomySpec::showcase(cli.seed)
    };
    let preview =
        PreviewControls::from_spec(&spec, PathBuf::from("runs/morph_003/preview_book.json"))
            .expect("valid material book");
    let session = CompositionSession::new(spec, path).expect("valid initial anatomy");
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "MOEBIUS — Anatomical Forge / MORPH-003".into(),
            resolution: (1600, 1000).into(),
            ..default()
        }),
        ..default()
    }))
    .insert_resource(ClearColor(Color::srgb(0.012, 0.018, 0.024)))
    .insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.72, 0.79, 0.88),
        brightness: 180.,
        ..default()
    })
    .insert_resource(Lab(session))
    .insert_resource(Emission(preview))
    .insert_resource(Orbit {
        yaw: 0.98,
        pitch: 0.29,
        distance: 7.8,
    })
    .init_resource::<Rendered>()
    .add_systems(Startup, setup)
    .add_systems(
        Update,
        (
            inject_walk,
            keys,
            emission_keys,
            advance,
            simulate,
            render_geometry,
            place_creature,
            orbit,
            update_text,
            record_walk,
        )
            .chain(),
    );
    moebius_morph_lab::register_ambient(&mut app, cli.walkthrough.is_none());
    if let Some(out) = cli.walkthrough {
        app.insert_resource(Walk {
            out,
            frame: 0,
            saved_digest: None,
            saved_spec: None,
            events: vec![],
            failure: None,
            captures: 0,
        });
    }
    app.run();
}
fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut rendered: ResMut<Rendered>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(6., 3., 4.).looking_at(Vec3::ZERO, Vec3::Y),
        CameraTag,
    ));
    for (position, power, color) in [
        (Vec3::new(4., 7., 5.), 4500., Color::srgb(1., 0.83, 0.70)),
        (Vec3::new(-5., 3., -4.), 2300., Color::srgb(0.48, 0.67, 1.)),
    ] {
        commands.spawn((
            DirectionalLight {
                illuminance: power,
                color,
                shadow_maps_enabled: true,
                ..default()
            },
            Transform::from_translation(position).looking_at(Vec3::ZERO, Vec3::Y),
        ));
    }
    rendered.material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.36,
        reflectance: 0.42,
        ..default()
    });
    commands.spawn((Node {position_type:PositionType::Absolute,left:Val::Px(20.),top:Val::Px(20.),bottom:Val::Px(20.),width:Val::Px(326.),padding:UiRect::all(Val::Px(18.)),flex_direction:FlexDirection::Column,row_gap:Val::Px(18.),..default()},BackgroundColor(Color::srgba(0.018,0.027,0.034,0.96)))).with_children(|p| {
        p.spawn((Text::new("MOEBIUS\nANATOMICAL FORGE"),TextFont {font_size: FontSize::Px(25.),..default()},TextColor(Color::srgb(0.89,0.84,0.70))));
        p.spawn((Text::new("MORPH-003 / SECRETION\nFree-space material book"),TextFont {font_size: FontSize::Px(14.),..default()},TextColor(Color::srgb(0.42,0.73,0.70))));
        p.spawn((Text::new("Compiling anatomy…"),TextFont {font_size: FontSize::Px(14.),..default()},TextColor(Color::srgb(0.88,0.87,0.80)),UiText::Controls));
        p.spawn((Text::new("Tab: next component (Shift: previous)\nA: flagellum  P: spine  N: nozzle\nD: duplicate  Delete: remove\nUp/Down: parameter  Left/Right: edit\n[/]: body length  -/=: thickness\n, / .: mouth aperture  M: cycle\nH: short-body showcase  R: seed  F8: mute drone\nS: save anatomy  L: reload anatomy\nE: emission  C: material clock\nU/I: rate  J/K: reset reservoir\nF/V: drag  G: weak +X accel\nO: medium flow  Q: trajectory\nX: reset book  F5/F9: book save/load\nSpace: pause the active clock\nRight-drag / wheel or Alt+arrows: orbit"),TextFont {font_size: FontSize::Px(14.),..default()},TextColor(Color::srgb(0.56,0.64,0.66))));
        p.spawn((Text::new("Anatomy does not grant emission.\nTrail lines are visual only, not pipes.\nNo floor. No locomotion claim."),TextFont {font_size: FontSize::Px(13.),..default()},TextColor(Color::srgb(0.68,0.49,0.39))));
    });
    commands.spawn((
        Text::new("WORM / compiling"),
        TextFont {
            font_size: FontSize::Px(16.),
            ..default()
        },
        TextColor(Color::srgb(0.57, 0.80, 0.75)),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(28.),
            top: Val::Px(28.),
            ..default()
        },
        UiText::Specimen,
    ));
    commands.spawn((
        Text::new("Local laboratory"),
        TextFont {
            font_size: FontSize::Px(15.),
            ..default()
        },
        TextColor(Color::srgb(0.74, 0.81, 0.76)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(378.),
            bottom: Val::Px(25.),
            right: Val::Px(25.),
            ..default()
        },
        UiText::Status,
    ));
}
fn keys(
    input: Res<ButtonInput<KeyCode>>,
    mut lab: ResMut<Lab>,
    mut mix: ResMut<moebius_morph_lab::AmbientMix>,
) {
    moebius_morph_lab::ambient_keys(&input, &mut mix, KeyCode::F8, false);
    if input.pressed(KeyCode::AltLeft) || input.pressed(KeyCode::AltRight) {
        return;
    }
    let mut edits = vec![];
    let i = ComponentField::ALL
        .iter()
        .position(|f| *f == lab.0.field)
        .unwrap();
    if input.just_pressed(KeyCode::ArrowDown) {
        edits.push(Edit::Field(i + 1));
    }
    if input.just_pressed(KeyCode::ArrowUp) {
        edits.push(Edit::Field((i + 11) % 12));
    }
    if input.just_pressed(KeyCode::Tab) {
        edits.push(if input.pressed(KeyCode::ShiftLeft) {
            Edit::SelectPrevious
        } else {
            Edit::SelectNext
        });
    }
    let step = if input.pressed(KeyCode::ShiftLeft) {
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
        (KeyCode::KeyL, Edit::Reload),
    ] {
        if input.just_pressed(key) {
            edits.push(edit);
        }
    }
    if input.just_pressed(KeyCode::KeyM) {
        lab.0.mouth_cycle = !lab.0.mouth_cycle;
    }
    for e in edits {
        if let Err(error) = lab.0.edit(e) {
            lab.0.status = format!("REFUSED: {error}");
            eprintln!("MORPH_EDIT_REFUSED {e:?}: {error}");
        }
    }
}
fn emission_keys(
    input: Res<ButtonInput<KeyCode>>,
    mut lab: ResMut<Lab>,
    mut emission: ResMut<Emission>,
) {
    if input.pressed(KeyCode::AltLeft) || input.pressed(KeyCode::AltRight) {
        return;
    }
    if input.just_pressed(KeyCode::Space) {
        if emission.0.clock_armed {
            emission.0.clock = !emission.0.clock;
            emission.0.note = if emission.0.clock {
                "Material clock running.".into()
            } else {
                "Material clock paused.".into()
            };
        } else {
            lab.0.animation = !lab.0.animation;
        }
    }
    let shift = input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight);
    for key in [
        KeyCode::KeyE,
        KeyCode::KeyC,
        KeyCode::KeyU,
        KeyCode::KeyI,
        KeyCode::KeyJ,
        KeyCode::KeyK,
        KeyCode::KeyF,
        KeyCode::KeyV,
        KeyCode::KeyG,
        KeyCode::KeyO,
        KeyCode::KeyQ,
        KeyCode::KeyX,
        KeyCode::F5,
        KeyCode::F9,
    ] {
        if input.just_pressed(key) {
            let Some(command) = preview_command(key, shift) else {
                continue;
            };
            if matches!(command, PreviewCommand::Reset) {
                lab.0.phase = 0.;
            }
            if let Err(error) = emission.0.command(&lab.0.spec, command) {
                lab.0.status = format!("MATERIAL REFUSED: {error}");
            }
        }
    }
}
fn preview_command(key: KeyCode, shift: bool) -> Option<PreviewCommand> {
    let step = if shift { 10 } else { 1 };
    Some(match key {
        KeyCode::KeyE => PreviewCommand::ToggleEmission,
        KeyCode::KeyC => PreviewCommand::ToggleClock,
        KeyCode::KeyU => PreviewCommand::Rate(-step),
        KeyCode::KeyI => PreviewCommand::Rate(step),
        KeyCode::KeyJ => PreviewCommand::StageReservoir {
            numerator: 1,
            denominator: 2,
        },
        KeyCode::KeyK => PreviewCommand::StageReservoir {
            numerator: 2,
            denominator: 1,
        },
        KeyCode::KeyF => PreviewCommand::Drag(-50),
        KeyCode::KeyV => PreviewCommand::Drag(50),
        KeyCode::KeyG => PreviewCommand::ToggleAcceleration,
        KeyCode::KeyO => PreviewCommand::ToggleFlow,
        KeyCode::KeyQ => PreviewCommand::ToggleTrajectory,
        KeyCode::KeyX => PreviewCommand::Reset,
        KeyCode::F5 => PreviewCommand::Save,
        KeyCode::F9 => PreviewCommand::Load,
        _ => return None,
    })
}
fn advance(time: Res<Time>, emission: Res<Emission>, mut lab: ResMut<Lab>) {
    if emission.0.clock {
        return;
    }
    if lab.0.animation {
        lab.0.phase = (lab.0.phase + time.delta_secs() * 0.9).min(99999.);
        if lab.0.phase >= 99999. {
            lab.0.animation = false;
            lab.0.status = "Preview time limit reached; reload session to restart".into();
        }
    }
}
fn mesh(surface: &Surface) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, surface.positions.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, surface.normals.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, surface.uvs.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, surface.colors.clone());
    mesh.insert_indices(Indices::U32(surface.indices.clone()));
    mesh
}
fn render_geometry(
    lab: Res<Lab>,
    mut rendered: ResMut<Rendered>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let geometry = match lab.0.geometry() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("MORPH_POSE_REFUSED {e}");
            return;
        }
    };
    if rendered.handles.len() != geometry.surfaces.len() {
        for (entity, handle) in rendered.handles.drain(..) {
            commands.entity(entity).despawn();
            meshes.remove(handle.id());
        }
        let material = rendered.material.clone();
        for s in &geometry.surfaces {
            let handle = meshes.add(mesh(s));
            let entity = commands
                .spawn((
                    Mesh3d(handle.clone()),
                    MeshMaterial3d(material.clone()),
                    Transform::default(),
                    NoFrustumCulling,
                    CreatureMesh,
                ))
                .id();
            rendered.handles.push((entity, handle));
        }
    } else {
        for ((_, handle), surface) in rendered.handles.iter().zip(&geometry.surfaces) {
            if let Some(mut m) = meshes.get_mut(handle) {
                *m = mesh(surface);
            }
        }
    }
    if let Some(mut m) = materials.get_mut(&rendered.material) {
        m.perceptual_roughness = lab.0.spec.body.appearance.roughness;
    }
    rendered.geometry = Some(geometry);
}
fn orbit(
    time: Res<Time>,
    motion: Res<AccumulatedMouseMotion>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut wheel: MessageReader<MouseWheel>,
    mut orbit: ResMut<Orbit>,
    mut camera: Query<&mut Transform, With<CameraTag>>,
) {
    if mouse.pressed(MouseButton::Right) {
        orbit.yaw -= motion.delta.x * 0.006;
        orbit.pitch = (orbit.pitch + motion.delta.y * 0.005).clamp(-1.1, 1.1);
    }
    if keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight) {
        let dt = time.delta_secs() * 1.5;
        orbit.yaw += dt
            * (keys.pressed(KeyCode::ArrowRight) as i32 - keys.pressed(KeyCode::ArrowLeft) as i32)
                as f32;
        orbit.pitch = (orbit.pitch
            + dt * (keys.pressed(KeyCode::ArrowUp) as i32 - keys.pressed(KeyCode::ArrowDown) as i32)
                as f32)
            .clamp(-1.1, 1.1);
    }
    for e in wheel.read() {
        orbit.distance = (orbit.distance - e.y * 0.35).clamp(3., 18.);
    }
    let target = Vec3::new(-0.45, 0., 0.);
    let pos = Vec3::new(
        orbit.yaw.sin() * orbit.pitch.cos(),
        orbit.pitch.sin(),
        orbit.yaw.cos() * orbit.pitch.cos(),
    ) * orbit.distance;
    if let Ok(mut t) = camera.single_mut() {
        *t = Transform::from_translation(target + pos).looking_at(target, Vec3::Y);
    }
}
fn simulate(time: Res<Time>, mut lab: ResMut<Lab>, mut emission: ResMut<Emission>) {
    if !emission.0.clock {
        return;
    }
    emission.0.accumulator += time.delta_secs();
    let mut steps = 0;
    while emission.0.accumulator >= emission.0.dt && steps < 4 {
        emission.0.accumulator -= emission.0.dt;
        if let Err(error) = emission.0.begin_tick() {
            lab.0.status = format!("MATERIAL STEP REFUSED: {error}");
            emission.0.clock = false;
            return;
        }
        lab.0.phase = PreviewControls::phase_for_tick(emission.0.book.tick);
        let pose = lab.0.pose();
        let selected = lab.0.selected;
        if let Err(error) = emission.0.emit_current(&lab.0.compiled, pose, selected) {
            lab.0.status = format!("MATERIAL EMIT REFUSED: {error}");
            emission.0.clock = false;
            return;
        }
        steps += 1;
    }
    if steps == 4 && emission.0.accumulator >= emission.0.dt {
        emission.0.accumulator = 0.;
        emission.0.note = "Preview dropped stalled ticks. The benchmark clock does not.".into();
    }
}
fn body_transform(emission: &PreviewControls) -> Transform {
    let translation = Vec3::from_array(world_point(emission.body_rigid.translation));
    let yaw = (emission.body_rigid.yaw_sin as f64).atan2(emission.body_rigid.yaw_cos as f64) as f32;
    Transform::from_translation(translation).with_rotation(Quat::from_rotation_y(yaw))
}
fn place_creature(emission: Res<Emission>, mut bodies: Query<&mut Transform, With<CreatureMesh>>) {
    let transform = body_transform(&emission.0);
    for mut body in &mut bodies {
        *body = transform;
    }
}
fn update_text(
    lab: Res<Lab>,
    emission: Res<Emission>,
    rendered: Res<Rendered>,
    mut text: Query<(&UiText, &mut Text)>,
    mut gizmos: Gizmos,
) {
    let s = &lab.0.spec;
    let component = lab.0.component();
    let selected = component.map_or("No component selected".into(), |c| {
        format!("{} #{}  / parent trunk0", c.kind(), c.id)
    });
    let controls = format!(
        "{} components\n{}\n\n{}",
        s.components.len(),
        selected,
        ComponentField::ALL
            .iter()
            .map(|f| format!(
                "{} {}: {}",
                if *f == lab.0.field { ">" } else { " " },
                f.label(),
                component
                    .and_then(|c| f.value(c))
                    .map_or("--".into(), |v| format!("{v:.3}"))
            ))
            .collect::<Vec<_>>()
            .join("\n")
    );
    let body = body_transform(&emission.0);
    if let Some(g) = &rendered.geometry {
        for (c, a) in s.components.iter().zip(&g.attachments) {
            let color = if Some(c.id) == lab.0.selected {
                Color::srgb(1., 0.65, 0.15)
            } else {
                Color::srgb(0.2, 0.8, 0.65)
            };
            gizmos.sphere(
                Isometry3d::from_translation(body.transform_point(Vec3::from_array(a.origin))),
                0.045,
                color,
            );
        }
    }
    // Each vertex is a parcel coordinate from the book. The segment between
    // consecutive parcels of one nozzle is a visual aid, not a body or a pipe.
    let mut previous: Option<(u64, Vec3)> = None;
    for parcel in &emission.0.book.parcels {
        let point = Vec3::from_array(world_point(parcel.position));
        if let Some((nozzle, prior)) = previous
            && nozzle == parcel.nozzle_id
        {
            gizmos.line(prior, point, Color::srgba(0.96, 0.58, 0.24, 0.9));
        }
        previous = Some((parcel.nozzle_id, point));
    }
    if let Some(parcel) = emission.0.book.parcels.last() {
        gizmos.sphere(
            Isometry3d::from_translation(Vec3::from_array(world_point(parcel.position))),
            0.04,
            Color::srgb(1., 0.82, 0.48),
        );
    }
    for (role, mut text) in &mut text {
        text.0 = match role {
            UiText::Controls => controls.clone(),
            UiText::Status => {
                if let Some(g) = &rendered.geometry {
                    format!(
                        "{}\n{} vertices | {} triangles | body {:.2} x {:.2} | mouth {:.0}%\n{}\nTrail lines are visual only. They are not pipes.",
                        lab.0.status,
                        g.surfaces.iter().map(|s| s.positions.len()).sum::<usize>(),
                        g.surfaces
                            .iter()
                            .map(|s| s.indices.len() / 3)
                            .sum::<usize>(),
                        s.body.length,
                        s.body.radius,
                        lab.0.pose().mouth * 100.,
                        emission.0.ledger_line()
                    )
                } else {
                    "Compiling".into()
                }
            }
            UiText::Specimen => {
                let nozzle = lab
                    .0
                    .selected
                    .filter(|id| emission.0.book.registered.contains(id))
                    .or_else(|| emission.0.book.registered.iter().next().copied());
                let admitted = nozzle.is_some_and(|id| emission.0.book.runtime_operational(id));
                let anatomical = s
                    .components
                    .iter()
                    .find_map(|component| match &component.shape {
                        Shape::SecretionNozzle { output, .. } => Some(output.operational),
                        _ => None,
                    });
                format!(
                    "COMPOSABLE WORM / seed {}\nruntime nozzle {} | anatomical operational {}",
                    s.body.seed,
                    if admitted {
                        "admitted"
                    } else {
                        "not yet admitted"
                    },
                    anatomical.map_or("no nozzle", |flag| if flag { "true" } else { "false" })
                )
            }
        };
    }
}
fn inject_walk(walk: Option<ResMut<Walk>>, mut input: ResMut<ButtonInput<KeyCode>>) {
    let Some(mut walk) = walk else {
        return;
    };
    walk.frame += 1;
    input.reset_all();
    let f = walk.frame;
    let key = match f {
        100 => Some(KeyCode::Space),
        102 => Some(KeyCode::KeyM),
        104..=122 if f % 2 == 0 => Some(KeyCode::Comma),
        144..=162 if f % 2 == 0 => Some(KeyCode::Period),
        180 => Some(KeyCode::KeyA),
        182 => Some(KeyCode::ArrowRight),
        184 => Some(KeyCode::KeyD),
        186 => Some(KeyCode::ArrowDown),
        188 => Some(KeyCode::ArrowDown),
        190 => Some(KeyCode::ArrowRight),
        192 => Some(KeyCode::Delete),
        194 => Some(KeyCode::KeyP),
        196 => Some(KeyCode::KeyN),
        198 => Some(KeyCode::ArrowRight),
        200 => Some(KeyCode::BracketLeft),
        202 => Some(KeyCode::KeyS),
        220 => Some(KeyCode::KeyR),
        240 => Some(KeyCode::KeyL),
        260 => Some(KeyCode::Tab),
        _ => None,
    };
    if let Some(key) = key {
        input.press(key);
        walk.events
            .push(serde_json::json!({"frame":f,"keyboard":format!("{key:?}")}));
    }
    if (290..=330).contains(&f) {
        input.press(KeyCode::AltLeft);
        input.press(KeyCode::ArrowRight);
    }
}
fn record_walk(
    walk: Option<ResMut<Walk>>,
    lab: Res<Lab>,
    rendered: Res<Rendered>,
    orbit: Res<Orbit>,
    mut commands: Commands,
    mut exit: MessageWriter<AppExit>,
) {
    let Some(mut walk) = walk else {
        return;
    };
    let Some(g) = &rendered.geometry else {
        return;
    };
    let frame = walk.frame;
    let mut tag = match frame {
        30 => Some("showcase_early"),
        80 => Some("showcase_later"),
        132 => Some("mouth_closed"),
        172 => Some("mouth_open"),
        204 => Some("composed_saved"),
        230 => Some("seed_variant"),
        250 => Some("composed_reloaded"),
        270 => Some("selected_component"),
        338 => Some("orbited"),
        _ => None,
    }
    .map(str::to_string);
    if tag.is_none() && (12..100).contains(&frame) && frame.is_multiple_of(8) {
        tag = Some(format!("motion_{frame:04}"));
    }
    if frame == 204 {
        walk.saved_digest = Some(g.digest());
        walk.saved_spec = Some(lab.0.spec.clone());
    }
    if frame == 250
        && (walk.saved_digest.as_deref() != Some(g.digest().as_str())
            || walk.saved_spec.as_ref() != Some(&lab.0.spec))
    {
        walk.failure = Some("save/reload mismatch".into());
    }
    if let Some(tag) = tag {
        walk.captures += 1;
        let audit = g.audit();
        if let Err(e) = &audit {
            walk.failure = Some(format!("invalid captured geometry: {e}"));
        }
        let path = walk.out.join(&tag);
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path.with_extension("png")));
        let data = serde_json::json!({"frame":frame,"specification":lab.0.spec,"pose":lab.0.pose(),"geometry_digest":g.digest(),"topology_digest":g.topology_digest(),"geometry_audit":audit.map_err(|e|e.to_string()),"attachments":g.attachments,"camera":{"yaw":orbit.yaw,"pitch":orbit.pitch,"distance":orbit.distance},"authored_preview_not_learning":true});
        std::fs::write(
            path.with_extension("json"),
            serde_json::to_vec_pretty(&data).unwrap(),
        )
        .expect("write capture provenance");
        println!("MORPH_CAPTURE {tag} frame{frame}");
    }
    if frame == 350 {
        let report = serde_json::json!({"status":if walk.failure.is_none(){"PASS"}else{"FAIL"},"failure":walk.failure,"saved_reloaded_geometry_matches":walk.failure.is_none(),"saved_digest":walk.saved_digest,"keyboard_events":walk.events,"screenshots":walk.captures,"note":"Real Bevy generated meshes; keyboard injection uses ordinary editor handlers. Mouse gestures are implemented; automated orbit uses Alt+arrow."});
        std::fs::write(
            walk.out.join("walkthrough.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .expect("write walkthrough report");
        println!("MORPH_WALK_COMPLETE {report}");
        exit.write(if walk.failure.is_none() {
            AppExit::Success
        } else {
            AppExit::error()
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_mouse_orbit_and_wheel_handlers_move_a_finite_camera() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<AccumulatedMouseMotion>()
            .insert_resource(Orbit {
                yaw: 0.98,
                pitch: 0.29,
                distance: 7.8,
            })
            .add_message::<MouseWheel>()
            .add_systems(Update, orbit);
        let camera = app
            .world_mut()
            .spawn((Transform::default(), CameraTag))
            .id();
        app.update();
        let before = app.world().get::<Transform>(camera).unwrap().translation;
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Right);
        app.world_mut()
            .resource_mut::<AccumulatedMouseMotion>()
            .delta = Vec2::new(120., -80.);
        app.world_mut().write_message(MouseWheel {
            unit: bevy::input::mouse::MouseScrollUnit::Line,
            x: 0.,
            y: 2.,
            window: Entity::PLACEHOLDER,
            phase: bevy::input::touch::TouchPhase::Moved,
        });
        app.update();
        let after = app.world().get::<Transform>(camera).unwrap().translation;
        assert_ne!(before, after);
        assert!(after.is_finite());
        assert!(app.world().resource::<Orbit>().distance < 7.8);
    }

    #[test]
    fn material_keys_cover_the_operator_controls() {
        assert_eq!(
            preview_command(KeyCode::KeyE, false),
            Some(PreviewCommand::ToggleEmission)
        );
        assert_eq!(
            preview_command(KeyCode::KeyC, false),
            Some(PreviewCommand::ToggleClock)
        );
        assert_eq!(
            preview_command(KeyCode::KeyI, true),
            Some(PreviewCommand::Rate(10))
        );
        assert_eq!(
            preview_command(KeyCode::KeyJ, false),
            Some(PreviewCommand::StageReservoir {
                numerator: 1,
                denominator: 2
            })
        );
        assert_eq!(
            preview_command(KeyCode::KeyF, false),
            Some(PreviewCommand::Drag(-50))
        );
        assert_eq!(
            preview_command(KeyCode::KeyG, false),
            Some(PreviewCommand::ToggleAcceleration)
        );
        assert_eq!(
            preview_command(KeyCode::KeyX, false),
            Some(PreviewCommand::Reset)
        );
        assert_eq!(
            preview_command(KeyCode::F5, false),
            Some(PreviewCommand::Save)
        );
        assert_eq!(
            preview_command(KeyCode::F9, false),
            Some(PreviewCommand::Load)
        );
        assert_eq!(
            preview_command(KeyCode::KeyQ, false),
            Some(PreviewCommand::ToggleTrajectory)
        );
    }
}
