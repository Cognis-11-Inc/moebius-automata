//! Locked-checkpoint comparisons and a separate lifetime-plasticity experiment.
use super::motor::*;
use moebius_core::neural::ControllerGenome;
use serde_json::{Value, json};
use std::{fs, path::Path};
pub fn assess(out: &Path, selected_path: &Path) -> Result<()> {
    let selected: ControllerGenome = serde_json::from_slice(&fs::read(selected_path)?)?;
    selected.validate()?;
    let original = original();
    save(&out.join("locked_selected.json"), &selected)?;
    let mut trace = Trace::new(&out.join("episodes.jsonl.gz"))?;
    let mut results = Vec::new();
    for seed in 601..=608 {
        for control in [
            "selected_frozen",
            "selected_live",
            "original_frozen",
            "original_live",
            "random",
            "rest",
            "pulse",
            "periodic",
            "selected_no_afferent",
            "selected_feedback_off",
            "selected_recurrence_off",
        ] {
            let mut e = Episode::held(seed);
            e.plasticity = control.ends_with("_live");
            let mut g = if control.starts_with("original")
                || ["random", "rest", "pulse", "periodic"].contains(&control)
            {
                original.clone()
            } else {
                selected.clone()
            };
            if control == "selected_no_afferent" {
                g.sensory.fill(0);
            }
            if control == "selected_feedback_off" {
                e.feedback = false;
            }
            // Recurrent connections are zeroed; leaky integration remains intact.
            let mode = match control {
                "random" => Mode::Random,
                "rest" => Mode::Rest,
                "pulse" => Mode::Pulse,
                "periodic" => Mode::Periodic,
                _ => Mode::Neural,
            };
            if control == "selected_recurrence_off" {
                g.recurrent.fill(0);
            }
            let name = format!("held_{seed}_{control}");
            let (m, _) = run(&name, &g, &e, mode, None, false, &mut trace)?;
            results.push(json!({"control":control,"measurement":m}));
        }
        save(&out.join("measurements.json"), &results)?;
    }
    trace.finish()?;
    println!(
        "{}",
        json!({"episodes":results.len(),"selected_sustained":results.iter().filter(|r|r["control"]=="selected_frozen" && r["measurement"]["sustained"]==true).count()})
    );
    Ok(())
}
pub fn lifetime(out: &Path, selected_path: &Path) -> Result<()> {
    let selected: ControllerGenome = serde_json::from_slice(&fs::read(selected_path)?)?;
    let original = original();
    let mut trace = Trace::new(&out.join("episodes.jsonl.gz"))?;
    let mut results: Vec<Value> = Vec::new();
    for (label, g) in [("original", original), ("selected", selected)] {
        for arm in [
            "plasticity_on",
            "plasticity_off",
            "feedback_off",
            "untrained",
        ] {
            let mut e = Episode::train(101);
            e.ticks = 512;
            e.plasticity = arm == "plasticity_on" || arm == "feedback_off";
            e.feedback = arm != "feedback_off";
            let (state, experience) = if arm == "untrained" {
                let w = world(&g, &e, None, false)?;
                (w.brains()[&1].neural.as_ref().unwrap().clone(), None)
            } else {
                let (m, w) = run(
                    &format!("experience_{label}_{arm}"),
                    &g,
                    &e,
                    Mode::Neural,
                    None,
                    false,
                    &mut trace,
                )?;
                save(
                    &out.join(format!("experience_{label}_{arm}.json")),
                    w.brains()[&1].neural.as_ref().unwrap(),
                )?;
                (w.brains()[&1].neural.as_ref().unwrap().clone(), Some(m))
            };
            let inherited = state.offspring(606, 2, 512, 4096, false).genome;
            let changed = state
                .plastic
                .iter()
                .chain(&state.plastic_output)
                .filter(|v| **v != 0)
                .count();
            for seed in [101, 102, 103, 601, 602, 603, 604, 605, 606, 607, 608] {
                let mut probe = if seed < 600 {
                    Episode::train(seed)
                } else {
                    Episode::held(seed)
                };
                probe.ticks = 128;
                probe.feedback = false;
                probe.plasticity = false;
                for kind in ["pre", "post_frozen", "post_carry", "fresh_inherited"] {
                    let gene = if kind == "fresh_inherited" {
                        &inherited
                    } else {
                        &g
                    };
                    let retained = if kind == "post_frozen" || kind == "post_carry" {
                        Some(&state)
                    } else {
                        None
                    };
                    let (m, _) = run(
                        &format!("life_{label}_{arm}_{seed}_{kind}"),
                        gene,
                        &probe,
                        Mode::Neural,
                        retained,
                        kind == "post_carry",
                        &mut trace,
                    )?;
                    results.push(json!({"controller":label,"arm":arm,"probe":kind,"changed_nonzero_synapses":changed,"parent_genome":g.id(),"inherited_genome":inherited.id(),"experience":experience,"measurement":m}));
                }
            }
            save(&out.join("measurements.json"), &results)?;
            println!("lifetime {label} {arm}: changed_synapses={changed}");
        }
    }
    trace.finish()?;
    Ok(())
}

pub fn direction(out: &Path, directional_path: &Path, movement_path: &Path) -> Result<()> {
    let selected: ControllerGenome = serde_json::from_slice(&fs::read(directional_path)?)?;
    let movement: ControllerGenome = serde_json::from_slice(&fs::read(movement_path)?)?;
    let factory = original();
    let mut trace = Trace::new(&out.join("episodes.jsonl.gz"))?;
    let mut results = Vec::new();
    for seed in 701..=708 {
        for control in [
            "direction_selected",
            "movement_selected",
            "original",
            "random",
            "rest",
            "periodic",
        ] {
            let mut e = Episode::held(seed);
            e.cue = Some(Cue {
                axis: if seed.is_multiple_of(2) { 1 } else { 0 },
                first_sign: if seed % 4 < 2 { 1 } else { -1 },
                switch_tick: 128,
            });
            let g = match control {
                "direction_selected" => &selected,
                "movement_selected" => &movement,
                _ => &factory,
            };
            let mode = match control {
                "random" => Mode::Random,
                "rest" => Mode::Rest,
                "periodic" => Mode::Periodic,
                _ => Mode::Neural,
            };
            let (m, _) = run(
                &format!("direction_{seed}_{control}"),
                g,
                &e,
                mode,
                None,
                false,
                &mut trace,
            )?;
            results.push(json!({"control":control,"direction_pass":m.directional_phases.iter().all(|v|*v>=0.1) && m.jets==0,"measurement":m}));
        }
        save(&out.join("measurements.json"), &results)?;
    }
    trace.finish()?;
    println!(
        "{}",
        json!({"episodes":results.len(),"selected_passes":results.iter().filter(|r|r["control"]=="direction_selected" && r["direction_pass"]==true).count()})
    );
    Ok(())
}

/// Post-selection causal probes; never fed back into optimizer selection.
pub fn falsify(out: &Path, selected_path: &Path) -> Result<()> {
    let selected: ControllerGenome = serde_json::from_slice(&fs::read(selected_path)?)?;
    let mut trace = Trace::new(&out.join("episodes.jsonl.gz"))?;
    let mut results = Vec::new();
    for seed in 601..=608 {
        for control in ["selected_vacuum", "selected_bias_only", "selected_shuffled"] {
            let mut e = Episode::held(seed);
            let mut g = selected.clone();
            if control == "selected_vacuum" {
                e.drag = 0;
            }
            if control == "selected_bias_only" {
                g.output[..26 * 8].fill(0);
            }
            if control == "selected_shuffled" {
                let mut rng = Rng(60620261124);
                let mut axes: Vec<_> = (0..26).collect();
                for i in (1..26).rev() {
                    let j = rng.below((i + 1) as u64) as usize;
                    axes.swap(i, j);
                }
                for (i, from) in axes.into_iter().enumerate() {
                    g.output[i * 8..(i + 1) * 8]
                        .copy_from_slice(&selected.output[from * 8..(from + 1) * 8]);
                    g.motor_biases[i] = selected.motor_biases[from];
                }
            }
            let (m, _) = run(
                &format!("falsify_{seed}_{control}"),
                &g,
                &e,
                Mode::Neural,
                None,
                false,
                &mut trace,
            )?;
            results.push(json!({"control":control,"measurement":m}));
        }
        save(&out.join("measurements.json"), &results)?;
    }
    trace.finish()?;
    Ok(())
}
