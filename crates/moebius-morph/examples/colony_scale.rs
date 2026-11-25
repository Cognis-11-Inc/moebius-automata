//! Founding-population scale record. Collisions stay on. The habitat budget
//! remains 16. Larger targets are refused and recorded, not simulated.
use moebius_morph::{runtime::*, *};
use std::{
    fs,
    path::PathBuf,
    time::{Duration, Instant},
};

fn nearest_rank_ms(samples: &[u128], fraction: f64) -> f64 {
    if samples.is_empty() {
        return 0.;
    }
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let index = ((sorted.len() - 1) as f64 * fraction).ceil() as usize;
    sorted[index.min(sorted.len() - 1)] as f64 / 1e6
}

fn kib(prefix: &str) -> Option<u64> {
    let text = fs::read_to_string("/proc/self/status").ok()?;
    text.lines().find_map(|line| {
        let rest = line.strip_prefix(prefix)?;
        rest.split_whitespace().next()?.parse().ok()
    })
}

fn refuse(founders: usize, seed: u64) -> serde_json::Value {
    let error = ShipRuntime::habitat_colony(AnatomySpec::showcase(seed), seed, 8, founders, None)
        .err()
        .map(|e| e.to_string());
    serde_json::json!({
        "status": "REFUSED",
        "founders": founders,
        "seed": seed,
        "error": error,
        "note": "The habitat rule ceiling is 16 Arachnitechs. This target was not simulated.",
    })
}

fn run(founders: usize, seed: u64, ticks: u64, wall: Duration) -> serde_json::Value {
    let opened = Instant::now();
    let mut world = match ShipRuntime::habitat_colony(
        AnatomySpec::showcase(seed),
        seed,
        ticks.min(16384) as usize,
        founders,
        None,
    ) {
        Ok(world) => world,
        Err(error) => {
            return serde_json::json!({
                "status": "REFUSED",
                "founders": founders,
                "seed": seed,
                "error": error.to_string(),
            });
        }
    };
    let mut tick_ns = Vec::new();
    let mut series = Vec::new();
    let mut stop = "completed";
    let mut error = None;
    for _ in 0..ticks {
        if opened.elapsed() > wall {
            stop = "wall_clock";
            break;
        }
        let started = Instant::now();
        if let Err(failure) = world.tick() {
            error = Some(failure.to_string());
            stop = "error";
            break;
        }
        let elapsed = started.elapsed().as_nanos();
        tick_ns.push(elapsed);
        if elapsed > 2_000_000_000 {
            stop = "tick_over_2s";
            break;
        }
        let state = world.state();
        if state.tick.is_multiple_of(30) {
            series.push(serde_json::json!({
                "tick": state.tick,
                "agents": state.habitat.as_ref().unwrap().agents.len(),
                "sacs": state.instances.values().filter(|i| i.definition == BLOOD_SAC).count(),
                "nodes": state.assembly.as_ref().map(|a| a.nodes()).unwrap_or(0),
                "condensate": state.condensate.as_ref().map(|c| c.biomass).unwrap_or(0),
                "stored": state.instances.values().filter(|i| i.definition != BLOOD_SAC).map(|i| i.book.reservoir).sum::<u64>(),
            }));
        }
    }
    let state = world.snapshot();
    let births = state
        .events
        .iter()
        .filter(|event| matches!(event, Event::Reproduced { .. }))
        .count();
    let meals = state
        .events
        .iter()
        .filter(|event| matches!(event, Event::Fed { .. }))
        .count();
    let profile = world.collision_profile();
    let mean = if tick_ns.is_empty() {
        0.
    } else {
        tick_ns.iter().sum::<u128>() as f64 / tick_ns.len() as f64 / 1e6
    };
    serde_json::json!({
        "status": if error.is_some() { "FAILED" } else { "MEASURED" },
        "stop": stop,
        "error": error,
        "founders": founders,
        "seed": seed,
        "requested_ticks": ticks,
        "ticks": state.tick,
        "digest": state.digest().ok(),
        "agents": state.habitat.as_ref().map(|h| h.agents.len()).unwrap_or(0),
        "peak_agents": series.iter().filter_map(|row| row.get("agents").and_then(|v| v.as_u64())).max().unwrap_or(state.habitat.as_ref().map(|h| h.agents.len() as u64).unwrap_or(0)),
        "births": births,
        "events": state.events.len(),
        "deaths": 0,
        "deaths_note": "No Arachnitech mortality rule is implemented. The death count stays 0.",
        "meals": meals,
        "sacs": state.instances.values().filter(|i| i.definition == BLOOD_SAC).count(),
        "nodes": state.assembly.as_ref().map(|a| a.nodes()).unwrap_or(0),
        "condensate": state.condensate.as_ref().map(|c| c.biomass).unwrap_or(0),
        "material_gap": state.condensate_gap(),
        "mean_tick_ms": mean,
        "p95_tick_ms": nearest_rank_ms(&tick_ns, 0.95),
        "p99_tick_ms": nearest_rank_ms(&tick_ns, 0.99),
        "max_tick_ms": tick_ns.iter().copied().max().unwrap_or(0) as f64 / 1e6,
        "queries": profile.queries,
        "pipe_builds": profile.pipe_builds,
        "pipe_hits": profile.pipe_hits,
        "pipe_rejects": profile.pipe_rejects,
        "resolve_ns": profile.resolve_ns,
        "elapsed_s": opened.elapsed().as_secs_f64(),
        "vm_hwm_kb": kib("VmHWM:"),
        "vm_rss_kb": kib("VmRSS:"),
        "series": series,
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "/tmp/moebius-colony-scale-20261125".into()),
    );
    fs::create_dir_all(&out)?;
    let mut rows = Vec::new();
    let plan = [
        (1usize, 77u64, 420u64),
        (1, 11, 180),
        (1, 23, 180),
        (2, 77, 180),
        (4, 77, 120),
        (8, 77, 90),
        (16, 77, 60),
    ];
    for (founders, seed, ticks) in plan {
        eprintln!("colony founders={founders} seed={seed} ticks={ticks}");
        let row = run(founders, seed, ticks, Duration::from_secs(90));
        eprintln!(
            "  stop={} ticks={} mean_ms={} agents={} births={} nodes={}",
            row["stop"],
            row["ticks"],
            row["mean_tick_ms"],
            row["agents"],
            row["births"],
            row["nodes"]
        );
        let mean = row["mean_tick_ms"].as_f64().unwrap_or(0.);
        rows.push(row);
        if founders >= 4 && mean > 400. {
            rows.push(serde_json::json!({
                "status": "SKIPPED_LARGER",
                "after_founders": founders,
                "mean_tick_ms": mean,
                "note": "Larger founding groups were not started. The previous measured mean tick was above 400 ms.",
            }));
            break;
        }
    }
    for founders in [30usize, 100, 300] {
        rows.push(refuse(founders, 77));
    }
    fs::write(out.join("summary.json"), serde_json::to_vec_pretty(&rows)?)?;
    println!("{}", out.join("summary.json").display());
    Ok(())
}
