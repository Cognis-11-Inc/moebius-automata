use serde::{Deserialize, Serialize};

use crate::error::Error;

/// Hard bounds for M0. Values outside these ranges are rejected.
/// The acceptance experiment stays 64³. Farm M1 defaults to the 256 cap.
pub const MAX_AXIS: u32 = 256;
pub const MAX_VOLUME: u64 = 256 * 256 * 256;
pub const MAX_TICKS: u64 = 100_000;
pub const MAX_SNAPSHOTS: u64 = 48;
pub const MAX_TRAIL_ROWS: u64 = 200_000;
pub const MAX_AMOUNT: u64 = 1_000_000_000_000;
pub const MAX_HYPOTHESIS_MARKS: usize = 4_096;

/// How the inert 8-byte genome is filled. Laws do not read it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GenomeFill {
    /// Eight bytes drawn from the placement stream after sites are chosen.
    Derived,
    /// Eight zero bytes. Placement of sites still uses the placement stream.
    Zero,
}

/// Law parameters. Seed and tick count are run parameters, not part of this struct.
///
/// `config_hash` is blake3 of the compact JSON encoding of this struct in field order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SimConfig {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub initial_population: u32,
    pub initial_energy: u64,
    pub initial_nutrient_per_site: u64,
    pub nutrient_wave_amplitude: u64,
    pub initial_reservoir: u64,
    pub initial_waste: u64,
    pub inflow_per_tick: u64,
    pub inject_per_tick: u64,
    pub uptake_per_tick: u64,
    pub energy_per_nutrient: u64,
    pub maintenance_cost: u64,
    pub export_per_tick: u64,
    pub site_cap: u64,
    pub reservoir_cap: u64,
    pub energy_cap: u64,
    pub genome_fill: GenomeFill,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub farm: Option<FarmRules>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub neural: Option<crate::neural::NeuralRules>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feeding: Option<FeedingRules>,
}

/// Optional nursery substrate and mouth-contact feeding. Absent in prior studies.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeedingRules {
    pub version: u32,
    pub spacing: u32,
    pub initial_stock: u64,
}

/// Optional M1 laws; absence preserves M0 and its historical config hashes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FarmRules {
    pub max_population: u32,
    pub birth_energy: u64,
    pub birth_cost: u64,
    pub maturity_ticks: u64,
    pub birth_cooldown: u64,
}

impl Default for FarmRules {
    fn default() -> Self {
        Self {
            max_population: 768,
            birth_energy: 80,
            birth_cost: 8,
            maturity_ticks: 64,
            birth_cooldown: 128,
        }
    }
}

impl SimConfig {
    /// Documented 64×64×64 M0 experiment. Not a claim about multicellularity.
    pub fn m0_64() -> Self {
        Self {
            width: 64,
            height: 64,
            depth: 64,
            initial_population: 128,
            initial_energy: 12,
            initial_nutrient_per_site: 4,
            nutrient_wave_amplitude: 18,
            initial_reservoir: 50_000,
            initial_waste: 0,
            inflow_per_tick: 256,
            inject_per_tick: 256,
            uptake_per_tick: 2,
            energy_per_nutrient: 3,
            maintenance_cost: 2,
            export_per_tick: 128,
            site_cap: 64,
            reservoir_cap: 50_000_000,
            energy_cap: 10_000,
            genome_fill: GenomeFill::Derived,
            farm: None,
            neural: None,
            feeding: None,
        }
    }

    /// Viewer default. A larger tank and a sparse founding population.
    /// Not the acceptance experiment, and not an organism.
    pub fn hall_128() -> Self {
        Self {
            width: 128,
            height: 128,
            depth: 128,
            initial_population: 192,
            initial_energy: 12,
            initial_nutrient_per_site: 1,
            nutrient_wave_amplitude: 14,
            initial_reservoir: 50_000,
            initial_waste: 0,
            inflow_per_tick: 256,
            inject_per_tick: 256,
            uptake_per_tick: 2,
            energy_per_nutrient: 3,
            maintenance_cost: 2,
            export_per_tick: 128,
            site_cap: 64,
            reservoir_cap: 50_000_000,
            energy_cap: 10_000,
            genome_fill: GenomeFill::Derived,
            farm: None,
            neural: None,
            feeding: None,
        }
    }

    pub fn farm_256() -> Self {
        let mut config = Self::hall_128();
        config.width = 256;
        config.height = 256;
        config.depth = 256;
        config.farm = Some(FarmRules::default());
        config
    }

    pub fn neural_farm_256() -> Self {
        let mut config = Self::farm_256();
        config.neural = Some(crate::neural::NeuralRules::default());
        config.initial_energy = 48;
        config.maintenance_cost = 1;
        config.farm.as_mut().expect("farm").max_population = 512;
        config
    }

    pub fn population_bound(&self) -> u32 {
        self.farm
            .as_ref()
            .map_or(self.initial_population, |farm| farm.max_population)
    }

    /// An explicitly provisioned nursery, not a learned-foraging result.
    pub fn nursery_farm_256() -> Self {
        let mut c = Self::neural_farm_256();
        // M4: honest articulation/action costs; scripted wave is experimental.
        let neural = c.neural.as_mut().expect("neural");
        neural.version = 2;
        neural.spinal_wave = false;
        c.feeding = Some(FeedingRules {
            version: 1,
            spacing: 16,
            initial_stock: 512,
        });
        // One bite is 4 energy. Maintenance is 1, so contact feeds the animal
        // and can pay for a birth, without filling the cap in a few ticks.
        c.uptake_per_tick = 1;
        c.initial_energy = 128;
        c.initial_nutrient_per_site = 0;
        c.nutrient_wave_amplitude = 0;
        c.site_cap = 4096;
        c.energy_per_nutrient = 4;
        c.inflow_per_tick = 1024;
        c.inject_per_tick = 1024;
        let farm = c.farm.as_mut().expect("farm");
        farm.birth_energy = 256;
        farm.maturity_ticks = 32;
        c
    }

    pub fn volume(&self) -> Result<u64, Error> {
        let volume = u64::from(self.width)
            .checked_mul(u64::from(self.height))
            .and_then(|v| v.checked_mul(u64::from(self.depth)))
            .ok_or_else(|| Error::Overflow("grid volume".into()))?;
        if self.width == 0
            || self.height == 0
            || self.depth == 0
            || self.width > MAX_AXIS
            || self.height > MAX_AXIS
            || self.depth > MAX_AXIS
            || volume == 0
            || volume > MAX_VOLUME
        {
            return Err(Error::Config(format!(
                "grid {}×{}×{} is outside each axis 1..={MAX_AXIS} and volume 1..={MAX_VOLUME}",
                self.width, self.height, self.depth
            )));
        }
        Ok(volume)
    }

    pub fn validate(&self) -> Result<(), Error> {
        let volume = self.volume()?;
        if let Some(food) = &self.feeding {
            if food.version != 1
                || !(4..=32).contains(&food.spacing)
                || self.neural.is_none()
                || self.initial_nutrient_per_site != 0
                || self.nutrient_wave_amplitude != 0
                || food.initial_stock > self.site_cap
                || [self.width, self.height, self.depth]
                    .iter()
                    .any(|n| *n < food.spacing)
            {
                return Err(Error::Config(
                    "invalid nursery food laws, spacing or field".into(),
                ));
            }
            let count: u64 = [self.width, self.height, self.depth]
                .iter()
                .map(|n| u64::from((n - 1 - food.spacing / 2) / food.spacing + 1))
                .product();
            if u64::from(self.initial_population) > count || count > 4096 {
                return Err(Error::Config("nursery needs one distinct founding food patch per founder; at most 4096 patches".into()));
            }
        }
        if let Some(neural) = &self.neural {
            neural.validate()?;
            if self.farm.is_none() || self.population_bound() > 512 {
                return Err(Error::Config(
                    "neural mode needs a farm and population bound <=512".into(),
                ));
            }
        }
        if let Some(farm) = &self.farm
            && (farm.max_population < self.initial_population
                || farm.max_population > 8192
                || u64::from(farm.max_population) > volume
                || farm.birth_energy > self.energy_cap
                || farm.birth_energy < 2
                || farm.birth_cost > farm.birth_energy - 2
                || farm.maturity_ticks == 0
                || farm.birth_cooldown == 0)
        {
            return Err(Error::Config(
                "invalid farm population or reproduction limits".into(),
            ));
        }
        if u64::from(self.initial_population) > volume {
            return Err(Error::Config(format!(
                "initial_population {} exceeds volume {volume}",
                self.initial_population
            )));
        }
        for (name, value) in [
            ("initial_energy", self.initial_energy),
            ("initial_nutrient_per_site", self.initial_nutrient_per_site),
            ("nutrient_wave_amplitude", self.nutrient_wave_amplitude),
            ("initial_reservoir", self.initial_reservoir),
            ("initial_waste", self.initial_waste),
            ("inflow_per_tick", self.inflow_per_tick),
            ("inject_per_tick", self.inject_per_tick),
            ("uptake_per_tick", self.uptake_per_tick),
            ("energy_per_nutrient", self.energy_per_nutrient),
            ("maintenance_cost", self.maintenance_cost),
            ("export_per_tick", self.export_per_tick),
            ("site_cap", self.site_cap),
            ("reservoir_cap", self.reservoir_cap),
            ("energy_cap", self.energy_cap),
        ] {
            if value > MAX_AMOUNT {
                return Err(Error::Config(format!(
                    "{name} {value} exceeds MAX_AMOUNT {MAX_AMOUNT}"
                )));
            }
        }
        let wave_span = if self.nutrient_wave_amplitude == 0 {
            0
        } else {
            self.nutrient_wave_amplitude - 1
        };
        let peak = self
            .initial_nutrient_per_site
            .checked_add(wave_span)
            .ok_or_else(|| Error::Overflow("initial nutrient peak".into()))?;
        if peak > self.site_cap {
            return Err(Error::Config(format!(
                "initial nutrient peak {peak} (base {} + wave span {wave_span}) exceeds site_cap {}",
                self.initial_nutrient_per_site, self.site_cap
            )));
        }
        if self.initial_reservoir > self.reservoir_cap {
            return Err(Error::Config(format!(
                "initial_reservoir {} exceeds reservoir_cap {}",
                self.initial_reservoir, self.reservoir_cap
            )));
        }
        if self.initial_energy > self.energy_cap {
            return Err(Error::Config(format!(
                "initial_energy {} exceeds energy_cap {}",
                self.initial_energy, self.energy_cap
            )));
        }
        if self.inject_per_tick > volume {
            return Err(Error::Config(format!(
                "inject_per_tick {} exceeds volume {volume}; one sweep can place at most one unit on each site",
                self.inject_per_tick
            )));
        }
        Ok(())
    }
}

pub fn load_config_str(text: &str) -> Result<SimConfig, Error> {
    let config: SimConfig = serde_json::from_str(text).map_err(|err| {
        Error::Config(format!(
            "configuration JSON rejected: {err}{}",
            negative_field_note(text)
        ))
    })?;
    config.validate()?;
    Ok(config)
}

/// Serde's u64 error names the column, not the field. Name any negative integer.
fn negative_field_note(text: &str) -> String {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(text) else {
        return String::new();
    };
    let Some(object) = value.as_object() else {
        return String::new();
    };
    let mut notes = Vec::new();
    for (key, item) in object {
        if item.as_i64().is_some_and(|number| number < 0) {
            notes.push(format!("{key} is negative"));
        } else if item.is_number() && item.as_u64().is_none() && item.as_i64().is_none() {
            notes.push(format!("{key} is not an integer"));
        }
    }
    if notes.is_empty() {
        String::new()
    } else {
        format!("; {}", notes.join(", "))
    }
}

pub fn config_hash_bytes(config: &SimConfig) -> Result<[u8; 32], Error> {
    let bytes = serde_json::to_vec(config)
        .map_err(|err| Error::Config(format!("cannot encode configuration: {err}")))?;
    Ok(*blake3::hash(&bytes).as_bytes())
}

pub fn config_hash(config: &SimConfig) -> Result<String, Error> {
    Ok(crate::hashutil::hex32(&config_hash_bytes(config)?))
}
