//! Conservative scalar reference and active-face storage; same face ordering.
use super::*;
fn neighbour(c: &Config, i: usize, axis: usize, offset: i32) -> Option<usize> {
    let mut p = c.coordinates(i);
    let at = p[axis] as i32 + offset;
    if !(0..c.cells as i32).contains(&at) && c.boundary == Boundary::Sealed {
        return None;
    }
    p[axis] = at.rem_euclid(c.cells as i32) as usize;
    Some(c.index(p))
}
pub(super) fn step(state: &mut State) -> Result<usize, MorphError> {
    if state.fields.is_empty() {
        return Ok(0);
    }
    let config = &state.config;
    let dt_h = config.dt() / config.h();
    let velocities = (0..config.count())
        .map(|i| state.sample_index(i).velocity_m_s)
        .collect::<Vec<_>>();
    let mut count = 0;
    for field in &mut state.fields {
        let slip = field.species.slip(config);
        let diffusion = field.species.diffusivity_m2_s / config.h();
        let faces = match &field.mass {
            Mass::Dense(_) => (0..config.count())
                .flat_map(|i| (0..3).map(move |a| (i, a)))
                .collect::<Vec<_>>(),
            Mass::Sparse(values) => {
                let mut active = BTreeSet::new();
                for &i in values.keys() {
                    for axis in 0..3 {
                        active.insert((i, axis));
                        if let Some(j) = neighbour(config, i, axis, -1) {
                            active.insert((j, axis));
                        }
                    }
                }
                active.into_iter().collect()
            }
        };
        let fresh = || match &field.mass {
            Mass::Dense(_) => Mass::Dense(vec![0.; config.count()]),
            Mass::Sparse(_) => Mass::Sparse(BTreeMap::new()),
        };
        let mut delta = fresh();
        let mut outgoing = fresh();
        for (i, axis) in faces {
            if let Some(j) = neighbour(config, i, axis, 1) {
                count += 1;
                let speed = 0.5 * (velocities[i][axis] + velocities[j][axis]) + slip[axis];
                let a = field.mass.get(i);
                let b = field.mass.get(j);
                let flux = dt_h * (speed.max(0.) * a + speed.min(0.) * b + diffusion * (a - b));
                if flux != 0. {
                    delta.accumulate(i, -flux);
                    delta.accumulate(j, flux);
                }
                if a > 0. {
                    outgoing.accumulate(i, dt_h * (speed.max(0.) + diffusion));
                }
                if b > 0. {
                    outgoing.accumulate(j, dt_h * ((-speed).max(0.) + diffusion));
                }
            }
        }
        if config.boundary == Boundary::Sealed {
            for (i, mass) in field.mass.nonzero() {
                let p = config.coordinates(i);
                for axis in 0..3 {
                    for side in 0..2 {
                        if (side == 0 && p[axis] == 0) || (side == 1 && p[axis] + 1 == config.cells)
                        {
                            let outward = (slip[axis] * if side == 0 { -1. } else { 1. }).max(0.);
                            let fraction = dt_h * outward;
                            let amount = fraction * mass;
                            outgoing.accumulate(i, fraction);
                            if amount != 0. {
                                delta.accumulate(i, -amount);
                                field.wall_kg[axis * 2 + side] += amount;
                            }
                        }
                    }
                }
            }
        }
        require(
            outgoing.coefficients_valid(),
            "H001 scalar positivity CFL exceeded",
        )?;
        match &mut field.mass {
            Mass::Dense(values) => {
                for (i, d) in delta.nonzero() {
                    values[i] += d;
                }
            }
            Mass::Sparse(values) => {
                for (i, d) in delta.nonzero() {
                    let next = values.get(&i).copied().unwrap_or(0.) + d;
                    if next == 0. {
                        values.remove(&i);
                    } else {
                        values.insert(i, next);
                    }
                }
            }
        }
    }
    Ok(count)
}
