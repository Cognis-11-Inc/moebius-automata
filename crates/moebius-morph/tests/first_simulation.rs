use moebius_morph::{
    AnatomySpec, CompiledAnatomy,
    emission::{SCALE, place_point},
    runtime::{BLOOD_SAC, Event, Position, ShipRuntime, WorldSnapshot},
};

fn first() -> ShipRuntime {
    ShipRuntime::first_simulation(AnatomySpec::bare(20261123), 20261123, 600).unwrap()
}

fn four_nozzles() -> AnatomySpec {
    let mut spec = AnatomySpec::bare(77);
    let template = spec.components[0].clone();
    spec.components.clear();
    for k in 0..4 {
        let mut nozzle = template.clone();
        nozzle.id = k + 2;
        nozzle.attachment.azimuth = k as f32 * std::f32::consts::FRAC_PI_2;
        nozzle.orientation = [0.; 2];
        spec.components.push(nozzle);
    }
    spec
}

#[test]
fn four_nozzles_lay_four_separate_paid_streams_and_stop_when_empty() {
    let ship = ShipRuntime::first_simulation(four_nozzles(), 77, 90).unwrap();
    let mut state = remove_food(ship.snapshot());
    state.steering.as_mut().unwrap().seeking_food = false;
    state
        .config
        .construction
        .as_mut()
        .unwrap()
        .foraging
        .as_mut()
        .unwrap()
        .refill_below = 0;
    let mut ship = ShipRuntime::restore(state).unwrap();
    ship.advance(30).unwrap();
    let state = ship.state();
    let book = &state.instances[&1].book;
    let assembly = state.assembly.as_ref().unwrap();
    assert_eq!(
        assembly.filaments.len(),
        4,
        "one continuous pipe per nozzle"
    );
    let nozzles: std::collections::BTreeSet<_> =
        assembly.filaments.values().map(|f| f.nozzle).collect();
    assert_eq!(nozzles, book.registered);
    for f in assembly.filaments.values() {
        assert!(f.nodes.len() >= 4);
        assert!(
            book.bound
                .iter()
                .filter(|b| b.filament == f.id)
                .all(|b| b.parcel.nozzle_id == f.nozzle)
        );
    }
    assert_eq!(book.reservoir, 0);
    assert_eq!(book.external_input, 0);
    assert_eq!(
        book.bound_quantity().unwrap() + book.parcels.iter().map(|p| p.quantity).sum::<u64>(),
        book.initial
    );
    assert_eq!(book.residual().unwrap(), 0);
    let before = assembly.clone();
    ship.advance(30).unwrap();
    assert_eq!(*ship.state().assembly.as_ref().unwrap(), before);
    ship.validate().unwrap();
}

#[test]
fn four_pipes_cost_four_times_the_stock_of_one_per_laying_tick() {
    let mut remaining = Vec::new();
    for spec in [AnatomySpec::bare(77), four_nozzles()] {
        let mut state = remove_food(
            ShipRuntime::first_simulation(spec, 77, 90)
                .unwrap()
                .snapshot(),
        );
        state.steering.as_mut().unwrap().seeking_food = false;
        state
            .config
            .construction
            .as_mut()
            .unwrap()
            .foraging
            .as_mut()
            .unwrap()
            .refill_below = 0;
        let mut ship = ShipRuntime::restore(state).unwrap();
        ship.advance(4).unwrap();
        let book = &ship.state().instances[&1].book;
        remaining.push(book.initial - book.reservoir);
        assert_eq!(book.residual().unwrap(), 0);
    }
    assert_eq!(remaining, vec![4, 16]);
}

#[test]
fn stationary_constructors_do_not_emit_and_each_trail_birth_has_a_moving_step() {
    for spec in [AnatomySpec::bare(77), four_nozzles()] {
        let mut state = remove_food(
            ShipRuntime::first_simulation(spec.clone(), 77, 240)
                .unwrap()
                .snapshot(),
        );
        state.steering.as_mut().unwrap().seeking_food = true;
        let mut stationary = ShipRuntime::restore(state).unwrap();
        let before = stationary.snapshot();
        stationary.advance(30).unwrap();
        assert_eq!(
            stationary.state().instances[&1].position,
            before.instances[&1].position
        );
        assert_eq!(
            stationary.state().instances[&1].book.reservoir,
            before.instances[&1].book.reservoir
        );
        assert_eq!(stationary.state().assembly.as_ref().unwrap().nodes(), 0);
        assert!(
            !stationary
                .state()
                .events
                .iter()
                .any(|e| matches!(e, Event::Emitted { organism: 1, .. }))
        );
        let mut active = ShipRuntime::first_simulation(spec, 77, 240).unwrap();
        let mut births = 0;
        for _ in 0..180 {
            let before = active.snapshot();
            active.tick().unwrap();
            let emitted = active
                .state()
                .events
                .iter()
                .filter(
                    |e| matches!(e, Event::Emitted { at, organism: 1, .. } if *at == before.tick),
                )
                .count();
            if emitted > 0 {
                births += emitted;
                assert_ne!(
                    active.state().instances[&1].position,
                    before.instances[&1].position
                );
            }
        }
        assert!(
            births > 0,
            "motion guard cannot disable all trail production"
        );
    }
}

#[test]
fn four_nozzles_refill_before_laying_and_then_move_and_spin() {
    let mut ship = ShipRuntime::first_simulation(four_nozzles(), 77, 300).unwrap();
    let mut building_ticks = 0;
    let mut seeking_ticks = 0;
    for _ in 0..180 {
        let before = ship.snapshot();
        ship.tick().unwrap();
        let after = ship.state();
        let steering = after.steering.unwrap();
        let emissions = after.events.iter().filter(|event| matches!(event, Event::Emitted { at, organism: 1, .. } if *at == before.tick)).count();
        if steering.seeking_food {
            seeking_ticks += 1;
            assert_eq!(emissions, 0, "refilling must accumulate stock");
        } else {
            building_ticks += 1;
            assert_eq!(emissions, 4);
            assert_ne!(after.instances[&1].position, before.instances[&1].position);
            assert_ne!(
                steering.rigid.axial_roll,
                before.steering.unwrap().rigid.axial_roll
            );
        }
        assert_eq!(after.instances[&1].book.residual().unwrap(), 0);
        assert_eq!(after.condensate_gap(), 0);
    }
    assert!(
        building_ticks >= 20 && seeking_ticks >= 20,
        "refill/build cycle absent"
    );
    assert_eq!(
        ship.state()
            .assembly
            .as_ref()
            .unwrap()
            .filaments
            .values()
            .map(|f| f.nozzle)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        4
    );
}

#[test]
fn legacy_mode_without_parallel_or_spin_fields_keeps_its_single_stream() {
    let mut state = ShipRuntime::first_simulation(four_nozzles(), 77, 90)
        .unwrap()
        .snapshot();
    let rules = state
        .config
        .construction
        .as_mut()
        .unwrap()
        .foraging
        .as_mut()
        .unwrap();
    rules.parallel_nozzles = false;
    rules.axial_spin = 0;
    rules.trail_on_motion = false;
    let json = serde_json::to_string(&state).unwrap();
    assert!(!json.contains("parallel_nozzles"));
    assert!(!json.contains("axial_spin"));
    assert!(!json.contains("trail_on_motion"));
    assert!(!json.contains("axial_roll"));
    let decoded: WorldSnapshot = serde_json::from_str(&json).unwrap();
    assert_eq!(serde_json::to_string(&decoded).unwrap(), json);
    let mut ship = ShipRuntime::restore(decoded).unwrap();
    ship.advance(30).unwrap();
    let primary = ship.state().config.construction.as_ref().unwrap().nozzle;
    assert!(!ship.state().assembly.as_ref().unwrap().filaments.is_empty());
    assert!(
        ship.state()
            .assembly
            .as_ref()
            .unwrap()
            .filaments
            .values()
            .all(|f| f.nozzle == primary)
    );
}

#[test]
fn axial_spin_winds_four_paid_pipes_and_restore_continues_the_same_braid() {
    let ship = ShipRuntime::first_simulation(four_nozzles(), 77, 240).unwrap();
    let mut state = remove_food(ship.snapshot());
    // Diagnostic founding stock, explicitly distinct from production defaults.
    state.instances.get_mut(&1).unwrap().book.initial += 1000;
    state.instances.get_mut(&1).unwrap().book.reservoir += 1000;
    state.condensate.as_mut().unwrap().founders += 1000;
    let mut still = state.clone();
    still
        .config
        .construction
        .as_mut()
        .unwrap()
        .foraging
        .as_mut()
        .unwrap()
        .axial_spin = 0;
    let mut spun = ShipRuntime::restore(state).unwrap();
    let mut still = ShipRuntime::restore(still).unwrap();
    let mut poses = Vec::new();
    for _ in 0..90 {
        poses.push(still.state().body_pose(1).unwrap());
        spun.tick().unwrap();
        still.tick().unwrap();
    }
    let serialized = serde_json::to_vec(&spun.snapshot()).unwrap();
    let mut restored = ShipRuntime::restore(serde_json::from_slice(&serialized).unwrap()).unwrap();
    for _ in 90..180 {
        poses.push(still.state().body_pose(1).unwrap());
        spun.tick().unwrap();
        still.tick().unwrap();
        restored.tick().unwrap();
    }
    assert_eq!(spun.snapshot(), restored.snapshot());
    let a = spun.state();
    let b = still.state();
    assert_eq!(
        a.instances[&1].book.reservoir,
        b.instances[&1].book.reservoir
    );
    assert_eq!(
        a.instances[&1].book.bound_quantity().unwrap(),
        b.instances[&1].book.bound_quantity().unwrap()
    );
    assert_eq!(a.assembly.as_ref().unwrap().filaments.len(), 4);
    assert_eq!(b.assembly.as_ref().unwrap().filaments.len(), 4);
    for nozzle in &a.instances[&1].book.registered {
        let rotated: Vec<_> = a.instances[&1]
            .book
            .bound
            .iter()
            .filter(|p| p.parcel.nozzle_id == *nozzle)
            .collect();
        let fixed: Vec<_> = b.instances[&1]
            .book
            .bound
            .iter()
            .filter(|p| p.parcel.nozzle_id == *nozzle)
            .collect();
        assert!(rotated.len() >= 170);
        assert_eq!(rotated.len(), fixed.len());
        let mut winding = 0.;
        let mut previous: Option<f64> = None;
        for (p, q) in rotated.iter().zip(&fixed) {
            assert_eq!(p.parcel.created_tick, q.parcel.created_tick);
            let pose = poses[p.parcel.created_tick as usize];
            let angle = |position: [i64; 3]| {
                let d: [f64; 3] =
                    std::array::from_fn(|k| (position[k] - pose.translation[k]) as f64);
                let x = (d[0] * pose.yaw_cos as f64 - d[2] * pose.yaw_sin as f64) / SCALE as f64;
                d[1].atan2(x)
            };
            let delta = angle(p.parcel.position) - angle(q.parcel.position);
            if let Some(prev) = previous {
                winding += (delta - prev + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU)
                    - std::f64::consts::PI;
            }
            previous = Some(delta);
        }
        assert!(
            winding > std::f64::consts::TAU,
            "nozzle {nozzle} did not complete a turn: {winding}"
        );
    }
    assert_eq!(a.condensate_gap(), 0);
    spun.validate().unwrap();
    let mut corrupted = a.clone();
    corrupted
        .assembly
        .as_mut()
        .unwrap()
        .filaments
        .values_mut()
        .next()
        .unwrap()
        .nozzle = 999;
    assert!(ShipRuntime::restore(corrupted).is_err());
}

#[test]
fn player_body_with_multiple_nozzles_starts_and_preserves_all_parts() {
    let mut spec = AnatomySpec::bare(77);
    let mut nozzle = spec
        .components
        .iter()
        .find(|c| matches!(c.shape, moebius_morph::Shape::SecretionNozzle { .. }))
        .unwrap()
        .clone();
    nozzle.id = spec.next_id();
    nozzle.attachment.azimuth += 1.;
    spec.components.push(nozzle);
    let mut ship = ShipRuntime::first_simulation(spec.clone(), 77, 120).unwrap();
    assert_eq!(ship.state().config.definitions[&1], spec);
    assert_eq!(ship.state().instances[&1].book.registered.len(), 2);
    ship.advance(90).unwrap();
    assert_eq!(ship.state().instances[&1].book.residual().unwrap(), 0);
    assert_eq!(ship.state().condensate_gap(), 0);
    ship.validate().unwrap();
    let restored = ShipRuntime::restore(ship.snapshot()).unwrap();
    assert_eq!(restored.snapshot(), ship.snapshot());
    // Historical route-only fixtures retain their existing one-nozzle contract.
    assert!(moebius_morph::filament::construction_from(spec, 77, 120, 24, "head-first").is_err());
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
    // Operator changed the contract: a stationary body must retain its stock,
    // rather than converting it into a pile at its resting nozzle.
    assert_eq!(unfed.reservoir, initial);
    assert_eq!(unfed.bound_quantity().unwrap(), 0);
    assert_eq!(unfed.in_world().unwrap(), 0);
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
