use moebius_core::{
    Cell, DeathRecord, Dir, GenomeFill, ObservationDeck, SimConfig, Site, Snapshot, SnapshotFile,
    Stocks, World, apply_receipt, audit_with_config, check_horizon, config_hash, fluid_samples,
    load_config_str, movement_dir, observe, parse_hypothesis, run_for, snapshot_schedule,
    split_trail, step_site, validate_run,
};

fn base() -> SimConfig {
    SimConfig {
        width: 6,
        height: 6,
        depth: 6,
        initial_population: 8,
        initial_energy: 20,
        initial_nutrient_per_site: 6,
        nutrient_wave_amplitude: 0,
        initial_reservoir: 100,
        initial_waste: 0,
        inflow_per_tick: 2,
        inject_per_tick: 2,
        uptake_per_tick: 1,
        energy_per_nutrient: 2,
        maintenance_cost: 1,
        export_per_tick: 1,
        site_cap: 40,
        reservoir_cap: 1_000_000,
        energy_cap: 100_000,
        genome_fill: GenomeFill::Derived,
    }
}

fn assert_closed(world: &World) {
    world.audit().unwrap();
    let accounts = world.accounts().unwrap();
    assert_eq!(accounts.material_residual(), 0, "{accounts:?}");
    assert_eq!(accounts.energy_residual(), 0, "{accounts:?}");
    let mut occupied = std::collections::BTreeSet::new();
    for cell in world.cells() {
        assert!(cell.id != 0);
        assert!(occupied.insert(cell.site), "duplicate site {:?}", cell.site);
        assert!(cell.site.x < world.config().width);
        assert!(cell.site.y < world.config().height);
        assert!(cell.site.z < world.config().depth);
    }
    assert_eq!(occupied.len(), world.live_count() as usize);
    for occupant in world.occupancy() {
        if *occupant != 0 {
            assert!(world.cells().any(|cell| cell.id == *occupant));
        }
    }
}

fn manual(config: &SimConfig, seed: u64, cells: Vec<Cell>, nutrient: Vec<u64>) -> World {
    let field: u64 = nutrient.iter().sum();
    let energy_credited: u64 = cells.iter().map(|cell| cell.energy).sum();
    let next_cell_id = cells.iter().map(|cell| cell.id).max().unwrap_or(0) + 1;
    let snapshot = Snapshot {
        tick: 0,
        seed,
        width: config.width,
        height: config.height,
        depth: config.depth,
        reservoir: config.initial_reservoir,
        waste: config.initial_waste,
        external_input: 0,
        external_output: 0,
        energy_credited,
        energy_maintenance: 0,
        energy_disposed: 0,
        material_initial: field + config.initial_reservoir + config.initial_waste,
        next_cell_id,
        inject_cursor: 0,
        cells,
        deaths: Vec::<DeathRecord>::new(),
        nutrient,
    };
    World::restore(config, &snapshot).unwrap()
}

#[test]
fn one_live_id_occupies_one_legal_site() {
    let mut world = World::genesis(&base(), 11).unwrap();
    for _ in 0..40 {
        world.tick().unwrap();
        assert_closed(&world);
    }
}

#[test]
fn material_and_action_energy_ledgers_stay_closed() {
    let mut world = World::genesis(&base(), 19).unwrap();
    let mut stocks = Stocks::from(&world.accounts().unwrap());
    for _ in 0..30 {
        let receipt = world.tick().unwrap();
        assert_eq!(receipt.nutrient_uptaken, receipt.waste_created);
        apply_receipt(&mut stocks, &receipt).unwrap();
    }
    let end = world.accounts().unwrap();
    assert_eq!(stocks.field, end.field);
    assert_eq!(stocks.reservoir, end.reservoir);
    assert_eq!(stocks.waste, end.waste);
    assert_eq!(stocks.external_input, end.external_input);
    assert_eq!(stocks.external_output, end.external_output);
    assert_eq!(stocks.energy_live, end.energy_live);
    assert_eq!(stocks.energy_credited, end.energy_credited);
    assert_eq!(stocks.energy_maintenance, end.energy_maintenance);
    assert_eq!(stocks.energy_disposed, end.energy_disposed);
    assert_eq!(end.material_residual(), 0);
    assert_eq!(end.energy_residual(), 0);
}

#[test]
fn contested_move_gives_the_site_to_the_lower_id_once() {
    let dims = moebius_core::Dims {
        width: 5,
        height: 5,
        depth: 5,
    };
    let mut found = None;
    for seed in 1..400 {
        let first = movement_dir(seed, 0, 1).unwrap();
        let second = movement_dir(seed, 0, 2).unwrap();
        if first == Dir::Stay || second == Dir::Stay {
            continue;
        }
        let origin1 = Site { x: 2, y: 2, z: 2 };
        let target = step_site(origin1, first, dims).unwrap();
        let origin2 = step_site(target, second.opposite(), dims).unwrap();
        if origin2 == origin1 || origin2 == target {
            continue;
        }
        found = Some((seed, origin1, origin2, target));
        break;
    }
    let (seed, origin1, origin2, target) = found.expect("a contested proposal exists");
    let mut config = base();
    config.width = 5;
    config.height = 5;
    config.depth = 5;
    config.initial_population = 2;
    config.initial_energy = 10;
    config.initial_nutrient_per_site = 9;
    config.initial_reservoir = 0;
    config.inflow_per_tick = 0;
    config.inject_per_tick = 0;
    config.uptake_per_tick = 4;
    config.energy_per_nutrient = 1;
    config.maintenance_cost = 0;
    config.export_per_tick = 0;
    config.site_cap = 20;
    config.reservoir_cap = 100;
    config.energy_cap = 100;
    config.genome_fill = GenomeFill::Zero;
    let nutrient = vec![9u64; 125];
    let cells = vec![
        Cell {
            id: 1,
            site: origin1,
            energy: 10,
            age: 0,
            genome: [0; 8],
        },
        Cell {
            id: 2,
            site: origin2,
            energy: 10,
            age: 0,
            genome: [0; 8],
        },
    ];
    let before = nutrient.clone();
    let mut world = manual(&config, seed, cells, nutrient);
    let receipt = world.tick().unwrap();
    assert_closed(&world);
    let cell1 = world.cells().find(|cell| cell.id == 1).unwrap();
    let cell2 = world.cells().find(|cell| cell.id == 2).unwrap();
    assert_eq!(cell1.site, target);
    assert_eq!(cell2.site, origin2);
    assert_ne!(cell1.site, cell2.site);
    assert_eq!(receipt.moves_applied, 1);
    assert_eq!(receipt.moves_blocked, 1);
    assert_eq!(receipt.nutrient_uptaken, 8);
    let index = |site: Site| (site.x + config.width * (site.y + config.height * site.z)) as usize;
    assert_eq!(world.nutrient()[index(target)], before[index(target)] - 4);
    assert_eq!(world.nutrient()[index(origin2)], before[index(origin2)] - 4);
    assert_eq!(world.nutrient()[index(origin1)], before[index(origin1)]);
    assert_eq!(world.accounts().unwrap().waste, 8);
    assert_eq!(cell1.energy, 14);
    assert_eq!(cell2.energy, 14);
}

#[test]
fn same_seed_and_config_reproduce_hashes() {
    let config = base();
    for ticks in [100u64, 1_000, 10_000] {
        let mut left = World::genesis(&config, 42).unwrap();
        let mut right = World::genesis(&config, 42).unwrap();
        let left_run = run_for(&mut left, ticks).unwrap();
        let right_run = run_for(&mut right, ticks).unwrap();
        assert_eq!(left_run.final_hash, right_run.final_hash, "ticks {ticks}");
        assert_eq!(left_run.metrics, right_run.metrics, "ticks {ticks}");
        assert_eq!(left_run.extinct, left_run.live_count == 0);
    }
    let mut other = World::genesis(&config, 43).unwrap();
    let same = World::genesis(&config, 42).unwrap();
    assert_ne!(other.snapshot().state_hash(), same.snapshot().state_hash());
    let _ = run_for(&mut other, 1).unwrap();
}

#[test]
fn smoke_one_thousand_ticks_records_extinction_explicitly() {
    let mut world = World::genesis(&base(), 7).unwrap();
    let outcome = run_for(&mut world, 1_000).unwrap();
    assert_eq!(outcome.metrics.len(), 1_001);
    assert_eq!(world.tick_index(), 1_000);
    assert_eq!(outcome.extinct, outcome.live_count == 0);
    assert_eq!(outcome.final_hash, world.snapshot().state_hash());
    for row in &outcome.metrics {
        assert_eq!(row.material_residual, 0);
        assert_eq!(row.energy_residual, 0);
    }
    assert_closed(&world);
}

#[test]
fn invalid_configuration_is_rejected_without_clamping() {
    let mut zero = base();
    zero.width = 0;
    assert!(zero.validate().unwrap_err().to_string().contains("grid"));
    let mut wide = base();
    wide.width = 65;
    assert!(wide.validate().unwrap_err().to_string().contains("65"));
    let mut crowded = base();
    crowded.width = 2;
    crowded.height = 2;
    crowded.depth = 2;
    crowded.initial_population = 9;
    crowded.inject_per_tick = 0;
    assert!(
        crowded
            .validate()
            .unwrap_err()
            .to_string()
            .contains("initial_population")
    );
    let mut value = serde_json::to_value(base()).unwrap();
    value["initial_energy"] = serde_json::json!(-1);
    let err = load_config_str(&value.to_string()).unwrap_err();
    assert!(err.to_string().contains("initial_energy"), "{err}");
    value = serde_json::to_value(base()).unwrap();
    value["organs"] = serde_json::json!(true);
    let err = load_config_str(&value.to_string()).unwrap_err();
    assert!(err.to_string().contains("organs"), "{err}");
    let err = validate_run(&base(), 1_000, 1).unwrap_err();
    assert!(err.to_string().contains("snapshot count"), "{err}");
    let mut capped = base();
    capped.initial_reservoir = 5;
    capped.reservoir_cap = 5;
    capped.inflow_per_tick = 1;
    capped.inject_per_tick = 0;
    let err = check_horizon(&capped, 1).unwrap_err();
    assert!(err.to_string().contains("reservoir_cap"), "{err}");
    let mut world = World::genesis(&capped, 3).unwrap();
    let hash = world.snapshot().state_hash();
    let err = world.tick().unwrap_err();
    assert!(err.to_string().contains("reservoir_cap"), "{err}");
    assert_eq!(world.snapshot().state_hash(), hash);
    assert_eq!(world.tick_index(), 0);
}

#[test]
fn zero_resource_starvation_full_grid_and_wrapping_are_explicit() {
    let mut empty = base();
    empty.width = 2;
    empty.height = 2;
    empty.depth = 2;
    empty.initial_population = 0;
    empty.initial_energy = 0;
    empty.initial_nutrient_per_site = 0;
    empty.initial_reservoir = 0;
    empty.inflow_per_tick = 0;
    empty.inject_per_tick = 0;
    empty.uptake_per_tick = 0;
    empty.energy_per_nutrient = 0;
    empty.maintenance_cost = 0;
    empty.export_per_tick = 0;
    empty.site_cap = 0;
    empty.reservoir_cap = 0;
    empty.energy_cap = 0;
    let mut world = World::genesis(&empty, 1).unwrap();
    let outcome = run_for(&mut world, 10).unwrap();
    assert!(outcome.extinct);
    assert_eq!(outcome.live_count, 0);
    assert_eq!(world.accounts().unwrap().field, 0);
    assert_closed(&world);

    let mut starve = base();
    starve.initial_population = 3;
    starve.initial_energy = 1;
    starve.initial_nutrient_per_site = 0;
    starve.initial_reservoir = 0;
    starve.inflow_per_tick = 0;
    starve.inject_per_tick = 0;
    starve.uptake_per_tick = 1;
    starve.maintenance_cost = 5;
    starve.export_per_tick = 0;
    starve.site_cap = 5;
    let mut world = World::genesis(&starve, 2).unwrap();
    world.tick().unwrap();
    assert_eq!(world.live_count(), 0);
    assert_eq!(world.dead_count(), 3);
    assert!(world.occupancy().iter().all(|id| *id == 0));
    let disposed = world.accounts().unwrap().energy_disposed;
    assert_eq!(disposed, 3);
    for _ in 0..5 {
        world.tick().unwrap();
    }
    assert_eq!(world.live_count(), 0);
    assert_eq!(world.dead_count(), 3);
    assert_closed(&world);

    let mut exact = base();
    exact.initial_population = 2;
    exact.initial_energy = 3;
    exact.initial_nutrient_per_site = 0;
    exact.initial_reservoir = 0;
    exact.inflow_per_tick = 0;
    exact.inject_per_tick = 0;
    exact.maintenance_cost = 3;
    exact.export_per_tick = 0;
    exact.uptake_per_tick = 0;
    exact.site_cap = 5;
    let mut world = World::genesis(&exact, 4).unwrap();
    world.tick().unwrap();
    assert_eq!(world.live_count(), 2);
    assert!(world.cells().all(|cell| cell.energy == 0));
    world.tick().unwrap();
    assert_eq!(world.live_count(), 0);
    let accounts = world.accounts().unwrap();
    assert_eq!(accounts.energy_disposed, 0);
    assert_eq!(accounts.energy_maintenance, 6);
    assert_eq!(accounts.energy_credited, 6);
    assert_closed(&world);

    let mut full = base();
    full.width = 2;
    full.height = 2;
    full.depth = 2;
    full.initial_population = 8;
    full.maintenance_cost = 0;
    full.inflow_per_tick = 0;
    full.inject_per_tick = 0;
    full.initial_reservoir = 0;
    full.reservoir_cap = 10;
    full.uptake_per_tick = 0;
    full.export_per_tick = 0;
    let mut world = World::genesis(&full, 5).unwrap();
    let start: Vec<_> = world.cells().map(|cell| (cell.id, cell.site)).collect();
    for _ in 0..15 {
        let receipt = world.tick().unwrap();
        assert_eq!(receipt.moves_applied, 0);
    }
    let end: Vec<_> = world.cells().map(|cell| (cell.id, cell.site)).collect();
    assert_eq!(start, end);
    assert_eq!(world.live_count(), 8);
    assert_closed(&world);

    let mut wrapped = base();
    wrapped.width = 4;
    wrapped.height = 4;
    wrapped.depth = 4;
    wrapped.initial_population = 1;
    wrapped.initial_energy = 5;
    wrapped.initial_nutrient_per_site = 0;
    wrapped.initial_reservoir = 0;
    wrapped.inflow_per_tick = 0;
    wrapped.inject_per_tick = 0;
    wrapped.uptake_per_tick = 0;
    wrapped.maintenance_cost = 0;
    wrapped.export_per_tick = 0;
    wrapped.site_cap = 1;
    wrapped.reservoir_cap = 1;
    wrapped.energy_cap = 20;
    let dims = moebius_core::Dims {
        width: 4,
        height: 4,
        depth: 4,
    };
    for wanted in [Dir::Xm, Dir::Ym, Dir::Zm] {
        let seed = (1..300)
            .find(|seed| movement_dir(*seed, 0, 1).unwrap() == wanted)
            .expect("direction appears");
        let world = manual(
            &wrapped,
            seed,
            vec![Cell {
                id: 1,
                site: Site { x: 0, y: 0, z: 0 },
                energy: 5,
                age: 0,
                genome: [0; 8],
            }],
            vec![0; 64],
        );
        let mut world = world;
        world.tick().unwrap();
        let site = world.cells().next().unwrap().site;
        let expected = step_site(Site { x: 0, y: 0, z: 0 }, wanted, dims).unwrap();
        assert_eq!(site, expected, "{wanted:?}");
        assert_closed(&world);
    }
}

#[test]
fn snapshot_roundtrip_preserves_the_next_tick() {
    let config = base();
    let world = World::genesis(&config, 99).unwrap();
    let bytes = SnapshotFile::encode(&config, &world).unwrap();
    let file = SnapshotFile::decode(&bytes).unwrap();
    assert_eq!(file.snapshot.state_hash(), world.snapshot().state_hash());
    audit_with_config(&config, &file).unwrap();
    let mut restored = World::restore(&config, &file.snapshot).unwrap();
    let mut original = world.clone();
    let left = observe(&original).unwrap();
    let _ = fluid_samples(original.nutrient(), original.dims(), 2).unwrap();
    let _ = original.trail_points();
    let mut deck = ObservationDeck::default();
    deck.insert_scaffold("organism_candidate").unwrap();
    assert!(deck.get("organism_candidate").unwrap().marks.is_empty());
    assert!(deck.get("organism_candidate").unwrap().note.contains("M0"));
    let _ =
        parse_hypothesis(r#"{"name":"manual_mark","note":"annotation only","sites":[]}"#).unwrap();
    assert_eq!(observe(&original).unwrap().state_hash, left.state_hash);
    original.tick().unwrap();
    restored.tick().unwrap();
    assert_eq!(
        original.snapshot().state_hash(),
        restored.snapshot().state_hash()
    );
}

#[test]
fn inert_genome_does_not_change_dynamics() {
    let mut derived = base();
    derived.genome_fill = GenomeFill::Derived;
    let mut zeros = base();
    zeros.genome_fill = GenomeFill::Zero;
    let mut left = World::genesis(&derived, 5).unwrap();
    let mut right = World::genesis(&zeros, 5).unwrap();
    assert_ne!(left.snapshot().state_hash(), right.snapshot().state_hash());
    for _ in 0..20 {
        left.tick().unwrap();
        right.tick().unwrap();
    }
    let left_body: Vec<_> = left
        .cells()
        .map(|cell| (cell.id, cell.site, cell.energy, cell.age))
        .collect();
    let right_body: Vec<_> = right
        .cells()
        .map(|cell| (cell.id, cell.site, cell.energy, cell.age))
        .collect();
    assert_eq!(left_body, right_body);
    assert_eq!(left.nutrient(), right.nutrient());
    assert_eq!(left.accounts().unwrap(), right.accounts().unwrap());
}

#[test]
fn default_experiment_file_matches_the_constructor() {
    let parsed = load_config_str(include_str!("../../../experiments/m0_64.json")).unwrap();
    let built = SimConfig::m0_64();
    assert_eq!(parsed, built);
    assert_eq!(config_hash(&parsed).unwrap(), config_hash(&built).unwrap());
    let schedule = validate_run(&built, 1_000, 50).unwrap();
    assert_eq!(schedule.first().copied(), Some(0));
    assert_eq!(schedule.last().copied(), Some(1_000));
    assert_eq!(schedule.len(), 21);
    assert_eq!(snapshot_schedule(5, 2), vec![0, 2, 4, 5]);
    let mut left = World::genesis(&built, 20261026).unwrap();
    let mut right = World::genesis(&built, 20261026).unwrap();
    assert_eq!(left.live_count(), 128);
    left.tick().unwrap();
    right.tick().unwrap();
    left.tick().unwrap();
    right.tick().unwrap();
    assert_eq!(left.snapshot().state_hash(), right.snapshot().state_hash());
    assert_eq!(left.accounts().unwrap().material_residual(), 0);
    assert_eq!(left.accounts().unwrap().energy_residual(), 0);
}

#[test]
fn layers_are_pure_projections() {
    let world = World::genesis(&base(), 8).unwrap();
    let hash = world.snapshot().state_hash();
    let samples = fluid_samples(world.nutrient(), world.dims(), 2).unwrap();
    let summed: u64 = samples.iter().map(|sample| sample.amount).sum();
    let field: u64 = world.nutrient().iter().copied().sum();
    assert_eq!(summed, field);
    assert!(fluid_samples(world.nutrient(), world.dims(), 0).is_err());
    let split = split_trail(&[
        Site { x: 0, y: 1, z: 2 },
        Site { x: 1, y: 1, z: 2 },
        Site { x: 7, y: 1, z: 2 },
    ]);
    assert_eq!(split.len(), 2);
    assert_eq!(split[0].len(), 2);
    let err = parse_hypothesis(r#"{"name":"Organism","note":"x","sites":[]}"#).unwrap_err();
    assert!(err.to_string().contains("layer name"), "{err}");
    let err = parse_hypothesis(r#"{"name":"mark","note":"x","sites":[[1,1,1,2],[1,1,1,3]]}"#)
        .unwrap_err();
    assert!(err.to_string().contains("duplicate"), "{err}");
    assert_eq!(world.snapshot().state_hash(), hash);
}

#[test]
fn illegal_coordinate_cannot_be_restored() {
    let mut config = base();
    config.width = 4;
    config.height = 4;
    config.depth = 4;
    config.initial_population = 1;
    config.initial_nutrient_per_site = 0;
    config.initial_reservoir = 0;
    config.inflow_per_tick = 0;
    config.inject_per_tick = 0;
    config.site_cap = 1;
    config.reservoir_cap = 1;
    let err = World::restore(
        &config,
        &Snapshot {
            tick: 0,
            seed: 1,
            width: 4,
            height: 4,
            depth: 4,
            reservoir: 0,
            waste: 0,
            external_input: 0,
            external_output: 0,
            energy_credited: 5,
            energy_maintenance: 0,
            energy_disposed: 0,
            material_initial: 0,
            next_cell_id: 2,
            inject_cursor: 0,
            cells: vec![Cell {
                id: 1,
                site: Site { x: 4, y: 0, z: 0 },
                energy: 5,
                age: 0,
                genome: [0; 8],
            }],
            deaths: Vec::new(),
            nutrient: vec![0; 64],
        },
    )
    .unwrap_err();
    assert!(
        err.to_string().contains("outside") || err.to_string().contains("site"),
        "{err}"
    );
}
