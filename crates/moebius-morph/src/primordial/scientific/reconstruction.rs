//! Explicit primitive minmod reconstruction; no positivity clipping.
use super::*;
pub fn sinc(x: f64) -> f64 {
    if x == 0. { 1. } else { x.sin() / x }
}
fn primitive(s: &State, key: u64) -> U {
    let (r, u, c) = s.primitive(key);
    [r, u[0], u[1], u[2], c[0], c[1], c[2], c[3]]
}
pub fn minmod(a: f64, b: f64) -> f64 {
    if (a > 0. && b > 0.) || (a < 0. && b < 0.) {
        a.signum() * a.abs().min(b.abs())
    } else {
        0.
    }
}
pub fn slopes(s: &State, faces: &[mesh::Face]) -> BTreeMap<u64, [U; 3]> {
    let mut sums = s
        .cells
        .keys()
        .map(|&key| (key, [[[0.; NF]; 2]; 3]))
        .collect::<BTreeMap<_, _>>();
    let mut areas = s
        .cells
        .keys()
        .map(|&key| (key, [[0.; 2]; 3]))
        .collect::<BTreeMap<_, _>>();
    for f in faces {
        if let (Some(a), Some(b)) = (f.left, f.right) {
            let l = primitive(s, a);
            let r = primitive(s, b);
            for (key, side) in [(a, 1), (b, 0)] {
                for k in 0..NF {
                    sums.get_mut(&key).unwrap()[f.axis][side][k] +=
                        f.area * (r[k] - l[k]) / f.distance;
                }
                areas.get_mut(&key).unwrap()[f.axis][side] += f.area;
            }
        }
    }
    sums.into_iter()
        .map(|(key, g)| {
            let a = areas[&key];
            (
                key,
                std::array::from_fn(|axis| {
                    std::array::from_fn(|k| {
                        if a[axis][0] > 0. && a[axis][1] > 0. {
                            minmod(g[axis][0][k] / a[axis][0], g[axis][1][k] / a[axis][1])
                        } else {
                            0.
                        }
                    })
                }),
            )
        })
        .collect()
}
pub(super) fn at_face(
    s: &State,
    grad: &BTreeMap<u64, [U; 3]>,
    key: u64,
    other: u64,
    axis: usize,
    left: bool,
) -> Result<(f64, V, [f64; 4]), MorphError> {
    let g = mesh::geometry(&s.config, key);
    let n = mesh::geometry(&s.config, other);
    let centre = g.centre();
    let neighbour = n.centre();
    let mut delta = [0.; 3];
    delta[axis] = if left { g.h * 0.5 } else { -g.h * 0.5 };
    if n.h < g.h {
        for k in 0..3 {
            if k != axis {
                delta[k] = neighbour[k] - centre[k];
            }
        }
    }
    let mut w = primitive(s, key);
    for k in 0..NF {
        for a in 0..3 {
            w[k] += grad[&key][a][k] * delta[a];
        }
    }
    require(
        w.iter().all(|x| x.is_finite()) && w[0] > 0. && w[4..].iter().all(|x| *x >= 0.),
        "H004 invalid reconstructed density/scalar",
    )?;
    let u = [w[1], w[2], w[3]];
    require(
        norm(u) / s.config.sound_speed <= 0.1,
        "H004 reconstructed Mach envelope",
    )?;
    Ok((w[0], u, [w[4], w[5], w[6], w[7]]))
}
