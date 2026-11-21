//! Operator rules for one run. Edits happen before the first tick.
//! A refused step leaves the previous legal value in place.

use std::path::Path;

use bevy::prelude::Resource;
use moebius_core::{GenomeFill, SimConfig, check_horizon, config_hash, validate_run};

#[derive(Clone, Copy)]
enum Knob {
    Open,
    Span,
    Population,
    InitialEnergy,
    Maintenance,
    Uptake,
    Inflow,
    Seed,
    Ticks,
    Interval,
    Width,
    Height,
    Depth,
    Nutrient,
    Wave,
    Reservoir,
    Waste,
    Inject,
    EnergyPer,
    Export,
    SiteCap,
    ReservoirCap,
    EnergyCap,
    Genome,
}

const KNOBS: [Knob; 24] = [
    Knob::Open,
    Knob::Span,
    Knob::Population,
    Knob::InitialEnergy,
    Knob::Maintenance,
    Knob::Uptake,
    Knob::Inflow,
    Knob::Seed,
    Knob::Ticks,
    Knob::Interval,
    Knob::Width,
    Knob::Height,
    Knob::Depth,
    Knob::Nutrient,
    Knob::Wave,
    Knob::Reservoir,
    Knob::Waste,
    Knob::Inject,
    Knob::EnergyPer,
    Knob::Export,
    Knob::SiteCap,
    Knob::ReservoirCap,
    Knob::EnergyCap,
    Knob::Genome,
];

impl Knob {
    fn label(self) -> &'static str {
        match self {
            Self::Open => "open_run",
            Self::Span => "span",
            Self::Population => "initial_population",
            Self::InitialEnergy => "initial_energy",
            Self::Maintenance => "maintenance_cost",
            Self::Uptake => "uptake_per_tick",
            Self::Inflow => "inflow_per_tick",
            Self::Seed => "seed",
            Self::Ticks => "ticks",
            Self::Interval => "snapshot_interval",
            Self::Width => "width",
            Self::Height => "height",
            Self::Depth => "depth",
            Self::Nutrient => "initial_nutrient_per_site",
            Self::Wave => "nutrient_wave_amplitude",
            Self::Reservoir => "initial_reservoir",
            Self::Waste => "initial_waste",
            Self::Inject => "inject_per_tick",
            Self::EnergyPer => "energy_per_nutrient",
            Self::Export => "export_per_tick",
            Self::SiteCap => "site_cap",
            Self::ReservoirCap => "reservoir_cap",
            Self::EnergyCap => "energy_cap",
            Self::Genome => "genome_fill",
        }
    }

    fn steps(self) -> (i64, i64) {
        match self {
            Self::Open | Self::Genome => (1, 1),
            Self::Span | Self::Width | Self::Height | Self::Depth => (1, 8),
            Self::Population => (1, 16),
            Self::InitialEnergy => (1, 10),
            Self::Nutrient | Self::Wave => (1, 4),
            Self::Reservoir => (1_000, 10_000),
            Self::Waste => (1, 100),
            Self::Inflow | Self::Inject => (1, 32),
            Self::Uptake | Self::EnergyPer | Self::Maintenance => (1, 1),
            Self::Export => (1, 16),
            Self::SiteCap => (1, 8),
            Self::ReservoirCap => (100_000, 1_000_000),
            Self::EnergyCap => (100, 1_000),
            Self::Seed => (1, 1_000),
            Self::Ticks => (50, 500),
            Self::Interval => (10, 50),
        }
    }
}

#[derive(Clone, Resource)]
pub struct Draft {
    pub config: SimConfig,
    pub seed: u64,
    pub ticks: u64,
    pub snapshot_interval: u64,
    pub until_stopped: bool,
    pub selected: usize,
    pub status: String,
    pub pending: bool,
    pub recording: Option<std::path::PathBuf>,
    baseline_config: SimConfig,
    baseline_seed: u64,
    baseline_ticks: u64,
    baseline_interval: u64,
    baseline_until_stopped: bool,
}

impl Draft {
    /// Recorded acceptance length. Not the window's default.
    #[cfg(test)]
    pub fn official() -> Self {
        let mut draft = Self::from_parts(SimConfig::m0_64(), 20_261_026, 1_000, 50, false)
            .expect("the documented 64³ run of 1000 ticks is valid");
        draft.status = "Acceptance recording. 64³, 1000 ticks.".into();
        draft
    }

    /// Game default. Keeps ticking. Does not pre-commit a length.
    pub fn hall() -> Self {
        let mut draft = Self::from_parts(SimConfig::farm_256(), 20_261_026, 1_000, 50, true)
            .expect("the 256³ farm is valid for one tick");
        draft.status =
            "Open farm. 256 on a side. Adaptive joint motors, inertia, feeding, and inherited controllers.".into();
        draft
    }

    pub fn from_parts(
        config: SimConfig,
        seed: u64,
        ticks: u64,
        snapshot_interval: u64,
        until_stopped: bool,
    ) -> Result<Self, String> {
        let draft = Self {
            baseline_config: config.clone(),
            baseline_seed: seed,
            baseline_ticks: ticks,
            baseline_interval: snapshot_interval,
            baseline_until_stopped: until_stopped,
            config,
            seed,
            ticks,
            snapshot_interval,
            until_stopped,
            selected: 0,
            status: String::new(),
            pending: false,
            recording: None,
        };
        accept(&draft)?;
        Ok(draft)
    }

    pub fn from_run_dir(dir: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(dir.join("manifest.json"))
            .map_err(|err| format!("cannot read manifest.json: {err}"))?;
        let manifest: serde_json::Value =
            serde_json::from_str(&text).map_err(|err| format!("manifest JSON rejected: {err}"))?;
        let config: SimConfig = serde_json::from_value(
            manifest
                .get("config")
                .cloned()
                .ok_or_else(|| "manifest is missing config".to_string())?,
        )
        .map_err(|err| format!("manifest config rejected: {err}"))?;
        let stated = manifest
            .get("config_hash")
            .and_then(|value| value.as_str())
            .ok_or_else(|| "manifest is missing config_hash".to_string())?;
        let actual = config_hash(&config).map_err(|err| err.to_string())?;
        if stated != actual {
            return Err(format!(
                "manifest config_hash {stated} != computed {actual}"
            ));
        }
        let seed = json_u64(&manifest, "seed")?;
        let ticks = json_u64(&manifest, "ticks")?;
        let snapshot_interval = json_u64(&manifest, "snapshot_interval")?;
        let mut draft = Self::from_parts(config, seed, ticks, snapshot_interval, false)?;
        draft.recording = Some(dir.to_path_buf());
        draft.status =
            "Committed recording. Enter watches it. open_run starts a live world instead.".into();
        Ok(draft)
    }

    pub fn unchanged(&self) -> bool {
        self.recording.is_some()
            && self.config == self.baseline_config
            && self.seed == self.baseline_seed
            && self.ticks == self.baseline_ticks
            && self.snapshot_interval == self.baseline_interval
            && self.until_stopped == self.baseline_until_stopped
    }
}

pub fn move_selection(draft: &mut Draft, down: bool) {
    let count = KNOBS.len();
    if down {
        draft.selected = (draft.selected + 1) % count;
    } else {
        draft.selected = (draft.selected + count - 1) % count;
    }
}

pub fn adjust(draft: &mut Draft, large: bool, forward: bool) {
    let knob = KNOBS[draft.selected % KNOBS.len()];
    let (small, big) = knob.steps();
    let magnitude = if large { big } else { small };
    let delta = if forward { magnitude } else { -magnitude };
    let mut next = draft.clone();
    if !apply_delta(&mut next, knob, delta) {
        draft.status = format!(
            "{} is unchanged: that step does not fit in the field",
            knob.label()
        );
        return;
    }
    if let Err(err) = accept(&next) {
        draft.status = format!("refused: {err}");
        return;
    }
    draft.config = next.config;
    draft.seed = next.seed;
    draft.ticks = next.ticks;
    draft.snapshot_interval = next.snapshot_interval;
    draft.until_stopped = next.until_stopped;
    draft.status = format!("{} is {}", knob.label(), value_of(draft, knob));
}

pub fn slate(draft: &Draft) -> String {
    let mut lines = String::from(
        "Möbius Automata M0\n\
         Founding cells, then Enter. The laws lock before tick 0.\n\
         Up/Down selects. Left/Right steps. Shift steps further.\n\
         open_run keeps ticking until a tick is refused or the window closes.\n\
         span sets width, height, and depth to the same length.\n\
         Founding cells are the initial population. They are not a membership.\n\
         The acceptance experiment stays 64³ and 1000 ticks.\n\
         Fixed: 6-neighbor periodic grid, lowest id wins an empty site.\n",
    );
    if draft.config.farm.is_some() {
        lines.push_str(
            "M1: joint motors, inertial coasting, adaptive controller and asexual reproduction.\n",
        );
    } else {
        lines.push_str("M0 recording: no brain or reproduction.\n");
    }
    if let Some(path) = &draft.recording {
        lines.push_str(&format!("recording {}\n", path.display()));
    }
    for (index, knob) in KNOBS.iter().enumerate() {
        let marker = if index == draft.selected % KNOBS.len() {
            ">"
        } else {
            " "
        };
        lines.push_str(&format!(
            "{marker} {:<28} {}\n",
            knob.label(),
            value_of(draft, *knob)
        ));
    }
    lines.push_str(&draft.status);
    lines.push('\n');
    if draft.pending {
        lines.push_str("Locking the rules. The picture waits.\n");
    } else if draft.unchanged() {
        lines.push_str("Enter watches this committed recording.\n");
    } else if draft.until_stopped {
        lines.push_str(
            "Enter locks these founding rules and keeps ticking from tick 0.\n\
             A refused tick pauses on the last committed state.\n",
        );
    } else {
        lines.push_str("Enter commits this finite run from tick 0, then watches it.\n");
    }
    lines
}

fn accept(draft: &Draft) -> Result<(), String> {
    if draft.until_stopped {
        draft.config.validate().map_err(|err| err.to_string())?;
        check_horizon(&draft.config, 1).map_err(|err| err.to_string())?;
        Ok(())
    } else {
        validate_run(&draft.config, draft.ticks, draft.snapshot_interval)
            .map(|_| ())
            .map_err(|err| err.to_string())
    }
}

fn value_of(draft: &Draft, knob: Knob) -> String {
    let config = &draft.config;
    match knob {
        Knob::Open => {
            if draft.until_stopped {
                "yes".into()
            } else {
                "no".into()
            }
        }
        Knob::Span => {
            if config.width == config.height && config.height == config.depth {
                config.width.to_string()
            } else {
                format!("{}×{}×{}", config.width, config.height, config.depth)
            }
        }
        Knob::Seed => draft.seed.to_string(),
        Knob::Ticks => {
            if draft.until_stopped {
                format!("{} (not a stop)", draft.ticks)
            } else {
                draft.ticks.to_string()
            }
        }
        Knob::Interval => draft.snapshot_interval.to_string(),
        Knob::Width => config.width.to_string(),
        Knob::Height => config.height.to_string(),
        Knob::Depth => config.depth.to_string(),
        Knob::Population => config.initial_population.to_string(),
        Knob::InitialEnergy => config.initial_energy.to_string(),
        Knob::Nutrient => config.initial_nutrient_per_site.to_string(),
        Knob::Wave => config.nutrient_wave_amplitude.to_string(),
        Knob::Reservoir => config.initial_reservoir.to_string(),
        Knob::Waste => config.initial_waste.to_string(),
        Knob::Inflow => config.inflow_per_tick.to_string(),
        Knob::Inject => config.inject_per_tick.to_string(),
        Knob::Uptake => config.uptake_per_tick.to_string(),
        Knob::EnergyPer => config.energy_per_nutrient.to_string(),
        Knob::Maintenance => config.maintenance_cost.to_string(),
        Knob::Export => config.export_per_tick.to_string(),
        Knob::SiteCap => config.site_cap.to_string(),
        Knob::ReservoirCap => config.reservoir_cap.to_string(),
        Knob::EnergyCap => config.energy_cap.to_string(),
        Knob::Genome => match config.genome_fill {
            GenomeFill::Derived => "derived".into(),
            GenomeFill::Zero => "zero".into(),
        },
    }
}

fn apply_delta(draft: &mut Draft, knob: Knob, delta: i64) -> bool {
    let config = &mut draft.config;
    match knob {
        Knob::Open => {
            draft.until_stopped = !draft.until_stopped;
            true
        }
        Knob::Span => {
            let mut next = config.width;
            if !add_u32(&mut next, delta) {
                return false;
            }
            config.width = next;
            config.height = next;
            config.depth = next;
            true
        }
        Knob::Seed => add_u64(&mut draft.seed, delta),
        Knob::Ticks => add_u64(&mut draft.ticks, delta),
        Knob::Interval => add_u64(&mut draft.snapshot_interval, delta),
        Knob::Width => add_u32(&mut config.width, delta),
        Knob::Height => add_u32(&mut config.height, delta),
        Knob::Depth => add_u32(&mut config.depth, delta),
        Knob::Population => add_u32(&mut config.initial_population, delta),
        Knob::InitialEnergy => add_u64(&mut config.initial_energy, delta),
        Knob::Nutrient => add_u64(&mut config.initial_nutrient_per_site, delta),
        Knob::Wave => add_u64(&mut config.nutrient_wave_amplitude, delta),
        Knob::Reservoir => add_u64(&mut config.initial_reservoir, delta),
        Knob::Waste => add_u64(&mut config.initial_waste, delta),
        Knob::Inflow => add_u64(&mut config.inflow_per_tick, delta),
        Knob::Inject => add_u64(&mut config.inject_per_tick, delta),
        Knob::Uptake => add_u64(&mut config.uptake_per_tick, delta),
        Knob::EnergyPer => add_u64(&mut config.energy_per_nutrient, delta),
        Knob::Maintenance => add_u64(&mut config.maintenance_cost, delta),
        Knob::Export => add_u64(&mut config.export_per_tick, delta),
        Knob::SiteCap => add_u64(&mut config.site_cap, delta),
        Knob::ReservoirCap => add_u64(&mut config.reservoir_cap, delta),
        Knob::EnergyCap => add_u64(&mut config.energy_cap, delta),
        Knob::Genome => {
            config.genome_fill = match config.genome_fill {
                GenomeFill::Derived => GenomeFill::Zero,
                GenomeFill::Zero => GenomeFill::Derived,
            };
            true
        }
    }
}

fn add_u64(slot: &mut u64, delta: i64) -> bool {
    let Some(next) = add_i64(*slot, delta) else {
        return false;
    };
    *slot = next;
    true
}

fn add_u32(slot: &mut u32, delta: i64) -> bool {
    let Some(wide) = add_i64(u64::from(*slot), delta) else {
        return false;
    };
    let Some(next) = u32::try_from(wide).ok() else {
        return false;
    };
    *slot = next;
    true
}

fn add_i64(value: u64, delta: i64) -> Option<u64> {
    if delta >= 0 {
        value.checked_add(u64::try_from(delta).ok()?)
    } else {
        value.checked_sub(delta.unsigned_abs())
    }
}

fn json_u64(manifest: &serde_json::Value, key: &str) -> Result<u64, String> {
    manifest
        .get(key)
        .and_then(|value| value.as_u64())
        .ok_or_else(|| format!("manifest is missing {key}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn knob(name: &str) -> usize {
        KNOBS
            .iter()
            .position(|knob| knob.label() == name)
            .expect(name)
    }

    #[test]
    fn official_run_is_valid_and_names_the_default_length() {
        let draft = Draft::official();
        assert_eq!(draft.ticks, 1_000);
        assert!(!draft.until_stopped);
        assert!(slate(&draft).contains("1000"));
        assert!(slate(&draft).contains("lowest id"));
        validate_run(&draft.config, draft.ticks, draft.snapshot_interval).unwrap();
    }

    #[test]
    fn hall_is_open_and_names_the_founding_rules() {
        let draft = Draft::hall();
        assert!(draft.until_stopped);
        assert_eq!(draft.config.width, 256);
        assert_eq!(draft.config.initial_population, 192);
        let text = slate(&draft);
        assert!(text.contains("open_run"));
        assert!(text.contains("joint motors"));
        assert!(text.contains("> open_run"));
    }

    #[test]
    fn axis_past_the_cap_is_refused_and_the_value_stays() {
        let mut draft = Draft::hall();
        draft.selected = knob("width");
        adjust(&mut draft, false, true);
        assert_eq!(draft.config.width, 256);
        assert_eq!(draft.config.height, 256);
        assert!(draft.status.contains("refused"), "{}", draft.status);
    }

    #[test]
    fn span_steps_the_cube_and_stops_at_the_cap() {
        let mut draft = Draft::hall();
        draft.selected = knob("span");
        adjust(&mut draft, false, false);
        assert_eq!(draft.config.width, 255);
        assert_eq!(draft.config.height, 255);
        assert_eq!(draft.config.depth, 255);
        adjust(&mut draft, false, true);
        assert_eq!(draft.config.width, 256);
        adjust(&mut draft, false, true);
        assert_eq!(draft.config.width, 256);
        assert_eq!(draft.config.depth, 256);
        assert!(draft.status.contains("refused"), "{}", draft.status);
    }

    #[test]
    fn open_run_does_not_precheck_an_infinite_trail() {
        let mut draft = Draft::hall();
        draft.ticks = 50_000;
        draft.selected = knob("seed");
        adjust(&mut draft, false, true);
        assert_eq!(draft.seed, 20_261_027);
        assert!(draft.until_stopped);
        assert!(validate_run(&draft.config, draft.ticks, draft.snapshot_interval).is_err());
        draft.selected = knob("open_run");
        adjust(&mut draft, false, true);
        assert!(draft.until_stopped, "{}", draft.status);
        assert!(draft.status.contains("refused"), "{}", draft.status);
    }

    #[test]
    fn population_past_the_volume_is_refused() {
        let config = SimConfig {
            width: 2,
            height: 2,
            depth: 2,
            initial_population: 8,
            initial_energy: 4,
            initial_nutrient_per_site: 1,
            nutrient_wave_amplitude: 0,
            initial_reservoir: 20,
            initial_waste: 0,
            inflow_per_tick: 1,
            inject_per_tick: 1,
            uptake_per_tick: 1,
            energy_per_nutrient: 1,
            maintenance_cost: 1,
            export_per_tick: 1,
            site_cap: 8,
            reservoir_cap: 10_000,
            energy_cap: 100,
            genome_fill: GenomeFill::Zero,
            farm: None,
        };
        let mut draft = Draft::from_parts(config, 3, 20, 10, false).unwrap();
        draft.selected = knob("initial_population");
        adjust(&mut draft, false, true);
        assert_eq!(draft.config.initial_population, 8);
        assert!(draft.status.contains("refused"), "{}", draft.status);
    }

    #[test]
    fn a_legal_step_applies_and_zero_does_not_wrap() {
        let mut draft = Draft::official();
        draft.selected = knob("seed");
        adjust(&mut draft, true, true);
        assert_eq!(draft.seed, 20_262_026);
        draft.selected = knob("maintenance_cost");
        let cost = draft.config.maintenance_cost;
        for _ in 0..cost + 2 {
            adjust(&mut draft, false, false);
        }
        assert_eq!(draft.config.maintenance_cost, 0);
        assert!(
            draft.status.contains("unchanged") || draft.status.contains("refused"),
            "{}",
            draft.status
        );
    }

    #[test]
    fn genome_toggles_and_a_reverted_recording_matches() {
        let mut draft = Draft::official();
        draft.recording = Some(std::path::PathBuf::from("runs/example"));
        assert!(draft.unchanged());
        draft.selected = knob("genome_fill");
        adjust(&mut draft, false, true);
        assert_eq!(draft.config.genome_fill, GenomeFill::Zero);
        assert!(!draft.unchanged());
        adjust(&mut draft, false, false);
        assert_eq!(draft.config.genome_fill, GenomeFill::Derived);
        assert!(draft.unchanged());
    }
}
