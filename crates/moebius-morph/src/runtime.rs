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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub construction: Option<crate::filament::ConstructionRules>,
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
    SetEmission { organism: u64, enabled: bool },
    Assemble { organism: u64, parcel: u64 },
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
    Contact {
        observation: crate::collision::ContactObservation,
    },
    Reproduced {
        at: u64,
        parent: u64,
        child: u64,
        quantity: u64,
    },
    Fed {
        at: u64,
        organism: u64,
        source: u64,
        quantity: u64,
        mouth: Position,
    },
    Deposited {
        at: u64,
        organism: u64,
        parcel: u64,
        filament: u64,
        node: u64,
        quantity: u64,
    },
    AssemblyRejected {
        at: u64,
        organism: u64,
        parcel: Option<u64>,
        reason: String,
    },
    EmissionRejected {
        at: u64,
        organism: u64,
        reason: String,
    },
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
    Chewed {
        at: u64,
        organism: u64,
        filament: u64,
        node: u64,
        parcel: u64,
        quantity: u64,
    },
    FoodBurst {
        at: u64,
        site: u64,
        seeds: u64,
    },
}
/// Ship-absorbed biomass. Light does not increase it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Condensate {
    pub initial: u64,
    pub biomass: u64,
    /// Non-sac book initials at the moment the stock opened.
    pub founders: u64,
}
/// Explicit, replayable state for the authored refill/build cycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConstructorState {
    pub rigid: RigidPose,
    pub seeking_food: bool,
}
/// Definition id of the blood-producing body.
pub const BLOOD_SAC: u64 = 2;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldSnapshot {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub solids: Vec<crate::collision::SolidBox>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub habitat: Option<crate::habitat::Habitat>,
    /// Opt-in kinematic constructor pose; absent in historical authored fixtures.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub steering: Option<ConstructorState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assembly: Option<crate::filament::Assembly>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub food: Option<crate::food::FoodField>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condensate: Option<Condensate>,
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
    animator: Option<CompiledAnatomy>,
    collision_cache: crate::collision::CollisionCache,
}
pub(crate) fn exposure(c: &WorldConfig, p: Position) -> Result<u64, MorphError> {
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
        (c.version == 1 || c.version == 2)
            && (c.version == 2) == c.construction.is_some()
            && c.tick_nanos == TICK_NANOS,
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
fn initial_snapshot(
    config: WorldConfig,
    instances: Vec<Instance>,
) -> Result<WorldSnapshot, MorphError> {
    let mut map = BTreeMap::new();
    for i in instances {
        require(map.insert(i.id, i).is_none(), "duplicate organism identity")?;
    }
    Ok(WorldSnapshot {
        solids: vec![],
        habitat: None,
        steering: None,
        assembly: config
            .construction
            .as_ref()
            .map(|_| crate::filament::Assembly::default()),
        food: None,
        condensate: None,
        version: if config.construction.is_some() { 2 } else { 1 },
        tick: 0,
        config,
        instances: map,
        pending: vec![],
        events: vec![],
    })
}
/// Bound work only after local capture eligibility. Old paid parcels remain in
/// the world; they must not starve newer, reachable material from the queue.
fn automatic_deposits(
    book: &MaterialBook,
    r: &crate::filament::ConstructionRules,
    nozzles: &BTreeMap<u64, Position>,
    local_capture: bool,
) -> Result<Vec<u64>, MorphError> {
    let mut per_nozzle = BTreeMap::<u64, usize>::new();
    let mut eligible = Vec::new();
    for p in &book.parcels {
        if book.tick.saturating_sub(p.created_tick) < r.age {
            continue;
        }
        if local_capture {
            let nozzle = nozzles
                .get(&p.nozzle_id)
                .ok_or_else(|| MorphError("construction nozzle missing".into()))?;
            let delta = Position::from_absolute(p.position)?.relative(*nozzle)?;
            if delta.iter().map(|x| i128::from(*x).pow(2)).sum::<i128>()
                > i128::from(r.capture_radius).pow(2)
            {
                continue;
            }
        }
        if r.parallel_nozzles() {
            let count = per_nozzle.entry(p.nozzle_id).or_default();
            if *count >= 32 {
                continue;
            }
            *count += 1;
        }
        eligible.push(p.id);
        if eligible.len() >= if r.parallel_nozzles() { 4096 } else { 32 } {
            break;
        }
    }
    Ok(eligible)
}
fn unit_heading(yaw: f64) -> [i64; 3] {
    let raw = [
        (yaw.sin() * SCALE as f64).round() as i64,
        0,
        (yaw.cos() * SCALE as f64).round() as i64,
    ];
    let length = raw
        .iter()
        .map(|x| (*x as f64).powi(2))
        .sum::<f64>()
        .sqrt()
        .max(1.);
    raw.map(|x| (x as f64 * SCALE as f64 / length).round() as i64)
}
fn template_pose(state: &WorldSnapshot) -> crate::emission::RigidPose {
    state
        .habitat
        .as_ref()
        .and_then(|habitat| habitat.agents.get(&1))
        .map(|agent| agent.rigid)
        .unwrap_or_else(crate::emission::RigidPose::identity)
}
fn sow_blood_garden_state(state: &mut WorldSnapshot) -> Result<(), MorphError> {
    require(state.condensate.is_none(), "condensate already open")?;
    require(
        !state.config.definitions.contains_key(&BLOOD_SAC),
        "blood sac already defined",
    )?;
    let spec = AnatomySpec::blood_sac(state.config.seed.wrapping_add(19));
    state.config.definitions.insert(BLOOD_SAC, spec);
    state.config.ambient_light = 0;
    state.config.ports.push(LightPort {
        id: 7,
        position: Position::from_absolute([SCALE / 2, 0, 0])?,
        intensity: 8_000,
    });
    state.config.ports.push(LightPort {
        id: 8,
        position: Position::from_absolute([0, 0, 40 * SCALE])?,
        intensity: 8_000,
    });
    let founders = state
        .instances
        .values()
        .map(|organism| organism.book.initial)
        .sum();
    state.condensate = Some(Condensate {
        initial: 4_000,
        biomass: 4_000,
        founders,
    });
    let placed: Vec<(u64, Position)> = state
        .instances
        .iter()
        .map(|(id, organism)| (*id, organism.position))
        .collect();
    for (id, position) in placed {
        let lit = exposure(&state.config, position)?;
        state.instances.get_mut(&id).unwrap().exposure = lit;
    }
    let clock = state.tick;
    for (at, quantity) in [
        ([SCALE, 0, 0], 6u64),
        ([8_000 * SCALE, 0, 0], 4),
        ([0, 0, 40 * SCALE], 4),
        ([6 * SCALE, 0, SCALE / 5], 22),
        ([6 * SCALE, SCALE / 3, 0], 22),
        ([6 * SCALE + SCALE, 0, -(SCALE / 4)], 22),
    ] {
        pay_sac(state, Position::from_absolute(at)?, quantity, clock)?;
    }
    Ok(())
}

impl ShipRuntime {
    pub fn new(config: WorldConfig, instances: Vec<Instance>) -> Result<Self, MorphError> {
        Self::restore(initial_snapshot(config, instances)?)
    }
    pub fn restore(state: WorldSnapshot) -> Result<Self, MorphError> {
        let out = outputs(&state.config)?;
        let animator = if let Some(r) = &state.config.construction {
            r.validate()?;
            let i = state
                .instances
                .get(&r.organism)
                .ok_or_else(|| MorphError("construction source missing".into()))?;
            Some(CompiledAnatomy::new(
                state.config.definitions[&i.definition].clone(),
            )?)
        } else {
            None
        };
        let result = Self {
            collision_cache: crate::collision::CollisionCache::default(),
            animator,
            state,
            outputs: out,
        };
        result.validate()?;
        Ok(result)
    }
    /// One functional constructor and paid living food. No inactive clone population.
    /// Steering is an authored proximity policy, not learned or physical propulsion.
    pub fn first_simulation(
        spec: AnatomySpec,
        seed: u64,
        steps: usize,
    ) -> Result<Self, MorphError> {
        let (mut config, instances) =
            crate::filament::construction_setup(spec, seed, steps, 24, "head-first")?;
        let rules = config.construction.as_mut().unwrap();
        rules.experiment = format!("first-simulation/seed{seed}");
        rules.foraging = Some(crate::filament::ForagingRules {
            infrastructure: false,
            trail_on_motion: true,
            parallel_nozzles: instances[0].book.registered.len() > 1,
            axial_spin: if instances[0].book.registered.len() > 1 {
                50_000
            } else {
                0
            },
            search_radius: 12 * SCALE,
            mouth_radius: SCALE / 4,
            step: SCALE / 40,
            bite: 4,
            refill_below: 12,
            build_above: 48,
        });
        let steering = ConstructorState {
            rigid: rules.rigid(0),
            // Spend founding stock on a short four-stream root before refill.
            // This gives nearby food the existing scaffold needed for fixation.
            seeking_food: !rules.parallel_nozzles(),
        };
        let mut state = initial_snapshot(config, instances)?;
        state.steering = Some(steering);
        sow_blood_garden_state(&mut state)?;
        Self::restore(state)
    }
    pub fn habitat_simulation(
        mut spec: AnatomySpec,
        seed: u64,
        steps: usize,
    ) -> Result<Self, MorphError> {
        if spec.food_capacity() == 0 {
            spec.add_food_pouch();
        }
        let mut state = Self::first_simulation(spec, seed, steps)?.snapshot();
        let r = state.steering.unwrap().rigid;
        state.habitat = Some(crate::habitat::Habitat {
            contacts: vec![],
            rules: crate::habitat::HabitatRules::default(),
            agents: BTreeMap::from([(
                1,
                crate::habitat::Agent {
                    travel: None,
                    refilling: false,
                    rigid: r,
                    heading: [r.yaw_sin, 0, r.yaw_cos],
                    eating: false,
                    born: 0,
                    last_birth: 0,
                    parent: None,
                },
            )]),
            sacs: BTreeMap::new(),
        });
        let rules = state.config.construction.as_mut().unwrap();
        rules.foraging.as_mut().unwrap().infrastructure = true;
        rules.foraging.as_mut().unwrap().parallel_nozzles = true;
        rules.age = 0;
        state.instances.get_mut(&1).unwrap().book.launch_speed = 0;
        crate::collision::place_founders(&mut state)?;
        Self::restore(state)
    }
    /// A founding group paid from the finite condensate.
    /// `founders == 1` and `free_biomass == None` is `habitat_simulation` unchanged.
    /// Extra founders are initial conditions. They are not scripted births, and
    /// movement stays the authored habitat step.
    pub fn habitat_colony(
        spec: AnatomySpec,
        seed: u64,
        steps: usize,
        founders: usize,
        free_biomass: Option<u64>,
    ) -> Result<Self, MorphError> {
        let mut state = Self::habitat_simulation(spec, seed, steps)?.snapshot();
        require(
            (1..=16).contains(&founders),
            "founding population is outside the habitat budget of 16",
        )?;
        if founders > state.habitat.as_ref().unwrap().rules.max_agents {
            state.habitat.as_mut().unwrap().rules.max_agents = founders;
        }
        if let Some(free) = free_biomass {
            require(free <= 1_000_000, "condensate abundance")?;
            let stock = state.condensate.as_mut().unwrap();
            let paid = stock
                .initial
                .checked_sub(stock.biomass)
                .ok_or_else(|| MorphError("condensate paid more than it opened with".into()))?;
            stock.initial = free
                .checked_add(paid)
                .ok_or_else(|| MorphError("condensate abundance overflow".into()))?;
            stock.biomass = free;
        }
        if founders > 1 {
            let mut placement_cache = crate::collision::CollisionCache::default();
            let template = state.instances[&1].clone();
            let cost = template.book.initial;
            let spec = state.config.definitions[&template.definition].clone();
            for n in 1..founders {
                let biomass = state.condensate.as_ref().unwrap().biomass;
                require(
                    biomass >= cost && cost > 0,
                    "condensate cannot pay another founder",
                )?;
                let yaw = n as f64 * std::f64::consts::TAU / founders as f64;
                let heading = unit_heading(yaw);
                let mut near = template_pose(&state);
                near.translation = template
                    .position
                    .offset([
                        (yaw.cos() * 4. * SCALE as f64).round() as i64,
                        0,
                        (yaw.sin() * 4. * SCALE as f64).round() as i64,
                    ])?
                    .absolute()?;
                near.yaw_sin = heading[0];
                near.yaw_cos = heading[2];
                near.axial_roll = 0;
                let pose = crate::collision::spawn_pose(
                    &state,
                    template.definition,
                    near,
                    None,
                    &mut placement_cache,
                )?
                .ok_or_else(|| MorphError("no nonpenetrating founder placement".into()))?;
                let position = Position::from_absolute(pose.translation)?;
                state.condensate.as_mut().unwrap().biomass -= cost;
                let id = state.instances.keys().max().copied().unwrap() + 1;
                let mut book = template.book.clone();
                book.organism_id = id;
                book.bound.clear();
                book.tick = state.tick;
                book.initial = cost;
                book.external_input = 0;
                book.reservoir = cost;
                book.sinks = 0;
                book.next_id = 1;
                book.parcels.clear();
                book.admitted.clear();
                book.last_admit_tick.clear();
                book.emission_enabled = true;
                book.launch_speed = 0;
                book.last_status = crate::emission::EmissionStatus::Disabled;
                book.sync_nozzles(&spec);
                state.instances.insert(
                    id,
                    Instance {
                        id,
                        definition: template.definition,
                        position,
                        velocity: [0; 3],
                        exposure: exposure(&state.config, position)?,
                        book,
                    },
                );
                let mut agent = state.habitat.as_ref().unwrap().agents[&1].clone();
                agent.rigid = pose;
                agent.heading = heading;
                agent.eating = false;
                agent.born = state.tick;
                agent.last_birth = state.tick;
                agent.parent = None;
                state.habitat.as_mut().unwrap().agents.insert(id, agent);
            }
        }
        Self::restore(state)
    }
    /// A dozen small bodies in the near food cloud. They do not lay the snailpipe.
    pub fn sow_brood(&mut self, count: usize) -> Result<(), MorphError> {
        require((1..=24).contains(&count), "brood size")?;
        let spec = self.state.config.definitions[&1].clone();
        let environment = self.state.config.environment;
        let seed = self.state.config.seed;
        let tick = self.state.tick;
        let mut next_id = self.state.instances.keys().max().copied().unwrap_or(0);
        for n in 0..count {
            next_id += 1;
            let turn = n as f64 / count as f64 * std::f64::consts::TAU;
            let radius = 1.2 + (n % 4) as f64 * 0.35;
            let position = Position::from_absolute([
                ((6. + radius * turn.cos()) * SCALE as f64).round() as i64,
                ((n % 2) as f64 * 0.2 * SCALE as f64).round() as i64,
                ((radius * turn.sin()) * SCALE as f64).round() as i64,
            ])?;
            let mut book = crate::emission::MaterialBook::new(next_id, seed, 8, environment, 256)?;
            book.sync_nozzles(&spec);
            book.tick = tick;
            book.emission_enabled = false;
            let exposure = exposure(&self.state.config, position)?;
            self.state.instances.insert(
                next_id,
                Instance {
                    id: next_id,
                    definition: 1,
                    position,
                    velocity: [0; 3],
                    exposure,
                    book,
                },
            );
        }
        self.validate()
    }
    /// Finite condensate, one local lantern, and Blood-Sacs paid out of that stock.
    /// Light gates fixation. Biomass does not appear. The witness sphere is not consulted.
    pub fn sow_blood_garden(&mut self) -> Result<(), MorphError> {
        sow_blood_garden_state(&mut self.state)?;
        self.outputs = outputs(&self.state.config)?;
        self.validate()
    }
    /// Sow the generic, patchy food field. Empty ship volume stays empty.
    pub fn sow_generic_food(&mut self) -> Result<(), MorphError> {
        require(self.state.food.is_none(), "food is already sown")?;
        self.state.food = Some(crate::food::generic_field(self.state.config.seed)?);
        self.validate()
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
        if let Some(r) = &self.state.config.construction {
            require(
                self.state.events.len() + self.state.pending.len() + 2 < r.max_events
                    && self.state.pending.len() < 256,
                "intervention/history budget",
            )?;
        }
        match &command.action {
            Operation::SetEnvironment { environment }
                if self
                    .state
                    .habitat
                    .as_ref()
                    .is_some_and(|h| h.rules.collisions) =>
            {
                require(
                    *environment == Environment::suspended(),
                    "force-field changes need collision-aware droplet integration",
                )?;
            }
            Operation::SetEmission { organism, .. } => require(
                self.state.instances.contains_key(organism),
                "unknown emission target",
            )?,
            Operation::Assemble { organism, parcel } => {
                let r = self
                    .state
                    .config
                    .construction
                    .as_ref()
                    .ok_or_else(|| MorphError("construction disabled".into()))?;
                require(
                    *organism == r.organism
                        && self.state.instances[organism]
                            .book
                            .parcels
                            .iter()
                            .any(|p| p.id == *parcel),
                    "unavailable assembly material/target",
                )?;
            }
            _ => {}
        }
        self.state.events.push(Event::Accepted {
            at: self.state.tick,
            command: command.clone(),
        });
        self.state.pending.push(command);
        self.state.pending.sort_by_key(|c| (c.tick, c.sequence));
        Ok(())
    }
    /// Query counters for the last ticks. Not part of the world digest.
    pub fn collision_profile(&self) -> crate::collision::CollisionProfile {
        self.collision_cache.profile()
    }
    /// Diagnostic only. Turns habitat collision mode off for this runtime.
    /// Sac offsets, birth placement, spawn checks, and `collision::resolve`
    /// all follow the flag. The product default stays on. The resulting world
    /// is not an acceptable simulation.
    pub fn diagnostic_disable_collisions(&mut self) -> Result<(), MorphError> {
        let habitat =
            self.state.habitat.as_mut().ok_or_else(|| {
                MorphError("diagnostic collision switch requires a habitat".into())
            })?;
        habitat.rules.collisions = false;
        Ok(())
    }
    pub fn tick(&mut self) -> Result<(), MorphError> {
        // Existing events are immutable during a tick. Copy the mutable state,
        // record this tick separately, and append only after every fallible gate.
        // Work no longer grows with the entire retained causal event history.
        let mut next = WorldSnapshot {
            solids: self.state.solids.clone(),
            habitat: self.state.habitat.clone(),
            steering: self.state.steering,
            assembly: self.state.assembly.clone(),
            food: self.state.food.clone(),
            condensate: self.state.condensate.clone(),
            version: self.state.version,
            tick: self.state.tick,
            config: self.state.config.clone(),
            instances: self.state.instances.clone(),
            pending: self.state.pending.clone(),
            events: Vec::new(),
        };
        let tick = next.tick;
        let count = next.pending.partition_point(|c| c.tick == tick);
        let commands = next.pending.drain(..count).collect::<Vec<_>>();
        let mut manual_deposits = Vec::new();
        for c in commands {
            match c.action {
                Operation::SetEnvironment { environment } => {
                    next.config.environment = environment;
                    for i in next.instances.values_mut() {
                        i.book.environment = environment;
                    }
                }
                Operation::SetAmbientLight { value } => next.config.ambient_light = value,
                Operation::SetEmission { organism, enabled } => {
                    next.instances
                        .get_mut(&organism)
                        .ok_or_else(|| MorphError("emission target missing".into()))?
                        .book
                        .emission_enabled = enabled
                }
                Operation::Assemble { organism, parcel } => {
                    manual_deposits.push((organism, parcel))
                }
            }
            next.events.push(Event::Applied {
                at: tick,
                sequence: c.sequence,
            });
        }
        let construction = next.config.construction.clone();
        let mut habitat_plans = if next.habitat.is_some() {
            crate::habitat::prepare(&mut next)?
        } else {
            BTreeMap::new()
        };
        crate::collision::resolve(
            &self.state,
            &mut next,
            &mut habitat_plans,
            &mut self.collision_cache,
        )?;
        let steering = if next.habitat.is_some() {
            habitat_plans
                .get(&construction.as_ref().unwrap().organism)
                .copied()
        } else {
            steer_constructor(&next)?
        };
        let physical_next = next
            .habitat
            .as_ref()
            .is_some_and(|h| h.rules.collisions)
            .then(|| next.clone());
        for i in next.instances.values_mut() {
            require(i.book.tick == tick, "material substate timeline mismatch")?;
            let mut per_body = construction.clone();
            if habitat_plans.contains_key(&i.id) {
                let r = per_body.as_mut().unwrap();
                r.organism = i.id;
                r.nozzle = *i
                    .book
                    .registered
                    .first()
                    .ok_or_else(|| MorphError("constructor nozzle missing".into()))?;
            }
            let active = per_body.as_ref().filter(|r| r.organism == i.id);
            let steering = if next.habitat.is_some() {
                habitat_plans.get(&i.id).copied()
            } else {
                steering
            };
            let mut nozzle_positions = BTreeMap::new();
            let frame_outputs = if let Some(r) = active {
                let outputs = self
                    .animator
                    .as_ref()
                    .ok_or_else(|| MorphError("missing animator".into()))?
                    .output_frames(self.state.anatomical_pose(i.id)?)?;
                outputs
                    .iter()
                    .map(|f| crate::emission::place_output(f, self.state.body_pose(r.organism)?))
                    .collect::<Result<Vec<_>, _>>()?
            } else {
                self.outputs[&i.definition]
                    .iter()
                    .cloned()
                    .map(|mut f| {
                        let origin = i.position.absolute()?;
                        for (a, b) in f.origin.iter_mut().zip(origin) {
                            *a = a
                                .checked_add(b)
                                .ok_or_else(|| MorphError("nozzle position overflow".into()))?;
                        }
                        Ok(f)
                    })
                    .collect::<Result<Vec<_>, MorphError>>()?
            };
            for nozzle in frame_outputs {
                if active.is_some() {
                    nozzle_positions
                        .insert(nozzle.nozzle_id, Position::from_absolute(nozzle.origin)?);
                }
                if let Some(h) = &next.habitat {
                    let capacity = next.config.definitions[&i.definition].food_capacity();
                    let reserve = h.rules.reserve_food.min(capacity / 4);
                    if i.book.external_input > 0
                        && i.book.bound.len() >= i.book.registered.len() * 8
                        && i.book.reservoir <= reserve
                    {
                        continue;
                    }
                }
                if active.is_some_and(|r| !r.emitting(tick)) {
                    continue;
                }
                if active.is_some_and(|r| r.foraging.as_ref().is_some_and(|f| f.trail_on_motion)) {
                    let origin = i.position.absolute()?;
                    if steering.is_none_or(|state| state.rigid.translation == origin) {
                        continue;
                    }
                }
                // Parallel nozzles can spend an entire bite in one tick. Refill
                // first, then lay while building; otherwise four outlets park
                // forever at a four-unit meal with no reserve for a building pass.
                if active.is_some_and(|rules| rules.parallel_nozzles())
                    && steering.is_some_and(|state| state.seeking_food)
                {
                    continue;
                }
                if let Some(physical) = &physical_next
                    && let Some(target) = crate::collision::emission_obstacle(
                        physical,
                        next.assembly.as_ref().unwrap(),
                        i.id,
                        nozzle.nozzle_id,
                        nozzle.origin,
                        &mut self.collision_cache,
                    )?
                {
                    let observation = crate::collision::ContactObservation {
                        at: tick,
                        predicted: true,
                        organism: i.id,
                        target,
                        point: Position::from_absolute(nozzle.origin)?,
                        normal: [0; 3],
                        response: crate::collision::ContactResponse::SecretionBlocked,
                        toi_millionths: 0,
                    };
                    next.habitat
                        .as_mut()
                        .unwrap()
                        .contacts
                        .push(observation.clone());
                    next.events.push(Event::Contact { observation });
                    next.events.push(Event::EmissionRejected {
                        at: tick,
                        organism: i.id,
                        reason: "PhysicalObstruction".into(),
                    });
                    continue;
                }
                let status = i.book.try_emit(EmitRequest {
                    nozzle_id: nozzle.nozzle_id,
                    origin: nozzle.origin,
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
                } else if active.is_some() {
                    next.events.push(Event::EmissionRejected {
                        at: tick,
                        organism: i.id,
                        reason: format!("{status:?}"),
                    });
                }
            }
            i.book.step()?;
            if let Some(r) = active {
                let eligible = if manual_deposits.iter().any(|(id, _)| *id == i.id) {
                    manual_deposits
                        .iter()
                        .filter(|(id, _)| *id == i.id)
                        .map(|(_, p)| *p)
                        .collect::<Vec<_>>()
                } else {
                    automatic_deposits(&i.book, r, &nozzle_positions, next.habitat.is_some())?
                };
                if eligible.is_empty() {
                    next.events.push(Event::AssemblyRejected {
                        at: tick,
                        organism: i.id,
                        parcel: None,
                        reason: "NoEligibleMaterial".into(),
                    });
                }
                let mut deposited_nozzles = BTreeSet::new();
                for parcel in eligible {
                    let nozzle = i
                        .book
                        .parcels
                        .iter()
                        .find(|p| p.id == parcel)
                        .ok_or_else(|| MorphError("assembly parcel missing".into()))?
                        .nozzle_id;
                    if deposited_nozzles.contains(&nozzle) {
                        continue;
                    }
                    if let Some(physical) = &physical_next
                        && !crate::collision::deposit_allowed(
                            physical,
                            next.assembly.as_ref().unwrap(),
                            &i.book,
                            r,
                            parcel,
                            &mut self.collision_cache,
                        )?
                    {
                        next.events.push(Event::AssemblyRejected {
                            at: tick,
                            organism: i.id,
                            parcel: Some(parcel),
                            reason: "PhysicalObstruction".into(),
                        });
                        continue;
                    }
                    let result =
                        next.assembly
                            .as_mut()
                            .ok_or_else(|| MorphError("assembly missing".into()))?
                            .deposit(
                                &mut i.book,
                                r,
                                parcel,
                                nozzle_positions.get(&nozzle).copied().ok_or_else(|| {
                                    MorphError("construction nozzle missing".into())
                                })?,
                            );
                    match result {
                        Ok((filament, node, quantity)) => {
                            next.events.push(Event::Deposited {
                                at: tick,
                                organism: i.id,
                                parcel,
                                filament,
                                node,
                                quantity,
                            });
                            deposited_nozzles.insert(nozzle);
                            if !r.parallel_nozzles() {
                                break;
                            }
                        }
                        Err(e) => next.events.push(Event::AssemblyRejected {
                            at: tick,
                            organism: i.id,
                            parcel: Some(parcel),
                            reason: e.to_string(),
                        }),
                    }
                }
                i.position = Position::from_absolute(
                    steering
                        .map_or_else(|| r.rigid(tick + 1), |state| state.rigid)
                        .translation,
                )?;
            } else {
                i.position = i.position.offset(i.velocity)?;
            }
            i.exposure = exposure(&next.config, i.position)?;
            require(i.book.residual()? == 0, "ship material balance failed")?;
            for p in &i.book.parcels {
                Position::from_absolute(p.position)?;
            }
        }
        if steering.is_some() {
            next.steering = steering;
        }
        if let Some(food) = next.food.as_mut() {
            let food_rules = food.rules;
            let construction = next.config.construction.clone();
            let mut assembly = next.assembly.take();
            let mut chewed = Vec::new();
            if let (Some(structure), Some(rules)) = (assembly.as_mut(), construction.as_ref()) {
                for organism in next.instances.values_mut() {
                    if organism.book.reservoir > food_rules.chew_below {
                        continue;
                    }
                    if let Some(chew) = structure.chew_dried(
                        &mut organism.book,
                        rules,
                        organism.position,
                        food_rules.chew_radius,
                        food_rules.dried_age,
                        food_rules.fresh_tail,
                    )? {
                        chewed.push((organism.id, chew));
                    }
                }
            }
            next.assembly = assembly;
            for (organism, chew) in chewed {
                next.events.push(Event::Chewed {
                    at: tick,
                    organism,
                    filament: chew.filament,
                    node: chew.node,
                    parcel: chew.parcel,
                    quantity: chew.quantity,
                });
            }
            let flesh = next
                .assembly
                .as_ref()
                .map(|structure| {
                    structure
                        .filaments
                        .values()
                        .flat_map(|filament| filament.nodes.iter())
                        .map(|node| node.position.absolute())
                        .collect::<Result<Vec<_>, _>>()
                })
                .transpose()?
                .unwrap_or_default();
            let mouths: Vec<(u64, Position)> = next
                .instances
                .values()
                .map(|organism| (organism.id, organism.position))
                .collect();
            let bursts = food.step(tick, &flesh, &mouths)?;
            for burst in bursts {
                if let Some(organism) = next.instances.get_mut(&burst.organism) {
                    organism.book.ingest(food_rules.bite)?;
                }
                next.events.push(Event::FoodBurst {
                    at: tick,
                    site: burst.site,
                    seeds: burst.seeds,
                });
            }
        }
        blood_tick(&mut next)?;
        if next.habitat.is_some() {
            crate::habitat::finish(&mut next, &mut self.collision_cache)?;
            crate::collision::admit_roots(&self.state, &mut next, &mut self.collision_cache)?;
        } else {
            birth(&mut next)?;
        }
        next.tick = tick
            .checked_add(1)
            .ok_or_else(|| MorphError("world tick overflow".into()))?;
        if let Some(r) = &next.config.construction {
            require(
                self.state.events.len().saturating_add(next.events.len()) <= r.max_events,
                "HistoryBudget: no authoritative ticks discarded",
            )?;
        }
        if next
            .config
            .construction
            .as_ref()
            .is_some_and(|r| r.foraging.is_some())
        {
            require(
                next.condensate_gap() == 0,
                "first-simulation condensate balance failed",
            )?;
        }
        let mut history = std::mem::take(&mut self.state.events);
        history.append(&mut next.events);
        next.events = history;
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
            ((s.version == 1 && s.config.construction.is_none())
                || (s.version == 2 && s.config.construction.is_some()))
                && s.instances.len() <= 100_000,
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
        require(s.solids.len() <= 1024, "physical obstacle budget")?;
        let mut solid_ids = BTreeSet::new();
        for obstacle in &s.solids {
            let min = obstacle.min.absolute()?;
            let max = obstacle.max.absolute()?;
            require(
                solid_ids.insert(obstacle.id) && (0..3).all(|k| max[k] > min[k]),
                "invalid physical obstacle",
            )?;
        }
        crate::habitat::validate(s)?;
        if let Some(r) = &s.config.construction {
            r.validate()?;
            require(
                s.events.len() <= r.max_events && s.pending.len() <= 256,
                "construction event/command budget",
            )?;
            let i = s
                .instances
                .get(&r.organism)
                .ok_or_else(|| MorphError("construction source missing".into()))?;
            require(
                (r.foraging.is_some() || i.book.registered.len() == 1)
                    && i.book.registered.contains(&r.nozzle),
                "one configured construction nozzle required",
            )?;
            if r.foraging.is_some() {
                let rigid = s
                    .steering
                    .ok_or_else(|| MorphError("missing constructor steering state".into()))?;
                crate::emission::place_point([0.; 3], rigid.rigid)?;
                require(
                    i.position.absolute()? == rigid.rigid.translation,
                    "constructor pose mismatch",
                )?;
                require(
                    s.food.is_none() && s.condensate.is_some() && s.condensate_gap() == 0,
                    "first simulation requires its finite food ledger",
                )?;
                let eaten = s
                    .events
                    .iter()
                    .filter_map(|event| match event {
                        Event::Fed {
                            organism, quantity, ..
                        } if *organism == r.organism => Some(*quantity),
                        _ => None,
                    })
                    .try_fold(0u64, |sum, q| sum.checked_add(q))
                    .ok_or_else(|| MorphError("intake event overflow".into()))?;
                require(
                    eaten == i.book.external_input,
                    "constructor intake provenance mismatch",
                )?;
            } else {
                require(
                    s.steering.is_none() && i.position.absolute()? == r.rigid(s.tick).translation,
                    "trajectory state mismatch",
                )?;
            }
            if let Some(h) = &s.habitat {
                for id in h.agents.keys() {
                    let mut per_body = r.clone();
                    per_body.organism = *id;
                    per_body.nozzle = *s.instances[id].book.registered.first().unwrap();
                    s.assembly
                        .as_ref()
                        .unwrap()
                        .validate(&s.instances[id].book, &per_body)?;
                }
            } else {
                s.assembly
                    .as_ref()
                    .ok_or_else(|| MorphError("missing assembly state".into()))?
                    .validate(&i.book, r)?;
            }
        } else {
            require(
                s.steering.is_none()
                    && s.assembly.is_none()
                    && s.instances.values().all(|i| i.book.bound.is_empty()),
                "unconfigured structural state",
            )?;
        }
        if let Some(food) = &s.food {
            food.validate()?;
        }
        let mut accepted = BTreeMap::new();
        let mut applied = BTreeSet::new();
        let mut births = BTreeSet::new();
        let mut deposits = BTreeSet::new();
        let chewed: BTreeMap<(u64, u64), u64> = s
            .events
            .iter()
            .filter_map(|e| match e {
                Event::Chewed {
                    organism,
                    parcel,
                    quantity,
                    ..
                } => Some(((*organism, *parcel), *quantity)),
                _ => None,
            })
            .collect();
        for e in &s.events {
            match e {
                Event::Contact { observation } => {
                    require(
                        observation.at < s.tick
                            && s.instances.contains_key(&observation.organism)
                            && observation.toi_millionths <= 1_000_000,
                        "invalid contact observation",
                    )?;
                    observation.point.absolute()?;
                }
                Event::Reproduced {
                    at,
                    parent,
                    child,
                    quantity,
                } => {
                    require(
                        *at < s.tick
                            && *quantity > 0
                            && s.instances.contains_key(parent)
                            && s.instances
                                .get(child)
                                .is_some_and(|i| i.book.initial == *quantity),
                        "invalid reproductive transfer",
                    )?;
                }
                Event::Deposited {
                    at,
                    organism,
                    parcel,
                    filament,
                    node,
                    quantity,
                } => {
                    let i = s
                        .instances
                        .get(organism)
                        .ok_or_else(|| MorphError("deposition source missing".into()))?;
                    let key = (*organism, *parcel);
                    if let Some(reclaimed) = chewed.get(&key) {
                        require(
                            deposits.insert(key)
                                && *at < s.tick
                                && *reclaimed == *quantity
                                && i.book.bound.iter().all(|b| b.parcel.id != *parcel),
                            "invalid chewed deposition",
                        )?;
                    } else {
                        require(
                            deposits.insert(key)
                                && *at < s.tick
                                && i.book.bound.iter().any(|b| {
                                    b.parcel.id == *parcel
                                        && b.filament == *filament
                                        && b.node == *node
                                        && b.parcel.quantity == *quantity
                                        && b.bound_tick == at + 1
                                }),
                            "invalid deposition event",
                        )?;
                    }
                }
                Event::Chewed {
                    at,
                    organism,
                    parcel,
                    quantity,
                    ..
                } => require(
                    *at < s.tick
                        && *quantity > 0
                        && s.instances.contains_key(organism)
                        && chewed.get(&(*organism, *parcel)) == Some(quantity),
                    "invalid chew event",
                )?,
                Event::Fed {
                    at,
                    organism,
                    source,
                    quantity,
                    mouth,
                } => {
                    require(
                        *at < s.tick
                            && *quantity > 0
                            && *organism != *source
                            && s.instances.contains_key(organism)
                            && s.instances
                                .get(source)
                                .is_some_and(|i| i.definition == BLOOD_SAC),
                        "invalid feeding event",
                    )?;
                    mouth.absolute()?;
                }
                Event::FoodBurst { at, seeds, .. } => {
                    require(*at < s.tick && *seeds > 0, "invalid food burst")?
                }
                Event::AssemblyRejected { at, organism, .. }
                | Event::EmissionRejected { at, organism, .. } => require(
                    *at < s.tick && s.instances.contains_key(organism),
                    "invalid rejection event",
                )?,
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
                    let reclaimed = chewed.contains_key(&(*organism, *parcel));
                    require(
                        births.insert((*organism, *parcel))
                            && *at < s.tick
                            && *quantity > 0
                            && (reclaimed
                                || i.book
                                    .parcels
                                    .iter()
                                    .chain(i.book.bound.iter().map(|b| &b.parcel))
                                    .any(|p| {
                                        p.id == *parcel
                                            && p.nozzle_id == *nozzle
                                            && p.quantity == *quantity
                                            && p.created_tick == *at
                                    })),
                        "invalid emission event",
                    )?;
                }
            }
        }
        let carried = s
            .instances
            .values()
            .map(|i| i.book.parcels.len() + i.book.bound.len())
            .sum::<usize>();
        require(
            births.len() == carried + chewed.len(),
            "material birth history mismatch",
        )?;
        require(
            deposits.len()
                == s.instances
                    .values()
                    .map(|i| i.book.bound.len())
                    .sum::<usize>()
                    + chewed.len(),
            "missing/duplicate deposition history",
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
fn birth(world: &mut WorldSnapshot) -> Result<(), MorphError> {
    if world.instances.len() >= 36 {
        return Ok(());
    }
    let ready: Vec<u64> = world
        .instances
        .iter()
        .filter(|(id, organism)| {
            !world
                .config
                .construction
                .as_ref()
                .is_some_and(|r| r.foraging.is_some() && r.organism == **id)
                && organism.book.external_input > 0
                && organism.book.reservoir >= 24
                && world.tick % 90 == *id % 90
        })
        .map(|(id, _)| *id)
        .collect();
    for parent_id in ready {
        if world.instances.len() >= 36 {
            break;
        }
        let (at, definition, environment, clock) = {
            let parent = world.instances.get_mut(&parent_id).unwrap();
            parent.book.endow(16)?;
            (
                parent.position,
                parent.definition,
                parent.book.environment,
                parent.book.tick,
            )
        };
        let child_id = world.instances.keys().max().copied().unwrap() + 1;
        let spec = world.config.definitions[&definition].clone();
        let mut book =
            crate::emission::MaterialBook::new(child_id, world.config.seed, 16, environment, 256)?;
        book.sync_nozzles(&spec);
        book.tick = clock;
        book.emission_enabled = false;
        let position = at.offset([
            SCALE + (child_id as i64 % 5) * (SCALE / 5),
            0,
            (child_id as i64 % 7) * (SCALE / 7),
        ])?;
        world.instances.insert(
            child_id,
            Instance {
                id: child_id,
                definition,
                position,
                velocity: [0; 3],
                exposure: 0,
                book,
            },
        );
    }
    Ok(())
}
fn validate_command(c: &Intervention, tick: u64) -> Result<(), MorphError> {
    require(
        c.tick >= tick && !c.origin.trim().is_empty() && c.origin.len() <= 128,
        "invalid intervention tick/origin",
    )?;
    match c.action {
        Operation::SetEnvironment { environment } => environment.validate(),
        Operation::SetEmission { .. } | Operation::Assemble { .. } => Ok(()),
        Operation::SetAmbientLight { value } => {
            require(value <= 1_000_000_000, "ambient light budget")
        }
    }
}
fn pay_sac(
    world: &mut WorldSnapshot,
    position: Position,
    quantity: u64,
    clock: u64,
) -> Result<(), MorphError> {
    {
        let stock = world
            .condensate
            .as_mut()
            .ok_or_else(|| MorphError("no condensate".into()))?;
        require(stock.biomass >= quantity, "condensate exhausted")?;
        stock.biomass -= quantity;
    }
    insert_sac(world, position, quantity, clock)
}

pub(crate) fn insert_sac(
    world: &mut WorldSnapshot,
    position: Position,
    quantity: u64,
    clock: u64,
) -> Result<(), MorphError> {
    let spec = world
        .config
        .definitions
        .get(&BLOOD_SAC)
        .ok_or_else(|| MorphError("blood sac definition missing".into()))?
        .clone();
    let id = world.instances.keys().max().copied().unwrap_or(0) + 1;
    let mut book = MaterialBook::new(
        id,
        world.config.seed,
        quantity,
        world.config.environment,
        256,
    )?;
    book.sync_nozzles(&spec);
    book.tick = clock;
    book.emission_enabled = false;
    let lit = exposure(&world.config, position)?;
    world.instances.insert(
        id,
        Instance {
            id,
            definition: BLOOD_SAC,
            position,
            velocity: [0; 3],
            exposure: lit,
            book,
        },
    );
    Ok(())
}

fn beside_flesh(position: Position, flesh: &[[i64; 3]]) -> Result<bool, MorphError> {
    let limit = i128::from(2 * SCALE) * i128::from(2 * SCALE);
    for point in flesh {
        let there = Position::from_absolute(*point)?;
        let d = position.relative(there)?;
        let distance: i128 = d.iter().map(|v| i128::from(*v) * i128::from(*v)).sum();
        if distance <= limit {
            return Ok(true);
        }
    }
    Ok(false)
}

fn spray_delta(parent: u64, n: u32) -> [i64; 3] {
    let angle = (parent as f64 * 0.7) + f64::from(n) * 0.9;
    let reach = 0.35 + f64::from(n % 5) * 0.18;
    [
        (reach * angle.cos() * SCALE as f64).round() as i64,
        ((0.2 + f64::from(n % 3) * 0.25) * SCALE as f64).round() as i64,
        (reach * angle.sin() * SCALE as f64).round() as i64,
    ]
}

/// A bite empties the sac. Spores are a one-step spray, then they stay.
/// That is the transport approximation: no fluid solver.
fn hemorrhage(
    world: &mut WorldSnapshot,
    sac: u64,
    clock: u64,
    limit: u32,
) -> Result<(), MorphError> {
    for n in 0..limit {
        if world.instances.len() >= 36 {
            break;
        }
        let reservoir = world.instances[&sac].book.reservoir;
        if reservoir <= 1 {
            break;
        }
        let at = world.instances[&sac].position;
        world.instances.get_mut(&sac).unwrap().book.endow(1)?;
        insert_sac(world, at.offset(spray_delta(sac, n))?, 1, clock)?;
    }
    Ok(())
}

/// Bounded nearest-food steering. Deliberately authored kinematics, with no
/// physics, learned weights, global food oracle, or observer input.
fn steer_constructor(world: &WorldSnapshot) -> Result<Option<ConstructorState>, MorphError> {
    let Some(construction) = &world.config.construction else {
        return Ok(None);
    };
    let Some(rules) = &construction.foraging else {
        return Ok(None);
    };
    let id = construction.organism;
    let mut state = world
        .steering
        .ok_or_else(|| MorphError("missing constructor state".into()))?;
    let body = &world.instances[&id];
    if body.book.reservoir <= rules.refill_below {
        state.seeking_food = true;
    }
    if body.book.reservoir >= rules.build_above {
        state.seeking_food = false;
    }
    let mut rigid = state.rigid;
    if !state.seeking_food {
        rigid.axial_roll = (rigid.axial_roll + rules.axial_spin).rem_euclid(6_283_185);
        // A fed constructor advances along its current axis while paying for
        // secretion. No learned controller or propulsion law is implied.
        if body.book.emission_enabled && body.book.reservoir > 0 {
            let delta = [
                i64::try_from(
                    i128::from(rigid.yaw_sin) * i128::from(rules.step) / i128::from(SCALE),
                )
                .unwrap(),
                0,
                i64::try_from(
                    i128::from(rigid.yaw_cos) * i128::from(rules.step) / i128::from(SCALE),
                )
                .unwrap(),
            ];
            rigid.translation = Position::from_absolute(rigid.translation)?
                .offset(delta)?
                .absolute()?;
        }
        state.rigid = rigid;
        return Ok(Some(state));
    }
    let Some(mouth) = world.mouth_position_at(id, world.tick + 1)? else {
        state.rigid = rigid;
        return Ok(Some(state));
    };
    let Some(target) = meal_within(world, mouth, rules.search_radius)? else {
        state.rigid = rigid;
        return Ok(Some(state));
    };
    let target = world.instances[&target].position;
    let delta = target.relative(mouth)?;
    let distance2: i128 = delta.iter().map(|v| i128::from(*v).pow(2)).sum();
    if distance2 <= i128::from(rules.mouth_radius).pow(2) {
        state.rigid = rigid;
        return Ok(Some(state));
    }
    let heading = target.relative(Position::from_absolute(rigid.translation)?)?;
    if heading[0] != 0 || heading[2] != 0 {
        let yaw = (heading[0] as f64).atan2(heading[2] as f64);
        rigid.yaw_sin = (yaw.sin() * SCALE as f64).round() as i64;
        rigid.yaw_cos = (yaw.cos() * SCALE as f64).round() as i64;
    }
    let geometry =
        CompiledAnatomy::new(world.config.definitions[&world.instances[&id].definition].clone())?
            .pose(world.anatomical_pose_at(id, world.tick + 1)?)?
            .geometry;
    let mouth =
        Position::from_absolute(crate::emission::place_point(geometry.mouth_center, rigid)?)?;
    let delta = target.relative(mouth)?;
    let length = delta
        .iter()
        .map(|v| (*v as f64).powi(2))
        .sum::<f64>()
        .sqrt();
    let step = (length - rules.mouth_radius as f64 * 0.5).clamp(0., rules.step as f64);
    if length > 0. {
        for (axis, value) in delta.into_iter().enumerate() {
            rigid.translation[axis] = rigid.translation[axis]
                .checked_add((value as f64 * step / length).round() as i64)
                .ok_or_else(|| MorphError("constructor movement overflow".into()))?;
        }
    }
    Position::from_absolute(rigid.translation)?;
    state.rigid = rigid;
    Ok(Some(state))
}

fn meal_within(
    world: &WorldSnapshot,
    mouth: Position,
    radius: i64,
) -> Result<Option<u64>, MorphError> {
    let mut best: Option<(i128, u64)> = None;
    let limit = i128::from(radius) * i128::from(radius);
    for organism in world.instances.values() {
        if organism.definition != BLOOD_SAC || organism.book.reservoir < 2 {
            continue;
        }
        let d = organism.position.relative(mouth)?;
        let distance: i128 = d.iter().map(|v| i128::from(*v) * i128::from(*v)).sum();
        if distance <= limit && best.is_none_or(|(so_far, _)| distance < so_far) {
            best = Some((distance, organism.id));
        }
    }
    Ok(best.map(|(_, id)| id))
}

fn blood_tick(world: &mut WorldSnapshot) -> Result<(), MorphError> {
    if world.condensate.is_none() || !world.config.definitions.contains_key(&BLOOD_SAC) {
        return Ok(());
    }
    let clock = world.tick + 1;
    let flesh = world
        .assembly
        .as_ref()
        .map(|structure| {
            structure
                .filaments
                .values()
                .flat_map(|filament| filament.nodes.iter())
                .map(|node| node.position.absolute())
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?
        .unwrap_or_default();
    let sacs: Vec<u64> = world
        .instances
        .values()
        .filter(|organism| organism.definition == BLOOD_SAC)
        .map(|organism| organism.id)
        .collect();
    for id in sacs {
        let organism = &world.instances[&id];
        if world
            .habitat
            .as_ref()
            .is_some_and(|h| !h.sacs.contains_key(&id))
        {
            continue;
        }
        if organism.book.reservoir >= 64
            || organism.exposure < 30
            || !beside_flesh(organism.position, &flesh)?
        {
            continue;
        }
        for _ in 0..2 {
            if world.condensate.as_ref().unwrap().biomass == 0
                || world.instances[&id].book.reservoir >= 64
            {
                break;
            }
            world.condensate.as_mut().unwrap().biomass -= 1;
            world.instances.get_mut(&id).unwrap().book.ingest(1)?;
        }
    }
    let builder = world
        .config
        .construction
        .as_ref()
        .map(|rules| rules.organism);
    let foraging = world
        .config
        .construction
        .as_ref()
        .and_then(|r| r.foraging.as_ref())
        .cloned();
    let mouths: Vec<(u64, Position)> = world
        .instances
        .values()
        .filter(|organism| {
            organism.definition != BLOOD_SAC
                && world
                    .habitat
                    .as_ref()
                    .is_none_or(|h| h.agents.get(&organism.id).is_some_and(|a| a.eating))
                && (foraging.is_some() || Some(organism.id) != builder)
                && (Some(organism.id) != builder
                    || world.steering.is_none_or(|state| state.seeking_food))
        })
        .map(|organism| {
            if foraging.is_some() {
                Ok(world
                    .mouth_position_at(organism.id, clock)?
                    .map(|mouth| (organism.id, mouth)))
            } else {
                Ok(Some((organism.id, organism.position)))
            }
        })
        .collect::<Result<Vec<_>, MorphError>>()?
        .into_iter()
        .flatten()
        .collect();
    for (eater, mouth) in mouths {
        let radius = if world.habitat.is_some() {
            420_000
        } else {
            foraging.as_ref().map_or(2 * SCALE, |r| r.mouth_radius)
        };
        let Some(sac) = meal_within(world, mouth, radius)? else {
            continue;
        };
        let room = if world.habitat.is_some() {
            world.config.definitions[&world.instances[&eater].definition]
                .food_capacity()
                .saturating_sub(world.instances[&eater].book.reservoir)
        } else {
            u64::MAX
        };
        let gulp = world.instances[&sac]
            .book
            .reservoir
            .saturating_sub(1)
            .min(foraging.as_ref().map_or(8, |r| r.bite))
            .min(room);
        if gulp == 0 {
            continue;
        }
        world.instances.get_mut(&sac).unwrap().book.endow(gulp)?;
        world.instances.get_mut(&eater).unwrap().book.ingest(gulp)?;
        if foraging.is_some() {
            world.events.push(Event::Fed {
                at: world.tick,
                organism: eater,
                source: sac,
                quantity: gulp,
                mouth,
            });
        }
        if world.habitat.is_none() {
            hemorrhage(world, sac, clock, 18)?;
        }
    }
    if world.habitat.is_some() {
        return Ok(());
    }
    let mature: Vec<u64> = world
        .instances
        .values()
        .filter(|organism| organism.definition == BLOOD_SAC && organism.book.reservoir >= 8)
        .map(|organism| organism.id)
        .collect();
    for id in mature {
        hemorrhage(world, id, clock, 6)?;
    }
    Ok(())
}

impl WorldSnapshot {
    pub fn contact_observations(
        &self,
        organism: u64,
    ) -> Vec<&crate::collision::ContactObservation> {
        self.habitat.as_ref().map_or_else(Vec::new, |h| {
            h.contacts
                .iter()
                .filter(|c| c.organism == organism)
                .collect()
        })
    }
    /// Shared by numerical landmarks and the renderer. Camera state is never read.
    pub fn body_pose(&self, organism: u64) -> Result<RigidPose, MorphError> {
        let instance = self
            .instances
            .get(&organism)
            .ok_or_else(|| MorphError("body missing".into()))?;
        if let Some(agent) = self.habitat.as_ref().and_then(|h| h.agents.get(&organism)) {
            return Ok(agent.rigid);
        }
        if let Some(rules) = self
            .config
            .construction
            .as_ref()
            .filter(|r| r.organism == organism)
        {
            if rules.foraging.is_some() {
                return self
                    .steering
                    .map(|state| state.rigid)
                    .ok_or_else(|| MorphError("missing constructor pose".into()));
            }
            return Ok(rules.rigid(self.tick));
        }
        Ok(RigidPose {
            translation: instance.position.absolute()?,
            ..RigidPose::identity()
        })
    }
    pub fn anatomical_pose(&self, organism: u64) -> Result<Pose, MorphError> {
        self.anatomical_pose_at(organism, self.tick)
    }
    fn anatomical_pose_at(&self, organism: u64, tick: u64) -> Result<Pose, MorphError> {
        let instance = self
            .instances
            .get(&organism)
            .ok_or_else(|| MorphError("body missing".into()))?;
        let spec = &self.config.definitions[&instance.definition];
        if self
            .habitat
            .as_ref()
            .is_some_and(|h| h.agents.contains_key(&organism))
        {
            let eating = self
                .habitat
                .as_ref()
                .unwrap()
                .agents
                .get(&organism)
                .is_some_and(|agent| agent.eating);
            return Ok(crate::collision::habitat_presentation_pose(
                spec, tick, eating,
            ));
        }
        if let Some(rules) = self
            .config
            .construction
            .as_ref()
            .filter(|r| r.organism == organism)
        {
            let mut pose = crate::filament::construction_pose(tick);
            if rules.foraging.is_some() {
                pose.mouth = spec.body.mouth.aperture;
            }
            Ok(pose)
        } else {
            Ok(Pose::rest(&spec.body))
        }
    }
    pub fn mouth_position(&self, organism: u64) -> Result<Option<Position>, MorphError> {
        self.mouth_position_at(organism, self.tick)
    }
    fn mouth_position_at(&self, organism: u64, tick: u64) -> Result<Option<Position>, MorphError> {
        let pose = self.anatomical_pose_at(organism, tick)?;
        if pose.mouth <= 0.01 {
            return Ok(None);
        }
        let instance = &self.instances[&organism];
        let spec = &self.config.definitions[&instance.definition];
        spec.validate()?;
        let mouth = crate::compiler::mouth_landmark(&spec.body, pose);
        Ok(Some(Position::from_absolute(
            crate::emission::place_point(mouth, self.body_pose(organism)?)?,
        )?))
    }

    /// Zero when every condensate quantum is still in the stock or in a book.
    pub fn condensate_gap(&self) -> i128 {
        let Some(stock) = &self.condensate else {
            return 0;
        };
        let mut gap =
            i128::from(stock.founders) + i128::from(stock.initial) - i128::from(stock.biomass);
        for organism in self.instances.values() {
            gap -= i128::from(organism.book.initial);
            gap -= i128::from(organism.book.external_input);
            gap += i128::from(organism.book.sinks);
        }
        gap
    }
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
        construction: None,
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

#[cfg(test)]
mod candidate_tests {
    use super::*;
    #[test]
    fn unreachable_old_parcels_cannot_starve_a_reachable_deposit() {
        let world = ShipRuntime::habitat_simulation(AnatomySpec::bare(77), 77, 64)
            .unwrap()
            .snapshot();
        let spec = &world.config.definitions[&1];
        let mut book = MaterialBook::new(1, 77, 64, Environment::suspended(), 4096).unwrap();
        book.sync_nozzles(spec);
        book.emission_enabled = true;
        let nozzle = *book.registered.first().unwrap();
        let mut last = 0;
        for k in 0..41 {
            let status = book.try_emit(EmitRequest {
                nozzle_id: nozzle,
                origin: if k < 40 { [10 * SCALE, 0, 0] } else { [0; 3] },
                direction: [0, 0, SCALE],
                quantity: 1,
                launch_speed: 0,
                channel: SECRETION_CHANNEL,
            });
            let crate::emission::EmissionStatus::Admitted { parcel_id, .. } = status else {
                panic!("fixture parcel must be paid")
            };
            last = parcel_id;
            book.step().unwrap();
        }
        let points = BTreeMap::from([(nozzle, Position::from_absolute([0; 3]).unwrap())]);
        let rules = world.config.construction.as_ref().unwrap();
        assert_eq!(
            automatic_deposits(&book, rules, &points, true).unwrap(),
            vec![last]
        );
        assert_eq!(
            automatic_deposits(&book, rules, &points, false)
                .unwrap()
                .len(),
            32
        );
        assert_eq!(
            book.parcels.len(),
            41,
            "unreachable material must remain in the physical world"
        );
        assert_eq!(book.residual().unwrap(), 0);
    }
}
