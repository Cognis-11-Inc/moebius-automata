use moebius_core::{
    SimConfig, SnapshotFile, World,
    neural::{ControllerGenome, NeuralRules, NeuralState, Q, schema::Anatomy},
};
fn config() -> SimConfig {
    serde_json::from_str(include_str!("../../../experiments/nursery_smoke.json")).unwrap()
}
#[test]
fn observation_preserves_transitions_accounts_deaths_births_and_restore() {
    let c = config();
    let mut observed = World::genesis(&c, 3).unwrap();
    let mut plain = observed.clone();
    let mut births = 0;
    for _ in 0..192 {
        let (a, rows) = observed.tick_observed().unwrap();
        let b = plain.tick().unwrap();
        assert_eq!(a, b);
        assert_eq!(
            observed.snapshot().state_hash(),
            plain.snapshot().state_hash()
        );
        assert_eq!(rows.iter().map(|r| r.gain).sum::<u64>(), a.energy_credited);
        assert_eq!(rows.iter().map(|r| r.actuator).sum::<u64>(), a.motor_energy);
        assert_eq!(rows.iter().map(|r| r.neural).sum::<u64>(), a.neural_energy);
        for r in rows {
            assert_eq!(
                r.energy_before + r.gain,
                r.energy_after
                    + r.neural
                    + r.actuator
                    + r.maintenance
                    + r.secretion
                    + r.digging
                    + r.birth_out
                    + r.death_disposal
            );
        }
        births += a.births;
    }
    assert!(births > 0);
    let snap = SnapshotFile::decode(&SnapshotFile::encode(&c, &observed).unwrap()).unwrap();
    let mut restored = World::restore(&c, &snap.snapshot).unwrap();
    for _ in 0..16 {
        observed.tick_observed().unwrap();
        restored.tick_observed().unwrap();
    }
    assert_eq!(
        observed.snapshot().state_hash(),
        restored.snapshot().state_hash()
    );
    let mut c = config();
    c.initial_energy = 1;
    c.feeding.as_mut().unwrap().initial_stock = 0;
    c.inflow_per_tick = 0;
    c.inject_per_tick = 0;
    let mut w = World::genesis(&c, 1).unwrap();
    let mut deaths = 0;
    for _ in 0..8 {
        let (_, rows) = w.tick_observed().unwrap();
        deaths += rows.iter().filter(|r| r.died).count();
    }
    assert_eq!(deaths, 12);
}
#[test]
fn wave_ablation_preserves_neural_commands_but_removes_scripted_drive() {
    let mut g = ControllerGenome::founder(4, 1, Anatomy::reference());
    g.output.fill(0);
    g.motor_biases.fill(0);
    let mut on = NeuralState::new(g.clone(), 128);
    let mut off = on.clone();
    let rules = NeuralRules::default();
    let no_wave = NeuralRules {
        spinal_wave: false,
        ..rules.clone()
    };
    for tick in 0..16 {
        let sensors = vec![0; g.anatomy.sensors()];
        on.integrate(sensors.clone(), &rules, 1, 1, tick, 128);
        off.integrate(sensors, &no_wave, 1, 1, tick, 128);
        assert_eq!(on.commands, off.commands);
        assert!(off.actuator_drive().iter().all(|v| *v == 0));
    }
    assert!(on.actuator_drive().iter().any(|v| v.abs() > Q / 2));
    let encoded = serde_json::to_value(&rules).unwrap();
    assert!(
        encoded.get("spinal_wave").is_none(),
        "legacy encoding must stay unchanged"
    );
    let decoded: NeuralRules = serde_json::from_value(encoded).unwrap();
    assert!(decoded.spinal_wave);
}
#[test]
fn off_ray_food_is_invisible_to_chemical_channels() {
    let c = config();
    let mut s = World::genesis(&c, 1).unwrap().snapshot();
    s.nutrient.fill(0);
    s.reservoir = 0;
    // All founders except #1 removed, accounting updated to fixture.
    s.cells.truncate(1);
    s.brains.truncate(1);
    s.cells[0].site = moebius_core::Site {
        x: 16,
        y: 16,
        z: 16,
    };
    s.energy_credited = s.cells[0].energy;
    s.material_initial = 512;
    s.nutrient[18 + 32 * (18 + 32 * 16)] = 512;
    let mut c = c;
    c.inflow_per_tick = 0;
    c.inject_per_tick = 0;
    let mut w = World::restore(&c, &s).unwrap();
    let (_, rows) = w.tick_observed().unwrap();
    let sensors = &rows[0].sensors;
    let base = 26 * 3 + 15;
    assert!(sensors[base..base + 6].iter().all(|v| *v == 0));
}
#[test]
fn v2_bills_scripted_actuation_and_rejects_unfunded_drive() {
    let mut genome = ControllerGenome::founder(2, 1, Anatomy::reference());
    genome.output.fill(0);
    genome.motor_biases.fill(0);
    let rules = NeuralRules {
        version: 2,
        ..NeuralRules::default()
    };
    let mut state = NeuralState::new(genome.clone(), 128);
    let sensors = vec![0; genome.anatomy.sensors()];
    let mut total = 0;
    for t in 0..32 {
        total += state.integrate(sensors.clone(), &rules, 1, 1, t, 128).1;
    }
    assert!(total > 0, "a scripted wave must pay for its drive");
    state.integrate(sensors.clone(), &rules, 1, 1, 32, 0);
    assert!(state.actuator_drive().iter().all(|v| *v == 0));
    let mut state = NeuralState::new(genome, 1);
    state.steps = 7;
    state.actuator_debt = 4095;
    let fees = state.integrate(sensors, &rules, 1, 1, 0, 1);
    assert_eq!(fees, (1, 0));
    assert!(state.actuator_drive().iter().all(|v| *v == 0));
}
#[test]
fn v2_joint_rates_are_angle_changes_including_at_stops() {
    use moebius_core::neural::body::Body;
    let anatomy = Anatomy::reference();
    let mut body = Body::new(&anatomy);
    let commands = vec![Q; anatomy.motors()];
    let strengths = commands.clone();
    let mut environment = [0; 3];
    for _ in 0..96 {
        let previous = body.angles.clone();
        let reaction = body.step_with_law(&anatomy, &commands, &strengths, 8, &[], 6, &[], 2);
        for a in 0..3 {
            environment[a] += reaction[a];
            assert_eq!(environment[a] + body.momentum[a], 0);
        }
        for (i, angle) in body.angles.iter().enumerate() {
            assert_eq!(body.rates[i], angle - previous[i]);
        }
        body.validate(&anatomy).unwrap();
    }
    assert!(
        body.rates.iter().all(|r| *r == 0),
        "held stops must not generate fictitious motion"
    );
    // Constant translation in a drag-bearing medium must dissipate to its ledger;
    // a completely still pose may not keep adding thrust.
    let before = body.momentum.map(|v| v.abs()).iter().sum::<i64>();
    body.step_with_law(&anatomy, &commands, &strengths, 8, &[], 6, &[], 2);
    assert!(body.momentum.map(|v| v.abs()).iter().sum::<i64>() <= before);
    let mut vacuum = Body::new(&anatomy);
    for _ in 0..32 {
        vacuum.step_with_law(&anatomy, &commands, &strengths, 0, &[], 6, &[], 2);
    }
    assert_eq!(vacuum.momentum, [0; 3]);
}
#[test]
fn v2_observation_and_restore_are_deterministic() {
    let mut c = config();
    c.neural.as_mut().unwrap().version = 2;
    let mut a = World::genesis(&c, 1).unwrap();
    for _ in 0..128 {
        a.tick_observed().unwrap();
    }
    let file = SnapshotFile::decode(&SnapshotFile::encode(&c, &a).unwrap()).unwrap();
    let mut b = World::restore(&c, &file.snapshot).unwrap();
    for _ in 0..32 {
        a.tick().unwrap();
        b.tick_observed().unwrap();
    }
    assert_eq!(a.snapshot().state_hash(), b.snapshot().state_hash());
    assert_eq!(a.accounts().unwrap().energy_residual(), 0);
    assert_eq!(a.accounts().unwrap().material_residual(), 0);
}
#[test]
fn nursery_default_uses_explicit_corrected_laws_without_scripted_wave() {
    let c = SimConfig::nursery_farm_256();
    let file: SimConfig =
        serde_json::from_str(include_str!("../../../experiments/nursery_m4_v2.json")).unwrap();
    assert_eq!(c, file);
    let rules = c.neural.unwrap();
    assert_eq!(rules.version, 2);
    assert!(!rules.spinal_wave);
    assert_eq!(
        NeuralRules::default().version,
        1,
        "legacy neural experiments retain v1"
    );
}
