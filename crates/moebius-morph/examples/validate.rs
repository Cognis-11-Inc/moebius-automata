//! Reproducible MORPH-001 population/pose audit. A new output directory is required.
use moebius_morph::*;
use serde_json::json;
use std::{io::Write, path::PathBuf};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = PathBuf::from(
        std::env::args()
            .nth(1)
            .ok_or("supply a new output directory")?,
    );
    std::fs::create_dir(&out)?;
    let mut rows = std::fs::File::create(out.join("trials.jsonl"))?;
    let mut passed = 0;
    let mut poses_checked = 0;
    let mut hashes = std::collections::BTreeSet::new();
    let mut geometry_hashes = std::collections::BTreeSet::new();
    let mut min_area = f32::MAX;
    let mut max_error = 0f32;
    let mut failures = vec![];
    for seed in 0..100u64 {
        let mut spec = WormSpec::from_seed(seed);
        if seed % 3 == 0 {
            spec.ridges = None;
        }
        if seed % 5 == 0 {
            spec.flagella.truncate((seed % 3) as usize);
        }
        let path = out.join(format!("spec_{seed:03}.json"));
        save(&path, &spec)?;
        let result = (|| -> Result<serde_json::Value, Box<dyn std::error::Error>> {
            let restored = load(&path)?;
            if restored != spec {
                return Err("definition persistence mismatch".into());
            }
            let c = CompiledWorm::new(spec.clone())?;
            let again = CompiledWorm::new(restored)?;
            let base = c.pose(Pose::rest(&spec))?;
            let topology = base.topology_digest();
            let mut pose_rows = vec![];
            for i in 0..8 {
                let pose = Pose {
                    phase: i as f32 * 0.83,
                    bend: if i < 2 { 0. } else { 1. },
                    mouth: if i % 2 == 0 { 0. } else { 1. },
                    flagella: if i < 2 { 0. } else { 1. },
                };
                let geometry = c.pose(pose)?;
                let audit = geometry.audit()?;
                if geometry != again.pose(pose)? || geometry.topology_digest() != topology {
                    return Err("geometry repeatability/topology mismatch".into());
                }
                for a in &audit {
                    min_area = min_area.min(a.minimum_double_area);
                }
                for f in &geometry.attachments {
                    let mut actual = [0.; 3];
                    for (&v, w) in f.triangle.iter().zip(f.barycentric) {
                        for (q, a) in actual.iter_mut().enumerate() {
                            *a += geometry.surfaces[0].positions[v as usize][q] * w;
                        }
                    }
                    let error = actual
                        .iter()
                        .zip(f.origin)
                        .map(|(a, b)| (a - b).powi(2))
                        .sum::<f32>()
                        .sqrt();
                    max_error = max_error.max(error);
                }
                if seed < 3 && i < 2 {
                    std::fs::write(
                        out.join(format!("geometry_{seed:03}_{i}.json")),
                        serde_json::to_vec(&geometry)?,
                    )?;
                }
                pose_rows.push(json!({"pose":pose,"digest":geometry.digest(),"topology":geometry.topology_digest(),"audit":audit}));
                poses_checked += 1;
            }
            hashes.insert(base.digest());
            let mut bytes = vec![];
            for p in &base.surfaces[0].positions {
                for x in p {
                    bytes.extend(x.to_bits().to_le_bytes());
                }
            }
            geometry_hashes.insert(blake3::hash(&bytes).to_hex().to_string());
            Ok(
                json!({"seed":seed,"status":"PASS","length":spec.length,"radius":spec.radius,"mouth_width":spec.mouth.width,"mouth_depth":spec.mouth.depth,"flagella":spec.flagella.len(),"poses":pose_rows}),
            )
        })();
        let row = match result {
            Ok(row) => {
                passed += 1;
                row
            }
            Err(error) => {
                failures.push(seed);
                json!({"seed":seed,"status":"FAIL","error":error.to_string(),"definition":spec})
            }
        };
        writeln!(rows, "{row}")?;
        rows.flush()?;
    }
    let report = json!({"status":if passed==100 {"PASS"}else{"FAIL"},"configurations":100,"passed":passed,"failed":failures,"poses_checked":poses_checked,"repeatable_definition_and_geometry_count":passed,"distinct_rest_geometry_digests":hashes.len(),"distinct_body_position_digests":geometry_hashes.len(),"minimum_double_triangle_area":min_area,"maximum_attachment_error":max_error,"save_reload_count":passed,"environment":std::env::consts::ARCH,"cross_platform_bit_identity":"NOT DEMONSTRATED","self_intersection":"Not globally certified; manifold/degeneracy/root checks only. Inspect live witnesses separately.","attachment_topology":"individually closed tubes; barycentric roots overlap-mounted, not welded union","mechanical_capability":"unbound preview; no new locomotion, feeding or learning"});
    std::fs::write(
        out.join("summary.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    println!("{report}");
    if passed != 100 {
        return Err("MORPH-001 population gate failed; all failed rows preserved".into());
    }
    Ok(())
}
