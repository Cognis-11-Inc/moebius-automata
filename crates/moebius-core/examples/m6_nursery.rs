#[path = "support/m6_assess.rs"]
mod assessment;
#[path = "support/m6.rs"]
mod motor;
#[path = "support/m6_search.rs"]
mod search;
use motor::*;
use serde_json::{Value, json};
use std::{fs, path::PathBuf};
fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let stage = args.next().ok_or("stage")?;
    let out = PathBuf::from(args.next().ok_or("new output directory")?);
    if out.exists() {
        return Err("refuse overwrite".into());
    }
    fs::create_dir_all(&out)?;
    fs::write(
        out.join("protocol.json"),
        include_bytes!("../../../docs/m6/PROTOCOL.json"),
    )?;
    if stage == "optimize" {
        let gate = args.next().ok_or("capability measurements path")?;
        fs::copy(gate, out.join("capability_gate.json"))?;
        return search::campaign(&out, None);
    }
    if stage == "direction" {
        let selected = PathBuf::from(args.next().ok_or("movement genome")?);
        let gate = args.next().ok_or("held-out movement gate")?;
        fs::copy(gate, out.join("movement_gate.json"))?;
        return search::campaign(&out, Some(&selected));
    }
    if stage == "direction_assess" {
        let selected = PathBuf::from(args.next().ok_or("direction genome")?);
        let movement = PathBuf::from(args.next().ok_or("movement genome")?);
        return assessment::direction(&out, &selected, &movement);
    }
    if stage == "falsify" {
        let selected = PathBuf::from(args.next().ok_or("locked movement genome")?);
        return assessment::falsify(&out, &selected);
    }
    if stage == "assess" || stage == "lifetime" {
        let checkpoint = PathBuf::from(args.next().ok_or("locked genome path")?);
        return if stage == "assess" {
            assessment::assess(&out, &checkpoint)
        } else {
            assessment::lifetime(&out, &checkpoint)
        };
    }
    let g = original();
    save(&out.join("original_genome.json"), &g)?;
    let mut trace = Trace::new(&out.join("episodes.jsonl.gz"))?;
    let mut results: Vec<Value> = Vec::new();
    if stage == "capability" {
        for seed in [101, 102, 103] {
            for drag in [8, 32] {
                for mode in [
                    Mode::Neural,
                    Mode::Random,
                    Mode::Rest,
                    Mode::Pulse,
                    Mode::Periodic,
                ] {
                    let mut e = Episode::train(seed);
                    e.drag = drag;
                    e.ticks = 256;
                    e.pose = if seed == 103 { 96 } else { 0 };
                    let name = format!("cap_{seed}_{drag}_{mode:?}");
                    let result = run(&name, &g, &e, mode, None, false, &mut trace);
                    results.push(match result {
                        Ok((m, _)) => serde_json::to_value(m)?,
                        Err(error) => {
                            json!({"name":name,"status":"FAIL","error":error.to_string()})
                        }
                    });
                    save(&out.join("measurements.json"), &results)?;
                }
            }
        }
    } else if stage == "babble" {
        for seed in 401..=408 {
            for condition in [101, 102, 103] {
                for mode in [
                    Mode::BabbleIndividual,
                    Mode::BabbleReverse,
                    Mode::BabbleSimultaneous,
                    Mode::BabbleSequential,
                    Mode::BabblePhase,
                ] {
                    let mut e = Episode::train(condition);
                    e.seed = seed;
                    e.plasticity = true;
                    e.pose = if condition == 103 { 128 } else { 0 };
                    let name = format!("babble_{seed}_{condition}_{mode:?}");
                    results.push(serde_json::to_value(
                        run(&name, &g, &e, mode, None, false, &mut trace)?.0,
                    )?);
                }
            }
        }
        save(&out.join("measurements.json"), &results)?;
    } else {
        return Err("unknown stage".into());
    }
    trace.finish()?;
    println!(
        "{}",
        json!({"out":out,"episodes":results.len(),"sustained":results.iter().filter(|r|r["sustained"]==true).count()})
    );
    Ok(())
}
