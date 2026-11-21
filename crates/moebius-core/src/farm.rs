//! M1: a bounded adaptive motor policy and articulated, inertial swimmer.
//! Strokes exchange momentum with an implicit medium. This is an authored thrust
//! approximation, not vacuum propulsion, fluid dynamics, or learned competence.
use crate::geom::site_index;
use crate::prng::domain_seed;
use crate::{Cell, Dims, Dir, Error, Site, step_site};
use serde::{Deserialize, Serialize};

pub const SUBSITE: i32 = 1024;
pub const SPINE_JOINTS: usize = 12;
pub const JOINTS: usize = SPINE_JOINTS + 2;
pub const SECRETE_ACTION: usize = 25;
pub const DIG_START: usize = 26;
pub const ACTIONS: usize = 32; // coast; 2 body sections × 6 headings × 2 strokes; secretion; 6 bites
pub const JOINT_LIMIT: i16 = 192;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrainRig {
    pub id: u64,
    pub parent: u64,
    pub generation: u32,
    pub weights: [[i16; 5]; ACTIONS],
    pub eligibility: [[i16; 5]; ACTIONS],
    pub joints: [i16; JOINTS],
    pub twists: [i16; JOINTS],
    pub last_dig: u64,
    pub last_action: u8,
    pub offset: [i32; 3],
    pub velocity: [i32; 3],
    pub last_birth: u64,
    pub updates: u64,
    pub last_meal: u64,
    pub reward_mean: i16,
}

impl BrainRig {
    pub fn new(cell: &Cell, seed: u64) -> Self {
        let bytes = domain_seed(seed, b"brain", 0, cell.id);
        let mut weights = [[0; 5]; ACTIONS];
        for (action, row) in weights.iter_mut().enumerate() {
            for (feature, weight) in row.iter_mut().enumerate() {
                *weight = i16::from(bytes[(action * 5 + feature) % 32] % 9) - 4;
            }
            // A weak innate nutrient response; motor preferences must adapt.
            row[0] += 6;
        }
        Self {
            id: cell.id,
            parent: 0,
            generation: 0,
            weights,
            eligibility: [[0; 5]; ACTIONS],
            joints: [0; JOINTS],
            twists: [0; JOINTS],
            last_dig: 0,
            last_action: 0,
            offset: [0; 3],
            velocity: [0; 3],
            last_birth: 0,
            updates: 0,
            last_meal: 0,
            reward_mean: 0,
        }
    }

    pub fn choose(
        &self,
        cell: &Cell,
        field: &[u64],
        occupied: &[u64],
        flesh: &std::collections::BTreeMap<Site, u64>,
        dims: Dims,
        clock: (u64, u64),
    ) -> Result<(usize, [i16; 5]), Error> {
        let (seed, tick) = clock;
        let current = field[site_index(cell.site, dims)?].min(64) as i16;
        let mut features = [[0; 5]; ACTIONS];
        let mut best = (i32::MIN, 0usize);
        for (action, sensed) in features.iter_mut().enumerate() {
            let dir = action_dir(action)?;
            let site = step_site(cell.site, dir, dims)?;
            let index = site_index(site, dims)?;
            let delta = dir.delta();
            let velocity = [delta.0, delta.1, delta.2]
                .iter()
                .zip(self.velocity)
                .map(|(d, v)| d * v)
                .sum::<i32>()
                / 32;
            let stroke = action_stroke(action);
            let available = if action == 0 || action == SECRETE_ACTION || action >= DIG_START {
                0
            } else {
                let first = ((action - 1) / 12) * 6;
                ((stroke * 160 - self.joints[first]) / 16).abs()
            };
            *sensed = [
                (field[index].min(64) as i16 - current).clamp(-16, 16),
                velocity.clamp(-16, 16) as i16,
                available,
                if occupied[index] != 0 && occupied[index] != cell.id {
                    -16
                } else {
                    let density = flesh.get(&site).copied().unwrap_or(0).min(8) as i16;
                    if action >= DIG_START {
                        density
                    } else {
                        -density
                    }
                },
                4,
            ];
            let score = self.weights[action]
                .iter()
                .zip(*sensed)
                .map(|(w, f)| i32::from(*w) * i32::from(f))
                .sum::<i32>();
            if score > best.0 {
                best = (score, action);
            }
        }
        let random = domain_seed(seed, b"motor-explore", tick, cell.id);
        let action = if random[0] < 64 {
            usize::from(random[1]) % ACTIONS
        } else {
            best.1
        };
        Ok((action, features[action]))
    }

    /// Joint displacement produces an effective impulse. Without a stroke,
    /// velocity persists, with 1/128 linear drag per tick from the medium.
    pub fn actuate(&mut self, action: usize, from: Site, dims: Dims) -> Result<(Site, u64), Error> {
        self.last_action = action as u8;
        let mut effort = 0;
        if (1..SECRETE_ACTION).contains(&action) {
            let stroke = action_stroke(action);
            let section = (action - 1) / 12;
            let dir = action_dir(action)?.delta();
            let plane = if dir.1 != 0 { -1 } else { 1 };
            // Separate front/rear commands produce broad traveling S-bends and
            // helical poses. The other section retains its motor state.
            for index in section * 6..section * 6 + 6 {
                let shape = [1, 1, 0, -1, -1, 0][index % 6];
                let target = stroke * shape * 160;
                let bend = (target - self.joints[index]).clamp(-72, 72);
                let twist_target = stroke * plane * [1, 0, -1, -1, 0, 1][index % 6] * 96;
                let twist = (twist_target - self.twists[index]).clamp(-48, 48);
                self.joints[index] += bend;
                self.twists[index] += twist;
                effort += i32::from(bend.abs()) + i32::from(twist.abs()) / 2;
            }
        } else if action >= DIG_START {
            // Opening is not excavation: the next closing stroke can shear.
            let target = if self.joints[SPINE_JOINTS] <= 0 {
                JOINT_LIMIT
            } else {
                -64
            };
            for joint in &mut self.joints[SPINE_JOINTS..] {
                effort += i32::from((target - *joint).abs());
                *joint = target;
            }
        }
        let dir = if (1..SECRETE_ACTION).contains(&action) {
            action_dir(action)?.delta()
        } else {
            (0, 0, 0)
        };
        for (axis, component) in [dir.0, dir.1, dir.2].into_iter().enumerate() {
            let impulse = component * effort / 16;
            self.velocity[axis] = ((self.velocity[axis] + impulse) * 127 / 128).clamp(-256, 256);
            self.offset[axis] += self.velocity[axis];
        }
        let mut coordinates = [from.x, from.y, from.z];
        for (axis, extent) in [dims.width, dims.height, dims.depth]
            .into_iter()
            .enumerate()
        {
            if self.offset[axis] >= SUBSITE / 2 {
                coordinates[axis] = (coordinates[axis] + 1) % extent;
                self.offset[axis] -= SUBSITE;
            } else if self.offset[axis] < -SUBSITE / 2 {
                coordinates[axis] = (coordinates[axis] + extent - 1) % extent;
                self.offset[axis] += SUBSITE;
            }
        }
        Ok((
            Site {
                x: coordinates[0],
                y: coordinates[1],
                z: coordinates[2],
            },
            u64::from(effort > 0),
        ))
    }

    pub fn learn(&mut self, action: usize, features: [i16; 5], reward: i16) {
        let advantage = (reward - self.reward_mean / 8).clamp(-4, 4);
        self.reward_mean = (self.reward_mean * 7 / 8 + reward).clamp(-256, 256);
        // Decaying eligibility assigns some later feeding feedback to recent
        // motor and secretion choices; it does not guarantee useful building.
        for row in &mut self.eligibility {
            for trace in row {
                *trace = *trace * 7 / 8;
            }
        }
        for (trace, feature) in self.eligibility[action].iter_mut().zip(features) {
            *trace = (*trace + feature.signum() * 8).clamp(-64, 64);
        }
        for (weights, traces) in self.weights.iter_mut().zip(self.eligibility) {
            for (weight, trace) in weights.iter_mut().zip(traces) {
                *weight = (*weight + trace * advantage / 8).clamp(-128, 128);
            }
        }
        self.updates += 1;
    }

    pub fn child(&self, id: u64, seed: u64, tick: u64) -> Self {
        let mut child = self.clone();
        child.id = id;
        child.parent = self.id;
        child.generation += 1;
        child.eligibility = [[0; 5]; ACTIONS];
        child.offset = [0; 3];
        child.joints = [0; JOINTS];
        child.twists = [0; JOINTS];
        child.last_dig = 0;
        child.last_action = 0;
        child.last_birth = tick;
        child.updates = 0;
        child.last_meal = 0;
        child.reward_mean = 0;
        let random = domain_seed(seed, b"motor-mutation", tick, id);
        let action = usize::from(random[0]) % ACTIONS;
        let feature = usize::from(random[1]) % 5;
        let delta = if random[2] & 1 == 0 { -1 } else { 1 };
        child.weights[action][feature] = (child.weights[action][feature] + delta).clamp(-128, 128);
        child
    }
}

pub fn action_dir(action: usize) -> Result<Dir, Error> {
    Dir::from_roll(if action == 0 || action == SECRETE_ACTION {
        0
    } else if action >= DIG_START {
        (action - DIG_START + 1) as u32
    } else {
        (((action - 1) % 12) / 2 + 1) as u32
    })
}
fn action_stroke(action: usize) -> i16 {
    if action % 2 == 1 { 1 } else { -1 }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FleshDeposit {
    pub site: Site,
    pub amount: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FarmPayload {
    pub brains: Vec<BrainRig>,
    pub flesh: Vec<FleshDeposit>,
}

/// Deposits sharing a face weld immediately, including across periodic seams.
/// A connected tissue of at least four units is a contact surface throughout.
pub fn fused_surface(
    flesh: &std::collections::BTreeMap<Site, u64>,
    dims: Dims,
) -> Result<std::collections::BTreeSet<Site>, Error> {
    let mut unseen: std::collections::BTreeSet<_> = flesh.keys().copied().collect();
    let mut solid = std::collections::BTreeSet::new();
    while let Some(start) = unseen.pop_first() {
        let mut component = vec![start];
        let mut index = 0;
        let mut total = flesh[&start];
        while index < component.len() {
            let site = component[index];
            index += 1;
            for roll in 1..=6 {
                let neighbor = step_site(site, Dir::from_roll(roll)?, dims)?;
                if unseen.remove(&neighbor) {
                    total += flesh[&neighbor];
                    component.push(neighbor);
                }
            }
        }
        if total >= 4 {
            solid.extend(component);
        }
    }
    Ok(solid)
}
