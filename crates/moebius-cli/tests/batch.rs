use std::fs;

use moebius_cli::{RunOpts, run_batch};
use moebius_core::{SnapshotFile, World, load_config_str};

#[test]
fn batch_writes_a_closed_manifest_and_reloadable_snapshot() {
    let root = std::env::temp_dir().join(format!("moebius-m0-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let config_path = root.join("config.json");
    fs::write(
        &config_path,
        r#"{
            "width": 4, "height": 4, "depth": 4,
            "initial_population": 4,
            "initial_energy": 8,
            "initial_nutrient_per_site": 3,
            "nutrient_wave_amplitude": 2,
            "initial_reservoir": 10,
            "initial_waste": 0,
            "inflow_per_tick": 1,
            "inject_per_tick": 1,
            "uptake_per_tick": 1,
            "energy_per_nutrient": 1,
            "maintenance_cost": 1,
            "export_per_tick": 1,
            "site_cap": 20,
            "reservoir_cap": 1000,
            "energy_cap": 100,
            "genome_fill": "derived"
        }"#,
    )
    .unwrap();
    let out = root.join("run");
    let report = run_batch(&RunOpts {
        seed: 21,
        ticks: 5,
        config: config_path,
        out: out.clone(),
        snapshot_interval: 5,
    })
    .unwrap();
    assert_eq!(report.material_residual, 0);
    assert_eq!(report.energy_residual, 0);
    assert_eq!(report.ticks, 5);
    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&report.manifest_path).unwrap()).unwrap();
    assert_eq!(manifest["material_residual"], 0);
    assert_eq!(manifest["energy_residual"], 0);
    assert_eq!(manifest["extinct"], report.extinct);
    assert_eq!(
        manifest["action_energy_is_conserved_physical_quantity"],
        false
    );
    let metrics = fs::read_to_string(out.join("metrics.csv")).unwrap();
    assert_eq!(metrics.lines().count(), 7);
    let final_snap =
        SnapshotFile::decode(&fs::read(out.join("snapshots/snap_0000000005.bin")).unwrap())
            .unwrap();
    let config = load_config_str(&fs::read_to_string(root.join("config.json")).unwrap()).unwrap();
    let world = World::restore(&config, &final_snap.snapshot).unwrap();
    assert_eq!(world.snapshot().state_hash(), report.final_state_hash);
    let again = run_batch(&RunOpts {
        seed: 21,
        ticks: 5,
        config: root.join("config.json"),
        out: out.clone(),
        snapshot_interval: 5,
    });
    assert!(again.unwrap_err().to_string().contains("not empty"));
    let _ = fs::remove_dir_all(&root);
}
