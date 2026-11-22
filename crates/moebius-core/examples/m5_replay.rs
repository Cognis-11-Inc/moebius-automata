//! Exact replay of expanded M5 archives, including every controlled input.
#[allow(dead_code)]
#[path = "support/m5.rs"]
mod support;
use moebius_core::{SimConfig, Site, SnapshotFile, World, audit_with_config};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use support::*;
fn bytes(root: &Path, name: &str) -> Result<Vec<u8>> {
    let plain = root.join(name);
    if plain.exists() {
        return Ok(fs::read(plain)?);
    }
    let compressed = root.join(format!("{name}.gz"));
    let output = Command::new("gzip").arg("-dc").arg(compressed).output()?;
    if !output.status.success() {
        return Err("gzip decode failed".into());
    }
    Ok(output.stdout)
}
fn sensor_replay(t: &Value) -> Result<()> {
    let case = Case {
        name: "sensor_probe".into(),
        seed: 1,
        drag: 8,
        energy: 128,
        target_delta: None,
        founding_bite: false,
        horizon: 0,
    };
    let (w, _) = fixture(&case)?;
    let mut s = w.snapshot();
    let orientation: [i32; 3] = serde_json::from_value(t["orientation"].clone())?;
    s.brains[0].neural.as_mut().unwrap().body.orientation = orientation;
    let target: Site = serde_json::from_value(t["target"].clone())?;
    s.nutrient[(target.x + SIDE * (target.y + SIDE * target.z)) as usize] = 512;
    s.material_initial = 512;
    let mut config = w.config().clone();
    if t["sensing_variant"] == "sensors_lobes" {
        config.neural.as_mut().unwrap().chemical_sensing =
            moebius_core::neural::ChemicalSensing::LocalLobesV1;
    }
    let w = World::restore(&config, &s)?;
    assert_eq!(
        serde_json::to_value(w.organism_sensors(ID)?)?,
        t["ordinary_sensors"]
    );
    assert_eq!(serde_json::to_value(com(&w))?, t["com"]);
    assert_eq!(serde_json::to_value(mouth(&w))?, t["mouth"]);
    Ok(())
}
fn main() -> Result<()> {
    let root = PathBuf::from(
        std::env::args()
            .nth(1)
            .ok_or("expanded dataset directory")?,
    );
    let data: Value = serde_json::from_slice(&fs::read(root.join("results.json"))?)?;
    let mut trials = 0;
    let mut ticks = 0;
    let mut sensors = 0;
    let mut snapshots = 0;
    let mut case_audits = Vec::new();
    for result in data["results"].as_array().ok_or("results array")? {
        if result["status"] != "PASS" {
            continue;
        }
        if result.get("case").is_none() {
            sensor_replay(result)?;
            sensors += 1;
            continue;
        }
        let case: Case = serde_json::from_value(result["case"].clone())?;
        let c: SimConfig =
            serde_json::from_slice(&fs::read(root.join(format!("{}.config.json", case.name)))?)?;
        let initial = SnapshotFile::decode(&bytes(&root, &format!("{}.initial.bin", case.name))?)?;
        let final_file = SnapshotFile::decode(&bytes(&root, &format!("{}.final.bin", case.name))?)?;
        audit_with_config(&c, &initial)?;
        audit_with_config(&c, &final_file)?;
        snapshots += 2;
        let mut w = World::restore(&c, &initial.snapshot)?;
        let initial_accounts = w.accounts()?;
        let strengths = w.brains()[&ID]
            .neural
            .as_ref()
            .unwrap()
            .genome
            .strengths
            .clone();
        let raw = bytes(&root, &format!("{}.jsonl", case.name))?;
        for line in std::str::from_utf8(&raw)?.lines() {
            let row: Value = serde_json::from_str(line)?;
            assert_eq!(
                serde_json::to_value(w.organism_sensors(ID)?)?,
                row["sensors_before"]
            );
            assert_eq!(serde_json::to_value(mouth(&w))?, row["mouth_before"]);
            let (receipt, observations) = if row["submitted_commands"].is_null() {
                w.tick_observed()?
            } else {
                let command: Vec<i32> = serde_json::from_value(row["submitted_commands"].clone())?;
                w.tick_controlled(&BTreeMap::from([(ID, command)]))?
            };
            assert_eq!(serde_json::to_value(&receipt)?, row["receipt"]);
            assert_eq!(serde_json::to_value(&observations[0])?, row["organism"]);
            assert_eq!(w.snapshot().state_hash(), row["state_hash"]);
            assert_eq!(w.accounts()?.energy_residual(), 0);
            assert_eq!(w.accounts()?.material_residual(), 0);
            ticks += 1;
        }
        assert_eq!(w.snapshot().state_hash(), result["final_hash"]);
        assert_eq!(final_file.snapshot.state_hash(), result["final_hash"]);
        let accounts = w.accounts()?;
        case_audits.push(json!({"name":case.name,"initial_field":initial_accounts.field,"final_field":accounts.field,
            "final_waste":accounts.waste,"external_input":accounts.external_input,"external_output":accounts.external_output,
            "initial_energy":initial_accounts.energy_live,"live_energy":accounts.energy_live,"energy_credited":accounts.energy_credited,
            "energy_maintenance":accounts.energy_maintenance,"energy_disposed":accounts.energy_disposed,"strengths":strengths,
            "actuator_debt_final_if_alive":w.brains().get(&ID).and_then(|b|b.neural.as_ref()).map(|s|s.actuator_debt)}));
        trials += 1;
    }
    println!(
        "{}",
        json!({"status":"PASS","dataset":root.file_name().unwrap().to_string_lossy(),"trials":trials,
        "snapshots":snapshots,"ticks":ticks,"sensor_probes":sensors,"cases":case_audits})
    );
    Ok(())
}
