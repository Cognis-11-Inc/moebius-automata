#![forbid(unsafe_code)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use moebius_core::format_trails;
use moebius_core::{
    Error, SnapshotFile, World, build_manifest, load_config_str, manifest_json, metrics_csv,
    observe, validate_run,
};

#[derive(Clone, Debug)]
pub struct RunOpts {
    pub seed: u64,
    pub ticks: u64,
    pub config: PathBuf,
    pub out: PathBuf,
    pub snapshot_interval: u64,
}

#[derive(Clone, Debug)]
pub struct BatchReport {
    pub manifest_path: PathBuf,
    pub final_state_hash: String,
    pub material_residual: i128,
    pub energy_residual: i128,
    pub extinct: bool,
    pub live: u32,
    pub ticks: u64,
}

pub fn code_version() -> String {
    if let Ok(value) = std::env::var("MOEBIUS_GIT_SHA") {
        let value = value.trim();
        if !value.is_empty() {
            return value.to_string();
        }
    }
    match std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
    {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }
        _ => "unknown".to_string(),
    }
}

pub fn run_batch(opts: &RunOpts) -> Result<BatchReport, Error> {
    if opts.out.exists() {
        let mut entries = fs::read_dir(&opts.out)
            .map_err(|err| Error::Io(format!("cannot read {}: {err}", opts.out.display())))?;
        if let Some(entry) = entries.next() {
            entry.map_err(|err| Error::Io(err.to_string()))?;
            return Err(Error::Run(format!(
                "{} is not empty; choose a new --out. This command does not delete existing output",
                opts.out.display()
            )));
        }
    }
    let text = fs::read_to_string(&opts.config)
        .map_err(|err| Error::Io(format!("cannot read {}: {err}", opts.config.display())))?;
    let config = load_config_str(&text)?;
    let schedule = validate_run(&config, opts.ticks, opts.snapshot_interval)?;
    fs::create_dir_all(opts.out.join("snapshots"))
        .map_err(|err| Error::Io(format!("cannot create output directory: {err}")))?;

    let mut world = World::genesis(&config, opts.seed)?;
    let mut metrics = vec![observe(&world)?];
    let mut trails = world.trail_points();
    let mut due: std::collections::BTreeSet<u64> = schedule.iter().copied().collect();
    if due.remove(&world.tick_index()) {
        write_snapshot(&opts.out, &config, &world)?;
    }
    for _ in 0..opts.ticks {
        world.tick()?;
        metrics.push(observe(&world)?);
        trails.extend(world.trail_points());
        if due.remove(&world.tick_index()) {
            write_snapshot(&opts.out, &config, &world)?;
        }
    }
    if !due.is_empty() {
        return Err(Error::Invariant(format!(
            "snapshot ticks were not written: {due:?}"
        )));
    }
    let metrics_text = metrics_csv(&metrics)?;
    write_atomic(&opts.out.join("metrics.csv"), metrics_text.as_bytes())?;
    write_atomic(
        &opts.out.join("trails.csv"),
        format_trails(&trails).as_bytes(),
    )?;
    let manifest = build_manifest(
        &code_version(),
        &config,
        opts.seed,
        opts.ticks,
        opts.snapshot_interval,
        &schedule,
        &world,
    )?;
    let manifest_path = opts.out.join("manifest.json");
    write_atomic(&manifest_path, manifest_json(&manifest)?.as_bytes())?;
    let accounts = world.accounts()?;
    Ok(BatchReport {
        manifest_path,
        final_state_hash: manifest.final_state_hash,
        material_residual: accounts.material_residual(),
        energy_residual: accounts.energy_residual(),
        extinct: manifest.extinct,
        live: manifest.final_live_count,
        ticks: opts.ticks,
    })
}

fn write_snapshot(
    out: &Path,
    config: &moebius_core::SimConfig,
    world: &World,
) -> Result<(), Error> {
    let bytes = SnapshotFile::encode(config, world)?;
    let path = out.join(format!("snapshots/snap_{:010}.bin", world.tick_index()));
    write_atomic(&path, &bytes)
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), Error> {
    let tmp = path.with_extension("tmp");
    {
        let mut file = fs::File::create(&tmp)
            .map_err(|err| Error::Io(format!("cannot create {}: {err}", tmp.display())))?;
        file.write_all(bytes)
            .map_err(|err| Error::Io(format!("cannot write {}: {err}", tmp.display())))?;
        file.sync_all()
            .map_err(|err| Error::Io(format!("cannot sync {}: {err}", tmp.display())))?;
    }
    fs::rename(&tmp, path).map_err(|err| {
        Error::Io(format!(
            "cannot rename {} to {}: {err}",
            tmp.display(),
            path.display()
        ))
    })?;
    Ok(())
}

pub fn summary_json(report: &BatchReport) -> String {
    serde_json::json!({
        "manifest": report.manifest_path,
        "final_state_hash": report.final_state_hash,
        "material_residual": report.material_residual,
        "energy_residual": report.energy_residual,
        "extinct": report.extinct,
        "live": report.live,
        "ticks": report.ticks,
    })
    .to_string()
}
