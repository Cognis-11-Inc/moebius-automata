use moebius_core::neural::{
    ControllerGenome, NeuralRules, NeuralState, Q, body::Body, schema::Anatomy,
};
use moebius_core::{SimConfig, SnapshotFile, Stocks, World, apply_receipt, audit_with_config};
fn config() -> SimConfig {
    let mut c = SimConfig::neural_farm_256();
    c.width = 8;
    c.height = 8;
    c.depth = 8;
    c.initial_population = 3;
    c.initial_energy = 200;
    c.inject_per_tick = 8;
    c.inflow_per_tick = 8;
    c.initial_nutrient_per_site = 20;
    c.nutrient_wave_amplitude = 0;
    c.farm.as_mut().unwrap().max_population = 24;
    c.farm.as_mut().unwrap().birth_energy = 60;
    c.farm.as_mut().unwrap().maturity_ticks = 16;
    c.farm.as_mut().unwrap().birth_cooldown = 16;
    c
}
#[test]
fn recurrent_state_and_simultaneous_continuous_axis_outputs() {
    let g = ControllerGenome::founder(11, 1, Anatomy::reference());
    let mut state = NeuralState::new(g, 1000);
    for (i, bias) in state.genome.motor_biases.iter_mut().enumerate() {
        *bias = (i as i32 * 67 % 1800) - 900;
    }
    let sensor = vec![0; state.genome.anatomy.sensors()];
    state.integrate(sensor.clone(), &NeuralRules::default(), 1, 1, 0, 1000);
    let values = state.commands[..state.genome.anatomy.axes.len()]
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    assert!(values.len() > 8);
    assert!(values.iter().any(|v| *v > 0));
    assert!(values.iter().any(|v| *v < 0));
    let mut other = state.clone();
    other.activations.fill(5000);
    other.integrate(sensor.clone(), &NeuralRules::default(), 1, 1, 1, 1000);
    state.integrate(sensor, &NeuralRules::default(), 1, 1, 1, 1000);
    assert_ne!(state.commands, other.commands);
    state.validate().unwrap();
    other.validate().unwrap();
}
#[test]
fn torque_has_rates_limits_and_no_vacuum_com_thrust() {
    let a = Anatomy::reference();
    let mut body = Body::new(&a);
    let motor = vec![Q; a.motors()];
    for _ in 0..128 {
        assert_eq!(
            body.step(
                &a,
                &motor,
                &vec![Q; a.motors()],
                0,
                &vec![0; a.axes.len()],
                6,
                &[]
            ),
            [0; 3]
        );
        body.validate(&a).unwrap();
        assert_eq!(body.momentum, [0; 3]);
        assert_eq!(body.velocity, [0; 3]);
    }
    assert!(body.angles.iter().any(|p| *p != 0));
    assert!(body.angular_velocity.iter().all(|v| v.unsigned_abs() < 128));
}
#[test]
fn learned_state_is_not_germline_and_offspring_reset_it() {
    let mut state = NeuralState::new(ControllerGenome::founder(22, 2, Anatomy::reference()), 100);
    let genome = state.genome.clone();
    state.activations.fill(3000);
    for step in 0..256 {
        state.feedback(
            if step % 2 == 0 { 105 } else { 100 },
            1,
            &NeuralRules::default(),
        );
    }
    assert!(state.plastic.iter().any(|w| *w != 0));
    assert_eq!(state.genome, genome);
    let child = state.offspring(9, 7, 10, 50, false);
    assert_eq!(child.genome, genome);
    assert!(child.plastic.iter().all(|w| *w == 0));
    assert!(child.activations.iter().all(|w| *w == 0));
    let a = state.offspring(9, 7, 10, 50, true);
    let b = state.offspring(9, 7, 10, 50, true);
    assert_eq!(a, b);
    assert_ne!(a.genome, genome);
    a.validate().unwrap();
}
#[test]
fn mappings_validate_other_compatible_anatomies_without_controller_rewrite() {
    for (spine, jaws) in [(3, 0), (7, 1), (12, 2), (24, 2)] {
        let anatomy = Anatomy::chain(spine, jaws).unwrap();
        assert_eq!(anatomy.sensors(), anatomy.sensor_names().len());
        let mut state = NeuralState::new(ControllerGenome::founder(11, 1, anatomy), 1000);
        state.integrate(
            vec![0; state.genome.anatomy.sensors()],
            &NeuralRules::default(),
            11,
            1,
            0,
            1000,
        );
        state.validate().unwrap();
    }
}
#[test]
fn world_replay_material_and_reaction_momentum_survive_reproduction() {
    let c = config();
    let world = World::genesis(&c, 7).unwrap();
    let mut snapshot = world.snapshot();
    // Commanded valve fixture tests mechanics and persistence, not discovery.
    for brain in &mut snapshot.brains {
        let state = brain.neural.as_mut().unwrap();
        state.genome.output.fill(0);
        state.genome.motor_biases.fill(0);
        let count = state.genome.anatomy.axes.len();
        state.genome.motor_biases[count] = 1024;
        state.genome.motor_biases[count + 1] = 1024;
    }
    let mut world = World::restore(&c, &snapshot).unwrap();
    let mut launches = 0;
    let mut births = 0;
    for _ in 0..80 {
        let mut stocks = Stocks::from(&world.accounts().unwrap());
        let receipt = world.tick().unwrap();
        launches += receipt.jet_released;
        births += receipt.births;
        apply_receipt(&mut stocks, &receipt).unwrap();
        assert_eq!(stocks, Stocks::from(&world.accounts().unwrap()));
        assert_eq!(world.accounts().unwrap().material_residual(), 0);
        assert_eq!(world.accounts().unwrap().energy_residual(), 0);
    }
    assert!(launches > 0);
    assert!(births > 0);
    let file = SnapshotFile::decode(&SnapshotFile::encode(&c, &world).unwrap()).unwrap();
    audit_with_config(&c, &file).unwrap();
    let mut restored = World::restore(&c, &file.snapshot).unwrap();
    for _ in 0..32 {
        assert_eq!(world.tick().unwrap(), restored.tick().unwrap());
        assert_eq!(world.snapshot(), restored.snapshot());
    }
}
#[test]
fn plasticity_off_stays_fixed_and_neural_state_bounded_long_run() {
    let g = ControllerGenome::founder(4, 1, Anatomy::reference());
    let mut on = NeuralState::new(g, 1000);
    let mut off = on.clone();
    let no = NeuralRules {
        plasticity: false,
        ..NeuralRules::default()
    };
    for step in 0..4096 {
        let sensors: Vec<_> = (0..on.genome.anatomy.sensors())
            .map(|i| if (i + step) % 3 == 0 { Q } else { -Q })
            .collect();
        on.integrate(
            sensors.clone(),
            &NeuralRules::default(),
            1,
            1,
            step as u64,
            1000,
        );
        off.integrate(sensors, &no, 1, 1, step as u64, 1000);
        let energy = if step % 3 == 0 { 1010 } else { 1000 };
        on.feedback(energy, 1, &NeuralRules::default());
        off.feedback(energy, 1, &no);
        on.validate().unwrap();
        off.validate().unwrap();
    }
    assert!(off.plastic.iter().all(|w| *w == 0));
    assert!(on.plastic.iter().any(|w| *w != 0));
}
#[test]
fn neural_failure_is_atomic_and_empty_energy_never_actuates() {
    let mut c = config();
    c.reservoir_cap = c.initial_reservoir;
    let mut w = World::genesis(&c, 3).unwrap();
    let before = w.snapshot();
    assert!(w.tick().is_err());
    assert_eq!(w.snapshot(), before);
    let mut state = NeuralState::new(ControllerGenome::founder(11, 1, Anatomy::reference()), 0);
    assert_eq!(
        state.integrate(
            vec![Q; state.genome.anatomy.sensors()],
            &NeuralRules::default(),
            11,
            1,
            0,
            0
        ),
        (0, 0)
    );
    assert!(state.commands.iter().all(|x| *x == 0));
}

#[test]
fn recurrent_dynamics_can_oscillate_without_clock_or_input_waveform() {
    // Constructed capability probe, not a founder template or an organism gait.
    let mut g = ControllerGenome::founder(0, 0, Anatomy::reference());
    g.recurrent.fill(0);
    g.sensory.fill(0);
    g.biases.fill(0);
    g.output.fill(0);
    g.motor_biases.fill(0);
    g.tau.fill(8);
    g.recurrent[0] = 1536;
    g.recurrent[1] = -2048;
    g.recurrent[8] = 2048;
    g.recurrent[9] = 1536;
    g.output[0] = 1024;
    let mut state = NeuralState::new(g, 1000);
    state.activations[0] = 256;
    let rules = NeuralRules {
        plasticity: false,
        ..NeuralRules::default()
    };
    let mut signs = 0;
    let mut previous = 1;
    for step in 0..1024 {
        state.integrate(
            vec![0; state.genome.anatomy.sensors()],
            &rules,
            0,
            0,
            step,
            1000,
        );
        let sign = state.commands[0].signum();
        if sign != 0 && sign != previous {
            signs += 1;
            previous = sign;
        }
    }
    assert!(
        signs >= 8,
        "observed {signs} reversals under constant zero input"
    );
}

#[test]
fn acceleration_feedback_includes_recoil_and_integrated_drag() {
    let a = Anatomy::reference();
    let mut body = Body::new(&a);
    body.impulse([256, 0, 0]);
    assert_eq!(body.acceleration, [16, 0, 0]);
    let previous = body.velocity;
    body.step(
        &a,
        &vec![0; a.motors()],
        &vec![Q; a.motors()],
        8,
        &vec![0; a.axes.len()],
        6,
        &[],
    );
    body.impulse([512, 0, 0]);
    assert_eq!(body.acceleration[0], body.velocity[0] - previous[0]);
    body.validate(&a).unwrap();
}

#[test]
fn lifetime_experience_can_change_a_fresh_response_without_genetic_change() {
    let mut experienced =
        NeuralState::new(ControllerGenome::founder(22, 2, Anatomy::reference()), 100);
    let mut control = experienced.clone();
    experienced.activations.fill(3000);
    for step in 0..1024 {
        experienced.feedback(
            if step % 2 == 0 { 105 } else { 100 },
            1,
            &NeuralRules::default(),
        );
    }
    experienced.activations.fill(0);
    experienced.integration_residual.fill(0);
    let mut changed = false;
    for tick in 0..64 {
        let inputs = vec![Q / 2; experienced.genome.anatomy.sensors()];
        experienced.integrate(inputs.clone(), &NeuralRules::default(), 0, 0, tick, 1000);
        control.integrate(inputs, &NeuralRules::default(), 0, 0, tick, 1000);
        changed |= experienced.commands != control.commands;
    }
    assert_eq!(experienced.genome, control.genome);
    assert!(changed);
}

#[test]
fn a_bent_joint_eases_back_instead_of_holding_a_servo_stop() {
    let anatomy = Anatomy::reference();
    let mut body = Body::new(&anatomy);
    body.angles[0] = 700;
    body.rates[0] = 20;
    let commands = vec![0; anatomy.motors()];
    let strengths = vec![Q; anatomy.motors()];
    let loads = vec![0; anatomy.axes.len()];
    for _ in 0..800 {
        let absorbed = body.step(&anatomy, &commands, &strengths, 0, &loads, 6, &[]);
        assert_eq!(absorbed, [0; 3]);
        assert_eq!(body.momentum, [0; 3]);
    }
    assert!(
        body.angles[0] < 500,
        "the ligament should have eased the joint down from 700, still at {}",
        body.angles[0]
    );
    assert!(body.rates.iter().all(|rate| rate.abs() <= 20));
}

#[test]
fn a_traveling_bend_exchanges_momentum_with_the_medium() {
    let anatomy = Anatomy::reference();
    let mut body = Body::new(&anatomy);
    let strengths = vec![Q; anatomy.motors()];
    let loads = vec![0; anatomy.axes.len()];
    let mut environment = [0i64; 3];
    let mut place = [0i64; 3];
    for tick in 0..240 {
        let mut commands = vec![0; anatomy.motors()];
        for joint in 0..anatomy.spine {
            let phase = tick * 3 + joint * 2;
            commands[joint * 2] = ((phase % 64) as i32 - 32) * 24;
        }
        let absorbed = body.step(&anatomy, &commands, &strengths, 32, &loads, 6, &[]);
        for axis in 0..3 {
            environment[axis] += absorbed[axis];
            place[axis] += i64::from(body.velocity[axis]);
        }
        body.validate(&anatomy).unwrap();
    }
    assert_eq!(
        environment,
        body.momentum.map(|momentum| -momentum),
        "shape thrust must stay inside the momentum ledger"
    );
    let traveled = place.iter().map(|axis| axis.abs()).max().unwrap_or(0);
    assert!(
        traveled > 64,
        "a traveling bend should drift, displacement {place:?}"
    );
}

#[test]
fn motor_synapses_follow_reward_and_are_not_inherited() {
    let mut state = NeuralState::new(ControllerGenome::founder(5, 1, Anatomy::reference()), 100);
    state.activations.fill(800);
    state.muscle.fill(400);
    for step in 0..128 {
        state.feedback(100 + step as u64, 1, &NeuralRules::default());
    }
    assert!(state.plastic_output.iter().any(|weight| *weight != 0));
    let child = state.offspring(3, 9, 4, 40, false);
    assert!(child.plastic_output.iter().all(|weight| *weight == 0));
    assert!(child.muscle.iter().all(|drive| *drive == 0));
    let frozen = NeuralRules {
        plasticity: false,
        ..NeuralRules::default()
    };
    let mut off = state.clone();
    let before = off.plastic_output.clone();
    off.feedback(200, 1, &frozen);
    assert_eq!(off.plastic_output, before);
}

#[test]
fn scent_reaches_eight_sites_and_not_beyond() {
    let mut config = config();
    config.width = 24;
    config.height = 24;
    config.depth = 24;
    config.initial_population = 1;
    config.initial_nutrient_per_site = 0;
    config.nutrient_wave_amplitude = 0;
    let world = World::genesis(&config, 1).unwrap();
    let mut snapshot = world.snapshot();
    snapshot.nutrient.fill(0);
    let origin = snapshot.cells[0].site;
    let near = moebius_core::Site {
        x: (origin.x + 4) % config.width,
        y: origin.y,
        z: origin.z,
    };
    let far = moebius_core::Site {
        x: (origin.x + 12) % config.width,
        y: origin.y,
        z: origin.z,
    };
    let dims = world.dims();
    let near_index = (near.x + dims.width * (near.y + dims.height * near.z)) as usize;
    snapshot.reservoir -= 40;
    snapshot.nutrient[near_index] = 40;
    let mut near_world = World::restore(&config, &snapshot).unwrap();
    near_world.tick().unwrap();
    let near_chemical = chemical_of(&near_world);
    snapshot.nutrient.fill(0);
    let far_index = (far.x + dims.width * (far.y + dims.height * far.z)) as usize;
    snapshot.nutrient[far_index] = 40;
    let mut far_world = World::restore(&config, &snapshot).unwrap();
    far_world.tick().unwrap();
    let far_chemical = chemical_of(&far_world);
    assert!(
        near_chemical.iter().any(|channel| *channel > 0),
        "a clot four sites away should scent a facing, got {near_chemical:?}"
    );
    assert!(
        far_chemical.iter().all(|channel| *channel == 0),
        "a clot twelve sites away is outside the scent, got {far_chemical:?}"
    );
}

fn chemical_of(world: &World) -> [i32; 6] {
    let state = world
        .brains()
        .values()
        .next()
        .unwrap()
        .neural
        .as_ref()
        .unwrap();
    let start = state
        .genome
        .anatomy
        .sensor_names()
        .iter()
        .position(|name| name == "chemical_xp")
        .expect("chemical channels");
    let mut chemical = [0; 6];
    chemical.copy_from_slice(&state.sensors[start..start + 6]);
    chemical
}
