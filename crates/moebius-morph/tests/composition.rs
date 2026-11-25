use moebius_morph::*;
use std::f32::consts::TAU;
#[test]
fn independent_identity_validation_and_typed_metadata() {
    let mut s = AnatomySpec::showcase(41);
    s.validate().unwrap();
    let json = serde_json::to_string(&s).unwrap();
    assert_eq!(s, anatomy_from_json(&json).unwrap());
    let id = s.components[0].id;
    s.components[1].id = id;
    assert!(s.validate().is_err());
    s.components[1].id = 0;
    assert!(s.validate().is_err());
    s.components[1].id = 100;
    s.components[1].parent = 999;
    assert!(s.validate().is_err());
    s.components[1].parent = 1;
    assert!(s.validate().is_err());
    s.components[1].parent = 0;
    if let Shape::SecretionNozzle { output, .. } = &mut s.components.last_mut().unwrap().shape {
        output.operational = true;
    }
    assert!(s.validate().is_err());
}
#[test]
fn zero_two_eight_sixteen_components_animate_attach_and_keep_topology() {
    for n in [0, 2, 8, 16] {
        let mut s = AnatomySpec::from_seed(12);
        s.components.clear();
        for i in 0..n {
            let mut c = Component::flagellum(
                i + 2,
                0.10 + (i % 4) as f32 * 0.2,
                TAU * i as f32 / n as f32,
            );
            c.orientation = [i as f32 * 0.1, -0.3 + i as f32 * 0.04];
            s.components.push(c);
        }
        let c = CompiledAnatomy::new(s.clone()).unwrap();
        let mut topology = None;
        for k in 0..8 {
            let p = Pose {
                phase: k as f32 * 0.83,
                bend: 1.,
                mouth: (k % 2) as f32,
                flagella: 1.,
            };
            let g = c.pose(p).unwrap();
            g.geometry.audit().unwrap();
            assert_eq!(g.component_ids.len(), n as usize);
            assert_eq!(g.geometry.surfaces.len(), n as usize + 1);
            if let Some(t) = &topology {
                assert_eq!(*t, g.geometry.topology_digest());
            } else {
                topology = Some(g.geometry.topology_digest());
            }
        }
    }
}
#[test]
fn legacy_migration_preserves_parameters_and_geometry_with_declared_tolerance() {
    for seed in [0, 1, 12, 99, 20261123] {
        let old = WormSpec::from_seed(seed);
        let migrated = anatomy_from_json(&to_json(&old).unwrap()).unwrap();
        let mut body = migrated.body.clone();
        body.flagella = old.flagella.clone();
        assert_eq!(body, old);
        for k in 0..8 {
            let p = Pose {
                phase: k as f32 * 0.83,
                bend: 1.,
                mouth: (k % 2) as f32,
                flagella: 1.,
            };
            let a = CompiledWorm::new(old.clone()).unwrap().pose(p).unwrap();
            let b = CompiledAnatomy::new(migrated.clone())
                .unwrap()
                .pose(p)
                .unwrap()
                .geometry;
            assert_eq!(a.surfaces.len(), b.surfaces.len());
            for (a, b) in a.surfaces.iter().zip(&b.surfaces) {
                assert_eq!(a.indices, b.indices);
                assert_eq!(a.uvs, b.uvs);
                assert_eq!(a.colors, b.colors);
                for (a, b) in a
                    .positions
                    .iter()
                    .flatten()
                    .zip(b.positions.iter().flatten())
                {
                    assert!(
                        (a - b).abs() < 1e-6,
                        "migration coordinate delta {}",
                        (a - b).abs()
                    );
                }
                for (a, b) in a.normals.iter().flatten().zip(b.normals.iter().flatten()) {
                    assert!(
                        (a - b).abs() < 2e-4,
                        "migration normal delta {}",
                        (a - b).abs()
                    );
                }
            }
        }
    }
}
#[test]
fn attachments_are_rebound_when_body_topology_changes() {
    let mut s = AnatomySpec::showcase(2);
    let a = CompiledAnatomy::new(s.clone())
        .unwrap()
        .pose(Pose::rest(&s.body))
        .unwrap();
    s.body.body_rings = 93;
    s.body.radial_segments = 47;
    s.body.length = 4.2;
    s.body.curvature = -0.5;
    let b = CompiledAnatomy::new(s.clone())
        .unwrap()
        .pose(Pose {
            phase: 1.7,
            bend: 1.,
            mouth: 0.,
            flagella: 1.,
        })
        .unwrap();
    b.geometry.audit().unwrap();
    assert_eq!(a.component_ids, b.component_ids);
    assert_ne!(
        a.geometry.attachments[0].triangle,
        b.geometry.attachments[0].triangle
    );
    assert_eq!(s.components[0].attachment.station, 0.18);
}
#[test]
fn mouth_seam_interior_and_nozzle_are_real_geometry() {
    let s = AnatomySpec::showcase(8);
    let c = CompiledAnatomy::new(s.clone()).unwrap();
    let closed = c
        .pose(Pose {
            phase: 0.,
            bend: 0.,
            mouth: 0.,
            flagella: 0.,
        })
        .unwrap();
    let opened = c
        .pose(Pose {
            phase: 0.,
            bend: 0.,
            mouth: 1.,
            flagella: 0.,
        })
        .unwrap();
    for g in [&closed.geometry, &opened.geometry] {
        g.audit().unwrap();
        let ring = &g.mouth_ring;
        let body = &g.surfaces[0];
        let min_y = ring
            .iter()
            .map(|&i| body.positions[i as usize][1])
            .fold(f32::INFINITY, f32::min);
        let max_y = ring
            .iter()
            .map(|&i| body.positions[i as usize][1])
            .fold(f32::NEG_INFINITY, f32::max);
        assert!(max_y - min_y > 0.);
        let d = body
            .ray_distance(
                [g.mouth_center[0], g.mouth_center[1], g.mouth_center[2] + 1.],
                [0., 0., -1.],
            )
            .unwrap();
        assert!(d > 1.);
    }
    assert_ne!(closed.geometry.digest(), opened.geometry.digest());
    assert_eq!(closed.outputs.len(), 1);
    assert!(!closed.outputs[0].metadata.operational);
    let nozzle = closed.geometry.surfaces.last().unwrap();
    let output = &closed.outputs[0];
    let origin = std::array::from_fn(|i| output.position[i] + output.direction[i] * 0.2);
    let direction = output.direction.map(|v| -v);
    let d = nozzle.ray_distance(origin, direction).unwrap();
    assert!(d > 0.3, "nozzle aperture blocked by exterior cap: {d}");
}
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}
fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}
fn length(a: [f32; 3]) -> f32 {
    dot(a, a).sqrt()
}
/// Spine and nozzle roots spread into the trunk colour and sit under the skin.
/// Flagella stay narrow tubes seated on the attachment point. Separate surfaces.
#[test]
fn spine_and_nozzle_roots_fuse_into_body_flesh() {
    let spec = AnatomySpec::showcase(8);
    let skin = spec.body.appearance.skin;
    assert_ne!(skin, [0.30, 0.24, 0.19]);
    assert_ne!(skin, [0.31, 0.075, 0.10]);
    let compiled = CompiledAnatomy::new(spec.clone()).unwrap();
    for pose in [
        Pose::rest(&spec.body),
        Pose {
            phase: 1.4,
            bend: 1.,
            mouth: 0.2,
            flagella: 1.,
        },
    ] {
        let posed = compiled.pose(pose).unwrap();
        posed.geometry.audit().unwrap();
        assert_eq!(posed.outputs.len(), 1);
        let mut flesh = 0;
        let mut flagella = 0;
        for (i, component) in spec.components.iter().enumerate() {
            let (kind, radius, distal) = match component.shape {
                Shape::FoodPouch { radius, .. } => ("pouch", radius, [0.30, 0.13, 0.23]),
                Shape::Spine { radius, .. } => ("spine", radius, [0.30, 0.24, 0.19]),
                Shape::SecretionNozzle { radius, .. } => ("nozzle", radius, [0.31, 0.075, 0.10]),
                Shape::Flagellum { radius, .. } => {
                    ("flagellum", radius, spec.body.appearance.flagella)
                }
            };
            let part = &posed.geometry.surfaces[i + 1];
            let frame = &posed.geometry.attachments[i];
            assert_eq!(part.positions[0], frame.origin);
            let mut centroid = [0f32; 3];
            let mut max_extent = 0f32;
            let mut min_extent = f32::MAX;
            let mut short = [0f32; 3];
            let mut long = [0f32; 3];
            for vertex in &part.positions[1..11] {
                for (sum, value) in centroid.iter_mut().zip(vertex) {
                    *sum += value;
                }
            }
            for sum in &mut centroid {
                *sum /= 10.;
            }
            for vertex in &part.positions[1..11] {
                let extent = length(sub(*vertex, centroid));
                if extent > max_extent {
                    max_extent = extent;
                    long = *vertex;
                }
                if extent < min_extent {
                    min_extent = extent;
                    short = *vertex;
                }
            }
            let inset = dot(sub(centroid, frame.origin), frame.normal);
            assert!(
                inset.abs() < 1e-3,
                "{kind} root left the attachment plane by {inset}"
            );
            let ratio = max_extent / min_extent;
            let root_color = [part.colors[1][0], part.colors[1][1], part.colors[1][2]];
            let shaft_color = [part.colors[81][0], part.colors[81][1], part.colors[81][2]];
            assert_eq!(shaft_color, distal, "{kind} shaft left the flesh blend");
            if kind == "flagellum" {
                flagella += 1;
                assert!(
                    max_extent < radius * 1.15 && ratio < 1.2,
                    "flagellum root extent {min_extent}..{max_extent} from {radius}"
                );
                assert_eq!(root_color, spec.body.appearance.flagella);
            } else {
                flesh += 1;
                assert!(
                    max_extent > radius * 1.7,
                    "{kind} root spread {max_extent} radius {radius}"
                );
                assert_eq!(root_color, skin, "{kind} root is not trunk colour");
                if kind == "spine" {
                    assert!(
                        ratio > 2.,
                        "spine root is not pressed into the trunk: {ratio}"
                    );
                    let short_n = dot(sub(short, centroid), frame.normal).abs();
                    let long_n = dot(sub(long, centroid), frame.normal).abs();
                    assert!(short_n > long_n, "spine short axis is not the trunk normal");
                } else {
                    assert!(ratio < 1.25, "nozzle foot became a ribbon: {ratio}");
                }
            }
        }
        assert_eq!(flesh, 2);
        assert_eq!(flagella, 8);
        let output = &posed.outputs[0];
        let nozzle = posed.geometry.surfaces.last().unwrap();
        let from = std::array::from_fn(|i| output.position[i] + output.direction[i] * 0.2);
        let toward = output.direction.map(|v| -v);
        let gap = nozzle.ray_distance(from, toward).unwrap();
        assert!(gap > 0.3, "fused root closed the nozzle: {gap}");
    }
}
