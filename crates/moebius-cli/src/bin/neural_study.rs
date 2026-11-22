//! Reproducible matched experiments. This harness authors initial conditions,
//! never routes policy actions. Controlled command probes are labeled separately.
use clap::Parser;
use moebius_core::{
    SimConfig, Site, SnapshotFile, World,
    neural::{ControllerGenome, NeuralRules, NeuralState, Policy},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    time::Instant,
};
#[derive(Parser)]
struct Args {
    #[arg(long)]
    out: PathBuf,
    #[arg(long, default_value_t = 8)]
    replicates: u64,
    #[arg(long, default_value_t = 256)]
    ticks: u64,
}
fn garden(genome: ControllerGenome) -> Vec<i32> {
    let mut state = NeuralState::new(genome, 1000);
    let rules = NeuralRules {
        plasticity: false,
        ..NeuralRules::default()
    };
    let mut values = Vec::new();
    let mut inputs = vec![0; state.genome.anatomy.sensors()];
    let offset = state.genome.anatomy.axes.len() * 3;
    inputs[offset + 9] = 512;
    for tick in 0..32 {
        state.integrate(inputs.clone(), &rules, 0, 1, tick, 1000);
        values.extend_from_slice(&state.commands);
    }
    values
}
fn make_world(seed: u64, environment: &str, condition: &str) -> Result<World, String> {
    let mut c = SimConfig::neural_farm_256();
    c.width = 16;
    c.height = 16;
    c.depth = 16;
    c.initial_population = if environment == "lineage" { 4 } else { 1 };
    c.initial_energy = 384;
    c.energy_cap = 4096;
    c.inflow_per_tick = 0;
    c.inject_per_tick = 0;
    c.export_per_tick = 0;
    c.initial_reservoir = 0;
    c.maintenance_cost = 1;
    c.uptake_per_tick = 1;
    c.initial_nutrient_per_site = 0;
    c.nutrient_wave_amplitude = 0;
    let farm = c.farm.as_mut().unwrap();
    farm.max_population = if environment == "lineage" { 24 } else { 1 };
    farm.birth_energy = if environment == "lineage" { 80 } else { 4096 };
    farm.maturity_ticks = 16;
    farm.birth_cooldown = 32;
    let rules = c.neural.as_mut().unwrap();
    match condition {
        "plastic_off" => rules.plasticity = false,
        "random" => {
            rules.policy = Policy::Random;
            rules.plasticity = false;
        }
        "recurrence_off" => rules.recurrence = false,
        "proprioception_off" => rules.proprioception = false,
        "chemistry_off" => rules.chemistry = false,
        "mutation_off" => rules.mutation = false,
        _ => {}
    }
    let world = World::genesis(&c, seed).map_err(|e| e.to_string())?;
    let mut s = world.snapshot();
    for (i, cell) in s.cells.iter_mut().enumerate() {
        cell.site = Site {
            x: 7 + (i as u32 % 2) * 2,
            y: 7,
            z: 7 + (i as u32 / 2) * 2,
        };
    }
    for z in 0..16 {
        for y in 0..16 {
            for x in 0..16 {
                let index = x + 16 * (y + 16 * z);
                s.nutrient[index] = if (9..=11).contains(&x) { 12 } else { 2 };
            }
        }
    }
    for cell in &s.cells {
        let i = cell.site.x as usize + 16 * (cell.site.y as usize + 16 * cell.site.z as usize);
        s.nutrient[i] = 8;
    }
    if environment == "contact" {
        for x in 5..=9 {
            for z in 5..=9 {
                s.flesh.push(moebius_core::farm::FleshDeposit {
                    site: Site { x, y: 8, z },
                    amount: 4,
                });
            }
        }
        s.flesh.sort_by_key(|d| d.site);
    }
    s.material_initial =
        s.nutrient.iter().sum::<u64>() + s.flesh.iter().map(|d| d.amount).sum::<u64>();
    World::restore(&c, &s).map_err(|e| e.to_string())
}
fn trial(
    out: &Path,
    seed: u64,
    environment: &str,
    condition: &str,
    ticks: u64,
    trace: bool,
) -> Result<Value, String> {
    let mut world = make_world(seed, environment, condition)?;
    let initial_hash = world.snapshot().state_hash();
    let initial_genomes: BTreeMap<_, _> = world
        .brains()
        .iter()
        .map(|(id, b)| (*id, b.neural.as_ref().unwrap().genome.id()))
        .collect();
    let initial_energy = world.accounts().unwrap().energy_live;
    let mut tracefile = if trace {
        Some(BufWriter::new(
            File::create(out.join(format!("trace_{environment}_{seed}.jsonl")))
                .map_err(|e| e.to_string())?,
        ))
    } else {
        None
    };
    let mut lineagefile = if environment == "lineage" {
        Some(BufWriter::new(
            File::create(out.join(format!("lineage_evidence_{condition}_{seed}.jsonl")))
                .map_err(|e| e.to_string())?,
        ))
    } else {
        None
    };
    let mut lineage = BTreeMap::new();
    let mut gardens: BTreeMap<u64, Vec<i32>> = BTreeMap::new();
    let mut inherited_changes = 0;
    let mut behavioral_descendants = 0;
    let mut descendant_total = 0;
    let mut intake = 0;
    let mut early_intake = 0;
    let mut late_intake = 0;
    let mut movement = 0f64;
    let mut launches = 0;
    let mut excavated = 0;
    let mut births = 0;
    let mut fees = 0;
    let mut neural_fees = 0;
    let mut last = BTreeMap::<u64, ([u32; 3], [i32; 3])>::new();
    let mut participation = vec![0u64; 26];
    let mut largest_plastic = 0;
    let mut commands_nonzero = 0;
    let mut mechanical_ticks = 0;
    let mut net_displacement = BTreeMap::<u64, [i64; 3]>::new();
    let started = Instant::now();
    for step in 0..ticks {
        for (id, b) in world.brains() {
            let state = b.neural.as_ref().unwrap();
            if !lineage.contains_key(id) {
                let probe = garden(state.genome.clone());
                if b.parent != 0 {
                    descendant_total += 1;
                    if lineage.get(&b.parent) != Some(&state.genome.id()) {
                        inherited_changes += 1;
                    }
                    if gardens
                        .get(&b.parent)
                        .is_some_and(|parent| parent != &probe)
                    {
                        behavioral_descendants += 1;
                    }
                }
                if let Some(file) = &mut lineagefile {
                    let record = json!({"tick":world.tick_index(),"id":id,"parent":b.parent,
                        "generation":b.generation,"genotype_id":state.genome.id(),
                        "genome":state.genome,"fresh_state_command_probe":probe});
                    writeln!(file, "{record}").map_err(|e| e.to_string())?;
                }
                lineage.insert(*id, state.genome.id());
                gardens.insert(*id, probe);
            }
        }
        let receipt = world.tick().map_err(|e| format!("tick {step}: {e}"))?;
        intake += receipt.nutrient_uptaken;
        if step < ticks / 4 {
            early_intake += receipt.nutrient_uptaken;
        }
        if step >= ticks - ticks / 4 {
            late_intake += receipt.nutrient_uptaken;
        }
        launches += receipt.jet_released;
        excavated += receipt.excavated;
        births += receipt.births;
        fees += receipt.motor_energy;
        neural_fees += receipt.neural_energy;
        for cell in world.cells() {
            let rig = &world.brains()[&cell.id];
            let state = rig.neural.as_ref().unwrap();
            let site = [cell.site.x, cell.site.y, cell.site.z];
            if let Some((old, offset)) = last.insert(cell.id, (site, rig.offset)) {
                let mut length2 = 0f64;
                for axis in 0..3 {
                    let mut delta = i64::from(site[axis]) - i64::from(old[axis]);
                    if delta > 8 {
                        delta -= 16;
                    }
                    if delta < -8 {
                        delta += 16;
                    }
                    let sub = delta * 1024 + i64::from(rig.offset[axis] - offset[axis]);
                    length2 += (sub as f64 / 1024.0).powi(2);
                    net_displacement.entry(cell.id).or_default()[axis] += sub;
                }
                movement += length2.sqrt();
            }
            let plastic = state
                .plastic
                .iter()
                .map(|v| v.unsigned_abs() as u64)
                .sum::<u64>();
            largest_plastic = largest_plastic.max(plastic);
            commands_nonzero += state.commands[..26].iter().filter(|u| **u != 0).count() as u64;
            mechanical_ticks += u64::from(state.body.rates.iter().any(|v| *v != 0));
            for (i, rate) in state.body.rates.iter().enumerate() {
                participation[i] += u64::from(*rate != 0);
            }
        }
        if let Some(file) = &mut tracefile {
            let selected = world
                .brains()
                .get(&1)
                .or_else(|| world.brains().values().next());
            let row = if let Some(rig) = selected {
                let n = rig.neural.as_ref().unwrap();
                json!({"tick":world.tick_index(),"id":rig.id,"parent":rig.parent,"generation":rig.generation,"genotype_id":n.genome.id(),"activations":n.activations,"sensors":n.sensors,"commands":n.commands,"realized_efforts":n.realized_efforts,"angles":n.body.angles,"angular_rates":n.body.rates,"body_orientation":n.body.orientation,"velocity":n.body.velocity,"acceleration":n.body.acceleration,"momentum":n.body.momentum,"plastic":n.plastic,"cost_neural":n.spent_neural,"cost_motor":n.spent_actuator,"reward":n.reward,"intake":n.last_intake,"environment_momentum":world.environment_momentum(),"jets":world.jets()})
            } else {
                json!({"tick":world.tick_index(),"extinct":true})
            };
            writeln!(file, "{row}").map_err(|e| e.to_string())?;
        }
    }
    if let Some(file) = &mut tracefile {
        file.flush().map_err(|e| e.to_string())?;
    }
    if let Some(file) = &mut lineagefile {
        file.flush().map_err(|e| e.to_string())?;
    }
    let accounts = world.accounts().map_err(|e| e.to_string())?;
    if trace {
        fs::write(
            out.join(format!("final_{environment}_{seed}.bin")),
            SnapshotFile::encode(world.config(), &world).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        fs::write(
            out.join(format!("config_{environment}_{seed}.json")),
            serde_json::to_vec_pretty(world.config()).unwrap(),
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(
        json!({"status":"COMPLETE","seed":seed,"environment":environment,"condition":condition,"ticks":ticks,"initial_hash":initial_hash,"initial_genotypes":initial_genomes,"final_hash":world.snapshot().state_hash(),"initial_energy":initial_energy,"final_energy":accounts.energy_live,"energy_credited":accounts.energy_credited,"intake":intake,"early_intake":early_intake,"late_intake":late_intake,"distance_sites":movement,"net_displacement_subsites":net_displacement,"jets_released":launches,"excavated":excavated,"births":births,"live":world.live_count(),"dead":world.dead_count(),"flesh":accounts.flesh,"in_flight":accounts.in_flight,"motor_cost":fees,"neural_cost":neural_fees,"plastic_l1_max":largest_plastic,"commands_nonzero":commands_nonzero,"mechanically_active_ticks":mechanical_ticks,"axis_participation":participation,"descendants_measured":descendant_total,"changed_germline_descendants":inherited_changes,"common_garden_different_descendants":behavioral_descendants,"material_residual":accounts.material_residual() as i64,"energy_residual":accounts.energy_residual() as i64,"elapsed_seconds":started.elapsed().as_secs_f64()}),
    )
}
fn benchmark(population: u32) -> Result<Value, String> {
    let mut c = SimConfig::neural_farm_256();
    c.width = 16;
    c.height = 16;
    c.depth = 16;
    c.initial_population = population;
    c.farm.as_mut().unwrap().max_population = population;
    c.farm.as_mut().unwrap().birth_energy = c.energy_cap;
    c.initial_energy = 512;
    c.initial_nutrient_per_site = 20;
    c.nutrient_wave_amplitude = 0;
    let mut w = World::genesis(&c, 1234).map_err(|e| e.to_string())?;
    let start = Instant::now();
    for _ in 0..64 {
        w.tick().map_err(|e| e.to_string())?;
    }
    Ok(
        json!({"initial_population":population,"final_population":w.live_count(),"ticks":64,"seconds":start.elapsed().as_secs_f64(),"grid":[16,16,16],"includes":"World::tick cloning, integration, contact, material/momentum audits; excludes rendering and snapshot hashing"}),
    )
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if args.replicates < 2 || args.replicates > 32 || args.ticks < 32 || args.ticks > 4096 {
        return Err("study bounds: 2..32 replicates, 32..4096 ticks".into());
    }
    if args.out.exists() && fs::read_dir(&args.out)?.next().is_some() {
        return Err("study output must be empty; old results are preserved".into());
    }
    fs::create_dir_all(&args.out)?;
    fs::write(
        args.out.join("plan.json"),
        serde_json::to_vec_pretty(
            &json!({"version":1,"replicates":args.replicates,"ticks":args.ticks,"seeds":"1..=replicates","motor_probe":"automated neural capability tests; controlled, not learned","conditions":["plastic_on","plastic_off","random","recurrence_off","proprioception_off","chemistry_off"],"environments":["foraging","contact"],"lineage_conditions":["plastic_on","mutation_off"],"primary_measures":["paired intake and energy outcomes","early versus late intake","distance and recoil events","actuator participation","plastic changes","common-garden descendant commands"],"null_reporting":"no direct locomotion/building/digging reward; all seeds and conditions retained, including failures/deaths; no purposeful learning verdict from reward alone"}),
        )?,
    )?;
    let anatomy = moebius_core::neural::schema::Anatomy::reference();
    fs::write(
        args.out.join("interfaces.json"),
        serde_json::to_vec_pretty(
            &json!({"anatomy":anatomy,"sensor_names":anatomy.sensor_names(),"motor_axes":anatomy.axes,"additional_motors":["secretion_valve","nozzle_body_x","nozzle_body_y","nozzle_body_z"],"normalization":"Q=1024; see docs/neural/ARCHITECTURE.md"}),
        )?,
    )?;
    let mut results = Vec::new();
    for env in ["foraging", "contact", "lineage"] {
        let conditions: Vec<_> = if env == "lineage" {
            vec!["plastic_on", "mutation_off"]
        } else {
            vec![
                "plastic_on",
                "plastic_off",
                "random",
                "recurrence_off",
                "proprioception_off",
                "chemistry_off",
            ]
        };
        for condition in conditions {
            for seed in 1..=args.replicates {
                let result=trial(&args.out,seed,env,condition,args.ticks,seed==1&&condition=="plastic_on").unwrap_or_else(|error|json!({"status":"FAIL","seed":seed,"environment":env,"condition":condition,"error":error}));
                results.push(result);
            }
            fs::write(
                args.out.join("results.partial.json"),
                serde_json::to_vec_pretty(&results)?,
            )?;
        }
    }
    let mut performance = Vec::new();
    for population in [1, 16, 64, 192, 512] {
        performance.push(benchmark(population).unwrap_or_else(
            |e| json!({"initial_population":population,"status":"FAIL","error":e}),
        ));
    }
    let failures = results.iter().filter(|r| r["status"] == "FAIL").count();
    let report = json!({"version":1,"base_commit":moebius_cli::code_version(),"source_state":"local working tree; source hashes must accompany delivery","results":results,"performance":performance,"failed_trials":failures,"verdicts":{"A_motor_activity":"see neural capability tests and measured per-axis participation","B_experience_dependent_change":"requires matched analysis; plastic deltas alone are not learned competence","C_locomotion_discovery":"NOT DEMONSTRATED by aggregate reward; recoil locomotion measured separately","D_genetic_divergence":"see common-garden comparisons; adaptive evolution NOT DEMONSTRATED","E_learned_tunnelling":"NOT DEMONSTRATED; excavation events are physical capability, not purpose","F_ablations":"matched conditions recorded; analysis required"}});
    fs::write(
        args.out.join("results.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    println!(
        "{}",
        json!({"out":args.out,"trials":report["results"].as_array().unwrap().len(),"failures":failures})
    );
    if failures > 0 {
        return Err("one or more trials failed; complete failure records retained".into());
    }
    Ok(())
}
