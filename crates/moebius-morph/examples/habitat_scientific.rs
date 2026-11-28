//! HABITAT-003 full selected CPU transaction recorder; raw evidence stays external.
use moebius_morph::primordial::scientific::{
    Chamber, Model,
    experiments::{self, Scenario},
};
use std::{
    fs,
    io::{BufWriter, Write},
    path::Path,
    time::Instant,
};
#[path = "support/habitat_alloc.rs"]
mod allocation;
fn write(p: &Path, v: &impl serde::Serialize) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(p, serde_json::to_vec_pretty(v)?)?;
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a = std::env::args().skip(1).collect::<Vec<_>>();
    if a.first().is_some_and(|s| s == "--build-identity") {
        println!("{}", moebius_morph::BENCHMARK_BUILD_IDENTITY);
        return Ok(());
    }
    if a.first().is_some_and(|s| s == "replay") {
        let mut c = Chamber::load(Path::new(&a[1]))?;
        let ticks: usize = a[3].parse()?;
        if ticks > 20000 {
            return Err("tick capacity".into());
        }
        for _ in 0..ticks {
            c.step()?;
        }
        c.save(Path::new(&a[2]))?;
        return Ok(());
    }
    if !(9..=10).contains(&a.len()) {
        return Err(
            "NEW_OUT Scenario Model N DT TICKS refined|uniform|fine mono|subcycle SPECIES [COARSE_H]"
                .into(),
        );
    }
    let out = Path::new(&a[0]);
    if out.exists() {
        return Err("immutable run directory exists".into());
    }
    fs::create_dir_all(out)?;
    let scenario: Scenario = serde_json::from_str(&format!("\"{}\"", a[1]))?;
    let model: Model = serde_json::from_str(&format!("\"{}\"", a[2]))?;
    let n: usize = a[3].parse()?;
    let dt: f64 = a[4].parse()?;
    let ticks: usize = a[5].parse()?;
    let species: usize = a[8].parse()?;
    if !(4..=2048).contains(&n)
        || ticks > 20000
        || species > 4
        || !matches!(a[6].as_str(), "refined" | "uniform" | "fine")
        || !matches!(a[7].as_str(), "mono" | "subcycle")
    {
        return Err("run envelope".into());
    }
    let mut cfg = experiments::config(n, dt, model, scenario, species);
    if let Some(h) = a.get(9) {
        cfg.h = h.parse()?;
        if model == Model::AdaptiveLbm {
            cfg.sound_speed = cfg.h / cfg.dt / 3f64.sqrt();
        }
    }
    write(&out.join("configuration.json"), &cfg)?;
    write(
        &out.join("build-identity.json"),
        &serde_json::from_str::<serde_json::Value>(moebius_morph::BENCHMARK_BUILD_IDENTITY)?,
    )?;
    let mut c = experiments::chamber(cfg, scenario, a[6] == "refined")?;
    if a[6] == "fine" {
        let roots = (0..c.state().config.count()).collect::<Vec<_>>();
        c.refine(&roots)?;
    }
    c.save(&out.join("initial.json"))?;
    let mut traces = BufWriter::new(fs::File::create(out.join("traces.jsonl"))?);
    let mut profiles = BufWriter::new(fs::File::create(out.join("profiles.jsonl"))?);
    let mut completed = 0usize;
    let mut endpoint = None;
    let mut failure = None;
    let mut pending_test = false;
    if a[7] == "subcycle" {
        c.begin_subcycle()?;
        c.save(&out.join("pending.json"))?;
        let mut other = Chamber::load(&out.join("pending.json"))?;
        c.finish_subcycle()?;
        other.finish_subcycle()?;
        pending_test = c.digest()? == other.digest()?;
        if !pending_test {
            return Err("pending continuation mismatch".into());
        }
        c = Chamber::load(&out.join("initial.json"))?;
    }
    for _ in 0..ticks {
        let before = c.digest()?;
        let memory_before = allocation::memory();
        let start = Instant::now();
        let mut detail = None;
        let result = if a[7] == "subcycle" {
            c.begin_subcycle().and_then(|_| c.finish_subcycle())
        } else {
            c.step().map(|p| detail = Some(p))
        };
        let elapsed = start.elapsed().as_nanos();
        let after = allocation::memory();
        let stats = serde_json::json!({"requested_delta":after[0]-memory_before[0],"live_before":memory_before[1],"live_after":after[1],"peak":after[2],"instrumented":cfg!(feature="allocation-attribution")});
        if let Err(e) = result {
            failure = Some(
                serde_json::json!({"message":e.to_string(),"digest_before":before,"digest_after":c.digest()?,"committed_cycle":c.state().cycle,"candidate":c.rejected(),"candidate_diagnostics":c.rejected().map(|s|s.diagnostics())}),
            );
            break;
        }
        serde_json::to_writer(
            &mut profiles,
            &serde_json::json!({"transaction_ns":elapsed,"detail":detail,"allocation":stats}),
        )?;
        writeln!(profiles)?;
        let row = serde_json::json!({"cycle":c.state().cycle,"digest":c.digest()?,"diagnostics":c.state().diagnostics(),"errors":experiments::errors(c.state(),scenario)});
        serde_json::to_writer(&mut traces, &row)?;
        writeln!(traces)?;
        endpoint = Some(row);
        completed += 1;
    }
    c.save(&out.join("final.json"))?;
    let mut restored = Chamber::load(&out.join("final.json"))?;
    let restore_exact = restored.digest()? == c.digest()?;
    let continuation = if failure.is_none() {
        let x = c.step();
        let y = restored.step();
        Some(x.is_ok() == y.is_ok() && c.digest()? == restored.digest()?)
    } else {
        None
    };
    traces.flush()?;
    profiles.flush()?;
    write(
        &out.join("summary.json"),
        &serde_json::json!({"scenario":scenario,"model":model,"n":n,"dt":dt,"requested_cycles":ticks,"completed_cycles":completed,"refined":a[6],"temporal":a[7],"species":species,"failure":failure,"restore_exact":restore_exact,"continuation_exact":continuation,"pending_restoration":pending_test,"initial":Chamber::load(&out.join("initial.json"))?.state().diagnostics(),"endpoint":endpoint,"source":moebius_morph::BENCHMARK_BUILD_IDENTITY}),
    )?;
    Ok(())
}
