use moebius_morph::{
    AnatomySpec, CompiledAnatomy,
    emission::{SCALE, place_point},
    runtime::{BLOOD_SAC, Event, Position, ShipRuntime, WorldSnapshot},
};

fn first() -> ShipRuntime {
    ShipRuntime::first_simulation(AnatomySpec::bare(20261123), 20261123, 600).unwrap()
}

fn dark(mut state: WorldSnapshot) -> WorldSnapshot {
    state.config.ambient_light = 0;
    state.config.ports.clear();
    for body in state.instances.values_mut() {
        body.exposure = 0;
    }
    state
}

fn remove_food(mut state: WorldSnapshot) -> WorldSnapshot {
    let ids: Vec<_> = state
        .instances
        .values()
        .filter(|i| i.definition == BLOOD_SAC)
        .map(|i| i.id)
        .collect();
    for id in ids {
        let food = state.instances.remove(&id).unwrap();
        // This intervention is at tick zero: return its paid founding quantity.
        assert_eq!(food.book.external_input, 0);
        assert_eq!(food.book.sinks, 0);
        state.condensate.as_mut().unwrap().biomass += food.book.initial;
    }
    assert_eq!(state.condensate_gap(), 0);
    state
}

#[test]
fn first_startup_has_one_active_constructor_and_finite_food() {
    let ship = first();
    let state = ship.state();
    assert_eq!(
        state
            .instances
            .values()
            .filter(|i| i.definition != BLOOD_SAC)
            .count(),
        1
    );
    assert_eq!(
        state
            .instances
            .values()
            .filter(|i| i.definition == BLOOD_SAC)
            .count(),
        6
    );
    assert!(state.instances[&1].book.emission_enabled);
    assert!(!state.instances[&1].book.registered.is_empty());
    assert!(
        state.food.is_none(),
        "no independent unlimited regrowth ledger"
    );
    assert_eq!(state.condensate_gap(), 0);
    assert!(state.condensate.as_ref().unwrap().biomass < 4000);
    ship.validate().unwrap();
}

#[test]
fn same_constructor_eats_then_builds_beyond_its_founding_stock() {
    let mut fed = first();
    let mut empty = ShipRuntime::restore(remove_food(fed.snapshot())).unwrap();
    let initial = fed.state().instances[&1].book.initial;
    for _ in 0..300 {
        let previous = fed.state().instances[&1].position;
        fed.tick().unwrap();
        let state = fed.state();
        let delta = state.instances[&1].position.relative(previous).unwrap();
        let displacement = delta
            .iter()
            .map(|v| (*v as f64).powi(2))
            .sum::<f64>()
            .sqrt();
        assert!(displacement <= SCALE as f64 / 40. + 2.);
        assert_eq!(state.condensate_gap(), 0);
        assert!(
            state
                .instances
                .values()
                .all(|i| i.book.residual().unwrap() == 0)
        );
    }
    empty.advance(300).unwrap();
    fed.validate().unwrap();
    empty.validate().unwrap();
    let body = &fed.state().instances[&1].book;
    let unfed = &empty.state().instances[&1].book;
    assert!(body.external_input > 0, "constructor never ate");
    assert!(
        body.bound_quantity().unwrap() > initial,
        "construction never used intake"
    );
    assert_eq!(unfed.external_input, 0);
    assert_eq!(unfed.reservoir, 0);
    assert!(unfed.bound_quantity().unwrap() + unfed.in_world().unwrap() <= initial);
    let events: u64 = fed
        .state()
        .events
        .iter()
        .filter_map(|e| match e {
            Event::Fed {
                organism: 1,
                quantity,
                ..
            } => Some(quantity),
            _ => None,
        })
        .sum();
    assert_eq!(events, body.external_input);
    println!(
        "LOOP tick={} founding={} intake={} reserve={} bound={} free={} empty_intake={} empty_reserve={} empty_bound={} stock={} gap={}",
        fed.state().tick,
        initial,
        body.external_input,
        body.reservoir,
        body.bound_quantity().unwrap(),
        body.in_world().unwrap(),
        unfed.external_input,
        unfed.reservoir,
        unfed.bound_quantity().unwrap(),
        fed.state().condensate.as_ref().unwrap().biomass,
        fed.state().condensate_gap()
    );
    assert!(
        fed.state()
            .instances
            .values()
            .any(|i| i.definition == BLOOD_SAC && i.book.external_input > 0),
        "paid structure/light food fixation did not run"
    );
}

#[test]
fn disabling_secretion_prevents_structure_while_feeding_still_works() {
    let mut state = first().snapshot();
    state.instances.get_mut(&1).unwrap().book.emission_enabled = false;
    let mut ship = ShipRuntime::restore(state).unwrap();
    ship.advance(200).unwrap();
    ship.validate().unwrap();
    assert!(ship.state().instances[&1].book.external_input > 0);
    assert_eq!(ship.state().assembly.as_ref().unwrap().nodes(), 0);
    assert!(
        ship.state()
            .instances
            .values()
            .filter(|i| i.definition == BLOOD_SAC)
            .all(|i| i.book.external_input == 0),
        "food fixation requires actual bound secretion"
    );
    assert_eq!(ship.state().instances[&1].book.in_world().unwrap(), 0);
}

#[test]
fn feeding_uses_the_generated_open_mouth_and_records_actual_contact() {
    let mut state = dark(first().snapshot());
    state
        .config
        .construction
        .as_mut()
        .unwrap()
        .foraging
        .as_mut()
        .unwrap()
        .step = 1;
    // Keep one paid meal, remove other targets without losing their matter.
    let source = *state
        .instances
        .values()
        .find(|i| i.definition == BLOOD_SAC)
        .map(|i| &i.id)
        .unwrap();
    let ids: Vec<_> = state
        .instances
        .keys()
        .copied()
        .filter(|id| *id != 1 && *id != source)
        .collect();
    for id in ids {
        state.condensate.as_mut().unwrap().biomass +=
            state.instances.remove(&id).unwrap().book.initial;
    }
    let mut at_body = state.clone();
    at_body.instances.get_mut(&source).unwrap().position = state.instances[&1].position;
    let mut body_only = ShipRuntime::restore(at_body).unwrap();
    body_only.tick().unwrap();
    assert_eq!(
        body_only.state().instances[&1].book.external_input,
        0,
        "body proximity is not mouth contact"
    );

    let geometry = CompiledAnatomy::new(state.config.definitions[&1].clone())
        .unwrap()
        .pose(state.anatomical_pose(1).unwrap())
        .unwrap()
        .geometry;
    let expected = Position::from_absolute(
        place_point(geometry.mouth_center, state.body_pose(1).unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(state.mouth_position(1).unwrap(), Some(expected));
    state.instances.get_mut(&source).unwrap().position = expected;
    let mut open = ShipRuntime::restore(state.clone()).unwrap();
    open.tick().unwrap();
    assert!(open.state().instances[&1].book.external_input > 0);
    for event in &open.state().events {
        if let Event::Fed { mouth, source, .. } = event {
            let delta = mouth
                .relative(open.state().instances[source].position)
                .unwrap();
            let d2: i128 = delta.iter().map(|v| i128::from(*v).pow(2)).sum();
            assert!(d2 <= i128::from(SCALE / 4).pow(2));
            assert_eq!(Some(*mouth), open.state().mouth_position(1).unwrap());
        }
    }
    state
        .config
        .definitions
        .get_mut(&1)
        .unwrap()
        .body
        .mouth
        .aperture = 0.;
    let mut closed = ShipRuntime::restore(state).unwrap();
    closed.advance(5).unwrap();
    assert_eq!(closed.state().instances[&1].book.external_input, 0);
    assert!(closed.state().mouth_position(1).unwrap().is_none());
}

#[test]
fn food_beyond_sensing_range_does_not_steer_or_feed() {
    let mut state = dark(first().snapshot());
    for food in state
        .instances
        .values_mut()
        .filter(|i| i.definition == BLOOD_SAC)
    {
        food.position = food.position.offset([100 * SCALE, 0, 0]).unwrap();
    }
    let initial = state.instances[&1].position;
    let mut ship = ShipRuntime::restore(state).unwrap();
    ship.advance(10).unwrap();
    assert_eq!(ship.state().instances[&1].position, initial);
    assert_eq!(ship.state().instances[&1].book.external_input, 0);
}

#[test]
fn approach_changes_with_food_location_instead_of_following_the_recorded_route() {
    let original = first().snapshot();
    let source = original
        .instances
        .values()
        .find(|i| i.definition == BLOOD_SAC)
        .unwrap()
        .clone();
    for side in [-1, 1] {
        let mut state = dark(remove_food(original.clone()));
        let mut meal = source.clone();
        meal.position = Position::from_absolute([side * 5 * SCALE, 0, 0]).unwrap();
        meal.exposure = 0;
        state.condensate.as_mut().unwrap().biomass -= meal.book.initial;
        state.instances.insert(meal.id, meal);
        let mut ship = ShipRuntime::restore(state).unwrap();
        ship.advance(180).unwrap();
        let body = &ship.state().instances[&1];
        assert!(body.position.absolute().unwrap()[0] * side > 2 * SCALE);
        assert!(
            body.book.external_input > 0,
            "did not reach meal on side {side}"
        );
        ship.validate().unwrap();
    }
}

#[test]
fn a_stocked_constructor_leaves_the_meal_and_pays_for_a_building_pass() {
    let mut ship = first();
    for _ in 0..80 {
        ship.tick().unwrap();
        if !ship.state().steering.unwrap().seeking_food {
            break;
        }
    }
    assert!(
        !ship.state().steering.unwrap().seeking_food,
        "never filled its stock"
    );
    let before = ship.snapshot();
    ship.advance(10).unwrap();
    let after = ship.state();
    assert!(!after.steering.unwrap().seeking_food);
    let displacement = after.instances[&1]
        .position
        .relative(before.instances[&1].position)
        .unwrap();
    assert!(
        displacement
            .iter()
            .map(|v| (*v as f64).powi(2))
            .sum::<f64>()
            .sqrt()
            > SCALE as f64 / 5.
    );
    let previous = &before.instances[&1].book;
    let current = &after.instances[&1].book;
    assert_eq!(
        current.external_input, previous.external_input,
        "should leave food while stocked"
    );
    assert_eq!(current.reservoir + 10, previous.reservoir);
    assert!(current.bound_quantity().unwrap() > previous.bound_quantity().unwrap());
    let mut returned = false;
    for _ in 0..80 {
        ship.tick().unwrap();
        if ship.state().steering.unwrap().seeking_food {
            returned = true;
            break;
        }
    }
    assert!(returned, "stock depletion must return it to food seeking");
    ship.validate().unwrap();
}

#[test]
fn feeding_constructor_save_restore_continues_exactly_and_detects_corruption() {
    let mut ship = first();
    ship.advance(90).unwrap();
    let state: WorldSnapshot =
        serde_json::from_slice(&serde_json::to_vec(&ship.snapshot()).unwrap()).unwrap();
    let mut resumed = ShipRuntime::restore(state.clone()).unwrap();
    ship.advance(120).unwrap();
    resumed.advance(120).unwrap();
    assert_eq!(ship.snapshot(), resumed.snapshot());
    resumed.validate().unwrap();
    let book = state.instances[&1].book.clone();
    assert_eq!(book.version, moebius_morph::emission::FED_BOOK_VERSION);
    let mut legacy = book.clone();
    legacy.version = 2;
    assert!(
        legacy.to_json().is_err(),
        "legacy zero-input books must reject intake"
    );
    let mut unpaid = book;
    unpaid.reservoir += 1;
    assert!(
        unpaid.to_json().is_err(),
        "food cannot create an unaccounted quantum"
    );
    let mut corrupt = state.clone();
    corrupt.steering.as_mut().unwrap().rigid.translation[0] += SCALE;
    assert!(ShipRuntime::restore(corrupt).is_err());
    let mut corrupt = state.clone();
    corrupt.condensate.as_mut().unwrap().biomass += 1;
    assert!(ShipRuntime::restore(corrupt).is_err());
    let mut corrupt = state;
    let feeding = corrupt
        .events
        .iter()
        .position(|e| matches!(e, Event::Fed { organism: 1, .. }))
        .unwrap();
    corrupt.events.remove(feeding);
    assert!(ShipRuntime::restore(corrupt).is_err());
}
