#[path = "support/m5_control.rs"]
mod control;
#[path = "support/m5.rs"]
mod support;
use serde_json::json;
use std::{fs, path::PathBuf};
use support::*;
fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let stage = args.next().ok_or("stage required")?;
    let root = PathBuf::from(args.next().ok_or("new output directory required")?);
    if root.exists() {
        return Err("refuse overwrite".into());
    }
    fs::create_dir_all(&root)?;
    fs::write(
        root.join("protocol.json"),
        include_bytes!("../../../docs/m5/PROTOCOL.json"),
    )?;
    let mut results = Vec::new();
    if stage == "screen" {
        for drag in [0, 8, 32, 128] {
            for energy in [128, 2048] {
                for pattern in [
                    "bend",
                    "opposed",
                    "front",
                    "back",
                    "twist",
                    "bend_twist",
                    "jaw",
                ] {
                    for amplitude in [-1024i32, -512, 512, 1024] {
                        for pulse in [4, 8, 16] {
                            let schedule = Schedule {
                                pattern: pattern.into(),
                                amplitude,
                                pulse,
                                period: 0,
                                recovery: 0,
                                ablate: None,
                            };
                            let case = Case {
                                name: format!("{pattern}_{amplitude}_{pulse}_d{drag}_e{energy}"),
                                seed: 1,
                                drag,
                                energy,
                                target_delta: Some([0, 3, 0]),
                                founding_bite: false,
                                horizon: if energy == 128 { 128 } else { 256 },
                            };
                            let result =
                                trial(&root, &case, json!({"schedule":schedule}), |w, t, _| {
                                    Ok(Some(schedule.commands(w, t)))
                                });
                            results.push(result.unwrap_or_else(
                                |e| json!({"status":"FAIL","case":case,"error":e.to_string()}),
                            ));
                        }
                    }
                }
                save_results(&root, &results)?;
                println!(
                    "screen drag={drag} energy={energy} results={}",
                    results.len()
                );
            }
        }
    } else if stage == "periodic" {
        for pattern in ["bend", "front", "back", "bend_twist"] {
            for amplitude in [-1024i32, -512, 512, 1024] {
                for pulse in [4, 8, 12] {
                    for period in [16, 24, 32] {
                        for recovery_factor in [0, -1] {
                            let recovery = recovery_factor * amplitude.signum() * 256;
                            let schedule = Schedule {
                                pattern: pattern.into(),
                                amplitude,
                                pulse,
                                period,
                                recovery,
                                ablate: None,
                            };
                            let case = Case {
                                name: format!(
                                    "{pattern}_{amplitude}_p{pulse}_r{period}_back{recovery}"
                                ),
                                seed: 1,
                                drag: 8,
                                energy: 128,
                                target_delta: Some([0, 3, 0]),
                                founding_bite: false,
                                horizon: 128,
                            };
                            results.push(
                                trial(&root, &case, json!({"schedule":schedule}), |w, t, _| {
                                    Ok(Some(schedule.commands(w, t)))
                                })
                                .unwrap_or_else(
                                    |e| json!({"status":"FAIL","case":case,"error":e.to_string()}),
                                ),
                            );
                        }
                    }
                }
                save_results(&root, &results)?;
            }
            println!("periodic {pattern} results={}", results.len());
        }
    } else if stage == "attribution" {
        let mut schedules = Vec::new();
        let base = Schedule {
            pattern: "bend_twist".into(),
            amplitude: 1024,
            pulse: 16,
            period: 0,
            recovery: 0,
            ablate: None,
        };
        schedules.push(base.clone());
        for axis in 0..26 {
            let mut ablated = base.clone();
            ablated.ablate = Some(axis);
            schedules.push(ablated);
            for amplitude in [-1024, 1024] {
                schedules.push(Schedule {
                    pattern: format!("axis_{axis}"),
                    amplitude,
                    pulse: 8,
                    period: 0,
                    recovery: 0,
                    ablate: None,
                });
            }
        }
        for (index, schedule) in schedules.into_iter().enumerate() {
            let case = Case {
                name: format!("joint_{index}"),
                seed: 1,
                drag: 8,
                energy: 128,
                target_delta: None,
                founding_bite: false,
                horizon: 128,
            };
            results.push(
                trial(&root, &case, json!({"schedule":schedule}), |w, t, _| {
                    Ok(Some(schedule.commands(w, t)))
                })
                .unwrap_or_else(|e| json!({"status":"FAIL","case":case,"error":e.to_string()})),
            );
        }
    } else if stage == "braking" {
        for suffix in [
            "release",
            "reverse4",
            "reverse8",
            "reverse16",
            "turn",
            "brake_oracle",
        ] {
            let prefix = Schedule {
                pattern: "bend_twist".into(),
                amplitude: 1024,
                pulse: 16,
                period: 0,
                recovery: 0,
                ablate: None,
            };
            let case = Case {
                name: format!("brake_{suffix}"),
                seed: 1,
                drag: 8,
                energy: 128,
                target_delta: None,
                founding_bite: false,
                horizon: 96,
            };
            let result = trial(
                &root,
                &case,
                json!({"prefix":prefix,"suffix":suffix,"switch_tick":16}),
                |w, t, _| {
                    if t < 16 {
                        return Ok(Some(prefix.commands(w, t)));
                    }
                    if suffix == "brake_oracle" {
                        return Ok(Some(control::brake(w)?));
                    }
                    let amplitude =
                        if suffix.starts_with("reverse") && t - 16 < suffix[7..].parse::<u64>()? {
                            -1024
                        } else if suffix == "turn" && t < 32 {
                            1024
                        } else {
                            0
                        };
                    let schedule = Schedule {
                        pattern: if suffix == "turn" {
                            "back".into()
                        } else {
                            "bend_twist".into()
                        },
                        amplitude,
                        pulse: 128,
                        period: 0,
                        recovery: 0,
                        ablate: None,
                    };
                    Ok(Some(schedule.commands(w, t)))
                },
            );
            results
                .push(result.unwrap_or_else(
                    |e| json!({"status":"FAIL","case":case,"error":e.to_string()}),
                ));
        }
    } else if stage == "benchmark"
        || stage == "second_meal"
        || stage == "local_model"
        || stage == "local_model_second"
        || stage == "local_lobes"
        || stage == "local_lobes_second"
    {
        let seeds: u64 = args.next().unwrap_or("1".into()).parse()?;
        let targets = [
            [3, 0, 0],
            [-3, 0, 0],
            [0, 3, 0],
            [0, -3, 0],
            [0, 0, 3],
            [0, 0, -3],
            [2, 2, 1],
            [-2, 2, 1],
            [2, -2, 1],
            [2, 1, 2],
            [1, 2, 2],
            [-1, -2, -2],
        ];
        for seed in 1..=seeds {
            for (direction, d) in targets.into_iter().enumerate() {
                let kinds: Vec<&str> = if stage.starts_with("local_lobes") {
                    vec!["local_lobes"]
                } else if stage.starts_with("local_model") {
                    vec!["local_model"]
                } else {
                    vec!["oracle", "local", "random", "rest", "recurrent"]
                };
                for kind in kinds {
                    let case = Case {
                        name: format!("{kind}_{direction}_s{seed}"),
                        seed,
                        drag: 8,
                        energy: 128,
                        target_delta: Some(d),
                        founding_bite: stage == "second_meal" || stage.ends_with("_second"),
                        horizon: 128,
                    };
                    let mut local = control::LocalController::default();
                    let mut model = control::LocalModelController::default();
                    let mut log = Vec::new();
                    let mut held = vec![0; 30];
                    let result = trial(&root, &case, json!({"kind":kind}), |w, t, target| {
                        Ok(match kind {
                            "oracle" => {
                                if t % 4 == 0 || (stage == "second_meal" && t == 1) {
                                    held = control::oracle(w, target.unwrap(), &mut log)?;
                                }
                                Some(held.clone())
                            }
                            "local" => Some(local.commands(&w.organism_sensors(ID)?)),
                            "local_model" | "local_lobes" => {
                                Some(model.commands(&w.organism_sensors(ID)?))
                            }
                            "rest" => Some(vec![0; 30]),
                            _ => None,
                        })
                    });
                    if !log.is_empty() {
                        fs::write(
                            root.join(format!("{}.oracle.json", case.name)),
                            serde_json::to_vec(&log)?,
                        )?;
                    }
                    let result = result.unwrap_or_else(
                        |e| json!({"status":"FAIL","case":case,"error":e.to_string()}),
                    );
                    println!(
                        "{kind} seed{seed} direction{direction} target_tick={}",
                        result["first_target_tick"]
                    );
                    results.push(result);
                    save_results(&root, &results)?;
                }
            }
        }
    } else if stage == "sensors" || stage == "sensors_lobes" {
        let mut displacements = vec![
            [3, 0, 0],
            [-3, 0, 0],
            [0, 3, 0],
            [0, -3, 0],
            [0, 0, 3],
            [0, 0, -3],
        ];
        for small_axis in 0..3 {
            for sx in [-1, 1] {
                for sy in [-1, 1] {
                    for sz in [-1, 1] {
                        let mut d = [2 * sx, 2 * sy, 2 * sz];
                        d[small_axis] /= 2;
                        displacements.push(d);
                    }
                }
            }
        }
        for anchor in ["com", "mouth"] {
            for orientation in [[0, 0, 0], [0, 0, 700], [0, 0, 804], [0, 600, 600]] {
                for d in &displacements {
                    let case = Case {
                        name: "sensor_probe".into(),
                        seed: 1,
                        drag: 8,
                        energy: 128,
                        target_delta: None,
                        founding_bite: false,
                        horizon: 0,
                    };
                    let (world, _) = fixture(&case)?;
                    let mut snapshot = world.snapshot();
                    snapshot.brains[0].neural.as_mut().unwrap().body.orientation = orientation;
                    let state = snapshot.brains[0].neural.as_ref().unwrap();
                    let head = state.body.head_site(
                        &state.genome.anatomy,
                        snapshot.cells[0].site,
                        snapshot.brains[0].offset,
                        world.dims(),
                    );
                    let origin = if anchor == "com" {
                        snapshot.cells[0].site
                    } else {
                        head
                    };
                    let coordinates = [origin.x, origin.y, origin.z];
                    let p: [u32; 3] = std::array::from_fn(|a| {
                        (i64::from(coordinates[a]) + i64::from(d[a])).rem_euclid(i64::from(SIDE))
                            as u32
                    });
                    let target = moebius_core::Site {
                        x: p[0],
                        y: p[1],
                        z: p[2],
                    };
                    snapshot.nutrient[(p[0] + SIDE * (p[1] + SIDE * p[2])) as usize] = 512;
                    snapshot.material_initial = 512;
                    let mut config = world.config().clone();
                    if stage == "sensors_lobes" {
                        config.neural.as_mut().unwrap().chemical_sensing =
                            moebius_core::neural::ChemicalSensing::LocalLobesV1;
                    }
                    let w = moebius_core::World::restore(&config, &snapshot)?;
                    let sensors = w.organism_sensors(ID)?;
                    let snapped:Vec<_>=(0..6).map(|face|{
                    let mut local=[0;3];local[face/2]=if face%2==0{1024}else{-1024};
                    let rotated=moebius_core::neural::body::rotate(local,orientation);
                    let axis=(0..3).max_by_key(|a|rotated[*a].abs()).unwrap();
                    json!({"face":face,"world_axis":axis,"sign":rotated[axis].signum(),"rotated":rotated})
                }).collect();
                    results.push(json!({"status":"PASS","sensing_variant":stage,"anchor":anchor,"offset":d,"distance_squared":d.iter().map(|v|v*v).sum::<i32>(),
                    "orientation":orientation,"com":com(&w),"mouth":mouth(&w),"head_site":head,"target":target,
                    "ordinary_sensors":sensors,"chemical":&sensors[93..99],"snapped_directions":snapped,
                    "capture_eligible":moebius_core::feeding::mouth_sites(head,w.dims()).contains(&target)}));
                }
            }
        }
    } else {
        return Err("unknown stage".into());
    }
    save_results(&root, &results)?;
    if results.iter().any(|r| r["status"] != "PASS") {
        return Err("execution failures preserved in results.json".into());
    }
    Ok(())
}
