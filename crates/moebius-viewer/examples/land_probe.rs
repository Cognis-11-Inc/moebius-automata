//! Exact native cathedral-demo recording/replay; no externally imposed gait.
#[path = "../src/motor_demo.rs"]
mod motor_demo;
use moebius_core::{SnapshotFile, World};
use serde_json::{Value, json};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    path::PathBuf,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let command = args.next().ok_or("record|replay")?;
    let root = PathBuf::from(args.next().ok_or("directory")?);
    if command == "record" {
        fs::create_dir(&root)?;
        let demo = PathBuf::from(args.next().ok_or("demo config")?);
        let mut w = motor_demo::load(&demo, "compare")?;
        fs::copy(demo, root.join("demo.json"))?;
        fs::write(
            root.join("initial.bin"),
            SnapshotFile::encode(w.config(), &w)?,
        )?;
        let mut trace = fs::File::create_new(root.join("ticks.jsonl"))?;
        for _ in 0..1024 {
            let receipt = w.tick()?;
            assert_eq!(w.accounts()?.material_residual(), 0);
            assert_eq!(w.accounts()?.energy_residual(), 0);
            let record = json!({"tick":w.tick_index(),"receipt":receipt,"state_hash":w.snapshot().state_hash(),"land":w.snapshot().flesh,"reservoir":w.accounts()?.reservoir,"land_units":w.accounts()?.flesh,"live":w.live_count(),"energy":w.cells().map(|c|json!({"id":c.id,"energy":c.energy})).collect::<Vec<_>>()});
            serde_json::to_writer(&mut trace, &record)?;
            writeln!(trace)?;
        }
        fs::write(
            root.join("final.bin"),
            SnapshotFile::encode(w.config(), &w)?,
        )?;
        let summary = json!({"status":"PASS","ticks":1024,"live":w.live_count(),"land_units":w.accounts()?.flesh,"reservoir":w.accounts()?.reservoir,"field":w.accounts()?.field,"material_residual":w.accounts()?.material_residual(),"energy_residual":w.accounts()?.energy_residual(),"state_hash":w.snapshot().state_hash(),"config_hash":moebius_core::config_hash(w.config())?});
        fs::write(
            root.join("summary.json"),
            serde_json::to_vec_pretty(&summary)?,
        )?;
        println!("{summary}");
    } else if command == "replay" {
        let initial = SnapshotFile::decode(&fs::read(root.join("initial.bin"))?)?;
        let genesis = motor_demo::load(&root.join("demo.json"), "compare")?;
        let mut w = World::restore(genesis.config(), &initial.snapshot)?;
        for line in BufReader::new(fs::File::open(root.join("ticks.jsonl"))?).lines() {
            let row: Value = serde_json::from_str(&line?)?;
            let receipt = w.tick()?;
            assert_eq!(serde_json::to_value(receipt)?, row["receipt"]);
            assert_eq!(w.snapshot().state_hash(), row["state_hash"]);
            assert_eq!(serde_json::to_value(w.snapshot().flesh)?, row["land"]);
            assert_eq!(w.accounts()?.material_residual(), 0);
            assert_eq!(w.accounts()?.energy_residual(), 0);
        }
        let final_file = SnapshotFile::decode(&fs::read(root.join("final.bin"))?)?;
        assert_eq!(w.snapshot().state_hash(), final_file.snapshot.state_hash());
        println!(
            "{}",
            json!({"status":"PASS","ticks":w.tick_index(),"state_hash":w.snapshot().state_hash()})
        );
    } else {
        return Err("record|replay".into());
    }
    Ok(())
}
