//! HABITAT-004 controls. Analytic initial averages are evaluated on actual leaves.
use super::super::*;
use std::f64::consts::PI;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Control {
    Equilibrium,
    Diffusion3,
    Diagonal,
    Shear3,
    Oblique,
    Vortex,
    Hydrostatic,
    Steep,
    Packet {
        direction: f64,
        centre: f64,
        width: f64,
        amplitude: f64,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spec {
    pub config: Config,
    pub control: Control,
    pub cycles: usize,
    pub refined: Option<[V; 2]>,
    pub fine: bool,
    pub cache: bool,
    pub subcycle: bool,
}
impl Spec {
    pub fn validate(&self) -> Result<(), MorphError> {
        self.config.validate()?;
        require(
            self.config.model == Model::FiniteVolume
                && self.cycles > 0
                && self.cycles <= 20000
                && !(self.fine && self.refined.is_some()),
            "H004 controlled case envelope",
        )?;
        if matches!(self.control, Control::Hydrostatic) {
            require(
                !self.config.periodic
                    && self.config.gravity[0] == 0.
                    && self.config.gravity[2] == 0.,
                "H004 hydrostatic reference boundary/source mismatch",
            )?;
        } else {
            require(
                self.config.gravity == [0.; 3],
                "H004 unforced reference with gravity",
            )?;
        }
        if matches!(self.control, Control::Steep) {
            require(
                self.config
                    .species
                    .first()
                    .is_some_and(|s| s.diffusivity_m2_s == 0.),
                "H004 sharp advection reference requires zero diffusivity",
            )?;
        }
        if matches!(self.control, Control::Oblique) {
            require(
                self.config.viscosity / self.config.density * (2. * PI * 3f64.sqrt())
                    < self.config.sound_speed,
                "H004 linear reference underdamped regime",
            )?;
        }
        if let Some([lo, hi]) = self.refined {
            require(
                (0..3).all(|k| {
                    lo[k].is_finite()
                        && hi[k].is_finite()
                        && lo[k] >= 0.
                        && hi[k] <= self.config.lengths()[k]
                        && lo[k] < hi[k]
                }),
                "H004 physical refinement box",
            )?;
        }
        if matches!(
            self.control,
            Control::Diffusion3 | Control::Diagonal | Control::Steep
        ) {
            require(
                !self.config.species.is_empty(),
                "H004 scalar control requires species",
            )?;
        }
        if matches!(
            self.control,
            Control::Diffusion3
                | Control::Diagonal
                | Control::Shear3
                | Control::Oblique
                | Control::Vortex
        ) {
            require(
                self.config.periodic && self.config.lengths() == [1.; 3],
                "H004 fixed unit-box analytic control",
            )?;
        }
        if matches!(self.control, Control::Vortex) {
            let f = self
                .config
                .vortex_forcing
                .as_ref()
                .ok_or_else(|| error("H004 manufactured vortex requires configured source"))?;
            require(
                f.amplitude == 0.005 && f.wavenumber == 2. * PI,
                "H004 manufactured source/reference mismatch",
            )?;
        } else {
            require(
                self.config.vortex_forcing.is_none(),
                "H004 source on unforced control",
            )?;
        }
        if let Control::Packet {
            direction,
            centre,
            width,
            amplitude,
        } = self.control
        {
            require(
                self.config.periodic
                    && [-1., 1.].contains(&direction)
                    && centre.is_finite()
                    && (0.1..=0.9).contains(&centre)
                    && width.is_finite()
                    && (0.02..=0.2).contains(&width)
                    && amplitude.is_finite()
                    && (1e-6..=1e-3).contains(&amplitude),
                "H004 packet envelope",
            )?;
        }
        if self.subcycle {
            require(
                !self.cache
                    && self.config.reconstruction == Reconstruction::Constant
                    && self.config.integrator == Integrator::Euler
                    && self.config.vortex_forcing.is_none(),
                "H004 higher-order/cached subcycle unqualified",
            )?;
        }
        Ok(())
    }
}
const G4: [(f64, f64); 4] = [
    (-0.8611363115940526, 0.3478548451374538),
    (-0.3399810435848563, 0.6521451548625461),
    (0.3399810435848563, 0.6521451548625461),
    (0.8611363115940526, 0.3478548451374538),
];
const G8: [(f64, f64); 8] = [
    (-0.9602898564975363, 0.1012285362903763),
    (-0.7966664774136267, 0.2223810344533745),
    (-0.525532409916329, 0.3137066458778873),
    (-0.1834346424956498, 0.362683783378362),
    (0.1834346424956498, 0.362683783378362),
    (0.525532409916329, 0.3137066458778873),
    (0.7966664774136267, 0.2223810344533745),
    (0.9602898564975363, 0.1012285362903763),
];
fn gaussian(g: mesh::Geometry, centre: f64, width: f64, power: f64, order: usize) -> f64 {
    let nodes: &[(f64, f64)] = if order == 4 { &G4 } else { &G8 };
    nodes
        .iter()
        .map(|&(x, w)| {
            0.5 * w * (-power * ((g.centre()[0] + x * g.h * 0.5 - centre) / width).powi(2)).exp()
        })
        .sum()
}
/// Exact cell-average controls except the declared linearized wave and kinematic
/// packet comparator. The latter is not an exact viscous packet evolution.
pub fn reference_average(spec: &Spec, g: mesh::Geometry, time: f64) -> U {
    let c = &spec.config;
    let p = g.centre();
    let rho = c.density;
    let k = 2. * PI;
    let avg = reconstruction::sinc(k * g.h * 0.5);
    let nu = c.viscosity / rho;
    let mut q = [0.; NF];
    q[0] = rho;
    match spec.control {
        Control::Equilibrium => {}
        Control::Diffusion3 | Control::Diagonal => {
            let u = if matches!(spec.control, Control::Diagonal) {
                [0.03, 0.02, 0.01]
            } else {
                [0.; 3]
            };
            for a in 0..3 {
                q[a + 1] = rho * u[a];
            }
            for (i, sp) in c.species.iter().enumerate() {
                let wave = (0..3)
                    .map(|a| (k * (p[a] - u[a] * time)).cos() * avg)
                    .product::<f64>();
                q[i + 4] = 1e-4
                    * (i + 1) as f64
                    * (1. + 0.5 * wave * (-3. * sp.diffusivity_m2_s * k * k * time).exp());
            }
        }
        Control::Shear3 => {
            let a = [0.006, -0.003, -0.003];
            let mode =
                (k * p.iter().sum::<f64>()).sin() * avg.powi(3) * (-3. * nu * k * k * time).exp();
            for i in 0..3 {
                q[i + 1] = rho * a[i] * mode;
            }
        }
        Control::Oblique => {
            // Linearized viscous (delta-rho, momentum) system. Include the second
            // harmonic/mean from the prescribed nonlinear initial rho*u product.
            let phase = k * p.iter().sum::<f64>();
            let magnitude = k * 3f64.sqrt();
            let amplitude = 1e-4;
            let mut relative = 0.;
            let mut momentum = amplitude * amplitude * 0.5;
            for harmonic in [1, 2] {
                let frequency = magnitude * harmonic as f64;
                let damping = 0.5 * nu * frequency * frequency;
                let omega = ((c.sound_speed * frequency).powi(2) - damping * damping).sqrt();
                let decay = (-damping * time).exp();
                let cosine = (omega * time).cos();
                let sine = (omega * time).sin();
                let r0 = if harmonic == 1 { amplitude } else { 0. };
                let v0 = if harmonic == 1 {
                    amplitude
                } else {
                    amplitude * amplitude * 0.5
                };
                let a = harmonic as f64 * phase;
                let average = reconstruction::sinc(harmonic as f64 * k * g.h * 0.5).powi(3);
                let exchange = c.sound_speed * frequency / omega * sine;
                relative += decay
                    * ((cosine + damping / omega * sine) * r0 * a.cos() + exchange * v0 * a.sin())
                    * average;
                momentum += decay
                    * ((cosine - damping / omega * sine) * v0 * a.cos() + exchange * r0 * a.sin())
                    * average;
            }
            q[0] = rho * (1. + relative);
            for v in q.iter_mut().take(4).skip(1) {
                *v = rho * c.sound_speed / 3f64.sqrt() * momentum;
            }
        }
        Control::Vortex => {
            let a = 0.005 * (-3. * nu * k * k * time).exp() * avg.powi(3);
            q[1] = rho * a * (k * p[0]).sin() * (k * p[1]).cos() * (k * p[2]).cos();
            q[2] = -rho * a * (k * p[0]).cos() * (k * p[1]).sin() * (k * p[2]).cos();
        }
        Control::Hydrostatic => {
            let a = c.gravity[1] / c.sound_speed.powi(2);
            let z = a * g.h * 0.5;
            let factor = if z == 0. { 1. } else { z.sinh() / z };
            q[0] = rho * (a * (p[1] - c.lengths()[1] / 2.)).exp() * factor;
        }
        Control::Steep => {
            q[1] = rho * 0.03;
            let lo = 0.25 + 0.03 * time;
            let hi = 0.5 + 0.03 * time;
            let overlap = (g.lower[0] + g.h).min(hi) - g.lower[0].max(lo);
            q[4] = 1e-4 * overlap.max(0.) / g.h;
        }
        Control::Packet {
            direction,
            centre,
            width,
            amplitude,
        } => {
            let x = centre + direction * c.sound_speed * time;
            let g1 = gaussian(g, x, width, 1., 8);
            let g2 = gaussian(g, x, width, 2., 8);
            q[0] = rho * (1. + amplitude * g1);
            q[1] = rho * direction * c.sound_speed * (amplitude * g1 + amplitude.powi(2) * g2);
        }
    }
    q
}
pub fn chamber(spec: &Spec) -> Result<Chamber, MorphError> {
    spec.validate()?;
    let rho = spec.config.density;
    let mut c = Chamber::new(spec.config.clone(), |_| (rho, [0.; 3], [0.; 4]))?;
    let roots = (0..spec.config.count())
        .filter(|r| {
            if spec.fine {
                return true;
            }
            let Some([lo, hi]) = spec.refined else {
                return false;
            };
            let p = mesh::geometry(&spec.config, (*r as u64) << 4).centre();
            (0..3).all(|a| p[a] >= lo[a] && p[a] < hi[a])
        })
        .collect::<Vec<_>>();
    c.refine(&roots)?;
    for (&key, payload) in &mut c.state.cells {
        let g = mesh::geometry(&spec.config, key);
        *payload = Payload::Fv(reference_average(spec, g, 0.).map(|a| a * g.volume()));
    }
    c.state.ledger.initial = c.state.diagnostics().total;
    c.state.validate()?;
    Ok(c)
}
pub fn metrics(spec: &Spec, s: &State) -> serde_json::Value {
    let time = s.cycle as f64 * s.config.dt;
    let mut density = 0.;
    let mut scalar = 0.;
    let mut velocity = 0.;
    let mut scalar_min = f64::INFINITY;
    let mut scalar_max = 0f64;
    let mut quad = 0f64;
    let mut acoustic = 0.;
    let mut peak_plus = 0f64;
    let mut peak_minus = 0f64;
    let mut plus_at = 0.;
    let mut minus_at = 0.;
    for (&key, p) in &s.cells {
        let g = mesh::geometry(&s.config, key);
        let v = g.volume();
        let target = reference_average(spec, g, time);
        let Payload::Fv(u) = p else { unreachable!() };
        density += (u[0] / v - target[0]).powi(2) * v;
        scalar += (u[4] / v - target[4]).abs() * v;
        let vel: V = std::array::from_fn(|k| u[k + 1] / u[0]);
        let ideal: V = std::array::from_fn(|k| target[k + 1] / target[0]);
        velocity += norm(sub(vel, ideal)) * v;
        scalar_min = scalar_min.min(u[4] / v);
        scalar_max = scalar_max.max(u[4] / v);
        let drho = u[0] / v - s.config.density;
        let plus = 0.5 * (drho / s.config.density + vel[0] / s.config.sound_speed);
        let minus = 0.5 * (drho / s.config.density - vel[0] / s.config.sound_speed);
        if plus.abs() > peak_plus {
            peak_plus = plus.abs();
            plus_at = g.centre()[0];
        }
        if minus.abs() > peak_minus {
            peak_minus = minus.abs();
            minus_at = g.centre()[0];
        }
        acoustic += (s.config.sound_speed.powi(2) * drho.powi(2) / (2. * s.config.density)
            + 0.5 * s.config.density * dot(vel, vel))
            * v;
        if let Control::Packet { centre, width, .. } = spec.control {
            quad = quad
                .max((gaussian(g, centre, width, 1., 4) - gaussian(g, centre, width, 1., 8)).abs());
        }
    }
    let volume = s.config.lengths().iter().product::<f64>();
    let velocity_scale = match spec.control {
        Control::Shear3 => norm([0.006, -0.003, -0.003]),
        Control::Oblique => s.config.sound_speed * 1e-4,
        Control::Packet { amplitude, .. } => s.config.sound_speed * amplitude,
        Control::Diagonal => norm([0.03, 0.02, 0.01]),
        _ => 0.005,
    };
    let probe_x = if matches!(spec.control,Control::Packet{direction,..} if direction<0.) {
        0.35
    } else {
        0.65
    };
    let probe = s
        .sample([
            probe_x,
            s.config.lengths()[1] * 0.5,
            s.config.lengths()[2] * 0.5,
        ])
        .ok();
    serde_json::json!({"reference_kind":match spec.control{Control::Packet{..}=>"inviscid kinematic Gaussian comparator ONLY; use matched homogeneous fine for viscous scattering",Control::Oblique=>"exact linearized viscous system; full nonlinear model error remains O(amplitude squared)",_=>"analytic cell averages of declared control"},"density_L2_normalized":(density/volume).sqrt()/(s.config.density*1e-4),"scalar_L1_normalized":scalar/(5e-5*volume),"velocity_L1_normalized":velocity/(velocity_scale*volume),"velocity_normalization_m_s":velocity_scale,"scalar_min_kg_m3":scalar_min,"scalar_max_kg_m3":scalar_max,"packet_initial_quad4_vs8_normalized":quad,"acoustic_linear_energy_j":if matches!(spec.control,Control::Packet{..}|Control::Oblique){Some(acoustic)}else{None},"plus_peak":peak_plus,"minus_peak":peak_minus,"plus_peak_x":plus_at,"minus_peak_x":minus_at,"probe_density_kg_m3":probe.map(|x|x.0),"physical_time_s":time})
}
pub fn temporal_attribution(c: &Chamber) -> Result<serde_json::Value, MorphError> {
    let before = c.digest()?;
    let t = Instant::now();
    let cloned = c.state().clone();
    let clone_ns = t.elapsed().as_nanos();
    drop(cloned);
    let t = Instant::now();
    let faces = mesh::faces(c.state())?;
    let topology_ns = t.elapsed().as_nanos();
    let t = Instant::now();
    let _ = fv::fluxes(c.state(), &faces, c.state().config.dt * 0.5)?;
    let half_flux_ns = t.elapsed().as_nanos();
    let t = Instant::now();
    let p = fv::begin(c.state())?;
    let begin = t.elapsed().as_nanos();
    let t = Instant::now();
    fv::validate_pending(c.state(), &p)?;
    let validation = t.elapsed().as_nanos();
    let t = Instant::now();
    let encoded = serde_json::to_vec(&encode_bits(serde_json::to_value(&p).map_err(error)?))
        .map_err(error)?;
    let encoding = t.elapsed().as_nanos();
    let pending_bytes = encoded.len();
    let t = Instant::now();
    let next = fv::finish(c.state(), &p)?;
    let finish = t.elapsed().as_nanos();
    let t = Instant::now();
    next.validate()?;
    let admission = t.elapsed().as_nanos();
    require(
        before == c.digest()?,
        "H004 diagnostic changed physical state",
    )?;
    Ok(
        serde_json::json!({"base_clone_ns":clone_ns,"face_construction_ns":topology_ns,"half_step_flux_ns":half_flux_ns,"begin_ns":begin,"validation_including_reconstruction_and_two_encodings_ns":validation,"one_pending_encoding_ns":encoding,"pending_encoded_bytes":pending_bytes,"finish_including_validation_ns":finish,"final_validation_ns":admission,"physical_digest_unchanged":true,"scope":"intrusive separate invocations, non-additive; committed state untouched"}),
    )
}
