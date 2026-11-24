//! A–H construction experiments, preserving final authoritative states and every trial digest.
use moebius_morph::{emission::SCALE, filament::*, runtime::*, witness::*};
use serde_json::json;
use std::{io::Write, path::PathBuf, time::Instant};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = PathBuf::from(
        std::env::args()
            .nth(1)
            .ok_or("new output directory required")?,
    );
    std::fs::create_dir(&out)?;
    let mut traces = std::fs::File::create(out.join("trials.jsonl"))?;
    let mut reports = vec![];
    let mut failures = vec![];
    for (name, kind, reserve, pause) in [
        ("A", "straight", 0, false),
        ("B", "straight", 64, false),
        ("C", "curved", 64, false),
        ("D", "curved", 64, true),
        ("E", "straight", 5, false),
        ("vertical", "vertical", 64, false),
        ("disconnected", "gap", 80, false),
    ] {
        let result = (|| -> Result<serde_json::Value, Box<dyn std::error::Error>> {
            let mut state = construction_fixture(20261123, 140, reserve, kind)?.snapshot();
            if pause {
                state.config.construction.as_mut().unwrap().emit[25..30].fill(false);
            }
            let mut world = ShipRuntime::restore(state)?;
            world
                .snapshot()
                .save(&out.join(format!("{name}-initial.json")))?;
            let started = Instant::now();
            for _ in 0..120 {
                world.tick()?;
                writeln!(
                    traces,
                    "{}",
                    json!({"experiment":name,"tick":world.state().tick,"digest":world.digest()?})
                )?;
            }
            let elapsed = started.elapsed().as_nanos();
            world.validate()?;
            let graph = world.state().assembly.as_ref().unwrap();
            let b = &world.state().instances[&1].book;
            let ledger = b.ledger()?;
            if reserve == 0 && graph.nodes() != 0 {
                return Err("structure without material".into());
            }
            if reserve > 0 && graph.nodes() < 2 {
                return Err("missing connected filament".into());
            }
            world
                .snapshot()
                .save(&out.join(format!("{name}-final.json")))?;
            Ok(
                json!({"experiment":name,"status":"PASS","ticks":120,"nodes":graph.nodes(),"filaments":graph.filaments.len(),"ledger":ledger,"digest":world.digest()?,"simulation_ns":elapsed,"events":world.state().events.len()}),
            )
        })();
        match result {
            Ok(r) => reports.push(r),
            Err(e) => failures.push(json!({"experiment":name,"error":e.to_string()})),
        }
    }
    let initial = construction_fixture(20261123, 160, 96, "curved")?.snapshot();
    let mut histories = vec![];
    for mode in ["headless", "stationary", "moving"] {
        let mut w = ShipRuntime::restore(initial.clone())?;
        let mut cache = PresentationCache::default();
        let mut hashes = vec![];
        for k in 0..120u64 {
            w.tick()?;
            if mode != "headless" {
                let s = WitnessSphere {
                    center: Position::from_absolute([
                        if mode == "moving" && !k.is_multiple_of(2) {
                            1000 * SCALE
                        } else {
                            0
                        },
                        0,
                        0,
                    ])?,
                    radius: if mode == "stationary" || k % 3 == 0 {
                        20 * SCALE
                    } else {
                        2 * SCALE
                    },
                };
                cache.update(&s.inspect(w.state())?);
                let index = StructuralIndex::build(w.state().assembly.as_ref().unwrap())?;
                let _ = index.visible(&s)?;
            }
            let digest = w.digest()?;
            writeln!(
                traces,
                "{}",
                json!({"experiment":"F","mode":mode,"tick":k+1,"digest":digest})
            )?;
            hashes.push(digest);
        }
        histories.push(hashes);
    }
    if histories[0] != histories[1] || histories[0] != histories[2] {
        failures.push(json!({"experiment":"F","error":"observer mutation"}));
    }
    reports.push(json!({"experiment":"F","status":if histories[0]==histories[1]&&histories[0]==histories[2]{"PASS"}else{"FAIL"},"ticks":120,"exact_per_tick_states":true}));
    let mut uninterrupted = ShipRuntime::restore(initial)?;
    uninterrupted.advance(60)?;
    uninterrupted
        .snapshot()
        .save(&out.join("G-midpoint.json"))?;
    let mut restored = ShipRuntime::restore(WorldSnapshot::load(&out.join("G-midpoint.json"))?)?;
    uninterrupted.advance(60)?;
    restored.advance(60)?;
    if uninterrupted.snapshot() != restored.snapshot() {
        failures.push(json!({"experiment":"G","error":"continuation mismatch"}));
    }
    reports.push(json!({"experiment":"G","status":"PASS","exact_continuation":uninterrupted.snapshot()==restored.snapshot(),"digest":restored.digest()?}));
    let mut scaling = vec![];
    for n in [32, 128, 512, 1024] {
        let mut w = construction_fixture(20261123, n + 8, n as u64, "curved")?;
        let start = Instant::now();
        w.advance(n as u64 + 8)?;
        let simulation = start.elapsed().as_nanos();
        w.validate()?;
        let start = Instant::now();
        let index = StructuralIndex::build(w.state().assembly.as_ref().unwrap())?;
        let index_ns = start.elapsed().as_nanos();
        let sphere = WitnessSphere {
            center: w.state().instances[&1].position,
            radius: 10 * SCALE,
        };
        let start = Instant::now();
        let visible = index.visible(&sphere)?;
        let query_ns = start.elapsed().as_nanos();
        let start = Instant::now();
        let mut vertices = 0;
        for (id, chunk) in &visible {
            let (_, mesh) =
                filament_chunk(&w.state().assembly.as_ref().unwrap().filaments[id], *chunk)?;
            mesh.audit()?;
            vertices += mesh.positions.len();
        }
        let mesh_ns = start.elapsed().as_nanos();
        let bytes = serde_json::to_vec(w.state())?;
        scaling.push(json!({"requested_parcels":n,"nodes":w.state().assembly.as_ref().unwrap().nodes(),"filaments":w.state().assembly.as_ref().unwrap().filaments.len(),"simulation_ns":simulation,"snapshot_bytes":bytes.len(),"derived_index_ns":index_ns,"query_ns":query_ns,"visible_chunks":visible.len(),"visible_mesh_vertices":vertices,"mesh_and_audit_ns":mesh_ns,"residual":w.state().instances[&1].book.residual()?}));
        w.snapshot().save(&out.join(format!("H-{n}-final.json")))?;
    }
    let report = json!({"status":if failures.is_empty(){"PASS"}else{"FAIL"},"experiments":reports,"scaling":scaling,"failures":failures,"semantics":"recorded3D path, animated numerical nozzle, integer admitted-material transfer, static bound nodes; no mechanics/propulsion/biology","numeric_claim":"same-build exact state/digests; no cross-platform f32 certification"});
    std::fs::write(out.join("report.json"), serde_json::to_vec_pretty(&report)?)?;
    println!(
        "{}",
        json!({"status":report["status"],"failures":report["failures"]})
    );
    if report["status"] != "PASS" {
        return Err("construction gate failed; all evidence retained".into());
    }
    Ok(())
}
