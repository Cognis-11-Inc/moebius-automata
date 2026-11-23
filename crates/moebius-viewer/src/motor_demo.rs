//! Explicit M6 demonstration; ordinary nursery startup never calls this.
use moebius_core::{
    SimConfig, Site, World,
    neural::{ControllerGenome, NeuralState, Policy},
};
use std::path::Path;
pub fn load(path: &Path, choice: &str) -> Result<World, String> {
    let data: serde_json::Value =
        serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    if data["format"] != "ma-motor-demo-v1" {
        return Err("unknown motor demo format".into());
    }
    let selected: ControllerGenome =
        serde_json::from_value(data["selected_genome"].clone()).map_err(|e| e.to_string())?;
    let original: ControllerGenome =
        serde_json::from_value(data["original_genome"].clone()).map_err(|e| e.to_string())?;
    selected.validate().map_err(|e| e.to_string())?;
    original.validate().map_err(|e| e.to_string())?;
    if selected.anatomy != original.anatomy
        || selected.strengths != original.strengths
        || selected.ligament_rest != original.ligament_rest
        || selected.ligament_stiffness != original.ligament_stiffness
        || selected.muscle_rate != original.muscle_rate
    {
        return Err("demo body genes must match".into());
    }
    let mut config: SimConfig =
        serde_json::from_value(data["config"].clone()).map_err(|e| e.to_string())?;
    if config
        .neural
        .as_ref()
        .is_none_or(|n| n.version != 2 || n.spinal_wave || n.policy != Policy::Recurrent)
        || config.initial_nutrient_per_site != 0
        || config.inflow_per_tick != 0
        || config.inject_per_tick != 0
    {
        return Err("demo requires explicit food-free native recurrent nursery".into());
    }
    if !["selected", "untrained", "compare"].contains(&choice) {
        return Err("motor control must be selected, untrained or compare".into());
    }
    let compare = choice == "compare";
    config.initial_population = if compare { 2 } else { 1 };
    config
        .farm
        .as_mut()
        .ok_or("demo farm rules required")?
        .max_population = config.initial_population;
    let seed = data["seed"].as_u64().ok_or("demo seed")?;
    let mut s = World::genesis(&config, seed)
        .map_err(|e| e.to_string())?
        .snapshot();
    for (i, (cell, brain)) in s.cells.iter_mut().zip(&mut s.brains).enumerate() {
        cell.site = Site {
            x: if compare {
                if i == 0 { 4 } else { 12 }
            } else {
                8
            },
            y: 8,
            z: 8,
        };
        let g = if choice == "untrained" || compare && i == 1 {
            &original
        } else {
            &selected
        };
        brain.neural = Some(NeuralState::new(g.clone(), cell.energy));
        brain.sync_neural_pose();
    }
    World::restore(&config, &s).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn demo_preserves_selected_genomes_in_native_world_and_limits_population() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/m6/DEMO.json");
        let mut comparison = load(&path, "compare").unwrap();
        assert_eq!(comparison.live_count(), 2);
        assert_eq!(comparison.config().farm.as_ref().unwrap().max_population, 2);
        let selected = comparison.brains()[&1].neural.as_ref().unwrap().genome.id();
        let original = comparison.brains()[&2].neural.as_ref().unwrap().genome.id();
        assert_ne!(selected, original);
        for _ in 0..128 {
            comparison.tick().unwrap();
        }
        assert_eq!(
            comparison.brains()[&1].neural.as_ref().unwrap().genome.id(),
            selected
        );
        assert_eq!(
            comparison.brains()[&2].neural.as_ref().unwrap().genome.id(),
            original
        );
        assert!(comparison.nutrient().iter().all(|n| *n == 0));
        assert_eq!(comparison.accounts().unwrap().material_residual(), 0);
        assert_eq!(comparison.accounts().unwrap().energy_residual(), 0);
        assert_eq!(
            load(&path, "selected").unwrap().brains()[&1]
                .neural
                .as_ref()
                .unwrap()
                .genome
                .id(),
            selected
        );
        assert_eq!(
            load(&path, "untrained").unwrap().brains()[&1]
                .neural
                .as_ref()
                .unwrap()
                .genome
                .id(),
            original
        );
        assert!(load(&path, "invalid").is_err());
    }
}
