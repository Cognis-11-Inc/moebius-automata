//! Diagnostic recipes for authentic canonical body/pipe casts. No alternate solver.
use super::*;
use std::cell::RefCell;

#[path = "collision_batch.rs"]
pub mod batch;

#[derive(Serialize)]
pub struct GeometryRecord {
    stage: String,
    tick: u64,
    key: PipeCacheKey,
    geometry_blake3: String,
    vertices: usize,
    triangles: usize,
    constructor_requested_bytes: Option<u64>,
    paid_binding: Option<(u64, u64, u64)>,
}
thread_local! {
    static GEOMETRY_RECORDS: RefCell<Option<Vec<GeometryRecord>>> = const { RefCell::new(None) };
    static GEOMETRY_BYTES: Cell<Option<u64>> = const { Cell::new(None) };
}
pub fn capture_geometry() {
    GEOMETRY_RECORDS.with(|r| *r.borrow_mut() = Some(Vec::new()));
}
pub fn take_geometry() -> Vec<GeometryRecord> {
    GEOMETRY_RECORDS.with(|r| r.borrow_mut().take().unwrap_or_default())
}
#[cfg(feature = "allocation-attribution")]
pub(super) fn geometry_bytes(bytes: u64) {
    GEOMETRY_BYTES.with(|r| r.set(Some(bytes)));
}
pub(super) fn geometry_record(
    stage: &str,
    tick: u64,
    key: PipeCacheKey,
    g: &Geometry,
    built: bool,
    paid: Option<(u64, u64, u64)>,
) {
    GEOMETRY_RECORDS.with(|records| {
        if let Some(rows) = &mut *records.borrow_mut() {
            assert!(rows.len() < 8192, "geometry diagnostic budget exceeded");
            rows.push(GeometryRecord {
                stage: stage.into(),
                tick,
                key,
                geometry_blake3: geometry_hash(g),
                vertices: g.vertices.len(),
                triangles: g.shape.as_trimesh().map_or(0, |s| s.num_triangles()),
                constructor_requested_bytes: if built {
                    GEOMETRY_BYTES.with(Cell::get)
                } else {
                    Some(0)
                },
                paid_binding: paid,
            });
        }
    });
}

#[derive(Serialize)]
pub struct RecoveryRecord {
    pair: Option<(u64, u64, PhysicalTarget)>,
    geometry_blake3: [String; 2],
    poses: [RigidPose; 4],
    samples: Vec<(u32, Option<u32>, u128)>,
    elapsed_ns: u128,
}
thread_local! {
    static RECOVERY: RefCell<Option<Vec<RecoveryRecord>>> = const { RefCell::new(None) };
    static RECOVERY_PAIR: RefCell<Option<(u64, u64, PhysicalTarget)>> = const { RefCell::new(None) };
}
pub(super) fn recovery_pair(tick: u64, body: u64, owner: PhysicalTarget) {
    if RECOVERY.with(|r| r.borrow().is_some()) {
        RECOVERY_PAIR.with(|r| *r.borrow_mut() = Some((tick, body, owner)));
    }
}
/// Explicit diagnostic selection; feature availability alone does not record.
pub fn capture_recovery() {
    RECOVERY.with(|r| *r.borrow_mut() = Some(Vec::new()));
}
pub fn take_recovery() -> Vec<RecoveryRecord> {
    RECOVERY.with(|r| r.borrow_mut().take().unwrap_or_default())
}
pub(super) struct RecoveryCapture {
    record: Option<RecoveryRecord>,
    started: Instant,
}
impl RecoveryCapture {
    pub(super) fn new(a: &Geometry, b: &Geometry, poses: [RigidPose; 4]) -> Self {
        let enabled = RECOVERY.with(|r| r.borrow().is_some());
        let record = enabled.then(|| RecoveryRecord {
            pair: RECOVERY_PAIR.with(|r| r.borrow().clone()),
            geometry_blake3: [geometry_hash(a), geometry_hash(b)],
            poses,
            samples: Vec::new(),
            elapsed_ns: 0,
        });
        Self {
            record,
            started: Instant::now(),
        }
    }
    pub(super) fn sample(&mut self, t: f32, depth: Option<f32>, ns: u128) {
        if let Some(record) = &mut self.record {
            record
                .samples
                .push((t.to_bits(), depth.map(f32::to_bits), ns));
        }
    }
}
impl Drop for RecoveryCapture {
    fn drop(&mut self) {
        if let Some(mut record) = self.record.take() {
            record.elapsed_ns = self.started.elapsed().as_nanos();
            RECOVERY.with(|r| {
                if let Some(rows) = &mut *r.borrow_mut() {
                    // Explicit bounded diagnostic refusal, never silent truncation.
                    assert!(rows.len() < 8192, "recovery diagnostic budget exceeded");
                    rows.push(record);
                }
            });
        }
    }
}

fn geometry_hash(g: &Geometry) -> String {
    fn update(g: &Geometry, h: &mut blake3::Hasher) {
        h.update(&g.radius.to_bits().to_le_bytes());
        for point in g.vertices.iter() {
            for v in point.to_array() {
                h.update(&v.to_bits().to_le_bytes());
            }
        }
        if let Some(hull) = g.shape.as_convex_polyhedron() {
            for p in hull.points() {
                for v in p.to_array() {
                    h.update(&v.to_bits().to_le_bytes());
                }
            }
            for face in hull.faces() {
                for v in face.normal.to_array() {
                    h.update(&v.to_bits().to_le_bytes());
                }
            }
        }
        if let Some(mesh) = g.shape.as_trimesh() {
            for triangle in mesh.indices() {
                for index in triangle {
                    h.update(&index.to_le_bytes());
                }
            }
            h.update(&mesh.flags().bits().to_le_bytes());
        }
        for (component, part) in &g.parts {
            h.update(&component.to_le_bytes());
            update(part, h);
        }
    }
    let mut hash = blake3::Hasher::new();
    update(g, &mut hash);
    hash.finalize().to_hex().to_string()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CaptureRequest {
    /// Pre-step world tick (the reported resulting tick is one greater).
    pub tick: u64,
    pub body: Option<u64>,
    pub filament: Option<u64>,
    pub chunk: Option<usize>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ExactHit {
    pub toi_bits: u32,
    pub normal: [i64; 3],
    pub point: Position,
}
fn exact(hit: Hit) -> Option<ExactHit> {
    hit.map(|(toi, normal, point)| ExactHit {
        toi_bits: toi.to_bits(),
        normal,
        point,
    })
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PipeCastFixture {
    pub schema: u32,
    pub snapshot_digest: String,
    pub tick: u64,
    pub body: u64,
    pub filament: u64,
    pub chunk: usize,
    pub from: RigidPose,
    pub to: RigidPose,
    pub pipe_from: RigidPose,
    pub pipe_to: RigidPose,
    pub radius_bits: [u32; 2],
    pub geometry_blake3: [String; 2],
    pub pipe_revision: [u8; 32],
    pub reference_hit: Option<ExactHit>,
    pub reference_error: Option<String>,
    pub source_build_identity: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KernelWork {
    pub support_calls: u64,
    pub bounded_gjk_calls: u64,
    /// Paired support queries inside the GJK loop, excluding simplex initialization.
    pub bounded_gjk_support_steps: u64,
    pub bounded_gjk_max_steps: u64,
    pub bounded_gjk_100_step_calls: u64,
    /// Includes exact-distance simplex initialization; not mislabeled loop iterations.
    pub exact_support_pairs: u64,
}
#[cfg(feature = "kernel-attribution")]
thread_local! { static CONTACT_EPA: Cell<u64> = const { Cell::new(0) }; }
#[cfg(feature = "kernel-attribution")]
pub(super) fn note_contact_epa() {
    CONTACT_EPA.with(|n| n.set(n.get() + 1));
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sample {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work: Option<KernelWork>,
    pub phase: String,
    pub lo_bits: u32,
    pub hi_bits: u32,
    pub mid_bits: u32,
    pub limit_bits: u32,
    pub roundoff_bits: u32,
    pub within: bool,
    pub triangles: u64,
    pub bounded: u64,
    pub exact: u64,
    pub ns: u128,
}
pub(super) fn phase(name: &str, ns: u128, within: bool) -> Sample {
    Sample {
        work: None,
        phase: name.to_owned(),
        lo_bits: 0,
        hi_bits: 0,
        mid_bits: 0,
        limit_bits: 0,
        roundoff_bits: 0,
        within,
        triangles: 0,
        bounded: 0,
        exact: 0,
        ns,
    }
}
#[derive(Default)]
struct Observer {
    requests: Vec<CaptureRequest>,
    worlds: BTreeMap<u64, (Arc<WorldSnapshot>, String)>,
    records: Vec<(PipeCastFixture, Vec<Sample>)>,
    active: Option<Vec<Sample>>,
}
thread_local! { static OBSERVER: RefCell<Observer> = RefCell::new(Observer::default()); }
pub fn request(requests: Vec<CaptureRequest>) {
    OBSERVER.with(|o| {
        *o.borrow_mut() = Observer {
            requests,
            ..Observer::default()
        }
    });
}
pub(super) fn begin(
    world: &WorldSnapshot,
    body: u64,
    owner: &PhysicalTarget,
    poses: [RigidPose; 4],
    geometry: [&Geometry; 2],
) -> Option<PipeCastFixture> {
    let PhysicalTarget::Pipe { filament, chunk } = *owner else {
        return None;
    };
    OBSERVER.with(|o| {
        let mut o = o.borrow_mut();
        if !o.requests.iter().any(|r| {
            r.tick == world.tick
                && r.body.is_none_or(|id| id == body)
                && r.filament.is_none_or(|id| id == filament)
                && r.chunk.is_none_or(|id| id == chunk)
        }) {
            return None;
        }
        let revision = pipe_key(
            filament,
            &world.assembly.as_ref()?.filaments[&filament],
            chunk,
            world.tick,
        )?
        .2;
        let digest = o
            .worlds
            .entry(world.tick)
            .or_insert_with(|| {
                (
                    Arc::new(world.clone()),
                    world.digest().expect("valid captured state"),
                )
            })
            .1
            .clone();
        o.active = Some(vec![]);
        Some(PipeCastFixture {
            schema: 1,
            snapshot_digest: digest,
            tick: world.tick,
            body,
            filament,
            chunk,
            from: poses[0],
            to: poses[1],
            pipe_from: poses[2],
            pipe_to: poses[3],
            radius_bits: geometry.map(|g| g.radius.to_bits()),
            geometry_blake3: geometry.map(geometry_hash),
            pipe_revision: revision,
            reference_hit: None,
            reference_error: None,
            source_build_identity: crate::BENCHMARK_BUILD_IDENTITY.to_owned(),
        })
    })
}
pub(super) fn finish(mut pending: PipeCastFixture, result: &Result<Hit, MorphError>) {
    match result {
        Ok(hit) => pending.reference_hit = exact(*hit),
        Err(e) => pending.reference_error = Some(e.to_string()),
    }
    OBSERVER.with(|o| {
        let mut o = o.borrow_mut();
        let samples = o.active.take().unwrap_or_default();
        o.records.push((pending, samples));
    });
}
pub(super) fn sample(sample: impl FnOnce() -> Sample) {
    OBSERVER.with(|o| {
        if let Some(samples) = &mut o.borrow_mut().active {
            samples.push(sample());
        }
    });
}
pub fn write_captures(directory: &std::path::Path) -> Result<usize, Box<dyn std::error::Error>> {
    let mut observer = OBSERVER.with(|o| std::mem::take(&mut *o.borrow_mut()));
    std::fs::create_dir(directory)?;
    for (tick, (world, _)) in observer.worlds {
        world.save(&directory.join(format!("world-{tick}.json")))?;
    }
    for (i, (fixture, samples)) in observer.records.iter_mut().enumerate() {
        std::fs::write(
            directory.join(format!("fixture-{i}.json")),
            serde_json::to_vec_pretty(fixture)?,
        )?;
        std::fs::write(
            directory.join(format!("trace-{i}.json")),
            serde_json::to_vec_pretty(samples)?,
        )?;
    }
    Ok(observer.records.len())
}

#[derive(Debug, Serialize)]
pub struct Probe {
    pub hit: Option<ExactHit>,
    pub cold_geometry_ns: u128,
    pub query_ns: u128,
    pub profile: CollisionProfile,
    pub body_parts: usize,
    pub body_vertices: usize,
    pub hull_vertices: usize,
    pub hull_faces: usize,
    pub pipe_triangles: usize,
    pub trace: Vec<Sample>,
    pub custom_contact_epa_calls: Option<u64>,
    pub observer_work: Option<Vec<u64>>,
}
/// Reconstruct only the requested authentic colliders using the canonical cache.
/// Geometry preparation is outside query timing. Each invocation has a cold cache.
pub fn probe(
    world: &WorldSnapshot,
    fixture: &PipeCastFixture,
    trace: bool,
) -> Result<Probe, MorphError> {
    require(
        fixture.schema == 1
            && world.tick == fixture.tick
            && world.digest()? == fixture.snapshot_digest,
        "fixture snapshot identity mismatch",
    )?;
    let mut cache = CollisionCache::default();
    let started = Instant::now();
    let a = cache.body(world, fixture.body)?;
    let (_, b) = cache.pipe(world, fixture.filament, fixture.chunk)?;
    require(
        [a.radius.to_bits(), b.radius.to_bits()] == fixture.radius_bits,
        "fixture geometry radius mismatch",
    )?;
    require(
        [geometry_hash(&a), geometry_hash(&b)] == fixture.geometry_blake3,
        "fixture collider geometry mismatch",
    )?;
    let revision = pipe_key(
        fixture.filament,
        &world.assembly.as_ref().unwrap().filaments[&fixture.filament],
        fixture.chunk,
        world.tick,
    )
    .unwrap()
    .2;
    require(
        revision == fixture.pipe_revision,
        "fixture pipe revision mismatch",
    )?;
    let cold_geometry_ns = started.elapsed().as_nanos();
    OBSERVER.with(|o| o.borrow_mut().active = trace.then(Vec::new));
    #[cfg(feature = "kernel-attribution")]
    CONTACT_EPA.with(|n| n.set(0));
    #[cfg(feature = "allocation-attribution")]
    crate::profiling::reset();
    let started = Instant::now();
    let hit = cache.cast(
        &a,
        fixture.from,
        fixture.to,
        &b,
        fixture.pipe_from,
        fixture.pipe_to,
    )?;
    let query_ns = started.elapsed().as_nanos();
    let samples = OBSERVER.with(|o| o.borrow_mut().active.take().unwrap_or_default());
    require(
        fixture.reference_error.is_none() && exact(hit) == fixture.reference_hit,
        "isolated cast differs from authoritative captured result",
    )?;
    Ok(Probe {
        hit: exact(hit),
        cold_geometry_ns,
        query_ns,
        profile: cache.profile(),
        body_parts: a.parts.len(),
        body_vertices: a.vertices.len(),
        hull_vertices: a
            .parts
            .iter()
            .filter_map(|(_, g)| g.shape.as_convex_polyhedron())
            .map(|g| g.points().len())
            .sum(),
        hull_faces: a
            .parts
            .iter()
            .filter_map(|(_, g)| g.shape.as_convex_polyhedron())
            .map(|g| g.faces().len())
            .sum(),
        pipe_triangles: b.shape.as_trimesh().unwrap().num_triangles(),
        trace: samples,
        observer_work: {
            #[cfg(feature = "allocation-attribution")]
            {
                Some(crate::profiling::work_snapshot().to_vec())
            }
            #[cfg(not(feature = "allocation-attribution"))]
            {
                None
            }
        },
        custom_contact_epa_calls: {
            #[cfg(feature = "kernel-attribution")]
            {
                Some(CONTACT_EPA.with(Cell::get))
            }
            #[cfg(not(feature = "kernel-attribution"))]
            {
                None
            }
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn authentic_recipe_reproduces_captured_query_and_rejects_tampering() {
        request(vec![CaptureRequest {
            tick: 31,
            body: Some(12),
            filament: Some(6),
            chunk: Some(0),
        }]);
        let mut world =
            ShipRuntime::habitat_colony(AnatomySpec::showcase(11), 11, 120, 8, Some(4000)).unwrap();
        world.advance(32).unwrap();
        let (snapshot, fixture) = OBSERVER.with(|o| {
            let mut o = o.borrow_mut();
            assert_eq!(o.records.len(), 1);
            let snapshot = o.worlds[&31].0.clone();
            let fixture = o.records.pop().unwrap().0;
            (snapshot, fixture)
        });
        let result = probe(&snapshot, &fixture, true).unwrap();
        assert_eq!(result.hit, fixture.reference_hit);
        let mut wrong = fixture.clone();
        wrong.radius_bits[0] ^= 1;
        assert!(
            probe(&snapshot, &wrong, false)
                .unwrap_err()
                .to_string()
                .contains("radius")
        );
        let mut wrong = fixture;
        wrong.snapshot_digest = "wrong".into();
        assert!(
            probe(&snapshot, &wrong, false)
                .unwrap_err()
                .to_string()
                .contains("identity")
        );
        request(vec![]);
    }
}
