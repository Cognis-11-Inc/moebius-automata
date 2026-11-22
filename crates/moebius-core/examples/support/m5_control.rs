//! Finite diagnostic oracle and independently authored sensor-only comparator.
//! No production module imports this file.
use super::support::*;
use moebius_core::{Site, World};
use serde_json::{Value, json};

pub fn actions(w: &World) -> Vec<Vec<i32>> {
    let mut commands = vec![vec![0; 30]];
    for pattern in [
        "bend",
        "front",
        "back",
        "opposed",
        "twist",
        "bend_twist",
        "jaw",
    ] {
        for amplitude in [-1024, 1024] {
            commands.push(
                Schedule {
                    pattern: pattern.into(),
                    amplitude,
                    pulse: 1,
                    period: 0,
                    recovery: 0,
                    ablate: None,
                }
                .commands(w, 0),
            );
        }
    }
    commands
}
fn clearance(w: &World, target: Site) -> f64 {
    let mouth = mouth(w);
    moebius_core::feeding::mouth_sites(target, w.dims())
        .into_iter()
        .map(|site| {
            let d = delta(mouth, point(site)).map(|x| (x.abs() - 511).max(0));
            norm(d)
        })
        .min_by(f64::total_cmp)
        .unwrap()
}
fn score(w: &World, target: Site) -> f64 {
    if w.live_count() == 0 {
        return 1e6;
    }
    let state = w.brains()[&ID].neural.as_ref().unwrap();
    if state.last_food_site == Some(target) && state.last_intake > 0 {
        return -1000. + w.tick_index() as f64 / 10000.;
    }
    clearance(w, target) + 0.08 * norm(delta(mouth(w), point(target)))
}
#[derive(Clone)]
struct Node {
    world: World,
    path: Vec<usize>,
    score: f64,
}
/// Target-aware finite beam search. Privileged world state stays inside this
/// diagnostic module. Results establish only the tested search's competence.
pub fn oracle(w: &World, target: Site, log: &mut Vec<Value>) -> Result<Vec<i32>> {
    let library = actions(w);
    let mut beam = vec![Node {
        world: w.clone(),
        path: Vec::new(),
        score: score(w, target),
    }];
    let mut evaluations = Vec::new();
    for depth in 0..5 {
        let mut next = Vec::new();
        for node in &beam {
            for (index, command) in library.iter().enumerate() {
                let mut candidate = node.world.clone();
                let mut path = node.path.clone();
                path.push(index);
                let mut error = None;
                for _ in 0..4 {
                    if candidate.live_count() == 0 {
                        break;
                    }
                    if let Err(e) = step(&mut candidate, Some(command)) {
                        error = Some(e.to_string());
                        break;
                    }
                }
                let value = score(&candidate, target);
                evaluations.push(json!({"depth":depth,"path":path,"score":value,"alive":candidate.live_count(),"error":error}));
                if error.is_none() && candidate.live_count() > 0 {
                    next.push(Node {
                        world: candidate,
                        path,
                        score: value,
                    });
                }
            }
        }
        next.sort_by(|a, b| {
            a.score
                .total_cmp(&b.score)
                .then_with(|| a.path.cmp(&b.path))
        });
        next.truncate(4);
        if next.is_empty() {
            break;
        }
        beam = next;
        if beam[0].score < -999. {
            break;
        }
    }
    let winner = beam
        .iter()
        .min_by(|a, b| {
            a.score
                .total_cmp(&b.score)
                .then_with(|| a.path.cmp(&b.path))
        })
        .unwrap();
    let action = winner.path.first().copied().unwrap_or(0);
    log.push(
        json!({"tick":w.tick_index(),"chosen_action":action,"winning_path":winner.path,
        "predicted_score":winner.score,"evaluations":evaluations}),
    );
    Ok(library[action].clone())
}

#[derive(Default)]
pub struct LocalController {
    phase: u64,
    last_chemical: [i32; 3],
}
impl LocalController {
    /// Only ordinary scalar sensor channels and this controller's own clock and
    /// previous chemical differential are available. No World, target, position,
    /// orientation, map, seed-to-target rule or forward simulator is accepted.
    pub fn commands(&mut self, sensors: &[i32]) -> Vec<i32> {
        let mut cmd = vec![0; 30];
        let gradient: [i32; 3] = std::array::from_fn(|a| sensors[93 + 2 * a] - sensors[94 + 2 * a]);
        let visible = gradient.iter().any(|v| *v != 0);
        if visible {
            self.last_chemical = gradient;
        }
        // Deliberately simple authored local reflex, not an extracted oracle
        // sequence. A blind controller rests rather than receiving a target.
        if visible {
            let g = self.last_chemical;
            let axis = (0..3).max_by_key(|a| g[*a].abs()).unwrap();
            let contracting = self.phase % 24 < 8;
            let amplitude = if contracting { 1024 } else { 0 };
            let sign = g[axis].signum();
            for joint in 0..12 {
                match axis {
                    0 => cmd[2 * joint] = -sign * amplitude,
                    1 => {
                        cmd[2 * joint] = sign * amplitude;
                        cmd[2 * joint + 1] = amplitude;
                    }
                    _ => {
                        cmd[2 * joint] = amplitude;
                        cmd[2 * joint + 1] = sign * amplitude;
                    }
                }
            }
        }
        self.phase += 1;
        cmd
    }
}
/// Brake oracle minimizes predicted actual linear momentum, not target distance.
pub fn brake(w: &World) -> Result<Vec<i32>> {
    let library = actions(w);
    let mut best = (f64::INFINITY, library[0].clone());
    for command in library {
        let mut candidate = w.clone();
        for _ in 0..4 {
            if candidate.live_count() == 0 {
                break;
            }
            step(&mut candidate, Some(&command))?;
        }
        if let Some(s) = candidate.brains().get(&ID).and_then(|b| b.neural.as_ref()) {
            let value = s.body.momentum.iter().map(|v| v.abs() as f64).sum::<f64>();
            if value < best.0 {
                best = (value, command);
            }
        }
    }
    Ok(best.1)
}

#[derive(Default)]
pub struct LocalModelController {
    last_direction: [i32; 3],
    blind_ticks: u32,
}
impl LocalModelController {
    /// Sensor-only, approximate body-local motor model. This independent
    /// gradient/proprioception reflex has no World or target-position API.
    pub fn commands(&mut self, sensors: &[i32]) -> Vec<i32> {
        use moebius_core::neural::{
            Q,
            body::{Body, rotate},
            schema::{Anatomy, Axis},
        };
        let anatomy = Anatomy::reference();
        let mut body = Body::new(&anatomy);
        let rest: Vec<i32> = anatomy
            .axes
            .iter()
            .enumerate()
            .map(|(i, b)| match b.axis {
                Axis::Bend => ((i as i32 % 8) - 4) * 28,
                Axis::Twist => ((i as i32 % 4) - 2) * 16,
                Axis::Jaw => 0,
            })
            .collect();
        let mut estimated_drive = vec![0; 30];
        let mut hold = vec![0; 30];
        for (i, binding) in anatomy.axes.iter().enumerate() {
            let limit = binding.min.abs().max(binding.max);
            body.angles[i] = sensors[3 * i] * limit / Q;
            body.rates[i] = sensors[3 * i + 1] * 64 / Q;
            body.efforts[i] = sensors[3 * i + 2] * 2048 / Q;
            estimated_drive[i] = (body.efforts[i] + (body.angles[i] - rest[i]) / 6).clamp(-Q, Q);
            hold[i] = ((body.angles[i] - rest[i]) / 6 - 4 * body.rates[i]).clamp(-Q, Q);
        }
        if sensors[88] > 0 {
            return hold;
        }
        let chemical: [i32; 3] = std::array::from_fn(|a| sensors[93 + 2 * a] - sensors[94 + 2 * a]);
        if chemical.iter().any(|v| *v != 0) {
            let largest = chemical.iter().map(|v| v.abs()).max().unwrap().max(1);
            self.last_direction = chemical.map(|v| v * Q / largest);
            self.blind_ticks = 0;
        } else {
            self.blind_ticks += 1;
            let angular: [i32; 3] = std::array::from_fn(|a| -sensors[84 + a] * 64 / Q);
            self.last_direction =
                rotate(self.last_direction.map(i64::from), angular).map(|v| v as i32);
        }
        if self.blind_ticks > 8 || self.last_direction == [0; 3] {
            return vec![0; 30];
        }
        body.velocity = std::array::from_fn(|a| sensors[78 + a] * 256 / Q);
        body.momentum = body.velocity.map(|v| i64::from(v) * 16);
        let mut library = vec![hold, vec![0; 30]];
        // Candidate torques are shared actuator primitives, not an oracle path.
        for group in 0..6 {
            for amplitude in [-1024, 1024] {
                let mut command = vec![0; 30];
                for joint in 0..12 {
                    if group == 0
                        || group == 1 && joint >= 6
                        || group == 2 && joint < 6
                        || group == 5
                    {
                        command[2 * joint] = amplitude;
                    }
                    if group == 3 || group == 5 {
                        command[2 * joint + 1] = amplitude;
                    }
                    if group == 4 && joint >= 10 {
                        command[2 * joint] = amplitude;
                    }
                }
                library.push(command);
            }
        }
        let mut best = (f64::NEG_INFINITY, vec![0; 30]);
        for command in library {
            let mut predicted = body.clone();
            let mut muscle = estimated_drive.clone();
            let mut translated = [0i64; 3];
            for _ in 0..6 {
                for (m, c) in muscle.iter_mut().zip(&command) {
                    let follow = if (*c - *m).unsigned_abs() > 128 { 2 } else { 3 };
                    *m += (*c - *m) / follow;
                }
                predicted.step_with_law(&anatomy, &muscle, &[Q; 30], 8, &[], 6, &rest, 2);
                for (a, item) in translated.iter_mut().enumerate() {
                    *item += i64::from(predicted.velocity[a]);
                }
            }
            let endpoint = mouth_pose(translated, &predicted, &anatomy);
            let value = endpoint
                .iter()
                .zip(self.last_direction)
                .map(|(p, g)| *p as f64 * f64::from(g))
                .sum::<f64>();
            if value > best.0 {
                best = (value, command);
            }
        }
        best.1
    }
}
