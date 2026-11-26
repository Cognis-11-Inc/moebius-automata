//! Isolated throughput assay over authenticated immutable query descriptors.
//! Results have no admission path into ShipRuntime. Captured queries may belong
//! to different causal solver passes: batch throughput is not tick speedup.
use super::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    mpsc,
};

struct Query {
    a: Arc<Geometry>,
    b: Geometry,
    fixture: PipeCastFixture,
}

pub struct Batch {
    queries: Vec<Query>,
    pub cold_geometry_ns: u128,
}

#[derive(Debug, Serialize)]
pub struct WorkerRow {
    pub queries: usize,
    /// Elapsed time inside canonical cast calls, not OS CPU time.
    pub query_ns: u128,
}

#[derive(Debug, Serialize)]
pub struct Round {
    pub wall_ns: u128,
    pub workers: Vec<WorkerRow>,
    pub ordered_hits_blake3: String,
    pub exact: bool,
}

impl Batch {
    /// Preserve input order; reject changed snapshots, colliders and revisions.
    /// A single canonical cache prepares shared immutable geometry once.
    pub fn prepare(
        world: &WorldSnapshot,
        fixtures: &[PipeCastFixture],
    ) -> Result<Self, MorphError> {
        require(
            !fixtures.is_empty() && fixtures.len() <= 8192,
            "batch query budget",
        )?;
        let digest = world.digest()?;
        let mut cache = CollisionCache::default();
        let mut queries = Vec::with_capacity(fixtures.len());
        let started = Instant::now();
        for f in fixtures {
            require(
                f.schema == 1 && world.tick == f.tick && digest == f.snapshot_digest,
                "fixture snapshot identity mismatch",
            )?;
            let a = cache.body(world, f.body)?;
            let (_, b) = cache.pipe(world, f.filament, f.chunk)?;
            require(
                [a.radius.to_bits(), b.radius.to_bits()] == f.radius_bits,
                "fixture geometry radius mismatch",
            )?;
            require(
                [geometry_hash(&a), geometry_hash(&b)] == f.geometry_blake3,
                "fixture collider geometry mismatch",
            )?;
            let revision = pipe_key(
                f.filament,
                &world.assembly.as_ref().unwrap().filaments[&f.filament],
                f.chunk,
                world.tick,
            )
            .ok_or_else(|| MorphError("missing pipe revision".into()))?
            .2;
            require(
                revision == f.pipe_revision,
                "fixture pipe revision mismatch",
            )?;
            require(
                f.reference_error.is_none(),
                "fixture contains reference error",
            )?;
            queries.push(Query {
                a,
                b,
                fixture: f.clone(),
            });
        }
        Ok(Self {
            queries,
            cold_geometry_ns: started.elapsed().as_nanos(),
        })
    }

    /// One bounded, explicitly sized scoped pool lives for all rounds. Workers
    /// receive only immutable geometry/poses and own all numerical workspaces.
    /// Atomic dispatch changes scheduling only; no floating-point reduction.
    pub fn run(&self, workers: usize, repeats: usize) -> Result<Vec<Round>, MorphError> {
        require(
            (1..=32).contains(&workers) && (1..=1000).contains(&repeats),
            "batch worker/repeat budget",
        )?;
        let next = AtomicUsize::new(0);
        std::thread::scope(|scope| {
            let mut commands = Vec::new();
            let mut outputs = Vec::new();
            for _ in 0..workers {
                let (command, receive) = mpsc::sync_channel::<()>(1);
                let (send, output) = mpsc::sync_channel(1);
                let next = &next;
                scope.spawn(move || {
                    let mut workspace = ContactWorkspace::default();
                    while receive.recv().is_ok() {
                        let mut hits = Vec::new();
                        let mut query_ns = 0;
                        loop {
                            let index = next.fetch_add(1, Ordering::Relaxed);
                            let Some(q) = self.queries.get(index) else {
                                break;
                            };
                            // Do not turn repeated rounds or captured duplicate
                            // queries into memo hits. Reuse scratch capacities only.
                            workspace.contacts.clear();
                            let mut profile = CollisionProfile::default();
                            let f = &q.fixture;
                            let started = Instant::now();
                            let hit = cast(
                                &q.a,
                                f.from,
                                f.to,
                                &q.b,
                                f.pipe_from,
                                f.pipe_to,
                                (&mut profile, &mut workspace),
                            )
                            .map(exact);
                            query_ns += started.elapsed().as_nanos();
                            hits.push((index, hit));
                        }
                        if send.send((hits, query_ns)).is_err() {
                            break;
                        }
                    }
                });
                commands.push(command);
                outputs.push(output);
            }
            let mut rounds = Vec::with_capacity(repeats);
            for _ in 0..repeats {
                next.store(0, Ordering::Relaxed);
                let started = Instant::now();
                for command in &commands {
                    command
                        .send(())
                        .map_err(|_| MorphError("batch worker stopped".into()))?;
                }
                let mut ordered = vec![None; self.queries.len()];
                let mut rows = Vec::with_capacity(workers);
                for output in &outputs {
                    let (hits, query_ns) = output
                        .recv()
                        .map_err(|_| MorphError("batch worker stopped".into()))?;
                    rows.push(WorkerRow {
                        queries: hits.len(),
                        query_ns,
                    });
                    for (index, hit) in hits {
                        ordered[index] = Some(hit);
                    }
                }
                let wall_ns = started.elapsed().as_nanos();
                let mut hits = Vec::with_capacity(ordered.len());
                for (q, hit) in self.queries.iter().zip(ordered) {
                    let hit = hit.ok_or_else(|| MorphError("batch result missing".into()))??;
                    require(
                        hit == q.fixture.reference_hit,
                        "batch cast differs from authoritative captured result",
                    )?;
                    hits.push(hit);
                }
                let bytes = serde_json::to_vec(&hits).map_err(|e| MorphError(e.to_string()))?;
                rounds.push(Round {
                    wall_ns,
                    workers: rows,
                    ordered_hits_blake3: blake3::hash(&bytes).to_hex().to_string(),
                    exact: true,
                });
            }
            // Drop commands on both success and error before scope joins workers.
            drop(commands);
            Ok(rounds)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parallel_captured_queries_are_exact_and_reject_stale_recipes() {
        request(vec![CaptureRequest {
            tick: 31,
            body: None,
            filament: None,
            chunk: None,
        }]);
        let mut world =
            ShipRuntime::habitat_colony(AnatomySpec::showcase(11), 11, 120, 8, Some(4000)).unwrap();
        world.advance(32).unwrap();
        let (snapshot, fixtures) = OBSERVER.with(|o| {
            let o = o.borrow();
            (
                o.worlds[&31].0.clone(),
                o.records.iter().map(|r| r.0.clone()).collect::<Vec<_>>(),
            )
        });
        assert!(!fixtures.is_empty());
        let batch = Batch::prepare(&snapshot, &fixtures).unwrap();
        let expected = batch.run(1, 2).unwrap()[0].ordered_hits_blake3.clone();
        for workers in [2, 4] {
            for row in batch.run(workers, 2).unwrap() {
                assert_eq!(row.ordered_hits_blake3, expected);
                assert_eq!(
                    row.workers.iter().map(|w| w.queries).sum::<usize>(),
                    fixtures.len()
                );
            }
        }
        let mut wrong = fixtures.clone();
        wrong[0].pipe_revision[0] ^= 1;
        assert!(
            Batch::prepare(&snapshot, &wrong)
                .err()
                .unwrap()
                .to_string()
                .contains("revision")
        );
        let mut wrong = fixtures.clone();
        wrong[0].snapshot_digest = "obsolete".into();
        assert!(
            Batch::prepare(&snapshot, &wrong)
                .err()
                .unwrap()
                .to_string()
                .contains("identity")
        );
        let mut wrong = fixtures;
        wrong[0].reference_hit = None;
        // Ensure tampering changes the expected result even for a no-hit recipe.
        if batch.queries[0].fixture.reference_hit.is_none() {
            wrong[0].reference_hit = Some(ExactHit {
                toi_bits: 0,
                normal: [0; 3],
                point: snapshot.instances[&wrong[0].body].position,
            });
        }
        assert!(
            Batch::prepare(&snapshot, &wrong)
                .unwrap()
                .run(2, 1)
                .unwrap_err()
                .to_string()
                .contains("differs")
        );
        assert!(batch.run(0, 1).is_err());
        assert!(batch.run(33, 1).is_err());
        request(vec![]);
    }
}
