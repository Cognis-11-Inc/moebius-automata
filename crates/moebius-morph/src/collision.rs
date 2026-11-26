//! Canonical geometry queries and bounded kinematic contact admission.
//! The library supplies detection; ShipRuntime owns response and contact observations.
use crate::{
    emission::{RigidPose, SCALE},
    filament::{CHUNK_EDGES, filament_chunk_at},
    habitat::Agent,
    runtime::*,
    *,
};
use parry3d::query::epa::EPA;
use parry3d::query::gjk::{self, CsoPoint, GJKResult, VoronoiSimplex};
use parry3d::{
    bounding_volume::{Aabb, BoundingVolume},
    math::{Pose as Iso, Rot3, Vector},
    query::{
        self, DefaultQueryDispatcher, NonlinearRigidMotion, QueryDispatcher, ShapeCastOptions,
        ShapeCastStatus,
    },
    shape::{
        Ball, Capsule, Compound, ConvexPolyhedron, Cuboid, Shape, SharedShape, TriMesh,
        TriMeshFlags, TypedCompositeShape,
    },
    utils::PoseOpt,
};
use serde::{Deserialize, Serialize};
use std::{
    cell::Cell,
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Mutex, OnceLock},
    time::Instant,
};

#[cfg(feature = "collision-fixture")]
#[path = "collision_fixtures.rs"]
pub mod fixtures;

#[path = "collision_execution.rs"]
mod execution;
pub use execution::{ExecutionPolicy, ExecutionProfile};

/// Tight sample admitted into the cached hit volume. The witness writhe uses the
/// same anatomy generator at a larger bend and a traveling phase.
pub const COLLISION_BEND: f32 = 0.12;
/// Authored presentation bend. Propulsion does not read this value.
pub const PRESENTATION_BEND: f32 = 0.72;

/// Deformation of the cached convex hull. One sample per definition, not per tick.
pub fn collision_pose(spec: &AnatomySpec) -> Pose {
    Pose {
        phase: 0.,
        bend: COLLISION_BEND,
        mouth: spec.body.mouth.aperture,
        flagella: 1.,
    }
}

/// Visual pose for a habitat body. Same generator as the hull, different sample.
pub fn habitat_presentation_pose(spec: &AnatomySpec, tick: u64, eating: bool) -> Pose {
    let mut pose = crate::filament::construction_pose(tick);
    pose.bend = PRESENTATION_BEND;
    pose.flagella = 1.;
    pose.mouth = spec.body.mouth.aperture;
    if eating {
        pose.mouth = (pose.mouth + 0.25).min(1.);
    }
    pose
}

fn slack_cache() -> &'static Mutex<BTreeMap<String, f32>> {
    static CACHE: OnceLock<Mutex<BTreeMap<String, f32>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(BTreeMap::new()))
}

/// Metres of presentation surface outside the tight hull.
/// Eight phases across one wave, both mouth states, plus 3 cm for the ticks
/// between samples. Cached per definition. Solids grow by this clearance.
/// Pipes, prey, and other builders stay on the tight hull.
pub fn presentation_slack(spec: &AnatomySpec) -> Result<f32, MorphError> {
    let key = blake3::hash(&serde_json::to_vec(spec).map_err(|e| MorphError(e.to_string()))?)
        .to_hex()
        .to_string();
    let mut cache = slack_cache()
        .lock()
        .map_err(|_| MorphError("writhe clearance cache poisoned".into()))?;
    if let Some(slack) = cache.get(&key).copied() {
        return Ok(slack);
    }
    let compiled = CompiledAnatomy::new(spec.clone())?;
    let hull = compiled.pose(collision_pose(spec))?;
    let mut shapes = Vec::new();
    for surface in &hull.geometry.surfaces {
        let points = surface
            .positions
            .iter()
            .map(|p| Vector::from_array(*p))
            .collect::<Vec<_>>();
        let shape: Arc<dyn Shape> = if let Some(hull) = ConvexPolyhedron::from_convex_hull(&points)
        {
            Arc::new(hull)
        } else {
            let min = points
                .iter()
                .copied()
                .reduce(Vector::min)
                .ok_or_else(|| MorphError("empty body envelope".into()))?;
            let max = points.iter().copied().reduce(Vector::max).unwrap();
            let (mut corners, triangles) =
                Cuboid::new((max - min) * 0.5 + Vector::splat(0.0001)).to_trimesh();
            corners.iter_mut().for_each(|p| *p += (min + max) * 0.5);
            Arc::new(
                ConvexPolyhedron::from_convex_mesh(corners, &triangles)
                    .ok_or_else(|| MorphError("invalid fallback body envelope".into()))?,
            )
        };
        shapes.push(shape);
    }
    let mut slack = 0f32;
    for tick in (0..160).step_by(20) {
        for eating in [false, true] {
            let shown = compiled.pose(habitat_presentation_pose(spec, tick, eating))?;
            for (index, surface) in shown.geometry.surfaces.iter().enumerate() {
                let shape = &shapes[index];
                for p in &surface.positions {
                    let projected =
                        shape.project_point(&Iso::IDENTITY, Vector::from_array(*p), true);
                    let distance = if projected.is_inside {
                        0.
                    } else {
                        (projected.point - Vector::from_array(*p)).length()
                    };
                    slack = slack.max(distance);
                }
            }
        }
    }
    let slack = slack + 0.03;
    cache.insert(key, slack);
    Ok(slack)
}

/// Counters for one runtime. They are not world state and do not enter digests.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct CollisionProfile {
    pub resolve_calls: u64,
    pub resolve_ns: u128,
    pub queries: u64,
    pub query_ns: u128,
    /// Intervals rejected by a conservative AABB distance lower bound.
    pub interval_rejects: u64,
    /// Distance-predicate searches; queries also counts bound-only visits.
    /// A search can inspect multiple leaves or finish on a certified bound.
    pub distance_queries: u64,
    /// Searches whose certified distance was inside the Lipschitz limit.
    pub distance_within: u64,
    /// Searches certified outside that limit. The interval was discarded.
    pub distance_clear: u64,
    pub distance_ns: u128,
    pub contact_ns: u128,
    pub cast_cache_hits: u64,
    pub contact_cache_hits: u64,
    pub contact_cache_misses: u64,
    pub linear_casts: u64,
    /// Complete library cast_shapes call, excluded from distance_ns.
    pub linear_cast_ns: u128,
    pub linear_nodes: u64,
    pub linear_node_rejects: u64,
    pub linear_leaf_dispatches: u64,
    pub mesh_interval_rejects: u64,
    pub body_builds: u64,
    pub body_hits: u64,
    pub body_geometry_ns: u128,
    pub pipe_builds: u64,
    pub pipe_hits: u64,
    pub pipe_geometry_ns: u128,
    /// Chunks whose sample sphere can meet the query. A candidate is not a contact.
    /// The hollow trimesh remains the authority.
    pub pipe_candidates: u64,
    /// Chunks rejected by the sample sphere before a mesh build or a Parry query.
    pub pipe_rejects: u64,
    /// Leaf triangles admitted by a mesh or compound BVH during distance searches.
    pub triangles_examined: u64,
    /// Distance searches answered by a separating plane or a certified GJK bound.
    pub distance_bounded: u64,
    /// Calls to the exact distance kernel. A bounded leaf does not increment this.
    pub distance_exact: u64,
    pub distance_nodes: u64,
    pub distance_node_rejects: u64,
    /// Chunks excluded by the collision horizon before the sample-sphere test.
    pub pipe_horizon_rejects: u64,
    /// Chunks whose horizon cell still required the authoritative sample sphere.
    pub pipe_horizon_candidates: u64,
    /// Live horizon spheres after the latest synchronization. Not a cumulative counter.
    pub pipe_horizon_indexed: u64,
    pub pipe_horizon_rebuilds: u64,
    pub pipe_horizon_ns: u128,
    /// Sample-sphere bounds computed while rebuilding the horizon.
    pub pipe_bounds_computed: u64,
    pub pipe_filament_stamps: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum PhysicalTarget {
    Organism { id: u64 },
    Pipe { filament: u64, chunk: usize },
    Solid { id: u64 },
    Root { sac: u64 },
    Parcel { organism: u64, parcel: u64 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContactResponse {
    Blocked,
    FeedingContact,
    SourceAttachment,
    SecretionBlocked,
    AttachmentRejected,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContactObservation {
    pub at: u64,
    /// Obstruction of an attempted path; this is not a claim of committed tactile penetration.
    pub predicted: bool,
    pub organism: u64,
    pub target: PhysicalTarget,
    pub point: Position,
    pub normal: [i64; 3],
    pub response: ContactResponse,
    pub toi_millionths: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SolidBox {
    pub id: u64,
    pub min: Position,
    pub max: Position,
}
#[derive(Clone)]
struct Geometry {
    shape: Arc<dyn Shape>,
    radius: f32,
    vertices: Arc<Vec<Vector>>,
    parts: Vec<(u64, Geometry)>,
    outlets: Vec<(u64, [f32; 3])>,
    /// Euclidean room between this hull and a presentation vertex, in metres.
    /// Solids are grown by this amount. Pipes and prey keep the tight hull.
    writhe_slack: OnceLock<f32>,
}
/// Filament, chunk, and digest of the geometry inputs including wet mesh time.
type PipeCacheKey = (u64, usize, [u8; 32]);
#[derive(Default)]
pub struct CollisionCache {
    geometry_reuse: bool,
    pending_geometry: Option<DepositGeometry>,
    promoted_geometry: BTreeMap<(u64, usize), [u8; 32]>,
    geometry_reuse_profile: GeometryReuseProfile,
    contact_workspace: ContactWorkspace,
    definitions: BTreeMap<(u64, u64, [u8; 32]), Arc<Geometry>>,
    specifications: BTreeMap<u64, (AnatomySpec, [u8; 32])>,
    /// Age stays in the key until the newest sample in the chunk is dry.
    /// Nodes past the one-neighbor lookahead do not.
    pipes: BTreeMap<(u64, usize), ([u8; 32], Position, Geometry)>,
    profile: CollisionProfile,
    casts: BTreeMap<CastKey, CastReceipt>,
    /// Collision-owned conservative pipe horizon. It is not serialized, not a
    /// witness index, and not a second geometric authority: a miss falls
    /// through to `pipe_separated` and the hollow trimesh.
    horizon: PipeHorizon,
}
#[derive(Clone, Debug, Default, Serialize)]
pub struct GeometryReuseProfile {
    pub candidate_builds: u64,
    pub promoted: u64,
    pub consumed: u64,
    pub mismatched: u64,
    pub rejected: u64,
}
struct DepositGeometry {
    parcel: u64,
    organism: u64,
    chunks: Vec<(PipeCacheKey, Position, Geometry)>,
}
type Hit = Option<(f32, [i64; 3], Position)>;
type CastKey = (usize, u32, usize, u32, [i64; 24]);
fn cast_key(a: &Geometry, b: &Geometry, endpoints: [RigidPose; 4]) -> CastKey {
    let mut poses = [0; 24];
    for (values, p) in poses.as_chunks_mut::<6>().0.iter_mut().zip(endpoints) {
        values[..3].copy_from_slice(&p.translation);
        values[3..].copy_from_slice(&[p.yaw_sin, p.yaw_cos, p.axial_roll]);
    }
    (
        Arc::as_ptr(&a.shape) as *const () as usize,
        a.radius.to_bits(),
        Arc::as_ptr(&b.shape) as *const () as usize,
        b.radius.to_bits(),
        poses,
    )
}
struct CastReceipt {
    // Keep both allocations alive: an address must never alias a new shape.
    _shapes: [Arc<dyn Shape>; 2],
    hit: Hit,
}
impl CollisionCache {
    pub fn configure_geometry_reuse(&mut self, enabled: bool) {
        self.geometry_reuse = enabled;
        self.pending_geometry = None;
    }
    pub fn geometry_reuse_profile(&self) -> GeometryReuseProfile {
        self.geometry_reuse_profile.clone()
    }
    pub(crate) fn discard_deposit_geometry(&mut self) {
        if self.pending_geometry.take().is_some() {
            self.geometry_reuse_profile.rejected += 1;
        }
    }
    pub(crate) fn admit_deposit_geometry(
        &mut self,
        assembly: &crate::filament::Assembly,
        book: &crate::emission::MaterialBook,
        tick: u64,
        admitted: (u64, u64, u64),
    ) {
        let Some(candidate) = self.pending_geometry.take() else {
            return;
        };
        let (filament, node, parcel) = admitted;
        let paid = candidate.parcel == parcel
            && candidate.organism == book.organism_id
            && book
                .bound
                .iter()
                .any(|b| b.filament == filament && b.node == node && b.parcel.id == parcel)
            && assembly.filaments.get(&filament).is_some_and(|f| {
                f.organism == book.organism_id
                    && f.nodes.iter().any(|n| n.id == node && n.parcel == parcel)
            });
        if !paid {
            self.geometry_reuse_profile.rejected += 1;
            return;
        }
        for (key, origin, g) in candidate.chunks {
            if assembly
                .filaments
                .get(&filament)
                .and_then(|f| pipe_key(filament, f, key.1, tick))
                == Some(key)
                && self.promoted_geometry.len() < 128
            {
                #[cfg(feature = "collision-fixture")]
                fixtures::geometry_record(
                    "promoted",
                    tick,
                    key,
                    &g,
                    false,
                    Some((filament, node, parcel)),
                );
                self.pipes.insert((key.0, key.1), (key.2, origin, g));
                self.promoted_geometry.insert((key.0, key.1), key.2);
                self.geometry_reuse_profile.promoted += 1;
            } else {
                self.geometry_reuse_profile.mismatched += 1;
            }
        }
    }
    fn parallel_graft(
        &mut self,
        body: &Geometry,
        nozzle: u64,
        obstacle: &Geometry,
        poses: [RigidPose; 4],
    ) -> Result<bool, MorphError> {
        let parts = body
            .parts
            .iter()
            .filter(|(id, _)| *id != nozzle)
            .collect::<Vec<_>>();
        let tasks = parts
            .iter()
            .enumerate()
            .map(|(ordinal, (component, part))| {
                execution::Task::cast(
                    self.contact_workspace.context.clone(),
                    ordinal,
                    *component,
                    part,
                    obstacle,
                    poses,
                )
            })
            .collect();
        let mut frontier = self.contact_workspace.executor.as_mut().unwrap().run(tasks);
        let mut result = Ok(true);
        for (ordinal, (component, part)) in parts.into_iter().enumerate() {
            let ready = frontier.as_mut().and_then(|f| {
                let key = execution::Task::cast(
                    self.contact_workspace.context.clone(),
                    ordinal,
                    *component,
                    part,
                    obstacle,
                    poses,
                )
                .key();
                self.contact_workspace
                    .executor
                    .as_mut()
                    .unwrap()
                    .consume(f, &key)
            });
            let fallback = ready.is_none();
            let started = Instant::now();
            let hit = match ready {
                Some(Ok(execution::Value::Cast(hit))) => Ok(hit),
                Some(Err(error)) => Err(error),
                _ => self.cast(part, poses[0], poses[1], obstacle, poses[2], poses[3]),
            };
            if fallback {
                self.contact_workspace
                    .executor
                    .as_mut()
                    .unwrap()
                    .profile
                    .fallback_ns += started.elapsed().as_nanos();
            }
            match hit {
                Ok(Some(_)) => {
                    result = Ok(false);
                    break;
                }
                Err(error) => {
                    result = Err(error);
                    break;
                }
                Ok(None) => {}
            }
        }
        if let Some(frontier) = frontier {
            self.contact_workspace
                .executor
                .as_mut()
                .unwrap()
                .finish(frontier);
        }
        result
    }
    pub fn configure_execution(&mut self, policy: ExecutionPolicy) -> Result<(), MorphError> {
        require(
            cfg!(feature = "experimental-collision-frontiers")
                || (!policy.graft && !policy.static_queries),
            "unqualified collision frontiers require explicit experimental feature",
        )?;
        require(
            (1..=32).contains(&policy.workers) && (2..=64).contains(&policy.minimum_frontier),
            "invalid collision executor policy",
        )?;
        require(
            (2..=64).contains(&policy.frontier_limit),
            "invalid frontier capacity",
        )?;
        let executor =
            if policy.workers > 1 && (policy.graft || policy.recovery || policy.static_queries) {
                Some(execution::Executor::new(policy)?)
            } else {
                None
            };
        self.contact_workspace.executor = executor;
        Ok(())
    }
    pub fn execution_profile(&self) -> ExecutionProfile {
        self.contact_workspace
            .executor
            .as_ref()
            .map_or_else(ExecutionProfile::default, |e| e.profile.clone())
    }
    pub fn execution_policy(&self) -> ExecutionPolicy {
        self.contact_workspace
            .executor
            .as_ref()
            .map_or_else(ExecutionPolicy::default, |e| e.policy)
    }
    fn cast(
        &mut self,
        a: &Geometry,
        from: RigidPose,
        to: RigidPose,
        b: &Geometry,
        bfrom: RigidPose,
        bto: RigidPose,
    ) -> Result<Hit, MorphError> {
        let key = cast_key(a, b, [from, to, bfrom, bto]);
        if let Some(receipt) = self.casts.get(&key) {
            self.profile.cast_cache_hits += 1;
            #[cfg(feature = "query-trace")]
            query_trace::note_cache_hit();
            return Ok(receipt.hit);
        }
        #[cfg(feature = "query-trace")]
        query_trace::note_cast(query_trace::pose_fingerprint(&key.4));
        #[cfg(feature = "query-trace")]
        let cast_started = Instant::now();
        let cast_result = cast(
            a,
            from,
            to,
            b,
            bfrom,
            bto,
            (&mut self.profile, &mut self.contact_workspace),
        );
        #[cfg(feature = "query-trace")]
        query_trace::note_phase("cast", cast_started.elapsed().as_nanos());
        let hit = cast_result?;
        #[cfg(feature = "query-trace")]
        query_trace::clear_label();
        if self.casts.len() < 8192 {
            self.casts.insert(
                key,
                CastReceipt {
                    _shapes: [a.shape.clone(), b.shape.clone()],
                    hit,
                },
            );
        }
        Ok(hit)
    }
    pub fn profile(&self) -> CollisionProfile {
        CollisionProfile {
            contact_cache_hits: self.contact_workspace.hits,
            contact_cache_misses: self.contact_workspace.misses,
            ..self.profile
        }
    }
}
fn vec(p: [i64; 3]) -> Vector {
    Vector::from_array(p.map(|x| x as f32 / SCALE as f32))
}
fn rotation(p: RigidPose) -> Rot3 {
    Rot3::from_rotation_y((p.yaw_sin as f32).atan2(p.yaw_cos as f32))
        * Rot3::from_rotation_z(p.axial_roll as f32 / 1e6)
}
fn motion(
    from: RigidPose,
    to: RigidPose,
    origin: Position,
) -> Result<NonlinearRigidMotion, MorphError> {
    let start = Iso::from_parts(
        vec(Position::from_absolute(from.translation)?.relative(origin)?),
        rotation(from),
    );
    let delta = rotation(to) * start.rotation.inverse();
    let (axis, angle) = delta.to_axis_angle();
    let velocity = vec(Position::from_absolute(to.translation)?
        .relative(Position::from_absolute(from.translation)?)?);
    Ok(NonlinearRigidMotion::new(
        start,
        Vector::ZERO,
        velocity,
        axis * angle,
    ))
}
fn geometry(surfaces: &[Surface], scale: f32) -> Result<Geometry, MorphError> {
    #[cfg(feature = "allocation-attribution")]
    let _allocation_scope = crate::profiling::enter(14);
    #[cfg(all(feature = "allocation-attribution", feature = "collision-fixture"))]
    let allocation_before = crate::profiling::snapshot()[14][5];

    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for surface in surfaces {
        let base = vertices.len() as u32;
        vertices.extend(
            surface
                .positions
                .iter()
                .map(|p| Vector::from_array(*p) * scale),
        );
        indices.extend(
            surface
                .indices
                .as_chunks::<3>()
                .0
                .iter()
                .map(|p| [base + p[0], base + p[1], base + p[2]]),
        );
    }
    let radius = vertices.iter().map(|p| p.length()).fold(0f32, f32::max);
    let mesh = TriMesh::with_flags(vertices.clone(), indices, TriMeshFlags::ORIENTED)
        .map_err(|e| MorphError(format!("physical mesh rejected: {e}")))?;
    let result = Geometry {
        shape: Arc::new(mesh),
        radius,
        vertices: Arc::new(vertices),
        parts: vec![],
        outlets: vec![],
        writhe_slack: OnceLock::from(0.),
    };
    #[cfg(all(feature = "allocation-attribution", feature = "collision-fixture"))]
    fixtures::geometry_bytes(crate::profiling::snapshot()[14][5] - allocation_before);
    Ok(result)
}
impl CollisionCache {
    fn body(&mut self, world: &WorldSnapshot, id: u64) -> Result<Arc<Geometry>, MorphError> {
        let body = &world.instances[&id];
        let spec = &world.config.definitions[&body.definition];
        let quantity = if body.definition == BLOOD_SAC {
            body.book.reservoir.max(64).div_ceil(64).saturating_mul(64)
        } else {
            0
        };
        // Definitions are explicit inputs. Reuse their hash only while their
        // value is unchanged; a restored/edited definition must invalidate it.
        if self
            .specifications
            .get(&body.definition)
            .is_none_or(|(saved, _)| saved != spec)
        {
            let hash =
                *blake3::hash(&serde_json::to_vec(spec).map_err(|e| MorphError(e.to_string()))?)
                    .as_bytes();
            self.specifications
                .insert(body.definition, (spec.clone(), hash));
        }
        let key = (
            body.definition,
            quantity,
            self.specifications[&body.definition].1,
        );
        if let Some(g) = self.definitions.get(&key) {
            self.profile.body_hits += 1;
            return Ok(g.clone());
        }
        let built = Instant::now();
        self.profile.body_builds += 1;
        let pose = if body.definition == BLOOD_SAC {
            Pose::rest(&spec.body)
        } else {
            collision_pose(spec)
        };
        let posed = CompiledAnatomy::new(spec.clone())?.pose(pose)?;
        let scale = if body.definition == BLOOD_SAC {
            0.1 + 0.035 * (quantity as f32).cbrt()
        } else {
            1.
        };
        let mut parts = vec![];
        let mut vertices = vec![];
        for (index, surface) in posed.geometry.surfaces.iter().enumerate() {
            let points = surface
                .positions
                .iter()
                .map(|p| Vector::from_array(*p) * scale)
                .collect::<Vec<_>>();
            let shape: Arc<dyn Shape> =
                if let Some(hull) = ConvexPolyhedron::from_convex_hull(&points) {
                    Arc::new(hull)
                } else {
                    // Numerical hull failure must not omit a physical component.
                    let min = points
                        .iter()
                        .copied()
                        .reduce(Vector::min)
                        .ok_or_else(|| MorphError("empty body envelope".into()))?;
                    let max = points.iter().copied().reduce(Vector::max).unwrap();
                    let (mut corners, triangles) =
                        Cuboid::new((max - min) * 0.5 + Vector::splat(0.0001)).to_trimesh();
                    corners.iter_mut().for_each(|p| *p += (min + max) * 0.5);
                    Arc::new(
                        ConvexPolyhedron::from_convex_mesh(corners, &triangles)
                            .ok_or_else(|| MorphError("invalid fallback body envelope".into()))?,
                    )
                };
            let bounds = shape.compute_local_aabb();
            let radius = bounds.mins.abs().max(bounds.maxs.abs()).length();
            let component = if index == 0 {
                0
            } else {
                spec.components[index - 1].id
            };
            vertices.extend(points.iter().copied());
            parts.push((
                component,
                Geometry {
                    shape,
                    radius,
                    vertices: Arc::new(points),
                    parts: vec![],
                    outlets: vec![],
                    writhe_slack: OnceLock::from(0.),
                },
            ));
        }
        let shape = Compound::new(
            parts
                .iter()
                .map(|(_, part)| (Iso::IDENTITY, SharedShape(part.shape.clone())))
                .collect(),
        );
        // Only solid collision consumes presentation clearance. Deferral keeps
        // the exact value and ties its lifetime to the immutable body geometry.
        let writhe_slack = if body.definition == BLOOD_SAC {
            OnceLock::from(0.)
        } else {
            OnceLock::new()
        };
        let g = Geometry {
            shape: Arc::new(shape),
            radius: parts.iter().map(|(_, p)| p.radius).fold(0f32, f32::max),
            vertices: Arc::new(vertices),
            parts,
            outlets: posed
                .outputs
                .iter()
                .map(|o| (o.component_id, o.position))
                .collect(),
            writhe_slack,
        };
        self.profile.body_geometry_ns += built.elapsed().as_nanos();
        let g = Arc::new(g);
        self.definitions.insert(key, g.clone());
        Ok(g)
    }
    fn pipe(
        &mut self,
        world: &WorldSnapshot,
        id: u64,
        chunk: usize,
    ) -> Result<(Position, Geometry), MorphError> {
        self.pipe_in(world.assembly.as_ref().unwrap(), world.tick, id, chunk)
    }
    fn pipe_in(
        &mut self,
        assembly: &crate::filament::Assembly,
        tick: u64,
        id: u64,
        chunk: usize,
    ) -> Result<(Position, Geometry), MorphError> {
        let f = &assembly.filaments[&id];
        if f.nodes.len() == 1 {
            let radius = f
                .section
                .as_ref()
                .map_or(0.025, |p| p.outer_radius as f32 / SCALE as f32);
            return Ok((
                f.nodes[0].position,
                Geometry {
                    shape: Arc::new(Ball::new(radius)),
                    radius,
                    vertices: Arc::new(vec![Vector::ZERO]),
                    parts: vec![],
                    outlets: vec![],
                    writhe_slack: OnceLock::from(0.),
                },
            ));
        }
        let Some(key) = pipe_key(id, f, chunk, tick) else {
            return Err(MorphError("pipe chunk has no cached sweep".into()));
        };
        if let Some((hash, origin, geometry)) = self.pipes.get(&(id, chunk))
            && *hash == key.2
        {
            if self.promoted_geometry.remove(&(id, chunk)) == Some(key.2) {
                self.geometry_reuse_profile.consumed += 1;
            }
            self.profile.pipe_hits += 1;
            #[cfg(feature = "collision-fixture")]
            fixtures::geometry_record("pipe-hit", tick, key, geometry, false, None);
            return Ok((*origin, geometry.clone()));
        }
        let built = Instant::now();
        self.promoted_geometry.remove(&(id, chunk));
        self.profile.pipe_builds += 1;
        let (origin, surface) = filament_chunk_at(f, chunk, tick)?;
        let g = geometry(&[surface], 1.)?;
        self.profile.pipe_geometry_ns += built.elapsed().as_nanos();
        #[cfg(feature = "collision-fixture")]
        fixtures::geometry_record("pipe-build", tick, key, &g, true, None);
        self.pipes.insert((id, chunk), (key.2, origin, g.clone()));
        Ok((origin, g))
    }

    /// Refresh the horizon when pipe positions, sections, or membership change.
    /// Wet aging is not a membership change: a bound captured at a younger tick
    /// stays a superset while radius only dries. An unknown or rewound tick
    /// rebuilds. Failure clears the horizon so the next query cannot reject
    /// with a stale sphere.
    fn sync_pipe_horizon(
        &mut self,
        assembly: &crate::filament::Assembly,
        tick: u64,
    ) -> Result<(), MorphError> {
        let started = Instant::now();
        let (owner, revision) = assembly.filaments.revision();
        let unchanged = self
            .horizon
            .input_revision
            .as_ref()
            .is_some_and(|(saved, seen)| Arc::ptr_eq(owner, saved) && *seen == revision);
        if self.horizon.ready && unchanged && tick >= self.horizon.built_tick {
            self.horizon.confirmed_tick = tick;
            self.profile.pipe_horizon_indexed = self.horizon.spheres.len() as u64;
            self.profile.pipe_horizon_ns += started.elapsed().as_nanos();
            return Ok(());
        }
        if let Err(error) = self.update_pipe_horizon(assembly, tick) {
            self.horizon = PipeHorizon::default();
            self.profile.pipe_horizon_ns += started.elapsed().as_nanos();
            return Err(error);
        }
        self.horizon.input_revision = Some((owner.clone(), revision));
        self.profile.pipe_horizon_ns += started.elapsed().as_nanos();
        Ok(())
    }

    /// Reindex only filaments whose nodes, section, or membership changed.
    /// A later tick with the same stamps keeps the younger, larger bound.
    /// A rewind drops the horizon and builds it again at the earlier tick.
    fn update_pipe_horizon(
        &mut self,
        assembly: &crate::filament::Assembly,
        tick: u64,
    ) -> Result<(), MorphError> {
        if self.horizon.ready && tick < self.horizon.built_tick {
            self.horizon = PipeHorizon::default();
        }
        let mut live = BTreeSet::new();
        let mut changed = false;
        for filament in assembly.filaments.values() {
            if filament.nodes.is_empty() {
                continue;
            }
            live.insert(filament.id);
            self.profile.pipe_filament_stamps += 1;
            let stamp = pipe_filament_stamp(filament)?;
            if self.horizon.ready && self.horizon.filament_stamps.get(&filament.id) == Some(&stamp)
            {
                continue;
            }
            self.reindex_filament(filament, tick)?;
            self.horizon.filament_stamps.insert(filament.id, stamp);
            changed = true;
        }
        let stale = self
            .horizon
            .filament_stamps
            .keys()
            .copied()
            .filter(|id| !live.contains(id))
            .collect::<Vec<_>>();
        for id in stale {
            self.forget_filament(id);
            changed = true;
        }
        self.pipes
            .retain(|key, _| self.horizon.spheres.contains_key(key));
        self.horizon.confirmed_tick = tick;
        self.horizon.built_tick = tick;
        self.horizon.ready = true;
        if changed {
            self.profile.pipe_horizon_rebuilds += 1;
        }
        self.profile.pipe_horizon_indexed = self.horizon.spheres.len() as u64;
        Ok(())
    }

    fn forget_filament(&mut self, id: u64) {
        self.horizon.spheres.retain(|key, _| key.0 != id);
        self.horizon.always_chunks.retain(|key| key.0 != id);
        self.horizon.filament_stamps.remove(&id);
        for chunks in self.horizon.cells.values_mut() {
            chunks.retain(|key| key.0 != id);
        }
        self.horizon.cells.retain(|_, chunks| !chunks.is_empty());
    }

    fn reindex_filament(
        &mut self,
        filament: &crate::filament::Filament,
        tick: u64,
    ) -> Result<(), MorphError> {
        self.forget_filament(filament.id);
        let chunks = filament
            .nodes
            .len()
            .saturating_sub(1)
            .div_ceil(CHUNK_EDGES)
            .max(1);
        for chunk in 0..chunks {
            let start = (chunk * CHUNK_EDGES).min(filament.nodes.len() - 1);
            let origin = filament.nodes[start].position.absolute()?;
            let bound = chunk_mesh_radius_bound(filament, chunk, tick)?;
            self.profile.pipe_bounds_computed += 1;
            let radius = meters_to_abs(bound + HORIZON_PAD_M);
            let mut sphere = ChunkSphere {
                origin,
                radius,
                always: false,
            };
            match cover_cells(origin, radius) {
                Some(cells) => {
                    for cell in cells {
                        self.horizon
                            .cells
                            .entry(cell)
                            .or_default()
                            .insert((filament.id, chunk));
                    }
                }
                None => {
                    sphere.always = true;
                    self.horizon.always_chunks.push((filament.id, chunk));
                }
            }
            self.horizon.spheres.insert((filament.id, chunk), sphere);
        }
        Ok(())
    }

    /// `true` only when the cached sphere cannot meet the query ball.
    /// Missing spheres and oversized chunks return `false` and keep the
    /// authoritative sample-sphere path. Production walks `interacting_chunks`.
    /// This predicate remains the honest-red oracle for that walk.
    #[cfg_attr(not(test), allow(dead_code))]
    fn pipe_beyond_horizon(
        &mut self,
        filament: u64,
        chunk: usize,
        at: Position,
        reach: f32,
    ) -> Result<bool, MorphError> {
        let Some(sphere) = self.horizon.spheres.get(&(filament, chunk)).copied() else {
            return Ok(false);
        };
        if sphere.always || !self.horizon.ready {
            self.profile.pipe_horizon_candidates += 1;
            return Ok(false);
        }
        let center = at.absolute()?;
        let reach_abs = meters_to_abs(reach + HORIZON_PAD_M);
        let limit = sphere.radius.saturating_add(reach_abs);
        let mut gap = 0i128;
        for (axis, &component) in center.iter().enumerate() {
            let delta = i128::from(component) - i128::from(sphere.origin[axis]);
            gap += delta * delta;
        }
        let sphere_near = gap <= i128::from(limit) * i128::from(limit);
        let Some(cells) = cover_cells(center, reach_abs) else {
            self.profile.pipe_horizon_candidates += 1;
            return Ok(false);
        };
        let cell_near = cells.iter().any(|cell| {
            self.horizon
                .cells
                .get(cell)
                .is_some_and(|chunks| chunks.contains(&(filament, chunk)))
        });
        // Either certificate is enough to keep the chunk. Reject only when both
        // the cell index and the absolute sphere say the pair cannot meet.
        let near = cell_near || sphere_near;
        if near {
            self.profile.pipe_horizon_candidates += 1;
            Ok(false)
        } else {
            self.profile.pipe_horizon_rejects += 1;
            Ok(true)
        }
    }

    /// Chunks that might meet the query, in filament-id then chunk order.
    /// The walk visits the candidate set rather than every distant chunk.
    /// An unready horizon or an enormous query returns every live chunk.
    fn interacting_chunks(
        &mut self,
        assembly: &crate::filament::Assembly,
        tick: u64,
        at: Position,
        reach: f32,
    ) -> Result<Vec<(u64, usize)>, MorphError> {
        self.sync_pipe_horizon(assembly, tick)?;
        if !self.horizon.ready {
            let all = all_pipe_chunks(assembly);
            self.profile.pipe_horizon_candidates += all.len() as u64;
            return Ok(all);
        }
        let center = at.absolute()?;
        let reach_abs = meters_to_abs(reach + HORIZON_PAD_M);
        let Some(cells) = cover_cells(center, reach_abs) else {
            let all = all_pipe_chunks(assembly);
            self.profile.pipe_horizon_candidates += all.len() as u64;
            return Ok(all);
        };
        let mut near = BTreeSet::new();
        for cell in &cells {
            if let Some(chunks) = self.horizon.cells.get(cell) {
                near.extend(chunks.iter().copied());
            }
        }
        near.extend(self.horizon.always_chunks.iter().copied());
        let indexed = self.horizon.spheres.len();
        self.profile.pipe_horizon_candidates += near.len() as u64;
        self.profile.pipe_horizon_rejects += indexed.saturating_sub(near.len()) as u64;
        Ok(near.into_iter().collect())
    }
}

fn all_pipe_chunks(assembly: &crate::filament::Assembly) -> Vec<(u64, usize)> {
    let mut chunks = Vec::new();
    for filament in assembly.filaments.values() {
        if filament.nodes.is_empty() {
            continue;
        }
        let count = filament
            .nodes
            .len()
            .saturating_sub(1)
            .div_ceil(CHUNK_EDGES)
            .max(1);
        for chunk in 0..count {
            chunks.push((filament.id, chunk));
        }
    }
    chunks
}

fn pipe_key(
    id: u64,
    f: &crate::filament::Filament,
    chunk: usize,
    tick: u64,
) -> Option<PipeCacheKey> {
    if f.nodes.len() <= 1 {
        return None;
    }
    let start = chunk * CHUNK_EDGES;
    let end = (start + CHUNK_EDGES).min(f.nodes.len() - 1);
    let mut h = blake3::Hasher::new();
    h.update(b"pipe-geometry-v2");
    h.update(&f.created_tick.to_le_bytes());
    h.update(&f.nozzle.to_le_bytes());
    h.update(&[u8::from(end + 1 == f.nodes.len())]);
    if let Some(section) = &f.section {
        h.update(&[
            1,
            u8::from(section.start_sealed),
            u8::from(section.end_sealed),
        ]);
        h.update(&section.outer_radius.to_le_bytes());
        h.update(&section.inner_radius.to_le_bytes());
        let newest = f.nodes[start..=end]
            .iter()
            .map(|n| n.deposited_tick)
            .max()
            .unwrap()
            .max(f.created_tick);
        // Fresh nodes may be deposited at tick+1 while roots are admitted at
        // tick. Both times clamp youngest-age to zero but older rings age!
        // Absolute mesh time distinguishes them; fully dry geometry stabilizes.
        h.update(&tick.min(newest.saturating_add(192)).to_le_bytes());
    } else {
        h.update(&[0]);
    }
    for n in &f.nodes[start.saturating_sub(1)..=(end + 1).min(f.nodes.len() - 1)] {
        h.update(&n.id.to_le_bytes());
        h.update(&n.deposited_tick.to_le_bytes());
        for v in n.position.region.iter().chain(&n.position.local) {
            h.update(&v.to_le_bytes());
        }
    }
    Some((id, chunk, *h.finalize().as_bytes()))
}

fn length(a: [f32; 3]) -> f32 {
    a[0] * a[0] + a[1] * a[1] + a[2] * a[2]
}

/// Upper bound on the distance from the chunk origin to a mesh vertex.
/// Centers, ages, ridges, and nozzle wobble match `filament_chunk_at`.
/// A ring vertex lies within its sample radius of that center. The slack
/// covers an f32 difference between this walk and the built vertex lengths.
/// The centerline chord test is not a cast gate: it moved contact samples.
fn chunk_mesh_radius_bound(
    f: &crate::filament::Filament,
    chunk: usize,
    tick: u64,
) -> Result<f32, MorphError> {
    use crate::math::{add, length as vlen, mul, sub, unit};
    let slack = 0.001;
    let ball = f
        .section
        .as_ref()
        .map_or(0.025, |p| p.outer_radius as f32 / SCALE as f32);
    if f.nodes.len() < 2 {
        return Ok(ball + slack);
    }
    let start = chunk * CHUNK_EDGES;
    let end = (start + CHUNK_EDGES).min(f.nodes.len() - 1);
    if end <= start {
        return Ok(ball + slack);
    }
    let origin = f.nodes[start].position;
    let point = |k: usize| -> Result<[f32; 3], MorphError> {
        Ok(f.nodes[k]
            .position
            .relative(origin)?
            .map(|v| v as f32 / SCALE as f32))
    };
    let Some(section) = &f.section else {
        let last = (start + CHUNK_EDGES + 1).min(f.nodes.len());
        let mut extent = 0f32;
        for k in start..last {
            extent = extent.max(vlen(point(k)?));
        }
        return Ok(extent + 0.025 + slack);
    };
    let tangent = |k: usize| -> Result<[f32; 3], MorphError> {
        let left = k.saturating_sub(1);
        let right = (k + 1).min(f.nodes.len() - 1);
        let scale = if left == k || right == k { 1. } else { 0.5 };
        Ok(mul(sub(point(right)?, point(left)?), scale))
    };
    let mut extent = 0f32;
    for edge in start..end {
        for sample in 0..=3 {
            if edge > start && sample == 0 {
                continue;
            }
            let t = sample as f32 / 3.;
            let p = point(edge)?;
            let q = point(edge + 1)?;
            let m = tangent(edge)?;
            let n = tangent(edge + 1)?;
            let center = add(
                add(
                    mul(p, 2. * t * t * t - 3. * t * t + 1.),
                    mul(m, t * t * t - 2. * t * t + t),
                ),
                add(
                    mul(q, -2. * t * t * t + 3. * t * t),
                    mul(n, t * t * t - t * t),
                ),
            );
            let direction = add(
                add(
                    mul(p, 6. * t * t - 6. * t),
                    mul(m, 3. * t * t - 4. * t + 1.),
                ),
                add(mul(q, -6. * t * t + 6. * t), mul(n, 3. * t * t - 2. * t)),
            );
            let born = f.nodes[edge].deposited_tick.max(f.created_tick) as f32 * (1. - t)
                + f.nodes[edge + 1].deposited_tick.max(f.created_tick) as f32 * t;
            let age = (tick as f32 - born).max(0.);
            let phase = edge as f32 + t;
            let ridges = 1. + 0.065 * (phase * 2.1).sin().powi(6);
            let (mut radius, lumen) = section.radii_at(age as u64);
            radius *= ridges * (1. + 0.035 * (phase * 0.37 + f.nozzle as f32).sin());
            extent = extent.max(vlen(center) + radius);
            let at_start = edge == 0 && sample == 0 && section.start_sealed;
            let at_end = edge + 1 == f.nodes.len() - 1 && sample == 3 && section.end_sealed;
            let cap_depth = (radius - lumen).min(vlen(sub(q, p)) * 0.25);
            let unit_direction = unit(direction);
            let inner_center = if at_start {
                add(center, mul(unit_direction, cap_depth))
            } else if at_end {
                sub(center, mul(unit_direction, cap_depth))
            } else {
                center
            };
            extent = extent.max(vlen(inner_center) + lumen);
        }
    }
    if end == f.nodes.len() - 1 && section.end_sealed {
        let center = point(end)?;
        let axis = unit(tangent(end)?);
        let (outer, inner) = section.radii_at(tick.saturating_sub(f.nodes[end].deposited_tick));
        let depth = (outer - inner).min(vlen(sub(point(end)?, point(end - 1)?)) * 0.25);
        extent = extent.max(vlen(center));
        extent = extent.max(vlen(sub(center, mul(axis, depth))));
    }
    Ok(extent + slack)
}

/// Rejects a chunk only when its sample sphere cannot meet the query ball.
/// That is the same test as `near` before a cast, with this bound in place of
/// the built mesh. A pass still builds the hollow trimesh.
fn pipe_separated(
    cache: &mut CollisionCache,
    f: &crate::filament::Filament,
    chunk: usize,
    tick: u64,
    at: Position,
    reach: f32,
) -> Result<bool, MorphError> {
    if f.nodes.is_empty() {
        return Ok(false);
    }
    let start = (chunk * CHUNK_EDGES).min(f.nodes.len() - 1);
    let origin = f.nodes[start].position;
    let distance = length(origin.relative(at)?.map(|v| v as f32 / SCALE as f32)).sqrt();
    let bound = chunk_mesh_radius_bound(f, chunk, tick)?;
    if distance <= reach + bound + 0.003 {
        cache.profile.pipe_candidates += 1;
        return Ok(false);
    }
    cache.profile.pipe_rejects += 1;
    Ok(true)
}

/// Cell size for the collision horizon. Occupied chunks are inserted into the
/// cells their conservative spheres cover. Empty habitat volume is not allocated.
const PIPE_CELL: i64 = 8 * SCALE;
/// Wider than `pipe_separated`'s 0.003 m gate, on both the stored bound and the
/// query reach, so f32 conversion cannot hide a sample-sphere candidate.
const HORIZON_PAD_M: f32 = 0.005;

#[derive(Clone, Copy)]
struct ChunkSphere {
    /// Read by the honest-red sphere predicate. The production walk uses cells.
    #[cfg_attr(not(test), allow(dead_code))]
    origin: [i64; 3],
    #[cfg_attr(not(test), allow(dead_code))]
    radius: i64,
    #[cfg_attr(not(test), allow(dead_code))]
    always: bool,
}

#[derive(Default)]
struct PipeHorizon {
    input_revision: Option<(Arc<std::sync::atomic::AtomicU64>, u64)>,
    confirmed_tick: u64,
    built_tick: u64,
    ready: bool,
    spheres: BTreeMap<(u64, usize), ChunkSphere>,
    cells: BTreeMap<[i64; 3], BTreeSet<(u64, usize)>>,
    /// Chunks too large for the cell cap. Every query keeps them.
    always_chunks: Vec<(u64, usize)>,
    /// Per-filament geometry stamp. An unchanged filament keeps its spheres.
    filament_stamps: BTreeMap<u64, [u8; 32]>,
}

fn meters_to_abs(meters: f32) -> i64 {
    let units = f64::from(meters.max(0.)) * SCALE as f64;
    if !units.is_finite() || units >= (i64::MAX / 4) as f64 {
        return i64::MAX / 4;
    }
    units.ceil() as i64 + 1
}

/// `None` means the sphere covers too many cells to index. Callers then keep
/// the authoritative test instead of allocating a volume-sized grid.
fn cover_cells(origin: [i64; 3], radius: i64) -> Option<Vec<[i64; 3]>> {
    let min = origin.map(|v| v.saturating_sub(radius).div_euclid(PIPE_CELL));
    let max = origin.map(|v| v.saturating_add(radius).div_euclid(PIPE_CELL));
    let mut count = 1i64;
    for axis in 0..3 {
        let span = max[axis].saturating_sub(min[axis]).saturating_add(1);
        if span <= 0 || span > 64 {
            return None;
        }
        count = count.saturating_mul(span);
        if count > 512 {
            return None;
        }
    }
    let mut cells = Vec::with_capacity(count as usize);
    for x in min[0]..=max[0] {
        for y in min[1]..=max[1] {
            for z in min[2]..=max[2] {
                cells.push([x, y, z]);
            }
        }
    }
    Some(cells)
}

fn pipe_filament_stamp(filament: &crate::filament::Filament) -> Result<[u8; 32], MorphError> {
    let mut hash = blake3::Hasher::new();
    hash.update(b"pipe-horizon-input-v2");
    for v in [
        filament.id,
        filament.created_tick,
        filament.nozzle,
        filament.quantity,
    ] {
        hash.update(&v.to_le_bytes());
    }
    if let Some(section) = &filament.section {
        hash.update(&[
            1,
            u8::from(section.start_sealed),
            u8::from(section.end_sealed),
        ]);
        hash.update(&section.outer_radius.to_le_bytes());
        hash.update(&section.inner_radius.to_le_bytes());
    } else {
        hash.update(&[0]);
    }
    for node in &filament.nodes {
        hash.update(&node.id.to_le_bytes());
        hash.update(&node.deposited_tick.to_le_bytes());
        for v in node.position.absolute()? {
            hash.update(&v.to_le_bytes());
        }
    }
    Ok(*hash.finalize().as_bytes())
}

fn near(a: RigidPose, b: RigidPose, ra: f32, rb: f32, da: f32, db: f32) -> bool {
    let d = vec(std::array::from_fn(|k| a.translation[k] - b.translation[k]));
    d.length() <= ra + rb + da + db + 0.003
}
struct QueryNanos<'a> {
    profile: &'a mut CollisionProfile,
    workspace: &'a mut ContactWorkspace,
    started: Instant,
    visits: u64,
}
impl Drop for QueryNanos<'_> {
    fn drop(&mut self) {
        self.profile.queries += self.visits;
        self.profile.query_ns += self.started.elapsed().as_nanos();
    }
}
struct DistanceCounts {
    triangles: Cell<u64>,
    bounded: Cell<u64>,
    exact: Cell<u64>,
    nodes: Cell<u64>,
    node_rejects: Cell<u64>,
    #[cfg(feature = "kernel-attribution")]
    plane_ns: Cell<u128>,
    #[cfg(feature = "kernel-attribution")]
    bounded_gjk_ns: Cell<u128>,
    #[cfg(feature = "kernel-attribution")]
    exact_ns: Cell<u128>,
    #[cfg(feature = "kernel-attribution")]
    support_calls: Cell<u64>,
    #[cfg(feature = "kernel-attribution")]
    gjk_calls: Cell<u64>,
    #[cfg(feature = "kernel-attribution")]
    gjk_steps: Cell<u64>,
    #[cfg(feature = "kernel-attribution")]
    gjk_max_steps: Cell<u64>,
    #[cfg(feature = "kernel-attribution")]
    gjk_100_steps: Cell<u64>,
    #[cfg(feature = "kernel-attribution")]
    exact_steps: Cell<u64>,
}
impl DistanceCounts {
    fn new() -> Self {
        Self {
            triangles: Cell::new(0),
            bounded: Cell::new(0),
            exact: Cell::new(0),
            nodes: Cell::new(0),
            node_rejects: Cell::new(0),
            #[cfg(feature = "kernel-attribution")]
            plane_ns: Cell::new(0),
            #[cfg(feature = "kernel-attribution")]
            bounded_gjk_ns: Cell::new(0),
            #[cfg(feature = "kernel-attribution")]
            exact_ns: Cell::new(0),
            #[cfg(feature = "kernel-attribution")]
            support_calls: Cell::new(0),
            #[cfg(feature = "kernel-attribution")]
            gjk_calls: Cell::new(0),
            #[cfg(feature = "kernel-attribution")]
            gjk_steps: Cell::new(0),
            #[cfg(feature = "kernel-attribution")]
            gjk_max_steps: Cell::new(0),
            #[cfg(feature = "kernel-attribution")]
            gjk_100_steps: Cell::new(0),
            #[cfg(feature = "kernel-attribution")]
            exact_steps: Cell::new(0),
        }
    }
    fn bump(field: &Cell<u64>) {
        field.set(field.get() + 1);
    }
}
/// Test interval clearance with original Parry leaf kernels. One leaf within
/// the bound proves the predicate for the union: stop then instead of finding
/// the globally nearest triangle/component. Expanded BVH bounds cover roundoff;
/// no physical shape is replaced or approximated.
fn within_distance(
    supports: &SupportCache,
    relative: &Iso,
    a: &dyn Shape,
    b: &dyn Shape,
    limit: f32,
    roundoff: f32,
    counts: &DistanceCounts,
) -> Result<bool, query::Unsupported> {
    let (composite, relative, other) = if let Some(c) = a.as_composite_shape() {
        (c, *relative, b)
    } else if let Some(c) = b.as_composite_shape() {
        (c, relative.inverse(), a)
    } else {
        if a.as_triangle().is_some() || b.as_triangle().is_some() {
            DistanceCounts::bump(&counts.triangles);
        }
        #[cfg(feature = "kernel-attribution")]
        let plane_started = Instant::now();
        let plane_clear = if let (Some(index), Some(triangle)) = (supports.get(a), b.as_triangle())
        {
            index.separates_triangle(relative, triangle, limit, roundoff)
        } else if let (Some(index), Some(triangle)) = (supports.get(b), a.as_triangle()) {
            index.separates_triangle(&relative.inverse(), triangle, limit, roundoff)
        } else {
            false
        };
        #[cfg(feature = "kernel-attribution")]
        counts
            .plane_ns
            .set(counts.plane_ns.get() + plane_started.elapsed().as_nanos());
        if plane_clear {
            DistanceCounts::bump(&counts.bounded);
            return Ok(false);
        }
        // Ask GJK for proximity without solving exact witnesses first. Accept
        // only certified lower/upper bounds outside a roundoff band; otherwise
        // preserve the original distance kernel, including uncertain failures.
        if a.as_ball().is_none()
            && b.as_ball().is_none()
            && !(a.as_cuboid().is_some() && b.as_cuboid().is_some())
            && let (Some(sa), Some(sb)) = (a.as_support_map(), b.as_support_map())
        {
            let indexed_a = IndexedSupport {
                inner: sa,
                index: supports.get(a),
                points: a.as_convex_polyhedron().map_or(0, |p| p.points().len()),
                #[cfg(feature = "kernel-attribution")]
                diagnostic_calls: Some(&counts.support_calls),
            };
            let indexed_b = IndexedSupport {
                inner: sb,
                index: supports.get(b),
                points: b.as_convex_polyhedron().map_or(0, |p| p.points().len()),
                #[cfg(feature = "kernel-attribution")]
                diagnostic_calls: Some(&counts.support_calls),
            };
            let (sa, sb): (
                &dyn parry3d::shape::SupportMap,
                &dyn parry3d::shape::SupportMap,
            ) = (&indexed_a, &indexed_b);
            let dir = -relative.translation;
            let dir = if dir.length_squared() > parry3d::math::DEFAULT_EPSILON.powi(2) {
                dir.normalize()
            } else {
                Vector::X
            };
            let mut simplex = VoronoiSimplex::new();
            simplex.reset(CsoPoint::from_shapes(relative, sa, sb, dir));
            #[cfg(feature = "kernel-attribution")]
            let bounded_started = Instant::now();
            #[cfg(feature = "kernel-attribution")]
            let support_before = counts.support_calls.get();
            let bounded_result =
                gjk::closest_points(relative, sa, sb, limit + roundoff, false, &mut simplex);
            #[cfg(feature = "kernel-attribution")]
            counts
                .bounded_gjk_ns
                .set(counts.bounded_gjk_ns.get() + bounded_started.elapsed().as_nanos());
            #[cfg(feature = "kernel-attribution")]
            {
                let steps = (counts.support_calls.get() - support_before) / 2;
                counts.gjk_calls.set(counts.gjk_calls.get() + 1);
                counts.gjk_steps.set(counts.gjk_steps.get() + steps);
                counts
                    .gjk_max_steps
                    .set(counts.gjk_max_steps.get().max(steps));
                if steps == 100 {
                    counts.gjk_100_steps.set(counts.gjk_100_steps.get() + 1);
                }
            }
            match bounded_result {
                GJKResult::Intersection => {
                    DistanceCounts::bump(&counts.bounded);
                    return Ok(true);
                }
                GJKResult::NoIntersection(axis) => {
                    // NoIntersection also represents the iteration limit in
                    // Parry; a real separating support plane is required.
                    let support = CsoPoint::from_shapes(relative, sa, sb, axis);
                    if -axis.dot(support.point) > limit + roundoff {
                        DistanceCounts::bump(&counts.bounded);
                        return Ok(false);
                    }
                }
                GJKResult::Proximity(_) => {
                    simplex.project_origin_and_reduce();
                    // Parry exposes three projected coordinates in 3D. A
                    // full tetrahedron has no readable fourth coordinate;
                    // leave this ambiguous case to the original kernel.
                    if simplex.dimension() < 3 {
                        let mut p = Vector::ZERO;
                        let mut q = Vector::ZERO;
                        for i in 0..=simplex.dimension() {
                            p += simplex.point(i).orig1 * simplex.proj_coord(i);
                            q += simplex.point(i).orig2 * simplex.proj_coord(i);
                        }
                        if (p - q).length() + roundoff < limit {
                            DistanceCounts::bump(&counts.bounded);
                            return Ok(true);
                        }
                    }
                }
                _ => {}
            }
            if !(a.as_segment().is_some() && b.as_segment().is_some()) {
                DistanceCounts::bump(&counts.exact);
                #[cfg(feature = "kernel-attribution")]
                let exact_started = Instant::now();
                #[cfg(feature = "kernel-attribution")]
                let exact_support_before = counts.support_calls.get();
                let result =
                    query::details::distance_support_map_support_map(relative, sa, sb) <= limit;
                #[cfg(feature = "kernel-attribution")]
                counts
                    .exact_ns
                    .set(counts.exact_ns.get() + exact_started.elapsed().as_nanos());
                #[cfg(feature = "kernel-attribution")]
                counts.exact_steps.set(
                    counts.exact_steps.get()
                        + (counts.support_calls.get() - exact_support_before) / 2,
                );
                return Ok(result);
            }
        }
        DistanceCounts::bump(&counts.exact);
        #[cfg(feature = "kernel-attribution")]
        let exact_started = Instant::now();
        let result = DefaultQueryDispatcher
            .distance(relative, a, b)
            .map(|d| d <= limit);
        #[cfg(feature = "kernel-attribution")]
        counts
            .exact_ns
            .set(counts.exact_ns.get() + exact_started.elapsed().as_nanos());
        return result;
    };
    let bounds = other.compute_aabb(&relative);
    let shift = -bounds.center();
    let margin = bounds.half_extents();
    let projection_planes = supports.get(other).and_then(|index| {
        index.projection_planes(&relative, &composite.bvh().root_aabb(), limit, roundoff)
    });
    Ok(composite
        .bvh()
        .find_best(
            limit + roundoff,
            |node, _| {
                DistanceCounts::bump(&counts.nodes);
                let bound = Aabb {
                    mins: node.mins() + shift - margin,
                    maxs: node.maxs() + shift + margin,
                }
                .distance_to_origin();
                if bound <= limit + roundoff
                    && projection_planes
                        .as_ref()
                        .is_some_and(|planes| planes.excludes(node.mins(), node.maxs()))
                {
                    DistanceCounts::bump(&counts.node_rejects);
                    f32::MAX
                } else {
                    bound
                }
            },
            |part, _| {
                composite
                    .map_untyped_part_at(part, |pose, shape, _| {
                        within_distance(
                            supports,
                            &pose.inv_mul(&relative),
                            shape,
                            other,
                            limit,
                            roundoff,
                            counts,
                        )
                        .ok()
                        .filter(|near| *near)
                        .map(|_| 0f32)
                    })
                    .flatten()
            },
        )
        .is_some())
}

fn cast(
    a: &Geometry,
    from: RigidPose,
    to: RigidPose,
    b: &Geometry,
    bfrom: RigidPose,
    bto: RigidPose,
    context: (&mut CollisionProfile, &mut ContactWorkspace),
) -> Result<Option<(f32, [i64; 3], Position)>, MorphError> {
    let (profile, workspace) = context;
    let mut charge = QueryNanos {
        profile,
        workspace,
        started: Instant::now(),
        visits: 0,
    };
    let origin = Position::from_absolute(from.translation)?;
    let m1 = motion(from, to, origin)?;
    let m2 = motion(bfrom, bto, origin)?;
    // Distance is Lipschitz-bounded by relative translation plus angular speed
    // times each shape's enclosing radius. An interval is empty only when its
    // midpoint separation exceeds that bound. Unresolved small intervals block.
    let speed = (m1.linvel - m2.linvel).length()
        + a.radius * m1.angvel.length()
        + b.radius * m2.angvel.length();
    let skin = 0.0001f32;
    // Already touching/penetrating pairs have a contact at t=0. Resolve that
    // actual contact once, rather than rediscover it by ~17 identical distance
    // evaluations while bisecting toward zero. The hollow geometry is unchanged.
    let contact_started = Instant::now();
    let initial = cached_world_contact(charge.workspace, &m1.start, a, &m2.start, b, skin)
        .ok()
        .flatten();
    let initial_contact_ns = contact_started.elapsed().as_nanos();
    charge.profile.contact_ns += initial_contact_ns;
    #[cfg(feature = "query-trace")]
    query_trace::note_phase("initial_contact", initial_contact_ns);
    #[cfg(feature = "collision-fixture")]
    fixtures::sample(|| fixtures::phase("initial_contact", initial_contact_ns, initial.is_some()));
    charge.visits += 1;
    if let Some(c) = initial {
        let point = origin.offset(
            c.point1
                .to_array()
                .map(|x| (x * SCALE as f32).round() as i64),
        )?;
        return Ok(Some((
            0.,
            c.normal1
                .to_array()
                .map(|x| (x * SCALE as f32).round() as i64),
            point,
        )));
    }
    if m1.angvel == Vector::ZERO && m2.angvel == Vector::ZERO {
        // Exact constant-rotation case: use the library's linear BVH sweep
        // rather than repeatedly asking for static distance on the same mesh.
        // Supply relative velocity explicitly in shape-1 coordinates.
        let local_delta = m1.start.rotation.inverse() * (m2.linvel - m1.linvel);
        let options = ShapeCastOptions {
            max_time_of_impact: 1.,
            target_distance: skin,
            stop_at_penetration: true,
            compute_impact_geometry_on_penetration: true,
        };
        let linear_started = Instant::now();
        let dispatcher = FilteredLinearDispatcher::new(&charge.workspace.supports);
        let result = dispatcher.cast_shapes(
            &m1.start.inv_mul(&m2.start),
            local_delta,
            a.shape.as_ref(),
            b.shape.as_ref(),
            options,
        );
        let linear_ns = linear_started.elapsed().as_nanos();
        charge.profile.linear_cast_ns += linear_ns;
        charge.profile.linear_nodes += dispatcher.nodes.load(std::sync::atomic::Ordering::Relaxed);
        charge.profile.linear_node_rejects += dispatcher
            .rejects
            .load(std::sync::atomic::Ordering::Relaxed);
        charge.profile.linear_leaf_dispatches +=
            dispatcher.leaves.load(std::sync::atomic::Ordering::Relaxed);
        #[cfg(feature = "query-trace")]
        query_trace::note_phase("linear", linear_ns);
        #[cfg(feature = "collision-fixture")]
        fixtures::sample(|| fixtures::Sample {
            phase: "linear_cast".into(),
            work: None,
            lo_bits: 0,
            hi_bits: 1f32.to_bits(),
            mid_bits: 0,
            limit_bits: skin.to_bits(),
            roundoff_bits: 0,
            within: result.as_ref().is_ok_and(Option::is_some),
            triangles: 0,
            bounded: 0,
            exact: 0,
            ns: linear_ns,
        });
        charge.profile.linear_casts += 1;
        charge.visits += 1;
        match result {
            Ok(None) => return Ok(None),
            Ok(Some(hit))
                if matches!(
                    hit.status,
                    ShapeCastStatus::Converged | ShapeCastStatus::PenetratingOrWithinTargetDist
                ) =>
            {
                let pose = m1.position_at_time(hit.time_of_impact);
                let normal = pose.rotation * hit.normal1;
                let point = origin.offset(
                    (pose * hit.witness1)
                        .to_array()
                        .map(|x| (x * SCALE as f32).round() as i64),
                )?;
                return Ok(Some((
                    hit.time_of_impact,
                    normal.to_array().map(|x| (x * SCALE as f32).round() as i64),
                    point,
                )));
            }
            _ => {} // Unsupported or nonconverged: preserve the conservative fallback.
        }
    }
    let mut pending = vec![(0f32, 1f32)];
    while let Some((lo, hi)) = pending.pop() {
        #[cfg(feature = "collision-fixture")]
        let bounds_started = Instant::now();
        charge.visits += 1;
        let mid = (lo + hi) * 0.5;
        let pa = m1.position_at_time(mid);
        let pb = m2.position_at_time(mid);
        // An enclosing AABB distance is a lower bound on shape distance. Apply
        // the same Lipschitz interval bound before the expensive hollow-mesh
        // query. Mesh creation, cache fills and interval ordering are unchanged.
        // Keep a millimetre margin beyond the contact skin for f32 roundoff.
        let aa = a.shape.compute_aabb(&pa);
        let bb = b.shape.compute_aabb(&pb);
        let gap = (aa.mins - bb.maxs).max(bb.mins - aa.maxs).max(Vector::ZERO);
        let magnitude = aa
            .mins
            .abs()
            .max(aa.maxs.abs())
            .max(bb.mins.abs())
            .max(bb.maxs.abs())
            .max_element();
        let roundoff = (magnitude * 16. * f32::EPSILON).max(0.001);
        if gap.length() > speed * (hi - lo) * 0.5 + skin + roundoff {
            charge.profile.interval_rejects += 1;
            #[cfg(feature = "collision-fixture")]
            fixtures::sample(|| {
                fixtures::phase("aabb_reject", bounds_started.elapsed().as_nanos(), false)
            });
            continue;
        }
        // Reuse the hollow mesh's own BVH. Every compound component must be
        // separated from every triangle's AABB by the full interval movement
        // margin before a costly distance evaluation may be skipped. No mesh
        // is replaced by a solid proxy; uncertain intervals keep the old query.
        if let Some(mesh) = b.shape.as_trimesh() {
            let relative = pb.inv_mul(&pa);
            let margin = speed * (hi - lo) * 0.5 + skin + roundoff;
            let clear = if a.parts.is_empty() {
                mesh.bvh()
                    .intersect_aabb(&a.shape.compute_aabb(&relative).loosened(margin))
                    .next()
                    .is_none()
            } else {
                a.parts.iter().all(|(_, part)| {
                    mesh.bvh()
                        .intersect_aabb(&part.shape.compute_aabb(&relative).loosened(margin))
                        .next()
                        .is_none()
                })
            };
            if clear {
                charge.profile.mesh_interval_rejects += 1;
                #[cfg(feature = "collision-fixture")]
                fixtures::sample(|| {
                    fixtures::phase(
                        "mesh_bvh_reject",
                        bounds_started.elapsed().as_nanos(),
                        false,
                    )
                });
                continue;
            }
        }
        #[cfg(feature = "collision-fixture")]
        fixtures::sample(|| {
            fixtures::phase("interval_bounds", bounds_started.elapsed().as_nanos(), true)
        });
        charge.profile.distance_queries += 1;
        let distance_started = Instant::now();
        let counts = DistanceCounts::new();
        let limit = speed * (hi - lo) * 0.5 + skin;
        let within = within_distance(
            &charge.workspace.supports,
            &pa.inv_mul(&pb),
            a.shape.as_ref(),
            b.shape.as_ref(),
            limit,
            roundoff,
            &counts,
        )
        .map_err(|_| MorphError("unsupported physical distance query; movement refused".into()))?;
        let distance_ns = distance_started.elapsed().as_nanos();
        charge.profile.distance_ns += distance_ns;
        #[cfg(feature = "query-trace")]
        query_trace::note_phase("distance", distance_ns);
        #[cfg(feature = "collision-fixture")]
        fixtures::sample(|| fixtures::Sample {
            phase: "distance".into(),
            work: None,
            lo_bits: lo.to_bits(),
            hi_bits: hi.to_bits(),
            mid_bits: mid.to_bits(),
            limit_bits: limit.to_bits(),
            roundoff_bits: roundoff.to_bits(),
            within,
            triangles: counts.triangles.get(),
            bounded: counts.bounded.get(),
            exact: counts.exact.get(),
            ns: distance_ns,
        });
        charge.profile.triangles_examined += counts.triangles.get();
        charge.profile.distance_bounded += counts.bounded.get();
        charge.profile.distance_exact += counts.exact.get();
        #[cfg(feature = "kernel-attribution")]
        fixtures::sample(|| fixtures::phase("leaf_planes", counts.plane_ns.get(), within));
        #[cfg(feature = "kernel-attribution")]
        fixtures::sample(|| fixtures::phase("bounded_gjk", counts.bounded_gjk_ns.get(), within));
        #[cfg(feature = "kernel-attribution")]
        fixtures::sample(|| fixtures::phase("exact_distance", counts.exact_ns.get(), within));
        #[cfg(feature = "kernel-attribution")]
        fixtures::sample(|| {
            let mut sample = fixtures::phase("kernel_work", 0, within);
            sample.work = Some(fixtures::KernelWork {
                support_calls: counts.support_calls.get(),
                bounded_gjk_calls: counts.gjk_calls.get(),
                bounded_gjk_support_steps: counts.gjk_steps.get(),
                bounded_gjk_max_steps: counts.gjk_max_steps.get(),
                bounded_gjk_100_step_calls: counts.gjk_100_steps.get(),
                exact_support_pairs: counts.exact_steps.get(),
            });
            sample
        });
        charge.profile.distance_nodes += counts.nodes.get();
        charge.profile.distance_node_rejects += counts.node_rejects.get();
        #[cfg(feature = "query-trace")]
        query_trace::note_distance(
            limit.to_bits(),
            mid.to_bits(),
            within,
            counts.triangles.get(),
            counts.bounded.get(),
            counts.exact.get(),
        );
        if !within {
            charge.profile.distance_clear += 1;
            continue;
        }
        charge.profile.distance_within += 1;
        if hi - lo <= 0.00001 || speed == 0. || charge.visits >= 512 {
            let contact_started = Instant::now();
            let closest =
                cached_world_contact(charge.workspace, &pa, a, &pb, b, skin + speed * (hi - lo))
                    .ok()
                    .flatten();
            let terminal_contact_ns = contact_started.elapsed().as_nanos();
            charge.profile.contact_ns += terminal_contact_ns;
            #[cfg(feature = "query-trace")]
            query_trace::note_phase("terminal_contact", terminal_contact_ns);
            #[cfg(feature = "collision-fixture")]
            fixtures::sample(|| {
                fixtures::phase("terminal_contact", terminal_contact_ns, closest.is_some())
            });
            let (normal, point) = if let Some(c) = closest {
                (c.normal1, c.point1)
            } else {
                (
                    (pb.translation - pa.translation).normalize_or_zero(),
                    pa.translation,
                )
            };
            let point =
                origin.offset(point.to_array().map(|x| (x * SCALE as f32).round() as i64))?;
            return Ok(Some((
                lo,
                normal.to_array().map(|x| (x * SCALE as f32).round() as i64),
                point,
            )));
        }
        pending.push((mid, hi));
        pending.push((lo, mid));
    }
    Ok(None)
}

fn record(
    world: &mut WorldSnapshot,
    id: u64,
    target: PhysicalTarget,
    result: (f32, [i64; 3], Position),
    response: ContactResponse,
) {
    let observation = ContactObservation {
        at: world.tick,
        predicted: true,
        organism: id,
        target,
        point: result.2,
        normal: result.1,
        response,
        toi_millionths: (result.0.clamp(0., 1.) * 1e6).round() as u32,
    };
    world
        .habitat
        .as_mut()
        .unwrap()
        .contacts
        .push(observation.clone());
    world.events.push(Event::Contact { observation });
}
/// A normal points out of the moving body into the obstacle. Project positive
/// (inward) travel and bias the correction outward, bounded by the requested step.
fn along_surface(start: RigidPose, aim: RigidPose, normal: [i64; 3]) -> RigidPose {
    let travel = vec(std::array::from_fn(|k| {
        aim.translation[k].saturating_sub(start.translation[k])
    }));
    let n = vec(normal).normalize_or_zero();
    if n.length_squared() < 0.5 {
        return start;
    }
    let into = travel.dot(n);
    if into < -0.00001 {
        return aim;
    }
    let tangent = travel - n * into.max(0.);
    let mut correction = tangent - n * 0.02;
    if correction.length() > travel.length() {
        correction = correction.normalize_or_zero() * travel.length();
    }
    let mut pose = start; // Obstructed bodies do not sweep an unchecked rotation.
    pose.translation = std::array::from_fn(|k| {
        start.translation[k].saturating_add((correction[k] * SCALE as f32).round() as i64)
    });
    pose
}

/// Existing overlap is not permission to tunnel. Accept a translation away only
/// when four intermediate/endpoint contacts do not deepen the initial overlap.
/// Tangential progress is allowed so a pre-existing wedged state can unwind. Every other obstacle is still swept normally.
// Adapted from Parry 0.27.0, Apache-2.0; see licenses/Parry-NOTICE.md.
// Same contact dispatch, leaf arithmetic, BVH order and strict tie rule.
// EPA::closest_points resets all logical workspace; only capacities persist.
fn reusable_contact(
    epa: &mut EPA,
    supports: &SupportCache,
    relative: &Iso,
    a: &dyn Shape,
    b: &dyn Shape,
    prediction: f32,
) -> Result<Option<query::Contact>, query::Unsupported> {
    if a.as_ball().is_some()
        || b.as_ball().is_some()
        || a.as_shape::<parry3d::shape::HalfSpace>().is_some()
        || b.as_shape::<parry3d::shape::HalfSpace>().is_some()
    {
        // Composite balls must still traverse through this dispatcher.
        if a.as_composite_shape().is_none() && b.as_composite_shape().is_none() {
            return DefaultQueryDispatcher.contact(relative, a, b, prediction);
        }
    }
    if let (Some(sa), Some(sb)) = (a.as_support_map(), b.as_support_map()) {
        if a.as_ball().is_some() || b.as_ball().is_some() {
            return DefaultQueryDispatcher.contact(relative, a, b, prediction);
        }
        // Same certified plane as the distance predicate. A rejection is farther
        // than `prediction`; penetration and borderline gaps still use GJK/EPA.
        const CONTACT_ROUNDOFF: f32 = 0.001;
        if let (Some(index), Some(triangle)) = (supports.get(a), b.as_triangle()) {
            if index.separates_triangle(relative, triangle, prediction, CONTACT_ROUNDOFF) {
                return Ok(None);
            }
        } else if let (Some(index), Some(triangle)) = (supports.get(b), a.as_triangle())
            && index.separates_triangle(&relative.inverse(), triangle, prediction, CONTACT_ROUNDOFF)
        {
            return Ok(None);
        }
        #[cfg(feature = "allocation-attribution")]
        crate::profiling::work(2, 1);
        let indexed_a = IndexedSupport {
            inner: sa,
            index: supports.get(a),
            points: a.as_convex_polyhedron().map_or(0, |p| p.points().len()),
            #[cfg(feature = "kernel-attribution")]
            diagnostic_calls: None,
        };
        let indexed_b = IndexedSupport {
            inner: sb,
            index: supports.get(b),
            points: b.as_convex_polyhedron().map_or(0, |p| p.points().len()),
            #[cfg(feature = "kernel-attribution")]
            diagnostic_calls: None,
        };
        let (sa, sb): (
            &dyn parry3d::shape::SupportMap,
            &dyn parry3d::shape::SupportMap,
        ) = (&indexed_a, &indexed_b);
        let dir = relative.translation.try_normalize().unwrap_or(Vector::X);
        let mut simplex = VoronoiSimplex::new();
        simplex.reset(CsoPoint::from_shapes(relative, sa, sb, dir));
        let mut points = gjk::closest_points(relative, sa, sb, prediction, true, &mut simplex);
        if points == GJKResult::Intersection {
            #[cfg(feature = "kernel-attribution")]
            fixtures::note_contact_epa();
            #[cfg(feature = "allocation-attribution")]
            crate::profiling::work(3, 1);
            points = epa
                .closest_points(relative, sa, sb, &simplex)
                .map_or(GJKResult::NoIntersection(Vector::X), |(p, q, n)| {
                    GJKResult::ClosestPoints(p, q, n)
                });
        }
        return Ok(match points {
            GJKResult::ClosestPoints(p, q, n) => Some(query::Contact::new(
                p,
                relative.inverse_transform_point(q),
                n,
                relative.rotation.inverse() * -n,
                (q - p).dot(n),
            )),
            GJKResult::NoIntersection(_) => None,
            _ => unreachable!("same exact-contact GJK contract as Parry"),
        });
    }
    let (composite, pose, other, flipped) = if let Some(c) = a.as_composite_shape() {
        (c, *relative, b, false)
    } else if let Some(c) = b.as_composite_shape() {
        (c, relative.inverse(), a, true)
    } else {
        return DefaultQueryDispatcher.contact(relative, a, b, prediction);
    };
    let bounds = other.compute_aabb(&pose).loosened(prediction);
    let mut result: Option<query::Contact> = None;
    for part in composite.bvh().intersect_aabb(&bounds) {
        composite.map_part_at(part, &mut |part_pose, shape, _| {
            if let Ok(Some(mut c)) = reusable_contact(
                epa,
                supports,
                &part_pose.inv_mul(&pose),
                shape,
                other,
                prediction,
            ) && result.is_none_or(|best| c.dist < best.dist)
            {
                if let Some(p) = part_pose {
                    c.transform1_by_mut(p)
                }
                result = Some(c);
            }
        });
    }
    Ok(if flipped {
        result.map(|c| c.flipped())
    } else {
        result
    })
}
// Persistent indices retain original immutable shape allocations. Bounded entries
// fall back to the original linear support scan when the cache is full.
#[derive(Default)]
struct SupportCache {
    shapes: BTreeMap<usize, Arc<dyn Shape>>,
    indices: BTreeMap<usize, crate::support_index::SupportIndex>,
}
impl SupportCache {
    fn register(&mut self, shape: &Arc<dyn Shape>) {
        if shape.as_convex_polyhedron().is_none() && shape.as_compound().is_none() {
            return; // Meshes/capsules have no indexed convex vertices to retain.
        }
        let key = Arc::as_ptr(shape) as *const () as usize;
        if self.shapes.contains_key(&key) || self.shapes.len() >= 8192 {
            return;
        }
        self.shapes.insert(key, shape.clone());
        if let Some(hull) = shape.as_convex_polyhedron() {
            if hull.points().len() > 24 && hull.points().iter().all(|p| p.is_finite()) {
                self.indices.insert(
                    key,
                    crate::support_index::SupportIndex::new(hull.points())
                        .with_normals(hull.faces().iter().map(|f| f.normal)),
                );
            }
        } else if let Some(compound) = shape.as_compound() {
            for (_, child) in compound.shapes() {
                self.register(&child.0);
            }
        }
    }
    fn get(&self, shape: &dyn Shape) -> Option<&crate::support_index::SupportIndex> {
        self.indices
            .get(&(shape as *const dyn Shape as *const () as usize))
    }
}
struct IndexedSupport<'a> {
    inner: &'a dyn parry3d::shape::SupportMap,
    index: Option<&'a crate::support_index::SupportIndex>,
    #[cfg_attr(not(feature = "allocation-attribution"), allow(dead_code))]
    points: usize,
    #[cfg(feature = "kernel-attribution")]
    diagnostic_calls: Option<&'a Cell<u64>>,
}
impl parry3d::shape::SupportMap for IndexedSupport<'_> {
    fn local_support_point(&self, d: Vector) -> Vector {
        #[cfg(feature = "kernel-attribution")]
        if let Some(calls) = self.diagnostic_calls {
            calls.set(calls.get() + 1);
        }

        #[cfg(feature = "allocation-attribution")]
        {
            crate::profiling::work(0, 1);
            crate::profiling::work(1, self.points as u64);
            if self.index.is_none() {
                crate::profiling::work(4, self.points as u64);
            }
        }
        self.index
            .map_or_else(|| self.inner.local_support_point(d), |i| i.point(d))
    }
}

struct FilteredLinearDispatcher<'a> {
    supports: &'a SupportCache,
    filter_nodes: bool,
    nodes: std::sync::atomic::AtomicU64,
    rejects: std::sync::atomic::AtomicU64,
    leaves: std::sync::atomic::AtomicU64,
}
impl QueryDispatcher for FilteredLinearDispatcher<'_> {
    fn intersection_test(
        &self,
        p: &Iso,
        a: &dyn Shape,
        b: &dyn Shape,
    ) -> Result<bool, query::Unsupported> {
        DefaultQueryDispatcher.intersection_test(p, a, b)
    }
    fn distance(&self, p: &Iso, a: &dyn Shape, b: &dyn Shape) -> Result<f32, query::Unsupported> {
        DefaultQueryDispatcher.distance(p, a, b)
    }
    fn contact(
        &self,
        p: &Iso,
        a: &dyn Shape,
        b: &dyn Shape,
        prediction: f32,
    ) -> Result<Option<query::Contact>, query::Unsupported> {
        DefaultQueryDispatcher.contact(p, a, b, prediction)
    }
    fn closest_points(
        &self,
        p: &Iso,
        a: &dyn Shape,
        b: &dyn Shape,
        max: f32,
    ) -> Result<query::ClosestPoints, query::Unsupported> {
        DefaultQueryDispatcher.closest_points(p, a, b, max)
    }
    fn cast_shapes(
        &self,
        p: &Iso,
        velocity: Vector,
        a: &dyn Shape,
        b: &dyn Shape,
        options: ShapeCastOptions,
    ) -> Result<Option<query::ShapeCastHit>, query::Unsupported> {
        if a.as_composite_shape().is_none() && b.as_composite_shape().is_none() {
            self.leaves
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        if (a.as_ball().is_some() && b.as_ball().is_some())
            || a.as_shape::<parry3d::shape::HalfSpace>().is_some()
            || b.as_shape::<parry3d::shape::HalfSpace>().is_some()
            || a.as_heightfield().is_some()
            || b.as_heightfield().is_some()
            || a.as_voxels().is_some()
            || b.as_voxels().is_some()
        {
            return DefaultQueryDispatcher.cast_shapes(p, velocity, a, b, options);
        }
        if a.as_support_map().is_some() && b.as_support_map().is_some() {
            return DefaultQueryDispatcher.cast_shapes(p, velocity, a, b, options);
        }
        if let Some(composite) = a.as_composite_shape() {
            return Ok(self.composite_cast(p, velocity, composite, b, options));
        }
        if let Some(composite) = b.as_composite_shape() {
            return Ok(self
                .composite_cast(
                    &p.inverse(),
                    -(p.rotation.inverse() * velocity),
                    composite,
                    a,
                    options,
                )
                .map(|h| h.swapped()));
        }
        DefaultQueryDispatcher.cast_shapes(p, velocity, a, b, options)
    }
    fn cast_shapes_nonlinear(
        &self,
        m1: &NonlinearRigidMotion,
        a: &dyn Shape,
        m2: &NonlinearRigidMotion,
        b: &dyn Shape,
        start: f32,
        end: f32,
        stop: bool,
    ) -> Result<Option<query::ShapeCastHit>, query::Unsupported> {
        DefaultQueryDispatcher.cast_shapes_nonlinear(m1, a, m2, b, start, end, stop)
    }
}

impl<'a> FilteredLinearDispatcher<'a> {
    fn new(supports: &'a SupportCache) -> Self {
        Self {
            supports,
            filter_nodes: true,
            nodes: Default::default(),
            rejects: Default::default(),
            leaves: Default::default(),
        }
    }
    // Adapted from Parry 0.27 CompositeShapeRef::cast_shape (Apache-2.0).
    // The same SAH tree, ray-AABB costs, part transforms and leaf dispatcher
    // remain. The extra certificate only removes impossible swept subtrees.
    fn composite_cast<C: TypedCompositeShape + ?Sized>(
        &self,
        p: &Iso,
        velocity: Vector,
        composite: &C,
        other: &dyn Shape,
        options: ShapeCastOptions,
    ) -> Option<query::ShapeCastHit> {
        use parry3d::query::{Ray, RayCast};
        let aabb = other.compute_aabb(p);
        let ray = Ray::new(Vector::ZERO, velocity);
        let shift = -aabb.center();
        let margin = aabb.half_extents() + Vector::splat(options.target_distance);
        let planes = if self.filter_nodes {
            self.supports
                .get(other)
                .and_then(|index| {
                    index.projection_planes(
                        p,
                        &composite.bvh().root_aabb(),
                        options.target_distance,
                        0.001,
                    )
                })
                .map(|planes| planes.swept(velocity, options.max_time_of_impact))
        } else {
            None
        };
        composite
            .bvh()
            .find_best(
                options.max_time_of_impact,
                |node, best| {
                    self.nodes
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let bounds = Aabb {
                        mins: node.mins() + shift - margin,
                        maxs: node.maxs() + shift + margin,
                    };
                    let cost = bounds.cast_local_ray(&ray, best, true).unwrap_or(f32::MAX);
                    if self.filter_nodes
                        && cost < best
                        && planes
                            .as_ref()
                            .is_some_and(|p| p.excludes(node.mins(), node.maxs()))
                    {
                        self.rejects
                            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        f32::MAX
                    } else {
                        cost
                    }
                },
                |part, _| {
                    composite.map_untyped_part_at(part, |pose, shape, _| {
                        if let Some(pose) = pose {
                            self.cast_shapes(
                                &pose.inv_mul(p),
                                pose.rotation.inverse() * velocity,
                                shape,
                                other,
                                options,
                            )
                            .ok()?
                            .map(|hit| hit.transform1_by(pose))
                        } else {
                            self.cast_shapes(p, velocity, shape, other, options).ok()?
                        }
                    })?
                },
            )
            .map(|(_, hit)| hit)
    }
}

// Exact query memo: only identical relative-transform bits and immutable shape
// allocations share an answer. Receipts retain both shapes to prevent address
// reuse. Cleared each resolve; the entry cap limits storage only, never work.
type ContactKey = (usize, usize, [u32; 8]);
struct ContactReceipt {
    _shapes: [Arc<dyn Shape>; 2],
    contact: Option<query::Contact>,
}
#[derive(Default)]
struct ContactWorkspace {
    executor: Option<execution::Executor>,
    context: execution::Context,
    epa: Option<EPA>,
    contacts: BTreeMap<ContactKey, ContactReceipt>,
    hits: u64,
    misses: u64,
    supports: SupportCache,
}
fn contact_key(relative: &Iso, a: &Geometry, b: &Geometry, prediction: f32) -> ContactKey {
    let t = relative.translation.to_array().map(f32::to_bits);
    let q = relative.rotation.to_array().map(f32::to_bits);
    (
        Arc::as_ptr(&a.shape) as *const () as usize,
        Arc::as_ptr(&b.shape) as *const () as usize,
        [
            t[0],
            t[1],
            t[2],
            q[0],
            q[1],
            q[2],
            q[3],
            prediction.to_bits(),
        ],
    )
}
fn cached_world_contact(
    workspace: &mut ContactWorkspace,
    pa: &Iso,
    a: &Geometry,
    pb: &Iso,
    b: &Geometry,
    prediction: f32,
) -> Result<Option<query::Contact>, query::Unsupported> {
    let relative = pa.inv_mul(pb);
    let key = contact_key(&relative, a, b, prediction);
    let mut contact = if let Some(receipt) = workspace.contacts.get(&key) {
        workspace.hits += 1;
        receipt.contact
    } else {
        workspace.misses += 1;
        workspace.supports.register(&a.shape);
        workspace.supports.register(&b.shape);
        let contact = reusable_contact(
            workspace.epa.get_or_insert_with(EPA::new),
            &workspace.supports,
            &relative,
            a.shape.as_ref(),
            b.shape.as_ref(),
            prediction,
        )?;
        if workspace.contacts.len() < 8192 {
            workspace.contacts.insert(
                key,
                ContactReceipt {
                    _shapes: [a.shape.clone(), b.shape.clone()],
                    contact,
                },
            );
        }
        contact
    };
    if let Some(c) = &mut contact {
        c.transform_by_mut(pa, pb);
    }
    Ok(contact)
}

fn nondeepening_recovery(
    a: &Geometry,
    from: RigidPose,
    to: RigidPose,
    b: &Geometry,
    bfrom: RigidPose,
    bto: RigidPose,
    workspace: &mut ContactWorkspace,
) -> Result<bool, MorphError> {
    #[cfg(feature = "collision-fixture")]
    let mut recovery_trace = fixtures::RecoveryCapture::new(a, b, [from, to, bfrom, bto]);
    #[cfg(feature = "allocation-attribution")]
    let _allocation_scope = crate::profiling::enter(5);

    if from == to && bfrom == bto {
        return Ok(false);
    }
    let same_rotation = |p: RigidPose, q: RigidPose| {
        p.yaw_sin == q.yaw_sin && p.yaw_cos == q.yaw_cos && p.axial_roll == q.axial_roll
    };
    if !same_rotation(from, to) || !same_rotation(bfrom, bto) {
        return Ok(false);
    }
    let origin = Position::from_absolute(from.translation)?;
    let am = motion(from, to, origin)?;
    let bm = motion(bfrom, bto, origin)?;
    let mut contact_at = |t| {
        #[cfg(feature = "collision-fixture")]
        let started = Instant::now();
        let result = cached_world_contact(
            workspace,
            &am.position_at_time(t),
            a,
            &bm.position_at_time(t),
            b,
            0.001,
        )
        .map_err(|_| MorphError("unsupported overlap recovery query".into()));
        #[cfg(feature = "collision-fixture")]
        if let Ok(contact) = &result {
            recovery_trace.sample(t, contact.map(|c| c.dist), started.elapsed().as_nanos());
        }
        result
    };
    let Some(first) = contact_at(0.)? else {
        return Ok(false);
    };
    let relative = am.linvel - bm.linvel;
    if am.angvel == Vector::ZERO
        && bm.angvel == Vector::ZERO
        && relative.dot(first.normal1) > 0.00001
    {
        return Ok(false);
    }
    let poses = [from, to, bfrom, bto];
    let mut frontier = if workspace
        .executor
        .as_ref()
        .is_some_and(|e| e.policy.recovery)
    {
        let tasks = [0.25, 0.5, 0.75, 1.]
            .into_iter()
            .enumerate()
            .filter(|(_, t)| {
                !workspace.contacts.contains_key(&contact_key(
                    &am.position_at_time(*t).inv_mul(&bm.position_at_time(*t)),
                    a,
                    b,
                    0.001,
                ))
            })
            .map(|(i, t)| {
                execution::Task::contact(
                    workspace.context.clone(),
                    i,
                    a,
                    b,
                    poses,
                    (am.position_at_time(t), bm.position_at_time(t), t),
                )
            })
            .collect();
        workspace.executor.as_mut().unwrap().run(tasks)
    } else {
        None
    };
    let mut previous = first.dist;
    let mut outcome = Ok(true);
    for (ordinal, t) in [0.25, 0.5, 0.75, 1.].into_iter().enumerate() {
        let pa = am.position_at_time(t);
        let pb = bm.position_at_time(t);
        let ready = frontier.as_mut().and_then(|f| {
            if workspace
                .contacts
                .contains_key(&contact_key(&pa.inv_mul(&pb), a, b, 0.001))
            {
                return None;
            }
            let key = execution::Task::contact(
                workspace.context.clone(),
                ordinal,
                a,
                b,
                poses,
                (pa, pb, t),
            )
            .key();
            workspace.executor.as_mut().unwrap().consume(f, &key)
        });
        let fallback = ready.is_none();
        let started = Instant::now();
        let result = match ready {
            Some(Ok(execution::Value::Contact(contact))) => {
                if workspace.contacts.len() < 8192 {
                    workspace.contacts.insert(
                        contact_key(&pa.inv_mul(&pb), a, b, 0.001),
                        ContactReceipt {
                            _shapes: [a.shape.clone(), b.shape.clone()],
                            contact,
                        },
                    );
                }
                let mut world_contact = contact;
                if let Some(c) = &mut world_contact {
                    c.transform_by_mut(&pa, &pb);
                }
                Ok(world_contact)
            }
            Some(Err(error)) => Err(error),
            _ => cached_world_contact(workspace, &pa, a, &pb, b, 0.001)
                .map_err(|_| MorphError("unsupported overlap recovery query".into())),
        };
        if fallback && let Some(executor) = workspace.executor.as_mut() {
            executor.profile.fallback_ns += started.elapsed().as_nanos();
        }
        #[cfg(feature = "collision-fixture")]
        if let Ok(contact) = &result {
            recovery_trace.sample(t, contact.map(|c| c.dist), started.elapsed().as_nanos());
        }
        let contact = match result {
            Ok(contact) => contact,
            Err(error) => {
                outcome = Err(error);
                break;
            }
        };
        let depth = contact.map_or(0.001, |c| c.dist);
        if depth + 0.00001 < previous {
            outcome = Ok(false);
            break;
        }
        previous = depth;
    }
    if let Some(frontier) = frontier {
        workspace.executor.as_mut().unwrap().finish(frontier);
    }
    Ok(outcome? && previous + 0.00001 >= first.dist)
}
fn retreat(agent: &mut Agent, normal: [i64; 3]) {
    let h = Vector::from_array(agent.heading.map(|v| v as f32 / SCALE as f32)).normalize();
    let n = Vector::from_array(normal.map(|v| v as f32 / SCALE as f32)).normalize_or_zero();
    if n.length_squared() < 0.5 {
        return;
    }
    // A curved pipe can reject a shallow "away" direction as well. Turn
    // toward clearance after an actual refusal instead of doing nothing
    // whenever the first normal component is already slightly outward.
    let away = -n;
    let alignment = h.dot(away).clamp(-1., 1.);
    let mut tangent = (away - h * alignment).normalize_or_zero();
    if tangent.length_squared() < 0.5 {
        // Head-on contact has no preferred side. Choose one deterministically;
        // preserve directional memory instead of reversing heading in one tick.
        tangent = h
            .cross(if h.y.abs() < 0.9 {
                Vector::Y
            } else {
                Vector::X
            })
            .normalize();
    }
    let angle = alignment.acos().min(0.08);
    agent.heading = (h * angle.cos() + tangent * angle.sin())
        .to_array()
        .map(|v| (v * SCALE as f32).round() as i64);
}

fn frozen(world: &WorldSnapshot, id: u64) -> Result<RigidPose, MorphError> {
    world.body_pose(id)
}

/// Detection and response are deliberately separate. Rejected rigid paths are
/// never partly committed. Recheck after each freeze to handle mutually moving bodies.
pub(crate) fn resolve(
    before: &WorldSnapshot,
    next: &mut WorldSnapshot,
    plans: &mut BTreeMap<u64, ConstructorState>,
    cache: &mut CollisionCache,
) -> Result<(), MorphError> {
    #[cfg(feature = "allocation-attribution")]
    let _allocation_scope = crate::profiling::enter(4);

    if !next.habitat.as_ref().is_some_and(|h| h.rules.collisions) {
        return Ok(());
    }
    let started = Instant::now();
    cache.casts.clear();
    cache.discard_deposit_geometry();
    cache
        .promoted_geometry
        .retain(|key, hash| cache.pipes.get(key).is_some_and(|p| p.0 == *hash));
    cache.contact_workspace.contacts.clear();
    require(
        next.config.environment == crate::emission::Environment::suspended(),
        "unsupported force field in collision mode",
    )?;
    next.habitat.as_mut().unwrap().contacts.clear();
    let ids = before.instances.keys().copied().collect::<Vec<_>>();
    let mut shapes = BTreeMap::new();
    let mut targets = BTreeMap::new();
    let mut starts = BTreeMap::new();
    for id in &ids {
        shapes.insert(*id, cache.body(before, *id)?);
        let old = frozen(before, *id)?;
        starts.insert(*id, old);
        targets.insert(
            *id,
            plans.get(id).map_or(
                RigidPose {
                    translation: next.instances[id].position.absolute()?,
                    ..old
                },
                |s| s.rigid,
            ),
        );
    }
    let mut stopped = BTreeSet::new();
    let mut stopping_normals = BTreeMap::new();
    let mut reported = BTreeSet::new();
    let hunters = next
        .habitat
        .as_ref()
        .unwrap()
        .agents
        .keys()
        .copied()
        .collect::<BTreeSet<_>>();
    let mut settled = false;
    // First compose one bounded avoidance proposal. Later passes only verify
    // or freeze it, so each changed verification pass stops another moving
    // target instead of oscillating between expensive alternate corrections.
    for pass in 0..=ids.len() + 1 {
        let pass_targets = targets.clone();
        for (index, &a) in ids.iter().enumerate() {
            for &b in &ids[index + 1..] {
                let pa = starts[&a];
                let pb = starts[&b];
                let ta = targets[&a];
                let tb = targets[&b];
                if pa == ta && pb == tb {
                    continue;
                }
                let da = vec(std::array::from_fn(|k| {
                    ta.translation[k] - pa.translation[k]
                }))
                .length();
                let db = vec(std::array::from_fn(|k| {
                    tb.translation[k] - pb.translation[k]
                }))
                .length();
                if !near(pa, pb, shapes[&a].radius, shapes[&b].radius, da, db) {
                    continue;
                }
                #[cfg(feature = "query-trace")]
                query_trace::set_pair(&format!("t{}:body:{a}:body:{b}", before.tick), 0);
                if cache.contact_workspace.executor.is_some() {
                    cache.contact_workspace.context = execution::Context {
                        tick: before.tick,
                        pass,
                        body: a,
                        target: Some(PhysicalTarget::Organism { id: b }),
                    };
                }
                #[cfg(feature = "collision-fixture")]
                fixtures::recovery_pair(before.tick, a, PhysicalTarget::Organism { id: b });
                if let Some(hit) = cache.cast(&shapes[&a], pa, ta, &shapes[&b], pb, tb)? {
                    let prey = |id: u64| before.instances[&id].definition == BLOOD_SAC;
                    let hunter = |id: u64| hunters.contains(&id);
                    // Prey is not a wall. A bite is recorded and both keep their step.
                    let response = if (hunter(a) && prey(b)) || (hunter(b) && prey(a)) {
                        ContactResponse::FeedingContact
                    } else {
                        ContactResponse::Blocked
                    };
                    if reported.insert((a, PhysicalTarget::Organism { id: b })) {
                        record(next, a, PhysicalTarget::Organism { id: b }, hit, response);
                    }
                    if reported.insert((b, PhysicalTarget::Organism { id: a })) {
                        record(
                            next,
                            b,
                            PhysicalTarget::Organism { id: a },
                            (hit.0, hit.1.map(|x| -x), hit.2),
                            response,
                        );
                    }
                    if response == ContactResponse::Blocked {
                        if hit.0 <= 0.00001
                            && nondeepening_recovery(
                                &shapes[&a],
                                pa,
                                ta,
                                &shapes[&b],
                                pb,
                                tb,
                                &mut cache.contact_workspace,
                            )?
                        {
                            continue;
                        }
                        stopping_normals.insert(a, hit.1);
                        stopping_normals.insert(b, hit.1.map(|x| -x));
                        if pass == 0 && hunter(a) && hunter(b) {
                            let slid_a = along_surface(pa, ta, hit.1);
                            let slid_b = along_surface(pb, tb, hit.1.map(|x| x.saturating_neg()));
                            if slid_a.translation != pa.translation && slid_a != ta {
                                targets.insert(a, slid_a);
                            } else {
                                targets.insert(a, pa);
                                stopped.insert(a);
                                stopping_normals.insert(a, hit.1);
                            }
                            if slid_b.translation != pb.translation && slid_b != tb {
                                targets.insert(b, slid_b);
                            } else {
                                targets.insert(b, pb);
                                stopped.insert(b);
                                stopping_normals.insert(b, hit.1.map(|x| -x));
                            }
                        } else {
                            targets.insert(a, pa);
                            targets.insert(b, pb);
                            stopped.extend([a, b]);
                            stopping_normals.insert(a, hit.1);
                            stopping_normals.insert(b, hit.1.map(|x| -x));
                        }
                    }
                }
            }
        }
        for &id in &ids {
            let start = starts[&id];
            let target = targets[&id];
            if start == target {
                continue;
            }
            let delta = vec(std::array::from_fn(|k| {
                target.translation[k] - start.translation[k]
            }))
            .length();
            let mut static_shapes = Vec::new();
            for solid in &before.solids {
                let min = solid.min.absolute()?;
                let max = solid.max.absolute()?;
                let center = std::array::from_fn(|k| min[k] + (max[k] - min[k]) / 2);
                let geometry = &shapes[&id];
                let clearance = if let Some(value) = geometry.writhe_slack.get() {
                    *value
                } else {
                    let value = presentation_slack(
                        &before.config.definitions[&before.instances[&id].definition],
                    )?;
                    let _ = geometry.writhe_slack.set(value);
                    value
                };
                let half =
                    vec(std::array::from_fn(|k| (max[k] - min[k]) / 2)) + Vector::splat(clearance);
                let pose = RigidPose {
                    translation: center,
                    ..RigidPose::identity()
                };
                static_shapes.push((
                    PhysicalTarget::Solid { id: solid.id },
                    pose,
                    Geometry {
                        shape: Arc::new(Cuboid::new(half)),
                        radius: half.length(),
                        vertices: Arc::new(vec![Vector::ZERO]),
                        parts: vec![],
                        outlets: vec![],
                        writhe_slack: OnceLock::from(0.),
                    },
                ));
            }
            if let Some(assembly) = &before.assembly {
                // The horizon visits local cells. The sample sphere and the
                // hollow trimesh remain the contact authority.
                let at = Position::from_absolute(start.translation)?;
                let reach = shapes[&id].radius + delta;
                for (filament_id, chunk) in
                    cache.interacting_chunks(assembly, before.tick, at, reach)?
                {
                    let f = &assembly.filaments[&filament_id];
                    if pipe_separated(cache, f, chunk, before.tick, at, reach)? {
                        continue;
                    }
                    let (origin, g) = cache.pipe(before, f.id, chunk)?;
                    static_shapes.push((
                        PhysicalTarget::Pipe {
                            filament: f.id,
                            chunk,
                        },
                        RigidPose {
                            translation: origin.absolute()?,
                            ..RigidPose::identity()
                        },
                        g,
                    ));
                }
            }
            if let Some(h) = &before.habitat {
                for (&sac, attachment) in &h.sacs {
                    if sac == id {
                        continue;
                    }
                    if let Some(anchor) = crate::habitat::anchor(before, attachment) {
                        let old = before.instances[&sac].position;
                        let new = next.instances[&sac].position;
                        let enlargement = vec(new.relative(old)?).length();
                        let g = root_geometry(
                            anchor,
                            old,
                            crate::habitat::ROOT_RADIUS as f32 / SCALE as f32 + enlargement,
                            false,
                        )?;
                        static_shapes.push((
                            PhysicalTarget::Root { sac },
                            RigidPose {
                                translation: anchor.absolute()?,
                                ..RigidPose::identity()
                            },
                            g,
                        ));
                    }
                }
            }
            for (&owner, body) in &before.instances {
                for parcel in &body.book.parcels {
                    let pose = RigidPose {
                        translation: parcel.position,
                        ..RigidPose::identity()
                    };
                    let g = Geometry {
                        shape: Arc::new(Ball::new(0.025)),
                        radius: 0.025,
                        vertices: Arc::new(vec![Vector::ZERO]),
                        parts: vec![],
                        outlets: vec![],
                        writhe_slack: OnceLock::from(0.),
                    };
                    static_shapes.push((
                        PhysicalTarget::Parcel {
                            organism: owner,
                            parcel: parcel.id,
                        },
                        pose,
                        g,
                    ));
                }
            }
            let mut static_frontier = None;
            let mut static_ordinals = Vec::new();
            let mut prefetched_until = 0;
            for (static_index, (owner, pose, g)) in static_shapes.iter().enumerate() {
                let owner = owner.clone();
                let pose = *pose;
                let target = targets[&id];
                if target == start {
                    break;
                }
                let delta = vec(std::array::from_fn(|k| {
                    target.translation[k] - start.translation[k]
                }))
                .length();
                if cache
                    .contact_workspace
                    .executor
                    .as_ref()
                    .is_some_and(|e| e.policy.static_queries)
                    && static_index >= prefetched_until
                {
                    if let Some(frontier) = static_frontier.take() {
                        cache
                            .contact_workspace
                            .executor
                            .as_mut()
                            .unwrap()
                            .finish(frontier);
                    }
                    static_ordinals.clear();
                    let limit = cache
                        .contact_workspace
                        .executor
                        .as_ref()
                        .unwrap()
                        .policy
                        .frontier_limit;
                    let mut tasks = Vec::new();
                    for (index, (target_owner, target_pose, geometry)) in
                        static_shapes.iter().enumerate().skip(static_index)
                    {
                        if near(
                            start,
                            *target_pose,
                            shapes[&id].radius,
                            geometry.radius,
                            delta,
                            0.,
                        ) && !cache.casts.contains_key(&cast_key(
                            &shapes[&id],
                            geometry,
                            [start, target, *target_pose, *target_pose],
                        )) {
                            static_ordinals.push(index);
                            tasks.push(execution::Task::cast(
                                execution::Context {
                                    tick: before.tick,
                                    pass,
                                    body: id,
                                    target: Some(target_owner.clone()),
                                },
                                tasks.len(),
                                0,
                                &shapes[&id],
                                geometry,
                                [start, target, *target_pose, *target_pose],
                            ));
                            if tasks.len() == limit {
                                break;
                            }
                        }
                    }
                    prefetched_until = static_ordinals
                        .last()
                        .map_or(static_shapes.len(), |i| i + 1);
                    static_frontier = cache
                        .contact_workspace
                        .executor
                        .as_mut()
                        .unwrap()
                        .run(tasks);
                }
                if !near(start, pose, shapes[&id].radius, g.radius, delta, 0.) {
                    continue;
                }
                #[cfg(feature = "query-trace")]
                trace_static_pair(before.tick, id, &owner, before.assembly.as_ref());
                if cache.contact_workspace.executor.is_some() {
                    cache.contact_workspace.context = execution::Context {
                        tick: before.tick,
                        pass,
                        body: id,
                        target: Some(owner.clone()),
                    };
                }
                #[cfg(feature = "collision-fixture")]
                fixtures::recovery_pair(before.tick, id, owner.clone());
                #[cfg(feature = "collision-fixture")]
                let pending_fixture = fixtures::begin(
                    before,
                    id,
                    &owner,
                    [start, target, pose, pose],
                    [&shapes[&id], g],
                );
                let ready = static_frontier.as_mut().and_then(|frontier| {
                    let ordinal = static_ordinals
                        .iter()
                        .position(|index| *index == static_index)?;
                    let key = execution::Task::cast(
                        cache.contact_workspace.context.clone(),
                        ordinal,
                        0,
                        &shapes[&id],
                        g,
                        [start, target, pose, pose],
                    )
                    .key();
                    cache
                        .contact_workspace
                        .executor
                        .as_mut()
                        .unwrap()
                        .consume(frontier, &key)
                });
                let fallback = ready.is_none();
                let started = Instant::now();
                let cast_result = match ready {
                    Some(Ok(execution::Value::Cast(hit))) => {
                        if cache.casts.len() < 8192 {
                            cache.casts.insert(
                                cast_key(&shapes[&id], g, [start, target, pose, pose]),
                                CastReceipt {
                                    _shapes: [shapes[&id].shape.clone(), g.shape.clone()],
                                    hit,
                                },
                            );
                        }
                        Ok(hit)
                    }
                    Some(Err(error)) => Err(error),
                    _ => cache.cast(&shapes[&id], start, target, g, pose, pose),
                };
                if fallback && let Some(executor) = cache.contact_workspace.executor.as_mut() {
                    executor.profile.fallback_ns += started.elapsed().as_nanos();
                }
                #[cfg(feature = "collision-fixture")]
                if let Some(pending) = pending_fixture {
                    fixtures::finish(pending, &cast_result);
                }
                if let Some(hit) = cast_result? {
                    let source_nozzle = match &owner {
                        PhysicalTarget::Pipe { filament, .. } => {
                            let f = &before.assembly.as_ref().unwrap().filaments[filament];
                            let at_outlet = shapes[&id]
                                .outlets
                                .iter()
                                .find(|(nozzle, _)| *nozzle == f.nozzle)
                                .map(|(_, p)| crate::emission::place_point(*p, start))
                                .transpose()?
                                .is_some_and(|p| {
                                    f.nodes
                                        .last()
                                        .unwrap()
                                        .position
                                        .relative(Position::from_absolute(p).unwrap())
                                        .is_ok_and(|d| {
                                            d.iter().map(|x| i128::from(*x).pow(2)).sum::<i128>()
                                                <= i128::from(150_000).pow(2)
                                        })
                                });
                            (f.organism == id && at_outlet).then_some(f.nozzle)
                        }
                        PhysicalTarget::Parcel { organism, parcel } if *organism == id => before
                            .instances[&id]
                            .book
                            .parcels
                            .iter()
                            .find(|p| p.id == *parcel)
                            .map(|p| p.nozzle_id),
                        _ => None,
                    };
                    // Contact with the secreting outlet is a declared graft.
                    // Query every other anatomical component rather than infer
                    // permission from an approximate whole-body contact point.
                    let mut source = source_nozzle.is_some();
                    if let Some(nozzle) = source_nozzle {
                        if cache
                            .contact_workspace
                            .executor
                            .as_ref()
                            .is_some_and(|e| e.policy.graft)
                        {
                            source = cache.parallel_graft(
                                &shapes[&id],
                                nozzle,
                                g,
                                [start, target, pose, pose],
                            )?;
                        } else {
                            for (component, part) in &shapes[&id].parts {
                                if *component != nozzle && {
                                    #[cfg(feature = "query-trace")]
                                    query_trace::set_pair(
                                        &format!(
                                            "t{}:body:{id}:part:{component}:{}",
                                            before.tick,
                                            owner_label(&owner)
                                        ),
                                        0,
                                    );
                                    cache.cast(part, start, target, g, pose, pose)?.is_some()
                                } {
                                    source = false;
                                    break;
                                }
                            }
                        }
                    }
                    let prey_root = matches!(
                        &owner,
                        PhysicalTarget::Root { sac }
                            if before.instances.get(sac).is_some_and(|body| body.definition == BLOOD_SAC)
                    );
                    let response = if source {
                        ContactResponse::SourceAttachment
                    } else if prey_root {
                        ContactResponse::FeedingContact
                    } else {
                        ContactResponse::Blocked
                    };
                    if reported.insert((id, owner.clone())) {
                        record(next, id, owner.clone(), hit, response);
                    }
                    if prey_root {
                        // The graft holds the sac. It is not a wall for the builder.
                        if let PhysicalTarget::Root { sac } = &owner {
                            targets.insert(*sac, starts[sac]);
                            stopped.insert(*sac);
                            if let Some(old) = before.habitat.as_ref().unwrap().sacs.get(sac) {
                                next.habitat
                                    .as_mut()
                                    .unwrap()
                                    .sacs
                                    .insert(*sac, old.clone());
                            }
                        }
                        continue;
                    }
                    if !source {
                        if hit.0 <= 0.00001
                            && nondeepening_recovery(
                                &shapes[&id],
                                start,
                                target,
                                g,
                                pose,
                                pose,
                                &mut cache.contact_workspace,
                            )?
                        {
                            continue;
                        }
                        stopping_normals.insert(id, hit.1);
                        if pass == 0
                            && matches!(
                                owner,
                                PhysicalTarget::Pipe { .. } | PhysicalTarget::Parcel { .. }
                            )
                            && next.habitat.as_ref().unwrap().agents.contains_key(&id)
                        {
                            let slid = along_surface(start, target, hit.1);
                            if slid.translation != start.translation && slid != target {
                                targets.insert(id, slid);
                                continue;
                            }
                        }
                        targets.insert(id, start);
                        stopped.insert(id);
                        stopping_normals.insert(id, hit.1);
                        break;
                    }
                }
            }
            if let Some(frontier) = static_frontier {
                cache
                    .contact_workspace
                    .executor
                    .as_mut()
                    .unwrap()
                    .finish(frontier);
            }
        }
        if targets == pass_targets {
            settled = true;
            break;
        }
    }
    // Never commit a last, unchecked correction when the bounded solver fails
    // to settle. Keep all starts; no authoritative tick or material is skipped.
    if !settled {
        targets.clone_from(&starts);
        stopped.extend(ids.iter().copied());
    }
    for id in ids {
        let target = targets[&id];
        let feeding = next
            .habitat
            .as_ref()
            .unwrap()
            .contacts
            .iter()
            .any(|c| c.organism == id && c.response == ContactResponse::FeedingContact);
        let blocking = next
            .habitat
            .as_ref()
            .unwrap()
            .contacts
            .iter()
            .find(|c| c.organism == id && c.response == ContactResponse::Blocked)
            .cloned();
        if let Some(agent) = next.habitat.as_mut().unwrap().agents.get_mut(&id) {
            agent.rigid = target;
            if stopped.contains(&id) {
                agent.eating = feeding;
                if let Some(normal) = stopping_normals
                    .get(&id)
                    .copied()
                    .or_else(|| blocking.map(|c| c.normal))
                {
                    retreat(agent, normal);
                }
            }
            if let Some(plan) = plans.get_mut(&id) {
                plan.rigid = agent.rigid;
                plan.seeking_food = agent.eating;
            }
        } else {
            next.instances.get_mut(&id).unwrap().position =
                Position::from_absolute(target.translation)?;
            next.instances.get_mut(&id).unwrap().exposure = crate::runtime::exposure(
                &next.config,
                Position::from_absolute(target.translation)?,
            )?;
        }
    }
    admit_roots(before, next, cache)?;
    require(
        next.habitat.as_ref().unwrap().contacts.len() <= 512,
        "contact budget exceeded",
    )?;
    cache.profile.resolve_calls += 1;
    cache.profile.resolve_ns += started.elapsed().as_nanos();
    Ok(())
}

/// Contact geometry can also be tested independently of any policy/response.
pub fn swept_ball_against_surface(
    from: [f32; 3],
    to: [f32; 3],
    radius: f32,
    surface: &Surface,
) -> Result<bool, MorphError> {
    let a = Geometry {
        shape: Arc::new(Ball::new(radius)),
        radius,
        vertices: Arc::new(vec![Vector::ZERO]),
        parts: vec![],
        outlets: vec![],
        writhe_slack: OnceLock::from(0.),
    };
    let b = geometry(std::slice::from_ref(surface), 1.)?;
    let p = |v: [f32; 3]| RigidPose {
        translation: v.map(|x| (x * SCALE as f32).round() as i64),
        ..RigidPose::identity()
    };
    let mut profile = CollisionProfile::default();
    Ok(cast(
        &a,
        p(from),
        p(to),
        &b,
        RigidPose::identity(),
        RigidPose::identity(),
        (&mut profile, &mut ContactWorkspace::default()),
    )?
    .is_some())
}

fn overlaps(a: &Geometry, pa: RigidPose, b: &Geometry, pb: RigidPose) -> Result<bool, MorphError> {
    #[cfg(feature = "allocation-attribution")]
    let _allocation_scope = crate::profiling::enter(16);
    if !near(pa, pb, a.radius, b.radius, 0., 0.) {
        return Ok(false);
    }
    let origin = Position::from_absolute(pa.translation)?;
    let aa = motion(pa, pa, origin)?.start;
    let bb = motion(pb, pb, origin)?.start;
    let aabb = a.shape.compute_aabb(&aa).loosened(0.0001);
    let babb = b.shape.compute_aabb(&bb).loosened(0.0001);
    if !aabb.intersects(&babb) {
        return Ok(false);
    }
    if query::intersection_test(&aa, a.shape.as_ref(), &bb, b.shape.as_ref())
        .map_err(|_| MorphError("unsupported overlap query".into()))?
    {
        return Ok(true);
    }
    // Convex component queries already include volumetric containment. Avoid
    // thousands of redundant point tests for body, parcel and root envelopes.
    if a.shape.as_trimesh().is_none() && b.shape.as_trimesh().is_none() {
        return Ok(false);
    }
    // Surface-only intersection tests miss complete containment. Oriented closed
    // canonical component surfaces also supply the occupied-side point test.
    #[cfg(feature = "allocation-attribution")]
    let _containment_scope = crate::profiling::enter(17);
    Ok(a.vertices.iter().any(|v| {
        let p = aa * (*v);
        babb.contains_local_point(p) && b.shape.contains_point(&bb, p)
    }) || b.vertices.iter().any(|v| {
        let p = bb * (*v);
        aabb.contains_local_point(p) && a.shape.contains_point(&aa, p)
    }))
}
fn clear_shape(
    world: &WorldSnapshot,
    g: &Geometry,
    pose: RigidPose,
    skip: Option<u64>,
    cache: &mut CollisionCache,
) -> Result<bool, MorphError> {
    #[cfg(feature = "allocation-attribution")]
    let _allocation_scope = crate::profiling::enter(15);

    for id in world
        .instances
        .keys()
        .copied()
        .filter(|id| Some(*id) != skip)
    {
        let other = cache.body(world, id)?;
        if overlaps(g, pose, &other, world.body_pose(id)?)? {
            return Ok(false);
        }
    }
    for solid in &world.solids {
        let min = solid.min.absolute()?;
        let max = solid.max.absolute()?;
        let center = std::array::from_fn(|k| min[k] + (max[k] - min[k]) / 2);
        let half = vec(std::array::from_fn(|k| (max[k] - min[k]) / 2));
        let other = Geometry {
            shape: Arc::new(Cuboid::new(half)),
            radius: half.length(),
            vertices: Arc::new(vec![Vector::ZERO]),
            parts: vec![],
            outlets: vec![],
            writhe_slack: OnceLock::from(0.),
        };
        if overlaps(
            g,
            pose,
            &other,
            RigidPose {
                translation: center,
                ..RigidPose::identity()
            },
        )? {
            return Ok(false);
        }
    }
    if let Some(a) = &world.assembly {
        let at = Position::from_absolute(pose.translation)?;
        for (filament_id, chunk) in cache.interacting_chunks(a, world.tick, at, g.radius)? {
            let f = &a.filaments[&filament_id];
            if pipe_separated(cache, f, chunk, world.tick, at, g.radius)? {
                continue;
            }
            let (origin, other) = cache.pipe(world, f.id, chunk)?;
            if overlaps(
                g,
                pose,
                &other,
                RigidPose {
                    translation: origin.absolute()?,
                    ..RigidPose::identity()
                },
            )? {
                return Ok(false);
            }
        }
    }
    Ok(true)
}
/// Bounded placement is an authored initialization/birth operation, never a
/// teleport response during ordinary motion. No stock is debited on refusal.
pub(crate) fn spawn_pose(
    world: &WorldSnapshot,
    definition: u64,
    near: RigidPose,
    skip: Option<u64>,
    cache: &mut CollisionCache,
) -> Result<Option<RigidPose>, MorphError> {
    #[cfg(feature = "allocation-attribution")]
    let _allocation_scope = crate::profiling::enter(6);

    let source = world
        .instances
        .values()
        .find(|i| i.definition == definition)
        .ok_or_else(|| MorphError("spawn definition lacks a physical source".into()))?
        .id;
    let g = cache.body(world, source)?;
    let distances = if definition == BLOOD_SAC {
        vec![0.25, 0.5, 0.8, 1.2, 2.]
    } else {
        vec![1.5, 3., 5., 8., 12.]
    };
    for distance in distances {
        for k in 0..24 {
            let theta = k as f64 * std::f64::consts::TAU / 24.;
            let mut pose = near;
            pose.translation = Position::from_absolute(near.translation)?
                .offset([
                    (theta.cos() * distance * 1e6).round() as i64,
                    (if k % 3 == 0 {
                        distance * 0.5
                    } else {
                        -distance * 0.25
                    } * 1e6)
                        .round() as i64,
                    (theta.sin() * distance * 1e6).round() as i64,
                ])?
                .absolute()?;
            if clear_shape(world, &g, pose, skip, cache)? {
                return Ok(Some(pose));
            }
        }
    }
    Ok(None)
}
pub(crate) fn place_founders(world: &mut WorldSnapshot) -> Result<(), MorphError> {
    if !world.habitat.as_ref().is_some_and(|h| h.rules.collisions) {
        return Ok(());
    }
    let ids = world.instances.keys().copied().collect::<Vec<_>>();
    let mut cache = CollisionCache::default();
    for id in ids {
        if world.instances[&id].definition != BLOOD_SAC {
            continue;
        }
        let pose = world.body_pose(id)?;
        let g = cache.body(world, id)?;
        if !clear_shape(world, &g, pose, Some(id), &mut cache)? {
            let candidate = spawn_pose(world, BLOOD_SAC, pose, Some(id), &mut cache)?
                .ok_or_else(|| MorphError("no nonpenetrating founding food placement".into()))?;
            world.instances.get_mut(&id).unwrap().position =
                Position::from_absolute(candidate.translation)?;
            world.instances.get_mut(&id).unwrap().exposure = crate::runtime::exposure(
                &world.config,
                Position::from_absolute(candidate.translation)?,
            )?;
        }
    }
    Ok(())
}

pub fn swept_surface_against_box(
    surface: &Surface,
    from: RigidPose,
    to: RigidPose,
    obstacle: &SolidBox,
) -> Result<bool, MorphError> {
    let a = geometry(std::slice::from_ref(surface), 1.)?;
    let min = obstacle.min.absolute()?;
    let max = obstacle.max.absolute()?;
    require((0..3).all(|k| max[k] > min[k]), "invalid box")?;
    let center = std::array::from_fn(|k| min[k] + (max[k] - min[k]) / 2);
    let half = vec(std::array::from_fn(|k| (max[k] - min[k]) / 2));
    let b = Geometry {
        shape: Arc::new(Cuboid::new(half)),
        radius: half.length(),
        vertices: Arc::new(vec![Vector::ZERO]),
        parts: vec![],
        outlets: vec![],
        writhe_slack: OnceLock::from(0.),
    };
    let pose = RigidPose {
        translation: center,
        ..RigidPose::identity()
    };
    let mut profile = CollisionProfile::default();
    Ok(cast(
        &a,
        from,
        to,
        &b,
        pose,
        pose,
        (&mut profile, &mut ContactWorkspace::default()),
    )?
    .is_some())
}

/// Preflight a material-backed candidate before binding. The canonical emitter
/// alone is an authorised source attachment; every other anatomical part blocks.
pub(crate) fn deposit_allowed(
    world: &WorldSnapshot,
    assembly: &crate::filament::Assembly,
    book: &crate::emission::MaterialBook,
    r: &crate::filament::ConstructionRules,
    parcel: u64,
    cache: &mut CollisionCache,
) -> Result<bool, MorphError> {
    cache.discard_deposit_geometry();
    #[cfg(feature = "allocation-attribution")]
    let _allocation_scope = crate::profiling::enter(7);

    if !world.habitat.as_ref().is_some_and(|h| h.rules.collisions) {
        return Ok(true);
    }
    use crate::filament::{Filament, Node, PipeSection};
    let p = book
        .parcels
        .iter()
        .find(|p| p.id == parcel)
        .ok_or_else(|| MorphError("physical deposition source missing".into()))?;
    let position = Position::from_absolute(p.position)?;
    let last = assembly
        .filaments
        .values()
        .rev()
        .find(|f| f.organism == book.organism_id && f.nozzle == p.nozzle_id);
    let mut f = if let Some(f) = last.filter(|f| {
        !f.section.as_ref().is_some_and(|s| s.end_sealed)
            && f.nodes.last().is_some_and(|n| {
                n.position.relative(position).is_ok_and(|d| {
                    d.iter().map(|x| i128::from(*x).pow(2)).sum::<i128>()
                        <= i128::from(r.max_length).pow(2)
                })
            })
    }) {
        f.clone()
    } else {
        Filament {
            id: assembly.next_id,
            section: Some(PipeSection {
                outer_radius: 50_000,
                inner_radius: 31_000,
                start_sealed: true,
                end_sealed: false,
            }),
            organism: book.organism_id,
            nozzle: p.nozzle_id,
            created_tick: book.tick,
            experiment: r.experiment.clone(),
            material: p.material,
            quantity: 0,
            nodes: vec![],
        }
    };
    f.nodes.push(Node {
        id: f.nodes.last().map_or(1, |n| n.id + 1),
        parcel,
        position,
        deposited_tick: book.tick,
    });
    let mut candidate_chunks = Vec::new();
    let end = f.nodes.len().saturating_sub(2) / CHUNK_EDGES;
    for chunk in end.saturating_sub(1)..=end {
        let (origin, g) = if f.nodes.len() == 1 {
            (
                position,
                Geometry {
                    shape: Arc::new(Ball::new(0.05)),
                    radius: 0.05,
                    vertices: Arc::new(vec![Vector::ZERO]),
                    parts: vec![],
                    outlets: vec![],
                    writhe_slack: OnceLock::from(0.),
                },
            )
        } else {
            let (origin, surface) = filament_chunk_at(&f, chunk, world.tick)?;
            (origin, geometry(&[surface], 1.)?)
        };
        let pose = RigidPose {
            translation: origin.absolute()?,
            ..RigidPose::identity()
        };
        if cache.geometry_reuse
            && let Some(key) = pipe_key(f.id, &f, chunk, world.tick)
        {
            cache.geometry_reuse_profile.candidate_builds += 1;
            candidate_chunks.push((key, origin, g.clone()));
        }
        #[cfg(feature = "collision-fixture")]
        if let Some(key) = pipe_key(f.id, &f, chunk, world.tick) {
            fixtures::geometry_record("preflight", world.tick, key, &g, true, None);
        }
        for (&id, body) in &world.instances {
            let other = cache.body(world, id)?;
            let rigid = world.body_pose(id)?;
            if !near(pose, rigid, g.radius, other.radius, 0., 0.) {
                continue;
            }
            if id == book.organism_id {
                // The source outlet may graft into its own deposit. That does
                // not authorize the new pipe to pass through the trunk, pouch
                // or other appendages. Own pipe/pipe braiding stays permitted.
                for (component, part) in &other.parts {
                    if *component != p.nozzle_id && overlaps(&g, pose, part, rigid)? {
                        return Ok(false);
                    }
                }
            } else if overlaps(&g, pose, &other, rigid)? {
                return Ok(false);
            }
            let _ = body;
        }
        for (other_id, c) in cache.interacting_chunks(assembly, world.tick, origin, g.radius)? {
            let other = &assembly.filaments[&other_id];
            // Own-pipe braiding stays permitted. Another nozzle of the same
            // organism is not an obstruction for this deposit.
            if other.id == f.id || other.organism == book.organism_id {
                continue;
            }
            if pipe_separated(cache, other, c, world.tick, origin, g.radius)? {
                continue;
            }
            let (anchor, shape) = cache.pipe_in(assembly, world.tick, other.id, c)?;
            if overlaps(
                &g,
                pose,
                &shape,
                RigidPose {
                    translation: anchor.absolute()?,
                    ..RigidPose::identity()
                },
            )? {
                return Ok(false);
            }
        }
        for obstacle in &world.solids {
            let min = obstacle.min.absolute()?;
            let max = obstacle.max.absolute()?;
            let center = std::array::from_fn(|k| min[k] + (max[k] - min[k]) / 2);
            let half = vec(std::array::from_fn(|k| (max[k] - min[k]) / 2));
            let shape = Geometry {
                shape: Arc::new(Cuboid::new(half)),
                radius: half.length(),
                vertices: Arc::new(vec![Vector::ZERO]),
                parts: vec![],
                outlets: vec![],
                writhe_slack: OnceLock::from(0.),
            };
            if overlaps(
                &g,
                pose,
                &shape,
                RigidPose {
                    translation: center,
                    ..RigidPose::identity()
                },
            )? {
                return Ok(false);
            }
        }
    }
    if cache.geometry_reuse {
        cache.pending_geometry = Some(DepositGeometry {
            parcel,
            organism: book.organism_id,
            chunks: candidate_chunks,
        });
    }
    Ok(true)
}

pub(crate) fn emission_obstacle(
    world: &WorldSnapshot,
    assembly: &crate::filament::Assembly,
    organism: u64,
    nozzle: u64,
    origin: [i64; 3],
    cache: &mut CollisionCache,
) -> Result<Option<PhysicalTarget>, MorphError> {
    #[cfg(feature = "allocation-attribution")]
    let _allocation_scope = crate::profiling::enter(8);

    if !world.habitat.as_ref().is_some_and(|h| h.rules.collisions) {
        return Ok(None);
    }
    let g = Geometry {
        shape: Arc::new(Ball::new(0.025)),
        radius: 0.025,
        vertices: Arc::new(vec![Vector::ZERO]),
        parts: vec![],
        outlets: vec![],
        writhe_slack: OnceLock::from(0.),
    };
    let pose = RigidPose {
        translation: origin,
        ..RigidPose::identity()
    };
    for id in world.instances.keys().copied() {
        let other = cache.body(world, id)?;
        let rigid = world.body_pose(id)?;
        if !near(pose, rigid, g.radius, other.radius, 0., 0.) {
            continue;
        }
        if id == organism {
            for (part, shape) in &other.parts {
                if *part != nozzle && overlaps(&g, pose, shape, rigid)? {
                    return Ok(Some(PhysicalTarget::Organism { id }));
                }
            }
        } else if overlaps(&g, pose, &other, rigid)? {
            return Ok(Some(PhysicalTarget::Organism { id }));
        }
    }
    let at = Position::from_absolute(origin)?;
    for (filament_id, chunk) in cache.interacting_chunks(assembly, world.tick, at, g.radius)? {
        let f = &assembly.filaments[&filament_id];
        if f.organism == organism && f.nozzle == nozzle {
            continue;
        }
        if pipe_separated(cache, f, chunk, world.tick, at, g.radius)? {
            continue;
        }
        let (anchor, other) = cache.pipe_in(assembly, world.tick, f.id, chunk)?;
        if overlaps(
            &g,
            pose,
            &other,
            RigidPose {
                translation: anchor.absolute()?,
                ..RigidPose::identity()
            },
        )? {
            return Ok(Some(PhysicalTarget::Pipe {
                filament: f.id,
                chunk,
            }));
        }
    }
    for obstacle in &world.solids {
        let min = obstacle.min.absolute()?;
        let max = obstacle.max.absolute()?;
        let half = vec(std::array::from_fn(|k| (max[k] - min[k]) / 2));
        let center = std::array::from_fn(|k| min[k] + (max[k] - min[k]) / 2);
        let other = Geometry {
            shape: Arc::new(Cuboid::new(half)),
            radius: half.length(),
            vertices: Arc::new(vec![Vector::ZERO]),
            parts: vec![],
            outlets: vec![],
            writhe_slack: OnceLock::from(0.),
        };
        if overlaps(
            &g,
            pose,
            &other,
            RigidPose {
                translation: center,
                ..RigidPose::identity()
            },
        )? {
            return Ok(Some(PhysicalTarget::Solid { id: obstacle.id }));
        }
    }
    Ok(None)
}

fn root_geometry(
    anchor: Position,
    tip: Position,
    radius: f32,
    crop_base: bool,
) -> Result<Geometry, MorphError> {
    let end = vec(tip.relative(anchor)?);
    let start = if crop_base {
        end.normalize_or_zero() * 0.2f32.min(end.length() * 0.8)
    } else {
        Vector::ZERO
    };
    Ok(Geometry {
        shape: Arc::new(Capsule::new(start, end, radius)),
        radius: end.length() + radius,
        vertices: Arc::new(vec![start, end]),
        parts: vec![],
        outlets: vec![],
        writhe_slack: OnceLock::from(0.),
    })
}
/// Physical root growth happens after admitted body motion. The host-pipe base
/// is an authorised graft; the remaining stalk cannot grow through another solid.
pub(crate) fn admit_roots(
    before: &WorldSnapshot,
    next: &mut WorldSnapshot,
    cache: &mut CollisionCache,
) -> Result<(), MorphError> {
    #[cfg(feature = "allocation-attribution")]
    let _allocation_scope = crate::profiling::enter(9);

    if !next.habitat.as_ref().is_some_and(|h| h.rules.collisions) {
        return Ok(());
    }
    let roots = next.habitat.as_ref().unwrap().sacs.clone();
    let mut admitted: Vec<(u64, Position, Geometry)> = Vec::new();
    for (sac, attachment) in roots {
        let Some(anchor) = crate::habitat::anchor(next, &attachment) else {
            continue;
        };
        let tip = next.instances[&sac].position;
        let old_attachment = before.habitat.as_ref().and_then(|h| h.sacs.get(&sac));
        let old_anchor = old_attachment
            .and_then(|a| crate::habitat::anchor(before, a))
            .unwrap_or(anchor);
        let old_tip = before.instances.get(&sac).map_or(tip, |i| i.position);
        let sweep = vec(anchor.relative(old_anchor)?)
            .length()
            .max(vec(tip.relative(old_tip)?).length());
        let g = root_geometry(
            old_anchor,
            old_tip,
            crate::habitat::ROOT_RADIUS as f32 / SCALE as f32 + sweep,
            false,
        )?;
        let graft = root_geometry(
            old_anchor,
            old_tip,
            crate::habitat::ROOT_RADIUS as f32 / SCALE as f32 + sweep,
            true,
        )?;
        let pose = RigidPose {
            translation: old_anchor.absolute()?,
            ..RigidPose::identity()
        };
        let mut blocker = None;
        for id in next.instances.keys().copied().filter(|id| *id != sac) {
            let body = cache.body(next, id)?;
            if overlaps(&g, pose, &body, next.body_pose(id)?)? {
                blocker = Some(PhysicalTarget::Organism { id });
                break;
            }
        }
        if blocker.is_none() {
            for (id, origin, other) in &admitted {
                if overlaps(
                    &g,
                    pose,
                    other,
                    RigidPose {
                        translation: origin.absolute()?,
                        ..RigidPose::identity()
                    },
                )? {
                    blocker = Some(PhysicalTarget::Root { sac: *id });
                    break;
                }
            }
        }
        if blocker.is_none() {
            for solid in &next.solids {
                let min = solid.min.absolute()?;
                let max = solid.max.absolute()?;
                let half = vec(std::array::from_fn(|k| (max[k] - min[k]) / 2));
                let center = std::array::from_fn(|k| min[k] + (max[k] - min[k]) / 2);
                let other = Geometry {
                    shape: Arc::new(Cuboid::new(half)),
                    radius: half.length(),
                    vertices: Arc::new(vec![Vector::ZERO]),
                    parts: vec![],
                    outlets: vec![],
                    writhe_slack: OnceLock::from(0.),
                };
                if overlaps(
                    &g,
                    pose,
                    &other,
                    RigidPose {
                        translation: center,
                        ..RigidPose::identity()
                    },
                )? {
                    blocker = Some(PhysicalTarget::Solid { id: solid.id });
                    break;
                }
            }
        }
        if blocker.is_none()
            && let Some(assembly) = &next.assembly
        {
            for (filament_id, chunk) in
                cache.interacting_chunks(assembly, next.tick, anchor, g.radius)?
            {
                let f = &assembly.filaments[&filament_id];
                if pipe_separated(cache, f, chunk, next.tick, anchor, g.radius)? {
                    continue;
                }
                let (origin, other) = cache.pipe(next, f.id, chunk)?;
                if overlaps(
                    if f.id == attachment.filament {
                        &graft
                    } else {
                        &g
                    },
                    pose,
                    &other,
                    RigidPose {
                        translation: origin.absolute()?,
                        ..RigidPose::identity()
                    },
                )? {
                    blocker = Some(PhysicalTarget::Pipe {
                        filament: f.id,
                        chunk,
                    });
                    break;
                }
            }
        }

        if let Some(target) = blocker {
            next.habitat.as_mut().unwrap().sacs.remove(&sac);
            // Detachment removes the obstructed stalk, not the living food body.
            // No candidate solid link is admitted on this outcome.
            record(
                next,
                sac,
                target,
                (0., [0; 3], anchor),
                ContactResponse::AttachmentRejected,
            );
            let _ = before;
        } else {
            if old_attachment != Some(&attachment) {
                let normal = vec(tip.relative(anchor)?)
                    .normalize_or_zero()
                    .to_array()
                    .map(|x| (x * SCALE as f32).round() as i64);
                let observation = ContactObservation {
                    at: next.tick,
                    predicted: false,
                    organism: sac,
                    target: PhysicalTarget::Pipe {
                        filament: attachment.filament,
                        chunk: 0,
                    },
                    point: anchor,
                    normal,
                    response: ContactResponse::SourceAttachment,
                    toi_millionths: 0,
                };
                next.habitat
                    .as_mut()
                    .unwrap()
                    .contacts
                    .push(observation.clone());
                next.events.push(Event::Contact { observation });
            }
            admitted.push((sac, old_anchor, g));
        }
    }
    Ok(())
}

#[cfg(feature = "query-trace")]
fn owner_label(owner: &PhysicalTarget) -> String {
    match owner {
        PhysicalTarget::Pipe { filament, chunk } => format!("pipe:{filament}:{chunk}"),
        PhysicalTarget::Organism { id } => format!("body:{id}"),
        PhysicalTarget::Solid { id } => format!("solid:{id}"),
        PhysicalTarget::Root { sac } => format!("root:{sac}"),
        PhysicalTarget::Parcel { organism, parcel } => format!("parcel:{organism}:{parcel}"),
    }
}

#[cfg(feature = "query-trace")]
fn trace_static_pair(
    tick: u64,
    body: u64,
    owner: &PhysicalTarget,
    assembly: Option<&crate::filament::Assembly>,
) {
    match owner {
        PhysicalTarget::Pipe { filament, chunk } => {
            let revision = assembly
                .and_then(|a| a.filaments.get(filament))
                .and_then(|f| pipe_key(*filament, f, *chunk, tick))
                .map(|key| u64::from_le_bytes(key.2[..8].try_into().unwrap()))
                .unwrap_or(0);
            query_trace::set_pair(
                &format!("t{tick}:body:{body}:{}", owner_label(owner)),
                revision,
            );
        }
        _ => query_trace::set_pair(&format!("t{tick}:body:{body}:{}", owner_label(owner)), 0),
    }
}

#[cfg(feature = "query-trace")]
mod query_trace {
    use serde::Serialize;
    use std::cell::RefCell;
    use std::collections::{BTreeMap, BTreeSet};

    thread_local! {
        static ACTIVE: RefCell<Active> = RefCell::new(Active::default());
        static PAIRS: RefCell<BTreeMap<String, PairStat>> = const { RefCell::new(BTreeMap::new()) };
    }

    #[derive(Default)]
    struct Active {
        label: String,
        revision: u64,
        pose: u64,
    }

    #[derive(Clone, Debug, Default, Serialize)]
    pub struct PairStat {
        pub casts: u64,
        pub cast_ns: u128,
        pub linear_ns: u128,
        pub initial_contact_ns: u128,
        pub terminal_contact_ns: u128,
        pub distance_ns: u128,
        pub cache_hits: u64,
        pub distance_queries: u64,
        pub within: u64,
        pub clear: u64,
        pub triangles: u64,
        pub max_triangles: u64,
        pub bounded: u64,
        pub exact: u64,
        pub revisions: BTreeSet<u64>,
        pub limits: BTreeSet<u32>,
        pub sample_keys: BTreeSet<u64>,
    }

    pub fn pose_fingerprint(poses: &[i64; 24]) -> u64 {
        let mut hash = 0x9e3779b97f4a7c15u64;
        for value in poses {
            hash = hash
                .wrapping_mul(0x517cc1b727220a95)
                .wrapping_add(*value as u64);
        }
        hash
    }

    pub fn set_pair(label: &str, revision: u64) {
        ACTIVE.with(|active| {
            let mut active = active.borrow_mut();
            active.label = label.to_string();
            active.revision = revision;
        });
    }

    pub fn clear_label() {
        ACTIVE.with(|active| *active.borrow_mut() = Active::default());
    }

    fn with_pair(update: impl FnOnce(&mut PairStat, &Active)) {
        ACTIVE.with(|active| {
            let active = active.borrow();
            let label = if active.label.is_empty() {
                "unlabeled".to_string()
            } else {
                active.label.clone()
            };
            PAIRS.with(|pairs| {
                let mut pairs = pairs.borrow_mut();
                let stat = pairs.entry(label).or_default();
                update(stat, &active);
            });
        });
    }

    pub fn note_cache_hit() {
        with_pair(|stat, active| {
            stat.cache_hits += 1;
            stat.revisions.insert(active.revision);
        });
    }

    pub fn note_cast(pose: u64) {
        ACTIVE.with(|active| active.borrow_mut().pose = pose);
        with_pair(|stat, active| {
            stat.casts += 1;
            stat.revisions.insert(active.revision);
        });
    }

    pub fn note_phase(phase: &str, ns: u128) {
        with_pair(|stat, _| match phase {
            "cast" => stat.cast_ns += ns,
            "linear" => stat.linear_ns += ns,
            "distance" => stat.distance_ns += ns,
            "initial_contact" => stat.initial_contact_ns += ns,
            "terminal_contact" => stat.terminal_contact_ns += ns,
            _ => unreachable!("observer phase"),
        });
    }

    pub fn note_distance(
        limit: u32,
        mid: u32,
        within: bool,
        triangles: u64,
        bounded: u64,
        exact: u64,
    ) {
        with_pair(|stat, active| {
            stat.distance_queries += 1;
            if within {
                stat.within += 1;
            } else {
                stat.clear += 1;
            }
            stat.triangles += triangles;
            stat.max_triangles = stat.max_triangles.max(triangles);
            stat.bounded += bounded;
            stat.exact += exact;
            stat.limits.insert(limit);
            stat.revisions.insert(active.revision);
            let sample = active.pose.wrapping_mul(0x9e3779b97f4a7c15)
                ^ (u64::from(limit) << 32 | u64::from(mid));
            stat.sample_keys.insert(sample);
        });
    }

    pub fn take() -> BTreeMap<String, PairStat> {
        PAIRS.with(|pairs| std::mem::take(&mut *pairs.borrow_mut()))
    }
}

#[cfg(feature = "query-trace")]
pub fn take_query_trace() -> BTreeMap<String, query_trace::PairStat> {
    query_trace::take()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filament::{Assembly, Filament, Node, PipeSection};

    #[test]
    fn filtered_linear_casts_preserve_reference_witnesses_and_primitive_dispatch() {
        let world = ShipRuntime::habitat_simulation(AnatomySpec::bare(77), 77, 8)
            .unwrap()
            .snapshot();
        let mut cache = CollisionCache::default();
        let body = cache.body(&world, 1).unwrap();
        let mut assembly = Assembly::default();
        assembly.filaments.insert(1, straight(7, 60000));
        let (_, pipe) = cache.pipe_in(&assembly, 40, 1, 0).unwrap();
        let shapes: Vec<Arc<dyn Shape>> = vec![
            body.shape.clone(),
            pipe.shape,
            Arc::new(Ball::new(0.025)),
            Arc::new(Capsule::new_y(0.3, 0.025)),
            Arc::new(Cuboid::new(Vector::new(0.015, 0.4, 0.015))),
        ];
        let mut supports = SupportCache::default();
        for shape in &shapes {
            supports.register(shape);
        }
        assert!(!supports.indices.is_empty());
        let dispatcher = FilteredLinearDispatcher::new(&supports);
        let bits = |hit: Option<query::ShapeCastHit>| {
            hit.map(|hit| {
                let mut values = vec![hit.time_of_impact.to_bits()];
                for v in [hit.normal1, hit.normal2, hit.witness1, hit.witness2] {
                    values.extend(v.to_array().map(f32::to_bits));
                }
                (values, format!("{:?}", hit.status))
            })
        };
        for &(a, b) in &[
            (0, 1),
            (1, 0),
            (0, 2),
            (2, 0),
            (0, 3),
            (3, 0),
            (0, 4),
            (0, 0),
        ] {
            for step in -7..=7 {
                let pose = Iso::from_parts(
                    Vector::new(step as f32 * 0.13, 0.03, 0.02),
                    Rot3::from_rotation_y(step as f32 * 0.17),
                );
                for velocity in [Vector::new(0.3, 0.01, -0.1), Vector::new(-3., 1., 2.)] {
                    let options = ShapeCastOptions {
                        max_time_of_impact: 1.,
                        target_distance: 0.0001,
                        stop_at_penetration: true,
                        compute_impact_geometry_on_penetration: true,
                    };
                    let reference = DefaultQueryDispatcher
                        .cast_shapes(
                            &pose,
                            velocity,
                            shapes[a].as_ref(),
                            shapes[b].as_ref(),
                            options,
                        )
                        .unwrap();
                    let indexed = dispatcher
                        .cast_shapes(
                            &pose,
                            velocity,
                            shapes[a].as_ref(),
                            shapes[b].as_ref(),
                            options,
                        )
                        .unwrap();
                    assert_eq!(bits(indexed), bits(reference), "pair {a}/{b}, step {step}");
                }
            }
        }
    }

    #[test]
    fn reused_contact_workspace_matches_fresh_dispatch_bit_for_bit() {
        let world = ShipRuntime::habitat_simulation(AnatomySpec::bare(77), 77, 8)
            .unwrap()
            .snapshot();
        let mut cache = CollisionCache::default();
        let body = cache.body(&world, 1).unwrap();
        let mut assembly = Assembly::default();
        assembly.filaments.insert(1, straight(7, 60_000));
        let (_, pipe) = cache.pipe_in(&assembly, 40, 1, 0).unwrap();
        let shapes: Vec<Arc<dyn Shape>> = vec![
            body.shape.clone(),
            pipe.shape,
            Arc::new(Cuboid::new(Vector::new(0.3, 0.2, 0.4))),
            Arc::new(Capsule::new_y(0.3, 0.1)),
            Arc::new(Ball::new(0.2)),
        ];
        let bits = |c: Option<query::Contact>| {
            c.map(|c| {
                let mut result = Vec::new();
                for v in [c.point1, c.point2, c.normal1, c.normal2] {
                    result.extend(v.to_array().map(f32::to_bits));
                }
                result.push(c.dist.to_bits());
                result
            })
        };
        let mut workspace = ContactWorkspace::default();
        let geometry = shapes
            .iter()
            .map(|shape| Geometry {
                shape: shape.clone(),
                radius: 10.,
                vertices: Arc::new(vec![]),
                parts: vec![],
                outlets: vec![],
                writhe_slack: OnceLock::from(0.),
            })
            .collect::<Vec<_>>();
        for reverse in [false, true] {
            for i in 0..shapes.len() {
                for j in 0..shapes.len() {
                    for step in -6..=6 {
                        let k = if reverse { -step } else { step };
                        let pose = Iso::from_parts(
                            Vector::new(k as f32 * 0.13, (k % 3) as f32 * 0.07, 0.03),
                            Rot3::from_rotation_y(k as f32 * 0.17),
                        );
                        for prediction in [0., 0.002, 0.1] {
                            let expected = query::contact(
                                &Iso::IDENTITY,
                                shapes[i].as_ref(),
                                &pose,
                                shapes[j].as_ref(),
                                prediction,
                            )
                            .unwrap();
                            let actual = cached_world_contact(
                                &mut workspace,
                                &Iso::IDENTITY,
                                &geometry[i],
                                &pose,
                                &geometry[j],
                                prediction,
                            )
                            .unwrap();
                            assert_eq!(
                                bits(actual),
                                bits(expected),
                                "pair {i}/{j}, step {k}, prediction {prediction}"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn support_index_does_not_retain_changing_meshes_or_capsules() {
        let mut cache = CollisionCache::default();
        let mut assembly = Assembly::default();
        assembly.filaments.insert(1, straight(7, 60_000));
        let (_, geometry) = cache.pipe_in(&assembly, 40, 1, 0).unwrap();
        let weak = Arc::downgrade(&geometry.shape);
        let mut supports = SupportCache::default();
        supports.register(&geometry.shape);
        let capsule: Arc<dyn Shape> = Arc::new(Capsule::new_y(0.3, 0.1));
        supports.register(&capsule);
        assert!(supports.shapes.is_empty());
        assert!(supports.indices.is_empty());
        drop(geometry);
        drop(cache);
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn dense_eight_founder_proximity_handles_tetrahedron_simplexes() {
        let mut world =
            ShipRuntime::habitat_colony(AnatomySpec::showcase(11), 11, 120, 8, Some(4000)).unwrap();
        world.advance(20).unwrap();
        assert_eq!(world.state().condensate_gap(), 0);
        world.validate().unwrap();
    }

    #[test]
    fn bounded_predicate_matches_original_distance_for_body_and_hollow_mesh() {
        let world = ShipRuntime::habitat_simulation(AnatomySpec::bare(77), 77, 8)
            .unwrap()
            .snapshot();
        let mut cache = CollisionCache::default();
        let body = cache.body(&world, 1).unwrap();
        let food_id = world
            .instances
            .values()
            .find(|i| i.definition == BLOOD_SAC)
            .unwrap()
            .id;
        let food = cache.body(&world, food_id).unwrap();
        let mut assembly = Assembly::default();
        assembly.filaments.insert(1, straight(7, 60_000));
        let (_, pipe) = cache.pipe_in(&assembly, 40, 1, 0).unwrap();
        let mut near_cases = 0;
        let mut far_cases = 0;
        for k in -8i64..=8 {
            let angle = k as f64 * 0.17;
            let pose = RigidPose {
                translation: [k * 250_000, (k % 3) * 120_000, k * 50_000],
                yaw_sin: (angle.sin() * SCALE as f64).round() as i64,
                yaw_cos: (angle.cos() * SCALE as f64).round() as i64,
                axial_roll: k * 15_000,
            };
            let relative = motion(pose, pose, Position::from_absolute([0; 3]).unwrap())
                .unwrap()
                .start;
            for other in [&food.shape, &pipe.shape] {
                let exact = DefaultQueryDispatcher
                    .distance(&relative, body.shape.as_ref(), other.as_ref())
                    .unwrap();
                for ceiling in [0.0001, 0.01, 0.1, 0.5, 2., 10.] {
                    let within = within_distance(
                        &SupportCache::default(),
                        &relative,
                        body.shape.as_ref(),
                        other.as_ref(),
                        ceiling,
                        0.001,
                        &DistanceCounts::new(),
                    )
                    .unwrap();
                    assert_eq!(
                        within,
                        exact <= ceiling,
                        "distance predicate changed: k={k}, limit={ceiling}, exact={exact}"
                    );
                    if within {
                        near_cases += 1;
                    } else {
                        far_cases += 1;
                    }
                }
            }
        }
        assert!(near_cases > 0 && far_cases > 0);
    }

    #[test]
    fn repeated_spawn_search_reuses_shapes_and_matches_a_cold_cache() {
        let world = ShipRuntime::habitat_simulation(AnatomySpec::bare(77), 77, 8)
            .unwrap()
            .snapshot();
        let near = world.body_pose(1).unwrap();
        let mut warm = CollisionCache::default();
        let first = spawn_pose(&world, 1, near, None, &mut warm).unwrap();
        assert!(first.is_some(), "fixture must have a legal placement");
        let builds = warm.profile().body_builds;
        assert!(builds > 0);
        let again = spawn_pose(&world, 1, near, None, &mut warm).unwrap();
        assert_eq!(first, again);
        assert_eq!(warm.profile().body_builds, builds);
        let cold = spawn_pose(&world, 1, near, None, &mut CollisionCache::default()).unwrap();
        assert_eq!(again, cold);
    }

    #[test]
    fn wet_pipe_cache_matches_cold_geometry_across_a_future_node_tick() {
        let mut filament = straight(7, 60_000);
        for (k, node) in filament.nodes.iter_mut().enumerate() {
            node.deposited_tick = 35 + k as u64;
        }
        let mut assembly = Assembly::default();
        assembly.filaments.insert(1, filament);
        let mut warm = CollisionCache::default();
        let (_, previous) = warm.pipe_in(&assembly, 40, 1, 0).unwrap();
        let (_, cached) = warm.pipe_in(&assembly, 41, 1, 0).unwrap();
        let (_, cold) = CollisionCache::default()
            .pipe_in(&assembly, 41, 1, 0)
            .unwrap();
        assert_ne!(
            previous.vertices, cold.vertices,
            "fixture must actually age some geometry"
        );
        assert!(
            cached.vertices == cold.vertices,
            "cache reused last tick's radii across youngest-node age zero"
        );
    }
    #[test]
    fn paid_geometry_handoff_matches_cold_mesh_and_refuses_mutation_or_unpaid_input() {
        use crate::emission::{EmissionStatus, EmitRequest, SECRETION_CHANNEL};
        let world = ShipRuntime::habitat_simulation(AnatomySpec::bare(77), 77, 8)
            .unwrap()
            .snapshot();
        let mut book = world.instances[&1].book.clone();
        let rules = world.config.construction.as_ref().unwrap();
        let nozzle = *book.registered.first().unwrap();
        let mut assembly = crate::filament::Assembly::default();
        for z in [0, 100_000] {
            let position = [10 * SCALE, 0, z];
            let EmissionStatus::Admitted { parcel_id, .. } = book.try_emit(EmitRequest {
                nozzle_id: nozzle,
                origin: position,
                direction: [0, 0, SCALE],
                quantity: 1,
                launch_speed: 0,
                channel: SECRETION_CHANNEL,
            }) else {
                panic!("paid fixture")
            };
            assembly
                .deposit(
                    &mut book,
                    rules,
                    parcel_id,
                    Position::from_absolute(position).unwrap(),
                )
                .unwrap();
            book.step().unwrap();
        }
        assembly.validate(&book, rules).unwrap();
        let binding = *book.bound.last().unwrap();
        let id = binding.filament;
        let tick = book.tick;
        let (origin, surface) = filament_chunk_at(&assembly.filaments[&id], 0, tick).unwrap();
        let g = geometry(&[surface], 1.).unwrap();
        let key = pipe_key(id, &assembly.filaments[&id], 0, tick).unwrap();
        for mutation in 0..10 {
            let mut actual = assembly.clone();
            let mut paid = book.clone();
            let mut time = tick;
            let mut cache = CollisionCache::default();
            cache.configure_geometry_reuse(true);
            cache.pending_geometry = Some(DepositGeometry {
                parcel: binding.parcel.id,
                organism: book.organism_id,
                chunks: vec![(key, origin, g.clone())],
            });
            match mutation {
                0 => {}
                1 => {
                    actual.filaments.get_mut(&id).unwrap().nodes[0].position =
                        origin.offset([1, 0, 0]).unwrap()
                }
                2 => {
                    actual
                        .filaments
                        .get_mut(&id)
                        .unwrap()
                        .section
                        .as_mut()
                        .unwrap()
                        .inner_radius += 1
                }
                3 => {
                    actual
                        .filaments
                        .get_mut(&id)
                        .unwrap()
                        .section
                        .as_mut()
                        .unwrap()
                        .end_sealed = true
                }
                4 => actual.filaments.get_mut(&id).unwrap().nozzle += 1,
                5 => time += 1,
                6 => actual.filaments.get_mut(&id).unwrap().nodes[0].deposited_tick += 1,
                7 => {
                    paid.bound.clear();
                }
                8 => cache.discard_deposit_geometry(),
                9 => actual
                    .filaments
                    .get_mut(&id)
                    .unwrap()
                    .nodes
                    .pop()
                    .map(|_| ())
                    .unwrap(),
                _ => unreachable!(),
            }
            cache.admit_deposit_geometry(
                &actual,
                &paid,
                time,
                (id, binding.node, binding.parcel.id),
            );
            if mutation == 0 {
                assert_eq!(cache.geometry_reuse_profile.promoted, 1);
                let (_, warm) = cache.pipe_in(&actual, time, id, 0).unwrap();
                let (_, cold) = CollisionCache::default()
                    .pipe_in(&actual, time, id, 0)
                    .unwrap();
                let bits = |g: &Geometry| {
                    g.shape
                        .as_trimesh()
                        .unwrap()
                        .vertices()
                        .iter()
                        .map(|p| p.to_array().map(f32::to_bits))
                        .collect::<Vec<_>>()
                };
                assert_eq!(bits(&warm), bits(&cold));
                assert_eq!(
                    warm.shape.as_trimesh().unwrap().indices(),
                    cold.shape.as_trimesh().unwrap().indices()
                );
                assert_eq!(cache.geometry_reuse_profile.consumed, 1);
            } else {
                assert!(cache.pipes.is_empty(), "mutation {mutation} was promoted");
            }
        }
    }
    #[test]
    fn paid_self_deposit_cannot_cross_the_trunk() {
        use crate::emission::{EmissionStatus, EmitRequest, SECRETION_CHANNEL};
        let world = ShipRuntime::habitat_simulation(AnatomySpec::bare(77), 77, 8)
            .unwrap()
            .snapshot();
        let mut book = world.instances[&1].book.clone();
        let nozzle = *book.registered.first().unwrap();
        let status = book.try_emit(EmitRequest {
            nozzle_id: nozzle,
            origin: world.instances[&1].position.absolute().unwrap(),
            direction: [0, 0, SCALE],
            quantity: 1,
            launch_speed: 0,
            channel: SECRETION_CHANNEL,
        });
        let EmissionStatus::Admitted { parcel_id, .. } = status else {
            panic!("fixture must pay for its parcel")
        };
        let saved = book.clone();
        assert!(
            !deposit_allowed(
                &world,
                world.assembly.as_ref().unwrap(),
                &book,
                world.config.construction.as_ref().unwrap(),
                parcel_id,
                &mut CollisionCache::default()
            )
            .unwrap(),
            "own source exemption admitted a pipe through the trunk"
        );
        assert_eq!(book, saved, "rejection must preserve paid material");
    }
    #[test]
    fn rotated_linear_sweeps_keep_world_velocity_and_relative_motion() {
        let a = Geometry {
            shape: Arc::new(Ball::new(0.01)),
            radius: 0.01,
            vertices: Arc::new(vec![Vector::ZERO]),
            parts: vec![],
            outlets: vec![],
            writhe_slack: OnceLock::from(0.),
        };
        let b = Geometry {
            shape: Arc::new(Cuboid::new(Vector::new(0.001, 1., 1.))),
            radius: 2.,
            vertices: Arc::new(vec![Vector::ZERO]),
            parts: vec![],
            outlets: vec![],
            writhe_slack: OnceLock::from(0.),
        };
        for (sin, cos) in [(0, SCALE), (SCALE, 0), (0, -SCALE)] {
            let from = RigidPose {
                translation: [-SCALE, 0, 0],
                yaw_sin: sin,
                yaw_cos: cos,
                axial_roll: 0,
            };
            let to = RigidPose {
                translation: [SCALE, 0, 0],
                ..from
            };
            let p = RigidPose::identity();
            let mut profile = CollisionProfile::default();
            assert!(
                cast(
                    &a,
                    from,
                    to,
                    &b,
                    p,
                    p,
                    (&mut profile, &mut ContactWorkspace::default())
                )
                .unwrap()
                .is_some(),
                "rotation changed world-space velocity"
            );
            let moving = RigidPose {
                translation: [2 * SCALE, 0, 0],
                ..p
            };
            assert!(
                cast(
                    &a,
                    from,
                    to,
                    &b,
                    p,
                    moving,
                    (&mut profile, &mut ContactWorkspace::default())
                )
                .unwrap()
                .is_none(),
                "equal velocities invented a collision"
            );
        }
    }
    #[test]
    fn overlap_recovery_rejects_inward_motion_and_allows_clearance() {
        let ball = |radius| Geometry {
            shape: Arc::new(Ball::new(radius)),
            radius,
            vertices: Arc::new(vec![Vector::ZERO]),
            parts: vec![],
            outlets: vec![],
            writhe_slack: OnceLock::from(0.),
        };
        let a = ball(0.1);
        let b = ball(0.1);
        let from = RigidPose::identity();
        let obstacle = RigidPose {
            translation: [150_000, 0, 0],
            ..from
        };
        let away = RigidPose {
            translation: [-40_000, 0, 0],
            ..from
        };
        let into = RigidPose {
            translation: [40_000, 0, 0],
            ..from
        };
        assert!(
            nondeepening_recovery(
                &a,
                from,
                away,
                &b,
                obstacle,
                obstacle,
                &mut ContactWorkspace::default()
            )
            .unwrap()
        );
        assert!(
            !nondeepening_recovery(
                &a,
                from,
                into,
                &b,
                obstacle,
                obstacle,
                &mut ContactWorkspace::default()
            )
            .unwrap()
        );
        let far = RigidPose {
            translation: [500_000, 0, 0],
            ..from
        };
        assert!(
            !nondeepening_recovery(
                &a,
                from,
                into,
                &b,
                far,
                far,
                &mut ContactWorkspace::default()
            )
            .unwrap(),
            "separated start must use the continuous sweep"
        );
    }
    #[test]
    fn body_cache_shares_geometry_and_invalidates_changed_definition() {
        let mut world = ShipRuntime::habitat_simulation(AnatomySpec::bare(77), 77, 8)
            .unwrap()
            .snapshot();
        let mut cache = CollisionCache::default();
        let a = cache.body(&world, 1).unwrap();
        let b = cache.body(&world, 1).unwrap();
        assert!(Arc::ptr_eq(&a, &b));
        world.config.definitions.get_mut(&1).unwrap().body.length *= 1.01;
        let changed = cache.body(&world, 1).unwrap();
        assert!(!Arc::ptr_eq(&a, &changed));
        assert_eq!(cache.profile.body_builds, 2);
    }
    #[test]
    fn blocked_shallow_escape_turns_further_toward_clearance() {
        let pose = RigidPose::identity();
        let mut a = Agent {
            travel: None,
            refilling: false,
            rigid: pose,
            heading: [-10_000, 999_950, 0],
            eating: false,
            born: 0,
            last_birth: 0,
            parent: None,
        };
        let before = a.heading[0];
        retreat(&mut a, [SCALE, 0, 0]);
        assert!(
            a.heading[0] < before - 10_000,
            "a refused shallow escape must not remain stuck on the same heading"
        );
    }
    #[test]
    fn slide_removes_inward_motion_and_preserves_escape() {
        let a = Ball::new(0.1);
        let b = Ball::new(0.1);
        let contact = query::contact(
            &Iso::IDENTITY,
            &a,
            &Iso::from_translation(Vector::X * 0.15),
            &b,
            0.1,
        )
        .unwrap()
        .unwrap();
        let normal = contact
            .normal1
            .to_array()
            .map(|v| (v * SCALE as f32).round() as i64);
        let start = RigidPose::identity();
        let inward = RigidPose {
            translation: [60_000, 10_000, 0],
            ..start
        };
        assert!(
            along_surface(start, inward, normal).translation[0] <= 0,
            "slide preserved inward penetration"
        );
        let away = RigidPose {
            translation: [-60_000, 10_000, 0],
            ..start
        };
        assert_eq!(
            along_surface(start, away, normal).translation,
            away.translation,
            "slide erased escape motion"
        );
    }
    #[test]
    fn cast_memo_reuses_only_identical_geometry_bounds_and_motion() {
        let make = |radius: f32| Geometry {
            shape: Arc::new(Ball::new(radius)),
            radius,
            vertices: Arc::new(vec![Vector::ZERO]),
            parts: vec![],
            outlets: vec![],
            writhe_slack: OnceLock::from(0.),
        };
        let a = make(0.1);
        let b = make(0.3);
        let mut memo = CollisionCache::default();
        for x in [-2 * SCALE, 0, 2 * SCALE] {
            for roll in [0, 100_000, 900_000] {
                let from = RigidPose {
                    translation: [x, 0, 0],
                    axial_roll: roll,
                    ..RigidPose::identity()
                };
                let to = RigidPose {
                    translation: [x + SCALE, 0, 0],
                    ..from
                };
                let p = RigidPose::identity();
                let expected = cast(
                    &a,
                    from,
                    to,
                    &b,
                    p,
                    p,
                    (
                        &mut CollisionProfile::default(),
                        &mut ContactWorkspace::default(),
                    ),
                )
                .unwrap();
                assert_eq!(memo.cast(&a, from, to, &b, p, p).unwrap(), expected);
                let visits = memo.profile.queries;
                assert_eq!(memo.cast(&a, from, to, &b, p, p).unwrap(), expected);
                assert_eq!(memo.profile.queries, visits);
                // Same shape, changed angular speed envelope must not reuse.
                let enlarged = Geometry {
                    radius: a.radius + 1.,
                    ..a.clone()
                };
                memo.cast(&enlarged, from, to, &b, p, p).unwrap();
                assert!(memo.profile.queries > visits);
            }
        }
        assert_eq!(memo.profile.cast_cache_hits, 9);
        let p = RigidPose::identity();
        let new_shape = make(0.6);
        memo.cast(&a, p, p, &new_shape, p, p).unwrap();
        assert_eq!(memo.profile.cast_cache_hits, 9);
        memo.casts.clear();
        assert!(memo.casts.is_empty());
    }
    fn straight(count: usize, spacing: i64) -> Filament {
        Filament {
            id: 1,
            section: Some(PipeSection {
                outer_radius: 50_000,
                inner_radius: 31_000,
                start_sealed: true,
                end_sealed: false,
            }),
            organism: 1,
            nozzle: 1,
            created_tick: 0,
            experiment: "bound".into(),
            material: crate::emission::MaterialKind::Secretion,
            quantity: count as u64,
            nodes: (0..count)
                .map(|k| Node {
                    deposited_tick: 0,
                    id: k as u64 + 1,
                    parcel: k as u64 + 1,
                    position: Position::from_absolute([0, 0, k as i64 * spacing]).unwrap(),
                })
                .collect(),
        }
    }

    #[test]
    fn sphere_gate_covers_the_built_mesh_and_keeps_a_wall_vertex() {
        let mut filament = straight(20, 500_000);
        filament.nozzle = 7;
        filament.nodes[2].position =
            Position::from_absolute([2_000_000, -800_000, 400_000]).unwrap();
        filament.nodes[5].position =
            Position::from_absolute([-1_500_000, 1_200_000, 900_000]).unwrap();
        filament.nodes[8].position = Position::from_absolute([200_000, 200_000, 200_000]).unwrap();
        filament.nodes[16].position = Position::from_absolute([0, 0, 8_000_000]).unwrap();
        filament.nodes[17].position = Position::from_absolute([0, 25_000_000, 8_000_000]).unwrap();
        filament.nodes[19].position = Position::from_absolute([30_000_000, 0, 9_000_000]).unwrap();
        for (k, node) in filament.nodes.iter_mut().enumerate() {
            node.deposited_tick = k as u64 * 3;
        }
        filament.section.as_mut().unwrap().end_sealed = true;
        let ticks = [0u64, 1, 50, 96, 192, 400];
        for tick in ticks {
            for chunk in 0..2 {
                let (origin, surface) = filament_chunk_at(&filament, chunk, tick).unwrap();
                let g = geometry(std::slice::from_ref(&surface), 1.).unwrap();
                let bound = chunk_mesh_radius_bound(&filament, chunk, tick).unwrap();
                assert!(
                    g.radius <= bound,
                    "tick {tick} chunk {chunk} mesh radius {} exceeded bound {bound}",
                    g.radius
                );
                for p in &surface.positions {
                    let at = origin
                        .offset(p.map(|v| (v * SCALE as f32).round() as i64))
                        .unwrap();
                    let mut cache = CollisionCache::default();
                    assert!(
                        !pipe_separated(&mut cache, &filament, chunk, tick, at, 0.).unwrap(),
                        "tick {tick} chunk {chunk} wall vertex rejected by the sphere gate"
                    );
                }
            }
        }
        let far = Position::from_absolute([0, -80 * SCALE, 0]).unwrap();
        let mut cache = CollisionCache::default();
        assert!(pipe_separated(&mut cache, &filament, 0, 0, far, 1.).unwrap());
        assert_eq!(cache.profile.pipe_rejects, 1);
        let mut bare = filament.clone();
        bare.section = None;
        let (_, surface) = filament_chunk_at(&bare, 0, 0).unwrap();
        let g = geometry(&[surface], 1.).unwrap();
        let bound = chunk_mesh_radius_bound(&bare, 0, 0).unwrap();
        assert!(
            g.radius <= bound,
            "sectionless mesh radius {} exceeded bound {bound}",
            g.radius
        );
    }

    #[test]
    fn finished_chunk_keeps_its_mesh_when_later_nodes_are_added_and_age_still_changes_it() {
        let mut filament = straight(18, 200_000);
        let mut assembly = Assembly::default();
        assembly.filaments.insert(1, filament.clone());
        let mut cache = CollisionCache::default();
        cache.pipe_in(&assembly, 0, 1, 0).unwrap();
        filament.nodes.push(Node {
            deposited_tick: 0,
            id: 19,
            parcel: 19,
            position: Position::from_absolute([0, 0, 18 * 200_000]).unwrap(),
        });
        assembly.filaments.insert(1, filament.clone());
        cache.pipe_in(&assembly, 0, 1, 0).unwrap();
        assert_eq!(
            cache.profile.pipe_builds, 1,
            "lookahead-stable chunk rebuilt"
        );
        assert_eq!(cache.profile.pipe_hits, 1);
        cache.pipe_in(&assembly, 1, 1, 0).unwrap();
        assert_eq!(
            cache.profile.pipe_builds, 2,
            "a wetter tick must change the wall"
        );
        cache.pipe_in(&assembly, 192, 1, 0).unwrap();
        cache.pipe_in(&assembly, 193, 1, 0).unwrap();
        assert_eq!(
            cache.profile.pipe_builds, 3,
            "dry geometry rebuilt after the radius had stopped"
        );
        assert_eq!(cache.profile.pipe_hits, 2);
    }

    #[test]
    fn drying_does_not_increase_the_chunk_radius_bound() {
        let filament = straight(20, 400_000);
        let mut previous = f32::MAX;
        for tick in [0u64, 1, 17, 96, 192, 400, 10_000] {
            let bound = chunk_mesh_radius_bound(&filament, 0, tick).unwrap();
            assert!(
                bound <= previous + 1e-4,
                "bound grew from {previous} to {bound} at tick {tick}"
            );
            previous = bound;
        }
    }

    #[test]
    fn pipe_horizon_keeps_every_sample_sphere_candidate() {
        let mut filament = straight(24, 500_000);
        filament.nodes[3].position =
            Position::from_absolute([1_500_000, -400_000, 900_000]).unwrap();
        filament.nodes[8].position =
            Position::from_absolute([-800_000, 1_200_000, 3_000_000]).unwrap();
        filament.section.as_mut().unwrap().end_sealed = true;
        let mut assembly = Assembly::default();
        assembly.filaments.insert(filament.id, filament.clone());
        let mut cache = CollisionCache::default();
        for tick in [0u64, 40, 192] {
            cache.sync_pipe_horizon(&assembly, tick).unwrap();
            for reach in [0.0, 0.25, 1.5, 6.0] {
                for step in 0..12 {
                    let at = Position::from_absolute([
                        (step as i64 - 4) * 2_000_000,
                        (step as i64 % 5 - 2) * 1_500_000,
                        step as i64 * 1_000_000,
                    ])
                    .unwrap();
                    for chunk in 0..2 {
                        let separated = pipe_separated(
                            &mut CollisionCache::default(),
                            &filament,
                            chunk,
                            tick,
                            at,
                            reach,
                        )
                        .unwrap();
                        let beyond = cache
                            .pipe_beyond_horizon(filament.id, chunk, at, reach)
                            .unwrap();
                        assert!(
                            !beyond || separated,
                            "horizon dropped a sample-sphere candidate at tick {tick} chunk {chunk} reach {reach}"
                        );
                    }
                }
            }
        }
        let far = Position::from_absolute([0, -400 * SCALE, 0]).unwrap();
        assert!(pipe_separated(&mut CollisionCache::default(), &filament, 0, 0, far, 1.).unwrap());
        assert!(cache.pipe_beyond_horizon(filament.id, 0, far, 1.).unwrap());
        assert!(cache.profile.pipe_horizon_rejects >= 1);
    }

    #[test]
    fn collapsed_horizon_radius_misses_a_wall_vertex() {
        let filament = straight(8, 300_000);
        let mut assembly = Assembly::default();
        assembly.filaments.insert(filament.id, filament.clone());
        let mut cache = CollisionCache::default();
        cache.sync_pipe_horizon(&assembly, 0).unwrap();
        let (origin, surface) = filament_chunk_at(&filament, 0, 0).unwrap();
        let vertex = surface.positions[surface.positions.len() / 2];
        let at = origin
            .offset(vertex.map(|v| (v * SCALE as f32).round() as i64))
            .unwrap();
        assert!(!pipe_separated(&mut CollisionCache::default(), &filament, 0, 0, at, 0.).unwrap());
        assert!(!cache.pipe_beyond_horizon(filament.id, 0, at, 0.).unwrap());
        let sphere = cache.horizon.spheres.get_mut(&(filament.id, 0)).unwrap();
        sphere.radius = 0;
        sphere.always = false;
        cache.horizon.cells.clear();
        assert!(
            cache.pipe_beyond_horizon(filament.id, 0, at, 0.).unwrap(),
            "a zeroed horizon radius must be detectable as a missed wall"
        );
        assert!(!pipe_separated(&mut CollisionCache::default(), &filament, 0, 0, at, 0.).unwrap());
    }

    #[test]
    fn appended_and_removed_pipes_refresh_the_horizon_without_retaining_stale_geometry() {
        let mut filament = straight(18, 200_000);
        let mut assembly = Assembly::default();
        assembly.filaments.insert(filament.id, filament.clone());
        let mut cache = CollisionCache::default();
        cache.sync_pipe_horizon(&assembly, 0).unwrap();
        cache.pipe_in(&assembly, 0, filament.id, 0).unwrap();
        assert_eq!(cache.profile.pipe_horizon_rebuilds, 1);
        assert_eq!(cache.pipes.len(), 1);
        filament.nodes.push(Node {
            deposited_tick: 0,
            id: 19,
            parcel: 19,
            position: Position::from_absolute([0, 0, 40 * SCALE]).unwrap(),
        });
        assembly.filaments.insert(filament.id, filament.clone());
        cache.sync_pipe_horizon(&assembly, 0).unwrap();
        assert_eq!(cache.profile.pipe_horizon_rebuilds, 2);
        let far_chunk = filament.nodes.len().saturating_sub(1).div_ceil(CHUNK_EDGES) - 1;
        let at = filament.nodes.last().unwrap().position;
        assert!(
            !cache
                .pipe_beyond_horizon(filament.id, far_chunk, at, 0.2)
                .unwrap()
        );
        assembly.filaments.clear();
        cache.sync_pipe_horizon(&assembly, 1).unwrap();
        assert!(
            cache.pipes.is_empty(),
            "removed filament geometry remained cached"
        );
        assert_eq!(cache.profile.pipe_horizon_indexed, 0);
        let before = cache.profile.pipe_bounds_computed;
        cache.sync_pipe_horizon(&assembly, 2).unwrap();
        assert_eq!(
            cache.profile.pipe_bounds_computed, before,
            "an unchanged empty assembly recomputed bounds"
        );
    }

    #[test]
    fn distant_chunks_do_not_allocate_cells_across_the_empty_span() {
        let mut assembly = Assembly::default();
        let local = straight(8, 200_000);
        let mut distant = straight(8, 200_000);
        distant.id = 2;
        let shift = 100 * 80 * SCALE;
        for node in &mut distant.nodes {
            node.position = node.position.offset([shift, 0, 0]).unwrap();
        }
        assembly.filaments.insert(local.id, local);
        assembly.filaments.insert(distant.id, distant);
        let mut cache = CollisionCache::default();
        cache.sync_pipe_horizon(&assembly, 0).unwrap();
        let cells = cache.horizon.cells.keys().copied().collect::<Vec<_>>();
        assert!(cells.len() < 64, "horizon allocated {} cells", cells.len());
        let xs = cells.iter().map(|cell| cell[0]).collect::<BTreeSet<_>>();
        assert!(
            xs.len() >= 2,
            "local and distant chunks collapsed into one region"
        );
        let span = xs.last().unwrap() - xs.first().unwrap();
        assert!(span > 50, "100x offset did not separate horizon cells");
        let at = Position::from_absolute([0, 0, 0]).unwrap();
        let bounds_before = cache.profile.pipe_bounds_computed;
        assert!(cache.pipe_beyond_horizon(2, 0, at, 2.).unwrap());
        assert_eq!(cache.profile.pipe_bounds_computed, bounds_before);
    }

    #[test]
    fn interacting_chunks_match_the_per_chunk_horizon_and_omit_distant_pipes() {
        let mut assembly = Assembly::default();
        let local = straight(20, 200_000);
        let mut distant = straight(8, 200_000);
        distant.id = 2;
        let shift = 100 * 80 * SCALE;
        for node in &mut distant.nodes {
            node.position = node.position.offset([shift, 0, 0]).unwrap();
        }
        assembly.filaments.insert(local.id, local.clone());
        assembly.filaments.insert(distant.id, distant);
        let mut cache = CollisionCache::default();
        let at = local.nodes[0].position;
        let got = cache.interacting_chunks(&assembly, 0, at, 1.5).unwrap();
        assert!(
            got.iter().any(|(id, _)| *id == local.id),
            "local chunk missing from the horizon"
        );
        assert!(
            got.iter().all(|(id, _)| *id != 2),
            "distant filament entered the local candidate set: {got:?}"
        );
        let mut expected = Vec::new();
        for f in assembly.filaments.values() {
            if f.nodes.is_empty() {
                continue;
            }
            let count = f.nodes.len().saturating_sub(1).div_ceil(CHUNK_EDGES).max(1);
            for chunk in 0..count {
                if !cache.pipe_beyond_horizon(f.id, chunk, at, 1.5).unwrap() {
                    expected.push((f.id, chunk));
                }
            }
        }
        assert_eq!(got, expected);
        let bounds = cache.profile.pipe_bounds_computed;
        let again = cache.interacting_chunks(&assembly, 0, at, 1.5).unwrap();
        assert_eq!(again, got);
        assert_eq!(
            cache.profile.pipe_bounds_computed, bounds,
            "a repeated query rebuilt chunk bounds"
        );
        let wide = cache.interacting_chunks(&assembly, 0, at, 20_000.).unwrap();
        assert!(
            wide.iter().any(|(id, _)| *id == 2),
            "an enormous query dropped distant geometry"
        );
        let scans = cache.profile.pipe_filament_stamps;
        let unchanged = assembly.clone();
        cache.interacting_chunks(&unchanged, 1, at, 1.5).unwrap();
        assert_eq!(
            cache.profile.pipe_filament_stamps, scans,
            "an unchanged immutable clone rescanned all filament nodes"
        );
        let mut moved = assembly.clone();
        moved.filaments.get_mut(&local.id).unwrap().nodes[3].position =
            Position::from_absolute([0, 40 * SCALE, 0]).unwrap();
        cache.sync_pipe_horizon(&moved, 0).unwrap();
        assert_eq!(
            cache.profile.pipe_horizon_rebuilds, 2,
            "same-tick middle-node edits must invalidate the interaction horizon immediately"
        );
        let new_position = moved.filaments[&local.id].nodes[3].position;
        assert!(
            cache
                .interacting_chunks(&moved, 0, new_position, 0.25)
                .unwrap()
                .contains(&(local.id, 0)),
            "the relocated physical chunk was missed"
        );
        cache.sync_pipe_horizon(&moved, 1).unwrap();
        assert_eq!(
            cache.profile.pipe_horizon_rebuilds, 2,
            "unchanged later ticks must retain the conservative younger bound"
        );
    }

    #[test]
    fn a_local_edit_does_not_recompute_unchanged_distant_bounds() {
        let mut assembly = Assembly::default();
        let mut local = straight(8, 200_000);
        let mut distant = straight(8, 200_000);
        distant.id = 2;
        let shift = 100 * 80 * SCALE;
        for node in &mut distant.nodes {
            node.position = node.position.offset([shift, 0, 0]).unwrap();
        }
        assembly.filaments.insert(local.id, local.clone());
        assembly.filaments.insert(distant.id, distant);
        let mut cache = CollisionCache::default();
        cache.sync_pipe_horizon(&assembly, 0).unwrap();
        let bounds = cache.profile.pipe_bounds_computed;
        assert_eq!(bounds, 2);
        local.nodes.push(Node {
            deposited_tick: 0,
            id: 9,
            parcel: 9,
            position: Position::from_absolute([0, 0, 8 * 200_000]).unwrap(),
        });
        assembly.filaments.insert(local.id, local);
        cache.sync_pipe_horizon(&assembly, 0).unwrap();
        assert_eq!(
            cache.profile.pipe_bounds_computed,
            bounds + 1,
            "a local edit recomputed distant chunk bounds"
        );
        let at = Position::from_absolute([0, 0, 0]).unwrap();
        let near = cache.interacting_chunks(&assembly, 0, at, 1.).unwrap();
        assert!(near.iter().all(|(id, _)| *id != 2));
    }
}
