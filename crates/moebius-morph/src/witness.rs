//! Read-only inspection of the canonical ship runtime. Caches and worker publication own no biology.
use crate::{MorphError, require, runtime::*};
use std::{
    collections::BTreeSet,
    sync::{Arc, Mutex, mpsc},
    thread,
    time::Duration,
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
    pub fn inspect(&self, state: &WorldSnapshot) -> Result<WitnessView, MorphError> {
        self.center.absolute()?;
        require(
            (0..=MAX_POSITION).contains(&self.radius),
            "invalid witness radius",
        )?;
        let inside = |p: Position| -> Result<bool, MorphError> {
            let d = p.relative(self.center)?;
            Ok(d.iter()
                .map(|&v| i128::from(v) * i128::from(v))
                .sum::<i128>()
                <= i128::from(self.radius) * i128::from(self.radius))
        };
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
pub struct Worker {
    latest: Arc<Mutex<Option<Arc<WorldSnapshot>>>>,
    result: mpsc::Receiver<Result<WorldSnapshot, MorphError>>,
    thread: Option<thread::JoinHandle<()>>,
}
impl Worker {
    /// Finite explicit run. Pacing may slow wall-clock progress; it never changes tick count.
    /// No camera/input/render timing or receiver acknowledgement is passed to the owner.
    pub fn spawn(
        mut world: ShipRuntime,
        ticks: u64,
        publish_every: u64,
        pace: Duration,
    ) -> Result<Self, MorphError> {
        require(publish_every > 0, "publication interval must be positive")?;
        let latest = Arc::new(Mutex::new(Some(Arc::new(world.snapshot()))));
        let slot = latest.clone();
        let (tx, result) = mpsc::channel();
        let handle = thread::Builder::new()
            .name("moebius-ship-authority".into())
            .spawn(move || {
                let run = (|| {
                    for _ in 0..ticks {
                        world.tick()?;
                        if world.state().tick.is_multiple_of(publish_every) {
                            *slot
                                .lock()
                                .map_err(|_| MorphError("snapshot slot poisoned".into()))? =
                                Some(Arc::new(world.snapshot()));
                        }
                        if !pace.is_zero() {
                            thread::sleep(pace);
                        }
                    }
                    let snapshot = world.snapshot();
                    *slot
                        .lock()
                        .map_err(|_| MorphError("snapshot slot poisoned".into()))? =
                        Some(Arc::new(snapshot.clone()));
                    Ok(snapshot)
                })();
                let _ = tx.send(run);
            })
            .map_err(|e| MorphError(e.to_string()))?;
        Ok(Self {
            latest,
            result,
            thread: Some(handle),
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
}
