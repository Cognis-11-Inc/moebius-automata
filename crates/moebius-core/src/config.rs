use serde::{Deserialize, Serialize};

use crate::error::Error;

/// Hard bounds for M0. Values outside these ranges are rejected.
pub const MAX_AXIS: u32 = 64;
pub const MAX_VOLUME: u64 = 64 * 64 * 64;
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
        }
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
