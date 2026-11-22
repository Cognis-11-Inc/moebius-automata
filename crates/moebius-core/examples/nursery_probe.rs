//! Authored-habitat viability probe. No learned-foraging or adaptation verdict.
//! cargo run -p moebius-core --release --example nursery_probe -- CONFIG OUT
use moebius_core::{SimConfig, SnapshotFile, World};
use serde_json::{Value, json};
use std::{fs, path::PathBuf};

fn trial(
    config: &SimConfig,
    seed: u64,
    ticks: u64,
    root: &std::path::Path,
    condition: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut world = World::genesis(config, seed)?;
    let initial_genotypes: Vec<_> = world
        .brains()
        .values()
        .map(|b| b.neural.as_ref().unwrap().genome.id())
        .collect();
    let mut births = 0;
    let mut intake = 0;
    let mut observed = Vec::new();
    for step in 0..ticks {
        let receipt = world.tick()?;
        births += receipt.births;
        intake += receipt.nutrient_uptaken;
        if (step + 1) % 64 == 0 {
            observed.push(json!({"tick":world.tick_index(),"live":world.live_count(),
                "dead":world.dead_count(),"births":births,"intake":intake,
                "energy":world.accounts()?.energy_live}));
        }
    }
    if seed == 1 {
        fs::write(
            root.join(format!("final_{condition}.bin")),
            SnapshotFile::encode(config, &world)?,
        )?;
        fs::write(
            root.join(format!("config_{condition}.json")),
            serde_json::to_vec_pretty(config)?,
        )?;
    }
    let generation = world
        .brains()
        .values()
        .map(|b| b.generation)
        .max()
        .unwrap_or(0);
    Ok(
        json!({"status":"PASS","condition":condition,"seed":seed,"ticks":ticks,
        "initial_genotypes":initial_genotypes,"births":births,"intake":intake,
        "final_live":world.live_count(),"final_dead":world.dead_count(),
        "maximum_surviving_generation":generation,"final_hash":world.snapshot().state_hash(),
        "observations":observed,"material_residual":world.accounts()?.material_residual() as i64,
        "energy_residual":world.accounts()?.energy_residual() as i64}),
    )
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let config: SimConfig = serde_json::from_slice(&fs::read(args.next().ok_or("config path")?)?)?;
    let root = PathBuf::from(args.next().ok_or("output path")?);
    if root.exists() {
        return Err("choose a new output directory".into());
    }
    fs::create_dir_all(&root)?;
    fs::write(
        root.join("plan.json"),
        include_bytes!("../../../docs/nursery/PROTOCOL.json"),
    )?;
    let mut results = Vec::new();
    for condition in ["provisioned", "food_absent"] {
        let mut c = config.clone();
        if condition == "food_absent" {
            c.feeding.as_mut().ok_or("nursery required")?.initial_stock = 0;
            c.inflow_per_tick = 0;
            c.inject_per_tick = 0;
        }
        for seed in 1..=8 {
            results.push(trial(&c,seed,1024,&root,condition).unwrap_or_else(|error|
                json!({"status":"FAIL","condition":condition,"seed":seed,"error":error.to_string()})));
            fs::write(
                root.join("results.partial.json"),
                serde_json::to_vec_pretty(&results)?,
            )?;
        }
    }
    let failures = results.iter().filter(|r| r["status"] == "FAIL").count();
    fs::write(
        root.join("results.json"),
        serde_json::to_vec_pretty(&json!({"results":results,
        "failures":failures,"verdict":"Provisioned habitat mechanics; useful learning and adaptive evolution NOT DEMONSTRATED"}))?,
    )?;
    println!(
        "{}",
        json!({"out":root,"trials":results.len(),"failures":failures})
    );
    if failures > 0 {
        return Err("probe failed; records preserved".into());
    }
    Ok(())
}
