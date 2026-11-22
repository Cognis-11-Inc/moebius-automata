//! Decode/audit compressed M4 snapshots and replay every primary trial.
use moebius_core::{SimConfig, SnapshotFile, World, audit_with_config};
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};
fn decode(path: &Path) -> Result<SnapshotFile, Box<dyn std::error::Error>> {
    let output = Command::new("gzip").arg("-dc").arg(path).output()?;
    if !output.status.success() {
        return Err("gzip failed".into());
    }
    Ok(SnapshotFile::decode(&output.stdout)?)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::PathBuf::from(std::env::args().nth(1).ok_or("dataset directory")?);
    let data: Value = serde_json::from_slice(&fs::read(root.join("results.json"))?)?;
    let mut checked = 0;
    let mut replayed = 0;
    for trial in data["results"].as_array().ok_or("results array")? {
        if trial["status"] != "PASS" {
            continue;
        }
        let prefix = format!(
            "{}_{}_{}",
            trial["environment"].as_str().unwrap(),
            trial["controller"].as_str().unwrap(),
            trial["seed"]
        );
        let c: SimConfig =
            serde_json::from_slice(&fs::read(root.join(format!("{prefix}.config.json")))?)?;
        let initial = decode(&root.join(format!("{prefix}.initial.bin.gz")))?;
        let final_file = decode(&root.join(format!("{prefix}.final.bin.gz")))?;
        audit_with_config(&c, &initial)?;
        audit_with_config(&c, &final_file)?;
        assert_eq!(initial.snapshot.state_hash(), trial["initial_hash"]);
        assert_eq!(final_file.snapshot.state_hash(), trial["final_hash"]);
        let mut world = World::restore(&c, &initial.snapshot)?;
        let end = trial["ticks"].as_u64().ok_or("ticks")?;
        while world.tick_index() < end {
            world.tick()?;
            replayed += 1;
        }
        assert_eq!(world.snapshot().state_hash(), trial["final_hash"]);
        checked += 1;
    }
    println!(
        "{}",
        json!({"status":"PASS","dataset":root,"trials_replayed":checked,
        "snapshots_audited":checked*2,"ticks_replayed":replayed})
    );
    Ok(())
}
