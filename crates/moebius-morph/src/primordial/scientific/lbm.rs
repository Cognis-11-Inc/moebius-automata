//! Exploratory acoustic 2:1 interface. No conservative repair; qualification required.
use super::*;
pub(super) fn rescale(c: &Config, f: [f64; 19], from: u8, to: u8) -> [f64; 19] {
    let (rho, u) = moments(&f);
    let eq = equilibrium(rho, u);
    let factor = c.tau(to) / c.tau(from) * 2f64.powi(from as i32 - to as i32);
    std::array::from_fn(|q| eq[q] + factor * (f[q] - eq[q]))
}
fn average_root(s: &State, root: u64) -> [f64; 19] {
    let mut f = [0.; 19];
    for id in 1..=8 {
        let Payload::Lbm(p) = s.cells[&(root | id)] else {
            unreachable!()
        };
        for q in 0..19 {
            f[q] += p[q] / 8.;
        }
    }
    rescale(&s.config, f, 1, 0)
}
fn sweep(
    s: &State,
    level: u8,
    coarse: Option<&BTreeMap<u64, Payload>>,
) -> Result<(BTreeMap<u64, Payload>, V), MorphError> {
    let c = &s.config;
    let mut result = BTreeMap::new();
    let mut wall = [0.; 3];
    for (&key, payload) in &s.cells {
        if mesh::geometry(c, key).level != level {
            continue;
        }
        let Payload::Lbm(old) = payload else {
            unreachable!()
        };
        let g = mesh::geometry(c, key);
        let centre = g.centre();
        let mut f = [0.; 19];
        for q in 0..19 {
            let mut at = std::array::from_fn(|k| centre[k] - C[q][k] as f64 * g.h);
            let outside = (0..3).any(|k| at[k] < 0. || at[k] >= c.lengths()[k]);
            let (incoming, source_level, component) = if outside && !c.periodic {
                (*old, level, OPP[q])
            } else {
                for (k, v) in at.iter_mut().enumerate() {
                    *v = v.rem_euclid(c.lengths()[k]);
                }
                let source = mesh::locate(s, at)?;
                let source_level = mesh::geometry(c, source).level;
                let data = if level == 0 && source_level == 1 {
                    average_root(s, source & !15)
                } else {
                    let source_map = if source_level == 0 {
                        coarse.unwrap_or(&s.cells)
                    } else {
                        &s.cells
                    };
                    let Payload::Lbm(v) = source_map[&source] else {
                        unreachable!()
                    };
                    rescale(c, v, source_level, level)
                };
                (data, level, q)
            };
            let (rho, u) = moments(&incoming);
            let eq = equilibrium(rho, u);
            let post =
                incoming[component] + (eq[component] - incoming[component]) / c.tau(source_level);
            require(
                post.is_finite() && post >= 0.,
                "H003 unstable LBM collision",
            )?;
            f[q] = post;
            if outside && !c.periodic {
                for k in 0..3 {
                    wall[k] += 2. * post * C[q][k] as f64 * g.volume() * c.h / c.dt;
                }
            }
        }
        result.insert(key, Payload::Lbm(f));
    }
    Ok((result, wall))
}
pub(super) fn advance(s: &mut State) -> Result<(), MorphError> {
    let (predicted, wall_c) = sweep(s, 0, None)?;
    let (fine1, wall_f1) = sweep(s, 1, None)?;
    let mut half = s.clone();
    half.cells.extend(fine1);
    let mut temporal = BTreeMap::new();
    for (&key, p) in &predicted {
        let Payload::Lbm(a) = s.cells[&key] else {
            unreachable!()
        };
        let Payload::Lbm(b) = p else { unreachable!() };
        temporal.insert(
            key,
            Payload::Lbm(std::array::from_fn(|q| 0.5 * (a[q] + b[q]))),
        );
    }
    let (fine2, wall_f2) = sweep(&half, 1, Some(&temporal))?;
    s.cells.extend(predicted);
    s.cells.extend(fine2);
    for k in 0..3 {
        s.ledger.walls_impulse[k] += wall_c[k] + wall_f1[k] + wall_f2[k];
    }
    Ok(())
}
