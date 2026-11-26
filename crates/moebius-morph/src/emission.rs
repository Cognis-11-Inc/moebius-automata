//! Experiment-local secretion book for MORPH-003.
//!
//! This is not `moebius_core::World` and it does not replace it. Nursery jets
//! (`Jet`, `World::advance_jets`, `SECRETE_ACTION`) stay on the periodic lattice,
//! debit nutrient and energy, and can impulse a body. Those paths are not called
//! here. `moebius-morph` does not depend on `moebius-core`.
//!
//! Anatomy still only names a nozzle. `EmissionMetadata.operational` stays false.
//! This book admits material, then reports that nozzle as runtime-operational.
//! After admission the book owns the parcel. Rendering may read the parcels.
//! It does not create, move, or delete them.
//!
//! Quantity is an integer quantum. One quantum leaves the reservoir and enters
//! one parcel, or an explicit sink. Position and velocity are fixed-point
//! micro-units: 1_000_000 micro-units = 1 anatomical world unit. One `step`
//! is one fixed tick. There is no random term. The recorded seed does not
//! affect motion.

use crate::{
    AnatomySpec, CompiledAnatomy, EmissionMetadata, MorphError, OutputFrame, Pose, Shape,
    write_atomic,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

pub const SCALE: i64 = 1_000_000;
pub const MAX_PARCELS: usize = 4096;
pub const BOOK_VERSION: u32 = 1;
/// Explicit opt-in serialization for books with provenance-tracked food intake.
pub const FED_BOOK_VERSION: u32 = 3;
pub const SECRETION_CHANNEL: &str = "future_conserved_substrate";
pub const MAX_LAUNCH: i64 = 1_000_000_000;
const BENCHMARK_SEED: u64 = 20261123;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Environment {
    /// 0 leaves velocity unchanged by the medium. 1000 replaces it with flow
    /// when the parcel response is also 1000. This is not an atmosphere.
    pub drag_per_mille: u16,
    /// Added to velocity on every tick, in micro-units per tick.
    pub acceleration: [i64; 3],
    /// Uniform medium velocity, micro-units per tick.
    pub medium_flow: [i64; 3],
    /// Scales drag for parcels born while this value is current. 1000 is full response.
    pub response_per_mille: u16,
}

impl Environment {
    pub fn suspended() -> Self {
        Self {
            drag_per_mille: 0,
            acceleration: [0, 0, 0],
            medium_flow: [0, 0, 0],
            response_per_mille: 1000,
        }
    }

    pub fn validate(self) -> Result<(), MorphError> {
        require_book(
            self.drag_per_mille <= 1000 && self.response_per_mille <= 1000,
            "drag and response are per-mille in 0..=1000",
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialKind {
    Secretion,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Parcel {
    pub id: u64,
    pub organism_id: u64,
    pub nozzle_id: u64,
    pub position: [i64; 3],
    pub velocity: [i64; 3],
    pub quantity: u64,
    pub material: MaterialKind,
    pub created_tick: u64,
    pub response_per_mille: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum EmissionStatus {
    Admitted { parcel_id: u64, quantity: u64 },
    Disabled,
    ReservoirExhausted { requested: u64, available: u64 },
    CapacityExhausted { capacity: u64 },
    DuplicateTick { nozzle_id: u64, tick: u64 },
    UnknownNozzle { nozzle_id: u64 },
    ZeroQuantity,
    ChannelMismatch,
    BadDirection,
    BadLaunch,
    IdentityExhausted,
}

impl EmissionStatus {
    pub fn admitted(self) -> bool {
        matches!(self, Self::Admitted { .. })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EmitRequest<'a> {
    pub nozzle_id: u64,
    pub origin: [i64; 3],
    pub direction: [i64; 3],
    pub quantity: u64,
    pub launch_speed: i64,
    pub channel: &'a str,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialBook {
    /// Conserved structural stock; empty fields preserve old v1 JSON/hash identities.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bound: Vec<Binding>,
    pub version: u32,
    pub organism_id: u64,
    /// Recorded experiment identity. Motion does not read it.
    pub seed: u64,
    pub tick: u64,
    pub initial: u64,
    /// Harvest credited when a feeding apparatus takes generic food.
    pub external_input: u64,
    pub reservoir: u64,
    pub sinks: u64,
    pub next_id: u64,
    pub environment: Environment,
    pub parcels: Vec<Parcel>,
    pub registered: BTreeSet<u64>,
    /// Nozzles that have had at least one admitted parcel.
    pub admitted: BTreeSet<u64>,
    pub last_admit_tick: BTreeMap<u64, u64>,
    pub emission_enabled: bool,
    pub rate: u64,
    pub launch_speed: i64,
    pub capacity: u64,
    pub last_status: EmissionStatus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Binding {
    pub filament: u64,
    pub node: u64,
    pub bound_tick: u64,
    pub parcel: Parcel,
}

fn zero(v: &u64) -> bool {
    *v == 0
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ledger {
    #[serde(default, skip_serializing_if = "zero")]
    pub bound: u64,
    pub initial: u64,
    pub external_input: u64,
    pub reservoir: u64,
    pub in_world: u64,
    pub sinks: u64,
    pub residual: i128,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidPose {
    /// Local +Z axial rotation, applied before world yaw. Absent in legacy poses.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub axial_roll: i64,
    pub translation: [i64; 3],
    pub yaw_sin: i64,
    pub yaw_cos: i64,
}

impl RigidPose {
    pub fn identity() -> Self {
        Self {
            axial_roll: 0,
            translation: [0, 0, 0],
            yaw_sin: 0,
            yaw_cos: SCALE,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthoredTrajectory {
    pub origin: [i64; 3],
    pub velocity_per_tick: [i64; 3],
    pub yaw_sin: i64,
    pub yaw_cos: i64,
}

impl AuthoredTrajectory {
    pub fn stationary() -> Self {
        Self {
            origin: [0, 0, 0],
            velocity_per_tick: [0, 0, 0],
            yaw_sin: 0,
            yaw_cos: SCALE,
        }
    }

    pub fn at(self, tick: u64) -> Result<RigidPose, MorphError> {
        require_book(
            yaw_ok(self.yaw_sin, self.yaw_cos),
            "trajectory yaw is not a unit angle",
        )?;
        let mut translation = self.origin;
        for (axis, slot) in translation.iter_mut().enumerate() {
            let delta = (self.velocity_per_tick[axis] as i128)
                .checked_mul(i128::from(tick))
                .ok_or_else(|| MorphError("trajectory overflow".into()))?;
            *slot = add_i64(i128::from(*slot), delta, "trajectory")?;
        }
        Ok(RigidPose {
            axial_roll: 0,
            translation,
            yaw_sin: self.yaw_sin,
            yaw_cos: self.yaw_cos,
        })
    }
}

pub(crate) fn is_zero(value: &i64) -> bool {
    *value == 0
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlacedNozzle {
    pub nozzle_id: u64,
    pub origin: [i64; 3],
    pub direction: [i64; 3],
    pub anatomical_operational: bool,
}

impl MaterialBook {
    pub fn new(
        organism_id: u64,
        seed: u64,
        initial: u64,
        environment: Environment,
        capacity: usize,
    ) -> Result<Self, MorphError> {
        environment.validate()?;
        require_book(
            (1..=MAX_PARCELS).contains(&capacity),
            "capacity must be 1..=4096",
        )?;
        Ok(Self {
            bound: vec![],
            version: BOOK_VERSION,
            organism_id,
            seed,
            tick: 0,
            initial,
            external_input: 0,
            reservoir: initial,
            sinks: 0,
            next_id: 1,
            environment,
            parcels: Vec::new(),
            registered: BTreeSet::new(),
            admitted: BTreeSet::new(),
            last_admit_tick: BTreeMap::new(),
            emission_enabled: false,
            rate: 1,
            launch_speed: 0,
            capacity: capacity as u64,
            last_status: EmissionStatus::Disabled,
        })
    }

    pub fn register_nozzle(&mut self, id: u64, channel: &str) -> Result<(), MorphError> {
        require_book(
            channel == SECRETION_CHANNEL,
            "nozzle channel is not the secretion substrate",
        )?;
        self.registered.insert(id);
        Ok(())
    }

    pub fn sync_nozzles(&mut self, spec: &AnatomySpec) {
        self.registered.clear();
        for component in &spec.components {
            if let Shape::SecretionNozzle { output, .. } = &component.shape
                && output.material_channel == SECRETION_CHANNEL
                && !output.operational
            {
                self.registered.insert(component.id);
            }
        }
    }

    pub fn runtime_operational(&self, nozzle_id: u64) -> bool {
        self.admitted.contains(&nozzle_id)
    }

    pub fn in_world(&self) -> Result<u64, MorphError> {
        let mut total = 0u64;
        for parcel in &self.parcels {
            total = total
                .checked_add(parcel.quantity)
                .ok_or_else(|| MorphError("in-world quantity overflow".into()))?;
        }
        Ok(total)
    }

    pub fn bound_quantity(&self) -> Result<u64, MorphError> {
        self.bound.iter().try_fold(0u64, |n, b| {
            n.checked_add(b.parcel.quantity)
                .ok_or_else(|| MorphError("bound stock overflow".into()))
        })
    }
    /// Whole-parcel transactional free -> bound transfer. No graph/renderer owns quantity.
    pub fn bind(
        &mut self,
        parcel_id: u64,
        quantity: u64,
        filament: u64,
        node: u64,
    ) -> Result<Binding, MorphError> {
        require_book(
            quantity > 0 && filament > 0 && node > 0,
            "invalid structural transfer",
        )?;
        require_book(
            !self
                .bound
                .iter()
                .any(|b| b.parcel.id == parcel_id || (b.filament == filament && b.node == node)),
            "duplicate structural transfer",
        )?;
        let index = self
            .parcels
            .iter()
            .position(|p| p.id == parcel_id)
            .ok_or_else(|| MorphError("unavailable admitted material".into()))?;
        require_book(
            self.parcels[index].quantity == quantity,
            "whole-parcel transfer requires exact available quantity",
        )?;
        let mut next = self.clone();
        let parcel = next.parcels.remove(index);
        let binding = Binding {
            filament,
            node,
            bound_tick: self.tick,
            parcel,
        };
        next.bound.push(binding);
        next.version = next.version.max(2);
        require_book(
            next.conservation_holds(),
            "structural transfer violates conservation",
        )?;
        *self = next;
        Ok(binding)
    }

    /// Bound secretion returns to the reservoir. The parcel leaves the book.
    pub fn reclaim_bound(
        &mut self,
        filament: u64,
        node: u64,
        dried_age: u64,
    ) -> Result<u64, MorphError> {
        let index = self
            .bound
            .iter()
            .position(|b| b.filament == filament && b.node == node)
            .ok_or_else(|| MorphError("dried secretion is not bound".into()))?;
        require_book(
            self.tick.saturating_sub(self.bound[index].bound_tick) >= dried_age,
            "secretion is still wet",
        )?;
        let binding = self.bound.remove(index);
        self.reservoir = self
            .reservoir
            .checked_add(binding.parcel.quantity)
            .ok_or_else(|| MorphError("reservoir overflow".into()))?;
        require_book(
            self.conservation_holds(),
            "chewing secretion violates conservation",
        )?;
        Ok(binding.parcel.quantity)
    }

    /// A bite becomes gut stock. The food ledger paid for it.
    pub fn ingest(&mut self, quantity: u64) -> Result<(), MorphError> {
        require_book(quantity > 0, "empty meal")?;
        require_book(self.conservation_holds(), "cannot feed an unbalanced book")?;
        let input = self
            .external_input
            .checked_add(quantity)
            .ok_or_else(|| MorphError("harvest overflow".into()))?;
        let reservoir = self
            .reservoir
            .checked_add(quantity)
            .ok_or_else(|| MorphError("reservoir overflow".into()))?;
        self.external_input = input;
        self.reservoir = reservoir;
        self.version = FED_BOOK_VERSION;
        require_book(self.conservation_holds(), "a meal violates conservation")?;
        Ok(())
    }

    /// Parent pays for a child. The cost leaves this book as an accounted endowment.
    pub fn endow(&mut self, cost: u64) -> Result<(), MorphError> {
        require_book(cost > 0 && self.reservoir >= cost, "cannot endow a child")?;
        self.reservoir -= cost;
        self.sinks = self
            .sinks
            .checked_add(cost)
            .ok_or_else(|| MorphError("endowment overflow".into()))?;
        require_book(self.conservation_holds(), "a birth violates conservation")?;
        Ok(())
    }
    /// `reservoir + free + bound + sinks - (initial + authorized input)`.
    ///
    /// Zero is the conservation invariant. Version3 records paid food input.
    pub fn residual(&self) -> Result<i128, MorphError> {
        let world = i128::from(self.in_world()?);
        let have = i128::from(self.reservoir)
            .checked_add(world)
            .and_then(|sum| sum.checked_add(i128::from(self.bound_quantity().ok()?)))
            .and_then(|sum| sum.checked_add(i128::from(self.sinks)))
            .ok_or_else(|| MorphError("ledger sum overflow".into()))?;
        let sources = i128::from(self.initial)
            .checked_add(i128::from(self.external_input))
            .ok_or_else(|| MorphError("ledger source overflow".into()))?;
        Ok(have - sources)
    }

    pub fn conservation_holds(&self) -> bool {
        self.residual().ok() == Some(0)
    }

    pub fn ledger(&self) -> Result<Ledger, MorphError> {
        Ok(Ledger {
            bound: self.bound_quantity()?,
            initial: self.initial,
            external_input: self.external_input,
            reservoir: self.reservoir,
            in_world: self.in_world()?,
            sinks: self.sinks,
            residual: self.residual()?,
        })
    }

    pub fn try_emit(&mut self, request: EmitRequest<'_>) -> EmissionStatus {
        let status = self.admit(request);
        self.last_status = status;
        status
    }

    fn admit(&mut self, request: EmitRequest<'_>) -> EmissionStatus {
        if !self.emission_enabled {
            return EmissionStatus::Disabled;
        }
        if request.quantity == 0 {
            return EmissionStatus::ZeroQuantity;
        }
        if request.channel != SECRETION_CHANNEL {
            return EmissionStatus::ChannelMismatch;
        }
        if !self.registered.contains(&request.nozzle_id) {
            return EmissionStatus::UnknownNozzle {
                nozzle_id: request.nozzle_id,
            };
        }
        if self.last_admit_tick.get(&request.nozzle_id) == Some(&self.tick) {
            return EmissionStatus::DuplicateTick {
                nozzle_id: request.nozzle_id,
                tick: self.tick,
            };
        }
        if !direction_ok(request.direction) {
            return EmissionStatus::BadDirection;
        }
        if !(0..=MAX_LAUNCH).contains(&request.launch_speed) {
            return EmissionStatus::BadLaunch;
        }
        if (self.parcels.len() + self.bound.len()) as u64 >= self.capacity {
            return EmissionStatus::CapacityExhausted {
                capacity: self.capacity,
            };
        }
        if self.reservoir < request.quantity {
            return EmissionStatus::ReservoirExhausted {
                requested: request.quantity,
                available: self.reservoir,
            };
        }
        if self.next_id == u64::MAX {
            return EmissionStatus::IdentityExhausted;
        }
        let velocity = launch_velocity(request.direction, request.launch_speed);
        let parcel_id = self.next_id;
        let parcel = Parcel {
            id: parcel_id,
            organism_id: self.organism_id,
            nozzle_id: request.nozzle_id,
            position: request.origin,
            velocity,
            quantity: request.quantity,
            material: MaterialKind::Secretion,
            created_tick: self.tick,
            response_per_mille: self.environment.response_per_mille,
        };
        self.reservoir -= request.quantity;
        self.next_id += 1;
        self.last_admit_tick.insert(request.nozzle_id, self.tick);
        self.admitted.insert(request.nozzle_id);
        self.parcels.push(parcel);
        debug_assert!(self.conservation_holds());
        EmissionStatus::Admitted {
            parcel_id,
            quantity: request.quantity,
        }
    }

    /// Integrate every parcel one tick, then advance `tick`.
    /// A parcel admitted at the current tick is not moved until a later step.
    pub fn step(&mut self) -> Result<(), MorphError> {
        #[cfg(feature = "allocation-attribution")]
        let _allocation_scope = crate::profiling::enter(11);

        self.environment.validate()?;
        let mut next = Vec::with_capacity(self.parcels.len());
        for parcel in &self.parcels {
            next.push(integrate(parcel, &self.environment)?);
        }
        let tick = self
            .tick
            .checked_add(1)
            .ok_or_else(|| MorphError("tick overflow".into()))?;
        self.parcels = next;
        self.tick = tick;
        debug_assert!(self.conservation_holds());
        Ok(())
    }

    /// Explicit accounted sink. This is not decay, metabolism, or cleanup.
    pub fn sink_parcel(&mut self, id: u64, quantity: u64) -> Result<(), MorphError> {
        let parcel = self
            .parcels
            .iter_mut()
            .find(|parcel| parcel.id == id)
            .ok_or_else(|| MorphError("sink target is not in the book".into()))?;
        require_book(
            quantity > 0 && quantity <= parcel.quantity,
            "sink quantity is outside the parcel",
        )?;
        parcel.quantity -= quantity;
        self.sinks = self
            .sinks
            .checked_add(quantity)
            .ok_or_else(|| MorphError("sink overflow".into()))?;
        if parcel.quantity == 0 {
            self.parcels.retain(|parcel| parcel.id != id);
        }
        debug_assert!(self.conservation_holds());
        Ok(())
    }

    /// Honest-red injector. Copies quantity into a new parcel without a debit.
    pub fn fault_duplicate_last(&mut self) -> bool {
        let Some(mut parcel) = self.parcels.last().cloned() else {
            return false;
        };
        if self.next_id == u64::MAX {
            return false;
        }
        parcel.id = self.next_id;
        self.next_id += 1;
        self.parcels.push(parcel);
        true
    }

    /// Honest-red injector. Drops a parcel without writing the sink ledger.
    pub fn fault_drop_last(&mut self) -> bool {
        self.parcels.pop().is_some()
    }

    pub fn to_json(&self) -> Result<String, MorphError> {
        self.validate_loaded()?;
        serde_json::to_string_pretty(self).map_err(|error| MorphError(error.to_string()))
    }

    pub fn from_json(json: &str) -> Result<Self, MorphError> {
        let book: Self =
            serde_json::from_str(json).map_err(|error| MorphError(error.to_string()))?;
        book.validate_loaded()?;
        Ok(book)
    }

    pub fn save(&self, path: &Path) -> Result<(), MorphError> {
        write_atomic(path, &self.to_json()?)
    }

    pub fn load(path: &Path) -> Result<Self, MorphError> {
        let json = std::fs::read_to_string(path).map_err(|error| MorphError(error.to_string()))?;
        Self::from_json(&json)
    }

    pub fn state_hash(&self) -> Result<String, MorphError> {
        let bytes = serde_json::to_vec(self).map_err(|error| MorphError(error.to_string()))?;
        Ok(blake3::hash(&bytes).to_hex().to_string())
    }

    fn validate_loaded(&self) -> Result<(), MorphError> {
        require_book(
            (self.version == BOOK_VERSION && self.bound.is_empty())
                || self.version == 2
                || self.version == FED_BOOK_VERSION,
            "unsupported material book/structural version",
        )?;
        self.environment.validate()?;
        require_book(
            self.external_input == 0 || self.version == FED_BOOK_VERSION,
            "external credit requires a fed material book",
        )?;
        require_book(
            (1..=MAX_PARCELS as u64).contains(&self.capacity),
            "capacity out of bounds",
        )?;
        require_book(
            (self.parcels.len() + self.bound.len()) as u64 <= self.capacity,
            "parcel count over capacity",
        )?;
        require_book(self.rate >= 1, "rate must be at least 1")?;
        require_book(
            (0..=MAX_LAUNCH).contains(&self.launch_speed),
            "launch speed out of bounds",
        )?;
        require_book(self.conservation_holds(), "loaded book breaks conservation")?;
        let mut ids = BTreeSet::new();
        let mut bindings = BTreeSet::new();
        for b in &self.bound {
            require_book(
                b.filament > 0
                    && b.node > 0
                    && b.bound_tick <= self.tick
                    && b.bound_tick >= b.parcel.created_tick
                    && bindings.insert((b.filament, b.node)),
                "invalid bound provenance",
            )?;
        }
        for parcel in self
            .parcels
            .iter()
            .chain(self.bound.iter().map(|b| &b.parcel))
        {
            require_book(ids.insert(parcel.id), "duplicate parcel id")?;
            require_book(parcel.id < self.next_id, "parcel id is past next_id")?;
            require_book(parcel.quantity >= 1, "empty parcel stored")?;
            require_book(
                parcel.created_tick <= self.tick,
                "parcel created in the future",
            )?;
            require_book(
                parcel.organism_id == self.organism_id,
                "parcel organism does not match the book",
            )?;
            require_book(
                parcel.material == MaterialKind::Secretion,
                "unknown material",
            )?;
            require_book(
                parcel.response_per_mille <= 1000,
                "parcel response out of bounds",
            )?;
        }
        for nozzle in &self.admitted {
            require_book(
                self.last_admit_tick.contains_key(nozzle),
                "admitted nozzle has no admission tick",
            )?;
        }
        for (nozzle, tick) in &self.last_admit_tick {
            require_book(*tick <= self.tick, "admission tick is in the future")?;
            require_book(
                self.admitted.contains(nozzle),
                "admission tick without admission",
            )?;
        }
        Ok(())
    }
}

pub fn position_from_f32(point: [f32; 3]) -> Result<[i64; 3], MorphError> {
    let mut out = [0; 3];
    for (axis, value) in point.into_iter().enumerate() {
        require_book(value.is_finite(), "non-finite position")?;
        let scaled = f64::from(value) * SCALE as f64;
        require_book(
            scaled.is_finite() && scaled.abs() <= i64::MAX as f64,
            "position does not fit the fixed point",
        )?;
        out[axis] = scaled.round() as i64;
    }
    Ok(out)
}

pub fn direction_from_f32(direction: [f32; 3]) -> Result<[i64; 3], MorphError> {
    require_book(
        direction.iter().all(|value| value.is_finite()),
        "non-finite direction",
    )?;
    let length = direction
        .iter()
        .map(|value| f64::from(*value) * f64::from(*value))
        .sum::<f64>()
        .sqrt();
    require_book(length.is_finite() && length > 1e-8, "degenerate direction")?;
    let mut out = [0; 3];
    for (axis, value) in direction.into_iter().enumerate() {
        let scaled = f64::from(value) / length * SCALE as f64;
        out[axis] = scaled.round() as i64;
    }
    require_book(direction_ok(out), "direction left the unit shell")?;
    Ok(out)
}

/// The same rigid placement for actual anatomical landmarks and nozzle origins.
pub fn place_point(point: [f32; 3], rigid: RigidPose) -> Result<[i64; 3], MorphError> {
    require_book(
        yaw_ok(rigid.yaw_sin, rigid.yaw_cos),
        "rigid yaw is not unit",
    )?;
    translate(
        rotate_y(position_from_f32(point)?, rigid)?,
        rigid.translation,
    )
}

pub fn place_output(frame: &OutputFrame, rigid: RigidPose) -> Result<PlacedNozzle, MorphError> {
    require_book(
        yaw_ok(rigid.yaw_sin, rigid.yaw_cos),
        "rigid yaw is not unit",
    )?;
    let origin = place_point(frame.position, rigid)?;
    let direction = rotate_y(direction_from_f32(frame.direction)?, rigid)?;
    require_book(
        direction_ok(direction),
        "rotated direction left the unit shell",
    )?;
    Ok(PlacedNozzle {
        nozzle_id: frame.component_id,
        origin,
        direction,
        anatomical_operational: frame.metadata.operational,
    })
}

pub fn launch_velocity(direction: [i64; 3], speed: i64) -> [i64; 3] {
    direction.map(|component| ((component as i128) * i128::from(speed) / i128::from(SCALE)) as i64)
}

fn integrate(parcel: &Parcel, environment: &Environment) -> Result<Parcel, MorphError> {
    let couple =
        u32::from(environment.drag_per_mille) * u32::from(parcel.response_per_mille) / 1000;
    require_book(couple <= 1000, "drag coupling exceeded 1000")?;
    let mut velocity = [0; 3];
    let mut position = [0; 3];
    for axis in 0..3 {
        let summed = add_i128(
            parcel.velocity[axis],
            environment.acceleration[axis],
            "acceleration",
        )?;
        let blended = blend(summed, environment.medium_flow[axis], couple)?;
        velocity[axis] = blended;
        position[axis] = add_i64(
            i128::from(parcel.position[axis]),
            i128::from(blended),
            "position",
        )?;
    }
    let mut moved = *parcel;
    moved.velocity = velocity;
    moved.position = position;
    Ok(moved)
}

fn blend(velocity: i64, flow: i64, couple: u32) -> Result<i64, MorphError> {
    let couple = i128::from(couple);
    let blended = (i128::from(velocity) * (1000 - couple) + i128::from(flow) * couple) / 1000;
    i64::try_from(blended).map_err(|_| MorphError("blended velocity overflow".into()))
}

fn rotate_y(vector: [i64; 3], rigid: RigidPose) -> Result<[i64; 3], MorphError> {
    require_book(
        (0..6_283_185).contains(&rigid.axial_roll),
        "axial roll outside one turn",
    )?;
    let vector = if rigid.axial_roll == 0 {
        vector
    } else {
        let angle = rigid.axial_roll as f64 / 1_000_000.;
        let sin = (angle.sin() * SCALE as f64).round() as i128;
        let cos = (angle.cos() * SCALE as f64).round() as i128;
        let x = i128::from(vector[0]);
        let y = i128::from(vector[1]);
        [
            i64::try_from((x * cos - y * sin) / i128::from(SCALE))
                .map_err(|_| MorphError("roll overflow".into()))?,
            i64::try_from((x * sin + y * cos) / i128::from(SCALE))
                .map_err(|_| MorphError("roll overflow".into()))?,
            vector[2],
        ]
    };
    let x = i128::from(vector[0]);
    let z = i128::from(vector[2]);
    let sin = i128::from(rigid.yaw_sin);
    let cos = i128::from(rigid.yaw_cos);
    let scale = i128::from(SCALE);
    let rotated_x = (x * cos + z * sin) / scale;
    let rotated_z = (-x * sin + z * cos) / scale;
    Ok([
        i64::try_from(rotated_x).map_err(|_| MorphError("yaw overflow".into()))?,
        vector[1],
        i64::try_from(rotated_z).map_err(|_| MorphError("yaw overflow".into()))?,
    ])
}

fn translate(vector: [i64; 3], by: [i64; 3]) -> Result<[i64; 3], MorphError> {
    let mut out = [0; 3];
    for axis in 0..3 {
        out[axis] = add_i64(
            i128::from(vector[axis]),
            i128::from(by[axis]),
            "translation",
        )?;
    }
    Ok(out)
}

fn add_i128(left: i64, right: i64, label: &str) -> Result<i64, MorphError> {
    add_i64(i128::from(left), i128::from(right), label)
}

fn add_i64(left: i128, right: i128, label: &str) -> Result<i64, MorphError> {
    i64::try_from(left + right).map_err(|_| MorphError(format!("{label} overflow")))
}

fn direction_ok(direction: [i64; 3]) -> bool {
    let target = i128::from(SCALE) * i128::from(SCALE);
    let norm: i128 = direction
        .into_iter()
        .map(|component| i128::from(component) * i128::from(component))
        .sum();
    (norm - target).unsigned_abs() * 50 <= target.unsigned_abs()
}

fn yaw_ok(sin: i64, cos: i64) -> bool {
    let target = i128::from(SCALE) * i128::from(SCALE);
    let norm = i128::from(sin) * i128::from(sin) + i128::from(cos) * i128::from(cos);
    (norm - target).unsigned_abs() * 1000 <= target.unsigned_abs()
}

fn require_book(ok: bool, message: &str) -> Result<(), MorphError> {
    if ok {
        Ok(())
    } else {
        Err(MorphError(message.into()))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Morph003Report {
    pub slice: String,
    pub model: String,
    pub numerical_limits: String,
    pub nursery_paths_left_untouched: Vec<String>,
    pub experiment_a: AccountSnapshot,
    pub experiment_b: PersistenceSnapshot,
    pub experiment_c: DetachSnapshot,
    pub experiment_d: EnvironmentSnapshot,
    pub experiment_e: ExhaustSnapshot,
    pub experiment_f: ReproductionSnapshot,
    pub conservation_faults: FaultSnapshot,
    pub capacity: CapacitySnapshot,
    pub nozzle: NozzleSnapshot,
    pub model_check: ModelCheck,
    pub cost: CostSnapshot,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountSnapshot {
    pub initial: u64,
    pub reservoir: u64,
    pub parcels: usize,
    pub sinks: u64,
    pub residual: i128,
    pub tick: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistenceSnapshot {
    pub birth: [i64; 3],
    pub after_steps: [i64; 3],
    pub expected_after_steps: [i64; 3],
    pub nozzle_after_body_shift: [i64; 3],
    pub parcel_after_body_shift: [i64; 3],
    pub residual: i128,
    pub runtime_operational: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DetachSnapshot {
    pub first_birth: [i64; 3],
    pub first_after_motion: [i64; 3],
    pub nozzle_at_end: [i64; 3],
    pub separation: [i64; 3],
    pub residual: i128,
    pub parcels: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvironmentSnapshot {
    pub drag_free: [i64; 3],
    pub drag_full: [i64; 3],
    pub accelerated: [i64; 3],
    pub residuals: [i128; 3],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExhaustSnapshot {
    pub initial: u64,
    pub reservoir: u64,
    pub in_world: u64,
    pub parcels: usize,
    pub second_status: String,
    pub residual: i128,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReproductionSnapshot {
    pub hash: String,
    pub repeat_hash: String,
    pub reloaded_hash: String,
    pub equal: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FaultSnapshot {
    pub duplicate_residual: i128,
    pub deletion_residual: i128,
    pub honest_residual: i128,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapacitySnapshot {
    pub capacity: u64,
    pub parcels: usize,
    pub extra_status: String,
    pub reservoir: u64,
    pub residual: i128,
    pub full_bound_parcels: usize,
    pub full_bound_status: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NozzleSnapshot {
    pub anatomical_operational: bool,
    pub runtime_before: bool,
    pub runtime_after: bool,
    pub translated: bool,
    pub rotated: bool,
    pub bent: bool,
    pub reattached: bool,
    pub channel: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelCheck {
    pub drag_free_steps: i64,
    pub drag_full_stays: bool,
    pub accel_displacement: i64,
    pub expected_accel_displacement: i64,
    pub negative_blend: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CostSnapshot {
    pub parcel_bytes: usize,
    pub bound_bytes: usize,
    pub fill_steps_micros: u128,
    pub integrate_steps: u64,
}

pub fn evaluate_morph003() -> Result<Morph003Report, MorphError> {
    let anatomy = CompiledAnatomy::new(AnatomySpec::showcase(BENCHMARK_SEED))?;
    let nozzle = nozzle_id(anatomy.spec())?;
    let frame = output_frame(&anatomy, 0., 0., nozzle)?;
    require_book(
        !frame.metadata.operational,
        "showcase nozzle must stay anatomically non-operational",
    )?;

    let experiment_a = experiment_a(&frame, nozzle)?;
    let experiment_b = experiment_b(&frame, nozzle)?;
    let experiment_c = experiment_c(&frame, nozzle)?;
    let experiment_d = experiment_d(&frame, nozzle)?;
    let experiment_e = experiment_e(nozzle)?;
    let experiment_f = experiment_f(&frame, nozzle)?;
    let conservation_faults = fault_snapshot(nozzle)?;
    let capacity = capacity_snapshot(nozzle)?;
    let nozzle_snapshot = nozzle_snapshot(&anatomy, nozzle)?;
    let model_check = model_check()?;
    let cost = cost_snapshot()?;

    Ok(Morph003Report {
        slice: "MORPH-003 conserving free-space secretion".into(),
        model: "Integer quanta. Fixed-point position and velocity, scale 1000000. \
Each step adds explicit acceleration, blends velocity toward uniform flow by \
drag_per_mille * response_per_mille / 1000, then adds velocity to position. \
Division truncates toward zero. No gravity term is implied. Drag is not an atmosphere."
            .into(),
        numerical_limits: "Parcel motion is integer and matches on any platform that \
implements Rust i128 division toward zero. Nozzle frames are quantized from the \
f32 anatomical pose; that quantization is same-build deterministic. Cross-platform \
f32 identity of the pose is not claimed. The interactive preview drops stalled \
ticks above four per frame; the benchmark does not."
            .into(),
        nursery_paths_left_untouched: vec![
            "crates/moebius-core/src/neural/body.rs Jet".into(),
            "crates/moebius-core/src/world.rs World::advance_jets".into(),
            "crates/moebius-core/src/world.rs SECRETE_ACTION".into(),
            "crates/moebius-core/src/layers.rs FluidSample".into(),
            "crates/moebius-core/src/layers.rs TrailPoint".into(),
        ],
        experiment_a,
        experiment_b,
        experiment_c,
        experiment_d,
        experiment_e,
        experiment_f,
        conservation_faults,
        capacity,
        nozzle: nozzle_snapshot,
        model_check,
        cost,
    })
}

fn experiment_a(frame: &OutputFrame, nozzle: u64) -> Result<AccountSnapshot, MorphError> {
    let mut book = open_book(100, Environment::suspended())?;
    book.register_nozzle(nozzle, SECRETION_CHANNEL)?;
    book.emission_enabled = false;
    book.rate = 1;
    for _ in 0..20 {
        let _ = emit_placed(&mut book, frame, RigidPose::identity(), nozzle, 0)?;
        book.step()?;
    }
    Ok(AccountSnapshot {
        initial: book.initial,
        reservoir: book.reservoir,
        parcels: book.parcels.len(),
        sinks: book.sinks,
        residual: book.residual()?,
        tick: book.tick,
    })
}

fn experiment_b(frame: &OutputFrame, nozzle: u64) -> Result<PersistenceSnapshot, MorphError> {
    let mut book = open_book(100, Environment::suspended())?;
    book.register_nozzle(nozzle, SECRETION_CHANNEL)?;
    book.emission_enabled = true;
    let birth_status = emit_placed(&mut book, frame, RigidPose::identity(), nozzle, 50_000)?;
    require_book(
        birth_status.admitted(),
        "stationary emission was not admitted",
    )?;
    require_book(
        book.runtime_operational(nozzle),
        "runtime operational before admission record",
    )?;
    let birth = book.parcels[0].position;
    book.emission_enabled = false;
    for _ in 0..10 {
        book.step()?;
    }
    let after_steps = book.parcels[0].position;
    let expected = translate(birth, book.parcels[0].velocity.map(|value| value * 10))?;
    let shifted = RigidPose {
        translation: [10 * SCALE, 0, 0],
        ..RigidPose::identity()
    };
    let moved_nozzle = place_output(frame, shifted)?;
    Ok(PersistenceSnapshot {
        birth,
        after_steps,
        expected_after_steps: expected,
        nozzle_after_body_shift: moved_nozzle.origin,
        parcel_after_body_shift: book.parcels[0].position,
        residual: book.residual()?,
        runtime_operational: book.runtime_operational(nozzle),
    })
}

fn experiment_c(frame: &OutputFrame, nozzle: u64) -> Result<DetachSnapshot, MorphError> {
    let mut book = open_book(100, Environment::suspended())?;
    book.register_nozzle(nozzle, SECRETION_CHANNEL)?;
    book.emission_enabled = true;
    let trajectory = AuthoredTrajectory {
        velocity_per_tick: [SCALE, 0, 0],
        ..AuthoredTrajectory::stationary()
    };
    let mut first_birth = [0; 3];
    for index in 0..8 {
        let rigid = trajectory.at(book.tick)?;
        let _ = emit_placed(&mut book, frame, rigid, nozzle, 0)?;
        if index == 0 {
            first_birth = book.parcels[0].position;
        }
        book.step()?;
    }
    let first_after = book.parcels[0].position;
    let nozzle_at_end = place_output(frame, trajectory.at(book.tick)?)?.origin;
    Ok(DetachSnapshot {
        first_birth,
        first_after_motion: first_after,
        nozzle_at_end,
        separation: [
            nozzle_at_end[0] - first_after[0],
            nozzle_at_end[1] - first_after[1],
            nozzle_at_end[2] - first_after[2],
        ],
        residual: book.residual()?,
        parcels: book.parcels.len(),
    })
}

fn experiment_d(frame: &OutputFrame, nozzle: u64) -> Result<EnvironmentSnapshot, MorphError> {
    let mut residuals = [0; 3];
    let mut positions = [[0; 3]; 3];
    for (index, environment) in comparison_environments().into_iter().enumerate() {
        let mut book = open_book(80, environment)?;
        book.register_nozzle(nozzle, SECRETION_CHANNEL)?;
        book.emission_enabled = true;
        let _ = emit_placed(&mut book, frame, RigidPose::identity(), nozzle, 80_000)?;
        book.emission_enabled = false;
        for _ in 0..12 {
            book.step()?;
        }
        positions[index] = book.parcels[0].position;
        residuals[index] = book.residual()?;
    }
    Ok(EnvironmentSnapshot {
        drag_free: positions[0],
        drag_full: positions[1],
        accelerated: positions[2],
        residuals,
    })
}

fn comparison_environments() -> [Environment; 3] {
    let mut full_drag = Environment::suspended();
    full_drag.drag_per_mille = 1000;
    let mut accelerated = Environment::suspended();
    accelerated.acceleration = [4_000, 0, 0];
    [Environment::suspended(), full_drag, accelerated]
}

fn experiment_e(nozzle: u64) -> Result<ExhaustSnapshot, MorphError> {
    let mut book = open_book(5, Environment::suspended())?;
    book.register_nozzle(nozzle, SECRETION_CHANNEL)?;
    book.emission_enabled = true;
    let first = book.try_emit(cardinal_request(nozzle, 3, 0));
    book.step()?;
    let second = book.try_emit(cardinal_request(nozzle, 3, 0));
    require_book(first.admitted(), "first exhaustion emit should admit")?;
    Ok(ExhaustSnapshot {
        initial: book.initial,
        reservoir: book.reservoir,
        in_world: book.in_world()?,
        parcels: book.parcels.len(),
        second_status: format!("{second:?}"),
        residual: book.residual()?,
    })
}

fn experiment_f(frame: &OutputFrame, nozzle: u64) -> Result<ReproductionSnapshot, MorphError> {
    let first = scripted(frame, nozzle)?;
    let second = scripted(frame, nozzle)?;
    let reloaded = MaterialBook::from_json(&first.to_json()?)?;
    let hash = first.state_hash()?;
    let repeat_hash = second.state_hash()?;
    let reloaded_hash = reloaded.state_hash()?;
    Ok(ReproductionSnapshot {
        hash: hash.clone(),
        repeat_hash,
        reloaded_hash,
        equal: first == second && first == reloaded,
    })
}

fn scripted(frame: &OutputFrame, nozzle: u64) -> Result<MaterialBook, MorphError> {
    let mut environment = Environment::suspended();
    environment.drag_per_mille = 250;
    environment.acceleration = [1_000, 0, 0];
    environment.medium_flow = [0, 0, 2_000];
    let mut book = open_book(40, environment)?;
    book.seed = BENCHMARK_SEED;
    book.register_nozzle(nozzle, SECRETION_CHANNEL)?;
    book.emission_enabled = true;
    book.rate = 2;
    let trajectory = AuthoredTrajectory {
        velocity_per_tick: [25_000, -4_000, 6_000],
        ..AuthoredTrajectory::stationary()
    };
    for _ in 0..6 {
        let rigid = trajectory.at(book.tick)?;
        let _ = emit_placed(&mut book, frame, rigid, nozzle, 30_000)?;
        book.step()?;
    }
    book.emission_enabled = false;
    for _ in 0..4 {
        book.step()?;
    }
    Ok(book)
}

fn fault_snapshot(nozzle: u64) -> Result<FaultSnapshot, MorphError> {
    let mut book = open_book(10, Environment::suspended())?;
    book.register_nozzle(nozzle, SECRETION_CHANNEL)?;
    book.emission_enabled = true;
    let _ = book.try_emit(cardinal_request(nozzle, 4, 1_000));
    let honest_residual = book.residual()?;
    let mut duplicated = book.clone();
    require_book(
        duplicated.fault_duplicate_last(),
        "fault duplicate found no parcel",
    )?;
    let mut deleted = book.clone();
    require_book(deleted.fault_drop_last(), "fault delete found no parcel")?;
    Ok(FaultSnapshot {
        duplicate_residual: duplicated.residual()?,
        deletion_residual: deleted.residual()?,
        honest_residual,
    })
}

fn capacity_snapshot(nozzle: u64) -> Result<CapacitySnapshot, MorphError> {
    let mut book = open_book(8, Environment::suspended())?;
    book.capacity = 2;
    book.register_nozzle(nozzle, SECRETION_CHANNEL)?;
    book.emission_enabled = true;
    let _ = book.try_emit(cardinal_request(nozzle, 1, 0));
    book.step()?;
    let _ = book.try_emit(cardinal_request(nozzle, 1, 0));
    book.step()?;
    let extra = book.try_emit(cardinal_request(nozzle, 1, 0));
    let small = CapacitySnapshot {
        capacity: book.capacity,
        parcels: book.parcels.len(),
        extra_status: format!("{extra:?}"),
        reservoir: book.reservoir,
        residual: book.residual()?,
        full_bound_parcels: 0,
        full_bound_status: String::new(),
    };
    let mut full = open_book(MAX_PARCELS as u64 + 2, Environment::suspended())?;
    full.capacity = MAX_PARCELS as u64;
    full.register_nozzle(nozzle, SECRETION_CHANNEL)?;
    full.emission_enabled = true;
    for _ in 0..MAX_PARCELS {
        let status = full.try_emit(cardinal_request(nozzle, 1, 0));
        require_book(status.admitted(), "bound fill rejected early")?;
        full.step()?;
    }
    let overflow = full.try_emit(cardinal_request(nozzle, 1, 0));
    Ok(CapacitySnapshot {
        full_bound_parcels: full.parcels.len(),
        full_bound_status: format!("{overflow:?}"),
        ..small
    })
}

fn nozzle_snapshot(anatomy: &CompiledAnatomy, nozzle: u64) -> Result<NozzleSnapshot, MorphError> {
    let spec = anatomy.spec().clone();
    let anatomical_operational = !anatomical_flag_is_false(&spec, nozzle)?;
    require_book(!anatomical_operational, "anatomical flag flipped")?;
    let rest = output_frame(anatomy, 0., 0., nozzle)?;
    let bent = output_frame(anatomy, 1., 0., nozzle)?;
    let mut moved = spec.clone();
    for component in &mut moved.components {
        if component.id == nozzle {
            component.attachment.station = 0.22;
        }
    }
    let rebound = CompiledAnatomy::new(moved)?;
    let attached = output_frame(&rebound, 0., 0., nozzle)?;
    let placed = place_output(&rest, RigidPose::identity())?;
    let translated = place_output(
        &rest,
        RigidPose {
            translation: [4 * SCALE, -SCALE, 2 * SCALE],
            ..RigidPose::identity()
        },
    )?;
    let quarter = RigidPose {
        yaw_sin: SCALE,
        yaw_cos: 0,
        ..RigidPose::identity()
    };
    let rotated = place_output(&rest, quarter)?;
    let synthetic = OutputFrame {
        component_id: nozzle,
        position: [1., 0., 0.],
        direction: [1., 0., 0.],
        metadata: EmissionMetadata {
            material_channel: SECRETION_CHANNEL.into(),
            operational: false,
        },
    };
    let synthetic_rotated = place_output(&synthetic, quarter)?;
    let mut book = open_book(3, Environment::suspended())?;
    book.register_nozzle(nozzle, SECRETION_CHANNEL)?;
    let before = book.runtime_operational(nozzle);
    book.emission_enabled = false;
    let _ = emit_placed(&mut book, &rest, RigidPose::identity(), nozzle, 0)?;
    require_book(
        !book.runtime_operational(nozzle),
        "disabled emit became operational",
    )?;
    book.emission_enabled = true;
    book.reservoir = 0;
    let _ = emit_placed(&mut book, &rest, RigidPose::identity(), nozzle, 0)?;
    require_book(
        !book.runtime_operational(nozzle),
        "rejected emit became operational",
    )?;
    book.reservoir = 3;
    let _ = emit_placed(&mut book, &rest, RigidPose::identity(), nozzle, 0)?;
    Ok(NozzleSnapshot {
        anatomical_operational,
        runtime_before: before,
        runtime_after: book.runtime_operational(nozzle),
        translated: translated.origin[0] == placed.origin[0] + 4 * SCALE
            && translated.origin[1] == placed.origin[1] - SCALE
            && translated.origin[2] == placed.origin[2] + 2 * SCALE
            && translated.direction == placed.direction,
        rotated: synthetic_rotated.direction == [0, 0, -SCALE]
            && synthetic_rotated.origin == [0, 0, -SCALE]
            && rotated.origin == rotate_y(placed.origin, quarter)?,
        bent: position_from_f32(rest.position)? != position_from_f32(bent.position)?,
        reattached: position_from_f32(rest.position)? != position_from_f32(attached.position)?,
        channel: rest.metadata.material_channel,
    })
}

fn model_check() -> Result<ModelCheck, MorphError> {
    let mut free = Environment::suspended();
    free.drag_per_mille = 0;
    let mut killed = Environment::suspended();
    killed.drag_per_mille = 1000;
    let mut accelerated = Environment::suspended();
    accelerated.acceleration = [3_000, 0, 0];
    let free_end = fly(free, 20_000, 0, 6)?;
    let killed_end = fly(killed, 20_000, 0, 6)?;
    let accel_end = fly(accelerated, 0, 3_000, 10)?;
    let mut negative = open_book(1, Environment::suspended())?;
    negative.environment.drag_per_mille = 500;
    negative.register_nozzle(1, SECRETION_CHANNEL)?;
    negative.emission_enabled = true;
    let _ = negative.try_emit(EmitRequest {
        nozzle_id: 1,
        origin: [0, 0, 0],
        direction: [0, 0, -SCALE],
        quantity: 1,
        launch_speed: 5,
        channel: SECRETION_CHANNEL,
    });
    negative.step()?;
    let steps = 10i64;
    let expected = 3_000 * steps * (steps + 1) / 2;
    Ok(ModelCheck {
        drag_free_steps: free_end[2],
        drag_full_stays: killed_end == [0, 0, 0],
        accel_displacement: accel_end[0],
        expected_accel_displacement: expected,
        negative_blend: negative.parcels[0].velocity[2],
    })
}

fn fly(
    environment: Environment,
    launch: i64,
    _accel_marker: i64,
    steps: u64,
) -> Result<[i64; 3], MorphError> {
    let mut book = open_book(4, environment)?;
    book.register_nozzle(1, SECRETION_CHANNEL)?;
    book.emission_enabled = true;
    let _ = book.try_emit(cardinal_request(1, 1, launch));
    book.emission_enabled = false;
    for _ in 0..steps {
        book.step()?;
    }
    Ok(book.parcels[0].position)
}

fn cost_snapshot() -> Result<CostSnapshot, MorphError> {
    let started = std::time::Instant::now();
    let mut book = open_book(MAX_PARCELS as u64, Environment::suspended())?;
    book.capacity = MAX_PARCELS as u64;
    book.register_nozzle(1, SECRETION_CHANNEL)?;
    book.emission_enabled = true;
    for _ in 0..MAX_PARCELS {
        let status = book.try_emit(cardinal_request(1, 1, 0));
        require_book(status.admitted(), "cost fill rejected")?;
        book.step()?;
    }
    for _ in 0..100 {
        book.step()?;
    }
    require_book(book.conservation_holds(), "cost run broke conservation")?;
    let parcel_bytes = std::mem::size_of::<Parcel>();
    Ok(CostSnapshot {
        parcel_bytes,
        bound_bytes: parcel_bytes * MAX_PARCELS,
        fill_steps_micros: started.elapsed().as_micros(),
        integrate_steps: 100,
    })
}

fn open_book(initial: u64, environment: Environment) -> Result<MaterialBook, MorphError> {
    MaterialBook::new(
        BENCHMARK_SEED,
        BENCHMARK_SEED,
        initial,
        environment,
        MAX_PARCELS,
    )
}

fn emit_placed(
    book: &mut MaterialBook,
    frame: &OutputFrame,
    rigid: RigidPose,
    nozzle: u64,
    launch: i64,
) -> Result<EmissionStatus, MorphError> {
    let placed = place_output(frame, rigid)?;
    require_book(
        !placed.anatomical_operational,
        "frame claimed anatomical operation",
    )?;
    Ok(book.try_emit(EmitRequest {
        nozzle_id: nozzle,
        origin: placed.origin,
        direction: placed.direction,
        quantity: book.rate,
        launch_speed: launch,
        channel: SECRETION_CHANNEL,
    }))
}

fn cardinal_request(nozzle: u64, quantity: u64, launch: i64) -> EmitRequest<'static> {
    EmitRequest {
        nozzle_id: nozzle,
        origin: [0, 0, 0],
        direction: [0, 0, SCALE],
        quantity,
        launch_speed: launch,
        channel: SECRETION_CHANNEL,
    }
}

fn output_frame(
    anatomy: &CompiledAnatomy,
    bend: f32,
    phase: f32,
    nozzle: u64,
) -> Result<OutputFrame, MorphError> {
    let mut pose = Pose::rest(&anatomy.spec().body);
    pose.bend = bend;
    pose.phase = phase;
    pose.flagella = 0.;
    let posed = anatomy.pose(pose)?;
    posed
        .outputs
        .into_iter()
        .find(|frame| frame.component_id == nozzle)
        .ok_or_else(|| MorphError("nozzle frame missing".into()))
}

fn nozzle_id(spec: &AnatomySpec) -> Result<u64, MorphError> {
    spec.components
        .iter()
        .find_map(|component| match component.shape {
            Shape::SecretionNozzle { .. } => Some(component.id),
            _ => None,
        })
        .ok_or_else(|| MorphError("anatomy has no secretion nozzle".into()))
}

fn anatomical_flag_is_false(spec: &AnatomySpec, nozzle: u64) -> Result<bool, MorphError> {
    for component in &spec.components {
        if component.id == nozzle
            && let Shape::SecretionNozzle { output, .. } = &component.shape
        {
            return Ok(!output.operational);
        }
    }
    Err(MorphError(
        "nozzle missing while reading anatomical flag".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quarter_turn_about_y_sends_positive_x_to_negative_z() {
        let rigid = RigidPose {
            yaw_sin: SCALE,
            yaw_cos: 0,
            ..RigidPose::identity()
        };
        let turned = rotate_y([SCALE, 2, 0], rigid).unwrap();
        assert_eq!(turned, [0, 2, -SCALE]);
    }

    #[test]
    fn blend_truncates_toward_zero() {
        assert_eq!(blend(-5, 0, 500).unwrap(), -2);
        assert_eq!(blend(1, 0, 500).unwrap(), 0);
        assert_eq!(blend(80, 10, 1000).unwrap(), 10);
    }
}
