//! Independent numerical replay of every rendered snapshot, including pre-reset histories.
use moebius_morph::runtime::*;
use std::{collections::BTreeMap, path::PathBuf};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = PathBuf::from(std::env::args().nth(1).ok_or("capture directory")?);
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(out.join("render-report.json"))?)?;
    let mut w = ShipRuntime::restore(WorldSnapshot::load(&out.join("world-initial.json"))?)?;
    let final_state = WorldSnapshot::load(&out.join("world-final.json"))?;
    let rows = report["rows"].as_array().ok_or("missing frame rows")?;
    let max = rows
        .iter()
        .filter_map(|r| r["tick"].as_u64())
        .chain(Some(final_state.tick))
        .max()
        .unwrap();
    let mut hashes = BTreeMap::new();
    hashes.insert(0, w.digest()?);
    for _ in 0..max {
        w.tick()?;
        hashes.insert(w.state().tick, w.digest()?);
    }
    for row in rows {
        let tick = row["tick"].as_u64().ok_or("bad tick")?;
        if row["digest"].as_str() != Some(hashes[&tick].as_str()) {
            return Err("rendered authoritative snapshot mismatch".into());
        }
        if row["meshes_generated"].as_u64().unwrap() > report["mesh_budget"].as_u64().unwrap() {
            return Err("mesh budget exceeded".into());
        }
    }
    for reset in report["reset_digests"]
        .as_array()
        .ok_or("missing reset provenance")?
    {
        let tick = reset[1].as_u64().unwrap();
        if hashes.get(&tick).map(String::as_str) != reset[2].as_str() {
            return Err("pre-reset state mismatch".into());
        }
    }
    let result = serde_json::json!({"status":"PASS","frames_verified":rows.len(),"final_tick":final_state.tick,"final_digest":final_state.digest()?,"per_frame_state_and_mesh_budget_match":true,"reset_histories_verified":report["reset_digests"].as_array().unwrap().len(),"consumer":"headless moebius-morph; no renderer"});
    println!("{result}");
    Ok(())
}
