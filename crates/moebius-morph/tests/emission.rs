use moebius_morph::emission::*;
use moebius_morph::{AnatomySpec, CompiledAnatomy, Shape};

#[test]
fn axial_roll_precedes_yaw_and_legacy_zero_roll_is_omitted() {
    let rigid = RigidPose {
        axial_roll: 1_570_796,
        yaw_sin: SCALE,
        yaw_cos: 0,
        ..RigidPose::identity()
    };
    let point = place_point([1., 0., 0.], rigid).unwrap();
    assert!(point[0].abs() <= 1 && point[2].abs() <= 1);
    assert_eq!(point[1], SCALE);
    assert!(
        !serde_json::to_string(&RigidPose::identity())
            .unwrap()
            .contains("axial_roll")
    );
    for roll in [-1, 6_283_185, i64::MAX] {
        assert!(
            place_point(
                [0.; 3],
                RigidPose {
                    axial_roll: roll,
                    ..RigidPose::identity()
                }
            )
            .is_err()
        );
    }
}

#[test]
fn controlled_experiments_conserve_and_reproduce() {
    let report = evaluate_morph003().unwrap();
    assert_eq!(report.experiment_a.parcels, 0);
    assert_eq!(report.experiment_a.reservoir, report.experiment_a.initial);
    assert_eq!(report.experiment_a.sinks, 0);
    assert_eq!(report.experiment_a.residual, 0);
    assert_eq!(report.experiment_a.tick, 20);

    assert_eq!(
        report.experiment_b.after_steps,
        report.experiment_b.expected_after_steps
    );
    assert_eq!(
        report.experiment_b.parcel_after_body_shift,
        report.experiment_b.after_steps
    );
    assert_ne!(
        report.experiment_b.parcel_after_body_shift,
        report.experiment_b.nozzle_after_body_shift
    );
    assert_eq!(report.experiment_b.residual, 0);
    assert!(report.experiment_b.runtime_operational);

    assert_eq!(
        report.experiment_c.first_birth,
        report.experiment_c.first_after_motion
    );
    assert_eq!(report.experiment_c.separation, [8 * SCALE, 0, 0]);
    assert_eq!(report.experiment_c.parcels, 8);
    assert_eq!(report.experiment_c.residual, 0);

    assert_ne!(report.experiment_d.drag_free, report.experiment_d.drag_full);
    assert_ne!(
        report.experiment_d.drag_free,
        report.experiment_d.accelerated
    );
    assert_ne!(
        report.experiment_d.drag_full,
        report.experiment_d.accelerated
    );
    assert_eq!(report.experiment_d.residuals, [0, 0, 0]);

    assert_eq!(report.experiment_e.initial, 5);
    assert_eq!(report.experiment_e.reservoir, 2);
    assert_eq!(report.experiment_e.in_world, 3);
    assert_eq!(report.experiment_e.parcels, 1);
    assert!(
        report
            .experiment_e
            .second_status
            .contains("ReservoirExhausted")
    );
    assert_eq!(report.experiment_e.residual, 0);

    assert!(report.experiment_f.equal);
    assert_eq!(report.experiment_f.hash, report.experiment_f.repeat_hash);
    assert_eq!(report.experiment_f.hash, report.experiment_f.reloaded_hash);

    assert!(report.conservation_faults.duplicate_residual > 0);
    assert!(report.conservation_faults.deletion_residual < 0);
    assert_eq!(report.conservation_faults.honest_residual, 0);

    assert_eq!(report.capacity.parcels, 2);
    assert!(report.capacity.extra_status.contains("CapacityExhausted"));
    assert_eq!(report.capacity.residual, 0);
    assert_eq!(report.capacity.full_bound_parcels, MAX_PARCELS);
    assert!(
        report
            .capacity
            .full_bound_status
            .contains("CapacityExhausted")
    );

    assert!(!report.nozzle.anatomical_operational);
    assert!(!report.nozzle.runtime_before);
    assert!(report.nozzle.runtime_after);
    assert!(report.nozzle.translated);
    assert!(report.nozzle.rotated);
    assert!(report.nozzle.bent);
    assert!(report.nozzle.reattached);
    assert_eq!(report.nozzle.channel, SECRETION_CHANNEL);

    assert_eq!(report.model_check.drag_free_steps, 120_000);
    assert!(report.model_check.drag_full_stays);
    assert_eq!(
        report.model_check.accel_displacement,
        report.model_check.expected_accel_displacement
    );
    assert_eq!(report.model_check.expected_accel_displacement, 165_000);
    assert_eq!(report.model_check.negative_blend, -2);
    assert!(report.cost.fill_steps_micros < 5_000_000);
    assert!(report.cost.bound_bytes > 0);
}

#[test]
fn duplicate_tick_and_accounted_sink_are_explicit() {
    let spec = AnatomySpec::showcase(20261123);
    let nozzle = spec
        .components
        .iter()
        .find_map(|component| match component.shape {
            Shape::SecretionNozzle { .. } => Some(component.id),
            _ => None,
        })
        .unwrap();
    let mut book = MaterialBook::new(7, 7, 9, Environment::suspended(), 16).unwrap();
    book.register_nozzle(nozzle, SECRETION_CHANNEL).unwrap();
    book.emission_enabled = true;
    let first = book.try_emit(EmitRequest {
        nozzle_id: nozzle,
        origin: [0, 0, 0],
        direction: [0, 0, SCALE],
        quantity: 4,
        launch_speed: 0,
        channel: SECRETION_CHANNEL,
    });
    assert!(first.admitted());
    let duplicate = book.try_emit(EmitRequest {
        nozzle_id: nozzle,
        origin: [1, 0, 0],
        direction: [0, 0, SCALE],
        quantity: 4,
        launch_speed: 0,
        channel: SECRETION_CHANNEL,
    });
    assert!(matches!(duplicate, EmissionStatus::DuplicateTick { .. }));
    assert_eq!(book.parcels.len(), 1);
    assert_eq!(book.reservoir, 5);
    book.sink_parcel(book.parcels[0].id, 4).unwrap();
    assert!(book.parcels.is_empty());
    assert_eq!(book.sinks, 4);
    assert_eq!(book.reservoir, 5);
    assert_eq!(book.residual().unwrap(), 0);
    let wrong = book.try_emit(EmitRequest {
        nozzle_id: nozzle,
        origin: [0, 0, 0],
        direction: [0, 0, SCALE],
        quantity: 1,
        launch_speed: 0,
        channel: "not-the-substrate",
    });
    assert_eq!(wrong, EmissionStatus::ChannelMismatch);
    assert_eq!(book.reservoir, 5);
}

#[test]
fn filesystem_roundtrip_rejects_a_broken_ledger() {
    let mut book = MaterialBook::new(3, 3, 6, Environment::suspended(), 8).unwrap();
    book.register_nozzle(4, SECRETION_CHANNEL).unwrap();
    book.emission_enabled = true;
    book.try_emit(EmitRequest {
        nozzle_id: 4,
        origin: [0, 0, 0],
        direction: [SCALE, 0, 0],
        quantity: 2,
        launch_speed: 10,
        channel: SECRETION_CHANNEL,
    });
    let path = std::env::temp_dir().join(format!("morph003-book-{}.json", std::process::id()));
    book.save(&path).unwrap();
    let loaded = MaterialBook::load(&path).unwrap();
    assert_eq!(loaded, book);
    let mut broken = std::fs::read_to_string(&path).unwrap();
    broken = broken.replacen("\"quantity\": 2", "\"quantity\": 9", 1);
    std::fs::write(&path, &broken).unwrap();
    assert!(MaterialBook::load(&path).is_err());
    std::fs::remove_file(path).unwrap();
}

#[test]
fn showcase_anatomy_is_unchanged_by_the_book() {
    let spec = AnatomySpec::showcase(20261123);
    let _ = CompiledAnatomy::new(spec.clone()).unwrap();
    assert!(spec.components.iter().any(|component| {
        matches!(
            &component.shape,
            Shape::SecretionNozzle {
                output,
                ..
            } if !output.operational && output.material_channel == SECRETION_CHANNEL
        )
    }));
}
