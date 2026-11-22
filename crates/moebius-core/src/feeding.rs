//! Authored nursery law: fixed local substrate sites, not a food-seeking policy.
use crate::{Dims, Site, config::FeedingRules};

pub fn sites(rules: &FeedingRules, dims: Dims) -> Vec<Site> {
    let mut sites = Vec::new();
    let start = rules.spacing / 2;
    for z in (start..dims.depth).step_by(rules.spacing as usize) {
        for y in (start..dims.height).step_by(rules.spacing as usize) {
            for x in (start..dims.width).step_by(rules.spacing as usize) {
                sites.push(Site { x, y, z });
            }
        }
    }
    sites
}

/// Short oral capture reach, including periodic boundaries. The deterministic
/// ordering prefers the head site, then nearest sites; no distant search.
pub fn mouth_sites(head: Site, dims: Dims) -> Vec<Site> {
    let mut offsets = Vec::new();
    for z in -1i32..=1 {
        for y in -1i32..=1 {
            for x in -1i32..=1 {
                let d2 = x * x + y * y + z * z;
                if d2 <= 2 {
                    offsets.push((d2, x, y, z));
                }
            }
        }
    }
    offsets.sort();
    offsets
        .into_iter()
        .map(|(_, x, y, z)| Site {
            x: (i64::from(head.x) + i64::from(x)).rem_euclid(i64::from(dims.width)) as u32,
            y: (i64::from(head.y) + i64::from(y)).rem_euclid(i64::from(dims.height)) as u32,
            z: (i64::from(head.z) + i64::from(z)).rem_euclid(i64::from(dims.depth)) as u32,
        })
        .collect()
}
