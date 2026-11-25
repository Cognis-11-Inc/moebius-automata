use moebius_morph::{collision::*, emission::SCALE, runtime::*, *};
#[test]
fn presentation_writhe_and_collision_hull_share_one_anatomy() {
    let spec = AnatomySpec::bare(77);
    let hull = collision_pose(&spec);
    let shown = habitat_presentation_pose(&spec, 40, true);
    assert!((hull.bend - COLLISION_BEND).abs() < 1e-6);
    assert!((shown.bend - PRESENTATION_BEND).abs() < 1e-6);
    assert_eq!(hull.flagella, 1.);
    assert_eq!(shown.flagella, 1.);
    assert!((hull.mouth - spec.body.mouth.aperture).abs() < 1e-6);
    assert!((shown.mouth - (spec.body.mouth.aperture + 0.25).min(1.)).abs() < 1e-6);
    let slack = presentation_slack(&spec).unwrap();
    println!("presentation_slack_metres={slack}");
    assert!(
        slack.is_finite() && slack > 0.005,
        "writhe clearance {slack}"
    );
    assert!(
        slack < 0.25,
        "writhe clearance {slack} m is larger than the authored wave"
    );
    let compiled = CompiledAnatomy::new(spec.clone()).unwrap();
    let tight = compiled.pose(hull).unwrap();
    let wave = compiled
        .pose(habitat_presentation_pose(&spec, 40, false))
        .unwrap();
    assert_eq!(
        tight.geometry.topology_digest(),
        wave.geometry.topology_digest(),
        "writhe changed topology"
    );
    let mut world = ShipRuntime::habitat_simulation(spec, 77, 30).unwrap();
    world.advance(8).unwrap();
    let state = world.state();
    let rigid = state.body_pose(1).unwrap();
    assert_eq!(
        rigid.translation,
        state.instances[&1].position.absolute().unwrap(),
        "propulsion pose is the rigid step"
    );
    assert!((state.anatomical_pose(1).unwrap().bend - PRESENTATION_BEND).abs() < 1e-6);
}

#[test]
fn swept_query_detects_thin_barriers_and_respects_a_hollow_lumen() {
    let mut s = Surface {
        name: "thin wall".into(),
        positions: vec![[0., -1., -1.], [0., 1., -1.], [0., 1., 1.], [0., -1., 1.]],
        normals: vec![],
        uvs: vec![],
        colors: vec![],
        indices: vec![0, 1, 2, 0, 2, 3],
    };
    assert!(
        swept_ball_against_surface([-1., 0., 0.], [1., 0., 0.], 0.01, &s).unwrap(),
        "thin obstacle was tunneled through"
    );
    s.positions.iter_mut().for_each(|p| p[1] += 5.);
    assert!(!swept_ball_against_surface([-1., 0., 0.], [1., 0., 0.], 0.01, &s).unwrap());
}
fn barrier_world() -> WorldSnapshot {
    let mut world = ShipRuntime::habitat_simulation(AnatomySpec::bare(77), 77, 90)
        .unwrap()
        .snapshot();
    world.config.ports.clear();
    world.config.ambient_light = 50;
    for body in world.instances.values_mut() {
        body.exposure = 50;
        if body.definition == BLOOD_SAC {
            body.position =
                Position::from_absolute([100 * SCALE + body.id as i64 * SCALE, 0, 0]).unwrap();
        }
    }
    world.habitat.as_mut().unwrap().rules.evasion = false;
    world.habitat.as_mut().unwrap().rules.reproduction = false;
    world.habitat.as_mut().unwrap().rules.max_sacs = 6;
    world.instances.get_mut(&1).unwrap().book.emission_enabled = false;
    let agent = world.habitat.as_mut().unwrap().agents.get_mut(&1).unwrap();
    agent.heading = [SCALE, 0, 0];
    // Intersect the proposed +X path of a real body/organ envelope, not a rendered mesh.
    world.solids.push(SolidBox {
        id: 1,
        // Exploration can legally go around the former 4m-wide box; keep the
        // plane assertion meaningful across the full 100-tick travel envelope.
        min: Position::from_absolute([2 * SCALE, -20 * SCALE, -20 * SCALE]).unwrap(),
        max: Position::from_absolute([2 * SCALE + 1000, 20 * SCALE, 20 * SCALE]).unwrap(),
    });
    world
}
#[test]
fn canonical_world_blocks_motion_and_exposes_biological_contacts() {
    let mut w = ShipRuntime::restore(barrier_world()).unwrap();
    let mut contacts = 0;
    let initial = w.state().instances[&1].position;
    let spec = w.state().config.definitions[&1].clone();
    for _ in 0..100 {
        w.tick().unwrap();
        contacts += w.state().contact_observations(1).len();
        let pose = w.state().body_pose(1).unwrap();
        // The live writhe, not a frozen frame, must stay outside the solid.
        let numerical = CompiledAnatomy::new(spec.clone())
            .unwrap()
            .pose(w.state().anatomical_pose(1).unwrap())
            .unwrap()
            .geometry;
        for point in numerical.surfaces.iter().flat_map(|s| &s.positions) {
            let p = moebius_morph::emission::place_point(*point, pose).unwrap();
            assert!(
                p[0] < 2 * SCALE,
                "presented anatomy penetrated the barrier: point={p:?}, body={pose:?}"
            );
        }
        let position = w.state().instances[&1].position.absolute().unwrap();
        assert!(
            position[0] < 2 * SCALE,
            "canonical world admitted forbidden penetration"
        );
    }
    assert!(
        contacts > 0,
        "collision handling emitted no sensory observations"
    );
    assert!(w.state().events.iter().any(|e|matches!(e,Event::Contact{observation}if observation.organism==1&&observation.response==ContactResponse::Blocked)));
    assert_eq!(w.state().condensate_gap(), 0);
    println!("initial={initial:?} contacts={contacts}");
}

#[test]
fn open_lumen_passes_a_small_probe_but_walls_and_a_sealed_end_block_it() {
    use moebius_morph::filament::*;
    let mut w = construction_fixture(77, 60, 40, "straight").unwrap();
    w.advance(20).unwrap();
    let mut f = w
        .state()
        .assembly
        .as_ref()
        .unwrap()
        .filaments
        .values()
        .next()
        .unwrap()
        .clone();
    f.nodes.truncate(5);
    for (k, n) in f.nodes.iter_mut().enumerate() {
        n.position = Position::from_absolute([0, 0, k as i64 * 250_000]).unwrap();
    }
    f.section = Some(PipeSection {
        outer_radius: 50_000,
        inner_radius: 31_000,
        start_sealed: false,
        end_sealed: false,
    });
    let (_, surface) = filament_chunk_at(&f, 0, 0).unwrap();
    assert!(
        !swept_ball_against_surface([0., 0., -0.1], [0., 0., 1.1], 0.005, &surface).unwrap(),
        "empty lumen became a solid collider"
    );
    assert!(
        swept_ball_against_surface([0.08, 0., 0.5], [0., 0., 0.5], 0.005, &surface).unwrap(),
        "pipe wall failed to obstruct"
    );
    f.section.as_mut().unwrap().end_sealed = true;
    let (_, sealed) = filament_chunk_at(&f, 0, 0).unwrap();
    assert!(
        swept_ball_against_surface([0., 0., 1.1], [0., 0., 0.5], 0.005, &sealed).unwrap(),
        "sealed lumen was traversed"
    );
}
#[test]
fn rotational_sweep_hits_an_obstacle_that_neither_endpoint_overlaps() {
    use moebius_morph::emission::RigidPose;
    let (vertices, triangles) =
        parry3d::shape::Cuboid::new(parry3d::math::Vector::new(0.01, 0.01, 2.)).to_trimesh();
    let surface = Surface {
        name: "physical appendage".into(),
        positions: vertices.iter().map(|v| v.to_array()).collect(),
        normals: vec![],
        uvs: vec![],
        colors: vec![],
        indices: triangles.into_iter().flatten().collect(),
    };
    let obstacle = SolidBox {
        id: 5,
        min: Position::from_absolute([650_000, -50_000, 650_000]).unwrap(),
        max: Position::from_absolute([750_000, 50_000, 750_000]).unwrap(),
    };
    let from = RigidPose::identity();
    let to = RigidPose {
        yaw_sin: SCALE,
        yaw_cos: 0,
        ..from
    };
    assert!(!swept_surface_against_box(&surface, from, from, &obstacle).unwrap());
    assert!(!swept_surface_against_box(&surface, to, to, &obstacle).unwrap());
    assert!(
        swept_surface_against_box(&surface, from, to, &obstacle).unwrap(),
        "rotation tunneled through a thin obstacle"
    );
}

#[test]
fn witness_visibility_and_reload_cannot_change_collision_or_biology() {
    use moebius_morph::witness::{PresentationCache, WitnessSphere};
    let initial = barrier_world();
    let mut unobserved = ShipRuntime::restore(initial.clone()).unwrap();
    let mut observed = ShipRuntime::restore(initial.clone()).unwrap();
    let mut distant = ShipRuntime::restore(initial).unwrap();
    let mut cache = PresentationCache::default();
    let mut saw_contact = false;
    for tick in 0..100 {
        let near = WitnessSphere {
            center: observed.state().instances[&1].position,
            radius: 4 * SCALE,
        };
        let far = WitnessSphere {
            center: Position::from_absolute([1_000_000 * SCALE, 0, 0]).unwrap(),
            radius: 0,
        };
        cache.update(&near.inspect(observed.state()).unwrap());
        assert!(far.inspect(distant.state()).unwrap().organisms.is_empty());
        unobserved.tick().unwrap();
        observed.tick().unwrap();
        distant.tick().unwrap();
        saw_contact |= !observed.state().contact_observations(1).is_empty();
        assert_eq!(
            unobserved.state(),
            observed.state(),
            "near observation changed tick {tick}"
        );
        assert_eq!(
            unobserved.state(),
            distant.state(),
            "offscreen collision changed tick {tick}"
        );
        if tick == 50 {
            let saved = serde_json::to_vec(distant.state()).unwrap();
            distant = ShipRuntime::restore(serde_json::from_slice(&saved).unwrap()).unwrap();
        }
    }
    assert!(saw_contact, "test never exercised the collision response");
}

#[test]
fn unsupported_ballistic_motion_is_rejected_in_collision_mode() {
    let mut world = barrier_world();
    world.instances.get_mut(&1).unwrap().velocity = [1, 0, 0];
    assert!(ShipRuntime::restore(world).is_err());
    let mut world = barrier_world();
    world.instances.get_mut(&1).unwrap().book.launch_speed = 1;
    assert!(ShipRuntime::restore(world).is_err());
}

#[test]
fn diagnostic_collision_switch_is_off_only_when_requested_and_skips_resolve() {
    let mut on = ShipRuntime::habitat_simulation(AnatomySpec::bare(77), 77, 4).unwrap();
    assert!(on.state().habitat.as_ref().unwrap().rules.collisions);
    on.tick().unwrap();
    assert!(on.collision_profile().resolve_calls >= 1);
    let mut off = ShipRuntime::habitat_simulation(AnatomySpec::bare(77), 77, 4).unwrap();
    off.diagnostic_disable_collisions().unwrap();
    assert!(!off.state().habitat.as_ref().unwrap().rules.collisions);
    off.tick().unwrap();
    let profile = off.collision_profile();
    assert_eq!(profile.resolve_calls, 0);
    assert_eq!(profile.queries, 0);
    assert_eq!(profile.pipe_candidates, 0);
    assert_eq!(profile.pipe_rejects, 0);
}
