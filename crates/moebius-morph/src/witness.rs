//! Read-only inspection of the canonical ship runtime. Caches and worker publication own no biology.
use crate::{MorphError, require, runtime::*};
use std::{
    collections::BTreeSet,
    sync::{Arc, Mutex, mpsc},
    thread,
    time::{Duration, Instant},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WitnessSphere {
    pub center: Position,
    pub radius: i64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WitnessView {
    pub tick: u64,
    pub organisms: Vec<u64>,
    pub parcels: Vec<(u64, u64)>,
    pub exposures: Vec<(u64, u64)>,
}
impl WitnessSphere {
    pub fn contains(&self, p: Position) -> Result<bool, MorphError> {
        require(
            (0..=MAX_POSITION).contains(&self.radius),
            "invalid witness radius",
        )?;
        Ok(p.relative(self.center)?
            .iter()
            .map(|v| i128::from(*v).pow(2))
            .sum::<i128>()
            <= i128::from(self.radius).pow(2))
    }

    pub fn inspect(&self, state: &WorldSnapshot) -> Result<WitnessView, MorphError> {
        self.center.absolute()?;
        require(
            (0..=MAX_POSITION).contains(&self.radius),
            "invalid witness radius",
        )?;
        let inside = |p| self.contains(p);
        let mut view = WitnessView {
            tick: state.tick,
            organisms: vec![],
            parcels: vec![],
            exposures: vec![],
        };
        for i in state.instances.values() {
            if inside(i.position)? {
                view.organisms.push(i.id);
                view.exposures.push((i.id, i.exposure));
            }
            for p in &i.book.parcels {
                if inside(Position::from_absolute(p.position)?)? {
                    view.parcels.push((i.id, p.id));
                }
            }
        }
        Ok(view)
    }
}
#[derive(Default)]
pub struct PresentationCache {
    pub organisms: BTreeSet<u64>,
    pub parcels: BTreeSet<(u64, u64)>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoundaryChanges {
    pub entered: Vec<u64>,
    pub exited: Vec<u64>,
    pub material_entered: usize,
    pub material_exited: usize,
}
impl PresentationCache {
    pub fn update(&mut self, view: &WitnessView) -> BoundaryChanges {
        let next = view.organisms.iter().copied().collect::<BTreeSet<_>>();
        let material = view.parcels.iter().copied().collect::<BTreeSet<_>>();
        let changes = BoundaryChanges {
            entered: next.difference(&self.organisms).copied().collect(),
            exited: self.organisms.difference(&next).copied().collect(),
            material_entered: material.difference(&self.parcels).count(),
            material_exited: self.parcels.difference(&material).count(),
        };
        self.organisms = next;
        self.parcels = material;
        changes
    }
}
/// Authority-thread timings. Not world state and not part of a digest.
#[derive(Clone, Debug, Default)]
pub struct RunTiming {
    /// `ShipRuntime::tick` only. The pace sleep is outside this sample.
    pub tick_ns: Vec<u128>,
    /// Snapshot copy published to the witness after a tick.
    pub publish_ns: Vec<u128>,
    pub collision: crate::collision::CollisionProfile,
}
pub struct Worker {
    latest: Arc<Mutex<Option<Arc<WorldSnapshot>>>>,
    result: mpsc::Receiver<Result<WorldSnapshot, MorphError>>,
    thread: Option<thread::JoinHandle<()>>,
    controls: mpsc::SyncSender<Playback>,
    status: Arc<Mutex<PlaybackStatus>>,
    timing: Arc<Mutex<RunTiming>>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Playback {
    Play,
    Pause,
    Step,
    Reset,
    Rate(u32),
    Stop,
}
#[derive(Clone, Debug)]
pub struct PlaybackStatus {
    pub tick: u64,
    pub generation: u64,
    pub playing: bool,
    pub rate: u32,
    pub controls: Vec<(u64, u64, Playback)>,
    pub reset_digests: Vec<(u64, u64, String)>,
    pub error: Option<String>,
}
impl Worker {
    pub fn spawn(
        world: ShipRuntime,
        ticks: u64,
        publish_every: u64,
        pace: Duration,
    ) -> Result<Self, MorphError> {
        Self::launch(world, ticks, publish_every, pace, false, false)
    }
    pub fn interactive(
        world: ShipRuntime,
        ticks: u64,
        pace: Duration,
        paused: bool,
    ) -> Result<Self, MorphError> {
        Self::launch(world, ticks, 1, pace, true, paused)
    }
    pub fn control(&self, command: Playback) -> Result<(), MorphError> {
        require(
            !matches!(command,Playback::Rate(n)if ![1,4,16].contains(&n)),
            "unsupported playback rate",
        )?;
        self.controls
            .try_send(command)
            .map_err(|_| MorphError("control queue full or runtime worker finished".into()))
    }
    pub fn playback(&self) -> Result<PlaybackStatus, MorphError> {
        Ok(self
            .status
            .lock()
            .map_err(|_| MorphError("playback status poisoned".into()))?
            .clone())
    }
    fn launch(
        mut world: ShipRuntime,
        ticks: u64,
        publish_every: u64,
        pace: Duration,
        interactive: bool,
        paused: bool,
    ) -> Result<Self, MorphError> {
        require(publish_every > 0, "publication interval must be positive")?;
        let execution_policy = world.collision_execution_policy();
        let initial = world.snapshot();
        let latest = Arc::new(Mutex::new(Some(Arc::new(initial.clone()))));
        let slot = latest.clone();
        let status = Arc::new(Mutex::new(PlaybackStatus {
            tick: initial.tick,
            generation: 0,
            playing: !paused,
            rate: 1,
            controls: vec![],
            reset_digests: vec![],
            error: None,
        }));
        let info = status.clone();
        let (tx, result) = mpsc::channel();
        let (controls, commands) = mpsc::sync_channel(64);
        let timing = Arc::new(Mutex::new(RunTiming::default()));
        let recorded = timing.clone();
        let handle = thread::Builder::new()
            .name("moebius-ship-authority".into())
            .spawn(move || {
                let run: Result<WorldSnapshot, MorphError> = (|| {
                    let mut left = ticks;
                    let mut playing = !paused;
                    let mut rate = 1;
                    let mut single = 0usize;
                    let mut stop = false;
                    let mut tick_ns = Vec::new();
                    let mut publish_ns = Vec::new();
                    loop {
                        for command in commands.try_iter() {
                            let mut state = info
                                .lock()
                                .map_err(|_| MorphError("playback status poisoned".into()))?;
                            require(
                                state.controls.len() < 512,
                                "playback control history budget",
                            )?;
                            let ordinal = state.controls.len() as u64;
                            state.controls.push((ordinal, world.state().tick, command));
                            match command {
                                Playback::Play => playing = true,
                                Playback::Pause => playing = false,
                                Playback::Step => {
                                    playing = false;
                                    single += 1;
                                }
                                Playback::Rate(n) => rate = n,
                                Playback::Reset => {
                                    let generation = state.generation;
                                    state.reset_digests.push((
                                        generation,
                                        world.state().tick,
                                        world.digest()?,
                                    ));
                                    world = ShipRuntime::restore(initial.clone())?;
                                    world.configure_collision_execution(execution_policy)?;
                                    left = ticks;
                                    single = 0;
                                    tick_ns.clear();
                                    publish_ns.clear();
                                    state.generation += 1;
                                    *slot.lock().map_err(|_| {
                                        MorphError("snapshot slot poisoned".into())
                                    })? = Some(Arc::new(world.snapshot()));
                                }
                                Playback::Stop => stop = true,
                            }
                        }
                        if stop || (!interactive && left == 0) {
                            break;
                        }
                        if left == 0 {
                            playing = false;
                        }
                        let budget = if playing {
                            rate as usize
                        } else {
                            single.min(1)
                        };
                        for _ in 0..budget.min(left as usize) {
                            let tick_started = Instant::now();
                            world.tick()?;
                            tick_ns.push(tick_started.elapsed().as_nanos());
                            left -= 1;
                            single = single.saturating_sub(1);
                            let publish_started = Instant::now();
                            if world.state().tick.is_multiple_of(publish_every) {
                                *slot
                                    .lock()
                                    .map_err(|_| MorphError("snapshot slot poisoned".into()))? =
                                    Some(Arc::new(world.snapshot()));
                            }
                            publish_ns.push(publish_started.elapsed().as_nanos());
                        }
                        {
                            let mut s = info
                                .lock()
                                .map_err(|_| MorphError("playback status poisoned".into()))?;
                            s.tick = world.state().tick;
                            s.playing = playing;
                            s.rate = rate;
                        }
                        if !pace.is_zero() {
                            thread::sleep(if budget == 0 {
                                Duration::from_millis(1)
                            } else {
                                pace
                            });
                        } else if budget == 0 {
                            thread::sleep(Duration::from_millis(1));
                        }
                    }
                    let snapshot = world.snapshot();
                    {
                        let mut timing = recorded
                            .lock()
                            .map_err(|_| MorphError("timing poisoned".into()))?;
                        timing.tick_ns = tick_ns;
                        timing.publish_ns = publish_ns;
                        timing.collision = world.collision_profile();
                    }
                    *slot
                        .lock()
                        .map_err(|_| MorphError("snapshot slot poisoned".into()))? =
                        Some(Arc::new(snapshot.clone()));
                    Ok(snapshot)
                })();
                if let Err(error) = &run
                    && let Ok(mut state) = info.lock()
                {
                    state.error = Some(error.to_string());
                    state.playing = false;
                }
                let _ = tx.send(run);
            })
            .map_err(|e| MorphError(e.to_string()))?;
        Ok(Self {
            latest,
            result,
            thread: Some(handle),
            controls,
            status,
            timing,
        })
    }
    pub fn latest(&self) -> Result<Arc<WorldSnapshot>, MorphError> {
        self.latest
            .lock()
            .map_err(|_| MorphError("snapshot slot poisoned".into()))?
            .clone()
            .ok_or_else(|| MorphError("no snapshot".into()))
    }
    pub fn finish(mut self) -> Result<WorldSnapshot, MorphError> {
        let result = self.result.recv().map_err(|e| MorphError(e.to_string()))?;
        if let Some(handle) = self.thread.take() {
            handle
                .join()
                .map_err(|_| MorphError("authority worker panic".into()))?;
        }
        result
    }
    /// Waits for the authority thread, then returns its tick samples.
    /// Samples are not world state. A failed run can leave them empty.
    pub fn finish_measured(self) -> Result<(WorldSnapshot, RunTiming), MorphError> {
        let timing = Arc::clone(&self.timing);
        let snapshot = self.finish()?;
        let timing = timing
            .lock()
            .map_err(|_| MorphError("timing poisoned".into()))?
            .clone();
        Ok((snapshot, timing))
    }
}
