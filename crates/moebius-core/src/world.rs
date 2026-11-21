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
    pub secreted: u64,
    pub excavated: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Accounts {
    pub material_initial: u64,
    pub external_input: u64,
    pub external_output: u64,
    pub field: u64,
    pub flesh: u64,
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
    stocks.flesh = sub(stocks.flesh, receipt.excavated, "excavated flesh")?;
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

/// Authoritative world. Mutation of scientific state goes through [`World::tick`].
#[derive(Clone, Debug)]
pub struct World {
    config: SimConfig,
    seed: u64,
    tick: u64,
    cells: BTreeMap<u64, Cell>,
    brains: BTreeMap<u64, BrainRig>,
    flesh: BTreeMap<Site, u64>,
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
}

impl World {
    pub fn genesis(config: &SimConfig, seed: u64) -> Result<Self, Error> {
        config.validate()?;
        let dims = dims_of(config);
        let volume_u64 = dims.volume()?;
        let volume = usize::try_from(volume_u64).expect("volume fits usize");
        let mut index: Vec<u32> =
            (0..u32::try_from(volume_u64).expect("volume fits u32")).collect();
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
                .map(|cell| (cell.id, BrainRig::new(cell, seed)))
                .collect()
        } else {
            BTreeMap::new()
        };
        let mut world = Self {
            config: config.clone(),
            brains,
            flesh: BTreeMap::new(),
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
    /// 5. Uptake converts site nutrient into waste 1:1 and credits action-energy.
    /// 6. Maintenance is paid in full or the cell dies and its action-energy is disposed.
    /// 7. Waste is exported up to the available stock.
    /// 8. The committed tick index increments.
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
        let mut proposals = Vec::with_capacity(self.cells.len());
        let mut decisions = BTreeMap::new();
        let mut motor_energy = 0;
        for cell in self.cells.values() {
            let to = if let Some(brain) = self.brains.get_mut(&cell.id) {
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
        let solid = fused_surface(&self.flesh, dims)?;
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
                }
                let cell = self.cells.get_mut(&proposal.id).expect("moving cell");
                cell.energy = sub(cell.energy, *cost, "motor effort")?;
                motor_energy = add(motor_energy, *cost, "motor energy")?;
                brain.last_meal = 0;
                brain.last_dig = 0;
            }
        }

        let ids: Vec<u64> = self.cells.keys().copied().collect();
        let mut uptaken = 0u64;
        let mut credited = 0u64;
        for id in &ids {
            let (site, energy) = {
                let cell = &self.cells[id];
                (cell.site, cell.energy)
            };
            let index = site_index(site, dims)?;
            let units = uptake_units(
                self.nutrient[index],
                self.config.uptake_per_tick,
                energy,
                self.config.energy_cap,
                self.config.energy_per_nutrient,
            );
            if let Some(brain) = self.brains.get_mut(id) {
                brain.last_meal = units;
            }
            if units == 0 {
                continue;
            }
            let gain = units
                .checked_mul(self.config.energy_per_nutrient)
                .ok_or_else(|| Error::Overflow(format!("energy credit for cell {id}")))?;
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

        let mut secretion_paid = std::collections::BTreeSet::new();
        let mut secreted = 0u64;
        let mut secretion_cost = 0u64;
        for (id, (action, _, _, _)) in &decisions {
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
        let mut excavated = 0u64;
        let mut digging_paid = std::collections::BTreeSet::new();
        let mut digging_cost = 0u64;
        for (id, (action, _, _, _)) in &decisions {
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
        let mut maintained = motor_energy;
        let mut disposed = secretion_cost + digging_cost;
        let mut deaths = 0u32;
        let cost = self.config.maintenance_cost;
        for id in ids {
            let energy = self.cells[&id].energy;
            if let Some((action, features, motor_cost, _)) = decisions.get(&id) {
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
            waste_created: uptaken + excavated,
            energy_credited: credited,
            energy_maintenance: maintained,
            energy_disposed: disposed,
            external_output: exported,
            deaths,
            moves_applied,
            moves_blocked,
            births,
            motor_energy,
            secreted,
            excavated,
        })
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
            let child_brain = brain.child(child_id, self.seed, self.tick);
            let parent = self.cells.get_mut(&id).expect("parent");
            parent.energy = parent.energy - rules.birth_cost - child.energy;
            self.brains.get_mut(&id).expect("parent brain").last_birth = self.tick;
            self.brains.insert(child_id, child_brain);
            self.occupancy[site_index(site, dims)?] = child_id;
            self.cells.insert(child_id, child);
            disposed = add(disposed, rules.birth_cost, "birth cost")?;
            births += 1;
        }
        Ok((births, disposed))
    }

    fn inject(&mut self, budget: u64) -> Result<u64, Error> {
        if budget == 0 {
            return Ok(0);
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
