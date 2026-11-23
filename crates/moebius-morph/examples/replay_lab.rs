//! Rebuild every captured lab pose using only the headless engine, not Bevy.
use moebius_morph::*;
use serde_json::json;
use std::path::PathBuf;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = PathBuf::from(
        std::env::args()
            .nth(1)
            .ok_or("supply walkthrough directory")?,
    );
    let mut captures = 0;
    let mut phases = vec![];
    let mut seeds = std::collections::BTreeSet::new();
    let mut body_positions = vec![];
    let mut flag_positions = vec![];
    let mut files = std::fs::read_dir(&dir)?
        .map(|r| r.map(|r| r.path()))
        .collect::<Result<Vec<_>, _>>()?;
    files.sort();
    for file in files {
        if file.extension().is_none_or(|v| v != "json") {
            continue;
        }
        let value: serde_json::Value = serde_json::from_slice(&std::fs::read(&file)?)?;
        if value.get("geometry_digest").is_none() {
            continue;
        }
        let spec = anatomy_from_json(&serde_json::to_string(&value["specification"])?)?;
        let pose: Pose = serde_json::from_value(value["pose"].clone())?;
        let geometry = if value["specification"]["version"] == 1 {
            CompiledWorm::new(from_json(&serde_json::to_string(&value["specification"])?)?)?
                .pose(pose)?
        } else {
            CompiledAnatomy::new(spec.clone())?.pose(pose)?.geometry
        };
        geometry.audit()?;
        if geometry.digest() != value["geometry_digest"].as_str().unwrap()
            || geometry.topology_digest() != value["topology_digest"].as_str().unwrap()
        {
            return Err(format!("capture replay mismatch: {}", file.display()).into());
        }
        if value["frame"].as_u64().unwrap() < 100 {
            phases.push(pose.phase);
            body_positions.push(geometry.surfaces[0].positions.clone());
            if geometry.surfaces.len() > 1 {
                flag_positions.push(geometry.surfaces[1].positions.clone());
            }
        }
        seeds.insert(spec.body.seed);
        captures += 1;
    }
    let changed = |list: &Vec<Vec<[f32; 3]>>| list.windows(2).any(|w| w[0] != w[1]);
    if captures < 10 || !changed(&body_positions) || !changed(&flag_positions) {
        return Err("missing rendered deformation evidence".into());
    }
    println!(
        "{}",
        json!({"status":"PASS","captures_replayed":captures,"body_positions_changed":changed(&body_positions),"flagellar_positions_changed":changed(&flag_positions),"motion_phase_samples":phases.len(),"seed_variants":seeds.len(),"consumer":"headless moebius-morph; no Bevy"})
    );
    Ok(())
}
