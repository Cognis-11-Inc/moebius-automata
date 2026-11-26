//! Spatial-scale measurements for the collision horizon.
//! Not a controller. Measurement filaments are outside the material ledger.
//! Build with `--features spatial-scale`. Feature-off physics does not include
//! this splice.
use moebius_morph::MorphError;
use moebius_morph::emission::{MaterialKind, SCALE};
use moebius_morph::filament::{Assembly, CHUNK_EDGES, Filament, Node, PipeSection};
use moebius_morph::runtime::*;
use std::time::Instant;

const MEASURED_TICKS: u64 = 8;
const MEASUREMENT_ORGANISM: u64 = 9_000_001;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().nth(1).as_deref() == Some("--build-identity") {
        println!("{}", moebius_morph::BENCHMARK_BUILD_IDENTITY);
        return Ok(());
    }
    #[cfg(not(feature = "spatial-scale"))]
    compile_error!("cargo build -p moebius-morph --example spatial_scale --features spatial-scale");
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let out = args
        .first()
        .ok_or("usage: spatial_scale OUT FRESH_SNAPSHOT EIGHT_SNAPSHOT")?;
    let fresh_path = args.get(1).ok_or("missing fresh snapshot")?;
    let eight_path = args.get(2).ok_or("missing eight-founder snapshot")?;
    std::fs::create_dir_all(out)?;
    let fresh = WorldSnapshot::load(std::path::Path::new(fresh_path))?;
    let eight = WorldSnapshot::load(std::path::Path::new(eight_path))?;
    let span = scene_span(&fresh)?;
    let mut rows = Vec::new();
    for factor in [1i64, 10, 100] {
        let shift = if factor == 1 {
            0
        } else {
            align_cell(span.saturating_mul(factor))
        };
        let mut scene = fresh.clone();
        let shifted = shift_scene(&mut scene, [shift, 0, 0]);
        let label = format!("A-empty-scale-{factor}x");
        match shifted.and_then(|_| run(&label, scene, MEASURED_TICKS)) {
            Ok(mut row) => {
                row["shift_x_m"] = serde_json::json!(shift as f64 / SCALE as f64);
                row["local_span_m"] = serde_json::json!(span as f64 / SCALE as f64);
                rows.push(row);
            }
            Err(error) => rows.push(failure(&label, &error.to_string())),
        }
    }
    for chunks in [0usize, 8, 32, 96] {
        let label = format!("B-distant-chunks-{chunks}");
        rows.push(run_prepared(
            &label,
            fresh.clone(),
            MEASURED_TICKS,
            |world| {
                if chunks == 0 {
                    return Ok(());
                }
                let origin = scene_anchor(world.state())?.offset([800 * SCALE, 0, 0])?;
                splice_chunks(world, 50_000, origin, chunks)
            },
        ));
    }
    for chunks in [0usize, 4, 16, 48] {
        let label = format!("C-local-chunks-{chunks}-distant-32");
        rows.push(run_prepared(
            &label,
            fresh.clone(),
            MEASURED_TICKS,
            |world| {
                let anchor = scene_anchor(world.state())?;
                splice_chunks(world, 60_000, anchor.offset([800 * SCALE, 0, 0])?, 32)?;
                if chunks > 0 {
                    splice_chunks(world, 70_000, anchor.offset([2 * SCALE, 0, 0])?, chunks)?;
                }
                Ok(())
            },
        ));
    }
    match grow_then_shift(&fresh, 0) {
        Ok(scene) => rows.push(run("D-grown-contact", scene, MEASURED_TICKS)?),
        Err(error) => rows.push(failure("D-grown-contact", &error.0)),
    }
    match grow_then_shift(&fresh, 800 * SCALE) {
        Ok(scene) => rows.push(run("D-grown-pipes-shifted", scene, MEASURED_TICKS)?),
        Err(error) => rows.push(failure("D-grown-pipes-shifted", &error.0)),
    }
    rows.push(run_prepared(
        "D-approaching-one-pipe",
        fresh.clone(),
        MEASURED_TICKS,
        |world| {
            let (origin, heading) = agent_heading(world.state())?;
            let ahead = origin.offset(heading.map(|value| value.saturating_mul(2)))?;
            splice_chunks(world, 80_000, ahead, 1)
        },
    ));
    rows.push(run("D-contact-fresh", fresh.clone(), MEASURED_TICKS)?);
    rows.push(run("D-congested-eight", eight, MEASURED_TICKS)?);
    let report = serde_json::json!({
        "purpose": "MORPH-005 spatial scaling A-D",
        "measured_ticks": MEASURED_TICKS,
        "warmup_ticks": 1,
        "production_max_position_unchanged": true,
        "measurement_filaments": "organism 9000001, no material-book entries, spliced after restore",
        "empty_scale_meaning": "no habitat extent field exists; the scene is translated by 10x and 100x its own axis span",
        "rows": rows,
    });
    std::fs::write(
        std::path::Path::new(out).join("spatial-scale.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn failure(label: &str, error: &str) -> serde_json::Value {
    serde_json::json!({"label": label, "error": error})
}

fn run(label: &str, scene: WorldSnapshot, ticks: u64) -> Result<serde_json::Value, MorphError> {
    run_prepared_inner(label, scene, ticks, |_| Ok(()))
}

fn run_prepared(
    label: &str,
    scene: WorldSnapshot,
    ticks: u64,
    prepare: impl FnOnce(&mut ShipRuntime) -> Result<(), MorphError>,
) -> serde_json::Value {
    match run_prepared_inner(label, scene, ticks, prepare) {
        Ok(row) => row,
        Err(error) => failure(label, &error.0),
    }
}

fn run_prepared_inner(
    label: &str,
    scene: WorldSnapshot,
    ticks: u64,
    prepare: impl FnOnce(&mut ShipRuntime) -> Result<(), MorphError>,
) -> Result<serde_json::Value, MorphError> {
    let mut world = ShipRuntime::restore(scene)?;
    prepare(&mut world)?;
    let warmup_start = Instant::now();
    world.tick()?;
    let warmup_ns = warmup_start.elapsed().as_nanos();
    let before = world.collision_profile();
    let mut samples = Vec::with_capacity(ticks as usize);
    for _ in 0..ticks {
        let started = Instant::now();
        world.tick()?;
        samples.push(started.elapsed().as_nanos());
    }
    let after = world.collision_profile();
    let state = world.snapshot();
    let sum = samples.iter().copied().sum::<u128>();
    let mean = sum / u128::from(ticks.max(1));
    let max = samples.iter().copied().max().unwrap_or(0);
    Ok(serde_json::json!({
        "label": label,
        "tick": state.tick,
        "agents": state.habitat.as_ref().map(|h| h.agents.len()).unwrap_or(0),
        "filaments": state.assembly.as_ref().map(|a| a.filaments.len()).unwrap_or(0),
        "nodes": state.assembly.as_ref().map(|a| a.nodes()).unwrap_or(0),
        "chunks": state.assembly.as_ref().map(chunk_count).unwrap_or(0),
        "warmup_ns": warmup_ns,
        "measured_ns": samples,
        "mean_ns": mean,
        "max_ns": max,
        "vm_rss_kib": proc_kib("VmRSS:"),
        "vm_hwm_kib": proc_kib("VmHWM:"),
        "distance_queries": delta(after.distance_queries, before.distance_queries),
        "distance_within": delta(after.distance_within, before.distance_within),
        "distance_clear": delta(after.distance_clear, before.distance_clear),
        "distance_bounded": delta(after.distance_bounded, before.distance_bounded),
        "distance_exact": delta(after.distance_exact, before.distance_exact),
        "triangles_examined": delta(after.triangles_examined, before.triangles_examined),
        "queries": delta(after.queries, before.queries),
        "cast_cache_hits": delta(after.cast_cache_hits, before.cast_cache_hits),
        "pipe_builds": delta(after.pipe_builds, before.pipe_builds),
        "pipe_candidates": delta(after.pipe_candidates, before.pipe_candidates),
        "pipe_rejects": delta(after.pipe_rejects, before.pipe_rejects),
        "pipe_horizon_candidates": delta(after.pipe_horizon_candidates, before.pipe_horizon_candidates),
        "pipe_horizon_rejects": delta(after.pipe_horizon_rejects, before.pipe_horizon_rejects),
        "pipe_horizon_rebuilds": delta(after.pipe_horizon_rebuilds, before.pipe_horizon_rebuilds),
        "pipe_horizon_ns": delta_u128(after.pipe_horizon_ns, before.pipe_horizon_ns),
        "pipe_bounds_computed": delta(after.pipe_bounds_computed, before.pipe_bounds_computed),
        "pipe_horizon_indexed": after.pipe_horizon_indexed,
        "body_geometry_ns": delta_u128(after.body_geometry_ns, before.body_geometry_ns),
        "distance_ns": delta_u128(after.distance_ns, before.distance_ns),
        "resolve_ns": delta_u128(after.resolve_ns, before.resolve_ns),
    }))
}

fn delta(after: u64, before: u64) -> u64 {
    after.saturating_sub(before)
}
fn delta_u128(after: u128, before: u128) -> u128 {
    after.saturating_sub(before)
}

fn proc_kib(key: &str) -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|text| {
            text.lines()
                .find(|line| line.starts_with(key))
                .and_then(|line| line.split_whitespace().nth(1))
                .and_then(|value| value.parse().ok())
        })
        .unwrap_or(0)
}

fn chunk_count(assembly: &Assembly) -> usize {
    assembly
        .filaments
        .values()
        .map(|filament| {
            if filament.nodes.is_empty() {
                0
            } else {
                filament
                    .nodes
                    .len()
                    .saturating_sub(1)
                    .div_ceil(CHUNK_EDGES)
                    .max(1)
            }
        })
        .sum()
}

fn scene_span(scene: &WorldSnapshot) -> Result<i64, MorphError> {
    let mut points = Vec::new();
    for instance in scene.instances.values() {
        points.push(instance.position.absolute()?);
    }
    if let Some(assembly) = &scene.assembly {
        for filament in assembly.filaments.values() {
            for node in &filament.nodes {
                points.push(node.position.absolute()?);
            }
        }
    }
    if points.is_empty() {
        return Ok(0);
    }
    let mut span = 0i64;
    for axis in 0..3 {
        let min = points.iter().map(|p| p[axis]).min().unwrap();
        let max = points.iter().map(|p| p[axis]).max().unwrap();
        span = span.max(max.saturating_sub(min));
    }
    Ok(span.max(SCALE))
}

fn align_cell(shift: i64) -> i64 {
    let cell = 2 * SCALE;
    shift.div_euclid(cell).saturating_mul(cell).max(cell)
}

fn shift_position(position: &mut Position, delta: [i64; 3]) -> Result<(), MorphError> {
    *position = position.offset(delta)?;
    Ok(())
}

fn shift_scene(scene: &mut WorldSnapshot, delta: [i64; 3]) -> Result<(), MorphError> {
    if delta == [0; 3] {
        return Ok(());
    }
    for instance in scene.instances.values_mut() {
        shift_position(&mut instance.position, delta)?;
        for parcel in &mut instance.book.parcels {
            parcel.position = Position::from_absolute(parcel.position)?
                .offset(delta)?
                .absolute()?;
        }
        for binding in &mut instance.book.bound {
            binding.parcel.position = Position::from_absolute(binding.parcel.position)?
                .offset(delta)?
                .absolute()?;
        }
    }
    for port in &mut scene.config.ports {
        shift_position(&mut port.position, delta)?;
    }
    for solid in &mut scene.solids {
        shift_position(&mut solid.min, delta)?;
        shift_position(&mut solid.max, delta)?;
    }
    if let Some(steering) = &mut scene.steering {
        steering.rigid.translation = Position::from_absolute(steering.rigid.translation)?
            .offset(delta)?
            .absolute()?;
    }
    if let Some(rules) = &mut scene.config.construction {
        for pose in &mut rules.route {
            pose.translation = Position::from_absolute(pose.translation)?
                .offset(delta)?
                .absolute()?;
        }
    }
    if let Some(habitat) = &mut scene.habitat {
        let cell = delta.map(|v| v.div_euclid(2 * SCALE));
        for agent in habitat.agents.values_mut() {
            agent.rigid.translation = Position::from_absolute(agent.rigid.translation)?
                .offset(delta)?
                .absolute()?;
            if let Some(memory) = &mut agent.travel {
                if let Some(target) = &mut memory.target {
                    shift_position(target, delta)?;
                }
                if let Some(food) = &mut memory.last_food {
                    shift_position(food, delta)?;
                }
                for visited in &mut memory.visited {
                    for axis in 0..3 {
                        visited[axis] = visited[axis].saturating_add(cell[axis]);
                    }
                }
            }
        }
        for contact in &mut habitat.contacts {
            shift_position(&mut contact.point, delta)?;
        }
    }
    if let Some(assembly) = &mut scene.assembly {
        for filament in assembly.filaments.values_mut() {
            for node in &mut filament.nodes {
                shift_position(&mut node.position, delta)?;
            }
        }
    }
    if let Some(food) = &mut scene.food {
        for site in food.sites.values_mut() {
            shift_position(&mut site.position, delta)?;
        }
    }
    for event in &mut scene.events {
        match event {
            Event::Contact { observation } => shift_position(&mut observation.point, delta)?,
            Event::Fed { mouth, .. } => shift_position(mouth, delta)?,
            _ => {}
        }
    }
    for instance in scene.instances.values_mut() {
        instance.exposure = exposure(&scene.config, instance.position)?;
    }
    Ok(())
}

fn grow_then_shift(scene: &WorldSnapshot, delta_x: i64) -> Result<WorldSnapshot, MorphError> {
    let mut world = ShipRuntime::restore(scene.clone())?;
    for _ in 0..4 {
        world.tick()?;
    }
    let mut grown = world.snapshot();
    if delta_x != 0 {
        grown = isolate_pipes(&grown, delta_x)?;
    }
    Ok(grown)
}

fn isolate_pipes(scene: &WorldSnapshot, delta_x: i64) -> Result<WorldSnapshot, MorphError> {
    let mut scene = scene.clone();
    let delta = [delta_x, 0, 0];
    let mut moved = std::collections::BTreeMap::<(u64, u64), [i64; 3]>::new();
    if let Some(assembly) = &mut scene.assembly {
        for filament in assembly.filaments.values_mut() {
            for node in &mut filament.nodes {
                shift_position(&mut node.position, delta)?;
                moved.insert((filament.id, node.id), node.position.absolute()?);
            }
        }
    }
    for instance in scene.instances.values_mut() {
        for binding in &mut instance.book.bound {
            if let Some(position) = moved.get(&(binding.filament, binding.node)) {
                binding.parcel.position = *position;
            }
        }
    }
    Ok(scene)
}

fn scene_anchor(scene: &WorldSnapshot) -> Result<Position, MorphError> {
    scene
        .instances
        .values()
        .find(|instance| instance.definition != BLOOD_SAC)
        .map(|instance| instance.position)
        .ok_or_else(|| MorphError("no organism anchor".into()))
}

fn agent_heading(scene: &WorldSnapshot) -> Result<(Position, [i64; 3]), MorphError> {
    let habitat = scene
        .habitat
        .as_ref()
        .ok_or_else(|| MorphError("habitat missing".into()))?;
    let (id, agent) = habitat
        .agents
        .iter()
        .next()
        .ok_or_else(|| MorphError("habitat agent missing".into()))?;
    let position = scene.instances[id].position;
    let heading = agent.heading;
    Ok((position, heading))
}

fn splice_chunks(
    world: &mut ShipRuntime,
    first_id: u64,
    origin: Position,
    chunks: usize,
) -> Result<(), MorphError> {
    let mut cursor = origin;
    for index in 0..chunks {
        let filament = straight_chunk(first_id + index as u64, cursor)?;
        let end = filament.nodes.last().unwrap().position;
        cursor = end.offset([0, 0, SCALE])?;
        world.splice_measurement_filament(filament)?;
    }
    Ok(())
}

fn straight_chunk(id: u64, origin: Position) -> Result<Filament, MorphError> {
    let mut nodes = Vec::new();
    for step in 0..=CHUNK_EDGES {
        nodes.push(Node {
            id: step as u64 + 1,
            parcel: id * 1000 + step as u64 + 1,
            deposited_tick: 0,
            position: origin.offset([0, 0, step as i64 * 200_000])?,
        });
    }
    Ok(Filament {
        id,
        section: Some(PipeSection {
            outer_radius: 50_000,
            inner_radius: 31_000,
            start_sealed: true,
            end_sealed: true,
        }),
        organism: MEASUREMENT_ORGANISM,
        nozzle: 1,
        created_tick: 0,
        experiment: "spatial-scale-measurement".into(),
        material: MaterialKind::Secretion,
        quantity: nodes.len() as u64,
        nodes,
    })
}
