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
use moebius_morph::{Organism, Surface, WormSpec};
use moebius_morph_lab::{Action, Field, Session};
use std::path::PathBuf;
#[derive(Parser)]
struct Cli {
    #[arg(long, default_value_t = 20261123)]
    seed: u64,
    #[arg(long, default_value = "runs/morph_001/organism.json")]
    definition: PathBuf,
    /// New evidence directory; drives real keyboard handlers and records internal screenshots.
    #[arg(long)]
    walkthrough: Option<PathBuf>,
}
#[derive(Resource)]
struct Lab(Session);
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
    saved_spec: Option<WormSpec>,
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
        moebius_morph::load(&path).expect("valid definition file")
    } else {
        WormSpec::from_seed(cli.seed)
    };
    let session = Session::new(spec, path).expect("valid initial anatomy");
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "MOEBIUS — Anatomical Forge / MORPH-001".into(),
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
            advance,
            render_geometry,
            orbit,
            update_text,
            record_walk,
        )
            .chain(),
    );
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
    mut meshes: ResMut<Assets<Mesh>>,
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
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(30., 30.))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.025, 0.033, 0.04),
            perceptual_roughness: 0.95,
            ..default()
        })),
        Transform::from_xyz(0., -1.45, 0.),
    ));
    rendered.material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.36,
        reflectance: 0.42,
        ..default()
    });
    commands.spawn((Node {position_type:PositionType::Absolute,left:Val::Px(20.),top:Val::Px(20.),bottom:Val::Px(20.),width:Val::Px(326.),padding:UiRect::all(Val::Px(18.)),flex_direction:FlexDirection::Column,row_gap:Val::Px(18.),..default()},BackgroundColor(Color::srgba(0.018,0.027,0.034,0.96)))).with_children(|p| {
        p.spawn((Text::new("MOEBIUS\nANATOMICAL FORGE"),TextFont {font_size: FontSize::Px(25.),..default()},TextColor(Color::srgb(0.89,0.84,0.70))));
        p.spawn((Text::new("MORPH-001   /   WORM\nLive procedural geometry"),TextFont {font_size: FontSize::Px(14.),..default()},TextColor(Color::srgb(0.42,0.73,0.70))));
        p.spawn((Text::new("Compiling anatomy…"),TextFont {font_size: FontSize::Px(16.),..default()},TextColor(Color::srgb(0.88,0.87,0.80)),UiText::Controls));
        p.spawn((Text::new("1-7 / Up / Down: select\nLeft / Right: edit\nShift: larger changes\n\nR: seed   F: flagella count\nG: ridges  Space: animation\nM: auto mouth\nS: save    L: reload\n\nRight-drag: orbit\nWheel: zoom\nAlt + arrows: orbit"),TextFont {font_size: FontSize::Px(14.),..default()},TextColor(Color::srgb(0.56,0.64,0.66))));
        p.spawn((Text::new("Authored preview only.\nNo simulation or learning."),TextFont {font_size: FontSize::Px(13.),..default()},TextColor(Color::srgb(0.68,0.49,0.39))));
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
fn keys(input: Res<ButtonInput<KeyCode>>, mut lab: ResMut<Lab>, walk: Option<Res<Walk>>) {
    if input.pressed(KeyCode::AltLeft) || input.pressed(KeyCode::AltRight) {
        return;
    }
    let mut actions = vec![];
    for (i, key) in [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
    ]
    .into_iter()
    .enumerate()
    {
        if input.just_pressed(key) {
            actions.push(Action::Select(i));
        }
    }
    let current = Field::ALL.iter().position(|f| *f == lab.0.field).unwrap();
    if input.just_pressed(KeyCode::ArrowDown) {
        actions.push(Action::Select(current + 1));
    }
    if input.just_pressed(KeyCode::ArrowUp) {
        actions.push(Action::Select((current + 6) % 7));
    }
    let step = if input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight) {
        3.
    } else {
        1.
    };
    for (key, action) in [
        (KeyCode::ArrowLeft, Action::Adjust(-step)),
        (KeyCode::ArrowRight, Action::Adjust(step)),
        (KeyCode::KeyR, Action::NextSeed),
        (KeyCode::KeyF, Action::CycleFlagCount),
        (KeyCode::KeyG, Action::ToggleRidges),
        (KeyCode::Space, Action::ToggleAnimation),
        (KeyCode::KeyM, Action::ToggleMouthCycle),
        (KeyCode::KeyS, Action::Save),
        (KeyCode::KeyL, Action::Reload),
    ] {
        if input.just_pressed(key) {
            actions.push(action);
        }
    }
    for action in actions {
        if let Err(e) = lab.0.action(action) {
            lab.0.status = format!("REFUSED: {e}");
            eprintln!("MORPH_EDIT_REFUSED {action:?}: {e}");
        }
        if walk.is_none() {
            println!("MORPH_EDIT {action:?} seed{}", lab.0.spec.seed);
        }
    }
}
fn advance(time: Res<Time>, mut lab: ResMut<Lab>) {
    if lab.0.animation {
        lab.0.phase = moebius_morph_lab::advance_phase(lab.0.phase, time.delta_secs());
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
        m.perceptual_roughness = lab.0.spec.appearance.roughness;
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
fn update_text(lab: Res<Lab>, rendered: Res<Rendered>, mut text: Query<(&UiText, &mut Text)>) {
    let s = &lab.0.spec;
    let flag = s.flagella.first().map_or(0., |f| f.length);
    let values = [
        s.seed.to_string(),
        format!("{:.2}", s.length),
        format!("{:.2}", s.radius),
        format!(
            "{:.0}%{}",
            lab.0.pose().mouth * 100.,
            if lab.0.mouth_cycle { " cycling" } else { "" }
        ),
        format!("{flag:.2}"),
        s.flagella.len().to_string(),
        format!("{:.2}", s.curvature),
    ];
    let controls = Field::ALL
        .iter()
        .zip(values)
        .enumerate()
        .map(|(i, (field, value))| {
            format!(
                "{} {}  {}: {}",
                if *field == lab.0.field { ">" } else { " " },
                i + 1,
                field.label(),
                value
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    for (role, mut text) in &mut text {
        text.0 = match role {
            UiText::Controls => controls.clone(),
            UiText::Status => {
                if let Some(g) = &rendered.geometry {
                    let vertices: usize = g.surfaces.iter().map(|m| m.positions.len()).sum();
                    let tris: usize = g.surfaces.iter().map(|m| m.indices.len() / 3).sum();
                    format!(
                        "{}\n{vertices} vertices | {tris} triangles | {} attachments | {} | ridges {}",
                        lab.0.status,
                        g.attachments.len(),
                        if lab.0.animation {
                            "motion running"
                        } else {
                            "motion paused"
                        },
                        if s.ridges.is_some() { "on" } else { "off" }
                    )
                } else {
                    "Compiling geometry".into()
                }
            }
            UiText::Specimen => {
                if let Some(g) = &rendered.geometry {
                    format!(
                        "WORM / seed {}\n{} | UNBOUND PREVIEW",
                        s.seed,
                        &g.digest()[..12]
                    )
                } else {
                    "WORM / compiling".into()
                }
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
        104 => Some(KeyCode::Digit4),
        110..=128 if f % 2 == 0 => Some(KeyCode::ArrowLeft),
        150..=168 if f % 2 == 0 => Some(KeyCode::ArrowRight),
        190 => Some(KeyCode::Digit2),
        192 | 194 | 196 => Some(KeyCode::ArrowRight),
        200 => Some(KeyCode::Digit3),
        202 => Some(KeyCode::ArrowRight),
        204 => Some(KeyCode::Digit5),
        206 | 222 => Some(KeyCode::ArrowRight),
        208 | 214 | 216 => Some(KeyCode::KeyF),
        218 | 220 => Some(KeyCode::KeyG),
        224 => Some(KeyCode::KeyS),
        234 | 250 => Some(KeyCode::KeyR),
        270 => Some(KeyCode::KeyL),
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
        30 => Some("motion_early"),
        80 => Some("motion_later"),
        140 => Some("mouth_closed"),
        180 => Some("mouth_open"),
        212 => Some("flagella_removed"),
        230 => Some("edited_saved"),
        246 => Some("seed_variant_a"),
        262 => Some("seed_variant_b"),
        280 => Some("edited_reloaded"),
        338 => Some("camera_orbited"),
        _ => None,
    }
    .map(str::to_string);
    if tag.is_none() && (12..100).contains(&frame) && frame.is_multiple_of(4) {
        tag = Some(format!("motion_seq_{frame:04}"));
    }
    if frame == 230 {
        walk.saved_digest = Some(g.digest());
        walk.saved_spec = Some(lab.0.spec.clone());
    }
    if frame == 280
        && (walk.saved_digest.as_deref() != Some(g.digest().as_str())
            || walk.saved_spec.as_ref() != Some(&lab.0.spec))
    {
        walk.failure = Some("saved/reloaded spec or posed structure mismatch".into());
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
}
