use super::*;
fn compact(cells: usize) -> Chamber {
    let mut c = Chamber::new(Config {
        cells,
        gravity_m_s2: [0.; 3],
        ..Default::default()
    })
    .unwrap();
    let h = c.state.config.h();
    c.initialize_velocity(|p| {
        if p[0] < 4. * h && p[1] < 4. * h && p[2] < 4. * h {
            [0.001, 0.0005, 0.]
        } else {
            [0.; 3]
        }
    })
    .unwrap();
    c.add_species(
        Species {
            material: "probe".into(),
            radius_m: 0.001,
            density_kg_m3: 20.,
            diffusivity_m2_s: 0.0001,
        },
        |p| {
            if p.iter().all(|v| *v < 4. * h) {
                1e-9
            } else {
                0.
            }
        },
    )
    .unwrap();
    c.seal().unwrap();
    c
}
#[test]
fn buffered_and_each_block_edge_preserve_each_physical_bit() {
    for edge in [4, 8, 16, 32] {
        let mut a = compact(8);
        let mut b = compact(8);
        let mut f = compact(8);
        let mut wf = Workspace::default();
        let mut c = compact(8);
        c.use_blocks(edge).unwrap();
        let mut wb = Workspace::default();
        let mut wc = Workspace::default();
        for _ in 0..24 {
            a.step().unwrap();
            b.step_candidate(Lane::Buffered, &mut wb).unwrap();
            c.step_candidate(Lane::Blocks, &mut wc).unwrap();
            assert!(a.physical_eq(&b));
            assert!(a.physical_eq(&c));
            f.step_candidate(Lane::InPlace, &mut wf).unwrap();
            assert!(a.physical_eq(&f));
        }
    }
}
#[test]
fn reused_scratch_refusal_preserves_commit_and_can_recover() {
    let mut a = compact(8);
    let mut b = compact(8);
    let before = a.digest().unwrap();
    let mut w = Workspace {
        max_cells: Some(1),
        ..Default::default()
    };
    assert!(a.step_candidate(Lane::Buffered, &mut w).is_err());
    assert_eq!(before, a.digest().unwrap());
    w.max_cells = None;
    w.fail_before_commit = true;
    assert!(a.step_candidate(Lane::Buffered, &mut w).is_err());
    assert_eq!(before, a.digest().unwrap());
    w.fail_before_commit = false;
    a.step_candidate(Lane::Buffered, &mut w).unwrap();
    b.step().unwrap();
    assert!(a.physical_eq(&b));
    a.state.fields[0].species.diffusivity_m2_s = 0.01;
    a.state.config.substeps = 1;
    a.state.config.length_m = 0.1;
    let before = a.digest().unwrap();
    assert!(a.step_candidate(Lane::Buffered, &mut w).is_err());
    assert_eq!(before, a.digest().unwrap());
}
#[test]
fn implicit_background_preserves_roundoff_and_diagonal_activation() {
    let mut a = compact(16);
    let mut b = compact(16);
    let mut w = Workspace::default();
    for _ in 0..12 {
        a.step().unwrap();
        b.step_candidate(Lane::ImplicitCarrier, &mut w).unwrap();
        assert!(a.physical_eq(&b), "tick{}", a.state.tick);
    }
    let mut a = Chamber::new(Config {
        cells: 8,
        ..Default::default()
    })
    .unwrap();
    let mut b = Chamber::new(a.state.config.clone()).unwrap();
    let mut w = Workspace::default();
    for _ in 0..24 {
        a.step().unwrap();
        b.step_candidate(Lane::ImplicitCarrier, &mut w).unwrap();
        assert!(a.physical_eq(&b));
        assert_eq!(w.activity.active_carrier_blocks, 0);
    }
}
#[test]
fn non_equilibrium_zero_velocity_cannot_be_retired() {
    let mut a = Chamber::new(Config {
        cells: 8,
        gravity_m_s2: [0.; 3],
        ..Default::default()
    })
    .unwrap();
    let cell = a.state.config.index([3, 3, 3]);
    a.state.populations[cell][1] += 1e-5;
    a.state.populations[cell][2] += 1e-5;
    a.state.populations[cell][0] -= 2e-5;
    a.seal().unwrap();
    let mut b = Chamber {
        state: a.state.clone(),
    };
    let mut w = Workspace::default();
    a.step().unwrap();
    b.step_candidate(Lane::ImplicitCarrier, &mut w).unwrap();
    assert!(a.physical_eq(&b));
    assert!(w.activity.active_carrier_blocks > 0);
}
#[test]
fn blocks_retire_only_exact_zero_and_preserve_subnormal_tail() {
    let mut b = BlockMass::new(8, 4);
    b.add(1, f64::from_bits(1));
    assert_eq!(b.retire_zero(), 0);
    assert_eq!(b.get(1).to_bits(), 1);
    b.add(1, -f64::from_bits(1));
    assert_eq!(b.retire_zero(), 1);
    b.add(1, 1e-9);
    assert_eq!(b.get(1), 1e-9);
    assert!(b.validate());
}
#[test]
fn block_checkpoint_and_workspace_continuation_are_source_bound() {
    let mut a = compact(8);
    a.use_blocks(4).unwrap();
    let mut w = Workspace::default();
    a.step_candidate(Lane::Blocks, &mut w).unwrap();
    let path = std::env::temp_dir().join(format!("h002-{}-block.json", std::process::id()));
    a.save_candidate(&path, Lane::Blocks, 1).unwrap();
    assert!(Chamber::load_candidate(&path, Lane::Buffered, 1).is_err());
    assert!(Chamber::load_candidate(&path, Lane::Blocks, 2).is_err());
    assert!(Chamber::load(&path).is_err());
    let mut b = Chamber::load_candidate(&path, Lane::Blocks, 1).unwrap();
    let mut fresh = Workspace::default();
    for _ in 0..3 {
        a.step_candidate(Lane::Blocks, &mut w).unwrap();
        b.step_candidate(Lane::Blocks, &mut fresh).unwrap();
        assert!(a.physical_eq(&b));
    }
    std::fs::remove_file(path).unwrap();
}
#[test]
fn naive_in_place_push_is_a_counterexample_not_a_backend() {
    let mut f = vec![[0.; 19]; 8];
    for (i, v) in f.iter_mut().enumerate() {
        v[1] = i as f64 + 1.;
    }
    let old = f.clone();
    for i in 0..8 {
        f[(i + 1) % 8][1] = f[i][1];
    }
    assert!((0..8).any(|i| f[(i + 1) % 8][1].to_bits() != old[i][1].to_bits()));
}

#[test]
fn multicore_collision_preserves_stream_and_wall_order() {
    let mut a = compact(8);
    let mut b = compact(8);
    let mut w = Workspace::with_workers(4);
    for _ in 0..12 {
        a.step().unwrap();
        b.step_candidate(Lane::Buffered, &mut w).unwrap();
        assert!(a.physical_eq(&b));
    }
}
#[test]
fn observer_and_attribution_never_mutate_or_select_next_state() {
    let a = compact(8);
    let before = a.digest().unwrap();
    let report = a.transport_attribution().unwrap();
    assert!(!report.phases.is_empty());
    assert_eq!(before, a.digest().unwrap());
}
