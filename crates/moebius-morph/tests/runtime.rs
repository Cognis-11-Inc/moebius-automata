use moebius_morph::{
    emission::{Environment, SCALE},
    runtime::*,
    witness::*,
};
use std::{collections::BTreeSet, time::Duration};
fn sphere(tick: u64) -> WitnessSphere {
    WitnessSphere {
        center: Position::from_absolute([
            if tick.is_multiple_of(2) {
                0
            } else {
                1000 * SCALE
            },
            0,
            0,
        ])
        .unwrap(),
        radius: if tick.is_multiple_of(3) {
            20 * SCALE
        } else {
            2 * SCALE
        },
    }
}
#[test]
fn observation_modes_are_exactly_neutral_and_boundary_caches_do_not_reset_identity() {
    let mut a = fixture(8, 7).unwrap();
    let mut b = ShipRuntime::restore(a.snapshot()).unwrap();
    let mut c = ShipRuntime::restore(a.snapshot()).unwrap();
    let mut cache = PresentationCache::default();
    let mut entries = 0;
    let mut exits = 0;
    let stationary = sphere(0);
    let mut material = 0;
    for tick in 0..40 {
        a.tick().unwrap();
        b.tick().unwrap();
        stationary.inspect(b.state()).unwrap();
        c.tick().unwrap();
        let before = c.digest().unwrap();
        let view = sphere(tick).inspect(c.state()).unwrap();
        let changes = cache.update(&view);
        entries += changes.entered.len();
        exits += changes.exited.len();
        material += changes.material_entered + changes.material_exited;
        assert_eq!(before, c.digest().unwrap());
        assert_eq!(a.snapshot(), b.snapshot());
        assert_eq!(a.snapshot(), c.snapshot());
    }
    assert!(entries > 8 && exits > 8 && material > 0);
    assert_eq!(
        c.state().instances.keys().copied().collect::<BTreeSet<_>>(),
        (1..=8).collect()
    );
    assert!(c.state().instances.values().all(|i| i.book.tick == 40
        && i.book.residual().unwrap() == 0
        && i.book.parcels.len() == 40));
}
#[test]
fn authority_worker_keeps_every_tick_with_no_listener_or_a_stalled_observer() {
    let initial = fixture(4, 8).unwrap().snapshot();
    let mut reference = ShipRuntime::restore(initial.clone()).unwrap();
    reference.advance(60).unwrap();
    let worker = Worker::spawn(
        ShipRuntime::restore(initial.clone()).unwrap(),
        60,
        1,
        Duration::ZERO,
    )
    .unwrap();
    std::thread::sleep(Duration::from_millis(20));
    assert_eq!(reference.snapshot(), worker.finish().unwrap());
    let worker = Worker::spawn(
        ShipRuntime::restore(initial).unwrap(),
        60,
        17,
        Duration::ZERO,
    )
    .unwrap();
    assert_eq!(reference.snapshot(), worker.finish().unwrap());
}
#[test]
fn interventions_are_typed_ordered_replayable_and_invalid_admission_is_atomic() {
    let mut a = fixture(2, 9).unwrap();
    let initial = a.snapshot();
    let invalid = Intervention {
        tick: 0,
        sequence: 1,
        origin: "".into(),
        action: Operation::SetAmbientLight { value: 12 },
    };
    assert!(a.submit(invalid).is_err());
    assert_eq!(initial, a.snapshot());
    let commands = [
        Intervention {
            tick: 3,
            sequence: 20,
            origin: "protocol".into(),
            action: Operation::SetAmbientLight { value: 30 },
        },
        Intervention {
            tick: 3,
            sequence: 10,
            origin: "protocol".into(),
            action: Operation::SetAmbientLight { value: 20 },
        },
        Intervention {
            tick: 4,
            sequence: 30,
            origin: "protocol".into(),
            action: Operation::SetEnvironment {
                environment: Environment {
                    drag_per_mille: 500,
                    ..Environment::suspended()
                },
            },
        },
    ];
    let mut b = ShipRuntime::restore(initial).unwrap();
    for c in commands {
        a.submit(c.clone()).unwrap();
        b.submit(c).unwrap();
    }
    let before = a.snapshot();
    let dupe = Intervention {
        tick: 9,
        sequence: 10,
        origin: "protocol".into(),
        action: Operation::SetAmbientLight { value: 1 },
    };
    assert!(a.submit(dupe).is_err());
    assert_eq!(before, a.snapshot());
    a.advance(8).unwrap();
    b.advance(8).unwrap();
    assert_eq!(a.snapshot(), b.snapshot());
    assert_eq!(a.state().config.ambient_light, 30);
    let order = a
        .state()
        .events
        .iter()
        .filter_map(|e| {
            if let Event::Applied { sequence, .. } = e {
                Some(*sequence)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(order, vec![10, 20, 30]);
    let late = Intervention {
        tick: 0,
        sequence: 40,
        origin: "protocol".into(),
        action: Operation::SetAmbientLight { value: 1 },
    };
    assert!(a.submit(late).is_err());
    a.validate().unwrap();
}
#[test]
fn snapshots_continue_exactly_and_reject_broken_clocks_stocks_and_identity() {
    let mut a = fixture(3, 10).unwrap();
    a.advance(7).unwrap();
    let path = std::env::temp_dir().join(format!("ship-state-{}.json", std::process::id()));
    a.snapshot().save(&path).unwrap();
    let mut b = ShipRuntime::restore(WorldSnapshot::load(&path).unwrap()).unwrap();
    a.advance(9).unwrap();
    b.advance(9).unwrap();
    assert_eq!(a.snapshot(), b.snapshot());
    for mode in 0..4 {
        let mut s = a.snapshot();
        match mode {
            0 => s.tick += 1,
            1 => s.instances.get_mut(&1).unwrap().book.reservoir += 1,
            2 => s.instances.get_mut(&1).unwrap().id = 9,
            _ => s.instances.get_mut(&1).unwrap().position.local[0] = REGION,
        };
        assert!(ShipRuntime::restore(s).is_err());
    }
    std::fs::remove_file(path).unwrap();
}
#[test]
fn ship_scale_micro_detail_and_region_crossings_survive_camera_origin_changes() {
    for x in [-30_000_000 * SCALE, -1, 0, REGION - 1, 30_000_000 * SCALE] {
        let p = Position::from_absolute([x, 1_500_000 * SCALE, -x]).unwrap();
        let q = p.offset([1, -1, 1]).unwrap();
        assert_eq!(q.relative(p).unwrap(), [1, -1, 1]);
        assert_eq!(Position::from_absolute(p.absolute().unwrap()).unwrap(), p);
        let local = q.relative(p).unwrap().map(|v| v as f32 / SCALE as f32);
        assert_eq!(local, [0.000001, -0.000001, 0.000001]);
    }
    assert!(Position::from_absolute([MAX_POSITION + 1, 0, 0]).is_err());
    assert!(
        Position {
            region: [i64::MAX, 0, 0],
            local: [0; 3]
        }
        .absolute()
        .is_err()
    );
}
#[test]
fn light_inputs_are_world_owned_and_readable_without_rendering_the_port() {
    let mut a = fixture(2, 11).unwrap();
    a.advance(4).unwrap();
    let before = a.digest().unwrap();
    for k in 0..30 {
        sphere(k).inspect(a.state()).unwrap();
    }
    assert_eq!(before, a.digest().unwrap());
    assert!(a.state().instances[&1].exposure > 10);
    assert_ne!(
        a.state().instances[&1].exposure,
        a.state().instances[&2].exposure
    );
}
#[test]
fn failed_tick_commits_no_partial_book_updates_or_lost_interventions() {
    let mut world = fixture(2, 13).unwrap().snapshot();
    world.instances.get_mut(&2).unwrap().position =
        Position::from_absolute([MAX_POSITION, 0, 0]).unwrap();
    // Set exposure from a valid zero-port configuration, then force drift beyond boundary.
    world.config.ports.clear();
    for i in world.instances.values_mut() {
        i.exposure = world.config.ambient_light;
    }
    let mut a = ShipRuntime::restore(world).unwrap();
    a.submit(Intervention {
        tick: 0,
        sequence: 1,
        origin: "test".into(),
        action: Operation::SetAmbientLight { value: 50 },
    })
    .unwrap();
    let before = a.snapshot();
    assert!(a.tick().is_err());
    assert_eq!(before, a.snapshot());
}
#[test]
fn malformed_history_and_unrestorable_illumination_are_rejected() {
    let mut a = fixture(2, 31).unwrap();
    a.advance(2).unwrap();
    let mut s = a.snapshot();
    let event = s
        .events
        .iter()
        .find(|e| matches!(e, Event::Emitted { .. }))
        .unwrap()
        .clone();
    s.events.push(event);
    assert!(ShipRuntime::restore(s).is_err());
    let mut s = fixture(1, 32).unwrap().snapshot();
    s.config.ambient_light = 0;
    s.config.ports[0].intensity = u64::MAX - 1;
    // Origin is20metres from the port, so exposure fits even though a large ambient budget would not.
    s.instances.get_mut(&1).unwrap().exposure = (u64::MAX - 1) / 401;
    let mut a = ShipRuntime::restore(s).unwrap();
    let before = a.snapshot();
    assert!(
        a.submit(Intervention {
            tick: 0,
            sequence: 1,
            origin: "operator".into(),
            action: Operation::SetAmbientLight { value: 100 }
        })
        .is_err()
    );
    assert_eq!(before, a.snapshot());
}
#[test]
fn actual_ship_instances_and_parcels_keep_micro_detail_at_vessel_scale() {
    let mut near = fixture(1, 72).unwrap().snapshot();
    near.config.ports.clear();
    near.instances.get_mut(&1).unwrap().exposure = near.config.ambient_light;
    let mut far = near.clone();
    let shift = [30_000_000 * SCALE, 1_500_000 * SCALE, -30_000_000 * SCALE];
    far.instances.get_mut(&1).unwrap().position = Position::from_absolute(shift).unwrap();
    let mut a = ShipRuntime::restore(near).unwrap();
    let mut b = ShipRuntime::restore(far).unwrap();
    a.advance(3).unwrap();
    b.advance(3).unwrap();
    assert_eq!(
        b.state().instances[&1]
            .position
            .relative(a.state().instances[&1].position)
            .unwrap(),
        shift
    );
    for (a, b) in a.state().instances[&1]
        .book
        .parcels
        .iter()
        .zip(&b.state().instances[&1].book.parcels)
    {
        assert_eq!(a.velocity, b.velocity);
        for (i, shift) in shift.iter().enumerate() {
            assert_eq!(b.position[i] - a.position[i], *shift);
        }
    }
    assert_eq!(a.state().instances[&1].book.residual().unwrap(), 0);
    assert_eq!(b.state().instances[&1].book.residual().unwrap(), 0);
}
