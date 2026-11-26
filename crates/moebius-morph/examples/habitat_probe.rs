//! Headless evidence recorder for the canonical habitat mode; never a controller.
use moebius_morph::{runtime::*, *};
use std::{
    fs,
    io::{BufWriter, Write},
    path::PathBuf,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().nth(1).as_deref() == Some("--build-identity") {
        println!("{}", moebius_morph::BENCHMARK_BUILD_IDENTITY);
        return Ok(());
    }
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let mut diagnostic_collisions_off = false;
    let mut founders = 1usize;
    let mut seed = 77u64;
    let mut abundance = None;
    let mut wall_seconds = None;
    let mut resume = None;
    let mut execution = moebius_morph::collision::ExecutionPolicy::default();
    let mut geometry_reuse = false;
    #[cfg(feature = "history-v2-prototype")]
    let mut history_root = None;
    #[cfg(feature = "collision-fixture")]
    let mut capture_requests = None;
    let mut positional = Vec::new();
    #[cfg(feature = "collision-fixture")]
    if args.iter().any(|s| s == "--capture-recovery") {
        moebius_morph::collision::fixtures::capture_recovery();
    }
    #[cfg(feature = "collision-fixture")]
    if args.iter().any(|s| s == "--capture-geometry") {
        moebius_morph::collision::fixtures::capture_geometry();
    }
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            #[cfg(feature = "collision-fixture")]
            "--capture-geometry" => {}
            "--geometry-reuse" => geometry_reuse = true,
            "--collision-workers" => {
                index += 1;
                execution.workers = args.get(index).ok_or("workers required")?.parse()?;
            }
            "--parallel-graft" => execution.graft = true,
            "--parallel-recovery" => execution.recovery = true,
            "--parallel-static" => execution.static_queries = true,
            "--frontier-limit" => {
                index += 1;
                execution.frontier_limit =
                    args.get(index).ok_or("frontier limit required")?.parse()?;
            }
            "--frontier-minimum" => {
                index += 1;
                execution.minimum_frontier = args.get(index).ok_or("minimum required")?.parse()?;
            }
            #[cfg(feature = "collision-fixture")]
            "--capture-recovery" => {}
            "--diagnostic-collisions-off" => diagnostic_collisions_off = true,
            "--founders" => {
                index += 1;
                founders = args.get(index).ok_or("--founders needs a count")?.parse()?;
            }
            "--seed" => {
                index += 1;
                seed = args.get(index).ok_or("--seed needs a value")?.parse()?;
            }
            #[cfg(feature = "collision-fixture")]
            "--capture-collisions" => {
                index += 1;
                capture_requests = Some(PathBuf::from(
                    args.get(index).ok_or("capture requests required")?,
                ));
            }
            #[cfg(feature = "history-v2-prototype")]
            "--history-v2" => {
                index += 1;
                history_root = Some(PathBuf::from(
                    args.get(index).ok_or("--history-v2 needs backing path")?,
                ));
            }
            "--resume" => {
                index += 1;
                resume = Some(PathBuf::from(
                    args.get(index).ok_or("--resume needs a snapshot")?,
                ));
            }
            "--wall-seconds" => {
                index += 1;
                wall_seconds = Some(
                    args.get(index)
                        .ok_or("--wall-seconds needs a value")?
                        .parse::<u64>()?,
                );
            }
            "--abundance" => {
                index += 1;
                abundance = Some(
                    args.get(index)
                        .ok_or("--abundance needs a value")?
                        .parse::<u64>()?,
                );
            }
            other if other.starts_with("--") => {
                return Err(format!("unknown flag {other}").into());
            }
            other => positional.push(other.to_string()),
        }
        index += 1;
    }
    let out = PathBuf::from(positional.first().ok_or(
        "usage: habitat_probe NEW_OUTPUT [ticks] [anatomy] [--seed N] [--founders N] [--abundance N] [--diagnostic-collisions-off]",
    )?);
    let ticks = positional.get(1).map_or(Ok(1200), |s| s.parse::<u64>())?;
    let spec = positional.get(2).map_or_else(
        || Ok(AnatomySpec::showcase(seed)),
        |p| load_anatomy(std::path::Path::new(p)),
    )?;
    fs::create_dir(&out)?;
    let sources = [
        (
            "collision.rs",
            include_bytes!("../src/collision.rs").as_slice(),
        ),
        ("runtime.rs", include_bytes!("../src/runtime.rs").as_slice()),
        ("habitat.rs", include_bytes!("../src/habitat.rs").as_slice()),
        (
            "filament.rs",
            include_bytes!("../src/filament.rs").as_slice(),
        ),
        ("anatomy.rs", include_bytes!("../src/anatomy.rs").as_slice()),
    ];
    let hashes = sources
        .into_iter()
        .map(|(path, bytes)| (path, blake3::hash(bytes).to_hex().to_string()))
        .collect::<std::collections::BTreeMap<_, _>>();
    fs::write(
        out.join("source-blake3.json"),
        serde_json::to_vec_pretty(&hashes)?,
    )?;
    let start = std::time::Instant::now();
    let steps = ticks.clamp(1, 16384) as usize;
    let mut w = if let Some(path) = &resume {
        restore_probe(path, &args)?
    } else if founders == 1 && abundance.is_none() {
        ShipRuntime::habitat_simulation(spec, seed, steps)?
    } else {
        ShipRuntime::habitat_colony(spec, seed, steps, founders, abundance)?
    };
    w.configure_collision_execution(execution)?;
    w.configure_geometry_reuse(geometry_reuse)?;
    #[cfg(feature = "history-v2-prototype")]
    if let Some(root) = &history_root
        && !w.state().segmented_history()
    {
        w.enable_history_v2(root, moebius_morph::history::Policy::default())?;
    }
    if diagnostic_collisions_off {
        w.diagnostic_disable_collisions()?;
    }
    #[cfg(feature = "collision-fixture")]
    if let Some(path) = capture_requests {
        moebius_morph::collision::fixtures::request(serde_json::from_slice(&fs::read(path)?)?);
    }
    let initial_tick = w.state().tick;
    let mut simulation_ns = 0u128;
    let mut tick_ns = Vec::new();
    let mut collision_ticks = Vec::new();
    let mut samples = Vec::new();
    save_probe(&w, &out.join("initial.json"))?;
    let mut trace = BufWriter::new(fs::File::create(out.join("ticks.jsonl"))?);
    let mut movements = 0u64;
    let mut meals = 0u64;
    let mut stationary_emissions = 0u64;
    let mut simultaneous = 0u64;
    let mut maximum_stock = 0;
    let mut path_um = 0f64;
    let mut journeys = std::collections::BTreeMap::<u64, (Position, f64, f64, u64, u64)>::new();
    let mut error = None;
    let mut stop = "completed";
    for _ in 0..ticks {
        if wall_seconds.is_some_and(|limit| start.elapsed().as_secs() >= limit) {
            stop = "wall_clock";
            break;
        }
        let before = w.snapshot();
        let profile_before = w.collision_profile();
        let tick_started = std::time::Instant::now();
        if let Err(e) = w.tick() {
            error = Some(e.to_string());
            stop = "error";
            break;
        }
        let elapsed_ns = tick_started.elapsed().as_nanos();
        simulation_ns += elapsed_ns;
        tick_ns.push(elapsed_ns);
        let profile_after = w.collision_profile();
        let delta = |after: u128, before: u128| {
            u64::try_from(after.saturating_sub(before)).unwrap_or(u64::MAX)
        };
        let count = |after: u64, before: u64| after.saturating_sub(before);
        collision_ticks.push(serde_json::json!({
            "tick": w.state().tick,
            "wall_ns": delta(elapsed_ns, 0),
            "resolve_ns": delta(profile_after.resolve_ns, profile_before.resolve_ns),
            "query_ns": delta(profile_after.query_ns, profile_before.query_ns),
            "distance_ns": delta(profile_after.distance_ns, profile_before.distance_ns),
            "contact_ns": delta(profile_after.contact_ns, profile_before.contact_ns),
            "pipe_geometry_ns": delta(profile_after.pipe_geometry_ns, profile_before.pipe_geometry_ns),
            "body_geometry_ns": delta(profile_after.body_geometry_ns, profile_before.body_geometry_ns),
            "queries": count(profile_after.queries, profile_before.queries),
            "distance_queries": count(profile_after.distance_queries, profile_before.distance_queries),
            "distance_within": count(profile_after.distance_within, profile_before.distance_within),
            "distance_clear": count(profile_after.distance_clear, profile_before.distance_clear),
            "interval_rejects": count(profile_after.interval_rejects, profile_before.interval_rejects),
            "mesh_interval_rejects": count(profile_after.mesh_interval_rejects, profile_before.mesh_interval_rejects),
            "linear_casts": count(profile_after.linear_casts, profile_before.linear_casts),
            "linear_cast_ns": delta(profile_after.linear_cast_ns, profile_before.linear_cast_ns),
            "linear_nodes": count(profile_after.linear_nodes,profile_before.linear_nodes),
            "linear_node_rejects": count(profile_after.linear_node_rejects,profile_before.linear_node_rejects),
            "linear_leaf_dispatches": count(profile_after.linear_leaf_dispatches,profile_before.linear_leaf_dispatches),
            "cast_cache_hits": count(profile_after.cast_cache_hits, profile_before.cast_cache_hits),
            "pipe_builds": count(profile_after.pipe_builds, profile_before.pipe_builds),
            "pipe_hits": count(profile_after.pipe_hits, profile_before.pipe_hits),
            "pipe_candidates": count(profile_after.pipe_candidates, profile_before.pipe_candidates),
            "pipe_rejects": count(profile_after.pipe_rejects, profile_before.pipe_rejects),
            "triangles_examined": count(profile_after.triangles_examined, profile_before.triangles_examined),
            "distance_bounded": count(profile_after.distance_bounded, profile_before.distance_bounded),
            "distance_exact": count(profile_after.distance_exact, profile_before.distance_exact),
            "pipe_horizon_rejects": count(profile_after.pipe_horizon_rejects, profile_before.pipe_horizon_rejects),
            "pipe_horizon_candidates": count(profile_after.pipe_horizon_candidates, profile_before.pipe_horizon_candidates),
            "pipe_horizon_indexed": profile_after.pipe_horizon_indexed,
            "pipe_horizon_rebuilds": count(profile_after.pipe_horizon_rebuilds, profile_before.pipe_horizon_rebuilds),
            "pipe_horizon_ns": delta(profile_after.pipe_horizon_ns, profile_before.pipe_horizon_ns),
            "pipe_bounds_computed": count(profile_after.pipe_bounds_computed, profile_before.pipe_bounds_computed),
        }));
        if w.state().tick.is_multiple_of(60) {
            let profile = w.collision_profile();
            samples.push(serde_json::json!({
                "tick": w.state().tick,
                "simulation_ns": simulation_ns,
                "resolve_ns": profile.resolve_ns.saturating_sub(profile_before.resolve_ns),
                "resolve_ns_total": profile.resolve_ns,
                "query_ns_total": profile.query_ns,
                "queries": profile.queries,
                "pipe_builds": profile.pipe_builds,
                "pipe_hits": profile.pipe_hits,
                "pipe_candidates": profile.pipe_candidates,
                "pipe_rejects": profile.pipe_rejects,
                "pipe_geometry_ns": profile.pipe_geometry_ns,
                "body_builds": profile.body_builds,
                "body_hits": profile.body_hits,
                "body_geometry_ns": profile.body_geometry_ns,
            }));
        }
        let s = w.state();
        let mut rows = Vec::new();
        for (&id, a) in &s.habitat.as_ref().unwrap().agents {
            let body = &s.instances[&id];
            maximum_stock = maximum_stock.max(body.book.reservoir);
            let journey = journeys.entry(id).or_insert((
                before
                    .instances
                    .get(&id)
                    .map_or(body.position, |b| b.position),
                0.,
                0.,
                0,
                0,
            ));
            let displacement = body.position.relative(journey.0)?;
            journey.2 = journey.2.max(
                displacement
                    .iter()
                    .map(|v| (*v as f64).powi(2))
                    .sum::<f64>()
                    .sqrt()
                    / 1e6,
            );
            journey.4 += u64::from(s.habitat.as_ref().unwrap().contacts.iter().any(|c| {
                c.organism == id && c.response == moebius_morph::collision::ContactResponse::Blocked
            }));
            if let Some(old) = before.instances.get(&id) {
                let d = body.position.relative(old.position)?;
                let moved = d != [0; 3];
                movements += u64::from(moved);
                journey.1 += d.iter().map(|v| (*v as f64).powi(2)).sum::<f64>().sqrt() / 1e6;
                journey.3 += u64::from(moved);
                path_um += d.iter().map(|x| (*x as f64).powi(2)).sum::<f64>().sqrt();
                let emitted=s.events.iter().filter(|e|matches!(e,Event::Emitted{at,organism,..}if *at==before.tick&&*organism==id)).count();
                let fed = s
                    .events
                    .iter()
                    .filter_map(|e| {
                        if let Event::Fed {
                            at,
                            organism,
                            quantity,
                            ..
                        } = e
                        {
                            (*at == before.tick && *organism == id).then_some(*quantity)
                        } else {
                            None
                        }
                    })
                    .sum::<u64>();
                meals += u64::from(fed > 0);
                stationary_emissions += if moved { 0 } else { emitted as u64 };
                simultaneous += if fed > 0 { emitted as u64 } else { 0 };
                rows.push(serde_json::json!({"id":id,"position":body.position,"heading":a.heading,"stock":body.book.reservoir,"capacity":s.config.definitions[&body.definition].food_capacity(),"intake":fed,"emitted":emitted,"feeding":a.eating,"parent":a.parent,"book_gap":body.book.residual()?}));
            }
        }
        writeln!(
            trace,
            "{}",
            serde_json::json!({"tick":s.tick,"agents":rows,"contacts":s.habitat.as_ref().unwrap().contacts,"substrate_sacs":s.habitat.as_ref().unwrap().sacs,"nodes":s.assembly.as_ref().unwrap().nodes(),"junctions":s.assembly.as_ref().unwrap().junctions.len(),"material_gap":s.condensate_gap()})
        )?;
        if s.tick.is_multiple_of(60)
            && let Err(e) = w.validate()
        {
            error = Some(e.to_string());
            stop = "error";
            break;
        }
    }
    trace.flush()?;
    let final_state = w.snapshot();
    save_probe(&w, &out.join("final.json"))?;
    let vm_hwm_kb_before_replay = proc_kib("VmHWM:");
    #[cfg(feature = "collision-fixture")]
    let _captured = moebius_morph::collision::fixtures::write_captures(&out.join("fixtures"))?;
    #[cfg(feature = "query-trace")]
    let forward_query_trace = moebius_morph::collision::take_query_trace();
    #[cfg(feature = "collision-fixture")]
    fs::write(
        out.join("recovery.json"),
        serde_json::to_vec_pretty(&moebius_morph::collision::fixtures::take_recovery())?,
    )?;
    let mut replay = restore_probe(&out.join("initial.json"), &args)?;
    #[cfg(feature = "collision-fixture")]
    fs::write(
        out.join("geometry-construction.json"),
        serde_json::to_vec_pretty(&moebius_morph::collision::fixtures::take_geometry())?,
    )?;
    replay.configure_collision_execution(execution)?;
    replay.configure_geometry_reuse(geometry_reuse)?;
    replay.advance(final_state.tick - initial_tick)?;
    let vm_hwm_kb_after_replay = proc_kib("VmHWM:");
    let vm_rss_kb = proc_kib("VmRSS:");
    let writhe_slack =
        moebius_morph::collision::presentation_slack(&final_state.config.definitions[&1])?;
    let profile = w.collision_profile();
    let tick_count = tick_ns.len().max(1) as f64;
    let claims = if diagnostic_collisions_off {
        "DIAGNOSTIC ONLY. Habitat collision mode was turned off for this process. Sac offsets, birth placement, spawn checks, and contact admission differ from the product simulation. This world is not an acceptable final simulation. Rendering cost is absent. An exterior collision primitive was not substituted for the hollow pipe."
    } else {
        "Headless throughput only. Authored local policy, finite stock and graph contacts. Collision mode stayed on. The writhe is not propulsion and is not learned. Rendering cost is absent from this run. Pipe candidates still use the hollow trimesh. An exterior primitive was not used."
    };
    let replay_equal = replay.snapshot() == final_state;
    let status = if error.is_some() || !replay_equal || final_state.condensate_gap() != 0 {
        "FAILED"
    } else if stop != "completed" {
        "PARTIAL"
    } else if diagnostic_collisions_off {
        "DIAGNOSTIC"
    } else {
        "PASS"
    };
    let tick_ms = time_summary(&tick_ns);
    let collision = serde_json::json!({
        "resolve_calls": profile.resolve_calls,
        "resolve_ns": profile.resolve_ns,
        "mean_resolve_ms": (profile.resolve_ns as f64 / tick_count) / 1e6,
        "queries": profile.queries,
        "query_ns": profile.query_ns,
        "interval_rejects": profile.interval_rejects,
        "distance_queries": profile.distance_queries,
        "distance_ns": profile.distance_ns,
        "contact_ns": profile.contact_ns,
        "cast_cache_hits": profile.cast_cache_hits,
        "linear_casts": profile.linear_casts,
        "linear_cast_ns": profile.linear_cast_ns,
        "linear_nodes": profile.linear_nodes,
        "linear_node_rejects": profile.linear_node_rejects,
        "linear_leaf_dispatches": profile.linear_leaf_dispatches,
        "mesh_interval_rejects": profile.mesh_interval_rejects,
        "body_builds": profile.body_builds,
        "body_hits": profile.body_hits,
        "body_geometry_ns": profile.body_geometry_ns,
        "pipe_builds": profile.pipe_builds,
        "pipe_hits": profile.pipe_hits,
        "pipe_candidates": profile.pipe_candidates,
        "pipe_rejects": profile.pipe_rejects,
        "pipe_geometry_ns": profile.pipe_geometry_ns,
        "triangles_examined": profile.triangles_examined,
        "distance_bounded": profile.distance_bounded,
        "distance_exact": profile.distance_exact,
        "pipe_horizon_rejects": profile.pipe_horizon_rejects,
        "pipe_horizon_candidates": profile.pipe_horizon_candidates,
        "pipe_horizon_indexed": profile.pipe_horizon_indexed,
        "pipe_horizon_rebuilds": profile.pipe_horizon_rebuilds,
        "pipe_horizon_ns": profile.pipe_horizon_ns,
        "pipe_bounds_computed": profile.pipe_bounds_computed,
    });
    let mut report = serde_json::json!({
        "status": status,
        "error": error,
        "diagnostic_collisions_off": diagnostic_collisions_off,
        "collisions_enabled": !diagnostic_collisions_off,
        "ticks": final_state.tick,
        "digest": final_state.digest()?,
        "replay_equal": replay_equal,
        "founders": founders,
        "seed": seed,
        "abundance": abundance,
        "births": final_state.total_births(),
        "history_total_events": final_state.total_events(),
        "history_resident_events": final_state.events.len(),
        "history_v2": final_state.segmented_history(),
        "agent_count": final_state.habitat.as_ref().unwrap().agents.len(),
        "sac_count": final_state.instances.values().filter(|i| i.definition == BLOOD_SAC).count(),
        "rooted_sacs": final_state.habitat.as_ref().unwrap().sacs.len(),
        "nodes": final_state.assembly.as_ref().unwrap().nodes(),
        "junctions": final_state.assembly.as_ref().unwrap().junctions.len(),
        "movements": movements,
        "path_metres": path_um / 1e6,
        "meal_ticks": meals,
        "max_stock": maximum_stock,
        "stationary_emissions": stationary_emissions,
        "feeding_emissions": simultaneous,
        "material_gap": final_state.condensate_gap(),
        "elapsed_seconds": start.elapsed().as_secs_f64(),
        "presentation_slack_metres": writhe_slack,
        "simulation_ns": simulation_ns,
        "mean_tick_ms": (simulation_ns as f64 / tick_count) / 1e6,
        "tick_ms": tick_ms,
        "percentile": "nearest-rank, index = ceil((n-1)*p)",
        "vm_hwm_kb_before_replay": vm_hwm_kb_before_replay,
        "vm_hwm_kb_after_replay": vm_hwm_kb_after_replay,
        "vm_rss_kb": vm_rss_kb,
        "collision": collision,
        "samples": samples,
        "claims": claims,
    });
    let experiment = serde_json::json!({
        "stop": stop,
        "requested_ticks": ticks,
        "initial_tick": initial_tick,
        "ticks_advanced": final_state.tick - initial_tick,
        "resume": resume,
        "wall_seconds": wall_seconds,
        "simulated_seconds": final_state.tick as f64 * TICK_NANOS as f64 / 1e9,
        "deaths": null,
        "mortality": "UNIMPLEMENTED: no death or metabolic consumption rule; survival duration is not measurable",
        "generation_depth": final_state.habitat.as_ref().unwrap().agents.keys().map(|id| {
            let mut depth = 0;
            let mut current = *id;
            while let Some(parent) = final_state.habitat.as_ref().unwrap().agents[&current].parent {
                depth += 1;
                current = parent;
            }
            depth
        }).max().unwrap_or(0),
    });
    report
        .as_object_mut()
        .unwrap()
        .extend(experiment.as_object().unwrap().clone());
    report["journeys"] = serde_json::json!(journeys.iter().map(|(id,(origin,path,max_net,moving,blocked))|serde_json::json!({"id":id,"origin":origin,"path_m":path,"max_net_m":max_net,"moving_ticks":moving,"blocked_ticks":blocked,"remembered_cells":final_state.habitat.as_ref().unwrap().agents[id].travel.as_ref().map_or(0,|m|m.visited.len())})).collect::<Vec<_>>());
    fs::write(
        out.join("tick-times-ns.json"),
        serde_json::to_vec(&tick_ns)?,
    )?;
    fs::write(
        out.join("collision-ticks.json"),
        serde_json::to_vec(&collision_ticks)?,
    )?;
    #[cfg(feature = "query-trace")]
    fs::write(
        out.join("query-trace.json"),
        serde_json::to_vec_pretty(&forward_query_trace)?,
    )?;
    #[cfg(feature = "query-trace")]
    {
        report["query_trace_scope"] = serde_json::json!("forward_only; replay excluded");
    }
    report["execution_policy"] = serde_json::to_value(execution)?;
    report["execution_profile"] = serde_json::to_value(w.collision_execution_profile())?;
    report["geometry_reuse"] = serde_json::to_value(w.geometry_reuse_profile())?;
    fs::write(out.join("report.json"), serde_json::to_vec_pretty(&report)?)?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn time_summary(samples: &[u128]) -> serde_json::Value {
    let sum = samples.iter().copied().sum::<u128>();
    let n = samples.len().max(1) as f64;
    serde_json::json!({
        "count": samples.len(),
        "mean_ms": (sum as f64 / n) / 1e6,
        "p50_ms": nearest_rank(samples, 0.50) as f64 / 1e6,
        "p95_ms": nearest_rank(samples, 0.95) as f64 / 1e6,
        "p99_ms": nearest_rank(samples, 0.99) as f64 / 1e6,
        "max_ms": nearest_rank(samples, 1.0) as f64 / 1e6,
    })
}

fn nearest_rank(samples: &[u128], fraction: f64) -> u128 {
    if samples.is_empty() {
        return 0;
    }
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let index = ((sorted.len() - 1) as f64 * fraction).ceil() as usize;
    sorted[index.min(sorted.len() - 1)]
}

fn proc_kib(prefix: &str) -> Option<u64> {
    let text = fs::read_to_string("/proc/self/status").ok()?;
    text.lines().find_map(|line| {
        let rest = line.strip_prefix(prefix)?;
        rest.split_whitespace().next()?.parse().ok()
    })
}

fn save_probe(runtime: &ShipRuntime, path: &std::path::Path) -> Result<(), MorphError> {
    #[cfg(feature = "history-v2-prototype")]
    if runtime.state().segmented_history() {
        return runtime.save_checkpoint_v2(path);
    }
    runtime.snapshot().save(path)
}
fn restore_probe(
    path: &std::path::Path,
    args: &[String],
) -> Result<ShipRuntime, Box<dyn std::error::Error>> {
    #[cfg(feature = "history-v2-prototype")]
    if let Some(index) = args.iter().position(|s| s == "--history-v2") {
        let snapshot: WorldSnapshot = serde_json::from_slice(&fs::read(path)?)?;
        if snapshot.segmented_history() {
            return Ok(ShipRuntime::restore_with_history(
                snapshot,
                std::path::Path::new(args.get(index + 1).ok_or("missing history backing")?),
            )?);
        }
    }
    let _ = args;
    Ok(ShipRuntime::restore(WorldSnapshot::load(path)?)?)
}
