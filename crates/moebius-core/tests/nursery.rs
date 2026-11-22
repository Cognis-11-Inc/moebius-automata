use moebius_core::{SimConfig, Site, SnapshotFile, World};

fn config() -> SimConfig {
    let mut c = SimConfig::nursery_farm_256();
    c.width = 16;
    c.height = 16;
    c.depth = 16;
    c.initial_population = 1;
    c.feeding.as_mut().unwrap().spacing = 8;
    c.farm.as_mut().unwrap().max_population = 8;
    c.farm.as_mut().unwrap().birth_energy = c.energy_cap;
    c.initial_reservoir = 0;
    c.inflow_per_tick = 0;
    c.inject_per_tick = 0;
    c
}

#[test]
fn intake_is_at_the_articulated_mouth_and_cannot_reach_distant_food() {
    let c = config();
    for (food, expected) in [
        (Site { x: 4, y: 4, z: 4 }, 1),
        (Site { x: 7, y: 4, z: 4 }, 0),
    ] {
        let mut s = World::genesis(&c, 1).unwrap().snapshot();
        s.cells[0].site = Site { x: 4, y: 3, z: 4 };
        s.nutrient.fill(0);
        let index = food.x as usize + 16 * (food.y as usize + 16 * food.z as usize);
        s.nutrient[index] = 10;
        s.material_initial = 10;
        // Controlled fixture isolates oral contact, not learned feeding.
        let state = s.brains[0].neural.as_mut().unwrap();
        state.genome.output.fill(0);
        state.genome.motor_biases.fill(0);
        let mut w = World::restore(&c, &s).unwrap();
        let receipt = w.tick().unwrap();
        assert_eq!(receipt.nutrient_uptaken, expected);
        assert_eq!(w.nutrient()[index], 10 - expected);
        assert_eq!(
            w.brains()[&1].neural.as_ref().unwrap().last_food_site,
            if expected > 0 { Some(food) } else { None }
        );
        assert_eq!(w.accounts().unwrap().material_residual(), 0);
        assert_eq!(w.accounts().unwrap().energy_residual(), 0);
    }
}

#[test]
fn nursery_refill_is_material_transfer_and_replays_through_reproduction() {
    let mut c = config();
    c.farm.as_mut().unwrap().birth_energy = 256;
    // The 256³ nursery keeps a 128-tick cooldown. This fixture shortens it so
    // one birth can fall inside the replay window.
    c.farm.as_mut().unwrap().birth_cooldown = 32;
    c.inflow_per_tick = 4;
    c.inject_per_tick = 4;
    let mut a = World::genesis(&c, 3).unwrap();
    let mut births = 0;
    for _ in 0..96 {
        births += a.tick().unwrap().births;
    }
    assert!(
        births > 0,
        "the provisioned founding fixture must permit reproduction"
    );
    let file = SnapshotFile::decode(&SnapshotFile::encode(&c, &a).unwrap()).unwrap();
    let mut b = World::restore(&c, &file.snapshot).unwrap();
    for _ in 0..32 {
        a.tick().unwrap();
        b.tick().unwrap();
    }
    assert_eq!(a.snapshot().state_hash(), b.snapshot().state_hash());
    assert_eq!(a.accounts().unwrap().material_residual(), 0);
    assert_eq!(a.accounts().unwrap().energy_residual(), 0);
}

#[test]
fn one_bite_covers_maintenance_and_leaves_a_small_surplus() {
    let rules = SimConfig::nursery_farm_256();
    assert_eq!(rules.uptake_per_tick, 1);
    assert_eq!(rules.energy_per_nutrient, 4);
    assert!(rules.energy_per_nutrient > rules.maintenance_cost);
    assert!(rules.initial_energy < rules.farm.unwrap().birth_energy);
    let mut c = config();
    c.farm.as_mut().unwrap().birth_energy = c.energy_cap;
    let mut snapshot = World::genesis(&c, 1).unwrap().snapshot();
    snapshot.cells[0].site = Site { x: 4, y: 3, z: 4 };
    snapshot.cells[0].energy = 128;
    snapshot.nutrient.fill(0);
    snapshot.reservoir = 0;
    snapshot.material_initial = 10;
    let index = 4 + 16 * (4 + 16 * 4);
    snapshot.nutrient[index] = 10;
    let before = snapshot.cells[0].energy;
    let mut world = World::restore(&c, &snapshot).unwrap();
    let receipt = world.tick().unwrap();
    assert_eq!(receipt.nutrient_uptaken, 1);
    assert_eq!(receipt.energy_credited, 4);
    let after = world.cells().next().unwrap().energy;
    assert!(after > before);
    assert!(
        after < before + 8,
        "a bite should not be a feast, energy {before} -> {after}"
    );
    assert_eq!(world.accounts().unwrap().material_residual(), 0);
    assert_eq!(world.accounts().unwrap().energy_residual(), 0);
}

#[test]
fn a_crossing_leaves_flesh_that_stays() {
    let mut c = config();
    c.farm.as_mut().unwrap().birth_energy = c.energy_cap;
    c.maintenance_cost = 0;
    c.initial_reservoir = 1000;
    let world = World::genesis(&c, 1).unwrap();
    let mut snapshot = world.snapshot();
    snapshot.environment_momentum = [-3072, 0, 0];
    let state = snapshot.brains[0].neural.as_mut().unwrap();
    state.body.momentum = [3072, 0, 0];
    state.body.velocity = [192, 0, 0];
    state.genome.output.fill(0);
    state.genome.motor_biases.fill(0);
    let mut world = World::restore(&c, &snapshot).unwrap();
    let mut laid = 0;
    for _ in 0..24 {
        laid += world.tick().unwrap().trail_laid;
    }
    assert!(laid > 0, "passage should condense a thread");
    let flesh = world.flesh().values().sum::<u64>();
    assert_eq!(flesh, laid);
    for _ in 0..16 {
        world.tick().unwrap();
    }
    assert!(world.flesh().values().sum::<u64>() >= flesh);
    assert_eq!(world.accounts().unwrap().material_residual(), 0);
    assert_eq!(world.accounts().unwrap().energy_residual(), 0);
}
