//! Experimental controls and fixtures only. Never imported by production code.
use moebius_core::{
    SimConfig, Site, SnapshotFile, World,
    neural::{
        Q,
        body::{Body, rotate},
        schema::Axis,
    },
    world::OrganismTick,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    io::{BufWriter, Write},
    path::Path,
};
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub const SIDE: u32 = 32;
pub const ID: u64 = 1;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Case {
    pub name: String,
    pub seed: u64,
    pub drag: u32,
    pub energy: u64,
    pub target_delta: Option<[i32; 3]>,
    pub founding_bite: bool,
    pub horizon: u64,
}
pub fn point(site: Site) -> [i64; 3] {
    [site.x, site.y, site.z].map(|v| i64::from(v) * 1024)
}
pub fn delta(a: [i64; 3], b: [i64; 3]) -> [i64; 3] {
    std::array::from_fn(|i| {
        (b[i] - a[i] + i64::from(SIDE) * 512).rem_euclid(i64::from(SIDE) * 1024)
            - i64::from(SIDE) * 512
    })
}
pub fn norm(v: [i64; 3]) -> f64 {
    v.iter().map(|v| (*v as f64).powi(2)).sum::<f64>().sqrt() / 1024.
}
pub fn com(w: &World) -> [i64; 3] {
    let c = w.cells().next().unwrap();
    let b = &w.brains()[&ID];
    let p = point(c.site);
    std::array::from_fn(|a| p[a] + i64::from(b.offset[a]))
}
pub fn mouth(w: &World) -> [i64; 3] {
    let state = w.brains()[&ID].neural.as_ref().unwrap();
    mouth_pose(com(w), &state.body, &state.genome.anatomy)
}
pub fn mouth_pose(
    p: [i64; 3],
    body: &Body,
    anatomy: &moebius_core::neural::schema::Anatomy,
) -> [i64; 3] {
    let local = body.local_geometry(anatomy).0[anatomy.spine - 3].map(i64::from);
    let local = rotate(local, body.orientation);
    std::array::from_fn(|a| p[a] + local[a])
}
pub fn fixture(case: &Case) -> Result<(World, Option<Site>)> {
    let mut c = SimConfig::nursery_farm_256();
    c.width = SIDE;
    c.height = SIDE;
    c.depth = SIDE;
    c.initial_population = 1;
    c.initial_energy = case.energy;
    c.initial_reservoir = 0;
    c.inflow_per_tick = 0;
    c.inject_per_tick = 0;
    c.feeding.as_mut().unwrap().initial_stock = 0;
    c.farm.as_mut().unwrap().max_population = 1;
    c.neural.as_mut().unwrap().drag = case.drag;
    let mut s = World::genesis(&c, case.seed)?.snapshot();
    s.cells[0].site = Site {
        x: 16,
        y: 16,
        z: 16,
    };
    let state = s.brains[0].neural.as_ref().unwrap();
    let head = state.body.head_site(
        &state.genome.anatomy,
        s.cells[0].site,
        s.brains[0].offset,
        moebius_core::Dims {
            width: SIDE,
            height: SIDE,
            depth: SIDE,
        },
    );
    let target = case.target_delta.map(|d| Site {
        x: (i64::from(head.x) + i64::from(d[0])).rem_euclid(i64::from(SIDE)) as u32,
        y: (i64::from(head.y) + i64::from(d[1])).rem_euclid(i64::from(SIDE)) as u32,
        z: (i64::from(head.z) + i64::from(d[2])).rem_euclid(i64::from(SIDE)) as u32,
    });
    if let Some(t) = target {
        s.nutrient[(t.x + SIDE * (t.y + SIDE * t.z)) as usize] = 512;
        s.material_initial = 512;
    }
    if case.founding_bite {
        s.nutrient[(head.x + SIDE * (head.y + SIDE * head.z)) as usize] = 1;
        s.material_initial += 1;
    }
    Ok((World::restore(&c, &s)?, target))
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Schedule {
    pub pattern: String,
    pub amplitude: i32,
    pub pulse: u64,
    pub period: u64,
    pub recovery: i32,
    pub ablate: Option<usize>,
}
impl Schedule {
    pub fn commands(&self, w: &World, tick: u64) -> Vec<i32> {
        let a = &w.brains()[&ID].neural.as_ref().unwrap().genome.anatomy;
        let mut cmd = vec![0; a.motors()];
        let phase = if self.period == 0 {
            tick
        } else {
            tick % self.period
        };
        let amp = if phase < self.pulse {
            self.amplitude
        } else {
            self.recovery
        };
        for (i, b) in a.axes.iter().enumerate() {
            let sign = match self.pattern.as_str() {
                "bend" if b.axis == Axis::Bend => 1,
                "opposed" if b.axis == Axis::Bend => {
                    if b.joint % 2 == 0 {
                        1
                    } else {
                        -1
                    }
                }
                "front" if b.axis == Axis::Bend && b.joint >= 6 => 1,
                "back" if b.axis == Axis::Bend && b.joint < 6 => 1,
                "twist" if b.axis == Axis::Twist => 1,
                "bend_twist" if b.axis != Axis::Jaw => 1,
                "jaw" if b.axis == Axis::Jaw => {
                    if b.joint % 2 == 0 {
                        1
                    } else {
                        -1
                    }
                }
                "rest" => 0,
                pattern if pattern.starts_with("axis_") => {
                    i32::from(pattern[5..].parse::<usize>().unwrap() == i)
                }
                _ => 0,
            };
            cmd[i] = (amp * sign).clamp(-Q, Q);
        }
        if let Some(i) = self.ablate {
            cmd[i] = 0;
        }
        cmd
    }
}
pub fn step(
    w: &mut World,
    commands: Option<&[i32]>,
) -> Result<(moebius_core::TickReceipt, OrganismTick)> {
    let (receipt, rows) = if let Some(commands) = commands {
        w.tick_controlled(&BTreeMap::from([(ID, commands.to_vec())]))?
    } else {
        w.tick_observed()?
    };
    Ok((
        receipt,
        rows.into_iter().next().ok_or("missing organism row")?,
    ))
}
pub fn trial<F>(root: &Path, case: &Case, description: Value, mut controller: F) -> Result<Value>
where
    F: FnMut(&World, u64, Option<Site>) -> Result<Option<Vec<i32>>>,
{
    let (mut w, target) = fixture(case)?;
    if description["kind"] == "local_lobes" {
        let mut config = w.config().clone();
        config.neural.as_mut().unwrap().chemical_sensing =
            moebius_core::neural::ChemicalSensing::LocalLobesV1;
        w = World::restore(&config, &w.snapshot())?;
    }
    if description["kind"] == "random" {
        let mut config = w.config().clone();
        config.neural.as_mut().unwrap().policy = moebius_core::neural::Policy::Random;
        w = World::restore(&config, &w.snapshot())?;
    }
    fs::write(
        root.join(format!("{}.config.json", case.name)),
        serde_json::to_vec_pretty(w.config())?,
    )?;
    fs::write(
        root.join(format!("{}.initial.bin", case.name)),
        SnapshotFile::encode(w.config(), &w)?,
    )?;
    let initial_com = com(&w);
    let initial_mouth = mouth(&w);
    let mut last_mouth = initial_mouth;
    let mut unwrapped = [0i64; 3];
    let mut path = 0.;
    let mut mouth_path = 0.;
    let mut minimum_target = f64::INFINITY;
    let mut first_food = None;
    let mut first_target = None;
    let mut first_signal = None;
    let mut cost = [0u64; 5];
    let mut intake = 0;
    let mut died = None;
    let mut rotation = 0i64;
    let mut prev_orientation = [0; 3];
    let mut peak_speed = 0.;
    let mut no_motion = 0;
    let mut first_zero = None;
    let mut zero_run = 0;
    let mut stopped_at = None;
    let mut trace = BufWriter::new(fs::File::create(root.join(format!("{}.jsonl", case.name)))?);
    let mut final_mouth = initial_mouth;
    let mut final_velocity = [0; 3];
    let mut final_momentum = [0; 3];
    let mut final_energy = case.energy;
    let mut final_orientation = [0; 3];
    for tick in 0..case.horizon {
        if w.live_count() == 0 {
            break;
        }
        let sensors = w.organism_sensors(ID)?;
        if sensors[93..99].iter().any(|v| *v > 0) && first_signal.is_none() {
            first_signal = Some(tick);
        }
        let before = com(&w);
        let mouth_before = mouth(&w);
        let mut body = w.brains()[&ID].neural.as_ref().unwrap().body.clone();
        let anatomy = w.brains()[&ID]
            .neural
            .as_ref()
            .unwrap()
            .genome
            .anatomy
            .clone();
        let commands = if case.founding_bite && tick == 0 {
            Some(vec![0; anatomy.motors()])
        } else {
            controller(&w, tick, target)?
        };
        let (receipt, row) = step(&mut w, commands.as_deref())?;
        let motion = delta(before, row.position_after);
        for a in 0..3 {
            unwrapped[a] += motion[a];
        }
        let distance = norm(motion);
        path += distance;
        if distance == 0. {
            no_motion += 1;
        }
        body.angles = row.angles.clone();
        body.orientation = row.orientation;
        final_mouth = mouth_pose(row.position_after, &body, &anatomy);
        mouth_path += norm(delta(last_mouth, final_mouth));
        last_mouth = final_mouth;
        if let Some(t) = target {
            minimum_target = minimum_target.min(norm(delta(final_mouth, point(t))));
        }
        if row.intake > 0 && first_food.is_none() {
            first_food = Some(tick);
        }
        if row.food_site == target && row.intake > 0 && first_target.is_none() {
            first_target = Some(tick);
        }
        for (i, v) in [
            row.maintenance,
            row.neural,
            row.actuator,
            row.secretion,
            row.digging,
        ]
        .into_iter()
        .enumerate()
        {
            cost[i] += v;
        }
        intake += row.intake;
        final_energy = row.energy_after;
        for (a, prev) in prev_orientation.iter_mut().enumerate() {
            rotation +=
                i64::from((row.orientation[a] - *prev + 3217).rem_euclid(6434) - 3217).abs();
            *prev = row.orientation[a];
        }
        if let Some(n) = w.brains().get(&ID).and_then(|b| b.neural.as_ref()) {
            final_velocity = n.body.velocity;
            final_momentum = n.body.momentum;
            final_orientation = n.body.orientation;
            peak_speed = f64::max(peak_speed, norm(n.body.velocity.map(i64::from)));
            if n.body.velocity == [0; 3] {
                zero_run += 1;
                if first_zero.is_none() {
                    first_zero = Some(tick);
                }
            } else {
                zero_run = 0;
            }
            if zero_run >= 8 {
                stopped_at = Some(tick - 7);
            }
        }
        if row.died {
            died = Some(tick);
        }
        let accounts = w.accounts()?;
        writeln!(
            trace,
            "{}",
            json!({"tick":tick,"submitted_commands":commands,"sensors_before":sensors,
            "mouth_before":mouth_before,
            "mouth_after":final_mouth,"com_delta":motion,"mouth_to_target":target.map(|t|norm(delta(final_mouth,point(t)))),
            "velocity_after_if_alive":if row.died{None}else{Some(final_velocity)},"momentum_after_if_alive":if row.died{None}else{Some(final_momentum)},
            "organism":row,"receipt":receipt,"state_hash":w.snapshot().state_hash(),
            "material_residual":accounts.material_residual().to_string(),"energy_residual":accounts.energy_residual().to_string(),"environment_momentum":w.environment_momentum()})
        )?;
    }
    trace.flush()?;
    fs::write(
        root.join(format!("{}.final.bin", case.name)),
        SnapshotFile::encode(w.config(), &w)?,
    )?;
    Ok(
        json!({"status":"PASS","case":case,"controller":description,"target":target,"ticks":w.tick_index(),
        "initial_com":initial_com,"final_com_unwrapped":std::array::from_fn::<_,3,_>(|a|initial_com[a]+unwrapped[a]),
        "com_path":path,"com_net":norm(unwrapped),"mouth_path":mouth_path,"mouth_net":norm(delta(initial_mouth,final_mouth)),
        "initial_mouth":initial_mouth,"final_mouth":final_mouth,"minimum_continuous_target_distance":if minimum_target.is_finite(){Some(minimum_target)}else{None},
        "first_food_tick":first_food,"first_target_tick":first_target,"first_chemical_signal_tick":first_signal,
        "costs_maintenance_neural_actuator_secretion_digging":cost,"actuator_energy_per_com_path":if path>0.{Some(cost[2] as f64/path)}else{None},
        "intake":intake,"death_tick":died,"final_energy":final_energy,"final_velocity":final_velocity,"final_momentum":final_momentum,
        "peak_speed":peak_speed,"rotation":rotation,"final_orientation":final_orientation,"zero_displacement_ticks":no_motion,
        "first_zero_speed_tick":first_zero,"last_eight_tick_zero_speed_start":stopped_at,"final_hash":w.snapshot().state_hash()}),
    )
}
pub fn save_results(root: &Path, results: &[Value]) -> Result<()> {
    fs::write(
        root.join("results.json"),
        serde_json::to_vec_pretty(
            &json!({"results":results,"failures":results.iter().filter(|v|v["status"]!="PASS").count()}),
        )?,
    )?;
    Ok(())
}
