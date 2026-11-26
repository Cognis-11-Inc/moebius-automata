//! Bounded numerical workers. Only the caller may consume a receipt or mutate
//! solver/world state. Geometry allocation identities are revision capabilities:
//! all shapes are immutable and owned until every receipt is discarded.
use super::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    mpsc,
};
use std::thread::JoinHandle;

const CAPACITY: usize = 64;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ExecutionPolicy {
    pub workers: usize,
    pub graft: bool,
    pub recovery: bool,
    pub static_queries: bool,
    pub frontier_limit: usize,
    pub minimum_frontier: usize,
}
impl Default for ExecutionPolicy {
    fn default() -> Self {
        Self {
            workers: 1,
            graft: false,
            recovery: false,
            static_queries: false,
            frontier_limit: 8,
            minimum_frontier: 4,
        }
    }
}
#[derive(Clone, Debug, Default, Serialize)]
pub struct ExecutionProfile {
    pub eligible: u64,
    pub dispatched: u64,
    pub consumed: u64,
    pub stale: u64,
    pub unnecessary: u64,
    pub recalculated: u64,
    pub failures: u64,
    pub capacity_bypass: u64,
    pub dispatch_wait_ns: u128,
    pub synchronization_upper_ns: u128,
    pub fallback_ns: u128,
    pub worker_query_ns: Vec<u128>,
    pub worker_support_entries_max: Vec<usize>,
    pub frontier_sizes: BTreeMap<usize, u64>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Context {
    pub tick: u64,
    pub pass: usize,
    pub body: u64,
    pub target: Option<PhysicalTarget>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Key {
    context: Context,
    ordinal: usize,
    component: u64,
    revisions: [usize; 2],
    radii: [u32; 2],
    poses: [RigidPose; 4],
    parameters: [u32; 16],
}
#[derive(Clone)]
enum Kind {
    Cast,
    Contact(Iso, Iso),
}
pub(super) struct Task {
    key: Key,
    a: Arc<Geometry>,
    b: Arc<Geometry>,
    kind: Kind,
    #[cfg(test)]
    fault: u8,
}
fn shared(g: &Geometry) -> Arc<Geometry> {
    // Query kernels consume shape/radius/vertices, not the anatomical catalogue.
    Arc::new(Geometry {
        shape: g.shape.clone(),
        radius: g.radius,
        vertices: g.vertices.clone(),
        parts: Vec::new(),
        outlets: Vec::new(),
        writhe_slack: OnceLock::new(),
    })
}
impl Task {
    pub(super) fn cast(
        context: Context,
        ordinal: usize,
        component: u64,
        a: &Geometry,
        b: &Geometry,
        poses: [RigidPose; 4],
    ) -> Self {
        Self {
            key: Key {
                context,
                ordinal,
                component,
                revisions: [
                    Arc::as_ptr(&a.shape) as *const () as usize,
                    Arc::as_ptr(&b.shape) as *const () as usize,
                ],
                radii: [a.radius.to_bits(), b.radius.to_bits()],
                poses,
                parameters: [0; 16],
            },
            a: shared(a),
            b: shared(b),
            kind: Kind::Cast,
            #[cfg(test)]
            fault: 0,
        }
    }
    pub(super) fn contact(
        context: Context,
        ordinal: usize,
        a: &Geometry,
        b: &Geometry,
        poses: [RigidPose; 4],
        sample: (Iso, Iso, f32),
    ) -> Self {
        let mut task = Self::cast(context, ordinal, 0, a, b, poses);
        let (pa, pb, t) = sample;
        let mut parameters = [0; 16];
        for (offset, p) in [(0, pa), (7, pb)] {
            parameters[offset..offset + 3]
                .copy_from_slice(&p.translation.to_array().map(f32::to_bits));
            parameters[offset + 3..offset + 7]
                .copy_from_slice(&p.rotation.to_array().map(f32::to_bits));
        }
        parameters[14] = t.to_bits();
        parameters[15] = 0.001f32.to_bits();
        task.key.parameters = parameters;
        task.kind = Kind::Contact(pa, pb);
        task
    }
    pub(super) fn key(&self) -> Key {
        self.key.clone()
    }
}
#[derive(Clone, Debug)]
pub(super) enum Value {
    Cast(Hit),
    Contact(Option<query::Contact>),
}
struct Receipt {
    generation: u64,
    worker: usize,
    key: Key,
    value: Result<Value, MorphError>,
    healthy: bool,
    ns: u128,
    support_entries: usize,
}
pub(super) struct Frontier {
    generation: u64,
    receipts: Vec<Option<Receipt>>,
    // Prevent pointer reuse even if caller invalidates geometry before consume.
    _inputs: Arc<Vec<Task>>,
}
struct Command {
    generation: u64,
    tasks: Arc<Vec<Task>>,
    next: Arc<AtomicUsize>,
    #[cfg(test)]
    park: Option<Arc<std::sync::Barrier>>,
}
struct Worker {
    command: Option<mpsc::SyncSender<Command>>,
    results: mpsc::Receiver<Vec<Receipt>>,
    handle: Option<JoinHandle<()>>,
}
pub(super) struct Executor {
    pub policy: ExecutionPolicy,
    pub profile: ExecutionProfile,
    generation: u64,
    workers: Vec<Worker>,
    #[cfg(test)]
    fault: Option<(usize, u8)>,
}
impl Executor {
    pub(super) fn new(policy: ExecutionPolicy) -> Result<Self, MorphError> {
        require(
            (2..=32).contains(&policy.workers) && (2..=CAPACITY).contains(&policy.minimum_frontier),
            "invalid collision executor policy",
        )?;
        let mut this = Self {
            policy,
            profile: ExecutionProfile {
                worker_query_ns: vec![0; policy.workers],
                worker_support_entries_max: vec![0; policy.workers],
                ..Default::default()
            },
            generation: 0,
            workers: Vec::new(),
            #[cfg(test)]
            fault: None,
        };
        for worker in 0..policy.workers {
            let (send, receive) = mpsc::sync_channel::<Command>(1);
            let (results, output) = mpsc::sync_channel(1);
            let handle = std::thread::Builder::new()
                .name(format!("collision-{worker}"))
                .spawn(move || {
                    let mut workspace = ContactWorkspace::default();
                    while let Ok(command) = receive.recv() {
                        #[cfg(test)]
                        if let Some(park) = &command.park {
                            park.wait();
                            park.wait();
                        }
                        let mut receipts = Vec::new();
                        loop {
                            let index = command.next.fetch_add(1, Ordering::Relaxed);
                            let Some(task) = command.tasks.get(index) else {
                                break;
                            };
                            workspace.contacts.clear();
                            let started = Instant::now();
                            let value =
                                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                    #[cfg(test)]
                                    if task.fault == 1 {
                                        panic!("injected numerical worker failure")
                                    }
                                    #[cfg(test)]
                                    if task.fault == 2 {
                                        return Err(MorphError(
                                            "injected unreachable query error".into(),
                                        ));
                                    }
                                    match task.kind {
                                        Kind::Cast => cast(
                                            &task.a,
                                            task.key.poses[0],
                                            task.key.poses[1],
                                            &task.b,
                                            task.key.poses[2],
                                            task.key.poses[3],
                                            (&mut CollisionProfile::default(), &mut workspace),
                                        )
                                        .map(Value::Cast),
                                        Kind::Contact(pa, pb) => cached_world_contact(
                                            &mut workspace,
                                            &pa,
                                            &task.a,
                                            &pb,
                                            &task.b,
                                            0.001,
                                        )
                                        .map(|_| {
                                            Value::Contact(
                                                workspace.contacts[&contact_key(
                                                    &pa.inv_mul(&pb),
                                                    &task.a,
                                                    &task.b,
                                                    0.001,
                                                )]
                                                    .contact,
                                            )
                                        })
                                        .map_err(|_| {
                                            MorphError("unsupported overlap recovery query".into())
                                        }),
                                    }
                                }));
                            let healthy = value.is_ok();
                            if !healthy {
                                workspace = ContactWorkspace::default();
                            }
                            receipts.push(Receipt {
                                generation: command.generation,
                                worker,
                                key: task.key.clone(),
                                value: value.unwrap_or_else(|_| {
                                    Err(MorphError("numerical worker failed".into()))
                                }),
                                healthy,
                                ns: started.elapsed().as_nanos(),
                                support_entries: workspace.supports.shapes.len(),
                            });
                        }
                        if results.send(receipts).is_err() {
                            break;
                        }
                    }
                })
                .map_err(|e| MorphError(format!("collision worker start: {e}")))?;
            this.workers.push(Worker {
                command: Some(send),
                results: output,
                handle: Some(handle),
            });
        }
        Ok(this)
    }
    pub(super) fn run(&mut self, tasks: Vec<Task>) -> Option<Frontier> {
        #[cfg(test)]
        let tasks = {
            let mut tasks = tasks;
            if let Some((ordinal, fault)) = self.fault
                && let Some(task) = tasks.get_mut(ordinal)
            {
                task.fault = fault;
            }
            tasks
        };
        let count = tasks.len();
        self.profile.eligible += count as u64;
        *self.profile.frontier_sizes.entry(count).or_default() += 1;
        if count < self.policy.minimum_frontier {
            return None;
        }
        if count > CAPACITY {
            self.profile.capacity_bypass += 1;
            return None;
        }
        let generation = self.generation.checked_add(1)?;
        self.generation = generation;
        let tasks = Arc::new(tasks);
        let next = Arc::new(AtomicUsize::new(0));
        let started = Instant::now();
        let mut sent = Vec::new();
        for (id, worker) in self.workers.iter().enumerate() {
            if worker.command.as_ref().is_some_and(|send| {
                send.send(Command {
                    generation,
                    tasks: tasks.clone(),
                    next: next.clone(),
                    #[cfg(test)]
                    park: None,
                })
                .is_ok()
            }) {
                sent.push(id);
            } else {
                self.profile.failures += 1;
            }
        }
        let receipt_capacity = tasks
            .iter()
            .map(|t| t.key.ordinal)
            .max()
            .map_or(0, |i| i + 1)
            .min(CAPACITY);
        let mut receipts = (0..receipt_capacity).map(|_| None).collect::<Vec<_>>();
        let mut busy = [0u128; 32];
        for id in sent {
            match self.workers[id].results.recv() {
                Ok(rows) => {
                    for receipt in rows {
                        if !receipt.healthy {
                            self.profile.failures += 1;
                        }
                        self.profile.dispatched += 1;
                        self.profile.worker_query_ns[id] += receipt.ns;
                        busy[id] += receipt.ns;
                        self.profile.worker_support_entries_max[id] =
                            self.profile.worker_support_entries_max[id]
                                .max(receipt.support_entries);
                        let ordinal = receipt.key.ordinal;
                        if ordinal < receipt_capacity
                            && receipt.worker == id
                            && receipts[ordinal].is_none()
                        {
                            receipts[ordinal] = Some(receipt);
                        } else {
                            self.profile.failures += 1;
                        }
                    }
                }
                Err(_) => self.profile.failures += 1,
            }
        }
        let elapsed = started.elapsed().as_nanos();
        self.profile.dispatch_wait_ns += elapsed;
        self.profile.synchronization_upper_ns +=
            elapsed.saturating_sub(*busy.iter().max().unwrap());
        Some(Frontier {
            generation,
            receipts,
            _inputs: tasks,
        })
    }
    pub(super) fn consume(
        &mut self,
        frontier: &mut Frontier,
        key: &Key,
    ) -> Option<Result<Value, MorphError>> {
        let receipt = frontier
            .receipts
            .get_mut(key.ordinal)
            .and_then(Option::take);
        if let Some(receipt) = receipt {
            let kind_matches = match &receipt.value {
                Ok(Value::Cast(_)) => key.parameters[15] == 0,
                Ok(Value::Contact(_)) => key.parameters[15] == 0.001f32.to_bits(),
                Err(_) => true,
            };
            if receipt.generation == frontier.generation
                && receipt.key == *key
                && receipt.healthy
                && receipt.worker < self.workers.len()
                && kind_matches
            {
                self.profile.consumed += 1;
                return Some(receipt.value);
            }
            self.profile.stale += 1;
        }
        self.profile.recalculated += 1;
        None
    }
    pub(super) fn finish(&mut self, frontier: Frontier) {
        self.profile.unnecessary += frontier.receipts.iter().filter(|r| r.is_some()).count() as u64;
    }
}
impl Drop for Executor {
    fn drop(&mut self) {
        for worker in &mut self.workers {
            worker.command.take();
        }
        for worker in &mut self.workers {
            if let Some(handle) = worker.handle.take() {
                let _ = handle.join();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ball() -> Geometry {
        Geometry {
            shape: Arc::new(Ball::new(0.2)),
            radius: 0.2,
            vertices: Arc::new(vec![Vector::ZERO]),
            parts: vec![],
            outlets: vec![],
            writhe_slack: OnceLock::new(),
        }
    }
    fn policy() -> ExecutionPolicy {
        ExecutionPolicy {
            workers: 2,
            graft: true,
            recovery: true,
            static_queries: true,
            minimum_frontier: 2,
            ..Default::default()
        }
    }
    fn tasks(a: &Geometry, b: &Geometry, n: usize) -> Vec<Task> {
        (0..n)
            .map(|i| {
                Task::cast(
                    Context {
                        tick: 7,
                        pass: 2,
                        body: 4,
                        target: Some(PhysicalTarget::Pipe {
                            filament: 9,
                            chunk: 0,
                        }),
                    },
                    i,
                    i as u64,
                    a,
                    b,
                    [RigidPose::identity(); 4],
                )
            })
            .collect()
    }
    #[test]
    fn receipts_reject_all_changed_inputs_order_and_worker_tokens() {
        let a = ball();
        let b = ball();
        let mut executor = Executor::new(policy()).unwrap();
        for mutation in 0..12 {
            let input = tasks(&a, &b, 2);
            let mut key = input[0].key();
            let mut frontier = executor.run(input).unwrap();
            match mutation {
                0 => key.poses[1].translation[0] += 1,
                1 => key.revisions[0] ^= 1,
                2 => {
                    key.context.target = Some(PhysicalTarget::Pipe {
                        filament: 10,
                        chunk: 0,
                    })
                }
                3 => key.context.tick += 1,
                4 => key.parameters[0] ^= 1,
                5 => frontier.receipts.swap(0, 1),
                6 => frontier.receipts[0].as_mut().unwrap().worker = 99,
                7 => frontier.receipts[0].as_mut().unwrap().generation += 1,
                8 => key.component += 1,
                9 => key.radii[1] ^= 1,
                10 => key.context.pass += 1,
                11 => frontier.receipts[0].as_mut().unwrap().value = Ok(Value::Contact(None)),
                _ => unreachable!(),
            }
            assert!(
                executor.consume(&mut frontier, &key).is_none(),
                "mutation {mutation} admitted"
            );
            executor.finish(frontier);
        }
        assert_eq!(executor.profile.recalculated, 12);
    }
    #[test]
    fn persistent_command_queue_refuses_saturation_and_recovers() {
        let mut executor = Executor::new(policy()).unwrap();
        let park = Arc::new(std::sync::Barrier::new(2));
        let command = |park| Command {
            generation: 0,
            tasks: Arc::new(Vec::new()),
            next: Arc::new(AtomicUsize::new(0)),
            park,
        };
        let worker = &executor.workers[0];
        let send = worker.command.as_ref().unwrap();
        send.send(command(Some(park.clone()))).unwrap();
        park.wait();
        send.try_send(command(None)).unwrap();
        assert!(matches!(
            send.try_send(command(None)),
            Err(mpsc::TrySendError::Full(_))
        ));
        park.wait();
        worker.results.recv().unwrap();
        worker.results.recv().unwrap();
        let a = ball();
        let b = ball();
        let input = tasks(&a, &b, 2);
        let key = input[0].key();
        let mut frontier = executor.run(input).unwrap();
        assert!(executor.consume(&mut frontier, &key).is_some());
        executor.finish(frontier);
    }
    #[test]
    fn same_tick_pipe_mutation_cannot_reuse_a_worker_receipt() {
        use crate::filament::{Assembly, Filament, Node, PipeSection};
        let mut assembly = Assembly::default();
        assembly.filaments.insert(
            1,
            Filament {
                id: 1,
                organism: 1,
                nozzle: 1,
                created_tick: 0,
                experiment: "receipt mutation test".into(),
                material: crate::emission::MaterialKind::Secretion,
                quantity: 3,
                section: Some(PipeSection {
                    start_sealed: true,
                    end_sealed: false,
                    outer_radius: 50_000,
                    inner_radius: 31_000,
                }),
                nodes: (0..3)
                    .map(|i| Node {
                        id: i + 1,
                        parcel: i + 1,
                        deposited_tick: 0,
                        position: Position::from_absolute([0, 0, i as i64 * 100_000]).unwrap(),
                    })
                    .collect(),
            },
        );
        let mut cache = CollisionCache::default();
        let (_, old) = cache.pipe_in(&assembly, 2, 1, 0).unwrap();
        let body = ball();
        let mut executor = Executor::new(policy()).unwrap();
        let mut frontier = executor.run(tasks(&body, &old, 2)).unwrap();
        assembly.filaments.get_mut(&1).unwrap().nodes[1].position =
            Position::from_absolute([1, 0, 100_000]).unwrap();
        let (_, current) = cache.pipe_in(&assembly, 2, 1, 0).unwrap();
        let expected = tasks(&body, &current, 2)[0].key();
        assert!(executor.consume(&mut frontier, &expected).is_none());
        executor.finish(frontier);
    }
    #[test]
    fn unreachable_errors_worker_panics_capacity_and_shutdown_keep_canonical_fallback() {
        let mut cache = CollisionCache::default();
        cache.contact_workspace.executor = Some(Executor::new(policy()).unwrap());
        let mut a = ball();
        a.parts = vec![(0, ball()), (1, ball())];
        let b = ball();
        cache.contact_workspace.executor.as_mut().unwrap().fault = Some((1, 2));
        assert!(
            !cache
                .parallel_graft(&a, 99, &b, [RigidPose::identity(); 4])
                .unwrap()
        );
        assert_eq!(cache.execution_profile().unnecessary, 1);
        cache.contact_workspace.executor.as_mut().unwrap().fault = Some((0, 1));
        assert!(
            !cache
                .parallel_graft(&a, 99, &b, [RigidPose::identity(); 4])
                .unwrap()
        );
        assert!(cache.execution_profile().recalculated > 0);
        let executor = cache.contact_workspace.executor.as_mut().unwrap();
        executor.fault = None;
        assert!(executor.run(tasks(&a, &b, 65)).is_none());
        assert_eq!(executor.profile.capacity_bypass, 1);
        // A failed persistent port is drained/joined; missing receipts cause fallback.
        for worker in &mut executor.workers {
            worker.command.take();
        }
        for worker in &mut executor.workers {
            worker.handle.take().unwrap().join().unwrap();
        }
        assert!(
            !cache
                .parallel_graft(&a, 99, &b, [RigidPose::identity(); 4])
                .unwrap()
        );
        assert!(cache.execution_profile().failures >= 3);
    }
    #[test]
    fn recovery_and_runtime_events_match_sequential_execution() {
        let mut base =
            ShipRuntime::habitat_colony(AnatomySpec::showcase(11), 11, 120, 2, Some(4000)).unwrap();
        base.advance(24).unwrap();
        let initial = base.snapshot();
        let mut reference = ShipRuntime::restore(initial.clone()).unwrap();
        let mut expected = Vec::new();
        for _ in 0..12 {
            reference.tick().unwrap();
            expected.push(reference.snapshot());
        }
        for workers in [2, 4] {
            let mut parallel = ShipRuntime::restore(initial.clone()).unwrap();
            parallel
                .configure_collision_execution(ExecutionPolicy {
                    workers,
                    graft: cfg!(feature = "experimental-collision-frontiers"),
                    static_queries: cfg!(feature = "experimental-collision-frontiers"),
                    ..policy()
                })
                .unwrap();
            for state in &expected {
                parallel.tick().unwrap();
                assert_eq!(parallel.snapshot(), *state);
            }
            assert!(parallel.collision_execution_profile().dispatched > 0);
        }
        let mut rejected = initial.clone();
        rejected.config.construction.as_mut().unwrap().max_events = rejected.events.len();
        let mut world = ShipRuntime::restore(rejected.clone()).unwrap();
        world
            .configure_collision_execution(ExecutionPolicy {
                workers: 4,
                graft: cfg!(feature = "experimental-collision-frontiers"),
                static_queries: cfg!(feature = "experimental-collision-frontiers"),
                ..policy()
            })
            .unwrap();
        world
            .configure_geometry_reuse(cfg!(feature = "experimental-geometry-reuse"))
            .unwrap();
        assert!(
            world
                .tick()
                .unwrap_err()
                .to_string()
                .contains("HistoryBudget")
        );
        assert_eq!(world.snapshot(), rejected);
    }
}
