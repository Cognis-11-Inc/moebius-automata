//! Version 1 local observation ABI. No coordinates, IDs, reward targets or
//! distant field samples enter the controller.
use super::{NeuralRules, NeuralState, Q, body};
pub struct Observation {
    pub energy: u64,
    pub energy_cap: u64,
    pub nutrient: u64,
    pub site_cap: u64,
    pub chemical: [i32; 6],
    pub contact: [i32; 6],
    pub jaw_load: [i32; 2],
}
pub fn encode(state: &NeuralState, observation: Observation, rules: &NeuralRules) -> Vec<i32> {
    let mut values = Vec::new();
    for (i, binding) in state.genome.anatomy.axes.iter().enumerate() {
        let limit = binding.min.abs().max(binding.max);
        values.push(if rules.proprioception {
            state.body.angles[i] * Q / limit
        } else {
            0
        });
        values.push(if rules.proprioception {
            state.body.rates[i] * Q / 64
        } else {
            0
        });
        values.push(if rules.proprioception {
            state.body.efforts[i] * Q / 2048
        } else {
            0
        });
    }
    let velocity = body::local(state.body.velocity, state.body.orientation);
    let acceleration = body::local(state.body.acceleration, state.body.orientation);
    for value in velocity {
        values.push(if rules.proprioception {
            value * Q / 256
        } else {
            0
        });
    }
    for value in acceleration {
        values.push(if rules.proprioception {
            value * Q / 64
        } else {
            0
        });
    }
    for value in state.body.angular_velocity {
        values.push(if rules.proprioception {
            value * Q / 64
        } else {
            0
        });
    }
    values.push(
        (u128::from(observation.energy) * Q as u128 / u128::from(observation.energy_cap.max(1)))
            .min(Q as u128) as i32,
    );
    values.push(if observation.nutrient > 0 && observation.energy >= 2 {
        Q
    } else {
        0
    });
    values.push((state.last_intake.min(4) as i32) * Q / 4);
    values.push(state.reward);
    values.push((state.spent_actuator.min(4) as i32) * Q / 4);
    values.push(if rules.chemistry {
        (observation.nutrient.min(observation.site_cap) as i128 * i128::from(Q)
            / i128::from(observation.site_cap.max(1))) as i32
    } else {
        0
    });
    values.extend(
        observation
            .chemical
            .map(|x| if rules.chemistry { x } else { 0 }),
    );
    values.extend(observation.contact);
    values.extend(observation.jaw_load);
    for value in &mut values {
        *value = (*value).clamp(-Q, Q);
    }
    debug_assert_eq!(values.len(), state.genome.anatomy.sensors());
    values
}
