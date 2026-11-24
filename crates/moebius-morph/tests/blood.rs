use moebius_morph::{
    AnatomySpec, CompiledAnatomy, Pose, Shape,
    filament::{construction_fixture, construction_from},
    runtime::BLOOD_SAC,
};

#[test]
fn bare_trunk_and_blood_sac_come_from_the_generator() {
    let bare = AnatomySpec::bare(3);
    bare.validate().unwrap();
    assert_eq!(bare.components.len(), 1);
    assert!(matches!(
        bare.components[0].shape,
        Shape::SecretionNozzle { .. }
    ));
    let sac = AnatomySpec::blood_sac(9);
    sac.validate().unwrap();
    assert!(
        sac.components
            .iter()
            .all(|part| !matches!(part.shape, Shape::SecretionNozzle { .. }))
    );
    let posed = CompiledAnatomy::new(sac.clone())
        .unwrap()
        .pose(Pose::rest(&sac.body))
        .unwrap();
    assert!(posed.geometry.surfaces[0].positions.len() > 32);
    let mut ship = construction_from(bare, 3, 20, 64, "head-first").unwrap();
    ship.advance(8).unwrap();
    assert!(ship.snapshot().assembly.unwrap().nodes() > 0);
    assert!(
        ship.snapshot()
            .instances
            .values()
            .all(|organism| organism.book.residual().unwrap() == 0)
    );
}

#[test]
fn blood_sacs_bleed_only_what_the_condensate_and_the_parent_can_pay() {
    let mut ship = construction_fixture(11, 40, 128, "head-first").unwrap();
    ship.sow_brood(12).unwrap();
    ship.sow_blood_garden().unwrap();
    let opened = ship.snapshot();
    let dark = opened
        .instances
        .values()
        .find(|organism| {
            organism.definition == BLOOD_SAC
                && organism.position.absolute().unwrap()[0] > 1_000 * 1_000_000
        })
        .unwrap()
        .id;
    let barren = opened
        .instances
        .values()
        .find(|organism| {
            organism.definition == BLOOD_SAC
                && organism.position.absolute().unwrap()[2] > 30 * 1_000_000
        })
        .unwrap()
        .id;
    let dark_reserve = opened.instances[&dark].book.reservoir;
    let barren_reserve = opened.instances[&barren].book.reservoir;
    let started = opened.instances.len();
    let stock = opened.condensate.as_ref().unwrap().biomass;
    ship.advance(24).unwrap();
    let state = ship.snapshot();
    assert_eq!(state.condensate_gap(), 0);
    assert!(
        state
            .instances
            .values()
            .all(|organism| organism.book.residual().unwrap() == 0)
    );
    assert_eq!(state.instances[&dark].book.reservoir, dark_reserve);
    assert_eq!(state.instances[&dark].book.external_input, 0);
    assert_eq!(state.instances[&barren].book.reservoir, barren_reserve);
    assert_eq!(state.instances[&barren].book.external_input, 0);
    assert!(state.condensate.as_ref().unwrap().biomass < stock);
    assert!(state.instances.len() > started);
    assert!(
        state.instances.values().any(|organism| {
            organism.definition != BLOOD_SAC && organism.book.external_input > 0
        })
    );
    assert!(
        state
            .instances
            .values()
            .any(|organism| { organism.definition == BLOOD_SAC && organism.book.initial == 1 })
    );
}
