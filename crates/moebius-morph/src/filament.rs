//! Authored static structural state. Topological continuity is not mechanical equilibrium.
use crate::{
    emission::{MaterialBook, MaterialKind, RigidPose, SCALE},
    runtime::Position,
    *,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ForagingRules {
    pub search_radius: i64,
    pub mouth_radius: i64,
    pub step: i64,
    pub bite: u64,
    pub refill_below: u64,
    pub build_above: u64,
}
impl ForagingRules {
    pub fn validate(&self) -> Result<(), MorphError> {
        require(
            self.mouth_radius > 0
                && self.mouth_radius <= SCALE
                && self.search_radius > self.mouth_radius
                && self.search_radius <= 80 * SCALE
                && self.step > 0
                && self.step <= SCALE / 10
                && (1..=8).contains(&self.bite)
                && self.refill_below < self.build_above
                && self.build_above <= 4096,
            "invalid first-simulation foraging rules",
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConstructionRules {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub foraging: Option<ForagingRules>,
    pub experiment: String,
    pub organism: u64,
    pub nozzle: u64,
    pub route: Vec<RigidPose>,
    pub emit: Vec<bool>,
    pub min_length: i64,
    pub max_length: i64,
    pub capture_radius: i64,
    pub age: u64,
    pub max_nodes: usize,
    pub max_filaments: usize,
    pub max_events: usize,
}
impl ConstructionRules {
    pub fn validate(&self) -> Result<(), MorphError> {
        if let Some(rules) = &self.foraging {
            rules.validate()?;
        }
        require(
            !self.experiment.is_empty()
                && self.experiment.len() <= 128
                && self.organism > 0
                && self.nozzle > 0,
            "construction identity",
        )?;
        require(
            !self.route.is_empty()
                && self.route.len() <= 16384
                && self.emit.len() == self.route.len(),
            "bounded path and emission samples required",
        )?;
        require(
            self.min_length > 0
                && self.max_length >= self.min_length
                && self.max_length <= SCALE
                && self.capture_radius > 0
                && self.capture_radius <= 2 * SCALE
                && self.age <= 64,
            "invalid assembly geometry/age envelope",
        )?;
        require(
            (1..=4096).contains(&self.max_nodes)
                && (1..=4096).contains(&self.max_filaments)
                && (1..=65536).contains(&self.max_events),
            "invalid structural/history budget",
        )?;
        for p in &self.route {
            Position::from_absolute(p.translation)?;
            crate::emission::AuthoredTrajectory {
                origin: p.translation,
                velocity_per_tick: [0; 3],
                yaw_sin: p.yaw_sin,
                yaw_cos: p.yaw_cos,
            }
            .at(0)?;
        }
        Ok(())
    }
    pub fn rigid(&self, tick: u64) -> RigidPose {
        self.route[(tick as usize).min(self.route.len() - 1)]
    }
    pub fn emitting(&self, tick: u64) -> bool {
        self.emit.get(tick as usize).copied().unwrap_or(false)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub id: u64,
    pub parcel: u64,
    pub position: Position,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Chew {
    pub filament: u64,
    pub node: u64,
    pub parcel: u64,
    pub quantity: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Filament {
    pub id: u64,
    pub organism: u64,
    pub nozzle: u64,
    pub created_tick: u64,
    pub experiment: String,
    pub material: MaterialKind,
    pub quantity: u64,
    pub nodes: Vec<Node>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Assembly {
    pub next_id: u64,
    pub filaments: BTreeMap<u64, Filament>,
    pub active: Option<u64>,
}
impl Default for Assembly {
    fn default() -> Self {
        Self {
            next_id: 1,
            filaments: BTreeMap::new(),
            active: None,
        }
    }
}
fn distance2(a: Position, b: Position) -> Result<i128, MorphError> {
    Ok(a.relative(b)?.iter().map(|v| i128::from(*v).pow(2)).sum())
}
impl Assembly {
    pub fn nodes(&self) -> usize {
        self.filaments.values().map(|f| f.nodes.len()).sum()
    }
    /// Caller is the candidate ShipRuntime transaction. No quantity is created here.
    pub fn deposit(
        &mut self,
        book: &mut MaterialBook,
        r: &ConstructionRules,
        parcel: u64,
        nozzle_position: Position,
    ) -> Result<(u64, u64, u64), MorphError> {
        let p = *book
            .parcels
            .iter()
            .find(|p| p.id == parcel)
            .ok_or_else(|| MorphError("NoAvailableMaterial".into()))?;
        require(
            p.organism_id == r.organism
                && p.nozzle_id == r.nozzle
                && p.material == MaterialKind::Secretion,
            "incompatible provenance",
        )?;
        require(
            book.tick.saturating_sub(p.created_tick) >= r.age,
            "MaterialTooYoung",
        )?;
        let position = Position::from_absolute(p.position)?;
        require(
            distance2(position, nozzle_position)? <= i128::from(r.capture_radius).pow(2),
            "BeyondCaptureRadius",
        )?;
        require(self.nodes() < r.max_nodes, "NodeBudget")?;
        let mut id = self.active.unwrap_or(self.next_id);
        if let Some(f) = self.filaments.get(&id) {
            let last = f
                .nodes
                .last()
                .ok_or_else(|| MorphError("empty existing filament".into()))?;
            let d = distance2(last.position, position)?;
            require(d >= i128::from(r.min_length).pow(2), "SegmentTooShort")?;
            if d > i128::from(r.max_length).pow(2) {
                id = self.next_id;
            }
        }
        let new = !self.filaments.contains_key(&id);
        require(
            !new || self.filaments.len() < r.max_filaments,
            "FilamentBudget",
        )?;
        let node = self
            .filaments
            .get(&id)
            .map_or(1, |f| f.nodes.iter().map(|n| n.id).max().unwrap_or(0) + 1);
        let total = self
            .filaments
            .get(&id)
            .map_or(0, |f| f.quantity)
            .checked_add(p.quantity)
            .ok_or_else(|| MorphError("filament quantity overflow".into()))?;
        let next_id = if new {
            self.next_id
                .checked_add(1)
                .ok_or_else(|| MorphError("filament identity exhausted".into()))?
        } else {
            self.next_id
        };
        book.bind(parcel, p.quantity, id, node)?;
        let f = self.filaments.entry(id).or_insert_with(|| Filament {
            id,
            organism: r.organism,
            nozzle: r.nozzle,
            created_tick: book.tick,
            experiment: r.experiment.clone(),
            material: p.material,
            quantity: 0,
            nodes: vec![],
        });
        f.quantity = total;
        f.nodes.push(Node {
            id: node,
            parcel,
            position,
        });
        self.next_id = next_id;
        self.active = Some(id);
        Ok((id, node, p.quantity))
    }
    pub fn validate(&self, book: &MaterialBook, r: &ConstructionRules) -> Result<(), MorphError> {
        require(
            self.nodes() <= r.max_nodes && self.filaments.len() <= r.max_filaments,
            "structural budget",
        )?;
        require(
            self.active
                .is_none_or(|id| self.filaments.contains_key(&id)),
            "dangling active filament",
        )?;
        let mut used = BTreeSet::new();
        for (&id, f) in &self.filaments {
            require(
                id == f.id
                    && id > 0
                    && id < self.next_id
                    && f.organism == r.organism
                    && f.nozzle == r.nozzle
                    && f.experiment == r.experiment
                    && !f.nodes.is_empty()
                    && f.created_tick <= book.tick
                    && f.material == MaterialKind::Secretion,
                "invalid filament identity/provenance",
            )?;
            let mut quantity = 0u64;
            for (n, node) in f.nodes.iter().enumerate() {
                let b = book
                    .bound
                    .iter()
                    .find(|b| b.filament == id && b.node == node.id)
                    .ok_or_else(|| MorphError("structure without bound material".into()))?;
                require(
                    node.id > 0
                        && (n == 0 || node.id > f.nodes[n - 1].id)
                        && used.insert(b.parcel.id)
                        && b.parcel.id == node.parcel
                        && b.parcel.position == node.position.absolute()?
                        && b.parcel.nozzle_id == f.nozzle
                        && b.parcel.organism_id == f.organism,
                    "bound node provenance mismatch",
                )?;
                if n == 0 {
                    require(
                        f.created_tick == b.bound_tick,
                        "filament creation provenance",
                    )?;
                }
                quantity = quantity
                    .checked_add(b.parcel.quantity)
                    .ok_or_else(|| MorphError("bound graph quantity overflow".into()))?;
                if n > 0 {
                    let distance = distance2(f.nodes[n - 1].position, node.position)?;
                    require(
                        distance >= i128::from(r.min_length).pow(2)
                            && distance <= i128::from(r.max_length).pow(2),
                        "invalid structural edge",
                    )?;
                }
            }
            require(quantity == f.quantity, "filament cached quantity mismatch")?;
        }
        require(
            used.len() == book.bound.len(),
            "unrepresented bound material",
        )?;
        Ok(())
    }

    /// Take one dried node near `at` back into the reservoir.
    /// The fresh tip is left alone. A bite that would break an edge is refused.
    pub fn chew_dried(
        &mut self,
        book: &mut MaterialBook,
        rules: &ConstructionRules,
        at: Position,
        reach: i64,
        dried_age: u64,
        fresh_tail: usize,
    ) -> Result<Option<Chew>, MorphError> {
        let limit = i128::from(reach) * i128::from(reach);
        let mut choice: Option<(i128, u64, usize)> = None;
        for (filament_id, filament) in &self.filaments {
            if filament.organism != book.organism_id {
                continue;
            }
            let keep = fresh_tail.min(filament.nodes.len());
            let edible = filament.nodes.len().saturating_sub(keep);
            for (index, node) in filament.nodes.iter().enumerate().take(edible) {
                let Some(binding) = book
                    .bound
                    .iter()
                    .find(|b| b.filament == *filament_id && b.node == node.id)
                else {
                    continue;
                };
                if book.tick.saturating_sub(binding.bound_tick) < dried_age {
                    continue;
                }
                let delta = at.relative(node.position)?;
                let distance: i128 = delta.iter().map(|v| i128::from(*v) * i128::from(*v)).sum();
                if distance > limit {
                    continue;
                }
                if !edge_survives(filament, index, rules)? {
                    continue;
                }
                if choice.is_none_or(|(best, _, _)| distance < best) {
                    choice = Some((distance, *filament_id, index));
                }
            }
        }
        let Some((_, filament_id, index)) = choice else {
            return Ok(None);
        };
        let (node, quantity, empty) = {
            let filament = self
                .filaments
                .get_mut(&filament_id)
                .ok_or_else(|| MorphError("chew target missing".into()))?;
            let node = filament.nodes[index].clone();
            let quantity = book.reclaim_bound(filament_id, node.id, dried_age)?;
            filament.nodes.remove(index);
            filament.quantity = filament.quantity.saturating_sub(quantity);
            if index == 0 && !filament.nodes.is_empty() {
                let first = filament.nodes[0].id;
                let born = book
                    .bound
                    .iter()
                    .find(|b| b.filament == filament_id && b.node == first)
                    .map(|b| b.bound_tick)
                    .ok_or_else(|| MorphError("chewed filament lost its root".into()))?;
                filament.created_tick = born;
            }
            (node, quantity, filament.nodes.is_empty())
        };
        if empty {
            self.filaments.remove(&filament_id);
            if self.active == Some(filament_id) {
                self.active = None;
            }
        }
        Ok(Some(Chew {
            filament: filament_id,
            node: node.id,
            parcel: node.parcel,
            quantity,
        }))
    }
}

fn edge_survives(
    filament: &Filament,
    index: usize,
    rules: &ConstructionRules,
) -> Result<bool, MorphError> {
    let left = index.checked_sub(1);
    let right = index + 1;
    if let (Some(left), true) = (left, right < filament.nodes.len()) {
        let distance = distance2(
            filament.nodes[left].position,
            filament.nodes[right].position,
        )?;
        return Ok(distance >= i128::from(rules.min_length).pow(2)
            && distance <= i128::from(rules.max_length).pow(2));
    }
    Ok(true)
}
/// Shared authored pose for numerical nozzle and visual body. No neural/physical controller.
pub fn construction_pose(tick: u64) -> Pose {
    let phase = (tick.min(1_000_000) as f32) * 0.045;
    Pose {
        phase,
        bend: 0.8,
        mouth: 0.7 + 0.25 * (phase * 1.2).sin(),
        flagella: 1.,
    }
}
/// Recorded 3D experiment; one true anatomical nozzle and finite reserves.
pub fn construction_fixture(
    seed: u64,
    steps: usize,
    reserve: u64,
    kind: &str,
) -> Result<crate::runtime::ShipRuntime, MorphError> {
    construction_from(AnatomySpec::showcase(seed), seed, steps, reserve, kind)
}

/// Same authored route as `construction_fixture`, using a body the player designed.
pub fn construction_from(
    spec: AnatomySpec,
    seed: u64,
    steps: usize,
    reserve: u64,
    kind: &str,
) -> Result<crate::runtime::ShipRuntime, MorphError> {
    use crate::{
        emission::{Environment, MaterialBook},
        runtime::*,
    };
    require((1..=16384).contains(&steps), "bounded fixture steps")?;
    spec.validate()?;
    let nozzle = spec
        .components
        .iter()
        .find(|c| matches!(c.shape, Shape::SecretionNozzle { .. }))
        .ok_or_else(|| MorphError("fixture nozzle".into()))?
        .id;
    let mut route = vec![];
    for k in 0..steps {
        let t = k as f64 * 0.025;
        let p = match kind {
            "vertical" => [0, (t * SCALE as f64) as i64, 0],
            "curved" | "head-first" => [
                (t * SCALE as f64) as i64,
                ((if kind == "head-first" { 0.12 } else { 0.8 }) * (t * 0.7).sin() * SCALE as f64)
                    as i64,
                (0.7 * (t * 0.5).sin() * SCALE as f64) as i64,
            ],
            "gap" => [
                ((t + if k > steps / 2 { 4. } else { 0. }) * SCALE as f64) as i64,
                0,
                0,
            ],
            _ => [(t * SCALE as f64) as i64, 0, 0],
        };
        route.push(RigidPose {
            translation: p,
            ..RigidPose::identity()
        });
    }
    if kind == "head-first" {
        // Anatomy's tail-to-mouth axis is +Z. Align its yaw with the recorded
        // path tangent; retain only a gentle rise because RigidPose supports yaw.
        // This is a new authored demonstration, not a change to earlier trials.
        for k in 0..route.len() {
            let (a, b) = if k + 1 < route.len() {
                (k, k + 1)
            } else {
                (k.saturating_sub(1), k)
            };
            let dx = (route[b].translation[0] - route[a].translation[0]) as f64;
            let dz = (route[b].translation[2] - route[a].translation[2]) as f64;
            let yaw = dx.atan2(dz);
            route[k].yaw_sin = (yaw.sin() * SCALE as f64).round() as i64;
            route[k].yaw_cos = (yaw.cos() * SCALE as f64).round() as i64;
        }
    }
    let r = ConstructionRules {
        foraging: None,
        experiment: format!("MORPH-005/{kind}/seed{seed}"),
        organism: 1,
        nozzle,
        route,
        emit: vec![true; steps],
        min_length: 1000,
        max_length: 500_000,
        capture_radius: 750_000,
        age: 2,
        max_nodes: 4096,
        max_filaments: 128,
        max_events: 65536,
    };
    let position = Position::from_absolute(r.rigid(0).translation)?;
    let environment = Environment::suspended();
    let mut book = MaterialBook::new(1, seed, reserve, environment, 4096)?;
    book.sync_nozzles(&spec);
    book.emission_enabled = true;
    book.launch_speed = 2000;
    let config = WorldConfig {
        version: 2,
        seed,
        tick_nanos: TICK_NANOS,
        environment,
        ambient_light: 10,
        ports: vec![],
        definitions: BTreeMap::from([(1, spec)]),
        construction: Some(r),
    };
    ShipRuntime::new(
        config,
        vec![Instance {
            id: 1,
            definition: 1,
            position,
            velocity: [0; 3],
            exposure: 10,
            book,
        }],
    )
}

/// Derived presentation index; never serialized as authoritative state.16-edge chunks,
/// conservative occupied region bins. No dense global region enumeration.
#[derive(Default)]
pub struct StructuralIndex {
    pub chunks: BTreeMap<(u64, usize), ([i64; 3], [i64; 3])>,
    bins: BTreeMap<[i64; 3], BTreeSet<(u64, usize)>>,
}
pub const CHUNK_EDGES: usize = 16;
impl StructuralIndex {
    pub fn build(a: &Assembly) -> Result<Self, MorphError> {
        let mut index = Self::default();
        for f in a.filaments.values() {
            for start in (0..f.nodes.len().saturating_sub(1)).step_by(CHUNK_EDGES) {
                let end = (start + CHUNK_EDGES + 1).min(f.nodes.len());
                let mut low = [i64::MAX; 3];
                let mut high = [i64::MIN; 3];
                for n in &f.nodes[start..end] {
                    let p = n.position.absolute()?;
                    for axis in 0..3 {
                        low[axis] = low[axis].min(p[axis]);
                        high[axis] = high[axis].max(p[axis]);
                    }
                }
                let key = (f.id, start / CHUNK_EDGES);
                index.chunks.insert(key, (low, high));
                let first = low.map(|v| v.div_euclid(crate::runtime::REGION));
                let last = high.map(|v| v.div_euclid(crate::runtime::REGION));
                require(
                    (0..3).all(|i| last[i] - first[i] <= 1),
                    "chunk exceeds sparse bin envelope",
                )?;
                for x in first[0]..=last[0] {
                    for y in first[1]..=last[1] {
                        for z in first[2]..=last[2] {
                            index.bins.entry([x, y, z]).or_default().insert(key);
                        }
                    }
                }
            }
        }
        Ok(index)
    }
    pub fn visible(
        &self,
        sphere: &crate::witness::WitnessSphere,
    ) -> Result<Vec<(u64, usize)>, MorphError> {
        require(
            sphere.radius >= 0 && sphere.radius <= crate::runtime::MAX_POSITION,
            "invalid structural witness radius",
        )?;
        let p = sphere.center.absolute()?;
        let first = p.map(|v| (v - sphere.radius).div_euclid(crate::runtime::REGION));
        let last = p.map(|v| (v + sphere.radius).div_euclid(crate::runtime::REGION));
        let mut candidates: BTreeSet<(u64, usize)> = BTreeSet::new();
        for (region, keys) in &self.bins {
            if (0..3).all(|i| region[i] >= first[i] && region[i] <= last[i]) {
                candidates.extend(keys.iter().copied());
            }
        }
        Ok(candidates
            .into_iter()
            .filter(|k| {
                let (low, high) = self.chunks[k];
                (0..3)
                    .map(|i| {
                        i128::from(if p[i] < low[i] {
                            low[i] - p[i]
                        } else if p[i] > high[i] {
                            p[i] - high[i]
                        } else {
                            0
                        })
                        .pow(2)
                    })
                    .sum::<i128>()
                    <= i128::from(sphere.radius).pow(2)
            })
            .collect())
    }
}
/// Presentation surface from existing graph samples; no quantity writes or implicit support.
pub fn filament_chunk(f: &Filament, chunk: usize) -> Result<(Position, Surface), MorphError> {
    use crate::math::*;
    let start = chunk * CHUNK_EDGES;
    let end = (start + CHUNK_EDGES + 1).min(f.nodes.len());
    require(end > start + 1, "no structural edge in chunk")?;
    let origin = f.nodes[start].position;
    let points = f.nodes[start..end]
        .iter()
        .map(|n| {
            n.position
                .relative(origin)
                .map(|v| v.map(|v| v as f32 / SCALE as f32))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut mesh = Surface::new("material_backed_filament_chunk");
    // Muted warm chitin/flesh, distinct from the living body's violet skin.
    let color = [0.48, 0.30, 0.22];
    let cap = mesh.vertex(points[0], [0.; 2], color);
    let mut previous = 0;
    let mut side = [1., 0., 0.];
    let sides = 8;
    for i in 0..points.len() {
        let t = if i == 0 {
            sub(points[1], points[0])
        } else if i + 1 == points.len() {
            sub(points[i], points[i - 1])
        } else {
            sub(points[i + 1], points[i - 1])
        };
        let (x, y) = frame(unit(t), side);
        side = x;
        let ring = crate::compiler::ring(
            &mut mesh,
            points[i],
            (x, y),
            [0.025, 0.025],
            i as f32,
            sides,
            color,
        );
        if i == 0 {
            for j in 0..sides as u32 {
                mesh.triangle(cap, ring + (j + 1) % sides as u32, ring + j);
            }
        } else {
            mesh.bridge(previous, ring, sides);
        }
        previous = ring;
    }
    let tip = mesh.vertex(*points.last().unwrap(), [0.5, 1.], color);
    for j in 0..sides as u32 {
        mesh.triangle(previous + j, previous + (j + 1) % sides as u32, tip);
    }
    mesh.normals();
    Ok((origin, mesh))
}
