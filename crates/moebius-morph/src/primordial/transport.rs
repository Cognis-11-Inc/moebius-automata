//! Conservative scalar reference and active-face storage; same face ordering.
use super::*;
pub(super) fn neighbour(c: &Config, i: usize, axis: usize, offset: i32) -> Option<usize> {
    let mut p = c.coordinates(i);
    let at = p[axis] as i32 + offset;
    if !(0..c.cells as i32).contains(&at) && c.boundary == Boundary::Sealed {
        return None;
    }
    p[axis] = at.rem_euclid(c.cells as i32) as usize;
    Some(c.index(p))
}
#[derive(Default, Debug, Serialize)]
pub struct Attribution {
    pub leaf_ns: BTreeMap<String, u128>,
    pub calls: BTreeMap<String, usize>,
    pub phases: Vec<serde_json::Value>,
}
fn timed<T>(a: &mut Option<&mut Attribution>, label: &str, f: impl FnOnce() -> T) -> T {
    if let Some(a) = a {
        let t = Instant::now();
        let v = f();
        *a.leaf_ns.entry(label.into()).or_default() += t.elapsed().as_nanos();
        *a.calls.entry(label.into()).or_default() += 1;
        v
    } else {
        f()
    }
}
fn phase(a: &mut Option<&mut Attribution>, label: &str, t: Option<Instant>, before: u128) {
    if let Some(a) = a {
        let gross = t.unwrap().elapsed().as_nanos();
        let leaves = a.leaf_ns.values().sum::<u128>() - before;
        a.phases.push(serde_json::json!({"name":label,"gross_ns":gross,"leaf_ns":leaves,"exclusive_remainder_ns":gross.saturating_sub(leaves),"interpretation":"remainder includes instrumentation gaps; do not add gross phases to their nested leaves"}));
    }
}
fn mark(a: &Option<&mut Attribution>) -> (Option<Instant>, u128) {
    (
        a.as_ref().map(|_| Instant::now()),
        a.as_ref().map_or(0, |a| a.leaf_ns.values().sum()),
    )
}
pub(super) fn step(state: &mut State) -> Result<usize, MorphError> {
    step_inner(state, None)
}
pub(super) fn attributed(state: &mut State, a: &mut Attribution) -> Result<usize, MorphError> {
    step_inner(state, Some(a))
}
fn step_inner(state: &mut State, mut attr: Option<&mut Attribution>) -> Result<usize, MorphError> {
    if state.fields.is_empty() {
        return Ok(0);
    }
    let config = &state.config;
    let dt_h = config.dt() / config.h();
    let (timer, before) = mark(&attr);
    let velocities = (0..config.count())
        .map(|i| state.sample_index(i).velocity_m_s)
        .collect::<Vec<_>>();
    phase(&mut attr, "carrier_velocity_extraction", timer, before);
    let mut count = 0;
    for field in &mut state.fields {
        let slip = field.species.slip(config);
        let diffusion = field.species.diffusivity_m2_s / config.h();
        let (timer, before) = mark(&attr);
        let faces = match &field.mass {
            Mass::Dense(_) => (0..config.count())
                .flat_map(|i| (0..3).map(move |a| (i, a)))
                .collect::<Vec<_>>(),
            Mass::Blocks(values) => {
                let mut active = BTreeSet::new();
                for (i, _) in values.nonzero() {
                    for axis in 0..3 {
                        active.insert((i, axis));
                        if let Some(j) = neighbour(config, i, axis, -1) {
                            active.insert((j, axis));
                        }
                    }
                }
                active.into_iter().collect()
            }
            Mass::Sparse(values) => {
                let mut active = BTreeSet::new();
                for &i in values.keys() {
                    for axis in 0..3 {
                        timed(&mut attr, "ordered_set_insert", || active.insert((i, axis)));
                        if let Some(j) = timed(&mut attr, "neighbour_calculation", || {
                            neighbour(config, i, axis, -1)
                        }) {
                            timed(&mut attr, "ordered_set_insert", || active.insert((j, axis)));
                        }
                    }
                }
                active.into_iter().collect()
            }
        };
        phase(&mut attr, "face_discovery_and_list", timer, before);
        let (timer, before) = mark(&attr);
        let fresh = || match &field.mass {
            Mass::Dense(_) => Mass::Dense(vec![0.; config.count()]),
            Mass::Sparse(_) => Mass::Sparse(BTreeMap::new()),
            Mass::Blocks(v) => Mass::Blocks(BlockMass::new(v.cells, v.edge)),
        };
        let mut delta = fresh();
        let mut outgoing = fresh();
        phase(
            &mut attr,
            "temporary_delta_and_outgoing_allocation",
            timer,
            before,
        );
        let (timer, before) = mark(&attr);
        for (i, axis) in faces {
            if let Some(j) = timed(&mut attr, "neighbour_calculation", || {
                neighbour(config, i, axis, 1)
            }) {
                count += 1;
                let speed = 0.5 * (velocities[i][axis] + velocities[j][axis]) + slip[axis];
                let a = timed(&mut attr, "cell_lookup", || field.mass.get(i));
                let b = timed(&mut attr, "cell_lookup", || field.mass.get(j));
                let flux = dt_h * (speed.max(0.) * a + speed.min(0.) * b + diffusion * (a - b));
                if flux != 0. {
                    timed(&mut attr, "delta_accumulation_and_entry_insertion", || {
                        delta.accumulate(i, -flux)
                    });
                    timed(&mut attr, "delta_accumulation_and_entry_insertion", || {
                        delta.accumulate(j, flux)
                    });
                }
                if a > 0. {
                    timed(
                        &mut attr,
                        "outgoing_accumulation_and_entry_insertion",
                        || outgoing.accumulate(i, dt_h * (speed.max(0.) + diffusion)),
                    );
                }
                if b > 0. {
                    timed(
                        &mut attr,
                        "outgoing_accumulation_and_entry_insertion",
                        || outgoing.accumulate(j, dt_h * ((-speed).max(0.) + diffusion)),
                    );
                }
            }
        }
        phase(&mut attr, "face_flux", timer, before);
        let (timer, before) = mark(&attr);
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
                            timed(
                                &mut attr,
                                "outgoing_accumulation_and_entry_insertion",
                                || outgoing.accumulate(i, fraction),
                            );
                            if amount != 0. {
                                timed(&mut attr, "delta_accumulation_and_entry_insertion", || {
                                    delta.accumulate(i, -amount)
                                });
                                field.wall_kg[axis * 2 + side] += amount;
                            }
                        }
                    }
                }
            }
        }
        phase(&mut attr, "wall_exchange", timer, before);
        let (timer, before) = mark(&attr);
        require(
            outgoing.coefficients_valid(),
            "H001 scalar positivity CFL exceeded",
        )?;
        phase(&mut attr, "outgoing_CFL_validation", timer, before);
        let (timer, before) = mark(&attr);
        match &mut field.mass {
            Mass::Dense(values) => {
                for (i, d) in delta.nonzero() {
                    values[i] += d;
                }
            }
            Mass::Blocks(values) => {
                for (i, d) in delta.nonzero() {
                    values.add(i, d);
                }
                values.retire_zero();
            }
            Mass::Sparse(values) => {
                for (i, d) in delta.nonzero() {
                    let next = values.get(&i).copied().unwrap_or(0.) + d;
                    if next == 0. {
                        timed(&mut attr, "entry_removal", || values.remove(&i));
                    } else {
                        timed(&mut attr, "entry_insertion", || values.insert(i, next));
                    }
                }
            }
        }
        phase(&mut attr, "material_apply", timer, before);
    }
    Ok(count)
}
