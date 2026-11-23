//! Replay full M6 neural/motor traces with ordinary or explicitly controlled ticks.
#[allow(dead_code)]
#[path = "support/m6.rs"]
mod motor;
use moebius_core::{SimConfig, Site, World, neural::NeuralState};
use motor::*;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    io::{BufRead, BufReader},
    path::PathBuf,
    process::{Command, Stdio},
};
fn main() -> Result<()> {
    let path = PathBuf::from(std::env::args().nth(1).ok_or("trace.jsonl.gz")?);
    let mut child = Command::new("gzip")
        .arg("-dc")
        .arg(&path)
        .stdout(Stdio::piped())
        .spawn()?;
    let mut w: Option<World> = None;
    let mut coast = None;
    let mut episode_ticks = 0;
    let mut actual_ticks = 0;
    let mut episodes = 0;
    let mut cue_events = 0;
    let mut generation_initial = String::new();
    for line in BufReader::new(child.stdout.take().unwrap()).lines() {
        let value: Value = serde_json::from_str(&line?)?;
        match value["type"].as_str().ok_or("record type")? {
            "start" => {
                let c: SimConfig = serde_json::from_value(value["config"].clone())?;
                let e: Episode = serde_json::from_value(value["episode"].clone())?;
                let mut s = World::genesis(&c, e.seed)?.snapshot();
                s.cells[0].site = Site { x: 8, y: 8, z: 8 };
                let n: NeuralState = serde_json::from_value(value["initial_neural_state"].clone())?;
                s.brains[0].neural = Some(n);
                s.brains[0].sync_neural_pose();
                if let Some(field) = value.get("initial_field") {
                    s.nutrient = serde_json::from_value(field.clone())?;
                    s.material_initial = s.nutrient.iter().sum();
                }
                let world = World::restore(&c, &s)?;
                assert_eq!(world.snapshot().state_hash(), value["initial_hash"]);
                generation_initial = value["initial_hash"].as_str().unwrap().into();
                episode_ticks = e.ticks;
                w = Some(world);
                coast = None;
            }
            "cue" => {
                let current = w.as_ref().ok_or("cue without episode")?;
                let mut s = current.snapshot();
                let old = s.nutrient.iter().sum::<u64>();
                s.nutrient = serde_json::from_value(value["field"].clone())?;
                assert_eq!(old, s.nutrient.iter().sum::<u64>());
                let changed = World::restore(current.config(), &s)?;
                assert_eq!(changed.snapshot().state_hash(), value["state_hash"]);
                w = Some(changed);
                cue_events += 1;
            }
            "tick" => {
                let t = value["tick"].as_u64().unwrap();
                let current = w.as_mut().ok_or("tick without start")?;
                let (r, rows) = if value["submitted_commands"].is_null() {
                    current.tick_observed()?
                } else {
                    let cmd: Vec<i32> =
                        serde_json::from_value(value["submitted_commands"].clone())?;
                    current.tick_controlled(&BTreeMap::from([(1, cmd)]))?
                };
                assert_eq!(serde_json::to_value(r)?, value["receipt"]);
                assert_eq!(serde_json::to_value(&rows[0])?, value["organism"]);
                let mut dynamic =
                    serde_json::to_value(current.brains()[&1].neural.as_ref().unwrap())?;
                dynamic.as_object_mut().unwrap().remove("genome");
                assert_eq!(dynamic, value["neural"]);
                assert_eq!(current.snapshot().state_hash(), value["state_hash"]);
                let mut momentum = current.environment_momentum().map(i128::from);
                for b in current.brains().values() {
                    if let Some(n) = &b.neural {
                        for (a, p) in momentum.iter_mut().enumerate() {
                            *p += i128::from(n.body.momentum[a]);
                        }
                    }
                }
                for jet in current.jets() {
                    for (a, p) in momentum.iter_mut().enumerate() {
                        *p += i128::from(jet.momentum[a]);
                    }
                }
                assert_eq!(momentum, [0; 3]);
                assert_eq!(current.accounts()?.material_residual(), 0);
                assert_eq!(current.accounts()?.energy_residual(), 0);
                if t + 1 == episode_ticks / 2 {
                    coast = Some(current.clone());
                }
                if t >= episode_ticks / 2 {
                    coast
                        .as_mut()
                        .ok_or("missing coast")?
                        .tick_controlled(&BTreeMap::from([(1, vec![0; 30])]))?;
                }
                actual_ticks += 1;
            }
            "end" => {
                let current = w.take().ok_or("end without start")?;
                assert_eq!(
                    current.snapshot().state_hash(),
                    value["measurement"]["final_hash"]
                );
                assert_eq!(generation_initial, value["measurement"]["initial_hash"]);
                assert_eq!(
                    serde_json::to_value(current.brains()[&1].neural.as_ref().unwrap())?,
                    value["final_neural_state"]
                );
                assert_eq!(
                    coast.take().ok_or("missing coast")?.snapshot().state_hash(),
                    value["coast_final_hash"]
                );
                episodes += 1;
            }
            other => return Err(format!("unknown record {other}").into()),
        }
    }
    if !child.wait()?.success() {
        return Err("gzip failed".into());
    }
    println!(
        "{}",
        json!({"trace":path,"status":"PASS","episodes":episodes,"ticks":actual_ticks,"cue_events":cue_events,"full_neural_motor_state_and_hashes":"exact","linear_momentum_residual":0})
    );
    Ok(())
}
