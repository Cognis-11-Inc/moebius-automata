//! Canonical geometry queries and bounded kinematic contact admission.
//! The library supplies detection; ShipRuntime owns response and contact observations.
use crate::{
    emission::{RigidPose, SCALE},
    filament::{CHUNK_EDGES, filament_chunk_at},
    habitat::Agent,
    runtime::*,
    *,
};
use parry3d::{
    bounding_volume::BoundingVolume,
    math::{Pose as Iso, Rot3, Vector},
    query::{
        self, DefaultQueryDispatcher, NonlinearRigidMotion, QueryDispatcher, ShapeCastOptions,
        ShapeCastStatus,
    },
    shape::{
        Ball, Capsule, Compound, ConvexPolyhedron, Cuboid, Shape, SharedShape, TriMesh,
        TriMeshFlags,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Mutex, OnceLock},
    time::Instant,
};

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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CollisionProfile {
    pub resolve_calls: u64,
    pub resolve_ns: u128,
    pub queries: u64,
    pub query_ns: u128,
    /// Intervals rejected by a conservative AABB distance lower bound.
    pub interval_rejects: u64,
    /// Actual Parry distance evaluations (queries also counts bound-only visits).
    pub distance_queries: u64,
    pub distance_ns: u128,
    pub contact_ns: u128,
    pub cast_cache_hits: u64,
    pub linear_casts: u64,
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
    writhe_slack: f32,
}
/// Filament, chunk, and digest of the geometry inputs including wet mesh time.
type PipeCacheKey = (u64, usize, [u8; 32]);
#[derive(Default)]
pub struct CollisionCache {
    definitions: BTreeMap<(u64, u64, [u8; 32]), Arc<Geometry>>,
    specifications: BTreeMap<u64, (AnatomySpec, [u8; 32])>,
    /// Age stays in the key until the newest sample in the chunk is dry.
    /// Nodes past the one-neighbor lookahead do not.
    pipes: BTreeMap<PipeCacheKey, (Position, Geometry)>,
    profile: CollisionProfile,
    casts: BTreeMap<CastKey, CastReceipt>,
}
type Hit = Option<(f32, [i64; 3], Position)>;
type CastKey = (usize, u32, usize, u32, [i64; 24]);
struct CastReceipt {
    // Keep both allocations alive: an address must never alias a new shape.
    _shapes: [Arc<dyn Shape>; 2],
    hit: Hit,
}
impl CollisionCache {
    fn cast(
        &mut self,
        a: &Geometry,
        from: RigidPose,
        to: RigidPose,
        b: &Geometry,
        bfrom: RigidPose,
        bto: RigidPose,
    ) -> Result<Hit, MorphError> {
        let mut poses = [0; 24];
        for (values, p) in poses
            .as_chunks_mut::<6>()
            .0
            .iter_mut()
            .zip([from, to, bfrom, bto])
        {
            values[..3].copy_from_slice(&p.translation);
            values[3..].copy_from_slice(&[p.yaw_sin, p.yaw_cos, p.axial_roll]);
        }
        let key = (
            Arc::as_ptr(&a.shape) as *const () as usize,
            a.radius.to_bits(),
            Arc::as_ptr(&b.shape) as *const () as usize,
            b.radius.to_bits(),
            poses,
        );
        if let Some(receipt) = self.casts.get(&key) {
            self.profile.cast_cache_hits += 1;
            return Ok(receipt.hit);
        }
        let hit = cast(a, from, to, b, bfrom, bto, &mut self.profile)?;
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
        self.profile
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
    Ok(Geometry {
        shape: Arc::new(mesh),
        radius,
        vertices: Arc::new(vertices),
        parts: vec![],
        outlets: vec![],
        writhe_slack: 0.,
    })
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
                    writhe_slack: 0.,
                },
            ));
        }
        let shape = Compound::new(
            parts
                .iter()
                .map(|(_, part)| (Iso::IDENTITY, SharedShape(part.shape.clone())))
                .collect(),
        );
        let writhe_slack = if body.definition == BLOOD_SAC {
            0.
        } else {
            presentation_slack(spec)?
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
                    writhe_slack: 0.,
                },
            ));
        }
        let Some(key) = pipe_key(id, f, chunk, tick) else {
            return Err(MorphError("pipe chunk has no cached sweep".into()));
        };
        if let Some(g) = self.pipes.get(&key) {
            self.profile.pipe_hits += 1;
            return Ok(g.clone());
        }
        let built = Instant::now();
        self.profile.pipe_builds += 1;
        let (origin, surface) = filament_chunk_at(f, chunk, tick)?;
        let g = geometry(&[surface], 1.)?;
        self.profile.pipe_geometry_ns += built.elapsed().as_nanos();
        self.pipes
            .retain(|(filament, c, _), _| *filament != id || *c != chunk);
        self.pipes.insert(key, (origin, g.clone()));
        Ok((origin, g))
    }
    /// True when skipping this chunk cannot drop a cache fill.
    /// A single-node ball is rebuilt on demand and stores nothing.
    fn pipe_has(&self, f: &crate::filament::Filament, chunk: usize, tick: u64) -> bool {
        match pipe_key(f.id, f, chunk, tick) {
            None => true,
            Some(key) => self.pipes.contains_key(&key),
        }
    }
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
fn near(a: RigidPose, b: RigidPose, ra: f32, rb: f32, da: f32, db: f32) -> bool {
    let d = vec(std::array::from_fn(|k| a.translation[k] - b.translation[k]));
    d.length() <= ra + rb + da + db + 0.003
}
struct QueryNanos<'a> {
    profile: &'a mut CollisionProfile,
    started: Instant,
    visits: u64,
}
impl Drop for QueryNanos<'_> {
    fn drop(&mut self) {
        self.profile.queries += self.visits;
        self.profile.query_ns += self.started.elapsed().as_nanos();
    }
}
fn cast(
    a: &Geometry,
    from: RigidPose,
    to: RigidPose,
    b: &Geometry,
    bfrom: RigidPose,
    bto: RigidPose,
    profile: &mut CollisionProfile,
) -> Result<Option<(f32, [i64; 3], Position)>, MorphError> {
    let mut charge = QueryNanos {
        profile,
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
    let initial = query::contact(
        &m1.start,
        a.shape.as_ref(),
        &m2.start,
        b.shape.as_ref(),
        skin,
    )
    .ok()
    .flatten();
    charge.profile.contact_ns += contact_started.elapsed().as_nanos();
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
        let result = DefaultQueryDispatcher.cast_shapes(
            &m1.start.inv_mul(&m2.start),
            local_delta,
            a.shape.as_ref(),
            b.shape.as_ref(),
            options,
        );
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
                continue;
            }
        }
        charge.profile.distance_queries += 1;
        let distance_started = Instant::now();
        let distance =
            query::distance(&pa, a.shape.as_ref(), &pb, b.shape.as_ref()).map_err(|_| {
                MorphError("unsupported physical distance query; movement refused".into())
            })?;
        charge.profile.distance_ns += distance_started.elapsed().as_nanos();
        if distance > speed * (hi - lo) * 0.5 + skin {
            continue;
        }
        if hi - lo <= 0.00001 || speed == 0. || charge.visits >= 512 {
            let contact_started = Instant::now();
            let closest = query::contact(
                &pa,
                a.shape.as_ref(),
                &pb,
                b.shape.as_ref(),
                skin + speed * (hi - lo),
            )
            .ok()
            .flatten();
            charge.profile.contact_ns += contact_started.elapsed().as_nanos();
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
fn nondeepening_recovery(
    a: &Geometry,
    from: RigidPose,
    to: RigidPose,
    b: &Geometry,
    bfrom: RigidPose,
    bto: RigidPose,
) -> Result<bool, MorphError> {
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
    let contact_at = |t| {
        query::contact(
            &am.position_at_time(t),
            a.shape.as_ref(),
            &bm.position_at_time(t),
            b.shape.as_ref(),
            0.001,
        )
        .map_err(|_| MorphError("unsupported overlap recovery query".into()))
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
    let mut previous = first.dist;
    for t in [0.25, 0.5, 0.75, 1.] {
        let depth = contact_at(t)?.map_or(0.001, |c| c.dist);
        if depth + 0.00001 < previous {
            return Ok(false);
        }
        previous = depth;
    }
    Ok(previous + 0.00001 >= first.dist)
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
    if !next.habitat.as_ref().is_some_and(|h| h.rules.collisions) {
        return Ok(());
    }
    let started = Instant::now();
    cache.casts.clear();
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
                            && nondeepening_recovery(&shapes[&a], pa, ta, &shapes[&b], pb, tb)?
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
                let half = vec(std::array::from_fn(|k| (max[k] - min[k]) / 2))
                    + Vector::splat(shapes[&id].writhe_slack);
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
                        writhe_slack: 0.,
                    },
                ));
            }
            if let Some(assembly) = &before.assembly {
                for f in assembly.filaments.values() {
                    for chunk in 0..f.nodes.len().saturating_sub(1).div_ceil(CHUNK_EDGES).max(1) {
                        // Sample sphere, then the hollow mesh. A miss is still
                        // filled: skipping that fill moved contact samples.
                        let at = Position::from_absolute(start.translation)?;
                        if pipe_separated(
                            cache,
                            f,
                            chunk,
                            before.tick,
                            at,
                            shapes[&id].radius + delta,
                        )? && cache.pipe_has(f, chunk, before.tick)
                        {
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
                        writhe_slack: 0.,
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
            for (owner, pose, g) in static_shapes {
                let target = targets[&id];
                if target == start {
                    break;
                }
                let delta = vec(std::array::from_fn(|k| {
                    target.translation[k] - start.translation[k]
                }))
                .length();
                if !near(start, pose, shapes[&id].radius, g.radius, delta, 0.) {
                    continue;
                }
                if let Some(hit) = cache.cast(&shapes[&id], start, target, &g, pose, pose)? {
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
                        for (component, part) in &shapes[&id].parts {
                            if *component != nozzle
                                && cache.cast(part, start, target, &g, pose, pose)?.is_some()
                            {
                                source = false;
                                break;
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
                            && nondeepening_recovery(&shapes[&id], start, target, &g, pose, pose)?
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
        writhe_slack: 0.,
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
        &mut profile,
    )?
    .is_some())
}

fn overlaps(a: &Geometry, pa: RigidPose, b: &Geometry, pb: RigidPose) -> Result<bool, MorphError> {
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
            writhe_slack: 0.,
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
        for f in a.filaments.values() {
            for chunk in 0..f.nodes.len().saturating_sub(1).div_ceil(CHUNK_EDGES).max(1) {
                let at = Position::from_absolute(pose.translation)?;
                if pipe_separated(cache, f, chunk, world.tick, at, g.radius)?
                    && cache.pipe_has(f, chunk, world.tick)
                {
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
) -> Result<Option<RigidPose>, MorphError> {
    let source = world
        .instances
        .values()
        .find(|i| i.definition == definition)
        .ok_or_else(|| MorphError("spawn definition lacks a physical source".into()))?
        .id;
    let mut cache = CollisionCache::default();
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
            if clear_shape(world, &g, pose, skip, &mut cache)? {
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
            let candidate = spawn_pose(world, BLOOD_SAC, pose, Some(id))?
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
        writhe_slack: 0.,
    };
    let pose = RigidPose {
        translation: center,
        ..RigidPose::identity()
    };
    let mut profile = CollisionProfile::default();
    Ok(cast(&a, from, to, &b, pose, pose, &mut profile)?.is_some())
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
                    writhe_slack: 0.,
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
        for other in assembly
            .filaments
            .values()
            .filter(|other| other.id != f.id && other.organism != book.organism_id)
        {
            for c in 0..other
                .nodes
                .len()
                .saturating_sub(1)
                .div_ceil(CHUNK_EDGES)
                .max(1)
            {
                if pipe_separated(cache, other, c, world.tick, origin, g.radius)?
                    && cache.pipe_has(other, c, world.tick)
                {
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
                writhe_slack: 0.,
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
    if !world.habitat.as_ref().is_some_and(|h| h.rules.collisions) {
        return Ok(None);
    }
    let g = Geometry {
        shape: Arc::new(Ball::new(0.025)),
        radius: 0.025,
        vertices: Arc::new(vec![Vector::ZERO]),
        parts: vec![],
        outlets: vec![],
        writhe_slack: 0.,
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
    for f in assembly.filaments.values() {
        if f.organism == organism && f.nozzle == nozzle {
            continue;
        }
        for chunk in 0..f.nodes.len().saturating_sub(1).div_ceil(CHUNK_EDGES).max(1) {
            let at = Position::from_absolute(origin)?;
            if pipe_separated(cache, f, chunk, world.tick, at, g.radius)?
                && cache.pipe_has(f, chunk, world.tick)
            {
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
            writhe_slack: 0.,
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
        writhe_slack: 0.,
    })
}
/// Physical root growth happens after admitted body motion. The host-pipe base
/// is an authorised graft; the remaining stalk cannot grow through another solid.
pub(crate) fn admit_roots(
    before: &WorldSnapshot,
    next: &mut WorldSnapshot,
    cache: &mut CollisionCache,
) -> Result<(), MorphError> {
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
                    writhe_slack: 0.,
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
            for f in assembly.filaments.values() {
                for chunk in 0..f.nodes.len().saturating_sub(1).div_ceil(CHUNK_EDGES).max(1) {
                    if pipe_separated(cache, f, chunk, next.tick, anchor, g.radius)?
                        && cache.pipe_has(f, chunk, next.tick)
                    {
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
                if blocker.is_some() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filament::{Assembly, Filament, Node, PipeSection};

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
            writhe_slack: 0.,
        };
        let b = Geometry {
            shape: Arc::new(Cuboid::new(Vector::new(0.001, 1., 1.))),
            radius: 2.,
            vertices: Arc::new(vec![Vector::ZERO]),
            parts: vec![],
            outlets: vec![],
            writhe_slack: 0.,
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
                cast(&a, from, to, &b, p, p, &mut profile)
                    .unwrap()
                    .is_some(),
                "rotation changed world-space velocity"
            );
            let moving = RigidPose {
                translation: [2 * SCALE, 0, 0],
                ..p
            };
            assert!(
                cast(&a, from, to, &b, p, moving, &mut profile)
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
            writhe_slack: 0.,
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
        assert!(nondeepening_recovery(&a, from, away, &b, obstacle, obstacle).unwrap());
        assert!(!nondeepening_recovery(&a, from, into, &b, obstacle, obstacle).unwrap());
        let far = RigidPose {
            translation: [500_000, 0, 0],
            ..from
        };
        assert!(
            !nondeepening_recovery(&a, from, into, &b, far, far).unwrap(),
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
            writhe_slack: 0.,
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
                let expected =
                    cast(&a, from, to, &b, p, p, &mut CollisionProfile::default()).unwrap();
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
}
