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
