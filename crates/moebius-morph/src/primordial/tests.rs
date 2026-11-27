use super::*;
fn inert(density: f64) -> Sphere {
    Sphere {
        id: 1,
        radius_m: 0.005,
        density_kg_m3: density,
        position_m: [0.425; 3],
        velocity_m_s: [0.; 3],
    }
}
#[test]
fn d3q19_equilibrium_preserves_mass_momentum_and_isotropic_second_moment() {
    for u in [[0.; 3], [0.01, -0.02, 0.005]] {
        let f = equilibrium(20., u);
        let (rho, v) = moments(&f);
        assert!((rho - 20.).abs() < 1e-12);
        assert!(norm(sub(v, u)) < 1e-16);
    }
    for a in [0, 1, 2] {
        for b in [0, 1, 2] {
            let moment = (0..19)
                .map(|q| weight(q) * f64::from(C[q][a] * C[q][b]))
                .sum::<f64>();
            assert!((moment - if a == b { 1. / 3. } else { 0. }).abs() < 1e-15);
        }
    }
    for q in 0..19 {
        assert_eq!(C[OPP[q]], C[q].map(|v| -v));
    }
}
#[test]
fn sealed_equilibrium_has_no_force_or_material_drift() {
    let mut c = Chamber::new(Config {
        cells: 8,
        gravity_m_s2: [0.; 3],
        ..Default::default()
    })
    .unwrap();
    c.seal().unwrap();
    let initial = c.digest().unwrap();
    c.advance(32).unwrap();
    let d = c.state.diagnostics();
    assert!(d.carrier_residual_kg.abs() < 1e-11);
    assert!(norm(d.momentum) < 1e-14);
    assert_eq!(d.kinetic_j, 0.);
    assert_ne!(initial, c.digest().unwrap());
}
#[test]
fn vacuum_free_fall_is_analytic_directional_and_substep_consistent() {
    for g in [[0.; 3], [0., -0.01, 0.], [0.005, 0., 0.], [0., 0., -0.0025]] {
        for substeps in [1, 2, 4] {
            let mut c = Chamber::new(Config {
                density_kg_m3: 0.,
                viscosity_pa_s: 0.,
                gravity_m_s2: g,
                substeps,
                ..Default::default()
            })
            .unwrap();
            c.add_sphere(inert(80.)).unwrap();
            c.seal().unwrap();
            c.advance(120).unwrap();
            let t = 120. * TICK_NANOS as f64 / 1e9;
            let b = &c.state.spheres[0];
            assert!(norm(sub(b.velocity_m_s, mul(g, t))) < 1e-14);
            assert!(norm(sub(b.position_m, add([0.425; 3], mul(g, 0.5 * t * t)))) < 1e-13);
            assert!(norm(c.state.diagnostics().momentum_residual) < 1e-16);
        }
    }
}
#[test]
fn exact_pair_drag_matches_reduced_mass_reference_and_reaction() {
    let mut c = Chamber::new(Config {
        gravity_m_s2: [0.; 3],
        ..Default::default()
    })
    .unwrap();
    let mut b = inert(8000.);
    b.radius_m = 0.01;
    b.velocity_m_s = [0.01, 0., 0.];
    let m = b.mass();
    c.add_sphere(b).unwrap();
    c.seal().unwrap();
    let cell = c.state.config.cell_at([0.425; 3]).unwrap();
    let mf = c.state.sample_index(cell).density_kg_m3 * c.state.config.h().powi(3);
    let gamma = 6. * std::f64::consts::PI * c.state.config.viscosity_pa_s * 0.01;
    let dt = c.state.config.dt();
    let expected =
        m / (m + mf) * 0.01 + mf / (m + mf) * 0.01 * (-gamma * (1. / m + 1. / mf) * dt).exp();
    couple(&mut c.state).unwrap();
    assert!((c.state.spheres[0].velocity_m_s[0] - expected).abs() < 1e-15);
    assert!(norm(c.state.diagnostics().momentum_residual) < 1e-14);
    assert!(c.state.sample_index(cell).velocity_m_s[0] > 0.);
    assert!(c.state.impulses.drag_loss_j > 0.);
}
#[test]
fn buoyancy_density_control_rises_sinks_and_neutral_stays_initially() {
    let mut velocities = Vec::new();
    for density in [10., 20., 40.] {
        let mut c = Chamber::new(Config::default()).unwrap();
        c.add_sphere(inert(density)).unwrap();
        c.seal().unwrap();
        couple(&mut c.state).unwrap();
        velocities.push(c.state.spheres[0].velocity_m_s[1]);
        assert!(norm(c.state.diagnostics().momentum_residual) < 1e-12);
    }
    assert!(velocities[0] > 0.);
    assert!(velocities[1].abs() < 1e-14);
    assert!(velocities[2] < 0.);
}
#[test]
fn two_way_coupling_and_wall_impulses_close_momentum() {
    let mut c = Chamber::new(Config {
        cells: 8,
        gravity_m_s2: [0., -0.0025, 0.],
        ..Default::default()
    })
    .unwrap();
    let mut b = inert(80.);
    b.position_m = [0.425, 0.006, 0.425];
    b.velocity_m_s = [0., -0.002, 0.];
    c.add_sphere(b).unwrap();
    c.seal().unwrap();
    c.advance(64).unwrap();
    assert!(norm(c.state.diagnostics().momentum_residual) < 1e-11);
    assert!(norm(c.state.impulses.walls_kg_m_s) > 0.);
    assert!(c.state.impulses.drag_loss_j > 0.);
}
#[test]
fn dense_and_sparse_faces_match_each_scalar_bit_and_wall_stock() {
    let mut worlds = Vec::new();
    for backend in [ScalarBackend::Dense, ScalarBackend::SparseActiveFaces] {
        let mut c = Chamber::new(Config {
            cells: 8,
            gravity_m_s2: [0., -0.01, 0.],
            scalar_backend: backend,
            ..Default::default()
        })
        .unwrap();
        c.initialize_velocity(|_| [0.01, 0., 0.]).unwrap();
        c.add_species(
            Species {
                material: "mineral".into(),
                radius_m: 0.01,
                density_kg_m3: 200.,
                diffusivity_m2_s: 0.0001,
            },
            |p| {
                if p[0] < 0.15 && p[1] < 0.15 && p[2] < 0.15 {
                    1e-7
                } else {
                    0.
                }
            },
        )
        .unwrap();
        c.seal().unwrap();
        worlds.push(c);
    }
    for _ in 0..32 {
        for c in &mut worlds {
            c.step().unwrap();
        }
        for i in 0..512 {
            assert_eq!(
                worlds[0].state.fields[0].mass.get(i).to_bits(),
                worlds[1].state.fields[0].mass.get(i).to_bits()
            );
        }
        assert_eq!(
            worlds[0].state.fields[0].wall_kg,
            worlds[1].state.fields[0].wall_kg
        );
    }
    assert!(worlds[0].state.fields[0].wall_kg.iter().sum::<f64>() > 0.);
}
#[test]
fn cfl_failure_is_atomic_and_no_inventory_is_clipped() {
    let mut c = Chamber::new(Config {
        cells: 32,
        gravity_m_s2: [0.; 3],
        ..Default::default()
    })
    .unwrap();
    c.add_species(
        Species {
            material: "unstable diffusivity control".into(),
            radius_m: 0.001,
            density_kg_m3: 100.,
            diffusivity_m2_s: 0.01,
        },
        |p| {
            if norm(sub(p, [0.4; 3])) < 0.04 {
                1e-8
            } else {
                0.
            }
        },
    )
    .unwrap();
    c.seal().unwrap();
    let before = c.state.clone();
    assert!(c.step().unwrap_err().0.contains("CFL"));
    assert_eq!(before, c.state);
}
#[test]
fn bit_checkpoint_replays_and_rejects_source_and_payload_mutation() {
    let dir = std::env::temp_dir().join(format!("habitat-bits-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("checkpoint.json");
    let mut c = experiments::demo(Config {
        cells: 8,
        ..Default::default()
    })
    .unwrap();
    c.advance(4).unwrap();
    c.save(&file).unwrap();
    let mut other = Chamber::load(&file).unwrap();
    assert_eq!(c.state, other.state);
    c.advance(4).unwrap();
    other.advance(4).unwrap();
    assert_eq!(c.digest().unwrap(), other.digest().unwrap());
    let data = std::fs::read(&file).unwrap();
    let mut v: serde_json::Value = serde_json::from_slice(&data).unwrap();
    v["source_identity"] = serde_json::json!("wrong source");
    std::fs::write(&file, serde_json::to_vec(&v).unwrap()).unwrap();
    assert!(Chamber::load(&file).unwrap_err().0.contains("source"));
    let mut v: serde_json::Value = serde_json::from_slice(&data).unwrap();
    v["state"]["tick"] = serde_json::json!(99);
    std::fs::write(&file, serde_json::to_vec(&v).unwrap()).unwrap();
    assert!(Chamber::load(&file).unwrap_err().0.contains("digest"));
    std::fs::remove_dir_all(dir).unwrap();
}
#[test]
fn observer_sampling_does_not_select_or_modify_evolution() {
    let mut a = experiments::demo(Config {
        cells: 8,
        ..Default::default()
    })
    .unwrap();
    let mut b = experiments::demo(Config {
        cells: 8,
        ..Default::default()
    })
    .unwrap();
    for _ in 0..8 {
        for i in 0..512 {
            let _ = a.state.sample_index(i);
        }
        a.step().unwrap();
        b.step().unwrap();
    }
    assert_eq!(a.digest().unwrap(), b.digest().unwrap());
}
#[test]
fn independent_density_viscosity_and_explicit_guards() {
    let c = Config::default();
    let mut d = c.clone();
    d.density_kg_m3 *= 2.;
    assert_eq!(d.viscosity_pa_s, c.viscosity_pa_s);
    assert_ne!(d.tau(), c.tau());
    assert!(d.validate().is_ok());
    d.viscosity_pa_s = 0.;
    assert!(d.validate().is_err());
    d = Config {
        gravity_m_s2: [f64::NAN, 0., 0.],
        ..Default::default()
    };
    assert!(d.validate().is_err());
    let mut world = Chamber::new(c).unwrap();
    world.seal().unwrap();
    assert!(world.add_sphere(inert(100.)).is_err());
}
