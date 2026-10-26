use std::collections::BTreeMap;

use serde::Deserialize;

use crate::config::MAX_HYPOTHESIS_MARKS;
use crate::error::Error;
use crate::geom::{Dims, Site, site_index};

/// A committed position sample. This is an observation, not a bond.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrailPoint {
    pub tick: u64,
    pub id: u64,
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

impl TrailPoint {
    pub fn site(self) -> Site {
        Site {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

pub fn format_trails(points: &[TrailPoint]) -> String {
    let mut out = String::from("tick,id,x,y,z\n");
    for point in points {
        out.push_str(&format!(
            "{},{},{},{},{}\n",
            point.tick, point.id, point.x, point.y, point.z
        ));
    }
    out
}

pub fn parse_trails(text: &str) -> Result<Vec<TrailPoint>, Error> {
    let mut lines = text.lines();
    let header = lines
        .next()
        .ok_or_else(|| Error::Run("trail file is empty".into()))?;
    if header != "tick,id,x,y,z" {
        return Err(Error::Run(format!("unexpected trail header {header}")));
    }
    let mut points = Vec::new();
    for (number, line) in lines.enumerate() {
        if line.is_empty() {
            continue;
        }
        if points.len() as u64 >= crate::config::MAX_TRAIL_ROWS {
            return Err(Error::Run(format!(
                "trail file exceeds {} rows",
                crate::config::MAX_TRAIL_ROWS
            )));
        }
        let mut parts = line.split(',');
        let tick = parse_u64(parts.next(), number, "tick")?;
        let id = parse_u64(parts.next(), number, "id")?;
        let x = parse_u32(parts.next(), number, "x")?;
        let y = parse_u32(parts.next(), number, "y")?;
        let z = parse_u32(parts.next(), number, "z")?;
        if parts.next().is_some() {
            return Err(Error::Run(format!(
                "trail row {} has extra columns",
                number + 2
            )));
        }
        points.push(TrailPoint { tick, id, x, y, z });
    }
    Ok(points)
}

/// Display segmentation. A step whose axis delta exceeds 1 is a periodic wrap or a gap,
/// not an adhesion. The pieces are not bonds and are not part of the state hash.
pub fn split_trail(points: &[Site]) -> Vec<Vec<Site>> {
    let mut runs: Vec<Vec<Site>> = Vec::new();
    for site in points {
        let extend = runs.last().and_then(|run| run.last()).is_some_and(|prev| {
            let dx = prev.x.abs_diff(site.x);
            let dy = prev.y.abs_diff(site.y);
            let dz = prev.z.abs_diff(site.z);
            dx <= 1 && dy <= 1 && dz <= 1
        });
        if extend {
            runs.last_mut().expect("run exists").push(*site);
        } else {
            runs.push(vec![*site]);
        }
    }
    runs
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FluidSample {
    pub origin: Site,
    pub amount: u64,
    pub sites: u32,
}

/// Block sums of the nutrient field. The samples add up to the field; they are not a new stock.
pub fn fluid_samples(nutrient: &[u64], dims: Dims, stride: u32) -> Result<Vec<FluidSample>, Error> {
    if stride == 0 {
        return Err(Error::Run("fluid stride must be >= 1".into()));
    }
    let volume = dims.volume()? as usize;
    if nutrient.len() != volume {
        return Err(Error::Invariant(format!(
            "fluid source length {} != volume {volume}",
            nutrient.len()
        )));
    }
    let mut samples = Vec::new();
    let mut covered = 0u64;
    let mut summed = 0u64;
    let mut z = 0u32;
    while z < dims.depth {
        let mut y = 0u32;
        while y < dims.height {
            let mut x = 0u32;
            while x < dims.width {
                let mut amount = 0u64;
                let mut sites = 0u32;
                for dz in 0..stride {
                    let zz = z + dz;
                    if zz >= dims.depth {
                        break;
                    }
                    for dy in 0..stride {
                        let yy = y + dy;
                        if yy >= dims.height {
                            break;
                        }
                        for dx in 0..stride {
                            let xx = x + dx;
                            if xx >= dims.width {
                                break;
                            }
                            let index = site_index(
                                Site {
                                    x: xx,
                                    y: yy,
                                    z: zz,
                                },
                                dims,
                            )?;
                            amount = amount
                                .checked_add(nutrient[index])
                                .ok_or_else(|| Error::Overflow("fluid block".into()))?;
                            sites += 1;
                        }
                    }
                }
                covered += u64::from(sites);
                summed = summed
                    .checked_add(amount)
                    .ok_or_else(|| Error::Overflow("fluid total".into()))?;
                samples.push(FluidSample {
                    origin: Site { x, y, z },
                    amount,
                    sites,
                });
                x = x.saturating_add(stride);
                if x == 0 {
                    break;
                }
            }
            y = y.saturating_add(stride);
            if y == 0 {
                break;
            }
        }
        z = z.saturating_add(stride);
        if z == 0 {
            break;
        }
    }
    if covered != volume as u64 {
        return Err(Error::Invariant(format!(
            "fluid samples covered {covered} sites, volume is {volume}"
        )));
    }
    let field: u64 = nutrient
        .iter()
        .try_fold(0u64, |acc, value| acc.checked_add(*value))
        .ok_or_else(|| Error::Overflow("nutrient sum".into()))?;
    if summed != field {
        return Err(Error::Invariant(format!(
            "fluid samples sum to {summed}, field is {field}"
        )));
    }
    Ok(samples)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HypothesisMark {
    pub site: Site,
    pub weight: u32,
}

/// Non-authoritative annotation. M0 never writes organism membership into the world.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HypothesisLayer {
    pub name: String,
    pub note: String,
    pub marks: Vec<HypothesisMark>,
}

impl HypothesisLayer {
    pub fn scaffold(name: &str) -> Result<Self, Error> {
        validate_layer_name(name)?;
        Ok(Self {
            name: name.to_string(),
            note: "M0 scaffold. No organism detector writes this layer.".into(),
            marks: Vec::new(),
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct ObservationDeck {
    hypotheses: BTreeMap<String, HypothesisLayer>,
}

impl ObservationDeck {
    pub fn insert_scaffold(&mut self, name: &str) -> Result<(), Error> {
        let layer = HypothesisLayer::scaffold(name)?;
        self.hypotheses.insert(layer.name.clone(), layer);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&HypothesisLayer> {
        self.hypotheses.get(name)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HypothesisFile {
    name: String,
    note: String,
    sites: Vec<[u64; 4]>,
}

pub fn parse_hypothesis(text: &str) -> Result<HypothesisLayer, Error> {
    let file: HypothesisFile = serde_json::from_str(text)
        .map_err(|err| Error::Run(format!("hypothesis JSON rejected: {err}")))?;
    validate_layer_name(&file.name)?;
    if file.note.is_empty() || file.note.len() > 200 {
        return Err(Error::Run(
            "hypothesis note must be 1..=200 characters".into(),
        ));
    }
    if file.sites.len() > MAX_HYPOTHESIS_MARKS {
        return Err(Error::Run(format!(
            "hypothesis marks {} exceed {MAX_HYPOTHESIS_MARKS}",
            file.sites.len()
        )));
    }
    let mut marks = Vec::with_capacity(file.sites.len());
    let mut seen = std::collections::BTreeSet::new();
    for row in file.sites {
        let site = Site {
            x: u32::try_from(row[0])
                .map_err(|_| Error::Run(format!("x {} does not fit u32", row[0])))?,
            y: u32::try_from(row[1])
                .map_err(|_| Error::Run(format!("y {} does not fit u32", row[1])))?,
            z: u32::try_from(row[2])
                .map_err(|_| Error::Run(format!("z {} does not fit u32", row[2])))?,
        };
        let weight = u32::try_from(row[3])
            .map_err(|_| Error::Run(format!("weight {} does not fit u32", row[3])))?;
        if weight == 0 {
            return Err(Error::Run(format!(
                "hypothesis weight at {site:?} is 0; omit the site instead"
            )));
        }
        if !seen.insert(site) {
            return Err(Error::Run(format!("duplicate hypothesis site {site:?}")));
        }
        marks.push(HypothesisMark { site, weight });
    }
    Ok(HypothesisLayer {
        name: file.name,
        note: file.note,
        marks,
    })
}

pub fn validate_hypothesis_grid(layer: &HypothesisLayer, dims: Dims) -> Result<(), Error> {
    for mark in &layer.marks {
        site_index(mark.site, dims)?;
    }
    Ok(())
}

fn validate_layer_name(name: &str) -> Result<(), Error> {
    let bytes = name.as_bytes();
    if !(1..=32).contains(&bytes.len()) {
        return Err(Error::Run(format!(
            "layer name {name:?} must be 1..=32 characters"
        )));
    }
    let Ok(first) = std::str::from_utf8(&bytes[..1]) else {
        return Err(Error::Run("layer name is not ascii".into()));
    };
    if !first
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_lowercase())
    {
        return Err(Error::Run(format!(
            "layer name {name:?} must start with a lowercase letter"
        )));
    }
    if !name
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
    {
        return Err(Error::Run(format!(
            "layer name {name:?} may contain only lowercase letters, digits, and underscore"
        )));
    }
    Ok(())
}

fn parse_u64(part: Option<&str>, row: usize, field: &str) -> Result<u64, Error> {
    let Some(text) = part else {
        return Err(Error::Run(format!("trail row {} missing {field}", row + 2)));
    };
    text.parse::<u64>()
        .map_err(|_| Error::Run(format!("trail row {} has bad {field}", row + 2)))
}

fn parse_u32(part: Option<&str>, row: usize, field: &str) -> Result<u32, Error> {
    let value = parse_u64(part, row, field)?;
    u32::try_from(value)
        .map_err(|_| Error::Run(format!("trail row {} {field} exceeds u32", row + 2)))
}
