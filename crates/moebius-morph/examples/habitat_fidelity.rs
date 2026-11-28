//! HABITAT-004 strict recipe recorder, one selected scientific owner.
use moebius_morph::primordial::scientific::{
    Chamber,
    experiments::fidelity::{self, Spec},
    topology::Workspace,
};
use std::{
    fs,
    io::{BufWriter, Write},
    path::Path,
    time::Instant,
};
#[path = "support/habitat_alloc.rs"]
mod allocation;
fn write(p: &Path, x: &impl serde::Serialize) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(p, serde_json::to_vec_pretty(x)?)?;
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a = std::env::args().skip(1).collect::<Vec<_>>();
    if a.len() == 1 && a[0] == "--build-identity" {
        println!("{}", moebius_morph::BENCHMARK_BUILD_IDENTITY);
        return Ok(());
    }
    if a.len() == 5 && a[0] == "replay" {
        let mut c = Chamber::load(Path::new(&a[1]))?;
        let cycles: usize = a[3].parse()?;
        if cycles > 20000 || !matches!(a[4].as_str(), "mono" | "cached" | "subcycle") {
            return Err("replay envelope".into());
        }
        let mut w = Workspace::default();
        for _ in 0..cycles {
            match a[4].as_str() {
                "subcycle" => {
                    c.begin_subcycle()?;
                    c.finish_subcycle()?;
                }
                "cached" => {
                    c.step_cached(&mut w)?;
                }
                _ => {
                    c.step()?;
                }
            }
        }
        c.save(Path::new(&a[2]))?;
        return Ok(());
    }
    if a.len() != 2 {
        return Err(
            "NEW_OUT SPEC_JSON; or replay INITIAL NEW_FINAL CYCLES mono|cached|subcycle".into(),
        );
    }
    let spec: Spec = serde_json::from_slice(&fs::read(&a[1])?)?;
    spec.validate()?;
    let out = Path::new(&a[0]);
    fs::create_dir(out)?;
    write(&out.join("spec.json"), &spec)?;
    write(
        &out.join("build-identity.json"),
        &serde_json::from_str::<serde_json::Value>(moebius_morph::BENCHMARK_BUILD_IDENTITY)?,
    )?;
    let mut c = fidelity::chamber(&spec)?;
    let initial_metrics = fidelity::metrics(&spec, c.state());
    c.save(&out.join("initial.json"))?;
    let mut w = Workspace::default();
    let mut rows = BufWriter::new(fs::File::create(out.join("ticks.jsonl"))?);
    let mut failure = None;
    let mut completed = 0;
    let mut last = None;
    for _ in 0..spec.cycles {
        let before = c.digest()?;
        let mem = allocation::memory();
        let t = Instant::now();
        let mut profile = None;
        let result = if spec.subcycle {
            c.begin_subcycle().and_then(|_| c.finish_subcycle())
        } else if spec.cache {
            c.step_cached(&mut w).map(|p| profile = Some(p))
        } else {
            c.step().map(|p| profile = Some(p))
        };
        let ns = t.elapsed().as_nanos();
        let m = allocation::memory();
        if let Err(e) = result {
            failure = Some(
                serde_json::json!({"message":e.to_string(),"committed_digest_before":before,"committed_digest_after":c.digest()?,"candidate":c.rejected(),"candidate_diagnostics":c.rejected().map(|s|s.diagnostics())}),
            );
            break;
        }
        let row = serde_json::json!({"cycle":c.state().cycle,"digest":c.digest()?,"transaction_ns":ns,"profile":profile,"metrics":fidelity::metrics(&spec,c.state()),"diagnostics":c.state().diagnostics(),"allocation":{"instrumented":cfg!(feature="allocation-attribution"),"requested":m[0]-mem[0],"live_before":mem[1],"live_after":m[1],"peak":m[2]}});
        serde_json::to_writer(&mut rows, &row)?;
        writeln!(rows)?;
        completed += 1;
        last = Some(row);
    }
    rows.flush()?;
    c.save(&out.join("final.json"))?;
    let mut restored = Chamber::load(&out.join("final.json"))?;
    let restore_exact = restored.digest()? == c.digest()?;
    let continuation = if failure.is_none() {
        let mut rw = Workspace::default();
        let x = if spec.subcycle {
            c.begin_subcycle().and_then(|_| c.finish_subcycle())
        } else {
            c.step_cached(&mut w).map(|_| ())
        };
        let y = if spec.subcycle {
            restored
                .begin_subcycle()
                .and_then(|_| restored.finish_subcycle())
        } else {
            restored.step_cached(&mut rw).map(|_| ())
        };
        Some(x.is_ok() && y.is_ok() && c.digest()? == restored.digest()?)
    } else {
        None
    };
    write(
        &out.join("summary.json"),
        &serde_json::json!({"completed":completed,"requested":spec.cycles,"failure":failure,"initial_metrics":initial_metrics,"endpoint":last,"restore_exact":restore_exact,"continuation_exact":continuation,"cache_rebuilds":w.rebuilds,"cache_hits":w.hits,"cache_estimated_bytes":w.bytes_estimate()}),
    )?;
    if spec.subcycle {
        let initial = Chamber::load(&out.join("initial.json"))?;
        write(
            &out.join("temporal_attribution.json"),
            &fidelity::temporal_attribution(&initial)?,
        )?;
    }
    Ok(())
}
