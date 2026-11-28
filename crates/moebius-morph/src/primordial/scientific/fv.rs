//! Independently authored isothermal FV face law; integrated SI inventories.
use super::*;
fn slip(c: &Config, s: &Species) -> V {
    mul(
        c.gravity,
        2. * s.radius_m.powi(2) * (s.density_kg_m3 - c.density) / (9. * c.viscosity),
    )
}
pub(super) fn fluxes(s: &State, faces: &[mesh::Face], dt: f64) -> Result<Vec<U>, MorphError> {
    fluxes_coarse(s, faces, dt, 1.)
}
fn fluxes_coarse(
    s: &State,
    faces: &[mesh::Face],
    dt: f64,
    coarse_factor: f64,
) -> Result<Vec<U>, MorphError> {
    let c = &s.config;
    require(
        faces.iter().all(|f| {
            f.left.is_none_or(|k| s.cells.contains_key(&k))
                && f.right.is_none_or(|k| s.cells.contains_key(&k))
        }),
        "H004 missing cached face owner",
    )?;
    let gradients = if c.reconstruction == Reconstruction::Minmod {
        Some(reconstruction::slopes(s, faces))
    } else {
        None
    };
    let mut rates: BTreeMap<u64, f64> = s.cells.keys().map(|&k| (k, 0.)).collect();
    let mut out = Vec::with_capacity(faces.len());
    for f in faces {
        let key = f.left.or(f.right).unwrap();
        let (rho, u, sc) = s.primitive(key);
        let mut l = f.left.map(|k| s.primitive(k)).unwrap_or((rho, u, sc));
        let mut r = f.right.map(|k| s.primitive(k)).unwrap_or((rho, u, sc));
        if f.left.is_none() {
            l.1[f.axis] = -r.1[f.axis];
        }
        if f.right.is_none() {
            r.1[f.axis] = -l.1[f.axis];
        }
        let original_l = l;
        let original_r = r;
        if let Some(grad) = &gradients
            && let (Some(a), Some(b)) = (f.left, f.right)
        {
            l = reconstruction::at_face(s, grad, a, b, f.axis, true)?;
            r = reconstruction::at_face(s, grad, b, a, f.axis, false)?;
        }
        let a = f.axis;
        let lam = l.1[a].abs().max(r.1[a].abs()) + c.sound_speed;
        let mut q = [0.; NF];
        q[0] = 0.5 * (l.0 * l.1[a] + r.0 * r.1[a]) - 0.5 * lam * (r.0 - l.0);
        for k in 0..3 {
            q[k + 1] = if k == a {
                0.5 * (l.0 * l.1[a].powi(2)
                    + c.sound_speed.powi(2) * l.0
                    + r.0 * r.1[a].powi(2)
                    + c.sound_speed.powi(2) * r.0)
                    - 0.5 * lam * (r.0 * r.1[a] - l.0 * l.1[a])
            } else {
                q[0] * if q[0] >= 0. { l.1[k] } else { r.1[k] }
            };
            q[k + 1] -= c.viscosity * (original_r.1[k] - original_l.1[k]) / f.distance;
        }
        let mut diffusion = c.viscosity / original_l.0.min(original_r.0);
        let mut speed = l.1[a].abs().max(r.1[a].abs());
        for (k, sp) in c.species.iter().enumerate() {
            let v = 0.5 * (l.1[a] + r.1[a]) + slip(c, sp)[a];
            speed = speed.max(v.abs());
            diffusion = diffusion.max(sp.diffusivity_m2_s);
            q[k + 4] = v * if v >= 0. { l.2[k] } else { r.2[k] };
            if f.left.is_some() && f.right.is_some() {
                q[k + 4] -= sp.diffusivity_m2_s * (original_r.2[k] - original_l.2[k]) / f.distance;
            } else {
                // absorbing outward terminal slip, no fictitious wall source
                q[k + 4] = if f.left.is_none() {
                    v.min(0.) * r.2[k]
                } else {
                    v.max(0.) * l.2[k]
                };
            }
        }
        for key in [f.left, f.right].into_iter().flatten() {
            *rates.get_mut(&key).unwrap() +=
                if key & 15 == 0 { coarse_factor } else { 1. } * dt * f.area
                    / mesh::geometry(c, key).volume()
                    * (speed + c.sound_speed + 2. * diffusion / f.distance);
        }
        out.push(q.map(|x| x * f.area * dt));
    }
    require(
        rates.values().all(|r| r.is_finite() && *r <= 0.9),
        "H003 FV CFL refused",
    )?;
    Ok(out)
}
fn interface(f: &mesh::Face) -> bool {
    matches!((f.left,f.right),(Some(a),Some(b)) if (a&15==0)!=(b&15==0))
}
fn apply(s: &mut State, faces: &[mesh::Face], flux: &[U], level: Option<u8>, skip_interface: bool) {
    for (face, q) in faces.iter().zip(flux) {
        if skip_interface && interface(face) {
            continue;
        }
        for (key, sign) in [(face.left, -1.), (face.right, 1.)] {
            if let Some(key) = key
                && level.is_none_or(|l| mesh::geometry(&s.config, key).level == l)
                && let Payload::Fv(u) = s.cells.get_mut(&key).unwrap()
            {
                for k in 0..NF {
                    u[k] += sign * q[k];
                }
            }
        }
        if face.left.is_none() || face.right.is_none() {
            let key = face.left.or(face.right).unwrap();
            if level.is_none_or(|l| mesh::geometry(&s.config, key).level == l) {
                let sign = if face.left.is_none() { 1. } else { -1. };
                for k in 0..3 {
                    s.ledger.walls_impulse[k] += sign * q[k + 1];
                }
                let side = 2 * face.axis + usize::from(face.right.is_none());
                for k in 0..s.config.species.len() {
                    s.ledger.wall_scalar[k][side] -= sign * q[k + 4];
                }
            }
        }
    }
}
fn gravity(s: &mut State, dt: f64, level: Option<u8>) {
    for (&key, p) in &mut s.cells {
        if level.is_none_or(|l| mesh::geometry(&s.config, key).level == l)
            && let Payload::Fv(u) = p
        {
            for k in 0..3 {
                let j = u[0] * s.config.gravity[k] * dt;
                u[k + 1] += j;
                s.ledger.gravity_impulse[k] += j;
            }
        }
    }
}
fn euler(s: &mut State, faces: &[mesh::Face], dt: f64, time: f64) -> Result<(), MorphError> {
    let f = fluxes(s, faces, dt)?;
    apply(s, faces, &f, None, false);
    gravity(s, dt, None);
    manufactured(s, dt, time);
    Ok(())
}
pub(super) fn advance(s: &mut State, faces: &[mesh::Face], dt: f64) -> Result<(), MorphError> {
    let time = s.cycle as f64 * s.config.dt;
    if s.config.integrator == Integrator::Euler {
        return euler(s, faces, dt, time);
    }
    let old = s.clone();
    euler(s, faces, dt, time)?;
    s.validate()?;
    euler(s, faces, dt, time + dt)?;
    s.validate()?;
    for (&key, p) in &mut s.cells {
        let Payload::Fv(u) = p else { unreachable!() };
        let Payload::Fv(v) = old.cells[&key] else {
            unreachable!()
        };
        for k in 0..NF {
            u[k] = 0.5 * (u[k] + v[k]);
        }
    }
    for k in 0..3 {
        s.ledger.gravity_impulse[k] =
            0.5 * (s.ledger.gravity_impulse[k] + old.ledger.gravity_impulse[k]);
        s.ledger.walls_impulse[k] = 0.5 * (s.ledger.walls_impulse[k] + old.ledger.walls_impulse[k]);
        s.ledger.manufactured_impulse[k] =
            0.5 * (s.ledger.manufactured_impulse[k] + old.ledger.manufactured_impulse[k]);
    }
    for k in 0..4 {
        for side in 0..6 {
            s.ledger.wall_scalar[k][side] =
                0.5 * (s.ledger.wall_scalar[k][side] + old.ledger.wall_scalar[k][side]);
        }
    }
    Ok(())
}
fn manufactured(s: &mut State, dt: f64, time: f64) {
    let Some(f) = &s.config.vortex_forcing else {
        return;
    };
    let k = f.wavenumber;
    let amplitude =
        f.amplitude * (-3. * s.config.viscosity / s.config.density * k * k * time).exp();
    for (&key, p) in &mut s.cells {
        let g = mesh::geometry(&s.config, key);
        let at = g.centre();
        let zfactor = 0.5 * (1. + (2. * k * at[2]).cos() * reconstruction::sinc(k * g.h));
        if let Payload::Fv(u) = p {
            for a in 0..2 {
                let force = s.config.density
                    * amplitude.powi(2)
                    * k
                    * 0.5
                    * (2. * k * at[a]).sin()
                    * reconstruction::sinc(k * g.h)
                    * zfactor;
                let j = force * g.volume() * dt;
                u[a + 1] += j;
                s.ledger.manufactured_impulse[a] += j;
            }
        }
    }
}
fn accumulate(register: &mut BTreeMap<u64, U>, faces: &[mesh::Face], flux: &[U]) {
    for (f, q) in faces.iter().zip(flux) {
        if interface(f) {
            for (key, sign) in [(f.left.unwrap(), -1.), (f.right.unwrap(), 1.)] {
                if key & 15 == 0 {
                    let u = register.entry(key).or_insert([0.; NF]);
                    for k in 0..NF {
                        u[k] += sign * q[k];
                    }
                }
            }
        }
    }
}
pub(super) fn begin(base: &State) -> Result<Pending, MorphError> {
    require(
        base.config.model == Model::FiniteVolume
            && base.bodies.is_empty()
            && base.config.reconstruction == Reconstruction::Constant
            && base.config.integrator == Integrator::Euler
            && base.config.vortex_forcing.is_none(),
        "H003 FV subcycle envelope",
    )?;
    let faces = mesh::faces(base)?;
    // Coarse predictor admits only the coarse-cell CFL; fine cells use half dt.
    let half = fluxes_coarse(base, &faces, base.config.dt * 0.5, 2.)?;
    let full = half.iter().map(|q| q.map(|v| 2. * v)).collect::<Vec<_>>();
    let mut predicted = base.clone();
    apply(&mut predicted, &faces, &full, Some(0), false);
    gravity(&mut predicted, base.config.dt, Some(0));
    let predictor = predicted
        .cells
        .iter()
        .filter_map(|(&key, p)| match p {
            Payload::Fv(u) if key & 15 == 0 => Some((key, *u)),
            _ => None,
        })
        .collect();
    let mut work = base.clone();
    apply(&mut work, &faces, &half, Some(1), false);
    gravity(&mut work, base.config.dt * 0.5, Some(1));
    let mut register = BTreeMap::new();
    accumulate(&mut register, &faces, &half);
    Ok(Pending {
        work,
        predictor,
        stage: 1,
        interface: register,
    })
}
pub(super) fn validate_pending(base: &State, p: &Pending) -> Result<(), MorphError> {
    let expected = begin(base)?;
    require(
        serde_json::to_vec(&encode_bits(
            serde_json::to_value(&expected).map_err(error)?,
        ))
        .map_err(error)?
            == serde_json::to_vec(&encode_bits(serde_json::to_value(p).map_err(error)?))
                .map_err(error)?,
        "H003 invalid pending causal work",
    )
}
pub(super) fn finish(base: &State, p: &Pending) -> Result<State, MorphError> {
    validate_pending(base, p)?;
    let faces = mesh::faces(base)?;
    let dt = base.config.dt;
    let mut ghosts = p.work.clone();
    for (&key, pred) in &p.predictor {
        let Payload::Fv(old) = base.cells[&key] else {
            unreachable!()
        };
        ghosts.cells.insert(
            key,
            Payload::Fv(std::array::from_fn(|k| 0.5 * (old[k] + pred[k]))),
        );
    }
    let half2 = fluxes(&ghosts, &faces, dt * 0.5)?;
    let half0 = fluxes(base, &faces, dt * 0.5)?;
    let full = half0.iter().map(|q| q.map(|v| 2. * v)).collect::<Vec<_>>();
    let mut next = p.work.clone();
    apply(&mut next, &faces, &half2, Some(1), false);
    gravity(&mut next, dt * 0.5, Some(1));
    apply(&mut next, &faces, &full, Some(0), true);
    gravity(&mut next, dt, Some(0));
    let mut register = p.interface.clone();
    accumulate(&mut register, &faces, &half2);
    for (key, delta) in register {
        if let Payload::Fv(u) = next.cells.get_mut(&key).unwrap() {
            for k in 0..NF {
                u[k] += delta[k];
            }
        }
    }
    next.cycle += 1;
    Ok(next)
}
