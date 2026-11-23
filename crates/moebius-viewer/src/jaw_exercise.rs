//! Explicit diagnostic motor exercise, never called by native world ticking.
use moebius_core::{World, neural::schema::Axis};
use std::collections::BTreeMap;

/// First close for32 ticks, then open for32; all other actuators rest.
/// Existing muscles, energy costs, contact resistance and joint stops apply.
pub fn controls(world: &World) -> BTreeMap<u64, Vec<i32>> {
    let effort = if world.tick_index() % 64 < 32 {
        -1024
    } else {
        1024
    };
    world
        .brains()
        .iter()
        .filter_map(|(&id, brain)| {
            let n = brain.neural.as_ref()?;
            let mut commands = vec![0; n.commands.len()];
            for (i, binding) in n.genome.anatomy.axes.iter().enumerate() {
                if binding.axis == Axis::Jaw {
                    commands[i] = effort;
                }
            }
            Some((id, commands))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn controlled_jaws_close_open_and_replay_with_accounted_energy() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../experiments/motor_land_demo.json");
        let mut a = crate::motor_demo::load(&path, "selected").unwrap();
        let mut b = a.clone();
        let starting_energy = a.cells().next().unwrap().energy;
        for tick in 1..=128 {
            let commands = controls(&a);
            assert!(commands[&1][..24].iter().all(|&c| c == 0));
            a.tick_controlled(&commands).unwrap();
            b.tick_controlled(&commands).unwrap();
            assert_eq!(a.snapshot().state_hash(), b.snapshot().state_hash());
            let angles = &a.brains()[&1].neural.as_ref().unwrap().body.angles[24..26];
            if tick % 64 == 32 {
                assert_eq!(angles, &[-256, -256]);
            }
            if tick % 64 == 0 {
                assert_eq!(angles, &[768, 768]);
            }
            assert_eq!(a.accounts().unwrap().energy_residual(), 0);
            assert_eq!(a.accounts().unwrap().material_residual(), 0);
        }
        assert!(a.cells().next().unwrap().energy < starting_energy);
    }
}
