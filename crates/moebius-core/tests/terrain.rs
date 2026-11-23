use moebius_core::{
    SimConfig, Site, SnapshotFile, TerrainLaw, World,
    farm::{DIG_START, FleshDeposit},
};
fn fixture(law: TerrainLaw) -> SimConfig {
    let mut c = SimConfig::neural_farm_256();
    c.neural.as_mut().unwrap().version = 2;
    c.neural.as_mut().unwrap().spinal_wave = false;
    c.width = 16;
    c.height = 16;
    c.depth = 16;
    c.initial_population = 1;
    c.initial_energy = 2048;
    c.initial_nutrient_per_site = 0;
    c.nutrient_wave_amplitude = 0;
    c.initial_reservoir = 0;
    c.inflow_per_tick = 0;
    c.inject_per_tick = 0;
    c.maintenance_cost = 0;
    c.uptake_per_tick = 0;
    c.farm.as_mut().unwrap().max_population = 1;
    c.farm.as_mut().unwrap().terrain_law = law;
    c
}
#[test]
fn omitted_terrain_rule_preserves_historical_encoding() {
    let c = fixture(TerrainLaw::RecyclableV1);
    let value = serde_json::to_value(&c).unwrap();
    assert!(value["farm"].get("terrain_law").is_none());
    let decoded: SimConfig = serde_json::from_value(value).unwrap();
    assert_eq!(decoded, c);
    let new = fixture(TerrainLaw::PermanentV1);
    assert_ne!(
        moebius_core::config_hash(&c).unwrap(),
        moebius_core::config_hash(&new).unwrap()
    );
}
#[test]
fn permanent_terrain_does_not_compost_and_replays_after_long_aging() {
    for law in [TerrainLaw::RecyclableV1, TerrainLaw::PermanentV1] {
        let mut c = fixture(law);
        c.feeding = SimConfig::nursery_farm_256().feeding;
        c.feeding.as_mut().unwrap().spacing = 8;
        c.feeding.as_mut().unwrap().initial_stock = 0;
        let mut s = World::genesis(&c, 1).unwrap().snapshot();
        s.cells[0].site = Site { x: 0, y: 1, z: 0 };
        s.flesh = vec![FleshDeposit {
            site: Site { x: 5, y: 4, z: 4 },
            amount: 3,
        }];
        s.material_initial = 3;
        let mut w = World::restore(&c, &s).unwrap();
        w.tick().unwrap();
        if law == TerrainLaw::RecyclableV1 {
            assert_eq!(w.accounts().unwrap().flesh, 2);
            continue;
        }
        let initial = w.flesh().clone();
        for _ in 0..512 {
            w.tick().unwrap();
            assert_eq!(*w.flesh(), initial);
            assert_eq!(w.accounts().unwrap().material_residual(), 0);
        }
        assert!(w.nutrient().iter().all(|n| *n == 0));
        let file = SnapshotFile::decode(&SnapshotFile::encode(&c, &w).unwrap()).unwrap();
        let mut restored = World::restore(&c, &file.snapshot).unwrap();
        for _ in 0..32 {
            w.tick().unwrap();
            restored.tick().unwrap();
        }
        assert_eq!(w.snapshot().state_hash(), restored.snapshot().state_hash());
        assert_eq!(*w.flesh(), initial);
    }
}
#[test]
fn permanent_terrain_refuses_legacy_destructive_excavation() {
    for law in [TerrainLaw::RecyclableV1, TerrainLaw::PermanentV1] {
        let mut c = fixture(law);
        c.neural = None;
        let mut s = World::genesis(&c, 123).unwrap().snapshot();
        s.cells[0].site = Site { x: 0, y: 0, z: 0 };
        s.flesh = vec![FleshDeposit {
            site: Site { x: 1, y: 0, z: 0 },
            amount: 4,
        }];
        s.material_initial = 4;
        s.brains[0].weights.fill([0, 0, 0, 0, -128]);
        s.brains[0].weights[DIG_START] = [0, 0, 0, 0, 128];
        let mut w = World::restore(&c, &s).unwrap();
        let mut removed = 0;
        for _ in 0..8 {
            removed += w.tick().unwrap().excavated;
            assert_eq!(w.accounts().unwrap().material_residual(), 0);
        }
        assert_eq!(removed, if law == TerrainLaw::PermanentV1 { 0 } else { 4 });
        if law == TerrainLaw::PermanentV1 {
            assert_eq!(w.accounts().unwrap().flesh, 4);
            assert_eq!(w.accounts().unwrap().waste, 0);
        }
    }
}

#[test]
fn carving_relocates_material_without_waste_or_free_energy() {
    use moebius_core::{Stocks, apply_receipt};
    let mut c = fixture(TerrainLaw::ConservedSculptV1);
    c.neural = None;
    let mut s = World::genesis(&c, 123).unwrap().snapshot();
    let origin = Site { x: 0, y: 0, z: 0 };
    let target = Site { x: 1, y: 0, z: 0 };
    s.cells[0].site = origin;
    s.flesh = vec![FleshDeposit {
        site: target,
        amount: 4,
    }];
    s.material_initial = 4;
    s.brains[0].weights.fill([0, 0, 0, 0, -128]);
    s.brains[0].weights[DIG_START] = [0, 0, 0, 0, 128];
    let mut w = World::restore(&c, &s).unwrap();
    let mut moved = 0;
    let initial_energy = w.accounts().unwrap().energy_live;
    for _ in 0..8 {
        let mut stocks = Stocks::from(&w.accounts().unwrap());
        let r = w.tick().unwrap();
        moved += r.sculpted;
        assert_eq!(r.excavated, 0);
        assert_eq!(r.waste_created, 0);
        apply_receipt(&mut stocks, &r).unwrap();
        assert_eq!(stocks, Stocks::from(&w.accounts().unwrap()));
        assert_eq!(w.accounts().unwrap().flesh, 4);
        assert_eq!(w.accounts().unwrap().material_residual(), 0);
        assert_eq!(w.accounts().unwrap().energy_residual(), 0);
    }
    assert_eq!(moved, 4);
    assert_eq!(w.flesh().get(&origin), Some(&4));
    assert!(!w.flesh().contains_key(&target));
    assert!(w.accounts().unwrap().energy_live < initial_energy);
}

#[test]
fn neural_sculpting_requires_paid_active_jaw_closure_not_passive_relaxation() {
    use std::collections::BTreeMap;
    for active in [false, true] {
        let c = fixture(TerrainLaw::ConservedSculptV1);
        let mut s = World::genesis(&c, 321).unwrap().snapshot();
        let origin = Site { x: 8, y: 8, z: 8 };
        s.cells[0].site = origin;
        let brain = &mut s.brains[0];
        let n = brain.neural.as_mut().unwrap();
        for (i, b) in n.genome.anatomy.axes.iter().enumerate() {
            if b.axis == moebius_core::neural::schema::Axis::Jaw {
                n.body.angles[i] = b.max;
            }
        }
        n.body.centroid = n.body.local_geometry(&n.genome.anatomy).1;
        let target = n.body.head_site(
            &n.genome.anatomy,
            origin,
            [0; 3],
            moebius_core::Dims {
                width: 16,
                height: 16,
                depth: 16,
            },
        );
        assert_ne!(target, origin);
        brain.sync_neural_pose();
        s.flesh = vec![FleshDeposit {
            site: target,
            amount: 4,
        }];
        s.material_initial = 4;
        let mut w = World::restore(&c, &s).unwrap();
        let mut moved = 0;
        let mut commands = vec![0; 30];
        if active {
            commands[24] = -1024;
            commands[25] = -1024;
        }
        for _ in 0..8 {
            let (receipt, _) = w
                .tick_controlled(&BTreeMap::from([(1, commands.clone())]))
                .unwrap();
            moved += receipt.sculpted;
            assert_eq!(receipt.excavated, 0);
            assert_eq!(w.accounts().unwrap().flesh, 4);
            assert_eq!(w.accounts().unwrap().material_residual(), 0);
            assert_eq!(w.accounts().unwrap().energy_residual(), 0);
        }
        if active {
            assert!(moved > 0);
            assert!(w.flesh().contains_key(&origin));
        } else {
            assert_eq!(moved, 0);
            assert_eq!(w.flesh().get(&target), Some(&4));
        }
    }
}

#[test]
fn sculpting_respects_receiving_capacity_and_available_energy() {
    for (energy, receiving, expected) in [(2048, 64, 0), (2048, 63, 1), (0, 0, 0)] {
        let mut c = fixture(TerrainLaw::ConservedSculptV1);
        c.neural = None;
        let mut s = World::genesis(&c, 123).unwrap().snapshot();
        let origin = Site { x: 0, y: 0, z: 0 };
        let target = Site { x: 1, y: 0, z: 0 };
        s.cells[0].site = origin;
        s.cells[0].energy = energy;
        s.energy_credited = energy;
        s.flesh = vec![FleshDeposit {
            site: target,
            amount: 4,
        }];
        if receiving > 0 {
            s.flesh.push(FleshDeposit {
                site: origin,
                amount: receiving,
            });
        }
        s.material_initial = 4 + receiving;
        s.brains[0].weights.fill([0, 0, 0, 0, -128]);
        s.brains[0].weights[DIG_START] = [0, 0, 0, 0, 128];
        let mut w = World::restore(&c, &s).unwrap();
        let mut moved = 0;
        for _ in 0..8 {
            moved += w.tick().unwrap().sculpted;
            assert_eq!(w.accounts().unwrap().flesh, 4 + receiving);
            assert_eq!(w.accounts().unwrap().material_residual(), 0);
            assert_eq!(w.accounts().unwrap().energy_residual(), 0);
        }
        assert_eq!(moved, expected);
    }
}
