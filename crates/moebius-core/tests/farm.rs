use moebius_core::farm::{BrainRig, SECRETE_ACTION};
use moebius_core::{FarmRules, SimConfig, SnapshotFile, World, audit_with_config};

fn farm() -> SimConfig {
    let mut config = SimConfig::m0_64();
    config.width = 8;
    config.height = 8;
    config.depth = 8;
    config.initial_population = 4;
    config.initial_energy = 24;
    config.initial_nutrient_per_site = 12;
    config.nutrient_wave_amplitude = 0;
    config.inject_per_tick = 8;
    config.inflow_per_tick = 8;
    config.maintenance_cost = 1;
    config.farm = Some(FarmRules {
        terrain_law: moebius_core::TerrainLaw::RecyclableV1,
        max_population: 24,
        birth_energy: 16,
        birth_cost: 2,
        maturity_ticks: 8,
        birth_cooldown: 8,
    });
    config
}

#[test]
fn joint_strokes_create_momentum_and_coasting_keeps_it() {
    let config = farm();
    let world = World::genesis(&config, 4).unwrap();
    let cell = world.cells().next().unwrap();
    let mut rig = BrainRig::new(cell, 4);
    let (_, cost) = rig.actuate(1, cell.site, world.dims()).unwrap();
    assert_eq!(cost, 1);
    assert!(rig.velocity[0] > 0);
    assert!(rig.joints.iter().any(|angle| *angle != 0));
    let before = rig.velocity[0];
    let (_, cost) = rig.actuate(0, cell.site, world.dims()).unwrap();
    assert_eq!(cost, 0);
    assert!(rig.velocity[0] > 0 && rig.velocity[0] < before);
    let velocity = rig.velocity[0];
    rig.actuate(3, cell.site, world.dims()).unwrap();
    assert!(
        rig.velocity[0] < velocity,
        "opposite impulse brakes the body"
    );
}

#[test]
fn births_learning_ledgers_and_restore_are_deterministic() {
    let config = farm();
    let mut world = World::genesis(&config, 19).unwrap();
    let initial = world.brains().clone();
    let mut births = 0;
    for _ in 0..40 {
        let receipt = world.tick().unwrap();
        births += receipt.births;
        assert_eq!(world.accounts().unwrap().energy_residual(), 0);
        assert_eq!(world.accounts().unwrap().material_residual(), 0);
        assert!(world.live_count() <= 24);
    }
    assert!(births > 0);
    assert!(
        world
            .brains()
            .values()
            .any(|b| b.parent > 0 && b.generation > 0)
    );
    assert!(
        world
            .brains()
            .iter()
            .any(|(id, b)| initial.get(id).is_some_and(|old| old.weights != b.weights))
    );
    let encoded = SnapshotFile::encode(&config, &world).unwrap();
    let decoded = SnapshotFile::decode(&encoded).unwrap();
    audit_with_config(&config, &decoded).unwrap();
    let mut restored = World::restore(&config, &decoded.snapshot).unwrap();
    for _ in 0..20 {
        assert_eq!(world.tick().unwrap(), restored.tick().unwrap());
        assert_eq!(world.snapshot(), restored.snapshot());
    }
}

#[test]
fn crowded_parents_do_not_create_overlapping_children() {
    let mut config = farm();
    config.width = 2;
    config.height = 2;
    config.depth = 2;
    config.initial_population = 8;
    config.farm.as_mut().unwrap().max_population = 8;
    let mut world = World::genesis(&config, 11).unwrap();
    for _ in 0..16 {
        assert_eq!(world.tick().unwrap().births, 0);
    }
    assert_eq!(world.live_count(), 8);
}

#[test]
fn failed_farm_tick_preserves_brain_rig_and_world() {
    let mut config = farm();
    config.reservoir_cap = config.initial_reservoir;
    let mut world = World::genesis(&config, 5).unwrap();
    let before = world.snapshot();
    assert!(world.tick().is_err());
    assert_eq!(before, world.snapshot());
}

#[test]
fn secretion_is_paid_material_persists_after_death_and_restores() {
    use moebius_core::{Stocks, apply_receipt};
    let mut config = farm();
    config.initial_population = 1;
    config.uptake_per_tick = 0;
    config.maintenance_cost = 0;
    let world = World::genesis(&config, 123).unwrap();
    let mut snapshot = world.snapshot();
    snapshot.brains[0].weights.fill([0, 0, 0, 0, -128]);
    snapshot.brains[0].weights[SECRETE_ACTION] = [0, 0, 0, 0, 128];
    let mut world = World::restore(&config, &snapshot).unwrap();
    for _ in 0..8 {
        let mut stocks = Stocks::from(&world.accounts().unwrap());
        let receipt = world.tick().unwrap();
        apply_receipt(&mut stocks, &receipt).unwrap();
        assert_eq!(stocks, Stocks::from(&world.accounts().unwrap()));
    }
    let flesh = world.flesh().clone();
    assert!(!flesh.is_empty(), "motor policy can choose secretion");
    assert!(world.accounts().unwrap().energy_disposed > 0);
    assert_eq!(world.accounts().unwrap().material_residual(), 0);
    config.maintenance_cost = 1000;
    let mut world = World::restore(&config, &world.snapshot()).unwrap();
    world.tick().unwrap();
    assert_eq!(world.live_count(), 0);
    assert!(
        flesh
            .iter()
            .all(|(site, amount)| world.flesh().get(site).is_some_and(|now| now >= amount))
    );
    let flesh = world.flesh().clone();
    let file = SnapshotFile::decode(&SnapshotFile::encode(&config, &world).unwrap()).unwrap();
    audit_with_config(&config, &file).unwrap();
    assert_eq!(
        World::restore(&config, &file.snapshot).unwrap().flesh(),
        &flesh
    );
}

#[test]
fn mature_flesh_surface_produces_damped_contact() {
    use moebius_core::{Site, farm::FleshDeposit};
    let mut config = farm();
    config.initial_population = 1;
    let world = World::genesis(&config, 33).unwrap();
    let mut snapshot = world.snapshot();
    snapshot.cells[0].site = Site { x: 0, y: 0, z: 0 };
    snapshot.brains[0].offset = [511, 0, 0];
    snapshot.brains[0].velocity = [256, 0, 0];
    snapshot.flesh.push(FleshDeposit {
        site: Site { x: 1, y: 0, z: 0 },
        amount: 4,
    });
    snapshot.nutrient[1] -= 4;
    let mut world = World::restore(&config, &snapshot).unwrap();
    world.tick().unwrap();
    assert_eq!(world.cells().next().unwrap().site.x, 0);
    assert!(world.brains().values().next().unwrap().velocity[0] < 0);
    assert_eq!(world.accounts().unwrap().material_residual(), 0);
}

#[test]
fn front_rear_twist_and_mandibles_have_distinct_control() {
    use moebius_core::farm::{DIG_START, JOINTS, SPINE_JOINTS};
    let world = World::genesis(&farm(), 11).unwrap();
    let cell = world.cells().next().unwrap();
    let mut rig = BrainRig::new(cell, 11);
    assert_eq!(rig.joints.len(), JOINTS);
    rig.actuate(1, cell.site, world.dims()).unwrap();
    assert!(rig.joints[..6].iter().any(|a| *a != 0));
    assert!(rig.twists[..6].iter().any(|a| *a != 0));
    assert_eq!(&rig.joints[6..SPINE_JOINTS], &[0; 6]);
    let front = rig.joints[..6].to_vec();
    rig.actuate(14, cell.site, world.dims()).unwrap();
    assert_eq!(&rig.joints[..6], front.as_slice());
    assert!(rig.joints[6..SPINE_JOINTS].iter().any(|a| *a != 0));
    let spine = rig.joints[..SPINE_JOINTS].to_vec();
    rig.actuate(DIG_START, cell.site, world.dims()).unwrap();
    assert!(rig.joints[SPINE_JOINTS] > 0, "first jaw stroke opens");
    rig.actuate(DIG_START, cell.site, world.dims()).unwrap();
    assert!(rig.joints[SPINE_JOINTS] < 0, "second stroke closes");
    assert_eq!(&rig.joints[..SPINE_JOINTS], spine.as_slice());
}

#[test]
fn face_neighbors_weld_transitively_and_across_periodic_seams() {
    use moebius_core::{Dims, Site, farm::fused_surface};
    use std::collections::BTreeMap;
    let dims = Dims {
        width: 8,
        height: 8,
        depth: 8,
    };
    let chain: BTreeMap<_, _> = (0..4).map(|x| (Site { x, y: 0, z: 0 }, 1)).collect();
    assert_eq!(fused_surface(&chain, dims).unwrap().len(), 4);
    let separated = BTreeMap::from([
        (Site { x: 0, y: 0, z: 0 }, 2),
        (Site { x: 1, y: 1, z: 0 }, 2),
    ]);
    assert!(fused_surface(&separated, dims).unwrap().is_empty());
    let seam = BTreeMap::from([
        (Site { x: 0, y: 0, z: 0 }, 2),
        (Site { x: 7, y: 0, z: 0 }, 2),
    ]);
    assert_eq!(fused_surface(&seam, dims).unwrap().len(), 2);
}

#[test]
fn closing_mandibles_excavate_paid_tissue_and_open_a_passage() {
    use moebius_core::{
        Site, Stocks, apply_receipt,
        farm::{DIG_START, FleshDeposit},
    };
    let mut config = farm();
    config.initial_population = 1;
    config.uptake_per_tick = 0;
    config.maintenance_cost = 0;
    let world = World::genesis(&config, 123).unwrap();
    let mut snapshot = world.snapshot();
    snapshot.cells[0].site = Site { x: 0, y: 0, z: 0 };
    snapshot.flesh.push(FleshDeposit {
        site: Site { x: 1, y: 0, z: 0 },
        amount: 4,
    });
    snapshot.nutrient[1] -= 4;
    snapshot.brains[0].weights.fill([0, 0, 0, 0, -128]);
    snapshot.brains[0].weights[DIG_START] = [0, 0, 0, 0, 128];
    let mut world = World::restore(&config, &snapshot).unwrap();
    let mut removed = 0;
    for _ in 0..8 {
        let mut stocks = Stocks::from(&world.accounts().unwrap());
        let receipt = world.tick().unwrap();
        removed += receipt.excavated;
        apply_receipt(&mut stocks, &receipt).unwrap();
        assert_eq!(stocks, Stocks::from(&world.accounts().unwrap()));
    }
    assert_eq!(removed, 4);
    assert!(!world.flesh().contains_key(&Site { x: 1, y: 0, z: 0 }));
    assert!(world.accounts().unwrap().energy_disposed >= 2);
    let file = SnapshotFile::decode(&SnapshotFile::encode(&config, &world).unwrap()).unwrap();
    audit_with_config(&config, &file).unwrap();
    let mut snapshot = world.snapshot();
    snapshot.cells[0].site = Site { x: 0, y: 0, z: 0 };
    snapshot.brains[0].velocity = [256, 0, 0];
    snapshot.brains[0].offset = [511, 0, 0];
    snapshot.brains[0].weights.fill([0, 0, 0, 0, -128]);
    snapshot.brains[0].weights[0] = [0, 0, 0, 0, 128];
    let mut world = World::restore(&config, &snapshot).unwrap();
    world.tick().unwrap();
    assert_eq!(world.cells().next().unwrap().site.x, 1);
}

#[test]
fn empty_energy_cannot_power_jaws_or_remove_tissue() {
    use moebius_core::{
        Site,
        farm::{DIG_START, FleshDeposit},
    };
    let mut config = farm();
    config.initial_population = 1;
    config.initial_energy = 0;
    config.uptake_per_tick = 0;
    config.maintenance_cost = 0;
    let world = World::genesis(&config, 123).unwrap();
    let mut snapshot = world.snapshot();
    snapshot.cells[0].site = Site { x: 0, y: 0, z: 0 };
    snapshot.flesh.push(FleshDeposit {
        site: Site { x: 1, y: 0, z: 0 },
        amount: 4,
    });
    snapshot.nutrient[1] -= 4;
    snapshot.brains[0].weights.fill([0, 0, 0, 0, -128]);
    snapshot.brains[0].weights[DIG_START] = [0, 0, 0, 0, 128];
    let mut world = World::restore(&config, &snapshot).unwrap();
    for _ in 0..4 {
        assert_eq!(world.tick().unwrap().excavated, 0);
    }
    assert_eq!(world.flesh().values().copied().sum::<u64>(), 4);
    assert!(
        world
            .brains()
            .values()
            .next()
            .unwrap()
            .joints
            .iter()
            .all(|a| *a == 0)
    );
}

#[test]
fn snapshot_identifies_the_new_rig_and_refuses_legacy_reinterpretation() {
    let config = farm();
    let world = World::genesis(&config, 3).unwrap();
    let mut bytes = SnapshotFile::encode(&config, &world).unwrap();
    let offset = bytes.windows(5).position(|part| part == b"FARM2").unwrap();
    bytes[offset..offset + 5].copy_from_slice(b"FARM1");
    let hash = blake3::hash(&bytes[80..]);
    bytes[12..44].copy_from_slice(hash.as_bytes());
    let error = SnapshotFile::decode(&bytes).err().unwrap().to_string();
    assert!(error.contains("legacy FARM1"), "{error}");
}
