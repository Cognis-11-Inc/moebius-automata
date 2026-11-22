use std::collections::BTreeMap;

use crate::farm::{
    BrainRig, DIG_START, JOINT_LIMIT, SECRETE_ACTION, SPINE_JOINTS, action_dir, fused_surface,
};
use rand::RngCore;

use crate::config::{GenomeFill, SimConfig};
use crate::error::Error;
use crate::geom::{
    Dims, MoveOutcome, MoveProposal, Site, lattice_wave, resolve_moves, site_from_index,
    site_index, uptake_units,
};
use crate::prng::{movement_target, placement_rng, unbiased_below};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cell {
    pub id: u64,
    pub site: Site,
    pub energy: u64,
    pub age: u64,
    pub genome: [u8; 8],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeathRecord {
    pub id: u64,
    pub site: Site,
    pub energy_disposed: u64,
    pub age: u64,
    pub during_tick: u64,
    pub genome: [u8; 8],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TickReceipt {
    pub tick_after: u64,
    pub external_input: u64,
    pub reservoir_to_field: u64,
    pub nutrient_uptaken: u64,
    pub waste_created: u64,
    pub energy_credited: u64,
    pub energy_maintenance: u64,
    pub energy_disposed: u64,
    pub external_output: u64,
    pub deaths: u32,
    pub moves_applied: u32,
    pub moves_blocked: u32,
    pub births: u32,
    pub motor_energy: u64,
    pub neural_energy: u64,
    pub jet_released: u64,
    pub jet_deposited: u64,
    pub jet_wasted: u64,
    pub secreted: u64,
    pub excavated: u64,
    /// Reservoir material condensed into lasting flesh on a vacated site.
    pub trail_laid: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Accounts {
    pub material_initial: u64,
    pub external_input: u64,
    pub external_output: u64,
    pub field: u64,
    pub flesh: u64,
    pub in_flight: u64,
    pub reservoir: u64,
    pub waste: u64,
    pub energy_live: u64,
    pub energy_credited: u64,
    pub energy_maintenance: u64,
    pub energy_disposed: u64,
}

impl Accounts {
    pub fn material_residual(&self) -> i128 {
        let left = i128::from(self.material_initial) + i128::from(self.external_input);
        let right = i128::from(self.field)
            + i128::from(self.flesh)
            + i128::from(self.in_flight)
            + i128::from(self.reservoir)
            + i128::from(self.waste)
            + i128::from(self.external_output);
        left - right
    }

    pub fn energy_residual(&self) -> i128 {
        let right = i128::from(self.energy_live)
            + i128::from(self.energy_maintenance)
            + i128::from(self.energy_disposed);
        i128::from(self.energy_credited) - right
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stocks {
    pub field: u64,
    pub flesh: u64,
    pub in_flight: u64,
    pub reservoir: u64,
    pub waste: u64,
    pub external_input: u64,
    pub external_output: u64,
    pub energy_live: u64,
    pub energy_credited: u64,
    pub energy_maintenance: u64,
    pub energy_disposed: u64,
}

impl From<&Accounts> for Stocks {
    fn from(accounts: &Accounts) -> Self {
        Self {
            field: accounts.field,
            flesh: accounts.flesh,
            in_flight: accounts.in_flight,
            reservoir: accounts.reservoir,
            waste: accounts.waste,
            external_input: accounts.external_input,
            external_output: accounts.external_output,
            energy_live: accounts.energy_live,
            energy_credited: accounts.energy_credited,
            energy_maintenance: accounts.energy_maintenance,
            energy_disposed: accounts.energy_disposed,
        }
    }
}

pub fn apply_receipt(stocks: &mut Stocks, receipt: &TickReceipt) -> Result<(), Error> {
    stocks.external_input = add(
        stocks.external_input,
        receipt.external_input,
        "external_input",
    )?;
    stocks.reservoir = add(stocks.reservoir, receipt.external_input, "reservoir inflow")?;
    stocks.reservoir = sub(
        stocks.reservoir,
        receipt.reservoir_to_field,
        "reservoir injection",
    )?;
    stocks.field = add(stocks.field, receipt.reservoir_to_field, "field injection")?;
    stocks.field = sub(stocks.field, receipt.nutrient_uptaken, "field uptake")?;
    stocks.field = sub(stocks.field, receipt.secreted, "secretion substrate")?;
    stocks.flesh = add(stocks.flesh, receipt.secreted, "deposited flesh")?;
    stocks.reservoir = sub(stocks.reservoir, receipt.trail_laid, "trail condensate")?;
    stocks.flesh = add(stocks.flesh, receipt.trail_laid, "trail flesh")?;
    stocks.flesh = sub(stocks.flesh, receipt.excavated, "excavated flesh")?;
    stocks.field = sub(
        stocks.field,
        receipt.jet_released,
        "reaction mass feedstock",
    )?;
    stocks.flesh = add(stocks.flesh, receipt.jet_deposited, "jet deposition")?;
    stocks.in_flight = add(stocks.in_flight, receipt.jet_released, "jet launch")?;
    stocks.in_flight = sub(
        stocks.in_flight,
        receipt.jet_deposited + receipt.jet_wasted,
        "jet arrival",
    )?;
    stocks.waste = add(stocks.waste, receipt.waste_created, "waste")?;
    stocks.waste = sub(stocks.waste, receipt.external_output, "waste export")?;
    stocks.external_output = add(
        stocks.external_output,
        receipt.external_output,
        "external_output",
    )?;
    stocks.energy_credited = add(
        stocks.energy_credited,
        receipt.energy_credited,
        "energy credit",
    )?;
    stocks.energy_maintenance = add(
        stocks.energy_maintenance,
        receipt.energy_maintenance,
        "energy maintenance",
    )?;
    stocks.energy_disposed = add(
        stocks.energy_disposed,
        receipt.energy_disposed,
        "energy disposed",
    )?;
    stocks.energy_live = add(
        stocks.energy_live,
        receipt.energy_credited,
        "live energy credit",
    )?;
    stocks.energy_live = sub(
        stocks.energy_live,
        receipt.energy_maintenance,
        "live energy maintenance",
    )?;
    stocks.energy_live = sub(
        stocks.energy_live,
        receipt.energy_disposed,
        "live energy disposal",
    )?;
    Ok(())
}

/// Observational telemetry, deliberately absent from snapshots and state hashes.
/// Positions are fixed point (1024 per site); tick is zero based.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct OrganismTick {
    pub id: u64,
    pub tick: u64,
    pub energy_before: u64,
    pub energy_after: u64,
    pub neural: u64,
    pub actuator: u64,
    pub maintenance: u64,
    pub secretion: u64,
    pub digging: u64,
    pub intake: u64,
    pub gain: u64,
    pub birth_out: u64,
    pub death_disposal: u64,
    pub died: bool,
    pub position_before: [i64; 3],
    pub position_after: [i64; 3],
    pub head: Option<Site>,
    pub food_available_at_attempt: u64,
    pub food_site: Option<Site>,
    pub commands: Vec<i32>,
    pub drive: Vec<i32>,
    pub angles: Vec<i32>,
    pub rates: Vec<i32>,
    pub efforts: Vec<i32>,
    pub orientation: [i32; 3],
    pub sensors: Vec<i32>,
}

/// Authoritative world. Mutation of scientific state goes through [`World::tick`].
#[derive(Clone, Debug)]
pub struct World {
    config: SimConfig,
    seed: u64,
    tick: u64,
    cells: BTreeMap<u64, Cell>,
    brains: BTreeMap<u64, BrainRig>,
    flesh: BTreeMap<Site, u64>,
    jets: Vec<crate::neural::body::Jet>,
    environment_momentum: [i64; 3],
    deaths: BTreeMap<u64, DeathRecord>,
    occupancy: Vec<u64>,
    nutrient: Vec<u64>,
    reservoir: u64,
    waste: u64,
    external_input: u64,
    external_output: u64,
    energy_credited: u64,
    energy_maintenance: u64,
    energy_disposed: u64,
    material_initial: u64,
    next_cell_id: u64,
    inject_cursor: u64,
    observations: Option<BTreeMap<u64, OrganismTick>>,
}

impl World {
    pub fn genesis(config: &SimConfig, seed: u64) -> Result<Self, Error> {
        config.validate()?;
        let dims = dims_of(config);
        let volume_u64 = dims.volume()?;
        let volume = usize::try_from(volume_u64).expect("volume fits usize");
        let mut index: Vec<u32> = if let Some(food) = &config.feeding {
            crate::feeding::sites(food, dims)
                .into_iter()
                .map(|mut site| {
                    site.y -= 1;
                    site_index(site, dims).expect("founding food site") as u32
                })
                .collect()
        } else {
            (0..u32::try_from(volume_u64).expect("volume fits u32")).collect()
        };
        let mut rng = placement_rng(seed);
        let mut cursor = index.len();
        while cursor > 1 {
            cursor -= 1;
            let pick =
                unbiased_below(&mut rng, u32::try_from(cursor + 1).expect("index fits")) as usize;
            index.swap(cursor, pick);
        }

        let mut nutrient = Vec::with_capacity(volume);
        let mut field_sum = 0u64;
        for z in 0..config.depth {
            for y in 0..config.height {
                for x in 0..config.width {
                    let amount = config
                        .initial_nutrient_per_site
                        .checked_add(lattice_wave(x, y, z, config.nutrient_wave_amplitude))
                        .ok_or_else(|| Error::Overflow("initial nutrient".into()))?;
                    if amount > config.site_cap {
                        return Err(Error::Config(format!(
                            "initial nutrient {amount} at ({x},{y},{z}) exceeds site_cap {}",
                            config.site_cap
                        )));
                    }
                    field_sum = add(field_sum, amount, "initial field")?;
                    nutrient.push(amount);
                }
            }
        }

        let mut cells = BTreeMap::new();
        if let Some(food) = &config.feeding {
            for site in crate::feeding::sites(food, dims) {
                nutrient[site_index(site, dims)?] = food.initial_stock;
                field_sum = add(field_sum, food.initial_stock, "initial nursery food")?;
            }
        }
        for n in 0..config.initial_population {
            let id = u64::from(n) + 1;
            let site = site_from_index(index[n as usize], dims)?;
            let genome = match config.genome_fill {
                GenomeFill::Zero => [0u8; 8],
                GenomeFill::Derived => {
                    let left = rng.next_u32().to_le_bytes();
                    let right = rng.next_u32().to_le_bytes();
                    let mut bytes = [0u8; 8];
                    bytes[..4].copy_from_slice(&left);
                    bytes[4..].copy_from_slice(&right);
                    bytes
                }
            };
            cells.insert(
                id,
                Cell {
                    id,
                    site,
                    energy: config.initial_energy,
                    age: 0,
                    genome,
                },
            );
        }
        let energy_credited = u64::from(config.initial_population)
            .checked_mul(config.initial_energy)
            .ok_or_else(|| Error::Overflow("seed action-energy".into()))?;
        let material_initial = add(
            add(field_sum, config.initial_reservoir, "initial reservoir")?,
            config.initial_waste,
            "initial waste",
        )?;
        let brains = if config.farm.is_some() {
            cells
                .values()
                .map(|cell| {
                    let mut brain = BrainRig::new(cell, seed);
                    if config.neural.is_some() {
                        brain.neural = Some(crate::neural::NeuralState::new(
                            crate::neural::ControllerGenome::founder(
                                seed,
                                cell.id,
                                crate::neural::schema::Anatomy::reference(),
                            ),
                            cell.energy,
                        ));
                    }
                    (cell.id, brain)
                })
                .collect()
        } else {
            BTreeMap::new()
        };
        let mut world = Self {
            config: config.clone(),
            brains,
            flesh: BTreeMap::new(),
            jets: Vec::new(),
            environment_momentum: [0; 3],
            seed,
            tick: 0,
            cells,
            deaths: BTreeMap::new(),
            occupancy: vec![0; volume],
            nutrient,
            reservoir: config.initial_reservoir,
            waste: config.initial_waste,
            external_input: 0,
            external_output: 0,
            energy_credited,
            energy_maintenance: 0,
            energy_disposed: 0,
            material_initial,
            next_cell_id: u64::from(config.initial_population) + 1,
            inject_cursor: 0,
            observations: None,
        };
        world.rebuild_occupancy()?;
        world.audit()?;
        Ok(world)
    }

    pub fn restore(
        config: &SimConfig,
        snapshot: &crate::snapshot::Snapshot,
    ) -> Result<Self, Error> {
        config.validate()?;
        if snapshot.neural_format != config.neural.is_some() {
            return Err(Error::Snapshot(
                "neural snapshot mode does not match configuration".into(),
            ));
        }
        if snapshot.width != config.width
            || snapshot.height != config.height
            || snapshot.depth != config.depth
        {
            return Err(Error::Snapshot(format!(
                "snapshot grid {}×{}×{} does not match config {}×{}×{}",
                snapshot.width,
                snapshot.height,
                snapshot.depth,
                config.width,
                config.height,
                config.depth
            )));
        }
        let mut cells = BTreeMap::new();
        for cell in &snapshot.cells {
            if cells.insert(cell.id, cell.clone()).is_some() {
                return Err(Error::Snapshot(format!("duplicate cell {}", cell.id)));
            }
        }
        let mut deaths = BTreeMap::new();
        for death in &snapshot.deaths {
            if deaths.insert(death.id, death.clone()).is_some() {
                return Err(Error::Snapshot(format!("duplicate death {}", death.id)));
            }
        }
        let volume = dims_of(config).volume()? as usize;
        let mut world = Self {
            config: config.clone(),
            flesh: snapshot
                .flesh
                .iter()
                .map(|deposit| (deposit.site, deposit.amount))
                .collect(),
            jets: snapshot.jets.clone(),
            environment_momentum: snapshot.environment_momentum,
            seed: snapshot.seed,
            brains: snapshot
                .brains
                .iter()
                .map(|brain| (brain.id, brain.clone()))
                .collect(),
            tick: snapshot.tick,
            cells,
            deaths,
            occupancy: vec![0; volume],
            nutrient: snapshot.nutrient.clone(),
            reservoir: snapshot.reservoir,
            waste: snapshot.waste,
            external_input: snapshot.external_input,
            external_output: snapshot.external_output,
            energy_credited: snapshot.energy_credited,
            energy_maintenance: snapshot.energy_maintenance,
            energy_disposed: snapshot.energy_disposed,
            material_initial: snapshot.material_initial,
            next_cell_id: snapshot.next_cell_id,
            inject_cursor: snapshot.inject_cursor,
            observations: None,
        };
        if world.brains.len() != snapshot.brains.len() || world.flesh.len() != snapshot.flesh.len()
        {
            return Err(Error::Snapshot("duplicate brain or flesh entry".into()));
        }
        world.rebuild_occupancy()?;
        world.audit()?;
        Ok(world)
    }

    pub fn tick(&mut self) -> Result<TickReceipt, Error> {
        let mut next = self.clone();
        let receipt = next.advance()?;
        next.audit()?;
        *self = next;
        Ok(receipt)
    }

    /// Identical scientific transition to `tick`, with optional external telemetry.
    pub fn tick_observed(&mut self) -> Result<(TickReceipt, Vec<OrganismTick>), Error> {
        let mut next = self.clone();
        next.observations = Some(BTreeMap::new());
        let receipt = next.advance()?;
        next.audit()?;
        let rows = next.observations.take().unwrap().into_values().collect();
        *self = next;
        Ok((receipt, rows))
    }

    pub fn config(&self) -> &SimConfig {
        &self.config
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn tick_index(&self) -> u64 {
        self.tick
    }

    pub fn live_count(&self) -> u32 {
        u32::try_from(self.cells.len()).expect("population fits u32")
    }

    pub fn dead_count(&self) -> u32 {
        u32::try_from(self.deaths.len()).expect("deaths fit u32")
    }

    pub fn cells(&self) -> impl Iterator<Item = &Cell> {
        self.cells.values()
    }

    pub fn jets(&self) -> &[crate::neural::body::Jet] {
        &self.jets
    }
    pub fn environment_momentum(&self) -> [i64; 3] {
        self.environment_momentum
    }

    pub fn flesh(&self) -> &BTreeMap<Site, u64> {
        &self.flesh
    }

    pub fn brains(&self) -> &BTreeMap<u64, BrainRig> {
        &self.brains
    }

    pub fn deaths(&self) -> impl Iterator<Item = &DeathRecord> {
        self.deaths.values()
    }

    pub fn nutrient(&self) -> &[u64] {
        &self.nutrient
    }

    pub fn occupancy(&self) -> &[u64] {
        &self.occupancy
    }

    pub fn dims(&self) -> Dims {
        dims_of(&self.config)
    }

    pub fn accounts(&self) -> Result<Accounts, Error> {
        let mut field = 0u64;
        for amount in &self.nutrient {
            field = add(field, *amount, "field sum")?;
        }
        let mut energy_live = 0u64;
        for cell in self.cells.values() {
            energy_live = add(energy_live, cell.energy, "live energy sum")?;
        }
        Ok(Accounts {
            material_initial: self.material_initial,
            external_input: self.external_input,
            external_output: self.external_output,
            field,
            in_flight: self.jets.len() as u64,
            flesh: self
                .flesh
                .values()
                .try_fold(0u64, |sum, amount| add(sum, *amount, "flesh stock"))?,
            reservoir: self.reservoir,
            waste: self.waste,
            energy_live,
            energy_credited: self.energy_credited,
            energy_maintenance: self.energy_maintenance,
            energy_disposed: self.energy_disposed,
        })
    }

    pub fn snapshot(&self) -> crate::snapshot::Snapshot {
        crate::snapshot::Snapshot {
            tick: self.tick,
            seed: self.seed,
            width: self.config.width,
            height: self.config.height,
            depth: self.config.depth,
            reservoir: self.reservoir,
            waste: self.waste,
            external_input: self.external_input,
            external_output: self.external_output,
            energy_credited: self.energy_credited,
            energy_maintenance: self.energy_maintenance,
            energy_disposed: self.energy_disposed,
            material_initial: self.material_initial,
            next_cell_id: self.next_cell_id,
            inject_cursor: self.inject_cursor,
            cells: self.cells.values().cloned().collect(),
            deaths: self.deaths.values().cloned().collect(),
            nutrient: self.nutrient.clone(),
            brains: self.brains.values().cloned().collect(),
            jets: self.jets.clone(),
            environment_momentum: self.environment_momentum,
            neural_format: self.config.neural.is_some(),
            flesh: self
                .flesh
                .iter()
                .map(|(site, amount)| crate::farm::FleshDeposit {
                    site: *site,
                    amount: *amount,
                })
                .collect(),
        }
    }

    pub fn trail_points(&self) -> Vec<crate::layers::TrailPoint> {
        self.cells
            .values()
            .map(|cell| crate::layers::TrailPoint {
                tick: self.tick,
                id: cell.id,
                x: cell.site.x,
                y: cell.site.y,
                z: cell.site.z,
            })
            .collect()
    }

    /// Tick law, in order:
    /// 1. External inflow is added to the reservoir or the tick is rejected.
    /// 2. Reservoir injection walks at most one full sweep, at most one unit per site.
    /// 3. Movement intents are drawn from independent per-cell streams.
    /// 4. Contention is resolved by [`resolve_moves`].
    /// 5. A flesh unit beside a nursery clot becomes one nutrient on that clot.
    /// 6. Uptake converts site nutrient into waste 1:1 and credits action-energy.
    /// 7. Maintenance is paid in full or the cell dies and its action-energy is disposed.
    /// 8. Waste is exported up to the available stock.
    /// 9. The committed tick index increments.
    fn advance(&mut self) -> Result<TickReceipt, Error> {
        let tick = self.tick;
        let inflow = self.config.inflow_per_tick;
        if inflow > 0 {
            let updated = add(self.reservoir, inflow, "reservoir inflow")?;
            if updated > self.config.reservoir_cap {
                return Err(Error::Run(format!(
                    "tick {tick}: reservoir {updated} would exceed reservoir_cap {} after inflow {inflow}; tick not committed",
                    self.config.reservoir_cap
                )));
            }
            self.reservoir = updated;
            self.external_input = add(self.external_input, inflow, "external_input")?;
        }

        let budget = self.config.inject_per_tick.min(self.reservoir);
        let injected = self.inject(budget)?;
        self.reservoir = sub(self.reservoir, injected, "reservoir after injection")?;

        let dims = self.dims();
        let (jet_deposited, jet_wasted) = self.advance_jets()?;
        let solid = fused_surface(&self.flesh, dims)?;
        let mut proposals = Vec::with_capacity(self.cells.len());
        let mut decisions = BTreeMap::new();
        if let Some(rows) = &mut self.observations {
            for cell in self.cells.values() {
                let offset = self
                    .brains
                    .get(&cell.id)
                    .map(|b| b.offset)
                    .unwrap_or([0; 3]);
                let coords = [cell.site.x, cell.site.y, cell.site.z];
                let position =
                    std::array::from_fn(|a| i64::from(coords[a]) * 1024 + i64::from(offset[a]));
                rows.insert(
                    cell.id,
                    OrganismTick {
                        id: cell.id,
                        tick,
                        energy_before: cell.energy,
                        position_before: position,
                        ..OrganismTick::default()
                    },
                );
            }
        }
        let mut motor_energy = 0;
        let mut neural_energy = 0;
        for cell in self.cells.values() {
            let neural_sensors = self
                .brains
                .get(&cell.id)
                .and_then(|brain| brain.neural.as_ref())
                .map(|state| self.observe_neural(cell, state, &solid));
            let to = if let Some(brain) = self.brains.get_mut(&cell.id) {
                if let Some(state) = &mut brain.neural {
                    let rules = self.config.neural.as_ref().expect("neural rules");
                    let before = brain.offset;
                    let (neural_fee, motor_fee) = state.integrate(
                        neural_sensors.expect("sensors"),
                        rules,
                        self.seed,
                        cell.id,
                        tick,
                        cell.energy,
                    );
                    neural_energy += neural_fee;
                    if let Some(rows) = &mut self.observations {
                        let row = rows.get_mut(&cell.id).unwrap();
                        row.neural = neural_fee;
                        row.actuator = motor_fee;
                    }
                    let mut loads = vec![0; state.genome.anatomy.axes.len()];
                    let base = state.sensors.len() - 2;
                    let mut jaw = 0;
                    for (index, binding) in state.genome.anatomy.axes.iter().enumerate() {
                        if binding.axis == crate::neural::schema::Axis::Jaw {
                            loads[index] = state.sensors[base + jaw];
                            jaw += 1;
                        }
                    }
                    let drive = state.actuator_drive().to_vec();
                    let stiffness = state.genome.ligament_stiffness();
                    let rest = state.genome.ligament_rest.clone();
                    let absorbed = state.body.step_with_law(
                        &state.genome.anatomy,
                        &drive,
                        &state.genome.strengths,
                        rules.drag,
                        &loads,
                        stiffness,
                        &rest,
                        rules.version,
                    );
                    for (axis, value) in absorbed.into_iter().enumerate() {
                        self.environment_momentum[axis] += value;
                    }
                    state.realized_efforts.fill(0);
                    let count = state.body.efforts.len();
                    state.realized_efforts[..count].clone_from_slice(&state.body.efforts);
                    for (index, rate) in state.body.rates.iter().enumerate() {
                        state.participation[index] += u64::from(*rate != 0);
                    }
                    let coordinates = [cell.site.x, cell.site.y, cell.site.z];
                    let extents = [dims.width, dims.height, dims.depth];
                    let next: [u32; 3] = std::array::from_fn(|axis| {
                        let position =
                            i64::from(before[axis]) + i64::from(state.body.velocity[axis]);
                        let crossing = (position + 512).div_euclid(1024);
                        brain.offset[axis] = (position - crossing * 1024) as i32;
                        (i64::from(coordinates[axis]) + crossing)
                            .rem_euclid(i64::from(extents[axis])) as u32
                    });
                    // The legacy tuple is bookkeeping only; no primitive is chosen.
                    let mut old = brain.clone();
                    old.offset = before;
                    decisions.insert(cell.id, (0, [0; 5], neural_fee + motor_fee, old));
                    brain.updates += 1;
                    brain.sync_neural_pose();
                    Site {
                        x: next[0],
                        y: next[1],
                        z: next[2],
                    }
                } else {
                    let (action, features) = brain.choose(
                        cell,
                        &self.nutrient,
                        &self.occupancy,
                        &self.flesh,
                        dims,
                        (self.seed, tick),
                    )?;
                    let before = brain.clone();
                    // An empty energy balance cannot actuate a motor.
                    let action = if cell.energy == 0 { 0 } else { action };
                    let (to, cost) = brain.actuate(action, cell.site, dims)?;
                    decisions.insert(cell.id, (action, features, cost, before));
                    to
                }
            } else {
                movement_target(self.seed, tick, cell.id, cell.site, dims)?.1
            };
            proposals.push(MoveProposal {
                id: cell.id,
                from: cell.site,
                to,
            });
        }
        // A mature deposit is a soft contact surface. It arrests a crossing;
        // contact reverses and damps momentum, instead of erasing its existence.
        let flesh_blocked: std::collections::BTreeSet<_> = proposals
            .iter()
            .filter(|p| p.to != p.from && solid.contains(&p.to))
            .map(|p| p.id)
            .collect();
        let resolved: Vec<_> = proposals
            .iter()
            .map(|p| {
                if flesh_blocked.contains(&p.id) {
                    MoveProposal {
                        id: p.id,
                        from: p.from,
                        to: p.from,
                    }
                } else {
                    *p
                }
            })
            .collect();
        let outcomes = resolve_moves(&resolved)?;
        let (moves_applied, moves_blocked) = apply_moves(&mut self.cells, &outcomes)?;
        self.rebuild_occupancy()?;
        for proposal in &proposals {
            if let Some((_, _, cost, before)) = decisions.get(&proposal.id) {
                let brain = self.brains.get_mut(&proposal.id).expect("brain present");
                if self.cells[&proposal.id].site != proposal.to {
                    brain.offset = before.offset;
                    brain.velocity = brain.velocity.map(|value| -value / 4);
                    if let Some(state) = &mut brain.neural {
                        for axis in 0..3 {
                            let previous_velocity = state.body.velocity[axis];
                            let old = state.body.momentum[axis];
                            state.body.momentum[axis] = -old / 4;
                            self.environment_momentum[axis] += old - state.body.momentum[axis];
                            state.body.velocity[axis] =
                                (state.body.momentum[axis] / crate::neural::body::INERTIA) as i32;
                            state.body.acceleration[axis] +=
                                state.body.velocity[axis] - previous_velocity;
                        }
                        brain.sync_neural_pose();
                    }
                }
                let cell = self.cells.get_mut(&proposal.id).expect("moving cell");
                cell.energy = sub(cell.energy, *cost, "motor effort")?;
                motor_energy = add(motor_energy, *cost, "motor energy")?;
                brain.last_meal = 0;
                brain.last_dig = 0;
            }
        }

        // A crossing leaves one unit of flesh on the site just vacated. The
        // material comes from the reservoir, so the thread stays in the ledger
        // and does not fade. Four linked units still fuse into a soft wall.
        let mut trail_laid = 0u64;
        if self.config.neural.is_some() {
            for outcome in &outcomes {
                if !outcome.moved || self.reservoir == 0 {
                    continue;
                }
                let stock = self.flesh.get(&outcome.from).copied().unwrap_or(0);
                if stock >= 64 || (self.flesh.len() >= 8192 && stock == 0) {
                    continue;
                }
                self.reservoir = sub(self.reservoir, 1, "trail condensate")?;
                self.flesh.insert(outcome.from, stock + 1);
                trail_laid = add(trail_laid, 1, "trail units")?;
            }
        }
        // A flesh unit beside a clot becomes one nutrient on that clot.
        // Threads elsewhere stay. The organism can keep laying beside food.
        self.compost()?;

        let ids: Vec<u64> = self.cells.keys().copied().collect();
        let mut uptaken = 0u64;
        let mut credited = 0u64;
        for id in &ids {
            let (site, energy) = {
                let cell = &self.cells[id];
                (self.feeding_site(cell), cell.energy)
            };
            let index = site_index(site, dims)?;
            let head = self
                .brains
                .get(id)
                .and_then(|b| b.neural.as_ref())
                .map(|state| self.neural_jaw_target(&self.cells[id], state));
            if let Some(rows) = &mut self.observations {
                let row = rows.get_mut(id).unwrap();
                row.head = head;
                row.food_available_at_attempt = self.nutrient[index];
            }
            let units = uptake_units(
                self.nutrient[index],
                self.config.uptake_per_tick,
                energy,
                self.config.energy_cap,
                self.config.energy_per_nutrient,
            );
            if let Some(brain) = self.brains.get_mut(id) {
                brain.last_meal = units;
                if self.config.feeding.is_some()
                    && let Some(n) = &mut brain.neural
                {
                    n.last_food_site = if units > 0 { Some(site) } else { None };
                }
            }
            if units == 0 {
                continue;
            }
            let gain = units
                .checked_mul(self.config.energy_per_nutrient)
                .ok_or_else(|| Error::Overflow(format!("energy credit for cell {id}")))?;
            if let Some(rows) = &mut self.observations {
                let row = rows.get_mut(id).unwrap();
                row.intake = units;
                row.gain = gain;
                row.food_site = Some(site);
            }
            self.nutrient[index] = sub(self.nutrient[index], units, "site nutrient")?;
            self.waste = add(self.waste, units, "waste from uptake")?;
            self.cells
                .get_mut(id)
                .expect("cell remains during uptake")
                .energy = add(energy, gain, "cell energy")?;
            uptaken = add(uptaken, units, "uptake total")?;
            credited = add(credited, gain, "credit total")?;
        }
        self.energy_credited = add(self.energy_credited, credited, "energy_credited")?;

        let mut jet_released = 0;
        let mut secretion_paid = std::collections::BTreeSet::new();
        let mut secreted = 0u64;
        let mut secretion_cost = 0u64;
        for (id, (action, _, _, _)) in &decisions {
            if self.brains[id].neural.is_some() {
                continue;
            }
            if *action != SECRETE_ACTION {
                continue;
            }
            let cell = self.cells.get_mut(id).expect("secreting cell");
            let index = site_index(cell.site, dims)?;
            let stock = self.flesh.get(&cell.site).copied().unwrap_or(0);
            if cell.energy >= 2
                && self.nutrient[index] > 0
                && stock < 64
                && (self.flesh.len() < 8192 || stock > 0)
            {
                self.nutrient[index] -= 1;
                self.flesh.insert(cell.site, stock + 1);
                cell.energy -= 2;
                secretion_paid.insert(*id);
                secreted += 1;
                secretion_cost += 2;
            }
        }
        for id in decisions.keys() {
            let source = self.feeding_site(&self.cells[id]);
            let Some(state) = self.brains.get_mut(id).and_then(|b| b.neural.as_mut()) else {
                continue;
            };
            let cell = &self.cells[id];
            let count = state.genome.anatomy.axes.len();
            let drive = state.actuator_drive();
            // The spinal wave does not write this channel. A resting flicker
            // stays shut. A saturated open, command near the squashed ceiling,
            // spends one nutrient as a droplet.
            let valve = drive[count];
            if valve > 480 {
                state.valve_accumulator +=
                    (i64::from(valve) * i64::from(state.genome.strengths[count]) / 1024).min(1024)
                        as i32;
            }
            if state.valve_accumulator < 1024 {
                continue;
            }
            state.valve_accumulator -= 1024;
            let index = site_index(source, dims)?;
            let drive = state.actuator_drive();
            let local = std::array::from_fn(|axis| {
                (i64::from(drive[count + 1 + axis])
                    * i64::from(state.genome.strengths[count + 1 + axis])
                    / 4096)
                    .clamp(-512, 512)
            });
            let momentum = crate::neural::body::rotate(local, state.body.orientation);
            let opposite = momentum.map(|x| -x);
            if cell.energy >= 2
                && self.nutrient[index] > 0
                && self.jets.len() < 4096
                && state.body.can_impulse(opposite)
            {
                let coordinates = [cell.site.x, cell.site.y, cell.site.z];
                let extents = [dims.width, dims.height, dims.depth];
                let offset = self.brains[id].offset;
                let state = self
                    .brains
                    .get_mut(id)
                    .and_then(|b| b.neural.as_mut())
                    .expect("neural state");
                state.body.impulse(opposite);
                state.realized_efforts[count] = 1024;
                for axis in 0..3 {
                    state.realized_efforts[count + 1 + axis] = state.commands[count + 1 + axis];
                }
                self.jets.push(crate::neural::body::Jet {
                    owner: *id,
                    position: std::array::from_fn(|axis| {
                        (i64::from(coordinates[axis]) * 1024 + i64::from(offset[axis]))
                            .rem_euclid(i64::from(extents[axis]) * 1024)
                    }),
                    momentum,
                    life: 4,
                });
                self.nutrient[index] -= 1;
                self.cells.get_mut(id).expect("secreting cell").energy -= 2;
                secretion_paid.insert(*id);
                secretion_cost += 2;
                jet_released += 1;
                self.brains.get_mut(id).expect("brain").sync_neural_pose();
            }
        }
        let mut excavated = 0u64;
        let mut digging_paid = std::collections::BTreeSet::new();
        let mut digging_cost = 0u64;
        for (id, (action, _, _, _)) in &decisions {
            if self.brains[id].neural.is_some() {
                continue;
            }
            if *action < DIG_START || self.brains[id].joints[SPINE_JOINTS] > 0 {
                continue;
            }
            let cell = self.cells.get_mut(id).expect("digging cell");
            let target = crate::step_site(cell.site, action_dir(*action)?, dims)?;
            let amount = self.flesh.get(&target).copied().unwrap_or(0);
            if amount == 0 || cell.energy < 1 {
                continue;
            }
            // Chewed tissue becomes waste rather than disappearing or creating food.
            let removed = amount.min(2);
            if amount == removed {
                self.flesh.remove(&target);
            } else {
                self.flesh.insert(target, amount - removed);
            }
            self.waste = add(self.waste, removed, "excavated tissue")?;
            cell.energy -= 1;
            digging_paid.insert(*id);
            digging_cost += 1;
            excavated += removed;
            self.brains.get_mut(id).expect("digging brain").last_dig = removed;
        }
        for id in decisions.keys() {
            let Some(state) = self.brains[id].neural.as_ref() else {
                continue;
            };
            let jaw_closed = state
                .genome
                .anatomy
                .axes
                .iter()
                .enumerate()
                .filter(|(_, b)| b.axis == crate::neural::schema::Axis::Jaw)
                .any(|(i, _)| {
                    state.body.rates[i] < 0 && state.body.loads[i] > 0 && state.body.efforts[i] < 0
                });
            if !jaw_closed || self.cells[id].energy < 1 {
                continue;
            }
            let target = self.neural_jaw_target(&self.cells[id], state);
            let amount = self.flesh.get(&target).copied().unwrap_or(0);
            if amount == 0 {
                continue;
            }
            let removed = amount.min(2);
            if amount == removed {
                self.flesh.remove(&target);
            } else {
                self.flesh.insert(target, amount - removed);
            }
            self.waste = add(self.waste, removed, "neural jaw excavation")?;
            self.cells.get_mut(id).expect("cell").energy -= 1;
            self.brains.get_mut(id).expect("brain").last_dig = removed;
            digging_paid.insert(*id);
            digging_cost += 1;
            excavated += removed;
        }
        // The tuple paid both charges; expose actuator and neural fees separately.
        motor_energy -= neural_energy;
        let mut maintained = motor_energy + neural_energy;
        let mut disposed = secretion_cost + digging_cost;
        let mut deaths = 0u32;
        let cost = self.config.maintenance_cost;
        for id in ids {
            let energy = self.cells[&id].energy;
            if let Some(rows) = &mut self.observations {
                let row = rows.get_mut(&id).unwrap();
                row.secretion = if secretion_paid.contains(&id) { 2 } else { 0 };
                row.digging = u64::from(digging_paid.contains(&id));
                row.maintenance = if energy >= cost { cost } else { 0 };
                row.died = energy < cost;
                row.death_disposal = if row.died { energy } else { 0 };
                let cell = &self.cells[&id];
                let coords = [cell.site.x, cell.site.y, cell.site.z];
                let offset = self.brains.get(&id).map(|b| b.offset).unwrap_or([0; 3]);
                row.position_after =
                    std::array::from_fn(|a| i64::from(coords[a]) * 1024 + i64::from(offset[a]));
                if let Some(state) = self.brains.get(&id).and_then(|b| b.neural.as_ref()) {
                    row.commands = state.commands.clone();
                    row.drive = state.actuator_drive().to_vec();
                    row.angles = state.body.angles.clone();
                    row.rates = state.body.rates.clone();
                    row.efforts = state.body.efforts.clone();
                    row.orientation = state.body.orientation;
                    row.sensors = state.sensors.clone();
                } else {
                    row.actuator = decisions.get(&id).map(|d| d.2).unwrap_or(0);
                }
            }
            if let Some((action, features, motor_cost, _)) = decisions.get(&id)
                && self.brains[&id].neural.is_none()
            {
                let brain = self.brains.get_mut(&id).expect("learning brain");
                let reward = (i128::from(brain.last_meal)
                    * i128::from(self.config.energy_per_nutrient)
                    - i128::from(cost)
                    - i128::from(*motor_cost)
                    - if secretion_paid.contains(&id) { 2 } else { 0 }
                    - if digging_paid.contains(&id) { 1 } else { 0 })
                .clamp(-16, 16) as i16;
                brain.learn(*action, *features, reward);
            }
            if energy >= cost {
                let cell = self.cells.get_mut(&id).expect("survivor present");
                cell.energy = sub(energy, cost, "maintenance")?;
                cell.age = add(cell.age, 1, "age")?;
                maintained = add(maintained, cost, "maintenance total")?;
            } else {
                let cell = self.cells.remove(&id).expect("dying cell present");
                if let Some(state) = self.brains.get(&id).and_then(|brain| brain.neural.as_ref()) {
                    for axis in 0..3 {
                        self.environment_momentum[axis] += state.body.momentum[axis];
                    }
                }
                self.brains.remove(&id);
                disposed = add(disposed, cell.energy, "disposal")?;
                self.deaths.insert(
                    id,
                    DeathRecord {
                        id,
                        site: cell.site,
                        energy_disposed: cell.energy,
                        age: cell.age,
                        during_tick: tick,
                        genome: cell.genome,
                    },
                );
                deaths += 1;
            }
        }
        let (births, birth_cost) = self.reproduce()?;
        if let Some(rules) = &self.config.neural {
            for (id, brain) in &mut self.brains {
                if let Some(state) = &mut brain.neural {
                    state.feedback(self.cells[id].energy, brain.last_meal, rules);
                }
            }
        }
        if let Some(rows) = &mut self.observations {
            for row in rows.values_mut() {
                row.energy_after = self.cells.get(&row.id).map(|c| c.energy).unwrap_or(0);
                let remaining = row.energy_before + row.gain
                    - row.neural
                    - row.actuator
                    - row.secretion
                    - row.digging
                    - row.maintenance
                    - row.death_disposal;
                row.birth_out = remaining - row.energy_after;
            }
        }
        disposed = add(disposed, birth_cost, "birth energy disposed")?;
        self.energy_maintenance = add(self.energy_maintenance, maintained, "energy_maintenance")?;
        self.energy_disposed = add(self.energy_disposed, disposed, "energy_disposed")?;
        self.rebuild_occupancy()?;

        let exported = self.config.export_per_tick.min(self.waste);
        self.waste = sub(self.waste, exported, "waste export")?;
        self.external_output = add(self.external_output, exported, "external_output")?;
        self.tick = add(self.tick, 1, "tick")?;

        Ok(TickReceipt {
            tick_after: self.tick,
            external_input: inflow,
            reservoir_to_field: injected,
            nutrient_uptaken: uptaken,
            waste_created: uptaken + excavated + jet_wasted,
            energy_credited: credited,
            energy_maintenance: maintained,
            energy_disposed: disposed,
            external_output: exported,
            deaths,
            moves_applied,
            moves_blocked,
            births,
            motor_energy,
            neural_energy,
            jet_released,
            jet_deposited,
            jet_wasted,
            secreted,
            excavated,
            trail_laid,
        })
    }

    fn advance_jets(&mut self) -> Result<(u64, u64), Error> {
        let dims = self.dims();
        let solid = fused_surface(&self.flesh, dims)?;
        let curing = self
            .config
            .neural
            .as_ref()
            .is_some_and(|rules| rules.drag > 0);
        let mut deposited = 0;
        let mut wasted = 0;
        for mut jet in std::mem::take(&mut self.jets) {
            jet.advance(dims);
            let site = jet.site(dims);
            let occupant = self.occupancy[site_index(site, dims)?];
            let stopped = solid.contains(&site)
                || (occupant != 0 && occupant != jet.owner)
                || (curing && jet.life == 0);
            if stopped {
                for axis in 0..3 {
                    self.environment_momentum[axis] += jet.momentum[axis];
                }
                let stock = self.flesh.get(&site).copied().unwrap_or(0);
                if stock < 64 && (self.flesh.len() < 8192 || stock > 0) {
                    self.flesh.insert(site, stock + 1);
                    deposited += 1;
                } else {
                    self.waste = add(self.waste, 1, "saturated jet destination")?;
                    wasted += 1;
                }
            } else {
                self.jets.push(jet);
            }
        }
        Ok((deposited, wasted))
    }
    fn feeding_site(&self, cell: &Cell) -> Site {
        if self.config.feeding.is_none() {
            return cell.site;
        }
        let state = self.brains[&cell.id]
            .neural
            .as_ref()
            .expect("nursery brain");
        let head = self.neural_jaw_target(cell, state);
        crate::feeding::mouth_sites(head, self.dims())
            .into_iter()
            .find(|site| self.nutrient[site_index(*site, self.dims()).expect("mouth site")] > 0)
            .unwrap_or(head)
    }

    fn neural_jaw_target(&self, cell: &Cell, state: &crate::neural::NeuralState) -> Site {
        state.body.head_site(
            &state.genome.anatomy,
            cell.site,
            self.brains[&cell.id].offset,
            self.dims(),
        )
    }
    fn observe_neural(
        &self,
        cell: &Cell,
        state: &crate::neural::NeuralState,
        solid: &std::collections::BTreeSet<Site>,
    ) -> Vec<i32> {
        let dims = self.dims();
        let current = self.nutrient[site_index(cell.site, dims).expect("cell site")];
        let feedstock =
            self.nutrient[site_index(self.feeding_site(cell), dims).expect("feeding site")];
        let mut chemical = [0; 6];
        let mut contact = [0; 6];
        for face in 0..6 {
            let mut local = [0i64; 3];
            local[face / 2] = if face % 2 == 0 { 1024 } else { -1024 };
            let direction = crate::neural::body::rotate(local, state.body.orientation);
            let axis = (0..3)
                .max_by_key(|axis| direction[*axis].abs())
                .expect("axis");
            let roll = axis * 2 + if direction[axis] >= 0 { 1 } else { 2 };
            let site = crate::step_site(
                cell.site,
                crate::Dir::from_roll(roll as u32).expect("face"),
                dims,
            )
            .expect("neighbor");
            // Eight steps of falloff, not a map. A clot several sites away can
            // change one body-facing channel. The channel is silent in the gap
            // beyond that reach.
            let ahead =
                self.ray_scent(cell.site, crate::Dir::from_roll(roll as u32).expect("face"));
            let scale = i128::from(self.config.site_cap.max(1)) * 8;
            chemical[face] = (i128::from(ahead) * 1024 / scale).clamp(0, 1024) as i32;
            let index = site_index(site, dims).expect("neighbor index");
            contact[face] = if solid.contains(&site)
                || (self.occupancy[index] != 0 && self.occupancy[index] != cell.id)
            {
                1024
            } else {
                0
            };
        }
        let target = self.neural_jaw_target(cell, state);
        let resistance = (self.flesh.get(&target).copied().unwrap_or(0).min(8) * 128) as i32;
        crate::neural::sensing::encode(
            state,
            crate::neural::sensing::Observation {
                energy: cell.energy,
                energy_cap: if self.config.feeding.is_some() {
                    self.config
                        .farm
                        .as_ref()
                        .expect("farm")
                        .birth_energy
                        .saturating_mul(2)
                        .max(1)
                } else {
                    self.config.energy_cap
                },
                nutrient: if self.config.feeding.is_some() {
                    feedstock
                } else {
                    current
                },
                site_cap: self.config.site_cap,
                chemical,
                contact,
                jaw_load: [resistance; 2],
            },
            self.config.neural.as_ref().expect("neural law"),
        )
    }

    /// Weighted nutrient along one facing, steps 1 through 8. Nearer stock
    /// counts more. Zero past that reach.
    fn ray_scent(&self, start: Site, dir: crate::Dir) -> i64 {
        let dims = self.dims();
        let mut site = start;
        let mut total = 0i64;
        for step in 1..=8 {
            site = match crate::step_site(site, dir, dims) {
                Ok(site) => site,
                Err(_) => break,
            };
            let index = match site_index(site, dims) {
                Ok(index) => index,
                Err(_) => break,
            };
            let amount = self.nutrient[index] as i64;
            if amount > 0 {
                total += amount * i64::from(9 - step);
            }
        }
        total
    }

    fn reproduce(&mut self) -> Result<(u32, u64), Error> {
        let Some(rules) = self.config.farm.clone() else {
            return Ok((0, 0));
        };
        // Rebuild after deaths before checking empty sites.
        self.rebuild_occupancy()?;
        let ids: Vec<_> = self.cells.keys().copied().collect();
        let dims = self.dims();
        let solid = fused_surface(&self.flesh, dims)?;
        let mut births = 0;
        let mut disposed = 0;
        for id in ids {
            if self.cells.len() >= rules.max_population as usize {
                break;
            }
            let parent = &self.cells[&id];
            let brain = &self.brains[&id];
            if parent.energy < rules.birth_energy
                || parent.age < rules.maturity_ticks
                || self.tick.saturating_sub(brain.last_birth) < rules.birth_cooldown
            {
                continue;
            }
            let start = (id.wrapping_add(self.tick) % 6) as u32;
            let mut destination = None;
            for n in 0..6 {
                let dir = crate::Dir::from_roll((start + n) % 6 + 1)?;
                let site = crate::step_site(parent.site, dir, dims)?;
                if self.occupancy[site_index(site, dims)?] == 0 && !solid.contains(&site) {
                    destination = Some(site);
                    break;
                }
            }
            let Some(site) = destination else {
                continue;
            };
            let child_id = self.next_cell_id;
            self.next_cell_id = add(child_id, 1, "next cell id")?;
            let mut child = parent.clone();
            child.id = child_id;
            child.site = site;
            child.age = 0;
            child.energy = (parent.energy - rules.birth_cost) / 2;
            // A small inheritable controller mutation; display genome is inherited.
            let mut child_brain = brain.child(child_id, self.seed, self.tick);
            if let Some(state) = &brain.neural {
                if let Some(site) = state.last_food_site {
                    site_index(site, self.dims())?;
                    if self.config.feeding.is_none() {
                        return Err(Error::Invariant(
                            "oral feeding record outside nursery mode".into(),
                        ));
                    }
                }
                let mut offspring = state.offspring(
                    self.seed,
                    child_id,
                    self.tick,
                    child.energy,
                    self.config.neural.as_ref().expect("rules").mutation,
                );
                offspring.body.orientation = state.body.orientation;
                offspring.body.impulse(state.body.momentum.map(|p| p / 2));
                child_brain.neural = Some(offspring);
                child_brain.sync_neural_pose();
            }
            let parent = self.cells.get_mut(&id).expect("parent");
            parent.energy = parent.energy - rules.birth_cost - child.energy;
            let parent_brain = self.brains.get_mut(&id).expect("parent brain");
            parent_brain.last_birth = self.tick;
            if let Some(state) = &mut parent_brain.neural {
                let transferred = child_brain
                    .neural
                    .as_ref()
                    .expect("neural child")
                    .body
                    .momentum;
                state.body.impulse(transferred.map(|p| -p));
                parent_brain.sync_neural_pose();
            }
            self.brains.insert(child_id, child_brain);
            self.occupancy[site_index(site, dims)?] = child_id;
            self.cells.insert(child_id, child);
            disposed = add(disposed, rules.birth_cost, "birth cost")?;
            births += 1;
        }
        Ok((births, disposed))
    }

    /// One neighbor flesh unit per clot becomes nutrient. No new matter.
    /// A clot with no adjacent thread does not recover by this rule.
    fn compost(&mut self) -> Result<(), Error> {
        let dims = self.dims();
        let origins = {
            let Some(food) = self.config.feeding.as_ref() else {
                return Ok(());
            };
            crate::feeding::sites(food, dims)
        };
        for origin in origins {
            let index = site_index(origin, dims)?;
            if self.nutrient[index] >= self.config.site_cap {
                continue;
            }
            let mut donor = None;
            for roll in 1..=6 {
                let site = crate::step_site(origin, crate::Dir::from_roll(roll)?, dims)?;
                if self.flesh.get(&site).copied().unwrap_or(0) > 0 {
                    donor = Some(site);
                    break;
                }
            }
            let Some(site) = donor else {
                continue;
            };
            let amount = self.flesh.get(&site).copied().unwrap_or(0);
            if amount <= 1 {
                self.flesh.remove(&site);
            } else {
                self.flesh.insert(site, amount - 1);
            }
            self.nutrient[index] = add(self.nutrient[index], 1, "compost")?;
        }
        Ok(())
    }

    fn inject(&mut self, budget: u64) -> Result<u64, Error> {
        if budget == 0 {
            return Ok(0);
        }
        if let Some(food) = &self.config.feeding {
            let sites = crate::feeding::sites(food, self.dims());
            let mut cursor = self.inject_cursor as usize % sites.len();
            let mut moved = 0;
            for _ in 0..sites.len() {
                if moved >= budget {
                    break;
                }
                let index = site_index(sites[cursor], self.dims())?;
                if self.nutrient[index] < self.config.site_cap {
                    self.nutrient[index] += 1;
                    moved += 1;
                }
                cursor = (cursor + 1) % sites.len();
            }
            self.inject_cursor = cursor as u64;
            return Ok(moved);
        }
        let volume = self.nutrient.len();
        if volume == 0 {
            return Err(Error::Invariant("cannot inject into an empty field".into()));
        }
        let mut moved = 0u64;
        let mut cursor = (self.inject_cursor as usize) % volume;
        let mut scanned = 0usize;
        while moved < budget && scanned < volume {
            if self.nutrient[cursor] < self.config.site_cap {
                self.nutrient[cursor] += 1;
                moved += 1;
            }
            cursor = (cursor + 1) % volume;
            scanned += 1;
        }
        self.inject_cursor = u64::try_from(cursor).expect("cursor fits");
        Ok(moved)
    }

    fn rebuild_occupancy(&mut self) -> Result<(), Error> {
        self.occupancy.fill(0);
        let dims = self.dims();
        for (id, cell) in &self.cells {
            let index = site_index(cell.site, dims)?;
            if self.occupancy[index] != 0 {
                return Err(Error::Invariant(format!(
                    "double occupancy at {:?} by {} and {id}",
                    cell.site, self.occupancy[index]
                )));
            }
            self.occupancy[index] = *id;
        }
        Ok(())
    }

    pub fn audit(&self) -> Result<(), Error> {
        self.config.validate()?;
        if self.config.farm.is_none() && !self.flesh.is_empty() {
            return Err(Error::Invariant("M0 has flesh state".into()));
        }
        if self.flesh.len() > 8192 {
            return Err(Error::Invariant("flesh site limit".into()));
        }
        for (site, amount) in &self.flesh {
            site_index(*site, self.dims())?;
            if *amount == 0 || *amount > 64 {
                return Err(Error::Invariant("invalid flesh amount".into()));
            }
        }
        if self.config.farm.is_some() {
            if self.brains.len() != self.cells.len()
                || self.cells.len() > self.config.population_bound() as usize
            {
                return Err(Error::Invariant("brain/population count mismatch".into()));
            }
            for (id, brain) in &self.brains {
                if *id != brain.id
                    || !self.cells.contains_key(id)
                    || brain.parent >= *id
                    || brain.last_birth > self.tick
                    || brain.updates > self.tick
                    || brain
                        .weights
                        .iter()
                        .flatten()
                        .any(|w| w.unsigned_abs() > 128)
                    || brain
                        .eligibility
                        .iter()
                        .flatten()
                        .any(|v| v.unsigned_abs() > 64)
                    || brain
                        .joints
                        .iter()
                        .any(|v| v.unsigned_abs() > JOINT_LIMIT as u16)
                    || brain.twists.iter().any(|v| v.unsigned_abs() > 128)
                    || brain.last_dig > 2
                    || usize::from(brain.last_action) >= crate::farm::ACTIONS
                    || brain.velocity.iter().any(|v| v.unsigned_abs() > 256)
                    || brain.offset.iter().any(|v| *v < -512 || *v >= 512)
                {
                    return Err(Error::Invariant(format!("invalid brain/rig {id}")));
                }
            }
        } else if !self.brains.is_empty() {
            return Err(Error::Invariant("M0 has brain state".into()));
        }
        let mut momentum = self.environment_momentum.map(i128::from);
        for brain in self.brains.values() {
            if brain.neural.is_some() != self.config.neural.is_some() {
                return Err(Error::Invariant("controller mode mismatch".into()));
            }
            if let Some(state) = &brain.neural {
                state.validate()?;
                if state.genome.anatomy != crate::neural::schema::Anatomy::reference() {
                    return Err(Error::Invariant("world/render anatomy is currently the reference schema; alternate mappings are supported by controller probes only".into()));
                }
                if state.steps > self.tick {
                    return Err(Error::Invariant("neural steps exceed world age".into()));
                }
                for (axis, total) in momentum.iter_mut().enumerate() {
                    *total += i128::from(state.body.momentum[axis]);
                }
            }
        }
        if self.jets.len() > 4096 {
            return Err(Error::Invariant("reaction mass capacity".into()));
        }
        for jet in &self.jets {
            if jet.owner == 0
                || jet.owner >= self.next_cell_id
                || jet.life > 4
                || jet.momentum.iter().any(|p| p.unsigned_abs() > 1024)
            {
                return Err(Error::Invariant("invalid reaction mass".into()));
            }
            for (axis, extent) in [self.config.width, self.config.height, self.config.depth]
                .into_iter()
                .enumerate()
            {
                if jet.position[axis] < 0 || jet.position[axis] >= i64::from(extent) * 1024 {
                    return Err(Error::Invariant("reaction mass position".into()));
                }
                momentum[axis] += i128::from(jet.momentum[axis]);
            }
        }
        if momentum != [0; 3] {
            return Err(Error::Invariant(format!(
                "linear momentum residual {momentum:?}"
            )));
        }
        let dims = self.dims();
        let volume = dims.volume()? as usize;
        if self.nutrient.len() != volume || self.occupancy.len() != volume {
            return Err(Error::Invariant(format!(
                "field length {} or occupancy length {} != volume {volume}",
                self.nutrient.len(),
                self.occupancy.len()
            )));
        }
        if self.inject_cursor as usize >= volume {
            return Err(Error::Invariant(format!(
                "inject cursor {} is outside volume {volume}",
                self.inject_cursor
            )));
        }
        if self.reservoir > self.config.reservoir_cap {
            return Err(Error::Invariant(format!(
                "reservoir {} exceeds cap {}",
                self.reservoir, self.config.reservoir_cap
            )));
        }
        let mut seen = vec![0u64; volume];
        for (id, cell) in &self.cells {
            if *id == 0 || cell.id != *id || *id >= self.next_cell_id {
                return Err(Error::Invariant(format!("illegal live id {id}")));
            }
            if cell.energy > self.config.energy_cap {
                return Err(Error::Invariant(format!(
                    "cell {id} energy {} exceeds energy_cap",
                    cell.energy
                )));
            }
            if cell.age > self.tick {
                return Err(Error::Invariant(format!(
                    "cell {id} age {} exceeds committed ticks {}",
                    cell.age, self.tick
                )));
            }
            let index = site_index(cell.site, dims)?;
            if seen[index] != 0 {
                return Err(Error::Invariant(format!(
                    "two live cells at {:?}",
                    cell.site
                )));
            }
            seen[index] = *id;
            if self.occupancy[index] != *id {
                return Err(Error::Invariant(format!(
                    "occupancy mismatch at {:?}",
                    cell.site
                )));
            }
        }
        for (index, occupant) in self.occupancy.iter().enumerate() {
            if *occupant != seen[index] {
                return Err(Error::Invariant(format!(
                    "ghost occupancy {occupant} at linear index {index}"
                )));
            }
        }
        for (id, death) in &self.deaths {
            if self.tick == 0 || *id == 0 || death.id != *id || *id >= self.next_cell_id {
                return Err(Error::Invariant(format!("illegal death record {id}")));
            }
            if self.cells.contains_key(id) {
                return Err(Error::Invariant(format!("cell {id} is both live and dead")));
            }
            if death.during_tick >= self.tick || death.age > death.during_tick {
                return Err(Error::Invariant(format!(
                    "death record {id} has tick {} age {} at world tick {}",
                    death.during_tick, death.age, self.tick
                )));
            }
            site_index(death.site, dims)?;
        }
        for amount in &self.nutrient {
            if *amount > self.config.site_cap {
                return Err(Error::Invariant(format!(
                    "nutrient {amount} exceeds site_cap {}",
                    self.config.site_cap
                )));
            }
        }
        let accounts = self.accounts()?;
        let material = accounts.material_residual();
        if material != 0 {
            return Err(Error::Invariant(format!(
                "material residual {material}: initial {} + input {} versus field {} + reservoir {} + waste {} + output {}",
                accounts.material_initial,
                accounts.external_input,
                accounts.field,
                accounts.reservoir,
                accounts.waste,
                accounts.external_output
            )));
        }
        let energy = accounts.energy_residual();
        if energy != 0 {
            return Err(Error::Invariant(format!(
                "action-energy residual {energy}: credited {} versus live {} + maintenance {} + disposed {}",
                accounts.energy_credited,
                accounts.energy_live,
                accounts.energy_maintenance,
                accounts.energy_disposed
            )));
        }
        Ok(())
    }
}

fn apply_moves(
    cells: &mut BTreeMap<u64, Cell>,
    outcomes: &[MoveOutcome],
) -> Result<(u32, u32), Error> {
    if outcomes.len() != cells.len() {
        return Err(Error::Invariant(format!(
            "movement outcomes {} != live cells {}",
            outcomes.len(),
            cells.len()
        )));
    }
    let mut applied = 0u32;
    let mut blocked = 0u32;
    for outcome in outcomes {
        if outcome.moved {
            applied += 1;
        } else if outcome.intended != outcome.from {
            blocked += 1;
        }
        let cell = cells.get_mut(&outcome.id).ok_or_else(|| {
            Error::Invariant(format!(
                "movement outcome names missing cell {}",
                outcome.id
            ))
        })?;
        cell.site = outcome.to;
    }
    Ok((applied, blocked))
}

pub fn dims_of(config: &SimConfig) -> Dims {
    Dims {
        width: config.width,
        height: config.height,
        depth: config.depth,
    }
}

fn add(left: u64, right: u64, what: &str) -> Result<u64, Error> {
    left.checked_add(right)
        .ok_or_else(|| Error::Overflow(what.into()))
}

fn sub(left: u64, right: u64, what: &str) -> Result<u64, Error> {
    left.checked_sub(right)
        .ok_or_else(|| Error::Invariant(format!("{what}: {left} - {right}")))
}
