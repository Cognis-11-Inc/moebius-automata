use moebius_morph::{
    emission::SCALE, filament::construction_fixture, food::FoodStage, runtime::Position,
    runtime::ShipRuntime,
};

#[test]
fn generic_food_is_patchy_across_the_ship_and_regrows() {
    let food = moebius_morph::food::generic_field(20261123).unwrap();
    let sites: Vec<_> = food.sites.values().collect();
    assert!(sites.len() > 40);
    let far = Position::from_absolute([2_000_000 * SCALE, 0, 0]).unwrap();
    let nearest = sites
        .iter()
        .map(|site| {
            let d = far.relative(site.position).unwrap();
            d.iter()
                .map(|v| (v.unsigned_abs() as u128).pow(2))
                .sum::<u128>()
        })
        .min()
        .unwrap();
    assert!(nearest > (100_000 * SCALE as u128).pow(2));
    let axis = sites
        .iter()
        .any(|site| site.position.absolute().unwrap()[0] > 10_000_000 * SCALE);
    assert!(axis);
    let mut field = food;
    let before = field.sites.values().next().unwrap().quantity;
    let id = field.sites.values().next().unwrap().id;
    field.step(1, &[], &[]).unwrap();
    assert_eq!(field.sites[&id].quantity, before);
    field.step(40, &[], &[]).unwrap();
    assert!(field.sites[&id].quantity > before);
    let capacity = {
        let full = field.sites.get_mut(&id).unwrap();
        full.quantity = full.capacity;
        full.capacity
    };
    field.step(80, &[], &[]).unwrap();
    assert_eq!(field.sites[&id].quantity, capacity);
}

#[test]
fn secretion_feeds_growth_and_a_bite_bursts_seeds() {
    let mut field = moebius_morph::food::generic_field(7).unwrap();
    let id = field.sites.values().next().unwrap().id;
    let at = field.sites[&id].position.absolute().unwrap();
    let alone = field.sites[&id].quantity;
    field.step(40, &[], &[]).unwrap();
    let grown = field.sites[&id].quantity;
    field.sites.get_mut(&id).unwrap().quantity = alone;
    field.step(40, &[at], &[]).unwrap();
    assert!(field.sites[&id].quantity > grown);
    let mouth = field.sites[&id].position;
    let before = field.sites.len();
    let bursts = field.step(41, &[], &[(1, mouth)]).unwrap();
    assert_eq!(bursts.len(), 1);
    assert_eq!(bursts[0].seeds, 12);
    assert!(field.sites.len() >= before + 11);
    assert!(
        field
            .sites
            .values()
            .any(|site| site.stage == FoodStage::Seed)
    );
}

#[test]
fn a_dried_snailpipe_node_can_be_chewed_back_into_the_reservoir() {
    let mut ship = construction_fixture(3, 90, 64, "straight").unwrap();
    ship.advance(70).unwrap();
    let rules = ship.state().config.construction.clone().unwrap();
    let mut assembly = ship.state().assembly.clone().unwrap();
    let mut book = ship.state().instances[&1].book.clone();
    let at = assembly.filaments.values().next().unwrap().nodes[0].position;
    let before = book.reservoir;
    let chew = assembly
        .chew_dried(&mut book, &rules, at, 2 * SCALE, 48, 12)
        .unwrap();
    let chew = chew.expect("dried node in reach");
    assert!(book.reservoir > before);
    assert_eq!(book.residual().unwrap(), 0);
    assert!(
        assembly
            .filaments
            .values()
            .all(|filament| filament.nodes.iter().all(|node| node.parcel != chew.parcel))
    );
    assembly.validate(&book, &rules).unwrap();
}

#[test]
fn a_fed_body_can_pay_for_a_child() {
    let mut ship = construction_fixture(11, 40, 64, "straight").unwrap();
    ship.sow_generic_food().unwrap();
    ship.sow_brood(12).unwrap();
    assert_eq!(ship.state().instances.len(), 13);
    ship.advance(100).unwrap();
    let state = ship.snapshot();
    assert!(state.instances.len() > 13);
    assert!(state.instances.values().any(|organism| {
        organism.id != 1 && organism.book.external_input > 0 && organism.book.initial == 16
    }));
    assert!(
        state
            .instances
            .values()
            .all(|organism| organism.book.residual().unwrap() == 0)
    );
    assert!(state.assembly.unwrap().nodes() > 0);
}

#[test]
fn sown_food_survives_the_ship_tick_and_replay() {
    let mut ship = construction_fixture(20261123, 80, 512, "head-first").unwrap();
    ship.sow_generic_food().unwrap();
    let mut replay = ShipRuntime::restore(ship.snapshot()).unwrap();
    ship.advance(45).unwrap();
    replay.advance(45).unwrap();
    assert_eq!(ship.snapshot(), replay.snapshot());
    let food = ship.snapshot().food.unwrap();
    assert!(
        food.sites
            .values()
            .any(|site| site.quantity > site.capacity / 2)
    );
    assert!(
        ship.snapshot()
            .instances
            .values()
            .all(|i| i.book.residual().unwrap() == 0)
    );
}
