use super::*;
fn uniform(c: Config) -> Chamber {
    let rho = c.density;
    Chamber::new(c, |_| (rho, [0.; 3], [0.; 4])).unwrap()
}
#[test]
fn native_partition_and_remap() {
    let mut c = uniform(Config::default());
    let before = c.state.diagnostics().total;
    c.refine(&[3, 4]).unwrap();
    assert_eq!(c.state.cells.len(), 46);
    c.state.validate().unwrap();
    c.coarsen(&[3, 4]).unwrap();
    assert_eq!(c.state.cells.len(), 32);
    assert_eq!(c.state.diagnostics().total, before);
    let mut invalid = c.state.clone();
    invalid.cells.insert(1, invalid.cells[&0].clone());
    assert!(invalid.validate().is_err());
}
#[test]
fn equilibrium_uniform_refined_and_subcycled() {
    for periodic in [false, true] {
        let mut c = uniform(Config {
            periodic,
            ..Default::default()
        });
        c.refine(&[2, 3]).unwrap();
        for _ in 0..10 {
            c.step().unwrap();
        }
        for _ in 0..10 {
            c.begin_subcycle().unwrap();
            c.finish_subcycle().unwrap();
        }
        assert!(c.state.diagnostics().max_speed < 1e-12);
        c.state.validate().unwrap();
    }
}
#[test]
fn rejection_capacity_and_recovery() {
    let mut c = uniform(Config {
        capacity: 32,
        ..Default::default()
    });
    let d = c.digest().unwrap();
    assert!(c.refine(&[0]).is_err());
    assert_eq!(c.digest().unwrap(), d);
    c.fail_before_commit = true;
    assert!(c.step().is_err());
    assert_eq!(c.digest().unwrap(), d);
    c.fail_before_commit = false;
    c.step().unwrap();
    let mut invalid = c.state.clone();
    if let Payload::Fv(u) = invalid.cells.get_mut(&0).unwrap() {
        u[0] *= 1.01;
    }
    assert!(invalid.validate().is_err());
}
#[test]
fn exact_checkpoint_and_pending_continuation() {
    let mut c = uniform(Config::default());
    c.refine(&[3]).unwrap();
    c.begin_subcycle().unwrap();
    let p = std::env::temp_dir().join(format!("h003-pending-{}.json", std::process::id()));
    c.save(&p).unwrap();
    let mut other = Chamber::load(&p).unwrap();
    assert_eq!(c.digest().unwrap(), other.digest().unwrap());
    c.finish_subcycle().unwrap();
    other.finish_subcycle().unwrap();
    for _ in 0..3 {
        c.step().unwrap();
        other.step().unwrap();
    }
    assert_eq!(c.digest().unwrap(), other.digest().unwrap());
    std::fs::remove_file(p).unwrap();
}
#[test]
fn scalar_and_momentum_interface_conserve() {
    let cfg = Config {
        periodic: true,
        species: vec![Species {
            material: "probe".into(),
            radius_m: 1e-5,
            density_kg_m3: 20.,
            diffusivity_m2_s: 0.001,
        }],
        ..Default::default()
    };
    let mut c = Chamber::new(cfg, |g| {
        (
            20.,
            [0.01, 0.005, 0.],
            [(1. + (g.centre()[0] * 5.).sin()) * 1e-4, 0., 0., 0.],
        )
    })
    .unwrap();
    c.refine(&[3, 4, 11, 12]).unwrap();
    for _ in 0..20 {
        c.begin_subcycle().unwrap();
        c.finish_subcycle().unwrap();
    }
    c.state.validate().unwrap();
    assert!(
        c.state
            .diagnostics()
            .residual
            .iter()
            .all(|r| r.abs() < 1e-10)
    );
}
#[test]
fn indicators_are_physical_and_versioned() {
    let mut c = uniform(Config {
        indicators: Some(Indicators {
            density: 0.001,
            velocity: 0.01,
            scalar: 1e-4,
            nonequilibrium: 0.01,
            coarsen_fraction: 0.5,
            quiet_cycles: 2,
        }),
        ..Default::default()
    });
    c.refine(&[0]).unwrap();
    c.step().unwrap();
    assert_eq!(c.state.cells.len(), 39);
    c.step().unwrap();
    assert_eq!(c.state.cells.len(), 32);
}
#[test]
fn lbm_uniform_equilibrium() {
    let h = 0.1;
    let dt = 0.001;
    let mut c = uniform(Config {
        model: Model::AdaptiveLbm,
        h,
        dt,
        sound_speed: h / dt / 3f64.sqrt(),
        viscosity: 1.,
        periodic: true,
        ..Default::default()
    });
    c.refine(&[3]).unwrap();
    for _ in 0..4 {
        c.step().unwrap();
    }
    assert!(c.state.diagnostics().max_speed < 1e-12);
}

#[test]
fn invalid_pending_and_cfl_are_refused() {
    let mut c = uniform(Config::default());
    c.refine(&[3]).unwrap();
    c.begin_subcycle().unwrap();
    let original = c.digest().unwrap();
    let mut p = c.pending.clone().unwrap();
    p.stage = 2;
    assert!(fv::validate_pending(&c.state, &p).is_err());
    assert_eq!(c.digest().unwrap(), original);
    let mut c = uniform(Config {
        dt: 0.1,
        ..Default::default()
    });
    let d = c.digest().unwrap();
    assert!(c.step().is_err());
    assert_eq!(c.digest().unwrap(), d);
    let mut bad = c.state.clone();
    bad.ledger.walls_impulse[0] = f64::NAN;
    assert!(bad.validate().is_err());
}
#[test]
fn adaptive_lbm_failure_is_not_normalized() {
    let cfg = experiments::config(
        16,
        0.001,
        Model::AdaptiveLbm,
        experiments::Scenario::Wave,
        0,
    );
    let mut c = experiments::chamber(cfg, experiments::Scenario::Wave, true).unwrap();
    let before = c.digest().unwrap();
    assert!(c.step().is_err());
    assert_eq!(c.digest().unwrap(), before);
    assert!(c.rejected().unwrap().diagnostics().residual[0].abs() > 1e-9);
}
#[test]
fn repeated_remap_and_pending_refusal() {
    let mut c = uniform(Config::default());
    let total = c.state.diagnostics().total;
    for _ in 0..100 {
        c.refine(&[0, 3, 7]).unwrap();
        c.coarsen(&[0, 3, 7]).unwrap();
    }
    assert_eq!(c.state.diagnostics().total, total);
    c.refine(&[3]).unwrap();
    c.begin_subcycle().unwrap();
    let before = c.digest().unwrap();
    c.fail_before_commit = true;
    assert!(c.finish_subcycle().is_err());
    assert_eq!(c.digest().unwrap(), before);
    c.fail_before_commit = false;
    c.finish_subcycle().unwrap();
}
fn fidelity_spec(control: experiments::fidelity::Control) -> experiments::fidelity::Spec {
    experiments::fidelity::Spec {
        config: Config {
            roots: [4; 3],
            h: 0.25,
            dt: 0.0005,
            periodic: true,
            capacity: 10000,
            ..Default::default()
        },
        control,
        cycles: 4,
        refined: None,
        fine: false,
        cache: true,
        subcycle: false,
    }
}
#[test]
fn cached_and_rebuilt_topology_match_after_mesh_changes() {
    let spec = fidelity_spec(experiments::fidelity::Control::Shear3);
    let mut a = experiments::fidelity::chamber(&spec).unwrap();
    let mut b = experiments::fidelity::chamber(&spec).unwrap();
    let mut w = topology::Workspace::default();
    for i in 0..8 {
        if i == 2 {
            a.refine(&[5]).unwrap();
            b.refine(&[5]).unwrap();
        }
        if i == 5 {
            a.coarsen(&[5]).unwrap();
            b.coarsen(&[5]).unwrap();
        }
        a.step().unwrap();
        b.step_cached(&mut w).unwrap();
        assert_eq!(a.digest().unwrap(), b.digest().unwrap());
    }
    assert_eq!(w.rebuilds, 3);
    assert_eq!(w.hits, 5);
}
#[test]
fn muscl_ssp2_and_manufactured_impulses_conserve() {
    let mut spec = fidelity_spec(experiments::fidelity::Control::Vortex);
    spec.config.reconstruction = Reconstruction::Minmod;
    spec.config.integrator = Integrator::Ssp2;
    spec.config.vortex_forcing = Some(VortexForcing {
        amplitude: 0.005,
        wavenumber: 2. * std::f64::consts::PI,
    });
    let mut c = experiments::fidelity::chamber(&spec).unwrap();
    for _ in 0..8 {
        c.step().unwrap();
    }
    c.state.validate().unwrap();
    assert!(
        c.state
            .diagnostics()
            .residual
            .iter()
            .all(|v| v.abs() < 1e-10)
    );
    assert!(c.begin_subcycle().is_err());
    let path = std::env::temp_dir().join(format!("h004-ssp2-{}.json", std::process::id()));
    c.save(&path).unwrap();
    let mut other = Chamber::load(&path).unwrap();
    c.step().unwrap();
    other.step().unwrap();
    assert_eq!(c.digest().unwrap(), other.digest().unwrap());
    std::fs::remove_file(path).unwrap();
}
#[test]
fn reconstructed_negative_density_is_refused() {
    let c = uniform(Config::default());
    let faces = mesh::faces(&c.state).unwrap();
    let mut g = reconstruction::slopes(&c.state, &faces);
    g.get_mut(&0).unwrap()[0][0] = -1e6;
    assert!(reconstruction::at_face(&c.state, &g, 0, 16, 0, true).is_err());
    assert_eq!(reconstruction::minmod(2., 3.), 2.);
    assert_eq!(reconstruction::minmod(-2., 3.), 0.);
}
#[test]
fn control_schema_rejects_missing_scalar_and_source() {
    let spec = fidelity_spec(experiments::fidelity::Control::Diffusion3);
    assert!(spec.validate().is_err());
    let spec = fidelity_spec(experiments::fidelity::Control::Vortex);
    assert!(spec.validate().is_err());
}
#[test]
fn manufactured_source_global_impulse_has_independent_integral() {
    // A bounded external source on the periodic box, not an exact vortex reference.
    let cfg = Config {
        roots: [4; 3],
        h: 0.25,
        dt: 0.001,
        periodic: true,
        vortex_forcing: Some(VortexForcing {
            amplitude: 0.005,
            wavenumber: 1.,
        }),
        ..Default::default()
    };
    let mut c = uniform(cfg);
    c.step().unwrap();
    let expected =
        20. * 0.005f64.powi(2) * 0.5 * 1f64.sin().powi(2) * (0.5 + 2f64.sin() / 4.) * 0.001;
    assert!(expected > 1e-8);
    for k in 0..2 {
        assert!((c.state.ledger.manufactured_impulse[k] - expected).abs() < 1e-18);
        assert!((c.state.diagnostics().total[k + 1] - expected).abs() < 1e-15);
    }
    c.state.validate().unwrap();
}
#[test]
fn minmod_retains_small_finite_gradients() {
    assert_eq!(reconstruction::minmod(1e-200, 2e-200), 1e-200);
}
#[test]
fn viscous_linear_reference_preserves_initial_wave_mean_momentum() {
    let spec = fidelity_spec(experiments::fidelity::Control::Oblique);
    let mut totals = Vec::new();
    for time in [0., 0.05] {
        let mut q = [0.; NF];
        for root in 0..spec.config.count() {
            let g = mesh::geometry(&spec.config, (root as u64) << 4);
            let u = experiments::fidelity::reference_average(&spec, g, time);
            for i in 0..NF {
                q[i] += u[i] * g.volume();
            }
        }
        totals.push(q);
    }
    let expected = spec.config.density * spec.config.sound_speed / 3f64.sqrt() * 0.5e-8;
    for q in totals {
        assert!((q[0] - 20.).abs() < 1e-12);
        for p in &q[1..4] {
            assert!((*p - expected).abs() < 1e-15);
        }
    }
}
#[test]
fn oblique_reference_includes_viscous_acoustic_decay() {
    let spec = fidelity_spec(experiments::fidelity::Control::Oblique);
    let energy = |t| {
        let mut energy = 0.;
        for root in 0..spec.config.count() {
            let g = mesh::geometry(&spec.config, (root as u64) << 4);
            let q = experiments::fidelity::reference_average(&spec, g, t);
            let dr = q[0] - spec.config.density;
            let momentum2 = q[1..4].iter().map(|p| p * p).sum::<f64>();
            energy += 0.5 * (spec.config.sound_speed.powi(2) * dr * dr + momentum2)
                / spec.config.density
                * g.volume();
        }
        energy
    };
    assert!(energy(0.05) < 0.99 * energy(0.));
}
