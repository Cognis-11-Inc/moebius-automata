//! M0 scientific core for Möbius Automata.
//!
//! A digital cell is an identifiable lifecycle with a site, not a pixel.
//! The lattice is a hybrid artificial-life substrate with one material field.
//! This crate does not implement binding, division, mutation, roles, signals,
//! collectives, or evolutionary success. It does not link to other Cognis systems.
//!
//! Action-energy is an integer accounting stock created by a stated conversion
//! and destroyed by maintenance or death. It is not a conserved physical quantity.
//! Material (field + reservoir + waste + external output) is conserved exactly
//! against the initial stock plus external input.

#![forbid(unsafe_code)]

mod error;
mod geom;
mod hashutil;
mod prng;

pub mod config;
pub mod layers;
pub mod report;
pub mod snapshot;
pub mod world;

pub use config::{
    GenomeFill, MAX_AMOUNT, MAX_AXIS, MAX_SNAPSHOTS, MAX_TICKS, MAX_TRAIL_ROWS, MAX_VOLUME,
    SimConfig, config_hash, load_config_str,
};
pub use geom::{
    Dims, Dir, MoveOutcome, MoveProposal, NEIGHBORHOOD, Site, resolve_moves, step_site,
    uptake_units, wrap_coord,
};
pub use layers::{
    FluidSample, HypothesisLayer, ObservationDeck, TrailPoint, fluid_samples, format_trails,
    parse_hypothesis, parse_trails, split_trail,
};
pub use prng::{MOVEMENT_STREAM, PLACEMENT_STREAM, PRNG, movement_dir};
pub use report::{
    Metrics, RunManifest, build_manifest, check_horizon, manifest_json, metrics_csv, observe,
    snapshot_schedule, validate_run,
};
pub use snapshot::{
    SNAP_MAGIC, SNAP_VERSION, STATE_FORMAT, Snapshot, SnapshotFile, audit_with_config,
};
pub use world::{Accounts, Cell, DeathRecord, Stocks, TickReceipt, World, apply_receipt};

#[derive(Clone, Debug)]
pub struct RunOutcome {
    pub receipts: Vec<TickReceipt>,
    pub metrics: Vec<Metrics>,
    pub final_hash: String,
    pub live_count: u32,
    pub extinct: bool,
}

/// Commit `ticks` steps. Observers are not invoked with mutable access.
pub fn run_for(world: &mut World, ticks: u64) -> Result<RunOutcome, Error> {
    check_horizon(world.config(), ticks)?;
    let mut metrics = vec![observe(world)?];
    let mut receipts = Vec::with_capacity(usize::try_from(ticks).unwrap_or(0));
    for _ in 0..ticks {
        receipts.push(world.tick()?);
        metrics.push(observe(world)?);
    }
    let live_count = world.live_count();
    Ok(RunOutcome {
        final_hash: snapshot::state_hash_of(world),
        extinct: live_count == 0,
        live_count,
        receipts,
        metrics,
    })
}

pub use error::Error;
