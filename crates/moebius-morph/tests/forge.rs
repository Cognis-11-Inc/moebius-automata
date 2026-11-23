use moebius_morph::*;
#[test]
fn repeatability_topology_and_attached_motion() {
    for seed in [0, 1, 42, u64::MAX] {
        let spec = WormSpec::from_seed(seed);
        assert_eq!(spec, WormSpec::from_seed(seed));
        let a = CompiledWorm::new(spec.clone()).unwrap();
        let b = CompiledWorm::new(from_json(&to_json(&spec).unwrap()).unwrap()).unwrap();
        let rest = a.pose(Pose::rest(&spec)).unwrap();
        for phase in [0., 1.3, 3.1, 5.4] {
            let p = Pose {
                phase,
                bend: 1.,
                mouth: phase / 6.,
                flagella: 1.,
            };
            let moved = a.pose(p).unwrap();
            moved.audit().unwrap();
            assert_eq!(moved, b.pose(p).unwrap());
            assert_eq!(rest.topology_digest(), moved.topology_digest());
            assert_ne!(rest.digest(), moved.digest());
            for f in &moved.attachments {
                assert_eq!(f.capability, "decorative_unbound");
            }
        }
    }
}
#[test]
fn real_opening_has_an_interior_and_never_collapses_at_closed_limit() {
    let spec = WormSpec::from_seed(93);
    let c = CompiledWorm::new(spec.clone()).unwrap();
    let mut p = Pose::rest(&spec);
    p.mouth = 1.;
    let opened = c.pose(p).unwrap();
    p.mouth = 0.;
    let closed = c.pose(p).unwrap();
    opened.audit().unwrap();
    closed.audit().unwrap();
    let extent = |g: &Organism| {
        let ps = &g.surfaces[0].positions;
        let y = g
            .mouth_ring
            .iter()
            .map(|&i| ps[i as usize][1])
            .collect::<Vec<_>>();
        y.iter().copied().fold(f32::MIN, f32::max) - y.iter().copied().fold(f32::MAX, f32::min)
    };
    assert!(extent(&opened) > extent(&closed) * 5.);
    let floor = opened.surfaces[0].positions[opened.throat_base as usize];
    assert!((opened.mouth_center[2] - floor[2]) > spec.radius * 0.2);
    assert_eq!(opened.mouth_ring.len(), spec.radial_segments);
}
#[test]
fn malformed_specs_and_poses_are_rejected_before_generation() {
    let base = WormSpec::from_seed(12);
    for field in 0..9 {
        let mut s = base.clone();
        match field {
            0 => s.version = 2,
            1 => s.radius = f32::NAN,
            2 => s.radius_profile[2] = 0.,
            3 => s.body_rings = usize::MAX,
            4 => s.mouth.aperture = -1.,
            5 => s.flagella[0].attachment.station = 4.,
            6 => s.appearance.roughness = f32::INFINITY,
            7 => s.flagella.push(s.flagella[0].clone()),
            _ => s.length = 0.5,
        };
        assert!(CompiledWorm::new(s).is_err());
    }
    assert!(
        from_json(
            &to_json(&base)
                .unwrap()
                .replace("\"version\": 1", "\"version\": 1,\"unknown\":2")
        )
        .is_err()
    );
    let c = CompiledWorm::new(base.clone()).unwrap();
    let mut p = Pose::rest(&base);
    p.bend = f32::NAN;
    assert!(c.pose(p).is_err());
}
#[test]
fn save_reload_preserves_definition_geometry_and_invalid_save_leaves_file_intact() {
    let dir = std::env::temp_dir().join(format!("moebius-morph-test-{}", std::process::id()));
    std::fs::create_dir(&dir).unwrap();
    let file = dir.join("organism.json");
    let s = WormSpec::from_seed(711);
    save(&file, &s).unwrap();
    let restored = load(&file).unwrap();
    assert_eq!(s, restored);
    let pose = Pose::rest(&s);
    assert_eq!(
        CompiledWorm::new(s.clone())
            .unwrap()
            .pose(pose)
            .unwrap()
            .digest(),
        CompiledWorm::new(restored)
            .unwrap()
            .pose(pose)
            .unwrap()
            .digest()
    );
    let before = std::fs::read(&file).unwrap();
    let mut bad = s;
    bad.version = 99;
    assert!(save(&file, &bad).is_err());
    assert_eq!(before, std::fs::read(&file).unwrap());
    std::fs::remove_dir_all(dir).unwrap();
}
#[test]
fn optional_components_have_predictable_topology_and_no_physics_binding() {
    let mut s = WormSpec::from_seed(5);
    s.flagella.clear();
    s.ridges = None;
    let c = CompiledWorm::new(s.clone()).unwrap();
    let g = c.pose(Pose::rest(&s)).unwrap();
    g.audit().unwrap();
    assert_eq!(g.surfaces.len(), 1);
    assert!(g.attachments.is_empty());
    assert!(g.mechanical_binding.is_none());
    let mut broken = g.surfaces[0].clone();
    broken.indices[0] = u32::MAX;
    assert!(broken.audit().is_err());
    let mut broken = g.surfaces[0].clone();
    broken.indices.truncate(broken.indices.len() - 3);
    assert!(broken.audit().is_err());
    let mut broken = g.surfaces[0].clone();
    broken.positions[0][0] = f32::NAN;
    assert!(broken.audit().is_err());
    let mut broken = g.surfaces[0].clone();
    broken.indices[1] = broken.indices[0];
    assert!(broken.audit().is_err());
}

#[test]
fn central_mouth_ray_reaches_recessed_floor_instead_of_a_face_patch() {
    let mut spec = WormSpec::from_seed(103);
    spec.curvature = 0.;
    let c = CompiledWorm::new(spec.clone()).unwrap();
    let mut pose = Pose::rest(&spec);
    pose.mouth = 1.;
    let g = c.pose(pose).unwrap();
    let mut origin = g.mouth_center;
    origin[2] += spec.radius;
    let hit = g.surfaces[0].ray_distance(origin, [0., 0., -1.]).unwrap();
    let expected = spec.radius + spec.radius * spec.radius_profile[3] * spec.mouth.depth;
    assert!(
        (hit - expected).abs() < 1e-4,
        "mouth ray hit{hit}, floor expected{expected}"
    );
}
