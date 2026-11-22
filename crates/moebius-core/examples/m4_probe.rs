//! Registered M4 diagnostics. Run: ... -- OUT [law version: 1 or 2]
use moebius_core::{SimConfig, Site, SnapshotFile, World, neural::Policy};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    io::{BufWriter, Write},
    path::Path,
};

fn position_delta(a: [i64; 3], b: [i64; 3], side: u32) -> [f64; 3] {
    let span = i64::from(side) * 1024;
    std::array::from_fn(|i| ((b[i] - a[i] + span / 2).rem_euclid(span) - span / 2) as f64 / 1024.)
}
fn norm(v: [f64; 3]) -> f64 {
    v.iter().map(|x| x * x).sum::<f64>().sqrt()
}
fn coordinates(site: Site) -> [i64; 3] {
    [site.x, site.y, site.z].map(|v| i64::from(v) * 1024)
}
fn nearest(head: Site, food: &[Site], side: u32) -> Option<f64> {
    food.iter()
        .map(|s| norm(position_delta(coordinates(head), coordinates(*s), side)))
        .min_by(f64::total_cmp)
}
#[derive(Default)]
struct Track {
    start: [i64; 3],
    unwrapped: [f64; 3],
    path: f64,
    intake: u64,
    gain: u64,
    costs: [u64; 5],
    attempts: u64,
    meals: u64,
    near_unfed: u64,
    first_food: Option<Site>,
    other_food_meals: u64,
    death: Option<u64>,
    rotation: f64,
    last_orientation: [i32; 3],
    previous_angles: Vec<i32>,
    actual_active: Vec<u64>,
    reported_active: Vec<u64>,
    excursion_min: Vec<i32>,
    excursion_max: Vec<i32>,
    effort: Vec<u64>,
    pinned: Vec<u64>,
    minimum_food_distance: Option<f64>,
    initial_energy: u64,
}
#[allow(clippy::too_many_arguments)] // Explicit registered trial coordinates.
fn trial(
    root: &Path,
    environment: &str,
    controller: &str,
    seed: u64,
    version: u32,
    base: &SimConfig,
    horizon: u64,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut c = base.clone();
    let side = c.width;
    if c.height != side || c.depth != side {
        return Err("probe requires cubic grid".into());
    }
    c.neural.as_mut().unwrap().version = version;
    if environment != "ecology" {
        c.initial_population = 1;
        c.farm.as_mut().unwrap().max_population = 1;
    }
    if environment == "free" {
        c.initial_energy = 2048;
    }
    if environment == "moderate" {
        c.initial_energy = 512;
    }
    let pure_wave = controller == "locked_wave_only" || controller == "fixed_wave_only";
    let rules = c.neural.as_mut().unwrap();
    rules.spinal_wave = controller == "current" || controller == "wave_only" || pure_wave;
    if pure_wave {
        rules.plasticity = false;
        if controller == "fixed_wave_only" {
            rules.mutation = false;
        }
    }
    if controller == "random" {
        rules.policy = Policy::Random;
    }
    if environment != "ecology" && environment != "sparse" {
        c.initial_reservoir = 0;
        c.inflow_per_tick = 0;
        c.inject_per_tick = 0;
        c.feeding.as_mut().unwrap().initial_stock = 0;
    }
    let mut s = World::genesis(&c, seed)?.snapshot();
    if environment != "ecology" && environment != "sparse" {
        s.cells[0].site = Site {
            x: 16,
            y: 16,
            z: 16,
        };
    }
    for brain in &mut s.brains {
        if controller == "wave_only" || controller == "rest" || pure_wave {
            let g = &mut brain.neural.as_mut().unwrap().genome;
            g.output.fill(0);
            g.motor_biases.fill(0);
        }
    }
    if environment == "easy" || environment == "moderate" {
        let state = s.brains[0].neural.as_ref().unwrap();
        let head = state.body.head_site(
            &state.genome.anatomy,
            s.cells[0].site,
            s.brains[0].offset,
            moebius_core::Dims {
                width: side,
                height: side,
                depth: side,
            },
        );
        let mut pos = [head.x, head.y, head.z];
        let axis = ((seed - 1) % 3) as usize;
        let sign = if seed.is_multiple_of(2) { -1i64 } else { 1 };
        pos[axis] = (i64::from(pos[axis]) + sign * if environment == "easy" { 3 } else { 6 })
            .rem_euclid(i64::from(side)) as u32;
        s.nutrient[(pos[0] + side * (pos[1] + side * pos[2])) as usize] = 512;
        s.material_initial = 512;
    }
    let prefix = format!("{environment}_{controller}_{seed}");
    fs::write(
        root.join(format!("{prefix}.config.json")),
        serde_json::to_vec_pretty(&c)?,
    )?;
    let mut w = World::restore(&c, &s)?;
    fs::write(
        root.join(format!("{prefix}.initial.bin")),
        SnapshotFile::encode(&c, &w)?,
    )?;
    let initial_hash = w.snapshot().state_hash();
    let mut trace = BufWriter::new(fs::File::create(root.join(format!("{prefix}.jsonl")))?);
    let mut tracks: BTreeMap<u64, Track> = BTreeMap::new();
    let mut births = 0;
    let mut max_live = w.live_count();
    for _ in 0..horizon {
        let foods: Vec<Site> = w
            .nutrient()
            .iter()
            .enumerate()
            .filter(|(_, n)| **n > 0)
            .map(|(i, _)| Site {
                x: (i % side as usize) as u32,
                y: ((i / side as usize) % side as usize) as u32,
                z: (i / (side * side) as usize) as u32,
            })
            .collect();
        let (receipt, rows) = w.tick_observed()?;
        births += receipt.births;
        max_live = max_live.max(w.live_count());
        for row in rows {
            let t = tracks.entry(row.id).or_insert_with(|| Track {
                start: row.position_before,
                initial_energy: row.energy_before,
                previous_angles: vec![0; row.angles.len()],
                actual_active: vec![0; row.angles.len()],
                reported_active: vec![0; row.angles.len()],
                excursion_min: vec![0; row.angles.len()],
                excursion_max: vec![0; row.angles.len()],
                effort: vec![0; row.angles.len()],
                pinned: vec![0; row.angles.len()],
                ..Track::default()
            });
            let delta = position_delta(row.position_before, row.position_after, side);
            let distance = norm(delta);
            t.path += distance;
            for (a, d) in delta.into_iter().enumerate() {
                t.unwrapped[a] += d;
            }
            t.intake += row.intake;
            t.gain += row.gain;
            t.attempts += 1;
            t.costs[0] += row.maintenance;
            t.costs[1] += row.neural;
            t.costs[2] += row.actuator;
            t.costs[3] += row.secretion;
            t.costs[4] += row.digging;
            if row.intake > 0 {
                t.meals += 1;
                if t.first_food.is_none() {
                    t.first_food = row.food_site;
                }
                if t.first_food != row.food_site {
                    t.other_food_meals += 1;
                }
            }
            let food_distance = row.head.and_then(|h| nearest(h, &foods, side));
            if let Some(d) = food_distance {
                t.minimum_food_distance =
                    Some(t.minimum_food_distance.map(|v| v.min(d)).unwrap_or(d));
                if d <= 2. && row.intake == 0 {
                    t.near_unfed += 1;
                }
            }
            let mut actual_rates = Vec::new();
            for (i, angle) in row.angles.iter().copied().enumerate() {
                let actual = angle - t.previous_angles[i];
                actual_rates.push(actual);
                t.actual_active[i] += u64::from(actual != 0);
                t.reported_active[i] += u64::from(row.rates[i] != 0);
                t.effort[i] += u64::from(row.efforts[i].unsigned_abs());
                t.excursion_min[i] = t.excursion_min[i].min(angle);
                t.excursion_max[i] = t.excursion_max[i].max(angle);
                t.pinned[i] += u64::from(actual == 0 && row.rates[i] != 0);
                t.previous_angles[i] = angle;
            }
            for a in 0..3 {
                t.rotation += f64::from(
                    (row.orientation[a] - t.last_orientation[a] + 3217).rem_euclid(6434) - 3217,
                )
                .abs()
                    / 1024.;
            }
            t.last_orientation = row.orientation;
            if row.died {
                t.death = Some(row.tick);
            }
            let balance = i128::from(row.energy_before) + i128::from(row.gain)
                - i128::from(row.energy_after)
                - i128::from(
                    row.maintenance
                        + row.neural
                        + row.actuator
                        + row.secretion
                        + row.digging
                        + row.birth_out
                        + row.death_disposal,
                );
            if balance != 0 {
                return Err(format!("individual balance {balance}").into());
            }
            writeln!(
                trace,
                "{}",
                json!({"organism":row,"actual_rates":actual_rates,
                "travel":distance,"nearest_food_head_distance":food_distance,
                "energy_balance_residual":balance.to_string()})
            )?;
        }
        if w.live_count() == 0 {
            break;
        }
    }
    trace.flush()?;
    let accounts = w.accounts()?;
    let organisms:Vec<_>=tracks.iter().map(|(id,t)| json!({"id":id,"initial_energy":t.initial_energy,
        "start":t.start,"path":t.path,"net_displacement":norm(t.unwrapped),"intake":t.intake,
        "gain":t.gain,"costs_maintenance_neural_motor_secretion_digging":t.costs,
        "feeding_attempts":t.attempts,"meals":t.meals,"near_food_without_eating":t.near_unfed,
        "meals_at_other_food_sites":t.other_food_meals,"death_tick":t.death,
        "death_cause":t.death.map(|_|"insufficient energy for maintenance"),
        "rotation_radians_approx":t.rotation,"minimum_food_distance":t.minimum_food_distance,
        "actual_active":t.actual_active,"reported_active":t.reported_active,"false_motion_ticks":t.pinned,
        "amplitude":t.excursion_max.iter().zip(&t.excursion_min).map(|(a,b)|a-b).collect::<Vec<_>>(),
        "effort_abs_sum":t.effort})).collect();
    fs::write(
        root.join(format!("{prefix}.final.bin")),
        SnapshotFile::encode(&c, &w)?,
    )?;
    Ok(
        json!({"status":"PASS","environment":environment,"controller":controller,"seed":seed,
        "law_version":version,"ticks":w.tick_index(),"births":births,"final_live":w.live_count(),
        "max_live":max_live,"initial_hash":initial_hash,"final_hash":w.snapshot().state_hash(),
        "material_residual":accounts.material_residual().to_string(),"energy_residual":accounts.energy_residual().to_string(),
        "organisms":organisms}),
    )
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let root = std::path::PathBuf::from(args.next().ok_or("output directory required")?);
    let version: u32 = args.next().unwrap_or("1".into()).parse()?;
    let base: SimConfig = if let Some(path) = args.next() {
        serde_json::from_slice(&fs::read(path)?)?
    } else {
        serde_json::from_slice(include_bytes!("../../../experiments/nursery_smoke.json"))?
    };
    let environment_filter = args.next().unwrap_or("all".into());
    let controller_filter = args.next().unwrap_or("all".into());
    let seeds: u64 = args.next().unwrap_or("8".into()).parse()?;
    let horizon: u64 = args.next().unwrap_or("256".into()).parse()?;
    if root.exists() {
        return Err("refusing to overwrite evidence".into());
    }
    fs::create_dir_all(&root)?;
    fs::write(
        root.join("protocol.json"),
        include_bytes!("../../../docs/m4/PROTOCOL.json"),
    )?;
    let mut results = Vec::new();
    for environment in ["free", "easy", "moderate", "sparse", "ecology"] {
        if environment_filter != "all" && environment_filter != environment {
            continue;
        }
        for controller in [
            "current",
            "no_wave",
            "random",
            "wave_only",
            "rest",
            "locked_wave_only",
            "fixed_wave_only",
        ] {
            if controller_filter == "all"
                && (controller == "locked_wave_only" || controller == "fixed_wave_only")
            {
                continue;
            }
            if controller_filter != "all" && controller_filter != controller {
                continue;
            }
            for seed in 1..=seeds {
                let r=trial(&root,environment,controller,seed,version,&base,horizon).unwrap_or_else(|e|
                    json!({"status":"FAIL","environment":environment,"controller":controller,"seed":seed,"error":e.to_string()}));
                println!("{environment} {controller} {seed}: {}", r["status"]);
                results.push(r);
                fs::write(
                    root.join("results.partial.json"),
                    serde_json::to_vec_pretty(&results)?,
                )?;
            }
        }
    }
    let failures = results.iter().filter(|r| r["status"] != "PASS").count();
    fs::write(
        root.join("results.json"),
        serde_json::to_vec_pretty(&json!({"failures":failures,"results":results}))?,
    )?;
    if failures > 0 {
        return Err("trial failures preserved".into());
    }
    Ok(())
}
