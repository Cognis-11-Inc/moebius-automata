//! Post-hoc decomposition of empty, no-jet selected traces; no world edits.
//! Reproduce lunge_impulse's authored integer formula with public body::rotate.
use moebius_core::neural::body::rotate;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    io::{BufRead, BufReader},
    process::{Command, Stdio},
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("heldout trace")?;
    let mut child = Command::new("gzip")
        .args(["-dc", &path])
        .stdout(Stdio::piped())
        .spawn()?;
    let mut name = String::new();
    let mut bend_axes = Vec::new();
    let mut previous = [0i64; 3];
    let mut drag = 0;
    let mut midpoint = 0;
    let mut stats = [0i64; 5];
    let mut result = BTreeMap::new();
    for line in BufReader::new(child.stdout.take().unwrap()).lines() {
        let v: Value = serde_json::from_str(&line?)?;
        match v["type"].as_str().ok_or("type")? {
            "start" => {
                name = v["name"].as_str().unwrap().into();
                bend_axes = v["initial_neural_state"]["genome"]["anatomy"]["axes"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .enumerate()
                    .filter_map(|(i, a)| (a["axis"] == "Bend").then_some(i))
                    .collect();
                assert_eq!(bend_axes.len(), 12);
                previous =
                    serde_json::from_value(v["initial_neural_state"]["body"]["momentum"].clone())?;
                drag = v["episode"]["drag"].as_i64().unwrap();
                midpoint = v["episode"]["ticks"].as_u64().unwrap() / 2;
                stats = [0; 5];
            }
            "tick" if name.ends_with("selected_frozen") => {
                assert_eq!(v["receipt"]["jet_released"], 0);
                let body = &v["neural"]["body"];
                let momentum: [i64; 3] = serde_json::from_value(body["momentum"].clone())?;
                assert!(momentum.iter().all(|p| p.abs() < 4096));
                if v["tick"].as_u64().unwrap() >= midpoint {
                    let bend: i64 = bend_axes
                        .iter()
                        .map(|i| body["rates"][i].as_i64().unwrap())
                        .sum();
                    let orientation: [i32; 3] =
                        serde_json::from_value(body["orientation"].clone())?;
                    let direction = rotate([0, 4096, 0], orientation);
                    let scale = bend.signum() * (bend.abs() - 280).max(0) * drag / 256;
                    let lunge = direction.map(|x| x * scale / 4096);
                    let shape: [i64; 3] = std::array::from_fn(|a| {
                        momentum[a] - previous[a] + previous[a] * drag / 1024 - lunge[a]
                    });
                    stats[0] += i64::from(bend.abs() > 280);
                    stats[1] += i64::from(lunge.iter().any(|x| *x != 0));
                    stats[2] += lunge.iter().map(|x| x.abs()).sum::<i64>();
                    stats[3] += i64::from(shape.iter().any(|x| *x != 0));
                    stats[4] += shape.iter().map(|x| x.abs()).sum::<i64>();
                }
                previous = momentum;
            }
            "end" if name.ends_with("selected_frozen") => {
                result.insert(name.clone(),json!({"late_bend_threshold_ticks":stats[0],"late_nonzero_lunge_ticks":stats[1],"late_lunge_impulse_L1":stats[2],"late_nonzero_inferred_shape_ticks":stats[3],"late_inferred_shape_impulse_L1":stats[4]}));
            }
            _ => {}
        }
    }
    assert!(child.wait()?.success());
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"status":"PASS","scope":"selected_frozen empty-world no-jet no-momentum-clamp episodes; integer lunge formula reproduced using public rotation; remainder is shape coupling, not independent continuum validation","episodes":result})
        )?
    );
    Ok(())
}
