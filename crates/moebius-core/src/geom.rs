use std::collections::{BTreeMap, BTreeSet};

use crate::error::Error;

/// Face-centered 6-neighborhood, the 3D analogue of a 4-neighbor lattice.
pub const NEIGHBORHOOD: &str = "von-neumann-6-periodic";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Dims {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

impl Dims {
    pub fn volume(self) -> Result<u64, Error> {
        let volume = u64::from(self.width)
            .checked_mul(u64::from(self.height))
            .and_then(|v| v.checked_mul(u64::from(self.depth)))
            .ok_or_else(|| Error::Overflow("grid volume".into()))?;
        if volume == 0 {
            return Err(Error::Invariant("grid volume is 0".into()));
        }
        Ok(volume)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Site {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dir {
    Stay,
    Xp,
    Xm,
    Yp,
    Ym,
    Zp,
    Zm,
}

impl Dir {
    pub fn from_roll(roll: u32) -> Result<Self, Error> {
        Ok(match roll {
            0 => Self::Stay,
            1 => Self::Xp,
            2 => Self::Xm,
            3 => Self::Yp,
            4 => Self::Ym,
            5 => Self::Zp,
            6 => Self::Zm,
            _ => {
                return Err(Error::Invariant(format!(
                    "movement roll {roll} is outside 0..=6"
                )));
            }
        })
    }

    pub fn delta(self) -> (i32, i32, i32) {
        match self {
            Self::Stay => (0, 0, 0),
            Self::Xp => (1, 0, 0),
            Self::Xm => (-1, 0, 0),
            Self::Yp => (0, 1, 0),
            Self::Ym => (0, -1, 0),
            Self::Zp => (0, 0, 1),
            Self::Zm => (0, 0, -1),
        }
    }

    pub fn opposite(self) -> Self {
        match self {
            Self::Stay => Self::Stay,
            Self::Xp => Self::Xm,
            Self::Xm => Self::Xp,
            Self::Yp => Self::Ym,
            Self::Ym => Self::Yp,
            Self::Zp => Self::Zm,
            Self::Zm => Self::Zp,
        }
    }
}

pub fn wrap_coord(coord: u32, delta: i32, extent: u32) -> Result<u32, Error> {
    if extent == 0 {
        return Err(Error::Invariant("cannot wrap an axis of extent 0".into()));
    }
    let extent_i = i64::from(extent);
    let value = i64::from(coord) + i64::from(delta);
    Ok(u32::try_from(value.rem_euclid(extent_i)).expect("wrapped coordinate fits u32"))
}

pub fn step_site(site: Site, dir: Dir, dims: Dims) -> Result<Site, Error> {
    let (dx, dy, dz) = dir.delta();
    Ok(Site {
        x: wrap_coord(site.x, dx, dims.width)?,
        y: wrap_coord(site.y, dy, dims.height)?,
        z: wrap_coord(site.z, dz, dims.depth)?,
    })
}

pub fn site_index(site: Site, dims: Dims) -> Result<usize, Error> {
    if site.x >= dims.width || site.y >= dims.height || site.z >= dims.depth {
        return Err(Error::Invariant(format!(
            "site {site:?} is outside {}×{}×{}",
            dims.width, dims.height, dims.depth
        )));
    }
    let width = dims.width as usize;
    let height = dims.height as usize;
    Ok(site.x as usize + width * (site.y as usize + height * site.z as usize))
}

pub fn site_from_index(linear: u32, dims: Dims) -> Result<Site, Error> {
    let volume = dims.volume()?;
    if u64::from(linear) >= volume {
        return Err(Error::Invariant(format!(
            "linear index {linear} is outside volume {volume}"
        )));
    }
    let width = u64::from(dims.width);
    let height = u64::from(dims.height);
    let plane = width * height;
    let lin = u64::from(linear);
    Ok(Site {
        x: u32::try_from(lin % width).expect("x fits"),
        y: u32::try_from((lin / width) % height).expect("y fits"),
        z: u32::try_from(lin / plane).expect("z fits"),
    })
}

/// Coordinate wave used only as an initial nutrient condition.
/// Amplitude 0 yields 0. Otherwise the value is in `0..amplitude`.
pub fn lattice_wave(x: u32, y: u32, z: u32, amplitude: u64) -> u64 {
    if amplitude == 0 {
        return 0;
    }
    let x = u64::from(x);
    let y = u64::from(y);
    let z = u64::from(z);
    (x * 3 + y * 5 + z * 7 + x * z + y * (x + 1)) % amplitude
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MoveProposal {
    pub id: u64,
    pub from: Site,
    pub to: Site,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MoveOutcome {
    pub id: u64,
    pub from: Site,
    pub intended: Site,
    pub to: Site,
    pub moved: bool,
}

/// M0 movement resolution.
///
/// A cell may enter a site only when that site is empty in the pre-move snapshot.
/// If several cells propose the same empty site, the lowest `CellId` wins.
/// Losers stay. Occupied destinations are refused, so cells do not swap or chain.
/// Origins must be unique. Outcomes are returned in ascending id order.
pub fn resolve_moves(proposals: &[MoveProposal]) -> Result<Vec<MoveOutcome>, Error> {
    let mut props = proposals.to_vec();
    props.sort_by_key(|proposal| proposal.id);
    for pair in props.windows(2) {
        if pair[0].id == pair[1].id {
            return Err(Error::Invariant(format!(
                "duplicate movement proposal for cell {}",
                pair[0].id
            )));
        }
    }
    let mut origins = BTreeSet::new();
    for proposal in &props {
        if !origins.insert(proposal.from) {
            return Err(Error::Invariant(format!(
                "two cells propose to leave {:?}",
                proposal.from
            )));
        }
    }
    let mut winner_at: BTreeMap<Site, u64> = BTreeMap::new();
    for proposal in &props {
        if proposal.to == proposal.from || origins.contains(&proposal.to) {
            continue;
        }
        winner_at.entry(proposal.to).or_insert(proposal.id);
    }
    let moving: BTreeSet<u64> = winner_at.values().copied().collect();
    let mut outcomes = Vec::with_capacity(props.len());
    let mut destinations = BTreeSet::new();
    for proposal in props {
        let moved = moving.contains(&proposal.id);
        let to = if moved { proposal.to } else { proposal.from };
        if !destinations.insert(to) {
            return Err(Error::Invariant(format!(
                "movement resolution placed two cells at {to:?}"
            )));
        }
        outcomes.push(MoveOutcome {
            id: proposal.id,
            from: proposal.from,
            intended: proposal.to,
            to,
            moved,
        });
    }
    Ok(outcomes)
}

/// Integer uptake. A nutrient unit is taken only when the whole energy credit fits.
/// Material conversion itself is 1 nutrient -> 1 waste; energy is a separate credit.
pub fn uptake_units(
    site_nutrient: u64,
    uptake_per_tick: u64,
    energy: u64,
    energy_cap: u64,
    energy_per_nutrient: u64,
) -> u64 {
    let by_stock = site_nutrient.min(uptake_per_tick);
    if energy_per_nutrient == 0 {
        return by_stock;
    }
    let room = energy_cap.saturating_sub(energy);
    let by_energy = room / energy_per_nutrient;
    by_stock.min(by_energy)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn site(x: u32, y: u32, z: u32) -> Site {
        Site { x, y, z }
    }

    #[test]
    fn wrap_crosses_each_min_edge() {
        assert_eq!(wrap_coord(0, -1, 4).unwrap(), 3);
        assert_eq!(wrap_coord(3, 1, 4).unwrap(), 0);
        assert_eq!(wrap_coord(2, 0, 4).unwrap(), 2);
    }

    #[test]
    fn index_roundtrip_includes_last_site() {
        let dims = Dims {
            width: 4,
            height: 3,
            depth: 2,
        };
        let last = site(3, 2, 1);
        let index = site_index(last, dims).unwrap();
        assert_eq!(index, 4 * 3 * 2 - 1);
        assert_eq!(site_from_index(index as u32, dims).unwrap(), last);
        assert_eq!(site_from_index(0, dims).unwrap(), site(0, 0, 0));
    }

    #[test]
    fn lowest_id_wins_an_empty_site() {
        let target = site(1, 0, 0);
        let outcomes = resolve_moves(&[
            MoveProposal {
                id: 2,
                from: site(2, 0, 0),
                to: target,
            },
            MoveProposal {
                id: 1,
                from: site(0, 0, 0),
                to: target,
            },
        ])
        .unwrap();
        assert!(outcomes[0].moved && outcomes[0].to == target);
        assert!(!outcomes[1].moved && outcomes[1].to == site(2, 0, 0));
    }

    #[test]
    fn occupied_destination_and_swaps_stay() {
        let left = site(0, 0, 0);
        let right = site(1, 0, 0);
        let outcomes = resolve_moves(&[
            MoveProposal {
                id: 1,
                from: left,
                to: right,
            },
            MoveProposal {
                id: 2,
                from: right,
                to: left,
            },
        ])
        .unwrap();
        assert!(!outcomes[0].moved && outcomes[0].to == left);
        assert!(!outcomes[1].moved && outcomes[1].to == right);
    }

    #[test]
    fn uptake_refuses_a_partial_energy_credit() {
        assert_eq!(uptake_units(5, 4, 9, 10, 3), 0);
        assert_eq!(uptake_units(5, 4, 0, 10, 3), 3);
        assert_eq!(uptake_units(1, 4, 0, 100, 2), 1);
        assert_eq!(uptake_units(9, 2, 0, 100, 0), 2);
    }
}
