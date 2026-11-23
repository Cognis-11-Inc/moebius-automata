//! Bounded genotype optimization; no authored gait or mutation of body genes.
use super::motor::*;
use moebius_core::neural::ControllerGenome;
use serde_json::{Value, json};
use std::{fs, path::Path, time::Instant};
#[derive(Clone)]
struct Candidate {
    g: ControllerGenome,
    parent: Option<String>,
    changes: Value,
}
fn fresh(base: &ControllerGenome, r: &mut Rng) -> Candidate {
    let mut g = base.clone();
    for w in &mut g.recurrent {
        *w = r.signed(4096);
    }
    for w in &mut g.sensory {
        *w = r.signed(512);
    }
    for w in &mut g.output[..26 * 8] {
        *w = r.signed(2048);
    }
    for w in &mut g.biases {
        *w = r.signed(1024);
    }
    for w in &mut g.motor_biases[..26] {
        *w = r.signed(1024);
    }
    for t in &mut g.tau {
        *t = 4 + r.below(29) as i32;
    }
    Candidate {
        g,
        parent: None,
        changes: json!({"initializer":"independent uniform bounded NN parameters; last4 motor rows/body unchanged"}),
    }
}
fn mutate(parent: &ControllerGenome, r: &mut Rng) -> Candidate {
    let mut g = parent.clone();
    let mut changes = Vec::new();
    for (name, weights, prob, scale, limit) in [
        ("recurrent", &mut g.recurrent[..], 15, 512, 4096),
        ("sensory", &mut g.sensory[..], 6, 256, 2048),
        ("output", &mut g.output[..26 * 8], 15, 512, 2048),
        ("biases", &mut g.biases[..], 40, 256, 2048),
        ("motor_biases", &mut g.motor_biases[..26], 15, 128, 1024),
    ] {
        for (i, w) in weights.iter_mut().enumerate() {
            if r.below(100) < prob {
                let old = *w;
                *w = (*w + r.signed(scale)).clamp(-limit, limit);
                if *w != old {
                    changes.push(json!({"locus":name,"index":i,"before":old,"after":*w}));
                }
            }
        }
    }
    for (i, t) in g.tau.iter_mut().enumerate() {
        if r.below(100) < 30 {
            let old = *t;
            *t = (*t + r.signed(4)).clamp(4, 64);
            if old != *t {
                changes.push(json!({"locus":"tau","index":i,"before":old,"after":*t}));
            }
        }
    }
    Candidate {
        g,
        parent: Some(parent.id()),
        changes: json!(changes),
    }
}
pub fn same_body(a: &ControllerGenome, b: &ControllerGenome) -> bool {
    a.anatomy == b.anatomy
        && a.strengths == b.strengths
        && a.ligament_stiffness == b.ligament_stiffness
        && a.ligament_rest == b.ligament_rest
        && a.muscle_rate == b.muscle_rate
        && a.sensory_gains == b.sensory_gains
        && a.motor_gains == b.motor_gains
        && a.plastic_rates == b.plastic_rates
        && a.output[26 * 8..] == b.output[26 * 8..]
        && a.motor_biases[26..] == b.motor_biases[26..]
}
pub fn campaign(out: &Path, direction_initial: Option<&Path>) -> Result<()> {
    let directional = direction_initial.is_some();
    let base: ControllerGenome = if let Some(path) = direction_initial {
        serde_json::from_slice(&fs::read(path)?)?
    } else {
        original()
    };
    if directional {
        let rows: Vec<Value> = serde_json::from_slice(&fs::read(out.join("movement_gate.json"))?)?;
        if rows
            .iter()
            .filter(|r| {
                r["control"] == "selected_frozen"
                    && r["measurement"]["genome"] == base.id()
                    && r["measurement"]["sustained"] == true
            })
            .count()
            < 6
        {
            return Err("sustained movement gate not met".into());
        }
    } else {
        let cap: Vec<Measurement> =
            serde_json::from_slice(&fs::read(out.join("capability_gate.json"))?)?;
        if !cap.iter().any(|m| m.sustained) {
            return Err("physical gate not passed".into());
        }
    }
    let start = Instant::now();
    let optimizer_seed = if directional {
        60620261123u64
    } else {
        60620261122u64
    };
    let mut r = Rng(optimizer_seed);
    let population = if directional { 12 } else { 16 };
    let generations = if directional { 6 } else { 12 };
    let elites = if directional { 3 } else { 4 };
    let mut pop = vec![Candidate {
        g: base.clone(),
        parent: None,
        changes: json!(if directional {
            "locked movement-selected warm start"
        } else {
            "untrained factory reference"
        }),
    }];
    while pop.len() < population {
        pop.push(fresh(&base, &mut r));
    }
    let mut trace = Trace::new(&out.join("episodes.jsonl.gz"))?;
    let mut lineage = Trace::new(&out.join("candidate_genomes.jsonl.gz"))?;
    let mut history = Vec::new();
    let mut best: Option<(f64, ControllerGenome, usize)> = None;
    let mut budget_stop = false;
    for generation in 0..generations {
        let mut measured = Vec::new();
        let mut seen = std::collections::BTreeSet::new();
        for (slot, candidate) in pop.iter().enumerate() {
            if start.elapsed().as_secs() > if directional { 300 } else { 900 }
                || fs::metadata(&trace.path)?.len() > 536870912
            {
                budget_stop = true;
                break;
            }
            candidate.g.validate()?;
            if !same_body(&candidate.g, &base) {
                return Err("body mutation forbidden".into());
            }
            let id = candidate.g.id();
            seen.insert(id.clone());
            lineage.row(&json!({"generation":generation,"slot":slot,"genome_id":id,"genome":candidate.g,"parent":candidate.parent,"changes":candidate.changes,"rng_state":r.0}))?;
            let mut episodes = Vec::new();
            for seed in if directional {
                [301, 302, 303]
            } else {
                [101, 102, 103]
            } {
                let mut e = Episode::train(seed);
                if directional {
                    e.orientation = if seed == 302 {
                        [0, 0, 1608]
                    } else if seed == 303 {
                        [0, 300, -600]
                    } else {
                        [0; 3]
                    };
                    e.drag = if seed == 303 { 32 } else { 8 };
                    e.cue = Some(Cue {
                        axis: if seed == 302 { 0 } else { 1 },
                        first_sign: if seed == 301 { 1 } else { -1 },
                        switch_tick: 64,
                    });
                }
                let name = format!("g{generation}_s{slot}_e{seed}");
                episodes.push(
                    run(
                        &name,
                        &candidate.g,
                        &e,
                        Mode::Neural,
                        None,
                        false,
                        &mut trace,
                    )?
                    .0,
                );
            }
            let score = episodes.iter().map(|m| m.score).sum::<f64>() / 3.;
            if best.as_ref().is_none_or(|b| score > b.0) {
                best = Some((score, candidate.g.clone(), generation));
                save(&out.join("selected_genome.json"), &candidate.g)?;
            }
            history.push(json!({"generation":generation,"slot":slot,"genome_id":id,"parent":candidate.parent,"score":score,"episodes":episodes}));
            measured.push((score, candidate.g.clone()));
        }
        measured.sort_by(|a, b| b.0.total_cmp(&a.0).then_with(|| a.1.id().cmp(&b.1.id())));
        save(&out.join("history.json"), &history)?;
        println!(
            "generation{generation}: candidates={} unique={} best={:.6} elapsed={}s",
            measured.len(),
            seen.len(),
            best.as_ref().unwrap().0,
            start.elapsed().as_secs()
        );
        if budget_stop || generation == generations - 1 {
            break;
        }
        let mut next = Vec::new();
        for (_, g) in measured.iter().take(elites) {
            next.push(Candidate {
                g: g.clone(),
                parent: Some(g.id()),
                changes: json!("elite retained"),
            });
        }
        for _ in 0..elites {
            next.push(fresh(&base, &mut r));
        }
        while next.len() < population {
            let parent = &measured[r.below(measured.len().min(8) as u64) as usize].1;
            next.push(mutate(parent, &mut r));
        }
        pop = next;
    }
    trace.finish()?;
    lineage.finish()?;
    let best = best.unwrap();
    save(
        &out.join("selection.json"),
        &json!({"selected":best.1.id(),"score":best.0,"generation":best.2,"evaluated_candidates":history.len(),"budget_stop":budget_stop,"elapsed_seconds":start.elapsed().as_secs(),"optimizer_seed":optimizer_seed,"held_out_opened":false}),
    )?;
    save(&out.join("original_genome.json"), &base)?;
    Ok(())
}
