//! 100 independent compositions, eight poses each; preserve every trial, including failures.
use moebius_morph::*;
use serde_json::json;
use std::{f32::consts::TAU, io::Write, path::PathBuf, time::Instant};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = PathBuf::from(
        std::env::args()
            .nth(1)
            .ok_or("new output directory required")?,
    );
    std::fs::create_dir(&out)?;
    let mut rows = std::fs::File::create(out.join("trials.jsonl"))?;
    let mut failures = vec![];
    let mut timing = vec![];
    let mut min_area = f32::MAX;
    let mut max_root_error = 0f32;
    let mut unique = std::collections::BTreeSet::new();
    let mut counts = std::collections::BTreeSet::new();
    let mut poses = 0;
    for seed in 0..100u64 {
        let mut s = AnatomySpec::from_seed(seed);
        let n = [0, 2, 8, 16][seed as usize % 4];
        s.components.clear();
        s.body.body_rings = 32 + (seed % 3) as usize * 16;
        s.body.radial_segments = 24 + (seed % 2) as usize * 8;
        for i in 0..n {
            let mut c = Component::flagellum(
                i + 2,
                0.1 + (i % 4) as f32 * 0.19,
                (TAU * (i as f32 * 0.618 + seed as f32 * 0.03)).rem_euclid(TAU),
            );
            c.orientation = [-0.4 + i as f32 * 0.08, 0.2 + (seed % 3) as f32 * 0.1];
            c.shape = Shape::Flagellum {
                length: 0.6 + (i as f32 * 0.1 + seed as f32 * 0.01) % 1.5,
                radius: 0.015 + (i % 4) as f32 * 0.007,
                taper: 0.6 + (i % 4) as f32 * 0.1,
                curvature: (i % 3) as f32 * 0.05 - 0.05,
                amplitude: 0.05 + (i % 4) as f32 * 0.04,
                frequency: 0.7 + (i % 5) as f32 * 0.31,
                phase: i as f32 * 0.7,
            };
            s.components.push(c);
        }
        if seed % 2 == 0 {
            let mut c = Component::flagellum(s.next_id(), 0.68, 0.7);
            c.shape = Shape::Spine {
                length: 0.35,
                radius: 0.04,
                taper: 0.9,
                curvature: 0.12,
            };
            s.components.push(c);
        }
        if seed % 3 == 0 {
            let mut c = Component::flagellum(s.next_id(), 0.42, 3.7);
            c.orientation = [0.2, 0.6];
            c.shape = Shape::SecretionNozzle {
                length: 0.4,
                radius: 0.07,
                aperture: 0.55,
                output: EmissionMetadata {
                    material_channel: "future_conserved_substrate".into(),
                    operational: false,
                },
            };
            s.components.push(c);
        }
        let result = (|| -> Result<serde_json::Value, Box<dyn std::error::Error>> {
            save_anatomy(&out.join(format!("spec_{seed:03}.json")), &s)?;
            let restored = load_anatomy(&out.join(format!("spec_{seed:03}.json")))?;
            if s != restored {
                return Err("persistence mismatch".into());
            }
            let started = Instant::now();
            let c = CompiledAnatomy::new(s.clone())?;
            let compile_us = started.elapsed().as_micros();
            let again = CompiledAnatomy::new(restored)?;
            let mut trials = vec![];
            for k in 0..8 {
                let p = Pose {
                    phase: k as f32 * 0.83,
                    bend: if k < 2 { 0. } else { 1. },
                    mouth: (k % 2) as f32,
                    flagella: 1.,
                };
                let started = Instant::now();
                let a = c.pose(p)?;
                let pose_us = started.elapsed().as_micros();
                let audits = a.geometry.audit()?;
                if a != again.pose(p)? {
                    return Err("geometry replay mismatch".into());
                }
                for audit in audits {
                    min_area = min_area.min(audit.minimum_double_area);
                }
                for (f, part) in a.geometry.attachments.iter().zip(&a.geometry.surfaces[1..]) {
                    let err = f
                        .origin
                        .iter()
                        .zip(part.positions[0])
                        .map(|(a, b)| (a - b).powi(2))
                        .sum::<f32>()
                        .sqrt();
                    max_root_error = max_root_error.max(err);
                }
                if seed == 0 && k == 0 {
                    std::fs::write(out.join("geometry_witness.json"), serde_json::to_vec(&a)?)?;
                }
                trials.push(json!({"pose":p,"digest":a.geometry.digest(),"topology":a.geometry.topology_digest(),"pose_us":pose_us}));
                poses += 1;
            }
            let a = c.pose(Pose::rest(&s.body))?;
            let vertices = a
                .geometry
                .surfaces
                .iter()
                .map(|s| s.positions.len())
                .sum::<usize>();
            let triangles = a
                .geometry
                .surfaces
                .iter()
                .map(|s| s.indices.len() / 3)
                .sum::<usize>();
            unique.insert(a.geometry.digest());
            counts.insert(n);
            timing.push(json!({"seed":seed,"flagella":n,"components":s.components.len(),"compile_us":compile_us,"vertices":vertices,"triangles":triangles}));
            Ok(
                json!({"seed":seed,"status":"PASS","flagella":n,"components":s.components.len(),"trials":trials}),
            )
        })();
        let row = match result {
            Ok(v) => v,
            Err(e) => {
                failures.push(seed);
                json!({"seed":seed,"status":"FAIL","error":e.to_string(),"spec":s})
            }
        };
        writeln!(rows, "{row}")?;
        rows.flush()?;
    }
    save_anatomy(&out.join("showcase.json"), &AnatomySpec::showcase(20261123))?;
    let s = AnatomySpec::showcase(20261123);
    let c = CompiledAnatomy::new(s.clone())?;
    let g = c
        .pose(Pose {
            phase: 0.,
            bend: 0.,
            mouth: 0.,
            flagella: 0.,
        })?
        .geometry;
    let ring = g
        .mouth_ring
        .iter()
        .map(|&i| g.surfaces[0].positions[i as usize][1])
        .collect::<Vec<_>>();
    let gap = ring.iter().copied().fold(f32::NEG_INFINITY, f32::max)
        - ring.iter().copied().fold(f32::INFINITY, f32::min);
    let report = json!({"status":if failures.is_empty(){"PASS"}else{"FAIL"},"configurations":100,"passed":100-failures.len(),"failed":failures,"poses":poses,"distinct_geometry_digests":unique.len(),"flagella_counts":counts,"minimum_double_area":min_area,"maximum_root_error":max_root_error,"near_closure_gap_showcase":gap,"near_closure_gap_law":"2 * radius * radius_profile[3] * aspect * 0.018; positive seam, not airtight","performance":timing,"numerics":"same-build bit-identical v2 replay; v1 migration positions1e-6/normals2e-4 tolerance; no cross-platform claim","attachments":"actual body barycentric root centers; overlap-mounted, not welded; no global self-intersection certification","physical_or_neural_binding":false});
    std::fs::write(
        out.join("summary.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    println!("{report}");
    if !failures.is_empty() {
        return Err("population failed; rows preserved".into());
    }
    Ok(())
}
