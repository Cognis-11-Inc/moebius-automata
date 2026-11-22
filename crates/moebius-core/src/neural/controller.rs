use super::{body::Body, schema::Anatomy};
use crate::{Error, prng::domain_seed};
use serde::{Deserialize, Serialize};
pub const Q: i32 = 1024;
pub const NEURONS: usize = 8;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Policy {
    #[default]
    Recurrent,
    Random,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NeuralRules {
    pub version: u32,
    pub plasticity: bool,
    pub recurrence: bool,
    pub proprioception: bool,
    pub chemistry: bool,
    pub mutation: bool,
    pub policy: Policy,
    pub drag: u32,
    /// Explicit ablation; omitted/true preserves historical law encoding.
    #[serde(default = "enabled", skip_serializing_if = "is_enabled")]
    pub spinal_wave: bool,
}
fn enabled() -> bool {
    true
}
fn is_enabled(value: &bool) -> bool {
    *value
}
impl Default for NeuralRules {
    fn default() -> Self {
        Self {
            version: 1,
            plasticity: true,
            recurrence: true,
            proprioception: true,
            chemistry: true,
            mutation: true,
            policy: Policy::Recurrent,
            drag: 8,
            spinal_wave: true,
        }
    }
}
impl NeuralRules {
    pub fn validate(&self) -> Result<(), Error> {
        if !(1..=2).contains(&self.version) || self.drag > 128 {
            return Err(Error::Config("unsupported neural laws or drag".into()));
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControllerGenome {
    pub version: u32,
    pub anatomy: Anatomy,
    pub recurrent: Vec<i32>,
    pub sensory: Vec<i32>,
    pub output: Vec<i32>,
    pub biases: Vec<i32>,
    pub motor_biases: Vec<i32>,
    pub tau: Vec<i32>,
    pub sensory_gains: Vec<i32>,
    pub motor_gains: Vec<i32>,
    pub plastic_rates: Vec<i32>,
    pub strengths: Vec<i32>,
    /// Pull back toward `ligament_rest`. Zero means the founder stiffness of 6.
    /// Higher is stiffer. This is an alien spring, not a vertebrate tendon.
    #[serde(default)]
    pub ligament_stiffness: i32,
    /// Ticks of nerve command the muscle takes to follow. Zero still means 8
    /// so an older genome loads. New founders are written as 3.
    #[serde(default)]
    pub muscle_rate: i32,
    /// Heritable resting curve, one angle per joint axis. Empty means straight.
    #[serde(default)]
    pub ligament_rest: Vec<i32>,
}
fn draw(seed: u64, label: &[u8], id: u64, index: usize) -> i32 {
    let bytes = domain_seed(seed, label, index as u64, id);
    i32::from_le_bytes(bytes[..4].try_into().unwrap()) & i32::MAX
}
impl ControllerGenome {
    pub fn founder(seed: u64, id: u64, anatomy: Anatomy) -> Self {
        let s = anatomy.sensors();
        let a = anatomy.motors();
        let ligament_rest = anatomy
            .axes
            .iter()
            .enumerate()
            .map(|(index, binding)| match binding.axis {
                super::schema::Axis::Bend => ((index as i32 % 8) - 4) * 28,
                super::schema::Axis::Twist => ((index as i32 % 4) - 2) * 16,
                super::schema::Axis::Jaw => 0,
            })
            .collect();
        // All founders share the same seeded architecture/template. Individual
        // bounded germline mutations create variants, not authored motor roles.
        let mut g = Self {
            version: 1,
            anatomy,
            recurrent: (0..NEURONS * NEURONS)
                .map(|i| draw(seed, b"ctrnn-template-rec", 0, i) % (8 * Q + 1) - 4 * Q)
                .collect(),
            sensory: (0..NEURONS * s)
                .map(|i| draw(seed, b"ctrnn-template-in", 0, i) % (Q + 1) - Q / 2)
                .collect(),
            output: (0..a * NEURONS)
                .map(|i| draw(seed, b"ctrnn-template-out", 0, i) % (Q / 2 + 1) - Q / 4)
                .collect(),
            biases: (0..NEURONS)
                .map(|i| draw(seed, b"ctrnn-template-bias", 0, i) % Q - Q / 2)
                .collect(),
            // Small resting drive. A large constant bias held every joint
            // against its stop, which reads as a servo dance.
            motor_biases: (0..a)
                .map(|i| draw(seed, b"ctrnn-motor-bias", 0, i) % 49 - 24)
                .collect(),
            tau: (0..NEURONS)
                .map(|i| 4 + draw(seed, b"ctrnn-tau", 0, i) % 29)
                .collect(),
            sensory_gains: vec![Q; s],
            motor_gains: vec![Q; a],
            plastic_rates: vec![32; NEURONS],
            strengths: vec![Q; a],
            ligament_stiffness: 6,
            muscle_rate: 3,
            ligament_rest,
        };
        if id != 0 {
            for i in 0..4 {
                g.mutate(seed, id, i);
            }
        }
        g
    }
    pub fn id(&self) -> String {
        blake3::hash(&serde_json::to_vec(self).expect("integer genotype"))
            .to_hex()
            .to_string()
    }
    pub fn mutate(&mut self, seed: u64, id: u64, tick: usize) {
        let b = domain_seed(seed, b"ctrnn-germline-mutation", tick as u64, id);
        let index = u32::from_le_bytes(b[4..8].try_into().unwrap()) as usize;
        let delta: i32 = if b[1] & 1 == 0 { -32 } else { 32 };
        let locus = b[0] % 13;
        if locus == 10 {
            self.ligament_stiffness = (self.ligament_stiffness() + delta.signum()).clamp(2, 24);
            return;
        }
        if locus == 11 {
            self.muscle_rate = (self.muscle_rate() + delta.signum()).clamp(2, 16);
            return;
        }
        if locus == 12 {
            if self.ligament_rest.len() != self.anatomy.axes.len() {
                self.ligament_rest = vec![0; self.anatomy.axes.len()];
            }
            let i = index % self.ligament_rest.len();
            self.ligament_rest[i] = (self.ligament_rest[i] + delta.signum() * 16).clamp(-384, 384);
            return;
        }
        let (values, min, max) = match locus {
            0 => (&mut self.recurrent, -4 * Q, 4 * Q),
            1 => (&mut self.sensory, -2 * Q, 2 * Q),
            2 => (&mut self.output, -2 * Q, 2 * Q),
            3 => (&mut self.biases, -2 * Q, 2 * Q),
            4 => (&mut self.motor_biases, -Q, Q),
            5 => {
                let i = index % self.tau.len();
                self.tau[i] = (self.tau[i] + delta.signum()).clamp(4, 64);
                return;
            }
            6 => (&mut self.sensory_gains, Q / 4, 2 * Q),
            7 => (&mut self.motor_gains, Q / 4, 2 * Q),
            8 => (&mut self.plastic_rates, -128, 128),
            _ => (&mut self.strengths, Q / 4, 2 * Q),
        };
        let i = index % values.len();
        values[i] = (values[i] + delta).clamp(min, max);
    }
    pub fn validate(&self) -> Result<(), Error> {
        self.anatomy.validate()?;
        let s = self.anatomy.sensors();
        let a = self.anatomy.motors();
        let valid = self.version == 1
            && self.recurrent.len() == NEURONS * NEURONS
            && self.sensory.len() == NEURONS * s
            && self.output.len() == a * NEURONS
            && self.biases.len() == NEURONS
            && self.tau.len() == NEURONS
            && self.plastic_rates.len() == NEURONS
            && self.motor_biases.len() == a
            && self.strengths.len() == a
            && self.sensory_gains.len() == s
            && self.motor_gains.len() == a
            && (self.ligament_stiffness == 0 || (2..=24).contains(&self.ligament_stiffness))
            && (self.muscle_rate == 0 || (2..=16).contains(&self.muscle_rate))
            && (self.ligament_rest.is_empty()
                || self.ligament_rest.len() == self.anatomy.axes.len())
            && self
                .ligament_rest
                .iter()
                .all(|angle| angle.unsigned_abs() <= 384);
        if !valid
            || self.recurrent.iter().any(|x| x.unsigned_abs() > 4096)
            || self
                .sensory
                .iter()
                .chain(&self.output)
                .chain(&self.biases)
                .any(|x| x.unsigned_abs() > 2048)
            || self.motor_biases.iter().any(|x| x.unsigned_abs() > 1024)
            || self.tau.iter().any(|x| !(4..=64).contains(x))
            || self.plastic_rates.iter().any(|x| x.unsigned_abs() > 128)
            || self
                .sensory_gains
                .iter()
                .chain(&self.motor_gains)
                .chain(&self.strengths)
                .any(|x| !(256..=2048).contains(x))
        {
            return Err(Error::Invariant("invalid neural genotype".into()));
        }
        Ok(())
    }
    pub fn ligament_stiffness(&self) -> i32 {
        if self.ligament_stiffness == 0 {
            6
        } else {
            self.ligament_stiffness
        }
    }
    pub fn muscle_rate(&self) -> i32 {
        if self.muscle_rate == 0 {
            8
        } else {
            self.muscle_rate
        }
    }
}
pub fn squash(x: i64) -> i32 {
    (i64::from(Q) * x / (i64::from(Q) + x.abs())) as i32
}
fn assimilate(genes: &mut [i32], learned: &[i32], limit: i32) {
    for (gene, weight) in genes.iter_mut().zip(learned) {
        *gene = (*gene + weight / 2).clamp(-limit, limit);
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NeuralState {
    pub genome: ControllerGenome,
    pub activations: Vec<i32>,
    pub integration_residual: Vec<i64>,
    pub plastic: Vec<i32>,
    pub plastic_residual: Vec<i64>,
    pub eligibility: Vec<i32>,
    /// Smoothed actuator request. The body and the valve follow this, so a
    /// one-tick command flicker does not snap a joint or open the nozzle.
    #[serde(default)]
    pub muscle: Vec<i32>,
    /// Lifetime change to motor synapses. Offspring assimilate half of it
    /// into the germline and then start this trace at zero.
    #[serde(default)]
    pub plastic_output: Vec<i32>,
    #[serde(default)]
    pub output_eligibility: Vec<i32>,
    #[serde(default)]
    pub output_residual: Vec<i64>,
    pub sensors: Vec<i32>,
    pub commands: Vec<i32>,
    pub realized_efforts: Vec<i32>,
    pub body: Body,
    pub reward_mean: i32,
    pub reward: i32,
    pub last_energy: u64,
    pub last_intake: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_food_site: Option<crate::Site>,
    pub spent_neural: u64,
    pub spent_actuator: u64,
    pub steps: u64,
    pub neural_updates: u64,
    pub valve_accumulator: i32,
    pub actuator_debt: i64,
    pub changed_connections: u64,
    pub participation: Vec<u64>,
}
impl NeuralState {
    pub fn new(genome: ControllerGenome, energy: u64) -> Self {
        let s = genome.anatomy.sensors();
        let a = genome.anatomy.motors();
        let body = Body::new(&genome.anatomy);
        Self {
            genome,
            body,
            activations: vec![0; NEURONS],
            integration_residual: vec![0; NEURONS],
            plastic: vec![0; NEURONS * NEURONS],
            plastic_residual: vec![0; NEURONS * NEURONS],
            eligibility: vec![0; NEURONS * NEURONS],
            muscle: vec![0; a],
            plastic_output: vec![0; a * NEURONS],
            output_eligibility: vec![0; a * NEURONS],
            output_residual: vec![0; a * NEURONS],
            sensors: vec![0; s],
            commands: vec![0; a],
            realized_efforts: vec![0; a],
            reward_mean: 0,
            reward: 0,
            last_energy: energy,
            last_intake: 0,
            last_food_site: None,
            spent_neural: 0,
            spent_actuator: 0,
            steps: 0,
            neural_updates: 0,
            valve_accumulator: 0,
            actuator_debt: 0,
            changed_connections: 0,
            participation: vec![0; a],
        }
    }
    pub fn integrate(
        &mut self,
        sensors: Vec<i32>,
        rules: &NeuralRules,
        seed: u64,
        id: u64,
        tick: u64,
        energy: u64,
    ) -> (u64, u64) {
        self.steps += 1;
        self.ensure_motor_memory();
        let fee = u64::from(self.steps.is_multiple_of(8));
        self.sensors = sensors;
        self.spent_neural = fee.min(energy);
        self.spent_actuator = 0;
        if energy == 0 || energy < fee {
            self.commands.fill(0);
            self.realized_efforts.fill(0);
            if rules.version >= 2 {
                self.muscle.fill(0);
            } else {
                self.slew_muscle();
                if rules.spinal_wave {
                    self.spinal_wave(energy, tick);
                }
            }
            return (0, 0);
        }
        if !rules.recurrence {
            self.activations.fill(0);
            self.integration_residual.fill(0);
        }
        for _ in 0..4 {
            let prior: Vec<_> = self
                .activations
                .iter()
                .map(|a| squash(i64::from(*a)))
                .collect();
            for i in 0..NEURONS {
                let recurrent = if rules.recurrence {
                    (0..NEURONS)
                        .map(|j| {
                            i64::from(
                                self.genome.recurrent[i * NEURONS + j]
                                    + self.plastic[i * NEURONS + j],
                            ) * i64::from(prior[j])
                                / i64::from(Q)
                        })
                        .sum::<i64>()
                } else {
                    0
                };
                let sensory = self
                    .sensors
                    .iter()
                    .enumerate()
                    .map(|(s, x)| {
                        i64::from(self.genome.sensory[i * self.sensors.len() + s])
                            * i64::from(*x)
                            * i64::from(self.genome.sensory_gains[s])
                            / i64::from(Q * Q)
                    })
                    .sum::<i64>();
                let drive = (i64::from(self.genome.biases[i]) + recurrent + sensory)
                    .clamp(-8 * i64::from(Q), 8 * i64::from(Q));
                let divisor = i64::from(4 * self.genome.tau[i]);
                self.integration_residual[i] += drive - i64::from(self.activations[i]);
                self.activations[i] =
                    (i64::from(self.activations[i]) + self.integration_residual[i] / divisor)
                        .clamp(-8 * i64::from(Q), 8 * i64::from(Q)) as i32;
                self.integration_residual[i] %= divisor;
            }
        }
        let firing: Vec<_> = self
            .activations
            .iter()
            .map(|x| squash(i64::from(*x)))
            .collect();
        for a in 0..self.commands.len() {
            self.commands[a] = if rules.policy == Policy::Random {
                let b = domain_seed(
                    seed,
                    b"neural-random-baseline",
                    tick,
                    id ^ (a as u64).rotate_left(32),
                );
                (u32::from_le_bytes(b[..4].try_into().unwrap()) % 2049) as i32 - Q
            } else {
                let drive = i64::from(self.genome.motor_biases[a])
                    + (0..NEURONS)
                        .map(|n| {
                            let learned = i64::from(self.plastic_output[a * NEURONS + n]);
                            (i64::from(self.genome.output[a * NEURONS + n]) + learned)
                                * i64::from(firing[n])
                                / i64::from(Q)
                        })
                        .sum::<i64>();
                (i64::from(squash(drive)) * i64::from(self.genome.motor_gains[a]) / i64::from(Q))
                    .clamp(-i64::from(Q), i64::from(Q)) as i32
            };
        }
        // Version 1 bills raw neural output before the authored wave, retained
        // for historical replay. Version 2 bills the actual requested drive.
        if rules.version >= 2 {
            self.slew_muscle();
            if rules.spinal_wave {
                self.spinal_wave(energy, tick);
            }
        }
        let billed = if rules.version >= 2 {
            self.actuator_drive()
        } else {
            &self.commands
        };
        let effort: i64 = billed[..self.genome.anatomy.axes.len()]
            .iter()
            .enumerate()
            .map(|(i, x)| {
                (i64::from(*x) * i64::from(self.genome.strengths[i]) / i64::from(Q)).abs()
            })
            .sum::<i64>()
            / 8;
        let debt = self.actuator_debt + effort;
        let motor_fee = (debt / 4096) as u64;
        if motor_fee > energy - fee {
            self.commands.fill(0);
            self.realized_efforts.fill(0);
            if rules.version >= 2 {
                self.muscle.fill(0);
            }
        } else {
            self.actuator_debt = debt % 4096;
            self.spent_actuator = motor_fee;
        }
        if rules.version == 1 {
            self.slew_muscle();
            if rules.spinal_wave {
                self.spinal_wave(energy, tick);
            }
        }
        self.neural_updates += 1;
        (self.spent_neural, self.spent_actuator)
    }
    fn ensure_motor_memory(&mut self) {
        let motors = self.genome.anatomy.motors();
        let synapses = motors * NEURONS;
        if self.muscle.len() != motors {
            self.muscle.resize(motors, 0);
        }
        if self.plastic_output.len() != synapses {
            self.plastic_output = vec![0; synapses];
            self.output_eligibility = vec![0; synapses];
            self.output_residual = vec![0; synapses];
        }
    }
    fn slew_muscle(&mut self) {
        self.ensure_motor_memory();
        let follow = self.genome.muscle_rate();
        for (muscle, command) in self.muscle.iter_mut().zip(&self.commands) {
            let gap = (*command - *muscle).unsigned_abs();
            // A small correction stays smooth. A real contraction arrives in a few ticks.
            let step = if gap > 128 { 2 } else { follow.max(2) };
            *muscle += (*command - *muscle) / step;
        }
    }
    /// The spine always works a traveling wave. Hunger throws it harder.
    /// Jaws and the valve are left alone, so a meal is not a vomit and a
    /// spit is not how the body moves. This is an authored motor drive, not
    /// a path to food and not evidence of foraging.
    fn spinal_wave(&mut self, energy: u64, tick: u64) {
        if energy == 0 || self.muscle.is_empty() {
            return;
        }
        let throwing = energy < 96 || (self.steps > 8 && self.last_intake == 0);
        let amplitude = if throwing { 960 } else { 640 };
        let pull = if throwing { 720 } else { 512 };
        let phase = (tick % 12) as i32;
        for (index, binding) in self.genome.anatomy.axes.iter().enumerate() {
            if binding.axis != super::schema::Axis::Bend || index >= self.muscle.len() {
                continue;
            }
            let wave = if ((index as i32 + phase) % 6) < 3 {
                1
            } else {
                -1
            };
            let target = amplitude * wave;
            self.muscle[index] += (target - self.muscle[index]) * pull / 1024;
        }
    }
    /// What the joints and the valve actually receive this tick.
    pub fn actuator_drive(&self) -> &[i32] {
        if self.muscle.len() == self.commands.len() {
            &self.muscle
        } else {
            &self.commands
        }
    }
    pub fn feedback(&mut self, energy: u64, intake: u64, rules: &NeuralRules) {
        let raw =
            ((i128::from(energy) - i128::from(self.last_energy)) * 128).clamp(-1024, 1024) as i32;
        self.reward = raw;
        let modulation = (raw - self.reward_mean).clamp(-1024, 1024);
        self.reward_mean += (raw - self.reward_mean) / 16;
        self.last_energy = energy;
        self.last_intake = intake;
        self.ensure_motor_memory();
        let firing: Vec<_> = self
            .activations
            .iter()
            .map(|x| squash(i64::from(*x)))
            .collect();
        for i in 0..NEURONS {
            for j in 0..NEURONS {
                let k = i * NEURONS + j;
                self.eligibility[k] =
                    (self.eligibility[k] * 7 / 8 + firing[i] * firing[j] / (8 * Q)).clamp(-Q, Q);
                if rules.plasticity {
                    self.plastic_residual[k] += i64::from(self.genome.plastic_rates[i])
                        * i64::from(modulation)
                        * i64::from(self.eligibility[k]);
                    let denominator = i64::from(Q) * i64::from(Q) * 8;
                    let change = self.plastic_residual[k] / denominator;
                    let next = (i64::from(self.plastic[k]) + change).clamp(-512, 512) as i32;
                    self.changed_connections += u64::from(next != self.plastic[k]);
                    self.plastic[k] = next;
                    self.plastic_residual[k] %= denominator;
                }
            }
        }
        // Motor synapses use the same authored three-factor rule. A meal can
        // strengthen the drive that preceded it. Birth writes half of that
        // change into the genes. The write is not evidence of useful foraging.
        let motors = self.genome.anatomy.motors();
        for a in 0..motors {
            let post = self.actuator_drive()[a];
            for (n, firing_value) in firing.iter().enumerate() {
                let k = a * NEURONS + n;
                self.output_eligibility[k] = (self.output_eligibility[k] * 7 / 8
                    + post * firing_value / (8 * Q))
                    .clamp(-Q, Q);
                if rules.plasticity {
                    self.output_residual[k] += i64::from(self.genome.plastic_rates[n])
                        * i64::from(modulation)
                        * i64::from(self.output_eligibility[k]);
                    let denominator = i64::from(Q) * i64::from(Q) * 8;
                    let change = self.output_residual[k] / denominator;
                    let next = (i64::from(self.plastic_output[k]) + change).clamp(-512, 512) as i32;
                    self.changed_connections += u64::from(next != self.plastic_output[k]);
                    self.plastic_output[k] = next;
                    self.output_residual[k] %= denominator;
                }
            }
        }
    }
    pub fn offspring(&self, seed: u64, id: u64, tick: u64, energy: u64, mutation: bool) -> Self {
        let mut genome = self.genome.clone();
        // Authored assimilation. Half of the lifetime weights, integer-truncated,
        // is written into the child genes before mutation. The child still
        // starts with empty traces, so it can learn further. A weight of 1
        // contributes nothing. This is not a demonstrated evolutionary result.
        assimilate(&mut genome.recurrent, &self.plastic, 4 * Q);
        assimilate(&mut genome.output, &self.plastic_output, 2 * Q);
        if mutation {
            for k in 0..4 {
                genome.mutate(seed, id, tick as usize + k);
            }
        }
        Self::new(genome, energy)
    }
    pub fn validate(&self) -> Result<(), Error> {
        self.genome.validate()?;
        let a = self.genome.anatomy.motors();
        let s = self.genome.anatomy.sensors();
        let motor_synapses = a * NEURONS;
        if self.reward.unsigned_abs() > 1024
            || self.reward_mean.unsigned_abs() > 1024
            || self.neural_updates > self.steps
            || self
                .integration_residual
                .iter()
                .zip(&self.genome.tau)
                .any(|(r, t)| r.unsigned_abs() >= (4 * *t) as u64)
            || self
                .plastic_residual
                .iter()
                .any(|r| r.unsigned_abs() >= (Q as u64 * Q as u64 * 8))
            || self.activations.len() != NEURONS
            || self.integration_residual.len() != NEURONS
            || self.plastic.len() != NEURONS * NEURONS
            || self.plastic_residual.len() != NEURONS * NEURONS
            || self.eligibility.len() != NEURONS * NEURONS
            || self.muscle.len() != a
            || self.plastic_output.len() != motor_synapses
            || self.output_eligibility.len() != motor_synapses
            || self.output_residual.len() != motor_synapses
            || self.commands.len() != a
            || self.realized_efforts.len() != a
            || self.participation.len() != a
            || self.sensors.len() != s
            || self.activations.iter().any(|x| x.unsigned_abs() > 8192)
            || self.plastic.iter().any(|x| x.unsigned_abs() > 512)
            || self.eligibility.iter().any(|x| x.unsigned_abs() > 1024)
            || self.muscle.iter().any(|x| x.unsigned_abs() > 1024)
            || self.plastic_output.iter().any(|x| x.unsigned_abs() > 512)
            || self
                .output_eligibility
                .iter()
                .any(|x| x.unsigned_abs() > 1024)
            || self
                .output_residual
                .iter()
                .any(|r| r.unsigned_abs() >= (Q as u64 * Q as u64 * 8))
            || self
                .commands
                .iter()
                .chain(&self.sensors)
                .chain(&self.realized_efforts)
                .any(|x| x.unsigned_abs() > 2048)
            || !(0..1024).contains(&self.valve_accumulator)
            || !(0..4096).contains(&self.actuator_debt)
        {
            return Err(Error::Invariant("invalid neural lifetime state".into()));
        }
        self.body.validate(&self.genome.anatomy)
    }
}
