//! Open run. The worker owns the world and calls `World::tick`.
//! Render frames only drain pictures. They do not tick.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use bevy::prelude::Resource;
use moebius_core::{Dims, SimConfig, Site, World, check_horizon, config_hash, fluid_samples};

use crate::layout::display_stride;

const LIVE_GAP: Duration = Duration::from_millis(125);
const PATH_RING: usize = 192;

#[derive(Clone)]
pub struct Founder {
    pub id: u64,
    pub site: Site,
    pub energy: u64,
    pub genome: [u8; 8],
    pub rig: Option<moebius_core::farm::BrainRig>,
}

#[derive(Clone)]
pub struct FluidBit {
    pub origin: Site,
    pub sites: u32,
    pub amount: f32,
}

#[derive(Clone)]
pub struct Picture {
    pub tick: u64,
    pub state_hash: String,
    pub cells: Vec<Founder>,
    pub fluid: Vec<FluidBit>,
    pub live: u32,
    pub dead: u32,
    pub field: u64,
    pub material_flesh: u64,
    pub flesh: Vec<moebius_core::farm::FleshDeposit>,
    pub reservoir: u64,
    pub waste: u64,
    pub external_input: u64,
    pub external_output: u64,
    pub energy_live: u64,
    pub energy_credited: u64,
    pub energy_maintenance: u64,
    pub energy_disposed: u64,
    pub material_initial: u64,
    pub error: Option<String>,
}

impl Picture {
    pub fn material_residual(&self) -> i128 {
        i128::from(self.field)
            + i128::from(self.material_flesh)
            + i128::from(self.reservoir)
            + i128::from(self.waste)
            + i128::from(self.external_output)
            - i128::from(self.external_input)
            - i128::from(self.material_initial)
    }

    pub fn energy_residual(&self) -> i128 {
        i128::from(self.energy_live)
            + i128::from(self.energy_maintenance)
            + i128::from(self.energy_disposed)
            - i128::from(self.energy_credited)
    }

    fn failed(err: String) -> Self {
        Self {
            tick: 0,
            state_hash: String::new(),
            cells: Vec::new(),
            fluid: Vec::new(),
            live: 0,
            dead: 0,
            field: 0,
            material_flesh: 0,
            flesh: Vec::new(),
            reservoir: 0,
            waste: 0,
            external_input: 0,
            external_output: 0,
            energy_live: 0,
            energy_credited: 0,
            energy_maintenance: 0,
            energy_disposed: 0,
            material_initial: 0,
            error: Some(err),
        }
    }
}

#[derive(Resource)]
pub struct Hall {
    pub seed: u64,
    pub config_hash: String,
    pub code_version: String,
    pub energy_cap: u64,
    pub site_cap: u64,
    pub founding: u32,
    pub dims: Dims,
    pub stride: u32,
    pause: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
    inbox: Mutex<Receiver<Picture>>,
    pub latest: Option<Picture>,
    pub paths: BTreeMap<u64, Vec<(u64, Site)>>,
    pub motion_paths: BTreeMap<u64, Vec<(u64, [f32; 3])>>,
    pub poses: BTreeMap<u64, ([f32; 3], [f32; 3])>,
    pub spawned_ids: std::collections::BTreeSet<u64>,
    pub fluid_origins: Vec<Site>,
    pub fluid_sites: Vec<u32>,
    pub fluid_amounts: Vec<f32>,
    pub fluid_before: Vec<f32>,
    pub note: String,
    pub halted: Option<String>,
    pub spawned: bool,
    committed: u64,
    received: Instant,
    gap: f32,
}

impl Hall {
    pub fn interpolated_tick(&self) -> f32 {
        if self.committed == 0 {
            return 0.0;
        }
        self.committed as f32 - 1.0
            + (self.received.elapsed().as_secs_f32() / self.gap).clamp(0.0, 1.0)
    }

    pub fn committed_tick(&self) -> u64 {
        self.committed
    }

    pub fn set_paused(&self, paused: bool) {
        self.pause.store(paused, Ordering::Relaxed);
    }

    pub fn is_paused(&self) -> bool {
        self.pause.load(Ordering::Relaxed)
    }

    pub fn drain(&mut self) {
        let batch = {
            let inbox = self.inbox.lock().unwrap_or_else(|err| err.into_inner());
            let mut batch = Vec::new();
            while let Ok(picture) = inbox.try_recv() {
                batch.push(picture);
            }
            batch
        };
        for picture in batch {
            self.absorb(picture);
        }
    }

    fn absorb(&mut self, picture: Picture) {
        if let Some(err) = picture.error.clone() {
            self.halted = Some(err.clone());
            self.note = format!("tick refused: {err}. The last committed state stays.");
            self.set_paused(true);
            if self
                .latest
                .as_ref()
                .is_some_and(|prior| prior.tick == picture.tick)
            {
                return;
            }
        }
        for cell in &picture.cells {
            let mut position = crate::layout::site_to_view(cell.site, self.dims);
            if let Some(rig) = &cell.rig {
                for (axis, coordinate) in position.iter_mut().enumerate() {
                    *coordinate += rig.offset[axis] as f32 / moebius_core::farm::SUBSITE as f32
                        * crate::layout::SPACING;
                }
            }
            let ring = self.motion_paths.entry(cell.id).or_default();
            let prior = ring.last().map_or(position, |(_, p)| *p);
            let wrap = prior
                .iter()
                .zip(position)
                .any(|(a, b)| (*a - b).abs() > crate::layout::SPACING * 2.0);
            self.poses
                .insert(cell.id, (if wrap { position } else { prior }, position));
            ring.push((picture.tick, position));
            if ring.len() > PATH_RING {
                ring.remove(0);
            }
            let ring = self.paths.entry(cell.id).or_default();
            if ring.last().is_none_or(|(tick, _)| *tick != picture.tick) {
                ring.push((picture.tick, cell.site));
                if ring.len() > PATH_RING {
                    let extra = ring.len() - PATH_RING;
                    ring.drain(0..extra);
                }
            }
        }
        if self.fluid_origins.is_empty() && !picture.fluid.is_empty() {
            self.fluid_origins = picture.fluid.iter().map(|bit| bit.origin).collect();
            self.fluid_sites = picture.fluid.iter().map(|bit| bit.sites).collect();
        }
        if picture.fluid.len() == self.fluid_origins.len() || self.fluid_origins.is_empty() {
            self.fluid_before.clone_from(&self.fluid_amounts);
            self.fluid_amounts = picture.fluid.iter().map(|bit| bit.amount).collect();
            if self.fluid_before.len() != self.fluid_amounts.len() {
                self.fluid_before.clone_from(&self.fluid_amounts);
            }
        }
        if picture.error.is_none() {
            self.note.clear();
        }
        let earliest = picture.tick.saturating_sub(PATH_RING as u64);
        self.motion_paths
            .retain(|_, path| path.last().is_some_and(|(tick, _)| *tick >= earliest));
        self.paths
            .retain(|_, path| path.last().is_some_and(|(tick, _)| *tick >= earliest));
        let elapsed = self.received.elapsed().as_secs_f32();
        if elapsed > 0.02 && self.latest.is_some() {
            self.gap = (self.gap * 0.5 + elapsed * 0.5).clamp(0.08, 2.0);
        }
        self.received = Instant::now();
        self.committed = picture.tick;
        self.latest = Some(picture);
    }
}

impl Drop for Hall {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

pub fn open(config: &SimConfig, seed: u64) -> Result<Hall, String> {
    config.validate().map_err(|err| err.to_string())?;
    check_horizon(config, 1).map_err(|err| err.to_string())?;
    let hash = config_hash(config).map_err(|err| err.to_string())?;
    let dims = Dims {
        width: config.width,
        height: config.height,
        depth: config.depth,
    };
    let stride = display_stride(dims);
    let energy_cap = config.energy_cap;
    let site_cap = config.site_cap;
    let founding = config.initial_population;
    let (tx, rx) = mpsc::sync_channel(4);
    let pause = Arc::new(AtomicBool::new(false));
    let stop = Arc::new(AtomicBool::new(false));
    let pause_worker = Arc::clone(&pause);
    let stop_worker = Arc::clone(&stop);
    let owned = config.clone();
    thread::Builder::new()
        .name("moebius-tick".into())
        .spawn(move || pump(owned, seed, stride, tx, pause_worker, stop_worker))
        .map_err(|err| format!("cannot start the tick thread: {err}"))?;
    Ok(Hall {
        seed,
        config_hash: hash,
        code_version: moebius_cli::code_version(),
        energy_cap,
        site_cap,
        founding,
        dims,
        stride,
        pause,
        stop,
        inbox: Mutex::new(rx),
        latest: None,
        paths: BTreeMap::new(),
        motion_paths: BTreeMap::new(),
        poses: BTreeMap::new(),
        spawned_ids: std::collections::BTreeSet::new(),
        fluid_origins: Vec::new(),
        fluid_sites: Vec::new(),
        fluid_amounts: Vec::new(),
        fluid_before: Vec::new(),
        note: "Waiting for tick 0. Render frames are not ticking the world.".into(),
        halted: None,
        spawned: false,
        committed: 0,
        received: Instant::now(),
        gap: 0.25,
    })
}

fn pump(
    config: SimConfig,
    seed: u64,
    stride: u32,
    tx: SyncSender<Picture>,
    pause: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
) {
    let mut world = match World::genesis(&config, seed) {
        Ok(world) => world,
        Err(err) => {
            let _ = tx.send(Picture::failed(err.to_string()));
            return;
        }
    };
    if tx.send(shoot(&world, stride, None)).is_err() {
        return;
    }
    loop {
        if !wait(&stop, &pause) {
            return;
        }
        if pause.load(Ordering::Relaxed) {
            continue;
        }
        match world.tick() {
            Ok(_) => {
                if tx.send(shoot(&world, stride, None)).is_err() {
                    return;
                }
            }
            Err(err) => {
                let _ = tx.send(shoot(&world, stride, Some(err.to_string())));
                return;
            }
        }
    }
}

fn wait(stop: &AtomicBool, pause: &AtomicBool) -> bool {
    let mut slept = Duration::ZERO;
    while slept < LIVE_GAP {
        if stop.load(Ordering::Relaxed) {
            return false;
        }
        thread::sleep(Duration::from_millis(25));
        if pause.load(Ordering::Relaxed) {
            continue;
        }
        slept += Duration::from_millis(25);
    }
    !stop.load(Ordering::Relaxed)
}

fn shoot(world: &World, stride: u32, error: Option<String>) -> Picture {
    let accounts = match world.accounts() {
        Ok(accounts) => accounts,
        Err(err) => return Picture::failed(err.to_string()),
    };
    let fluid = match fluid_samples(world.nutrient(), world.dims(), stride) {
        Ok(samples) => samples
            .into_iter()
            .map(|sample| FluidBit {
                origin: sample.origin,
                sites: sample.sites,
                amount: sample.amount as f32,
            })
            .collect(),
        Err(err) => return Picture::failed(err.to_string()),
    };
    let mut cells: Vec<Founder> = world
        .cells()
        .map(|cell| Founder {
            id: cell.id,
            site: cell.site,
            energy: cell.energy,
            genome: cell.genome,
            rig: world.brains().get(&cell.id).cloned(),
        })
        .collect();
    cells.sort_by_key(|cell| cell.id);
    Picture {
        tick: world.tick_index(),
        state_hash: world.snapshot().state_hash(),
        cells,
        fluid,
        live: world.live_count(),
        dead: world.dead_count(),
        field: accounts.field,
        material_flesh: accounts.flesh,
        flesh: world
            .flesh()
            .iter()
            .map(|(site, amount)| moebius_core::farm::FleshDeposit {
                site: *site,
                amount: *amount,
            })
            .collect(),
        reservoir: accounts.reservoir,
        waste: accounts.waste,
        external_input: accounts.external_input,
        external_output: accounts.external_output,
        energy_live: accounts.energy_live,
        energy_credited: accounts.energy_credited,
        energy_maintenance: accounts.energy_maintenance,
        energy_disposed: accounts.energy_disposed,
        material_initial: accounts.material_initial,
        error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use moebius_core::GenomeFill;

    fn tiny() -> SimConfig {
        SimConfig {
            width: 4,
            height: 4,
            depth: 4,
            initial_population: 3,
            initial_energy: 12,
            initial_nutrient_per_site: 3,
            nutrient_wave_amplitude: 0,
            initial_reservoir: 40,
            initial_waste: 0,
            inflow_per_tick: 1,
            inject_per_tick: 1,
            uptake_per_tick: 1,
            energy_per_nutrient: 2,
            maintenance_cost: 1,
            export_per_tick: 1,
            site_cap: 20,
            reservoir_cap: 10_000,
            energy_cap: 1_000,
            genome_fill: GenomeFill::Zero,
            farm: None,
        }
    }

    #[test]
    fn worker_commits_past_tick_zero_and_stops_on_drop() {
        let mut hall = open(&tiny(), 11).unwrap();
        let mut advanced = false;
        for _ in 0..300 {
            hall.drain();
            if let Some(picture) = &hall.latest {
                assert_eq!(picture.material_residual(), 0);
                assert_eq!(picture.energy_residual(), 0);
                assert!(picture.error.is_none());
            }
            if hall.committed_tick() >= 1
                && hall
                    .latest
                    .as_ref()
                    .is_some_and(|picture| picture.live == 3)
            {
                advanced = true;
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(advanced, "the worker did not commit tick 1");
        assert!(hall.paths.values().all(|ring| ring.len() <= PATH_RING));
        drop(hall);
    }

    #[test]
    fn ring_keeps_the_recent_steps() {
        let (tx, rx) = mpsc::sync_channel(1);
        drop(tx);
        let mut hall = Hall {
            seed: 1,
            config_hash: String::new(),
            code_version: String::new(),
            energy_cap: 10,
            site_cap: 8,
            founding: 1,
            dims: Dims {
                width: 4,
                height: 4,
                depth: 4,
            },
            stride: 1,
            pause: Arc::new(AtomicBool::new(false)),
            stop: Arc::new(AtomicBool::new(false)),
            inbox: Mutex::new(rx),
            latest: None,
            paths: BTreeMap::new(),
            motion_paths: BTreeMap::new(),
            poses: BTreeMap::new(),
            spawned_ids: std::collections::BTreeSet::new(),
            fluid_origins: Vec::new(),
            fluid_sites: Vec::new(),
            fluid_amounts: Vec::new(),
            fluid_before: Vec::new(),
            note: String::new(),
            halted: None,
            spawned: false,
            committed: 0,
            received: Instant::now(),
            gap: 0.25,
        };
        for tick in 0..(PATH_RING as u64 + 16) {
            hall.absorb(Picture {
                tick,
                state_hash: String::new(),
                cells: vec![Founder {
                    id: 1,
                    site: Site {
                        x: u32::try_from(tick % 4).unwrap(),
                        y: 0,
                        z: 0,
                    },
                    energy: 4,
                    genome: [0; 8],
                    rig: None,
                }],
                fluid: Vec::new(),
                live: 1,
                dead: 0,
                field: 0,
                material_flesh: 0,
                flesh: Vec::new(),
                reservoir: 0,
                waste: 0,
                external_input: 0,
                external_output: 0,
                energy_live: 4,
                energy_credited: 4,
                energy_maintenance: 0,
                energy_disposed: 0,
                material_initial: 0,
                error: None,
            });
        }
        let ring = &hall.paths[&1];
        assert_eq!(ring.len(), PATH_RING);
        assert_eq!(ring.first().unwrap().0, 16);
        assert_eq!(ring.last().unwrap().0, PATH_RING as u64 + 15);
    }
}
