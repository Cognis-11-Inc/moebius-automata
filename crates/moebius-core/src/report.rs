use serde::Serialize;

use crate::config::{
    MAX_AMOUNT, MAX_AXIS, MAX_SNAPSHOTS, MAX_TICKS, MAX_TRAIL_ROWS, MAX_VOLUME, SimConfig,
    config_hash,
};
use crate::error::Error;
use crate::geom::NEIGHBORHOOD;
use crate::prng::{MOVEMENT_STREAM, PLACEMENT_STREAM, PRNG};
use crate::snapshot::{STATE_FORMAT, state_hash_of};
use crate::world::{Accounts, World};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Metrics {
    pub tick: u64,
    pub live: u32,
    pub dead: u32,
    pub field: u64,
    pub reservoir: u64,
    pub waste: u64,
    pub external_input: u64,
    pub external_output: u64,
    pub material_residual: i128,
    pub energy_live: u64,
    pub energy_credited: u64,
    pub energy_maintenance: u64,
    pub energy_disposed: u64,
    pub energy_residual: i128,
    pub state_hash: String,
}

pub fn observe(world: &World) -> Result<Metrics, Error> {
    let accounts = world.accounts()?;
    Ok(metrics_from(world, &accounts))
}

fn metrics_from(world: &World, accounts: &Accounts) -> Metrics {
    Metrics {
        tick: world.tick_index(),
        live: world.live_count(),
        dead: world.dead_count(),
        field: accounts.field,
        reservoir: accounts.reservoir,
        waste: accounts.waste,
        external_input: accounts.external_input,
        external_output: accounts.external_output,
        material_residual: accounts.material_residual(),
        energy_live: accounts.energy_live,
        energy_credited: accounts.energy_credited,
        energy_maintenance: accounts.energy_maintenance,
        energy_disposed: accounts.energy_disposed,
        energy_residual: accounts.energy_residual(),
        state_hash: state_hash_of(world),
    }
}

pub fn metrics_csv(rows: &[Metrics]) -> Result<String, Error> {
    if rows.len() as u64 > MAX_TICKS + 1 {
        return Err(Error::Run(format!(
            "metrics rows {} exceed {}",
            rows.len(),
            MAX_TICKS + 1
        )));
    }
    let mut out = String::from(
        "tick,live,dead,field,reservoir,waste,external_input,external_output,material_residual,energy_live,energy_credited,energy_maintenance,energy_disposed,energy_residual,state_hash\n",
    );
    for row in rows {
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            row.tick,
            row.live,
            row.dead,
            row.field,
            row.reservoir,
            row.waste,
            row.external_input,
            row.external_output,
            row.material_residual,
            row.energy_live,
            row.energy_credited,
            row.energy_maintenance,
            row.energy_disposed,
            row.energy_residual,
            row.state_hash
        ));
    }
    Ok(out)
}

#[derive(Clone, Debug, Serialize)]
pub struct BoundsEcho {
    pub max_axis: u32,
    pub max_volume: u64,
    pub max_ticks: u64,
    pub max_snapshots: u64,
    pub max_trail_rows: u64,
    pub max_amount: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct RunManifest {
    pub format: String,
    pub code_version: String,
    pub config: SimConfig,
    pub config_hash: String,
    pub seed: u64,
    pub prng: String,
    pub placement_stream: String,
    pub movement_stream: String,
    pub neighborhood: String,
    pub state_format: String,
    pub ticks: u64,
    pub snapshot_interval: u64,
    pub snapshot_ticks: Vec<u64>,
    pub final_state_hash: String,
    pub final_live_count: u32,
    pub final_dead_count: u32,
    pub extinct: bool,
    pub material_initial: u64,
    pub external_input: u64,
    pub external_output: u64,
    pub material_field: u64,
    pub material_reservoir: u64,
    pub material_waste: u64,
    pub material_residual: i64,
    pub energy_credited: u64,
    pub energy_live: u64,
    pub energy_maintenance: u64,
    pub energy_disposed: u64,
    pub energy_residual: i64,
    pub grid: [u32; 3],
    pub bounds: BoundsEcho,
    /// Action-energy is an accounting stock, not a conserved physical quantity.
    pub action_energy_is_conserved_physical_quantity: bool,
}

pub fn build_manifest(
    code_version: &str,
    config: &SimConfig,
    seed: u64,
    ticks: u64,
    snapshot_interval: u64,
    snapshot_ticks: &[u64],
    world: &World,
) -> Result<RunManifest, Error> {
    let accounts = world.accounts()?;
    let material = accounts.material_residual();
    let energy = accounts.energy_residual();
    if material != 0 || energy != 0 {
        return Err(Error::Invariant(format!(
            "refusing manifest with residuals material {material} energy {energy}"
        )));
    }
    Ok(RunManifest {
        format: "ma-manifest-1".into(),
        code_version: code_version.to_string(),
        config: config.clone(),
        config_hash: config_hash(config)?,
        seed,
        prng: PRNG.into(),
        placement_stream: PLACEMENT_STREAM.into(),
        movement_stream: MOVEMENT_STREAM.into(),
        neighborhood: NEIGHBORHOOD.into(),
        state_format: STATE_FORMAT.into(),
        ticks,
        snapshot_interval,
        snapshot_ticks: snapshot_ticks.to_vec(),
        final_state_hash: state_hash_of(world),
        final_live_count: world.live_count(),
        final_dead_count: world.dead_count(),
        extinct: world.live_count() == 0,
        material_initial: accounts.material_initial,
        external_input: accounts.external_input,
        external_output: accounts.external_output,
        material_field: accounts.field,
        material_reservoir: accounts.reservoir,
        material_waste: accounts.waste,
        material_residual: 0,
        energy_credited: accounts.energy_credited,
        energy_live: accounts.energy_live,
        energy_maintenance: accounts.energy_maintenance,
        energy_disposed: accounts.energy_disposed,
        energy_residual: 0,
        grid: [config.width, config.height, config.depth],
        bounds: BoundsEcho {
            max_axis: MAX_AXIS,
            max_volume: MAX_VOLUME,
            max_ticks: MAX_TICKS,
            max_snapshots: MAX_SNAPSHOTS,
            max_trail_rows: MAX_TRAIL_ROWS,
            max_amount: MAX_AMOUNT,
        },
        action_energy_is_conserved_physical_quantity: false,
    })
}

pub fn manifest_json(manifest: &RunManifest) -> Result<String, Error> {
    let mut text = serde_json::to_string_pretty(manifest)
        .map_err(|err| Error::Run(format!("manifest encode failed: {err}")))?;
    text.push('\n');
    Ok(text)
}

/// Reject a run whose worst-case reservoir, material counter, or energy counter cannot fit.
/// This does not change the requested numbers.
pub fn check_horizon(config: &SimConfig, ticks: u64) -> Result<(), Error> {
    if ticks > MAX_TICKS {
        return Err(Error::Run(format!("ticks {ticks} exceeds max {MAX_TICKS}")));
    }
    let inflow = config
        .inflow_per_tick
        .checked_mul(ticks)
        .ok_or_else(|| Error::Overflow("inflow * ticks".into()))?;
    let worst_reservoir = config
        .initial_reservoir
        .checked_add(inflow)
        .ok_or_else(|| Error::Overflow("reservoir horizon".into()))?;
    if worst_reservoir > config.reservoir_cap {
        return Err(Error::Run(format!(
            "reservoir_cap {} cannot hold initial_reservoir {} plus inflow_per_tick {} over {ticks} ticks (worst case {worst_reservoir} if injection removes nothing). Raise reservoir_cap or lower inflow or ticks",
            config.reservoir_cap, config.initial_reservoir, config.inflow_per_tick
        )));
    }
    let population = u64::from(config.initial_population);
    let seeded = population
        .checked_mul(config.initial_energy)
        .ok_or_else(|| Error::Overflow("seed energy horizon".into()))?;
    let per_tick = if config.energy_per_nutrient == 0 {
        0
    } else {
        population
            .checked_mul(config.uptake_per_tick)
            .ok_or_else(|| Error::Overflow("population * uptake".into()))?
            .checked_mul(config.energy_per_nutrient)
            .ok_or_else(|| Error::Overflow("energy conversion horizon".into()))?
    };
    let _energy_horizon = per_tick
        .checked_mul(ticks)
        .ok_or_else(|| Error::Overflow("energy * ticks".into()))?
        .checked_add(seeded)
        .ok_or_else(|| Error::Overflow("energy horizon".into()))?;
    let volume = config.volume()?;
    let field_cap = volume
        .checked_mul(config.site_cap)
        .ok_or_else(|| Error::Overflow("field cap horizon".into()))?;
    let _material_horizon = field_cap
        .checked_add(config.initial_reservoir)
        .and_then(|sum| sum.checked_add(config.initial_waste))
        .and_then(|sum| sum.checked_add(inflow))
        .ok_or_else(|| Error::Overflow("material horizon".into()))?;
    Ok(())
}

pub fn snapshot_schedule(ticks: u64, interval: u64) -> Vec<u64> {
    let mut out = vec![0];
    if ticks == 0 || interval == 0 {
        return out;
    }
    let mut cursor = interval;
    while cursor < ticks {
        out.push(cursor);
        let Some(next) = cursor.checked_add(interval) else {
            break;
        };
        cursor = next;
    }
    if out.last().copied() != Some(ticks) {
        out.push(ticks);
    }
    out
}

pub fn validate_run(
    config: &SimConfig,
    ticks: u64,
    snapshot_interval: u64,
) -> Result<Vec<u64>, Error> {
    config.validate()?;
    check_horizon(config, ticks)?;
    if snapshot_interval == 0 || snapshot_interval > MAX_TICKS {
        return Err(Error::Run(format!(
            "snapshot_interval {snapshot_interval} is outside 1..={MAX_TICKS}"
        )));
    }
    let snaps = snapshot_schedule(ticks, snapshot_interval);
    if snaps.len() as u64 > MAX_SNAPSHOTS {
        return Err(Error::Run(format!(
            "snapshot count {} exceeds max {MAX_SNAPSHOTS}. Increase snapshot_interval or reduce ticks",
            snaps.len()
        )));
    }
    let trail_upper = ticks
        .saturating_add(1)
        .checked_mul(u64::from(config.initial_population))
        .ok_or_else(|| Error::Overflow("trail row horizon".into()))?;
    if trail_upper > MAX_TRAIL_ROWS {
        return Err(Error::Run(format!(
            "trail upper bound {trail_upper} rows exceeds max {MAX_TRAIL_ROWS}"
        )));
    }
    Ok(snaps)
}
