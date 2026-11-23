//! M6 research harness. No production gait or target-coordinate sensor.
use moebius_core::{
    SimConfig, Site, World, config_hash,
    neural::{ControllerGenome, NeuralState, Policy, body::Body},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
};
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub const BASE_SEED: u64 = 20261122;
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Cue {
    pub axis: usize,
    pub first_sign: i32,
    pub switch_tick: u64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Episode {
    pub seed: u64,
    pub orientation: [i32; 3],
    pub pose: i32,
    pub drag: u32,
    pub ticks: u64,
    pub energy: u64,
    pub plasticity: bool,
    pub feedback: bool,
    #[serde(default)]
    pub cue: Option<Cue>,
}
impl Episode {
    pub fn train(seed: u64) -> Self {
        Self {
            seed,
            orientation: match seed {
                102 => [0, 0, 1608],
                103 => [0, 512, -700],
                _ => [0; 3],
            },
            pose: 0,
            drag: if seed == 103 { 32 } else { 8 },
            ticks: 128,
            energy: 4096,
            plasticity: false,
            feedback: true,
            cue: None,
        }
    }
    pub fn held(seed: u64) -> Self {
        let mut rng = Rng(seed);
        Self {
            seed,
            orientation: [
                (rng.below(1000) as i32) - 500,
                (rng.below(1600) as i32) - 800,
                (rng.below(5000) as i32) - 2500,
            ],
            pose: 128,
            drag: if seed.is_multiple_of(2) { 8 } else { 32 },
            ticks: 256,
            energy: 4096,
            plasticity: false,
            feedback: true,
            cue: None,
        }
    }
}
#[derive(Clone)]
pub struct Rng(pub u64);
impl Rng {
    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
    pub fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
    pub fn signed(&mut self, max: i32) -> i32 {
        self.below((max * 2 + 1) as u64) as i32 - max
    }
}
pub fn config(e: &Episode) -> SimConfig {
    let mut c = SimConfig::nursery_farm_256();
    c.width = 16;
    c.height = 16;
    c.depth = 16;
    c.initial_population = 1;
    c.initial_energy = e.energy;
    c.initial_reservoir = 0;
    c.inflow_per_tick = 0;
    c.inject_per_tick = 0;
    c.feeding.as_mut().unwrap().initial_stock = 0;
    c.feeding.as_mut().unwrap().spacing = 8;
    c.farm.as_mut().unwrap().max_population = 1;
    let n = c.neural.as_mut().unwrap();
    n.drag = e.drag;
    n.plasticity = e.plasticity;
    n.physiological_feedback = e.feedback;
    if e.cue.is_some() {
        n.chemical_sensing = moebius_core::neural::ChemicalSensing::LocalLobesV1;
        c.uptake_per_tick = 0;
    }
    c
}
pub fn original() -> ControllerGenome {
    World::genesis(&config(&Episode::train(101)), BASE_SEED)
        .unwrap()
        .brains()[&1]
        .neural
        .as_ref()
        .unwrap()
        .genome
        .clone()
}
pub fn world(
    g: &ControllerGenome,
    e: &Episode,
    retained: Option<&NeuralState>,
    carry_fast: bool,
) -> Result<World> {
    g.validate()?;
    let c = config(e);
    let mut s = World::genesis(&c, e.seed)?.snapshot();
    s.cells[0].site = Site { x: 8, y: 8, z: 8 };
    let mut n = if carry_fast {
        retained.ok_or("missing retained state")?.clone()
    } else {
        let mut fresh = NeuralState::new(g.clone(), e.energy);
        if let Some(old) = retained {
            fresh.plastic = old.plastic.clone();
            fresh.plastic_output = old.plastic_output.clone();
        }
        fresh
    };
    n.genome = g.clone();
    n.body = Body::new(&g.anatomy);
    n.body.orientation = e.orientation;
    let mut rng = Rng(e.seed ^ 0x706f7365);
    for (i, a) in n.body.angles.iter_mut().enumerate() {
        *a = rng
            .signed(e.pose)
            .clamp(g.anatomy.axes[i].min, g.anatomy.axes[i].max);
    }
    n.body.centroid = n.body.local_geometry(&g.anatomy).1;
    n.last_energy = e.energy;
    n.last_intake = 0;
    n.last_food_site = None;
    n.steps = 0;
    n.neural_updates = 0;
    n.participation.fill(0);
    if let Some(cue) = &e.cue {
        s.nutrient = cue_field(47104, cue.axis, cue.first_sign);
        s.material_initial = 47104;
    }
    s.brains[0].neural = Some(n);
    s.brains[0].sync_neural_pose();
    Ok(World::restore(&c, &s)?)
}
pub fn delta(a: [i64; 3], b: [i64; 3]) -> [f64; 3] {
    std::array::from_fn(|i| ((b[i] - a[i] + 8192).rem_euclid(16384) - 8192) as f64 / 1024.)
}
pub fn norm(v: [f64; 3]) -> f64 {
    v.iter().map(|x| x * x).sum::<f64>().sqrt()
}
pub fn position(w: &World) -> [i64; 3] {
    let c = w.cells().next().unwrap();
    let b = &w.brains()[&1];
    let p = [c.site.x, c.site.y, c.site.z];
    std::array::from_fn(|a| i64::from(p[a]) * 1024 + i64::from(b.offset[a]))
}
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Neural,
    Random,
    Rest,
    Pulse,
    Periodic,
    BabbleIndividual,
    BabbleReverse,
    BabbleSimultaneous,
    BabbleSequential,
    BabblePhase,
}
pub fn commands(mode: &Mode, t: u64, seed: u64) -> Option<Vec<i32>> {
    if *mode == Mode::Neural || *mode == Mode::Random {
        return None;
    }
    let mut c = vec![0; 30];
    let mut r = Rng(seed ^ ((t / 8).wrapping_mul(0x9e3779b97f4a7c15)));
    match mode {
        Mode::Pulse | Mode::Periodic => {
            let value =
                if (*mode == Mode::Pulse && t < 16) || (*mode == Mode::Periodic && t % 32 < 12) {
                    if *mode == Mode::Pulse { 1024 } else { 512 }
                } else {
                    0
                };
            c[..24].fill(value);
        }
        Mode::BabbleIndividual => {
            let i = r.below(26) as usize;
            c[i] = if r.below(2) == 0 { -1 } else { 1 } * [256, 512, 1024][r.below(3) as usize];
        }
        Mode::BabbleReverse => {
            let mut r = Rng(seed ^ ((t / 16).wrapping_mul(0x9e3779b97f4a7c15)));
            let i = r.below(26) as usize;
            c[i] = if t % 16 < 8 { 1024 } else { -1024 };
        }
        Mode::BabbleSimultaneous => {
            for item in &mut c[..26] {
                *item = match r.below(5) {
                    0 => -1024,
                    1 => -512,
                    2 => 0,
                    3 => 512,
                    _ => 1024,
                };
            }
        }
        Mode::BabbleSequential => {
            let mut r = Rng(seed);
            let mut axes: Vec<_> = (0..26).collect();
            for i in (1..26).rev() {
                let j = r.below((i + 1) as u64) as usize;
                axes.swap(i, j);
            }
            c[axes[(t / 4 % 26) as usize]] = if (t / 104).is_multiple_of(2) {
                768
            } else {
                -768
            };
        }
        Mode::BabblePhase => {
            let mut r = Rng(seed);
            for item in &mut c[..26] {
                let period = 16 + r.below(33);
                let phase = r.below(period);
                let on = 3 + r.below(10);
                let amplitude = [256, 512, 1024][r.below(3) as usize];
                let sign = if r.below(2) == 0 { -1 } else { 1 };
                *item = if (t + phase) % period < on {
                    sign * amplitude
                } else {
                    0
                };
            }
        }
        _ => {}
    }
    Some(c)
}
pub struct Trace {
    writer: Option<BufWriter<ChildStdin>>,
    child: Child,
    pub path: PathBuf,
}
impl Trace {
    pub fn new(path: &Path) -> Result<Self> {
        let file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)?;
        let mut child = Command::new("gzip")
            .args(["-n", "-3"])
            .stdin(Stdio::piped())
            .stdout(Stdio::from(file))
            .spawn()?;
        let writer = BufWriter::new(child.stdin.take().unwrap());
        Ok(Self {
            writer: Some(writer),
            child,
            path: path.into(),
        })
    }
    pub fn row(&mut self, row: &Value) -> Result<()> {
        serde_json::to_writer(self.writer.as_mut().unwrap(), row)?;
        self.writer.as_mut().unwrap().write_all(b"\n")?;
        Ok(())
    }
    pub fn finish(mut self) -> Result<()> {
        self.writer.take().unwrap().flush()?;
        drop(self.writer.take());
        if !self.child.wait()?.success() {
            return Err("gzip failed".into());
        }
        Ok(())
    }
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Measurement {
    pub name: String,
    pub genome: String,
    pub episode: Episode,
    pub mode: Mode,
    pub net: f64,
    pub path: f64,
    pub quarters: [f64; 4],
    pub last_two_min: f64,
    pub active_gain: f64,
    pub paid_motor: u64,
    pub pending_motor: f64,
    pub energy_per_net: Option<f64>,
    pub rotation: f64,
    pub score: f64,
    pub sustained: bool,
    pub active_axes: usize,
    pub joint_changes: Vec<u64>,
    pub final_velocity: [i32; 3],
    pub final_momentum: [i64; 3],
    pub final_hash: String,
    pub initial_hash: String,
    pub visits: usize,
    pub jets: u64,
    pub changed_connections: u64,
    #[serde(default)]
    pub directional_phases: [f64; 2],
}
#[allow(clippy::too_many_arguments)]
pub fn run(
    name: &str,
    g: &ControllerGenome,
    e: &Episode,
    mode: Mode,
    retained: Option<&NeuralState>,
    carry_fast: bool,
    trace: &mut Trace,
) -> Result<(Measurement, World)> {
    let mut w = world(g, e, retained, carry_fast)?;
    if mode == Mode::Random {
        let mut c = w.config().clone();
        c.neural.as_mut().unwrap().policy = Policy::Random;
        w = World::restore(&c, &w.snapshot())?;
    }
    let initial = w.snapshot();
    let initial_hash = initial.state_hash();
    let mut total = [0f64; 3];
    let mut path = 0.;
    let mut quarter = [[0f64; 3]; 4];
    let mut paid = 0;
    let mut rotation = 0.;
    let mut previous_orientation = e.orientation;
    let mut previous_angles = w.brains()[&1].neural.as_ref().unwrap().body.angles.clone();
    let mut changes = vec![0u64; 26];
    let mut visited = BTreeSet::new();
    let mut jets = 0;
    let mut coast = None;
    let mut coast_total = [0f64; 3];
    let mut actual_tail = [0f64; 3];
    let mut late_drive = 0;
    let mut directional_phases = [0f64; 2];
    trace.row(&json!({"type":"start","name":name,"genome":g.id(),"episode":e,"mode":mode,"config":w.config(),"config_hash":config_hash(w.config())?,"initial_hash":initial_hash,"initial_neural_state":w.brains()[&1].neural.as_ref().unwrap(),"initial_field":w.nutrient()}))?;
    for t in 0..e.ticks {
        if let Some(cue) = &e.cue
            && t == cue.switch_tick
        {
            w = change_cue(&w, cue.axis, -cue.first_sign)?;
            trace.row(&json!({"type":"cue","name":name,"tick":t,"field":w.nutrient(),"state_hash":w.snapshot().state_hash()}))?;
        }
        let before = position(&w);
        let submitted = commands(&mode, t, e.seed);
        let (receipt, rows) = if let Some(ref cmd) = submitted {
            w.tick_controlled(&BTreeMap::from([(1, cmd.clone())]))?
        } else {
            w.tick_observed()?
        };
        if rows.len() != 1 || rows[0].died {
            return Err("provisioned motor assay lost organism".into());
        }
        let row = &rows[0];
        let d = delta(before, row.position_after);
        path += norm(d);
        if let Some(cue) = &e.cue {
            let phase = usize::from(t >= cue.switch_tick);
            directional_phases[phase] += d[cue.axis]
                * f64::from(if phase == 0 {
                    cue.first_sign
                } else {
                    -cue.first_sign
                });
        }
        for a in 0..3 {
            total[a] += d[a];
            quarter[((t * 4 / e.ticks).min(3)) as usize][a] += d[a];
            if t >= e.ticks / 2 {
                actual_tail[a] += d[a];
            }
        }
        let n = w.brains()[&1].neural.as_ref().unwrap();
        for (i, angle) in n.body.angles.iter().enumerate() {
            changes[i] += u64::from(*angle != previous_angles[i]);
            previous_angles[i] = *angle;
        }
        for (a, previous) in previous_orientation.iter_mut().enumerate() {
            rotation +=
                f64::from((n.body.orientation[a] - *previous + 3217).rem_euclid(6434) - 3217).abs()
                    / 1024.;
            *previous = n.body.orientation[a];
        }
        if t >= e.ticks / 2 {
            late_drive += n.actuator_drive()[..26]
                .iter()
                .map(|v| u64::from(v.unsigned_abs()))
                .sum::<u64>();
        }
        paid += receipt.motor_energy;
        jets += receipt.jet_released;
        let cell = w.cells().next().unwrap();
        visited.insert((cell.site.x, cell.site.y, cell.site.z));
        let mut neural = serde_json::to_value(n)?;
        neural.as_object_mut().unwrap().remove("genome");
        trace.row(&json!({"type":"tick","name":name,"tick":t,"submitted_commands":submitted,"organism":row,"neural":neural,"receipt":receipt,"environment_momentum":w.environment_momentum(),"state_hash":w.snapshot().state_hash()}))?;
        if t + 1 == e.ticks / 2 {
            coast = Some(w.clone());
        }
        if t >= e.ticks / 2 {
            let other = coast.as_mut().unwrap();
            let before = position(other);
            let (r, _) = other.tick_controlled(&BTreeMap::from([(1, vec![0; 30])]))?;
            if r.deaths > 0 {
                return Err("coast assay died".into());
            }
            let step = delta(before, position(other));
            for a in 0..3 {
                coast_total[a] += step[a];
            }
        }
    }
    let n = w.brains()[&1].neural.as_ref().unwrap();
    let net = norm(total);
    let direction = if net > 0. {
        total.map(|v| v / net)
    } else {
        [0.; 3]
    };
    let project = |v: [f64; 3]| v.iter().zip(direction).map(|(a, b)| a * b).sum::<f64>();
    let q = quarter.map(project);
    let last = q[2].min(q[3]).max(0.);
    let gain = project(actual_tail) - project(coast_total);
    let pending = n.actuator_debt as f64 / 4096.;
    let effective = paid as f64 + pending;
    let efficiency = if net >= 0.05 {
        net / (1. + effective)
    } else {
        0.
    };
    let mut score = net + 4. * last + 2. * gain.max(0.) + 0.25 * efficiency
        - 0.001 * effective
        - 0.002 * rotation;
    if e.cue.is_some() {
        score = if jets > 0 {
            -1000.
        } else {
            2. * (directional_phases[0] + directional_phases[1])
                + 2. * directional_phases[0].min(directional_phases[1])
                - 0.001 * effective
                - 0.002 * rotation
        };
    }
    let measurement = Measurement {
        name: name.into(),
        genome: g.id(),
        episode: e.clone(),
        mode,
        net,
        path,
        quarters: q,
        last_two_min: last,
        active_gain: gain,
        paid_motor: paid,
        pending_motor: pending,
        energy_per_net: if net >= 0.05 {
            Some(effective / net)
        } else {
            None
        },
        rotation,
        score,
        sustained: net >= 0.5 && last >= 0.10 && gain >= 0.05 && late_drive > 0 && jets == 0,
        active_axes: changes.iter().filter(|v| **v > 0).count(),
        joint_changes: changes,
        final_velocity: n.body.velocity,
        final_momentum: n.body.momentum,
        final_hash: w.snapshot().state_hash(),
        initial_hash,
        visits: visited.len(),
        jets,
        changed_connections: n.changed_connections,
        directional_phases,
    };
    trace.row(&json!({"type":"end","name":name,"measurement":measurement,"final_neural_state":w.brains()[&1].neural.as_ref().unwrap(),"coast_final_hash":coast.as_ref().unwrap().snapshot().state_hash(),"coast_tail":coast_total}))?;
    Ok((measurement, w))
}
pub fn save(path: &Path, value: &impl Serialize) -> Result<()> {
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}
/// Authored cue environment; uptake is disabled and material sum conserved.
pub fn cue_field(total: u64, axis: usize, sign: i32) -> Vec<u64> {
    let weights: Vec<u64> = (0..4096)
        .map(|i| {
            let coords = [i % 16, (i / 16) % 16, i / 256];
            if sign == 0 {
                11
            } else {
                4 + if sign > 0 {
                    coords[axis]
                } else {
                    15 - coords[axis]
                }
            }
        })
        .collect();
    let sum = weights.iter().sum::<u64>();
    let mut field: Vec<u64> = weights.iter().map(|v| total * v / sum).collect();
    let left = total - field.iter().sum::<u64>();
    for v in field.iter_mut().take(left as usize) {
        *v += 1;
    }
    field
}
pub fn change_cue(w: &World, axis: usize, sign: i32) -> Result<World> {
    let mut s = w.snapshot();
    let total = s.nutrient.iter().sum();
    s.nutrient = cue_field(total, axis, sign);
    Ok(World::restore(w.config(), &s)?)
}
