use moebius_morph::{emission::SCALE, runtime::*, *};
fn spec() -> AnatomySpec {
    let mut s = AnatomySpec::bare(77);
    let n = s.components[0].clone();
    for k in 1..4 {
        let mut c = n.clone();
        c.id = s.next_id();
        c.attachment.azimuth = (k as f32 * 1.57).rem_euclid(std::f32::consts::TAU);
        s.components.push(c);
    }
    s.add_food_pouch();
    s
}
fn world() -> ShipRuntime {
    ShipRuntime::habitat_simulation(spec(), 77, 1600).unwrap()
}
fn dinner() -> WorldSnapshot {
    let mut s = world().snapshot();
    s.habitat.as_mut().unwrap().rules.evasion = false;
    s.config.ports.clear();
    s.config.ambient_light = 50;
    for i in s.instances.values_mut() {
        i.exposure = 50;
        if i.definition == BLOOD_SAC {
            i.position = Position::from_absolute([100 * SCALE, 0, 0]).unwrap();
        }
    }
    let mouth = s.mouth_position(1).unwrap().unwrap();
    let sac = s.instances.get_mut(&2).unwrap();
    sac.position = mouth;
    sac.book.initial += 700;
    sac.book.reservoir += 700;
    s.condensate.as_mut().unwrap().biomass -= 700;
    assert_eq!(s.condensate_gap(), 0);
    s
}
#[test]
fn pouch_is_anatomy_with_finite_capacity_and_valid_persistence() {
    let s = spec();
    assert_eq!(s.food_capacity(), 1024);
    let c = CompiledAnatomy::new(s.clone()).unwrap();
    c.pose(Pose::rest(&s.body))
        .unwrap()
        .geometry
        .audit()
        .unwrap();
    let j = serde_json::to_string(&s).unwrap();
    assert_eq!(anatomy_from_json(&j).unwrap(), s);
    let mut w = ShipRuntime::restore(dinner()).unwrap();
    let mut peak_stock = 0;
    for _ in 0..120 {
        w.tick().unwrap();
        peak_stock = peak_stock.max(w.state().instances[&1].book.reservoir);
        assert!(w.state().instances[&1].book.reservoir <= 1024);
    }
    eprintln!(
        "POUCH observed_peak={peak_stock} final_stock={}",
        w.state().instances[&1].book.reservoir
    );
    assert!(
        w.state().instances[&1].book.reservoir > 256,
        "pouch never stored a large meal"
    );
    assert!(
        w.state()
            .events
            .iter()
            .any(|e| matches!(e, Event::Fed { organism: 1, .. }))
    );
    w.validate().unwrap();
}
#[test]
fn feeding_closes_secretion_and_exploration_has_directional_memory() {
    let mut w = world();
    let mut moving = 0;
    let mut meals = 0;
    let mut moved_after_food = false;
    for _ in 0..420 {
        let old = w.snapshot();
        w.tick().unwrap();
        let s = w.state();
        let emitted = s
            .events
            .iter()
            .filter(|e| matches!(e,Event::Emitted{at,organism:1,..}if *at==old.tick))
            .count();
        let fed = s
            .events
            .iter()
            .any(|e| matches!(e,Event::Fed{at,organism:1,..}if *at==old.tick));
        if fed {
            meals += 1;
            assert_eq!(emitted, 0, "a gulp tick must not lay pipe");
        }
        if meals > 0 && s.instances[&1].position != old.instances[&1].position {
            moved_after_food = true;
        }
        if emitted > 0 {
            moving += 1;
            assert_ne!(s.instances[&1].position, old.instances[&1].position);
        }
        let a = &old.habitat.as_ref().unwrap().agents[&1];
        let b = &s.habitat.as_ref().unwrap().agents[&1];
        let dot: i128 = (0..3)
            .map(|k| i128::from(a.heading[k]) * i128::from(b.heading[k]))
            .sum();
        assert!(
            dot > i128::from(SCALE).pow(2) * 98 / 100,
            "abrupt heading reversal"
        );
        assert_eq!(s.condensate_gap(), 0);
        if s.tick.is_multiple_of(60) {
            w.validate().unwrap();
        }
    }
    println!(
        "moving={moving} meals={meals} intake={} bound={}",
        w.state().instances[&1].book.external_input,
        w.state().instances[&1].book.bound.len()
    );
    assert!(
        moving > 5 && meals > 0 && moved_after_food,
        "moving={moving} meals={meals} moved_after_food={moved_after_food} contacts: {:?}",
        w.state().habitat.as_ref().unwrap().contacts
    );
}
#[test]
fn paid_offspring_are_active_and_resume_exactly_after_reload() {
    let mut s = dinner();
    s.habitat.as_mut().unwrap().rules.birth_interval = 60;
    let mut w = ShipRuntime::restore(s).unwrap();
    w.advance(120).unwrap();
    w.validate().unwrap();
    assert!(
        w.state().habitat.as_ref().unwrap().agents.len() > 1,
        "no paid birth"
    );
    let child = *w
        .state()
        .habitat
        .as_ref()
        .unwrap()
        .agents
        .keys()
        .find(|&&id| id != 1)
        .unwrap();
    assert!(w.state().instances[&child].book.emission_enabled);
    let saved = serde_json::to_vec(&w.snapshot()).unwrap();
    let mut r = ShipRuntime::restore(serde_json::from_slice(&saved).unwrap()).unwrap();
    let p = w.state().instances[&child].position;
    for _ in 0..60 {
        w.tick().unwrap();
        r.tick().unwrap();
    }
    assert_ne!(
        w.state().instances[&child].position,
        p,
        "child {:?}; contacts {:?}",
        w.state().habitat.as_ref().unwrap().agents[&child],
        w.state().habitat.as_ref().unwrap().contacts
    );
    assert!(
        w.state()
            .assembly
            .as_ref()
            .unwrap()
            .filaments
            .values()
            .any(|f| f.organism == child)
    );
    assert_eq!(w.snapshot(), r.snapshot());
    w.validate().unwrap();
}
#[test]
fn prey_evasion_is_local_and_substrate_anchors_are_real() {
    let mut s = world().snapshot();
    let mut still = s.clone();
    still.habitat.as_mut().unwrap().rules.evasion = false;
    let before = s.instances[&2].position;
    let mut w = ShipRuntime::restore(s.clone()).unwrap();
    let mut q = ShipRuntime::restore(still).unwrap();
    w.tick().unwrap();
    q.tick().unwrap();
    assert_ne!(w.state().instances[&2].position, before);
    assert_eq!(q.state().instances[&2].position, before);
    w.advance(120).unwrap();
    w.validate().unwrap();
    for (id, a) in &w.state().habitat.as_ref().unwrap().sacs {
        assert_eq!(w.state().instances[id].definition, BLOOD_SAC);
        assert!(moebius_morph::habitat::anchor(w.state(), a).is_some());
    }
    s.habitat.as_mut().unwrap().rules.step = 0;
    assert!(ShipRuntime::restore(s).is_err());
}

#[test]
fn smooth_ageing_sweeps_share_chunk_seams_without_changing_material() {
    use moebius_morph::filament::{CHUNK_EDGES, filament_chunk_at};
    let mut s = world().snapshot();
    s.habitat.as_mut().unwrap().rules.reproduction = false;
    // Explicit numerical fixture: extra founding stock, not production defaults.
    s.instances.get_mut(&1).unwrap().book.initial += 900;
    s.instances.get_mut(&1).unwrap().book.reservoir += 900;
    s.condensate.as_mut().unwrap().founders += 900;
    let mut w = ShipRuntime::restore(s).unwrap();
    w.advance(90).unwrap();
    w.validate().unwrap();
    let before = w.snapshot();
    let f = w
        .state()
        .assembly
        .as_ref()
        .unwrap()
        .filaments
        .values()
        .find(|f| f.nodes.len() > CHUNK_EDGES * 2)
        .unwrap();
    let (o, a) = filament_chunk_at(f, 0, 90).unwrap();
    let (p, b) = filament_chunk_at(f, 1, 90).unwrap();
    let delta = p.relative(o).unwrap().map(|v| v as f32 / SCALE as f32);
    let seam = &a.positions[a.positions.len() - 20..a.positions.len() - 10];
    for (a, b) in seam.iter().zip(&b.positions[..10]) {
        for axis in 0..3 {
            assert!(
                (a[axis] - b[axis] - delta[axis]).abs() < 0.00002,
                "chunk seam disconnected"
            );
        }
    }
    let (_, fresh) = filament_chunk_at(f, 0, f.nodes[0].deposited_tick).unwrap();
    let (_, old) = filament_chunk_at(f, 0, 1000).unwrap();
    let radius = |v: [f32; 3]| v.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!(radius(fresh.positions[0]) > radius(old.positions[0]) * 1.2);
    assert_ne!(fresh.colors, old.colors);
    let profile = f
        .section
        .as_ref()
        .expect("habitat produces actual hollow pipes");
    assert!(profile.inner_radius > 0 && profile.inner_radius < profile.outer_radius);
    // Both sides of the hollow sweep share the inter-chunk boundary.
    for (a, b) in a.positions[a.positions.len() - 10..]
        .iter()
        .zip(&b.positions[10..20])
    {
        for axis in 0..3 {
            assert!((a[axis] - b[axis] - delta[axis]).abs() < 0.00002);
        }
    }
    let inner = radius(fresh.positions[10]);
    let outer = radius(fresh.positions[0]);
    assert!(
        inner > 0.015 && outer > inner * 1.3,
        "no open bore/wall thickness"
    );
    let dot = |a: [f32; 3], b: [f32; 3]| (0..3).map(|k| a[k] * b[k]).sum::<f32>();
    assert!(dot(fresh.positions[0], fresh.normals[0]) > 0.);
    let center: [f32; 3] =
        std::array::from_fn(|k| fresh.positions[10..20].iter().map(|v| v[k]).sum::<f32>() / 10.);
    let radial = std::array::from_fn(|k| fresh.positions[10][k] - center[k]);
    assert!(
        dot(radial, fresh.normals[10]) < 0.,
        "inner normals must face the lumen"
    );
    assert!(a.positions.len() < 1100 && a.indices.len() < 6500);
    assert_eq!(
        w.snapshot(),
        before,
        "surface generation mutated paid state"
    );
}

#[test]
fn junctions_link_existing_paid_nodes_and_never_span_empty_space() {
    use moebius_morph::{emission::*, filament::*};
    let source = world().snapshot();
    let mut rules = source.config.construction.unwrap();
    rules.age = 0;
    rules.organism = 1;
    rules.nozzle = 2;
    let mut book = MaterialBook::new(1, 77, 4, Environment::suspended(), 16).unwrap();
    book.registered.extend([2, 3]);
    book.emission_enabled = true;
    let mut a = Assembly::default();
    for (nozzle, x) in [(2, 0), (3, 50_000)] {
        let status = book.try_emit(EmitRequest {
            nozzle_id: nozzle,
            origin: [x, 0, 0],
            direction: [SCALE, 0, 0],
            quantity: 1,
            launch_speed: 0,
            channel: SECRETION_CHANNEL,
        });
        let EmissionStatus::Admitted { parcel_id, .. } = status else {
            panic!("paid emission refused")
        };
        a.deposit(
            &mut book,
            &rules,
            parcel_id,
            Position::from_absolute([x, 0, 0]).unwrap(),
        )
        .unwrap();
    }
    assert_eq!(a.junctions.len(), 1);
    assert_eq!(book.bound_quantity().unwrap(), 2);
    assert_eq!(book.residual().unwrap(), 0);
    a.validate(&book, &rules).unwrap();
    a.junctions[0].second = (999, 1);
    assert!(a.validate(&book, &rules).is_err());
}

#[test]
fn continuous_lumens_preserve_seals_and_wall_contacts_are_not_passages() {
    let mut s = world().snapshot();
    s.habitat.as_mut().unwrap().rules.reproduction = false;
    s.instances.get_mut(&1).unwrap().book.initial += 900;
    s.instances.get_mut(&1).unwrap().book.reservoir += 900;
    s.condensate.as_mut().unwrap().founders += 900;
    let mut w = ShipRuntime::restore(s).unwrap();
    w.advance(40).unwrap();
    let assembly = w.state().assembly.as_ref().unwrap();
    let f = assembly
        .filaments
        .values()
        .find(|f| f.nodes.len() > 10)
        .unwrap();
    let id = f.id;
    let middle = f.nodes[5].id;
    let root = f.nodes[0].id;
    let tip = f.nodes.last().unwrap().id;
    assert!(assembly.lumen_connected((id, middle), (id, tip)));
    assert!(
        !assembly.lumen_connected((id, middle), (id, root)),
        "sealed root must block passage"
    );
    for other in assembly.filaments.keys().filter(|&&other| other != id) {
        let nodes = &assembly.filaments[other].nodes;
        if nodes.len() < 2 {
            continue;
        }
        let sample = nodes.len() / 2;
        assert!(
            !assembly.lumen_connected((id, middle), (*other, nodes[sample].id)),
            "contact cannot open a branch"
        );
    }
    let mut state = w.snapshot();
    // Seal all existing streams: the positive new-stream check must not depend
    // on which nozzle remains physically clear after contact avoidance.
    for filament in state.assembly.as_mut().unwrap().filaments.values_mut() {
        filament.section.as_mut().unwrap().end_sealed = true;
    }
    let snapshot = serde_json::to_vec(&state).unwrap();
    let mut sealed = ShipRuntime::restore(serde_json::from_slice(&snapshot).unwrap()).unwrap();
    assert!(
        !sealed
            .state()
            .assembly
            .as_ref()
            .unwrap()
            .lumen_connected((id, middle), (id, tip))
    );
    let before = sealed.state().assembly.as_ref().unwrap().filaments[&id].clone();
    let chunk = (before.nodes.len() - 2) / moebius_morph::filament::CHUNK_EDGES;
    let (origin, sealed_mesh) =
        moebius_morph::filament::filament_chunk_at(&before, chunk, 40).unwrap();
    let end = before
        .nodes
        .last()
        .unwrap()
        .position
        .relative(origin)
        .unwrap()
        .map(|x| x as f32 / SCALE as f32);
    assert!(
        sealed_mesh.positions.contains(&end),
        "sealed terminal lacks a membrane"
    );
    let mut opened = before.clone();
    opened.section.as_mut().unwrap().end_sealed = false;
    let (_, open_mesh) = moebius_morph::filament::filament_chunk_at(&opened, chunk, 40).unwrap();
    assert!(
        !open_mesh.positions.contains(&end),
        "open terminal was plugged"
    );
    let all_sealed = sealed.state().assembly.as_ref().unwrap().filaments.clone();
    sealed.advance(5).unwrap();
    for (id, stream) in &all_sealed {
        assert_eq!(
            &sealed.state().assembly.as_ref().unwrap().filaments[id],
            stream,
            "sealed stream was mutated"
        );
    }
    assert_eq!(
        sealed.state().assembly.as_ref().unwrap().filaments[&id],
        before,
        "growth reopened a sealed end"
    );
    assert!(sealed.state().assembly.as_ref().unwrap().filaments.len() > assembly.filaments.len());
    let mut corrupt = state;
    corrupt
        .assembly
        .as_mut()
        .unwrap()
        .filaments
        .get_mut(&id)
        .unwrap()
        .section
        .as_mut()
        .unwrap()
        .inner_radius = 100_000;
    assert!(ShipRuntime::restore(corrupt).is_err());
}

#[test]
fn food_pouch_admission_clips_the_last_bite_and_rejects_overcapacity_saves() {
    let mut s = dinner();
    for c in &mut s.config.definitions.get_mut(&1).unwrap().components {
        if let Shape::FoodPouch { capacity, .. } = &mut c.shape {
            *capacity = 64;
        }
    }
    s.instances.get_mut(&1).unwrap().book.initial += 1;
    s.instances.get_mut(&1).unwrap().book.reservoir += 1;
    s.condensate.as_mut().unwrap().founders += 1;
    s.habitat.as_mut().unwrap().rules.reproduction = false;
    let mut w = ShipRuntime::restore(s).unwrap();
    let mut peak = 0;
    for _ in 0..14 {
        w.tick().unwrap();
        let stock = w.state().instances[&1].book.reservoir;
        peak = peak.max(stock);
        assert!(stock <= 64);
    }
    assert_eq!(peak, 64);
    assert!(
        w.state().events.iter().any(|e| matches!(
            e,
            Event::Fed {
                organism: 1,
                quantity: 3,
                ..
            }
        )),
        "last bite must respect remaining room"
    );
    let mut corrupt = w.snapshot();
    corrupt.instances.get_mut(&1).unwrap().book.reservoir = 65;
    assert!(ShipRuntime::restore(corrupt).is_err());
}

#[test]
fn witness_chunk_bounds_contain_the_entire_curved_hollow_surface() {
    use moebius_morph::filament::{StructuralIndex, filament_chunk_at};
    let mut s = world().snapshot();
    s.habitat.as_mut().unwrap().rules.reproduction = false;
    s.instances.get_mut(&1).unwrap().book.initial += 900;
    s.instances.get_mut(&1).unwrap().book.reservoir += 900;
    s.condensate.as_mut().unwrap().founders += 900;
    let mut w = ShipRuntime::restore(s).unwrap();
    w.advance(10).unwrap();
    let mut a = w.state().assembly.as_ref().unwrap().clone();
    let id = *a.filaments.keys().next().unwrap();
    let f = a.filaments.get_mut(&id).unwrap();
    f.nodes.truncate(4);
    for (n, p) in f.nodes.iter_mut().zip([
        [0, 500_000, 0],
        [0, 0, 0],
        [500_000, 0, 0],
        [500_000, 500_000, 0],
    ]) {
        n.position = Position::from_absolute(p).unwrap();
    }
    let index = StructuralIndex::build(&a).unwrap();
    let (low, high) = index.chunks[&(id, 0)];
    let (origin, surface) = filament_chunk_at(&a.filaments[&id], 0, 0).unwrap();
    let absolute = origin.absolute().unwrap();
    for vertex in surface.positions {
        for axis in 0..3 {
            let p = absolute[axis] + (vertex[axis] * SCALE as f32).round() as i64;
            assert!(
                p >= low[axis] && p <= high[axis],
                "spline protrudes outside Witness chunk bound"
            );
        }
    }
}

#[test]
fn one_founder_colony_matches_habitat_simulation() {
    let steps = 24;
    let direct = ShipRuntime::habitat_simulation(AnatomySpec::showcase(77), 77, steps).unwrap();
    let colony =
        ShipRuntime::habitat_colony(AnatomySpec::showcase(77), 77, steps, 1, None).unwrap();
    assert_eq!(direct.digest().unwrap(), colony.digest().unwrap());
    assert_eq!(direct.snapshot(), colony.snapshot());
}

#[test]
fn extra_founders_are_paid_from_finite_condensate() {
    let steps = 16;
    let one = ShipRuntime::habitat_simulation(AnatomySpec::bare(11), 11, steps).unwrap();
    let cost = one.state().instances[&1].book.initial;
    let biomass = one.state().condensate.as_ref().unwrap().biomass;
    let two = ShipRuntime::habitat_colony(AnatomySpec::bare(11), 11, steps, 2, None).unwrap();
    let agents = &two.state().habitat.as_ref().unwrap().agents;
    assert_eq!(agents.len(), 2);
    let (extra_id, extra) = agents.iter().find(|(id, _)| **id != 1).unwrap();
    assert!(extra.parent.is_none());
    assert_ne!(two.state().instances[extra_id].definition, BLOOD_SAC);
    assert_eq!(two.state().instances[extra_id].book.initial, cost);
    assert_eq!(
        two.state().condensate.as_ref().unwrap().biomass,
        biomass - cost
    );
    assert_eq!(two.state().condensate_gap(), 0);
    assert!(ShipRuntime::habitat_colony(AnatomySpec::bare(11), 11, steps, 17, None).is_err());
    assert!(ShipRuntime::habitat_colony(AnatomySpec::bare(11), 11, steps, 2, Some(0)).is_err());
    let rich =
        ShipRuntime::habitat_colony(AnatomySpec::bare(11), 11, steps, 1, Some(8_000)).unwrap();
    assert_eq!(rich.state().condensate.as_ref().unwrap().biomass, 8_000);
    assert_eq!(rich.state().condensate_gap(), 0);
    let mut live = two;
    live.tick().unwrap();
    assert_eq!(live.state().condensate_gap(), 0);
    live.validate().unwrap();
}

#[test]
fn a_fed_body_pursues_new_space_with_persisted_travel_memory() {
    let mut s = world().snapshot();
    s.habitat.as_mut().unwrap().rules.reproduction = false;
    s.config.ports.clear();
    s.config.ambient_light = 50;
    for i in s.instances.values_mut() {
        i.exposure = 50;
        if i.definition == BLOOD_SAC {
            i.position = Position::from_absolute([100 * SCALE, i.id as i64 * SCALE, 0]).unwrap();
        }
    }
    s.instances.get_mut(&1).unwrap().book.initial += 400;
    s.instances.get_mut(&1).unwrap().book.reservoir += 400;
    s.condensate.as_mut().unwrap().founders += 400;
    let origin = s.instances[&1].position;
    let mut w = ShipRuntime::restore(s).unwrap();
    w.advance(160).unwrap();
    let memory = w.state().habitat.as_ref().unwrap().agents[&1]
        .travel
        .as_ref()
        .unwrap();
    assert!(
        memory.target.is_some() && memory.visited.len() > 3,
        "no new space was visited"
    );
    let d = w.state().instances[&1].position.relative(origin).unwrap();
    assert!(
        d.iter().map(|v| i128::from(*v).pow(2)).sum::<i128>() > i128::from(6 * SCALE).pow(2),
        "fed body did not leave its founding patch"
    );
    let mut restored = ShipRuntime::restore(w.snapshot()).unwrap();
    w.advance(20).unwrap();
    restored.advance(20).unwrap();
    assert!(
        w.snapshot() == restored.snapshot(),
        "warm/cold exploration replay diverged"
    );
    assert_eq!(w.state().condensate_gap(), 0);
    w.validate().unwrap();
}
