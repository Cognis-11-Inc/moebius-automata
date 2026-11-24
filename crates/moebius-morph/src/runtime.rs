//! Canonical ship runtime, authorized MORPH-004 option2. Nursery core remains separate.
//! Geometry instances prescribe drift; no physiology or propulsion is implied.
use crate::{
    emission::{
        EmitRequest, Environment, MAX_LAUNCH, MaterialBook, PlacedNozzle, RigidPose, SCALE,
        SECRETION_CHANNEL,
    },
    *,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
pub const REGION: i64 = 1_000 * SCALE;
pub const MAX_POSITION: i64 = 1_000_000_000 * SCALE;
pub const TICK_NANOS: u64 = 16_666_667;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Position {
    pub region: [i64; 3],
    pub local: [i64; 3],
}
impl Position {
    pub fn from_absolute(p: [i64; 3]) -> Result<Self, MorphError> {
        require(
            p.iter().all(|v| v.unsigned_abs() <= MAX_POSITION as u64),
            "position outside supported metric envelope",
        )?;
        Ok(Self {
            region: p.map(|v| v.div_euclid(REGION)),
            local: p.map(|v| v.rem_euclid(REGION)),
        })
    }
    pub fn absolute(self) -> Result<[i64; 3], MorphError> {
        let mut a = [0; 3];
        for (i, v) in a.iter_mut().enumerate() {
            require(
                (0..REGION).contains(&self.local[i]),
                "unnormalized local coordinate",
            )?;
            let x = i128::from(self.region[i]) * i128::from(REGION) + i128::from(self.local[i]);
            require(
                x.abs() <= i128::from(MAX_POSITION),
                "region outside supported metric envelope",
            )?;
            *v = x as i64;
        }
        Ok(a)
    }
    pub fn offset(self, d: [i64; 3]) -> Result<Self, MorphError> {
        let mut p = self.absolute()?;
        for i in 0..3 {
            p[i] = p[i]
                .checked_add(d[i])
                .ok_or_else(|| MorphError("position overflow".into()))?;
        }
        Self::from_absolute(p)
    }
    pub fn relative(self, origin: Self) -> Result<[i64; 3], MorphError> {
        let a = self.absolute()?;
        let b = origin.absolute()?;
        Ok(std::array::from_fn(|i| a[i] - b[i]))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LightPort {
    pub id: u64,
    pub position: Position,
    pub intensity: u64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldConfig {
    pub version: u32,
    pub seed: u64,
    pub tick_nanos: u64,
    pub environment: Environment,
    pub ambient_light: u64,
    pub ports: Vec<LightPort>,
    pub definitions: BTreeMap<u64, AnatomySpec>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Instance {
    pub id: u64,
    pub definition: u64,
    pub position: Position,
    pub velocity: [i64; 3],
    pub exposure: u64,
    pub book: MaterialBook,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operation", deny_unknown_fields)]
pub enum Operation {
    SetEnvironment { environment: Environment },
    SetAmbientLight { value: u64 },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Intervention {
    pub tick: u64,
    pub sequence: u64,
    pub origin: String,
    pub action: Operation,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "event", deny_unknown_fields)]
pub enum Event {
    Accepted {
        at: u64,
        command: Intervention,
    },
    Applied {
        at: u64,
        sequence: u64,
    },
    Emitted {
        at: u64,
        organism: u64,
        nozzle: u64,
        parcel: u64,
        quantity: u64,
    },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldSnapshot {
    pub version: u32,
    pub tick: u64,
    pub config: WorldConfig,
    pub instances: BTreeMap<u64, Instance>,
    pub pending: Vec<Intervention>,
    pub events: Vec<Event>,
}
/// No mutable state reference is exposed to observers or renderers.
pub struct ShipRuntime {
    state: WorldSnapshot,
    outputs: BTreeMap<u64, Vec<PlacedNozzle>>,
}
fn exposure(c: &WorldConfig, p: Position) -> Result<u64, MorphError> {
    let mut total = c.ambient_light;
    for port in &c.ports {
        let d = p.relative(port.position)?;
        let metres2 = d
            .iter()
            .map(|&v| i128::from(v) * i128::from(v))
            .sum::<i128>()
            / (i128::from(SCALE) * i128::from(SCALE));
        let value = i128::from(port.intensity) / (1 + metres2);
        total = total
            .checked_add(value as u64)
            .ok_or_else(|| MorphError("illumination overflow".into()))?;
    }
    Ok(total)
}
fn outputs(c: &WorldConfig) -> Result<BTreeMap<u64, Vec<PlacedNozzle>>, MorphError> {
    require(
        c.version == 1 && c.tick_nanos == TICK_NANOS,
        "unsupported ship configuration/timestep",
    )?;
    c.environment.validate()?;
    require(
        !c.definitions.is_empty() && c.definitions.len() <= 64,
        "definition budget",
    )?;
    let mut ports = BTreeSet::new();
    let mut light = c.ambient_light;
    for p in &c.ports {
        require(ports.insert(p.id), "duplicate light port")?;
        p.position.absolute()?;
        light = light
            .checked_add(p.intensity)
            .ok_or_else(|| MorphError("illumination budget overflow".into()))?;
    }
    c.definitions
        .iter()
        .map(|(&id, s)| {
            let g = CompiledAnatomy::new(s.clone())?.pose(Pose::rest(&s.body))?;
            let out = g
                .outputs
                .iter()
                .map(|f| crate::emission::place_output(f, RigidPose::identity()))
                .collect::<Result<Vec<_>, _>>()?;
            Ok((id, out))
        })
        .collect()
}
impl ShipRuntime {
    pub fn new(config: WorldConfig, instances: Vec<Instance>) -> Result<Self, MorphError> {
        let mut map = BTreeMap::new();
        for i in instances {
            require(map.insert(i.id, i).is_none(), "duplicate organism identity")?;
        }
        Self::restore(WorldSnapshot {
            version: 1,
            tick: 0,
            config,
            instances: map,
            pending: vec![],
            events: vec![],
        })
    }
    pub fn restore(state: WorldSnapshot) -> Result<Self, MorphError> {
        let out = outputs(&state.config)?;
        let result = Self {
            state,
            outputs: out,
        };
        result.validate()?;
        Ok(result)
    }
    pub fn snapshot(&self) -> WorldSnapshot {
        self.state.clone()
    }
    pub fn state(&self) -> &WorldSnapshot {
        &self.state
    }
    pub fn digest(&self) -> Result<String, MorphError> {
        self.state.digest()
    }
    pub fn submit(&mut self, command: Intervention) -> Result<(), MorphError> {
        validate_command(&command, self.state.tick)?;
        if let Operation::SetAmbientLight { value } = command.action {
            self.state
                .config
                .ports
                .iter()
                .try_fold(value, |a, p| a.checked_add(p.intensity))
                .ok_or_else(|| MorphError("illumination budget overflow".into()))?;
        }
        require(
            !self
                .state
                .events
                .iter()
                .any(|e| matches!(e,Event::Accepted{command:c,..} if c.sequence==command.sequence)),
            "duplicate intervention sequence",
        )?;
        self.state.events.push(Event::Accepted {
            at: self.state.tick,
            command: command.clone(),
        });
        self.state.pending.push(command);
        self.state.pending.sort_by_key(|c| (c.tick, c.sequence));
        Ok(())
    }
    pub fn tick(&mut self) -> Result<(), MorphError> {
        let mut next = self.state.clone();
        let tick = next.tick;
        let count = next.pending.partition_point(|c| c.tick == tick);
        for c in next.pending.drain(..count) {
            match c.action {
                Operation::SetEnvironment { environment } => {
                    next.config.environment = environment;
                    for i in next.instances.values_mut() {
                        i.book.environment = environment;
                    }
                }
                Operation::SetAmbientLight { value } => next.config.ambient_light = value,
            }
            next.events.push(Event::Applied {
                at: tick,
                sequence: c.sequence,
            });
        }
        for i in next.instances.values_mut() {
            require(i.book.tick == tick, "material substate timeline mismatch")?;
            let origin = i.position.absolute()?;
            for nozzle in &self.outputs[&i.definition] {
                let mut at = origin;
                for (a, b) in at.iter_mut().zip(nozzle.origin) {
                    *a = a
                        .checked_add(b)
                        .ok_or_else(|| MorphError("nozzle position overflow".into()))?;
                }
                let status = i.book.try_emit(EmitRequest {
                    nozzle_id: nozzle.nozzle_id,
                    origin: at,
                    direction: nozzle.direction,
                    quantity: i.book.rate,
                    launch_speed: i.book.launch_speed,
                    channel: SECRETION_CHANNEL,
                });
                if let crate::emission::EmissionStatus::Admitted {
                    parcel_id,
                    quantity,
                } = status
                {
                    next.events.push(Event::Emitted {
                        at: tick,
                        organism: i.id,
                        nozzle: nozzle.nozzle_id,
                        parcel: parcel_id,
                        quantity,
                    });
                }
            }
            i.book.step()?;
            i.position = i.position.offset(i.velocity)?;
            i.exposure = exposure(&next.config, i.position)?;
            require(i.book.residual()? == 0, "ship material balance failed")?;
            for parcel in &i.book.parcels {
                Position::from_absolute(parcel.position)?;
            }
        }
        next.tick = tick
            .checked_add(1)
            .ok_or_else(|| MorphError("world tick overflow".into()))?;
        self.state = next;
        Ok(())
    }
    pub fn advance(&mut self, ticks: u64) -> Result<(), MorphError> {
        for _ in 0..ticks {
            self.tick()?;
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<(), MorphError> {
        let s = &self.state;
        require(
            s.version == 1 && s.instances.len() <= 100_000,
            "unsupported state or population budget",
        )?;
        for (&id, i) in &s.instances {
            require(
                id == i.id && id != 0 && self.outputs.contains_key(&i.definition),
                "invalid instance identity/definition",
            )?;
            i.position.absolute()?;
            require(
                i.velocity
                    .iter()
                    .all(|v| v.unsigned_abs() <= MAX_LAUNCH as u64),
                "drift outside envelope",
            )?;
            require(
                i.book.organism_id == id
                    && i.book.tick == s.tick
                    && i.book.environment == s.config.environment,
                "book ownership/timeline/environment mismatch",
            )?;
            i.book.to_json()?;
            for parcel in &i.book.parcels {
                Position::from_absolute(parcel.position)?;
            }
            let expected = self.outputs[&i.definition]
                .iter()
                .map(|f| f.nozzle_id)
                .collect::<BTreeSet<_>>();
            require(
                i.book.registered == expected,
                "nozzle registration mismatch",
            )?;
            require(
                i.exposure == exposure(&s.config, i.position)?,
                "exposure mismatch",
            )?;
        }
        let mut accepted = BTreeMap::new();
        let mut applied = BTreeSet::new();
        let mut births = BTreeSet::new();
        for e in &s.events {
            match e {
                Event::Accepted { at, command } => {
                    validate_command(command, *at)?;
                    require(
                        *at <= s.tick && accepted.insert(command.sequence, command).is_none(),
                        "invalid admission event",
                    )?;
                }
                Event::Applied { at, sequence } => {
                    let c = accepted
                        .get(sequence)
                        .ok_or_else(|| MorphError("application without admission".into()))?;
                    require(
                        *at == c.tick && *at < s.tick && applied.insert(*sequence),
                        "invalid application event",
                    )?;
                }
                Event::Emitted {
                    at,
                    organism,
                    nozzle,
                    parcel,
                    quantity,
                } => {
                    let i = s
                        .instances
                        .get(organism)
                        .ok_or_else(|| MorphError("event organism missing".into()))?;
                    require(
                        births.insert((*organism, *parcel))
                            && *at < s.tick
                            && *quantity > 0
                            && i.book.parcels.iter().any(|p| {
                                p.id == *parcel
                                    && p.nozzle_id == *nozzle
                                    && p.quantity == *quantity
                                    && p.created_tick == *at
                            }),
                        "invalid emission event",
                    )?;
                }
            }
        }
        require(
            births.len()
                == s.instances
                    .values()
                    .map(|i| i.book.parcels.len())
                    .sum::<usize>(),
            "material birth history mismatch",
        )?;
        let mut pending = BTreeSet::new();
        let mut previous = None;
        for c in &s.pending {
            validate_command(c, s.tick)?;
            let key = (c.tick, c.sequence);
            require(
                previous.is_none_or(|p| p < key)
                    && pending.insert(c.sequence)
                    && accepted.get(&c.sequence) == Some(&c)
                    && !applied.contains(&c.sequence),
                "invalid pending intervention",
            )?;
            previous = Some(key);
        }
        require(
            accepted.len() == pending.len() + applied.len(),
            "lost intervention history",
        )?;
        Ok(())
    }
}
fn validate_command(c: &Intervention, tick: u64) -> Result<(), MorphError> {
    require(
        c.tick >= tick && !c.origin.trim().is_empty() && c.origin.len() <= 128,
        "invalid intervention tick/origin",
    )?;
    match c.action {
        Operation::SetEnvironment { environment } => environment.validate(),
        Operation::SetAmbientLight { value } => {
            require(value <= 1_000_000_000, "ambient light budget")
        }
    }
}
impl WorldSnapshot {
    pub fn digest(&self) -> Result<String, MorphError> {
        Ok(
            blake3::hash(&serde_json::to_vec(self).map_err(|e| MorphError(e.to_string()))?)
                .to_hex()
                .to_string(),
        )
    }
    pub fn save(&self, path: &Path) -> Result<(), MorphError> {
        ShipRuntime::restore(self.clone())?;
        crate::write_atomic(
            path,
            &serde_json::to_string_pretty(self).map_err(|e| MorphError(e.to_string()))?,
        )
    }
    pub fn load(path: &Path) -> Result<Self, MorphError> {
        let data = std::fs::read_to_string(path).map_err(|e| MorphError(e.to_string()))?;
        let s: Self = serde_json::from_str(&data).map_err(|e| MorphError(e.to_string()))?;
        ShipRuntime::restore(s.clone())?;
        Ok(s)
    }
}
/// Reproducible supported reference: prescribed drifting instances with actual conserved material.
pub fn fixture(population: usize, seed: u64) -> Result<ShipRuntime, MorphError> {
    require(
        (1..=100_000).contains(&population),
        "fixture population budget",
    )?;
    let spec = AnatomySpec::showcase(seed);
    let mut definitions = BTreeMap::new();
    definitions.insert(1, spec.clone());
    let config = WorldConfig {
        version: 1,
        seed,
        tick_nanos: TICK_NANOS,
        environment: Environment::suspended(),
        ambient_light: 10,
        ports: vec![LightPort {
            id: 1,
            position: Position::from_absolute([0, 20 * SCALE, 0])?,
            intensity: 100_000,
        }],
        definitions,
    };
    let mut instances = vec![];
    for n in 0..population {
        let id = n as u64 + 1;
        let position =
            Position::from_absolute([n as i64 % 100 * 4 * SCALE, 0, n as i64 / 100 * 4 * SCALE])?;
        let mut book = MaterialBook::new(id, seed, 64, config.environment, 64)?;
        book.sync_nozzles(&spec);
        book.emission_enabled = true;
        book.launch_speed = 30_000;
        instances.push(Instance {
            id,
            definition: 1,
            position,
            velocity: [1000, 0, 0],
            exposure: exposure(&config, position)?,
            book,
        });
    }
    ShipRuntime::new(config, instances)
}
