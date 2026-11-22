//! Read an existing batch, decode/audit every stored snapshot and replay from
//! the first snapshot to the final tick, comparing every stored checkpoint.
//! cargo run -p moebius-core --release --example audit_run -- RUN_DIRECTORY
use moebius_core::{SimConfig, SnapshotFile, World, audit_with_config};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(std::env::args().nth(1).ok_or("expected run directory")?);
    let manifest: Value = serde_json::from_slice(&fs::read(root.join("manifest.json"))?)?;
    let config: SimConfig = serde_json::from_value(manifest["config"].clone())?;
    let mut checkpoints = BTreeMap::new();
    let mut first = None;
    let mut paths = fs::read_dir(root.join("snapshots"))?
        .map(|p| p.map(|p| p.path()))
        .collect::<Result<Vec<_>, _>>()?;
    paths.sort();
    for path in paths {
        let file = SnapshotFile::decode(&fs::read(path)?)?;
        audit_with_config(&config, &file)?;
        let snapshot = &file.snapshot;
        checkpoints.insert(snapshot.tick, snapshot.state_hash());
        if first.is_none() {
            first = Some(World::restore(&config, snapshot)?);
        }
    }
    let mut world = first.ok_or("no snapshots")?;
    let end = manifest["ticks"].as_u64().ok_or("missing ticks")?;
    let start = world.tick_index();
    loop {
        if let Some(expected) = checkpoints.get(&world.tick_index()) {
            assert_eq!(&world.snapshot().state_hash(), expected);
        }
        if world.tick_index() == end {
            break;
        }
        world.tick()?;
    }
    assert_eq!(world.snapshot().state_hash(), manifest["final_state_hash"]);
    println!(
        "{}",
        json!({"status":"PASS","run":root,"snapshots_audited":checkpoints.len(),
        "replayed_ticks":end-start,"final_state_hash":world.snapshot().state_hash()})
    );
    Ok(())
}
