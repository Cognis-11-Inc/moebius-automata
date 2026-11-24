//! Reproducible headless/observer/performance comparison; no Bevy or GPU dependencies.
use moebius_morph::{runtime::*, witness::*};
use serde_json::json;
use std::{
    collections::BTreeMap,
    io::Write,
    path::PathBuf,
    time::{Duration, Instant},
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = PathBuf::from(
        std::env::args()
            .nth(1)
            .ok_or("new output directory required")?,
    );
    std::fs::create_dir(&out)?;
    let mut rows = std::fs::File::create(out.join("trials.jsonl"))?;
    let mut reports = vec![];
    let mut failures = vec![];
    for population in [100, 1000, 10000] {
        let result = (|| -> Result<serde_json::Value, Box<dyn std::error::Error>> {
            let initial = fixture(population, 20261123)?.snapshot();
            let mut digests = BTreeMap::new();
            let mut modes = vec![];
            for mode in ["headless", "stationary", "moving"] {
                let mut world = ShipRuntime::restore(initial.clone())?;
                let mut cache = PresentationCache::default();
                let mut simulation = 0;
                let mut observation = 0;
                let mut crossing = 0;
                let mut entered = 0;
                let mut exited = 0;
                let mut history = vec![];
                for tick in 0..16 {
                    let started = Instant::now();
                    world.tick()?;
                    simulation += started.elapsed().as_nanos();
                    if mode != "headless" {
                        let sphere = WitnessSphere {
                            center: Position::from_absolute([
                                if mode == "moving" && tick % 2 == 1 {
                                    1000 * 1_000_000
                                } else {
                                    0
                                },
                                0,
                                0,
                            ])?,
                            radius: if mode == "stationary" || tick % 3 == 0 {
                                24_000_000
                            } else {
                                8_000_000
                            },
                        };
                        let started = Instant::now();
                        let view = sphere.inspect(world.state())?;
                        observation += started.elapsed().as_nanos();
                        let started = Instant::now();
                        let change = cache.update(&view);
                        crossing += started.elapsed().as_nanos();
                        entered += change.entered.len();
                        exited += change.exited.len();
                    }
                    let digest = world.digest()?;
                    history.push(digest.clone());
                    writeln!(
                        rows,
                        "{}",
                        json!({"population":population,"mode":mode,"tick":tick+1,"digest":digest})
                    )?;
                    rows.flush()?;
                }
                let started = Instant::now();
                let snapshot = world.snapshot();
                let snapshot_ns = started.elapsed().as_nanos();
                let started = Instant::now();
                let bytes = serde_json::to_vec(&snapshot)?;
                let serialization_ns = started.elapsed().as_nanos();
                let started = Instant::now();
                let decoded: WorldSnapshot = serde_json::from_slice(&bytes)?;
                let restored = ShipRuntime::restore(decoded)?;
                let restore_ns = started.elapsed().as_nanos();
                if restored.snapshot() != snapshot {
                    return Err("persistence mismatch".into());
                }
                world.validate()?;
                let digest = world.digest()?;
                digests.insert(mode, history);
                modes.push(json!({"mode":mode,"final_digest":digest,"simulation_ns":simulation,"simulation_ticks_per_second":16e9/simulation as f64,"organism_ticks_per_second":population as f64*16e9/simulation as f64,"observation_ns":observation,"boundary_cache_ns":crossing,"entered":entered,"exited":exited,"snapshot_construction_ns":snapshot_ns,"snapshot_serialized_bytes":bytes.len(),"serialization_ns":serialization_ns,"restore_validation_ns":restore_ns,"parcels":snapshot.instances.values().map(|i|i.book.parcels.len()).sum::<usize>(),"material_residual":snapshot.instances.values().map(|i|i.book.residual().unwrap()).sum::<i128>()}));
            }
            if digests["headless"] != digests["stationary"]
                || digests["headless"] != digests["moving"]
            {
                return Err("observation neutrality failed".into());
            }
            let started = Instant::now();
            let mut replay = ShipRuntime::restore(initial)?;
            let mut replay_digests = vec![];
            for _ in 0..16 {
                replay.tick()?;
                replay_digests.push(replay.digest()?);
            }
            let replay_ns = started.elapsed().as_nanos();
            if replay_digests != digests["headless"] {
                return Err("independent replay failed".into());
            }
            Ok(
                json!({"status":"PASS","population":population,"ticks_per_mode":16,"all_per_tick_digests_equal":true,"modes":modes,"independent_replay_with_digests_ns":replay_ns}),
            )
        })();
        match result {
            Ok(r) => reports.push(r),
            Err(e) => {
                let r = json!({"status":"FAIL","population":population,"error":e.to_string()});
                writeln!(rows, "{r}")?;
                failures.push(r);
            }
        }
    }
    let mut reference = fixture(8, 20261123)?;
    reference.advance(120)?;
    let started = Instant::now();
    let worker = Worker::spawn(fixture(8, 20261123)?, 120, 1, Duration::ZERO)?;
    std::thread::sleep(Duration::from_millis(25));
    let worker = worker.finish()?;
    let delayed_equal = worker == reference.snapshot();
    let report = json!({"status":if failures.is_empty()&&delayed_equal{"PASS"}else{"FAIL"},"base_revision":"20f0feb","fixture_semantics":"shared fixed rest anatomy/nozzle bindings, prescribed drift, exposure sampling, one conserved emission per nozzle per tick, full integer material integration; no physiology, controllers, learning, propulsion or mesh generation per tick","populations":reports,"failures":failures,"delayed_no_reader_worker":{"ticks":120,"exact_final_state_equal":delayed_equal,"elapsed_including_25ms_sleep_ns":started.elapsed().as_nanos()},"memory":"Use /usr/bin/time -v for process peak RSS; JSON size is storage cost, not resident heap size","gpu":"No GPU or renderer in this executable","numeric_tolerance":"zero for same-build serialized state and digests; initial f32 anatomy compilation not cross-platform certified"});
    std::fs::write(out.join("report.json"), serde_json::to_vec_pretty(&report)?)?;
    println!(
        "{}",
        json!({"status":report["status"],"populations":[100,1000,10000]})
    );
    if report["status"] != "PASS" {
        return Err("benchmark gate failed; failed rows retained".into());
    }
    Ok(())
}
