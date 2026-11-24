//! Generic food in the ship volume. One kind, uneven sites, bounded regrowth.
//! A bite bursts seeds. Seeds grow faster, and faster again beside bound secretion.
//! This is not a metabolism and not locomotor learning.

use crate::{MorphError, emission::SCALE, runtime::Position};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum FoodKind {
    Generic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum FoodStage {
    Patch,
    Seed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FoodRules {
    pub regen_every: u64,
    pub seed_every: u64,
    pub flesh_radius: i64,
    pub eat_radius: i64,
    pub chew_radius: i64,
    pub dried_age: u64,
    pub fresh_tail: usize,
    pub chew_below: u64,
    pub bite: u64,
    pub seeds_per_bite: u32,
    pub max_sites: usize,
}

impl FoodRules {
    pub fn generic() -> Self {
        Self {
            regen_every: 40,
            seed_every: 4,
            flesh_radius: 2 * SCALE,
            eat_radius: 2 * SCALE,
            chew_radius: 2 * SCALE,
            dried_age: 48,
            fresh_tail: 12,
            chew_below: 4,
            bite: 1,
            seeds_per_bite: 12,
            max_sites: 2048,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FoodSite {
    pub id: u64,
    pub kind: FoodKind,
    pub stage: FoodStage,
    pub position: Position,
    pub quantity: u64,
    pub capacity: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FoodField {
    pub rules: FoodRules,
    pub next_id: u64,
    pub sites: BTreeMap<u64, FoodSite>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bite {
    pub site: u64,
    pub seeds: u64,
    pub organism: u64,
}

impl FoodField {
    pub fn validate(&self) -> Result<(), MorphError> {
        let r = self.rules;
        crate::require(
            r.regen_every > 0
                && r.seed_every > 0
                && r.seed_every < r.regen_every
                && r.flesh_radius > 0
                && r.eat_radius > 0
                && r.chew_radius > 0
                && r.bite > 0
                && r.seeds_per_bite > 0
                && (1..=4096).contains(&r.max_sites)
                && self.sites.len() <= r.max_sites
                && self.next_id > 0,
            "invalid generic food rules",
        )?;
        for (&id, site) in &self.sites {
            crate::require(
                id == site.id
                    && id < self.next_id
                    && site.kind == FoodKind::Generic
                    && site.capacity > 0
                    && site.quantity <= site.capacity
                    && site.position.absolute().is_ok(),
                "invalid food site",
            )?;
        }
        Ok(())
    }

    /// Grow, then let each mouth take one bite from the nearest mature patch.
    /// Returns one record per burst. Growth is state, not an event.
    pub fn step(
        &mut self,
        tick: u64,
        flesh: &[[i64; 3]],
        mouths: &[(u64, Position)],
    ) -> Result<Vec<Bite>, MorphError> {
        let rules = self.rules;
        let ids: Vec<u64> = self.sites.keys().copied().collect();
        for id in ids {
            let Some(site) = self.sites.get(&id) else {
                continue;
            };
            let every = match site.stage {
                FoodStage::Patch => rules.regen_every,
                FoodStage::Seed => rules.seed_every,
            };
            if !tick.is_multiple_of(every) || site.quantity >= site.capacity {
                continue;
            }
            let ripe = near(&site.position, flesh, rules.flesh_radius)?;
            let Some(site) = self.sites.get_mut(&id) else {
                continue;
            };
            let gain = if ripe { 2 } else { 1 };
            site.quantity = (site.quantity + gain).min(site.capacity);
            if site.stage == FoodStage::Seed && site.quantity == site.capacity {
                site.stage = FoodStage::Patch;
            }
        }
        let mut bursts = Vec::new();
        for (organism, mouth) in mouths {
            let Some(id) = nearest_patch(&self.sites, *mouth, rules.eat_radius, rules.bite)? else {
                continue;
            };
            let site = self.sites.get_mut(&id).unwrap();
            site.quantity -= rules.bite;
            let origin = site.position;
            let parent = site.id;
            if site.quantity == 0 {
                self.sites.remove(&id);
            }
            let room = rules.max_sites.saturating_sub(self.sites.len()) as u32;
            let born = rules.seeds_per_bite.min(room);
            for n in 0..born {
                let id = self.next_id;
                self.next_id = self.next_id.saturating_add(1);
                self.sites.insert(
                    id,
                    FoodSite {
                        id,
                        kind: FoodKind::Generic,
                        stage: FoodStage::Seed,
                        position: origin.offset(seed_offset(parent, tick, n))?,
                        quantity: 1,
                        capacity: 8,
                    },
                );
            }
            bursts.push(Bite {
                site: parent,
                seeds: u64::from(born),
                organism: *organism,
            });
        }
        self.validate()?;
        Ok(bursts)
    }
}

/// Uneven generic stock. Most of the 30,000 km cylinder is empty.
pub fn generic_field(seed: u64) -> Result<FoodField, MorphError> {
    let mut field = FoodField {
        rules: FoodRules::generic(),
        next_id: 1,
        sites: BTreeMap::new(),
    };
    let clusters = [
        ([6 * SCALE, 0, 0], 28, 5 * SCALE),
        ([8_000 * SCALE, 200 * SCALE, -80 * SCALE], 10, 40 * SCALE),
        (
            [400_000 * SCALE, -1_000 * SCALE, 600 * SCALE],
            8,
            80 * SCALE,
        ),
        ([12_000_000 * SCALE, 0, 0], 6, 200 * SCALE),
    ];
    let mut n = seed | 1;
    for (center, count, radius) in clusters {
        for _ in 0..count {
            n = n.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let unit = |shift: u32| -> i64 {
                let span = (radius as u64) * 2 + 1;
                let draw = ((n >> shift) % span) as i64;
                draw - radius
            };
            let position = Position::from_absolute([
                center[0] + unit(0),
                center[1] + unit(8) / 3,
                center[2] + unit(16),
            ])?;
            let capacity = 6 + (n % 19);
            let id = field.next_id;
            field.next_id += 1;
            field.sites.insert(
                id,
                FoodSite {
                    id,
                    kind: FoodKind::Generic,
                    stage: FoodStage::Patch,
                    position,
                    quantity: capacity / 2,
                    capacity,
                },
            );
        }
    }
    field.validate()?;
    Ok(field)
}

fn nearest_patch(
    sites: &BTreeMap<u64, FoodSite>,
    mouth: Position,
    radius: i64,
    bite: u64,
) -> Result<Option<u64>, MorphError> {
    let mut best: Option<(i128, u64)> = None;
    let limit = i128::from(radius) * i128::from(radius);
    for site in sites.values() {
        if site.stage != FoodStage::Patch || site.quantity < bite {
            continue;
        }
        let d = mouth.relative(site.position)?;
        let distance: i128 = d.iter().map(|v| i128::from(*v) * i128::from(*v)).sum();
        if distance <= limit && best.is_none_or(|(best_d, _)| distance < best_d) {
            best = Some((distance, site.id));
        }
    }
    Ok(best.map(|(_, id)| id))
}

fn near(position: &Position, flesh: &[[i64; 3]], radius: i64) -> Result<bool, MorphError> {
    let limit = i128::from(radius) * i128::from(radius);
    for point in flesh {
        let there = Position::from_absolute(*point)?;
        let d = position.relative(there)?;
        let distance: i128 = d.iter().map(|v| i128::from(*v) * i128::from(*v)).sum();
        if distance <= limit {
            return Ok(true);
        }
    }
    Ok(false)
}

fn seed_offset(parent: u64, tick: u64, n: u32) -> [i64; 3] {
    let mixed = parent
        .wrapping_mul(0x9E37)
        .wrapping_add(tick)
        .wrapping_add(u64::from(n) * 17);
    let axis = |shift: u32| -> i64 {
        let span = 400_000i64 + ((mixed >> shift) % 900_000) as i64;
        if (mixed >> (shift + 3)) & 1 == 0 {
            span
        } else {
            -span
        }
    };
    [axis(0), axis(8) / 2, axis(16)]
}
