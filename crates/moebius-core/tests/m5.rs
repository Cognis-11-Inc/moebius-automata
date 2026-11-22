use moebius_core::{SimConfig, SnapshotFile, World};
use std::collections::BTreeMap;
fn config() -> SimConfig {
    let mut c = SimConfig::nursery_farm_256();
    c.width = 16;
    c.height = 16;
    c.depth = 16;
    c.initial_population = 1;
    c.initial_reservoir = 0;
    c.inflow_per_tick = 0;
    c.inject_per_tick = 0;
    c.feeding.as_mut().unwrap().initial_stock = 0;
    c.farm.as_mut().unwrap().max_population = 1;
    c
}
#[test]
fn empty_diagnostic_input_preserves_production_transitions_and_sensors() {
    let c = config();
    let mut a = World::genesis(&c, 1).unwrap();
    let mut b = a.clone();
    for _ in 0..64 {
        let sensors = a.organism_sensors(1).unwrap();
        let (ra, rows) = a.tick_observed().unwrap();
        let (rb, other) = b.tick_controlled(&BTreeMap::new()).unwrap();
        assert_eq!(ra, rb);
        assert_eq!(rows[0].sensors, sensors);
        assert_eq!(rows[0].commands, other[0].commands);
        assert_eq!(a.snapshot().state_hash(), b.snapshot().state_hash());
    }
}
#[test]
fn invalid_diagnostic_commands_are_atomic_and_not_silently_clamped() {
    let c = config();
    let mut w = World::genesis(&c, 1).unwrap();
    let before = w.snapshot().state_hash();
    for input in [
        BTreeMap::from([(1, vec![0; 29])]),
        BTreeMap::from([(99, vec![0; 30])]),
        BTreeMap::from([(1, vec![1025; 30])]),
    ] {
        assert!(w.tick_controlled(&input).is_err());
        assert_eq!(before, w.snapshot().state_hash());
    }
    let mut c = c;
    c.neural.as_mut().unwrap().version = 1;
    let mut old = World::genesis(&c, 1).unwrap();
    assert!(old.tick_controlled(&BTreeMap::new()).is_err());
}
#[test]
fn controlled_commands_pay_costs_and_replay_across_snapshot_boundary() {
    let c = config();
    let mut a = World::genesis(&c, 3).unwrap();
    let mut inputs = Vec::new();
    let mut fees = 0;
    for tick in 0..32 {
        let mut cmd = vec![0; 30];
        for i in (0..24).step_by(2) {
            cmd[i] = if tick % 16 < 8 { 1024 } else { 0 };
        }
        let input = BTreeMap::from([(1, cmd)]);
        inputs.push(input.clone());
        let (r, rows) = a.tick_controlled(&input).unwrap();
        fees += r.motor_energy;
        assert_eq!(
            rows[0].energy_before + rows[0].gain,
            rows[0].energy_after + rows[0].actuator + rows[0].neural + rows[0].maintenance
        );
    }
    assert!(fees > 0);
    let file = SnapshotFile::decode(&SnapshotFile::encode(&c, &a).unwrap()).unwrap();
    let mut b = World::restore(&c, &file.snapshot).unwrap();
    for input in &inputs {
        a.tick_controlled(input).unwrap();
        b.tick_controlled(input).unwrap();
    }
    assert_eq!(a.snapshot().state_hash(), b.snapshot().state_hash());
    assert_eq!(a.accounts().unwrap().energy_residual(), 0);
    assert_eq!(a.accounts().unwrap().material_residual(), 0);
}
fn sensor_fixture(
    mode: moebius_core::neural::ChemicalSensing,
    offset: [i32; 3],
    orientation: [i32; 3],
    side: u32,
) -> World {
    let mut c = config();
    c.width = side;
    c.height = side;
    c.depth = side;
    c.feeding.as_mut().unwrap().spacing = 8;
    c.neural.as_mut().unwrap().chemical_sensing = mode;
    let mut s = World::genesis(&c, 1).unwrap().snapshot();
    let center = side / 2;
    s.cells[0].site = moebius_core::Site {
        x: center,
        y: center,
        z: center,
    };
    s.brains[0].neural.as_mut().unwrap().body.orientation = orientation;
    let p: [u32; 3] = std::array::from_fn(|a| {
        (i64::from(center) + i64::from(offset[a])).rem_euclid(i64::from(side)) as u32
    });
    s.nutrient[(p[0] + side * (p[1] + side * p[2])) as usize] = 512;
    s.material_initial = 512;
    World::restore(&c, &s).unwrap()
}
#[test]
fn bounded_directional_sensor_observes_off_ray_food_and_rotates_channels() {
    use moebius_core::neural::ChemicalSensing::{LocalLobesV1, RaysV1};
    let rays = sensor_fixture(RaysV1, [2, -1, 1], [0; 3], 32)
        .organism_sensors(1)
        .unwrap();
    let lobes = sensor_fixture(LocalLobesV1, [2, -1, 1], [0; 3], 32)
        .organism_sensors(1)
        .unwrap();
    assert!(rays[93..99].iter().all(|v| *v == 0));
    assert!(lobes[93] > 0 && lobes[96] > 0 && lobes[97] > 0);
    assert_eq!([lobes[94], lobes[95], lobes[98]], [0; 3]);
    let rotated = sensor_fixture(LocalLobesV1, [2, -1, 1], [0, 0, 1608], 32)
        .organism_sensors(1)
        .unwrap();
    assert!(rotated[94] > 0 && rotated[96] > 0 && rotated[97] > 0);
    assert_eq!(rotated[93], 0);
}
#[test]
fn bounded_sensor_respects_range_stock_ablation_and_unique_periodic_sites() {
    use moebius_core::neural::ChemicalSensing::LocalLobesV1;
    let far = sensor_fixture(LocalLobesV1, [9, 0, 0], [0; 3], 32)
        .organism_sensors(1)
        .unwrap();
    assert!(far[93..99].iter().all(|v| *v == 0));
    let at_edge = sensor_fixture(LocalLobesV1, [8, 0, 0], [0; 3], 32)
        .organism_sensors(1)
        .unwrap();
    assert!(at_edge[93] > 0);
    let tie = sensor_fixture(LocalLobesV1, [8, 0, 0], [0; 3], 16)
        .organism_sensors(1)
        .unwrap();
    assert_eq!(tie[93], 0);
    assert_eq!(
        tie[94], at_edge[93],
        "unique minimum-image site, negative half-axis tie"
    );
    let w = sensor_fixture(LocalLobesV1, [2, -1, 1], [0; 3], 32);
    let mut snapshot = w.snapshot();
    snapshot.nutrient.fill(0);
    snapshot.material_initial = 0;
    let absent = World::restore(w.config(), &snapshot)
        .unwrap()
        .organism_sensors(1)
        .unwrap();
    assert!(absent[93..99].iter().all(|v| *v == 0));
    let mut config = w.config().clone();
    config.neural.as_mut().unwrap().chemistry = false;
    let ablated = World::restore(&config, &w.snapshot())
        .unwrap()
        .organism_sensors(1)
        .unwrap();
    assert!(ablated[93..99].iter().all(|v| *v == 0));
}
#[test]
fn sensor_variant_preserves_old_encoding_and_does_not_change_paid_physics() {
    use moebius_core::neural::{ChemicalSensing, NeuralRules};
    let encoded = serde_json::to_value(NeuralRules::default()).unwrap();
    assert!(encoded.get("chemical_sensing").is_none());
    let decoded: NeuralRules = serde_json::from_value(encoded).unwrap();
    assert_eq!(decoded.chemical_sensing, ChemicalSensing::RaysV1);
    let mut old = decoded;
    old.chemical_sensing = ChemicalSensing::LocalLobesV1;
    assert!(old.validate().is_err());
    let mut a = sensor_fixture(ChemicalSensing::RaysV1, [2, -1, 1], [0; 3], 32);
    let mut changed = a.config().clone();
    changed.neural.as_mut().unwrap().chemical_sensing = ChemicalSensing::LocalLobesV1;
    let mut b = World::restore(&changed, &a.snapshot()).unwrap();
    for tick in 0..24 {
        let mut cmd = vec![0; 30];
        for i in (0..24).step_by(2) {
            cmd[i] = if tick < 8 { 1024 } else { 0 };
        }
        let controls = BTreeMap::from([(1, cmd)]);
        let (ra, oa) = a.tick_controlled(&controls).unwrap();
        let (rb, ob) = b.tick_controlled(&controls).unwrap();
        assert_eq!(ra, rb);
        assert_eq!(oa[0].position_after, ob[0].position_after);
        assert_eq!(oa[0].angles, ob[0].angles);
        assert_eq!(oa[0].energy_after, ob[0].energy_after);
    }
}
#[allow(dead_code)]
#[path = "../examples/support/m5_control.rs"]
mod control;
#[allow(dead_code)]
#[path = "../examples/support/m5.rs"]
mod support;
#[test]
fn preserved_paid_commands_reach_a_distinct_second_meal_under_original_budget() {
    let record: serde_json::Value = serde_json::from_str(include_str!(
        "../../../docs/m5/REGRESSION_REAR_COMMANDS.json"
    ))
    .unwrap();
    let case = support::Case {
        name: "regression".into(),
        seed: 1,
        drag: 8,
        energy: 128,
        target_delta: Some([0, -3, 0]),
        founding_bite: true,
        horizon: 128,
    };
    let (mut w, target) = support::fixture(&case).unwrap();
    let mut first = None;
    let mut reached = None;
    for (tick, input) in record["commands"].as_array().unwrap().iter().enumerate() {
        let command: Vec<i32> = serde_json::from_value(input.clone()).unwrap();
        let (_, row) = support::step(&mut w, Some(&command)).unwrap();
        if tick == 0 {
            first = row.food_site;
            assert_eq!(row.intake, 1);
        }
        if row.intake > 0 && row.food_site == target {
            reached = Some(tick);
            assert_ne!(first, target);
        }
        assert_eq!(w.accounts().unwrap().energy_residual(), 0);
        assert_eq!(w.accounts().unwrap().material_residual(), 0);
    }
    assert_eq!(
        reached,
        Some(record["expected_first_target_tick"].as_u64().unwrap() as usize)
    );
    assert_eq!(w.config().initial_energy, 128);
    assert_eq!(w.config().maintenance_cost, 1);
}
#[test]
fn sensor_only_model_can_reach_aligned_food_without_target_input() {
    let case = support::Case {
        name: "local_regression".into(),
        seed: 1,
        drag: 8,
        energy: 128,
        target_delta: Some([0, -3, 0]),
        founding_bite: false,
        horizon: 128,
    };
    let (mut w, target) = support::fixture(&case).unwrap();
    let mut controller = control::LocalModelController::default();
    let mut reached = false;
    for _ in 0..128 {
        if w.live_count() == 0 {
            break;
        }
        let command = controller.commands(&w.organism_sensors(1).unwrap());
        let (_, row) = support::step(&mut w, Some(&command)).unwrap();
        if row.intake > 0 && row.food_site == target {
            reached = true;
            break;
        }
    }
    assert!(reached);
}
#[test]
fn ordinary_ray_blind_fixture_and_food_absent_fixture_are_indistinguishable_to_local_model() {
    let case = support::Case {
        name: "blind".into(),
        seed: 1,
        drag: 8,
        energy: 128,
        target_delta: Some([2, -2, 1]),
        founding_bite: false,
        horizon: 128,
    };
    let (mut food, _) = support::fixture(&case).unwrap();
    let mut empty_case = case.clone();
    empty_case.target_delta = None;
    let (mut empty, _) = support::fixture(&empty_case).unwrap();
    let mut a = control::LocalModelController::default();
    let mut b = control::LocalModelController::default();
    for _ in 0..16 {
        let sa = food.organism_sensors(1).unwrap();
        let sb = empty.organism_sensors(1).unwrap();
        assert_eq!(sa, sb);
        let ca = a.commands(&sa);
        let cb = b.commands(&sb);
        assert_eq!(ca, cb);
        let (_, ra) = support::step(&mut food, Some(&ca)).unwrap();
        let (_, rb) = support::step(&mut empty, Some(&cb)).unwrap();
        assert_eq!(ra.position_after, rb.position_after);
    }
}
