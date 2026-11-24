use moebius_morph::{emission::*, filament::*, runtime::*, witness::*};
use std::collections::BTreeSet;
#[test]
fn head_first_demo_keeps_the_actual_mouth_ahead_and_conserves_material() {
    let mut w = construction_fixture(20261123, 360, 512, "head-first").unwrap();
    let rules = w.state().config.construction.as_ref().unwrap();
    let anatomy =
        moebius_morph::CompiledAnatomy::new(w.state().config.definitions[&1].clone()).unwrap();
    for (k, pair) in rules.route.windows(2).enumerate() {
        let rigid = pair[0];
        let delta = std::array::from_fn::<_, 3, _>(|a| {
            (pair[1].translation[a] - rigid.translation[a]) as f64
        });
        let length = delta.iter().map(|v| v * v).sum::<f64>().sqrt();
        let forward = [
            rigid.yaw_sin as f64 / SCALE as f64,
            0.,
            rigid.yaw_cos as f64 / SCALE as f64,
        ];
        let alignment = (0..3).map(|a| forward[a] * delta[a]).sum::<f64>() / length;
        assert!(alignment > 0.99, "head must lead, tick {k}: {alignment}");
        if k % 24 == 0 {
            let g = anatomy.pose(construction_pose(k as u64)).unwrap().geometry;
            let tail = g.surfaces[0].positions[0];
            let local = std::array::from_fn::<_, 3, _>(|a| (g.mouth_center[a] - tail[a]) as f64);
            let world = [
                local[0] * forward[2] + local[2] * forward[0],
                local[1],
                -local[0] * forward[0] + local[2] * forward[2],
            ];
            assert!((0..3).map(|a| world[a] * delta[a]).sum::<f64>() > 0.);
        }
    }
    w.advance(360).unwrap();
    w.validate().unwrap();
    assert!(w.state().assembly.as_ref().unwrap().nodes() > 300);
    assert_eq!(w.state().instances[&1].book.residual().unwrap(), 0);
}
#[test]
fn transfer_is_exact_once_and_free_bound_ledger_detects_duplication_and_loss() {
    let mut w = construction_fixture(5, 64, 32, "straight").unwrap();
    w.advance(20).unwrap();
    w.validate().unwrap();
    let i = &w.state().instances[&1];
    let ledger = i.book.ledger().unwrap();
    assert!(ledger.bound > 0 && ledger.in_world > 0);
    assert_eq!(
        ledger.initial,
        ledger.reservoir + ledger.in_world + ledger.bound + ledger.sinks
    );
    assert_eq!(ledger.residual, 0);
    let mut book = i.book.clone();
    let before = book.clone();
    let b = &book.bound[0];
    assert!(
        book.bind(b.parcel.id, b.parcel.quantity, b.filament, b.node)
            .is_err()
    );
    assert_eq!(book, before);
    let p = book.parcels[0];
    assert!(book.bind(p.id, p.quantity + 1, 100, 1).is_err());
    assert_eq!(book, before);
    assert!(book.bind(p.id, 0, 100, 1).is_err());
    assert_eq!(book, before);
    let b = book.bound[0];
    book.parcels.push(b.parcel);
    assert!(book.residual().unwrap() > 0);
    book.parcels.pop();
    book.bound.remove(0);
    assert!(book.residual().unwrap() < 0);
}
#[test]
fn horizontal_vertical_curved_gapped_and_rotating_growth_have_valid_persistent_topology() {
    for kind in ["straight", "vertical", "curved", "gap"] {
        let mut w = construction_fixture(7, 96, 70, kind).unwrap();
        w.advance(80).unwrap();
        w.validate().unwrap();
        let a = w.state().assembly.as_ref().unwrap().clone();
        assert!(a.nodes() > 20, "{kind}: {} nodes", a.nodes());
        assert!(a.filaments.values().any(|f| f.nodes.len() > 2));
        if kind == "gap" {
            assert!(a.filaments.len() > 1);
        }
        w.advance(20).unwrap();
        assert_eq!(a, *w.state().assembly.as_ref().unwrap());
        assert_eq!(w.state().instances[&1].book.residual().unwrap(), 0);
        assert_eq!(w.state().instances[&1].book.reservoir, 0);
    }
    let mut s = construction_fixture(8, 80, 50, "curved")
        .unwrap()
        .snapshot();
    let r = s.config.construction.as_mut().unwrap();
    for (k, p) in r.route.iter_mut().enumerate() {
        let yaw = k as f64 * 0.01;
        p.yaw_sin = (yaw.sin() * SCALE as f64).round() as i64;
        p.yaw_cos = (yaw.cos() * SCALE as f64).round() as i64;
    }
    let mut w = ShipRuntime::restore(s).unwrap();
    w.advance(70).unwrap();
    w.validate().unwrap();
    assert!(w.state().assembly.as_ref().unwrap().nodes() > 20);
}
#[test]
fn no_material_exhaustion_pause_restart_and_hard_budgets_are_explicit() {
    let mut empty = construction_fixture(9, 32, 0, "straight").unwrap();
    empty.advance(10).unwrap();
    assert_eq!(empty.state().assembly.as_ref().unwrap().nodes(), 0);
    assert!(empty.state().events.iter().any(
        |e| matches!(e,Event::EmissionRejected{reason,..}if reason.contains("ReservoirExhausted"))
    ));
    let mut s = construction_fixture(9, 100, 64, "straight")
        .unwrap()
        .snapshot();
    s.config.construction.as_mut().unwrap().emit[20..25].fill(false);
    let mut w = ShipRuntime::restore(s).unwrap();
    w.advance(60).unwrap();
    w.validate().unwrap();
    assert_eq!(w.state().assembly.as_ref().unwrap().filaments.len(), 1);
    let mut s = construction_fixture(9, 100, 64, "straight")
        .unwrap()
        .snapshot();
    s.config.construction.as_mut().unwrap().max_nodes = 4;
    let mut limited = ShipRuntime::restore(s).unwrap();
    limited.advance(30).unwrap();
    assert_eq!(limited.state().assembly.as_ref().unwrap().nodes(), 4);
    assert!(
        limited
            .state()
            .events
            .iter()
            .any(|e| matches!(e,Event::AssemblyRejected{reason,..}if reason=="NodeBudget"))
    );
    limited.validate().unwrap();
    let mut s = construction_fixture(9, 64, 40, "straight")
        .unwrap()
        .snapshot();
    s.config.construction.as_mut().unwrap().max_events = 3;
    let mut capped = ShipRuntime::restore(s).unwrap();
    capped.tick().unwrap();
    let before = capped.snapshot();
    assert!(capped.tick().is_err());
    assert_eq!(before, capped.snapshot());
}
#[test]
fn assembly_and_animated_emission_are_witness_neutral_and_midway_restore_exact() {
    let mut a = construction_fixture(11, 100, 64, "curved").unwrap();
    let mut b = ShipRuntime::restore(a.snapshot()).unwrap();
    let mut c = ShipRuntime::restore(a.snapshot()).unwrap();
    let mut restored = None;
    let sphere = WitnessSphere {
        center: Position::from_absolute([0; 3]).unwrap(),
        radius: 10 * SCALE,
    };
    let mut cache = PresentationCache::default();
    for k in 0..90 {
        a.tick().unwrap();
        b.tick().unwrap();
        sphere.inspect(b.state()).unwrap();
        c.tick().unwrap();
        let before = c.digest().unwrap();
        let moving = WitnessSphere {
            center: Position::from_absolute([if k % 2 == 0 { 0 } else { 100 * SCALE }, 0, 0])
                .unwrap(),
            radius: if k % 3 == 0 { 5 * SCALE } else { SCALE },
        };
        cache.update(&moving.inspect(c.state()).unwrap());
        assert_eq!(before, c.digest().unwrap());
        assert_eq!(a.snapshot(), b.snapshot());
        assert_eq!(a.snapshot(), c.snapshot());
        if k == 44 {
            let path = std::env::temp_dir().join(format!("morph005-{}.json", std::process::id()));
            a.snapshot().save(&path).unwrap();
            restored = Some(ShipRuntime::restore(WorldSnapshot::load(&path).unwrap()).unwrap());
            std::fs::remove_file(path).unwrap();
        } else if k > 44 {
            restored.as_mut().unwrap().tick().unwrap();
        }
    }
    assert_eq!(a.snapshot(), restored.unwrap().snapshot());
    a.validate().unwrap();
    let mut duplicate = a.snapshot();
    let event = duplicate
        .events
        .iter()
        .find(|e| matches!(e, Event::Deposited { .. }))
        .unwrap()
        .clone();
    duplicate.events.push(event);
    assert!(ShipRuntime::restore(duplicate).is_err());
    let mut negative = serde_json::to_value(a.state()).unwrap();
    negative["instances"]["1"]["book"]["bound"][0]["parcel"]["quantity"] = serde_json::json!(-1);
    assert!(serde_json::from_value::<WorldSnapshot>(negative).is_err());
    let mut bad = a.snapshot();
    bad.assembly
        .as_mut()
        .unwrap()
        .filaments
        .values_mut()
        .next()
        .unwrap()
        .quantity += 1;
    assert!(ShipRuntime::restore(bad).is_err());
}
#[test]
fn structural_geometry_rejects_zero_length_teleport_and_foreign_material() {
    let mut w = construction_fixture(13, 50, 32, "straight").unwrap();
    w.advance(8).unwrap();
    let mut s = w.snapshot();
    let a = s.assembly.as_mut().unwrap();
    let f = a.filaments.values_mut().next().unwrap();
    f.nodes[1].position = f.nodes[0].position;
    assert!(ShipRuntime::restore(s).is_err());
    let r = w.state().config.construction.as_ref().unwrap().clone();
    let mut a = Assembly::default();
    let mut book = MaterialBook::new(1, 13, 10, Environment::suspended(), 64).unwrap();
    book.register_nozzle(r.nozzle, SECRETION_CHANNEL).unwrap();
    book.emission_enabled = true;
    assert!(
        book.try_emit(EmitRequest {
            nozzle_id: r.nozzle,
            origin: [0; 3],
            direction: [SCALE, 0, 0],
            quantity: 1,
            launch_speed: 0,
            channel: SECRETION_CHANNEL
        })
        .admitted()
    );
    book.step().unwrap();
    book.step().unwrap();
    let p = book.parcels[0];
    let position = Position::from_absolute(p.position).unwrap();
    let result = a.deposit(&mut book, &r, p.id, position).unwrap();
    let before = (a.clone(), book.clone());
    assert!(a.deposit(&mut book, &r, p.id, position).is_err());
    assert_eq!((a, book), before);
    assert!(result.0 > 0);
}
#[test]
fn earlier_m4_serialized_state_is_unchanged_when_assembly_is_absent() {
    let mut golden = fixture(16, 20261123).unwrap();
    golden.advance(120).unwrap();
    assert_eq!(
        golden.digest().unwrap(),
        "4f34d8daad96fd1fc84860a53168ed0e6580a68fae095bbdf3ed7bf4b3154552"
    );
    let mut w = fixture(4, 14).unwrap();
    w.advance(10).unwrap();
    let text = serde_json::to_string(w.state()).unwrap();
    assert!(
        !text.contains("\"bound\"")
            && !text.contains("\"assembly\"")
            && !text.contains("\"construction\"")
    );
    let ids = w.state().instances.keys().copied().collect::<BTreeSet<_>>();
    assert_eq!(ids, (1..=4).collect());
}
#[test]
fn structural_index_meshes_and_boundary_reentry_are_bounded_read_only() {
    let mut w = construction_fixture(77, 200, 128, "curved").unwrap();
    w.advance(160).unwrap();
    let before = w.digest().unwrap();
    let a = w.state().assembly.as_ref().unwrap();
    let index = StructuralIndex::build(a).unwrap();
    assert!(index.chunks.len() <= a.nodes().div_ceil(CHUNK_EDGES) + a.filaments.len());
    let sphere = WitnessSphere {
        center: w.state().instances[&1].position,
        radius: 20 * SCALE,
    };
    let visible = index.visible(&sphere).unwrap();
    assert!(!visible.is_empty());
    for (id, chunk) in &visible {
        let (_, mesh) = filament_chunk(&a.filaments[id], *chunk).unwrap();
        mesh.audit().unwrap();
        assert!(mesh.positions.len() <= CHUNK_EDGES * 8 + 16);
    }
    let away = WitnessSphere {
        center: Position::from_absolute([1000 * SCALE, 0, 0]).unwrap(),
        radius: SCALE,
    };
    assert!(index.visible(&away).unwrap().is_empty());
    assert_eq!(visible, index.visible(&sphere).unwrap());
    assert_eq!(before, w.digest().unwrap());
}
#[test]
fn controlled_clock_pauses_steps_resets_and_preserves_every_numerical_tick() {
    use std::time::{Duration, Instant};
    let initial = construction_fixture(88, 100, 64, "curved")
        .unwrap()
        .snapshot();
    let worker = Worker::interactive(
        ShipRuntime::restore(initial.clone()).unwrap(),
        90,
        Duration::from_millis(1),
        true,
    )
    .unwrap();
    let wait = |test: &dyn Fn(&PlaybackStatus) -> bool| {
        let start = Instant::now();
        loop {
            let status = worker.playback().unwrap();
            if test(&status) {
                return status;
            }
            assert!(start.elapsed() < Duration::from_secs(5));
            std::thread::sleep(Duration::from_millis(1));
        }
    };
    std::thread::sleep(Duration::from_millis(15));
    assert_eq!(worker.latest().unwrap().tick, 0);
    worker.control(Playback::Step).unwrap();
    wait(&|s| s.tick == 1);
    std::thread::sleep(Duration::from_millis(15));
    assert_eq!(worker.latest().unwrap().tick, 1);
    worker.control(Playback::Rate(16)).unwrap();
    worker.control(Playback::Play).unwrap();
    wait(&|s| s.tick == 90);
    let mut reference = ShipRuntime::restore(initial.clone()).unwrap();
    reference.advance(90).unwrap();
    assert_eq!(reference.snapshot(), *worker.latest().unwrap());
    worker.control(Playback::Reset).unwrap();
    wait(&|s| s.generation == 1 && s.tick == 0);
    worker.control(Playback::Step).unwrap();
    wait(&|s| s.tick == 1);
    worker.control(Playback::Stop).unwrap();
    let final_state = worker.finish().unwrap();
    let mut reference = ShipRuntime::restore(initial).unwrap();
    reference.tick().unwrap();
    assert_eq!(final_state, reference.snapshot());
}
