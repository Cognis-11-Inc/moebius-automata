//! Two-level nonoverlapping leaves; face construction touches actual leaves only.
use super::*;
#[derive(Clone, Copy, Debug, Serialize)]
pub struct Geometry {
    pub key: u64,
    pub lower: V,
    pub h: f64,
    pub level: u8,
}
impl Geometry {
    pub fn volume(self) -> f64 {
        self.h.powi(3)
    }
    pub fn centre(self) -> V {
        self.lower.map(|x| x + self.h * 0.5)
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct Face {
    pub left: Option<u64>,
    pub right: Option<u64>,
    pub axis: usize,
    pub area: f64,
    pub distance: f64,
}
pub fn geometry(c: &Config, key: u64) -> Geometry {
    let root = (key >> 4) as usize;
    let id = key & 15;
    let n = c.roots;
    let p = [root % n[0], root / n[0] % n[1], root / (n[0] * n[1])];
    let level = u8::from(id != 0);
    let child = id.saturating_sub(1) as usize;
    Geometry {
        key,
        lower: std::array::from_fn(|k| {
            p[k] as f64 * c.h
                + if level > 0 {
                    ((child >> k) & 1) as f64 * c.h * 0.5
                } else {
                    0.
                }
        }),
        h: c.h / (1usize << level) as f64,
        level,
    }
}
pub fn validate(s: &State) -> Result<(), MorphError> {
    require(s.cells.len() <= s.config.capacity, "H003 leaf capacity")?;
    for &key in s.cells.keys() {
        require(
            (key >> 4) < s.config.count() as u64 && (key & 15) <= 8,
            "H003 invalid leaf key",
        )?;
    }
    for root in 0..s.config.count() {
        let parent = (root as u64) << 4;
        let children = (1..=8)
            .filter(|&k| s.cells.contains_key(&(parent | k)))
            .count();
        require(
            (s.cells.contains_key(&parent) && children == 0)
                || (!s.cells.contains_key(&parent) && children == 8),
            "H003 overlapping or incomplete physical ownership",
        )?;
    }
    Ok(())
}
pub fn locate(s: &State, p: V) -> Result<u64, MorphError> {
    let c = &s.config;
    let lengths = c.lengths();
    require(
        p.iter()
            .enumerate()
            .all(|(k, v)| v.is_finite() && *v >= 0. && *v < lengths[k]),
        "H003 point outside domain",
    )?;
    let xyz = p.map(|v| (v / c.h).floor() as usize);
    let root = xyz[0] + c.roots[0] * (xyz[1] + c.roots[1] * xyz[2]);
    let key = (root as u64) << 4;
    if s.cells.contains_key(&key) {
        Ok(key)
    } else {
        let child = (0..3)
            .map(|k| usize::from(p[k] - xyz[k] as f64 * c.h >= c.h * 0.5) << k)
            .sum::<usize>();
        let key = key | ((child + 1) as u64);
        require(s.cells.contains_key(&key), "H003 missing leaf")?;
        Ok(key)
    }
}
pub fn faces(s: &State) -> Result<Vec<Face>, MorphError> {
    type FineFace = (usize, usize, usize, usize);
    let c = &s.config;
    let mut tiles: BTreeMap<FineFace, [Option<u64>; 2]> = BTreeMap::new();
    for &key in s.cells.keys() {
        let g = geometry(c, key);
        let width = if g.level == 0 { 2 } else { 1 };
        let p = g.lower.map(|x| (x / (c.h * 0.5)).round() as usize);
        for axis in 0..3 {
            for side in 0..2 {
                for u in 0..width {
                    for v in 0..width {
                        let mut at = p;
                        at[axis] += side * width;
                        at[(axis + 1) % 3] += u;
                        at[(axis + 2) % 3] += v;
                        if c.periodic && at[axis] == 2 * c.roots[axis] {
                            at[axis] = 0;
                        }
                        let f = tiles
                            .entry((axis, at[0], at[1], at[2]))
                            .or_insert([None, None]);
                        let slot = 1 - side;
                        require(f[slot].is_none(), "H003 duplicate face owner")?;
                        f[slot] = Some(key);
                    }
                }
            }
        }
    }
    // Merge coplanar tiles with identical owner pair; preserve fine subfaces where
    // neighbors differ. This is an addressing optimization, not flux normalization.
    let mut pairs: BTreeMap<(usize, Option<u64>, Option<u64>), f64> = BTreeMap::new();
    for ((axis, _, _, _), [left, right]) in tiles {
        require(
            !c.periodic || left.is_some() && right.is_some(),
            "H003 periodic hole",
        )?;
        *pairs.entry((axis, left, right)).or_default() += (c.h * 0.5).powi(2);
    }
    Ok(pairs
        .into_iter()
        .map(|((axis, left, right), area)| {
            let h = |k| geometry(c, k).h;
            let distance = match (left, right) {
                (Some(a), Some(b)) => 0.5 * (h(a) + h(b)),
                (Some(a), None) | (None, Some(a)) => h(a),
                (None, None) => unreachable!(),
            };
            Face {
                left,
                right,
                axis,
                area,
                distance,
            }
        })
        .collect())
}
pub fn refine(s: &mut State, roots: &[usize]) -> Result<(), MorphError> {
    let add = roots.iter().copied().collect::<BTreeSet<_>>();
    require(
        add.iter().all(|r| *r < s.config.count()),
        "H003 refinement root",
    )?;
    let needed = add
        .iter()
        .filter(|r| s.cells.contains_key(&((**r as u64) << 4)))
        .count();
    require(
        s.cells.len() + 7 * needed <= s.config.capacity,
        "H003 refinement capacity",
    )?;
    for root in add {
        let key = (root as u64) << 4;
        if let Some(payload) = s.cells.remove(&key) {
            for child in 1..=8 {
                let value = match payload {
                    Payload::Fv(u) => Payload::Fv(u.map(|v| v / 8.)),
                    Payload::Lbm(f) => Payload::Lbm(super::lbm::rescale(&s.config, f, 0, 1)),
                };
                s.cells.insert(key | child, value);
            }
            s.refinements += 1;
        }
    }
    Ok(())
}
pub fn coarsen(s: &mut State, roots: &[usize]) -> Result<(), MorphError> {
    for &root in roots {
        require(root < s.config.count(), "H003 coarsening root")?;
        let key = (root as u64) << 4;
        if s.cells.contains_key(&key) {
            continue;
        }
        require(
            (1..=8).all(|k| s.cells.contains_key(&(key | k))),
            "H003 incomplete children",
        )?;
        let payload = match s.config.model {
            Model::FiniteVolume => {
                let mut sum = [0.; NF];
                for child in 1..=8 {
                    if let Payload::Fv(u) = s.cells.remove(&(key | child)).unwrap() {
                        for k in 0..NF {
                            sum[k] += u[k];
                        }
                    }
                }
                Payload::Fv(sum)
            }
            Model::AdaptiveLbm => {
                let mut sum = [0.; 19];
                for child in 1..=8 {
                    if let Payload::Lbm(f) = s.cells.remove(&(key | child)).unwrap() {
                        for q in 0..19 {
                            sum[q] += f[q] / 8.;
                        }
                    }
                }
                Payload::Lbm(super::lbm::rescale(&s.config, sum, 1, 0))
            }
        };
        s.cells.insert(key, payload);
        s.quiet.remove(&root);
        s.coarsenings += 1;
    }
    Ok(())
}
pub fn scores(s: &State, faces: &[Face]) -> BTreeMap<u64, f64> {
    let Some(ind) = &s.config.indicators else {
        return BTreeMap::new();
    };
    let mut scores = s
        .cells
        .keys()
        .map(|&key| (key, 0f64))
        .collect::<BTreeMap<_, _>>();
    for face in faces {
        if let (Some(a), Some(b)) = (face.left, face.right) {
            let (ra, ua, sa) = s.primitive(a);
            let (rb, ub, sb) = s.primitive(b);
            let density = (ra - rb).abs() / s.config.density / ind.density;
            let velocity = norm(sub(ua, ub)) / ind.velocity;
            let scalar = (0..s.config.species.len())
                .map(|k| (sa[k] - sb[k]).abs() / ind.scalar)
                .fold(0f64, f64::max);
            let value = density.max(velocity).max(scalar);
            for key in [a, b] {
                *scores.get_mut(&key).unwrap() = scores[&key].max(value);
            }
        }
    }
    for (&key, p) in &s.cells {
        if let Payload::Lbm(f) = p {
            let (rho, u) = moments(f);
            let eq = equilibrium(rho, u);
            let v = f.iter().zip(eq).map(|(a, b)| (a - b).abs()).sum::<f64>()
                / rho
                / ind.nonequilibrium;
            *scores.get_mut(&key).unwrap() = scores[&key].max(v);
        }
    }
    scores
}
pub fn adapt(s: &mut State, old_faces: &[Face]) -> Result<(), MorphError> {
    let Some(ind) = s.config.indicators.clone() else {
        return Ok(());
    };
    let score = scores(s, old_faces);
    let mut up = Vec::new();
    let mut down = Vec::new();
    for root in 0..s.config.count() {
        let parent = (root as u64) << 4;
        if s.cells.contains_key(&parent) {
            if score[&parent] >= 1. {
                up.push(root);
            }
        } else {
            let low = (1..=8).all(|k| score[&(parent | k)] < ind.coarsen_fraction);
            let quiet = s.quiet.entry(root).or_default();
            if low {
                *quiet += 1;
            } else {
                *quiet = 0;
            }
            if *quiet >= ind.quiet_cycles {
                down.push(root);
            }
        }
    }
    coarsen(s, &down)?;
    refine(s, &up)?;
    Ok(())
}
