//! Independent HABITAT-002 lane recorder. Timings only measure the selected step.
use moebius_morph::primordial::{
    Chamber, Config, ScalarBackend, experiments,
    scalable::{Lane, Workspace},
};
use std::{fs, path::Path, time::Instant};
#[path = "support/habitat_alloc.rs"]
mod allocation;
fn write(p: &Path, v: &impl serde::Serialize) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(p, serde_json::to_vec_pretty(v)?)?;
    Ok(())
}
fn rss() -> u64 {
    fs::read_to_string("/proc/self/status")
        .unwrap_or_default()
        .lines()
        .find(|l| l.starts_with("VmHWM:"))
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a = std::env::args().skip(1).collect::<Vec<_>>();
    if a.first().is_some_and(|x| x == "--build-identity") {
        println!("{}", moebius_morph::BENCHMARK_BUILD_IDENTITY);
        return Ok(());
    }
    if a.first().is_some_and(|x| x == "replay") {
        let lane: Lane = serde_json::from_str(&format!("\"{}\"", a.get(4).ok_or("lane")?))?;
        let workers: usize = a.get(5).map_or(Ok(1), |v| v.parse())?;
        let mut c = Chamber::load_candidate(Path::new(a.get(1).ok_or("initial")?), lane, workers)?;
        let ticks: usize = a.get(3).ok_or("ticks")?.parse()?;
        if ticks > 16384 {
            return Err("tick budget".into());
        }
        let mut w = Workspace::with_workers(workers);
        for _ in 0..ticks {
            c.step_candidate(lane, &mut w)?;
        }
        c.save_candidate(Path::new(a.get(2).ok_or("final")?), lane, workers)?;
        return Ok(());
    }
    if !(11..=13).contains(&a.len()) {
        return Err(
            "NEW_OUT LANE N EDGE OCCUPANCY TICKS LENGTH DIFFUSION REGIONS BODIES WORKERS(1..8) [SPECIES1..4] [SPEED]"
                .into(),
        );
    }
    let out = Path::new(&a[0]);
    let lane: Lane = serde_json::from_str(&format!(
        "\"{}\"",
        if a[1] == "ReferenceSparse" {
            "Reference"
        } else {
            &a[1]
        }
    ))?;
    let n = a[2].parse()?;
    let edge = a[3].parse()?;
    let fraction: f64 = a[4].parse()?;
    let ticks: usize = a[5].parse()?;
    let length = a[6].parse()?;
    let diffusion = a[7].parse()?;
    let regions: usize = a[8].parse()?;
    let bodies: usize = a[9].parse()?;
    let workers: usize = a[10].parse()?;
    let species: usize = a.get(11).map_or(Ok(1), |v| v.parse())?;
    let speed: f64 = a.get(12).map_or(Ok(0.002), |v| v.parse())?;
    if ticks == 0
        || ticks > 16384
        || !(1..=8).contains(&workers)
        || regions > 16
        || !(-1. ..=1.).contains(&fraction)
    {
        return Err("bounded recorder arguments".into());
    }
    let mut config = Config {
        cells: n,
        length_m: length,
        gravity_m_s2: if fraction < 0. {
            [0., -0.0025, 0.]
        } else {
            [0.; 3]
        },
        ..Default::default()
    };
    if a[1] == "ReferenceSparse" {
        config.scalar_backend = ScalarBackend::SparseActiveFaces;
    }
    let mut c = if fraction < 0. {
        experiments::demo(config.clone())?
    } else {
        moebius_morph::primordial::scalable::cloud(
            config.clone(),
            fraction,
            regions,
            bodies,
            diffusion,
            species,
            speed,
        )?
    };
    if lane == Lane::Blocks {
        c.use_blocks(edge)?;
    }
    fs::create_dir(out)?;
    c.save_candidate(&out.join("initial.json"), lane, workers)?;
    write(&out.join("config.json"), &c.state().config)?;
    let mut rows = Vec::new();
    let mut w = Workspace::with_workers(workers);
    let wall = Instant::now();
    for t in 0..ticks {
        let m = allocation::memory();
        let profile = c.step_candidate(lane, &mut w)?;
        let after = allocation::memory();
        let attribution = if [0, 1, 3, 15, 63, 127].contains(&t) {
            Some(c.transport_attribution()?)
        } else {
            None
        };
        rows.push(serde_json::json!({"tick":c.state().tick,"physical_digest":c.physical_digest()?,"profile":profile,"requested_bytes":after[0]-m[0],"live_bytes":after[1],"peak_live_bytes":after[2],"peak_rss_kib":rss(),"diagnostics":c.state().diagnostics(),"activity":w.activity,"scalar_nonzero":c.state().fields.iter().map(|f|f.mass.nonzero().len()).sum::<usize>(),"scalar_slots":c.state().fields.iter().map(|f|f.mass.stored()).sum::<usize>(),"attribution":attribution}));
    }
    c.save_candidate(&out.join("final.json"), lane, workers)?;
    write(&out.join("rows.json"), &rows)?;
    let mut replay = Chamber::load_candidate(&out.join("initial.json"), lane, workers)?;
    let mut wr = Workspace::with_workers(workers);
    for row in &rows {
        replay.step_candidate(lane, &mut wr)?;
        assert_eq!(
            replay.physical_digest()?,
            row["physical_digest"].as_str().unwrap()
        );
    }
    assert!(c.physical_eq(&replay));
    write(
        &out.join("summary.json"),
        &serde_json::json!({"lane":lane,"edge":edge,"requested_fraction":fraction,"regions":regions,"workers":workers,"ticks":ticks,"headless_wall_seconds_including_observers_and_replay":wall.elapsed().as_secs_f64(),"final_physical_digest":c.physical_digest()?,"every_tick_replay_exact":true,"build_identity":serde_json::from_str::<serde_json::Value>(moebius_morph::BENCHMARK_BUILD_IDENTITY)?,"scope":"selected candidate full Chamber.step timing; attribution is separate intrusive scalar-only diagnostic. GPU kernel is a separate lane, not this environment owner."}),
    )?;
    Ok(())
}
