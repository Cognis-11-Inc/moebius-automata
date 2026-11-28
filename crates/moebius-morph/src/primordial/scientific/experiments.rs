//! Authored bounded physical controls, distinct from donor results and ship inputs.
use super::*;
use std::f64::consts::PI;
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Scenario {
    Equilibrium,
    Diffusion,
    Advection,
    Shear,
    Wave,
    Hydrostatic,
    Disturbance,
    Interacting,
}
pub fn config(n: usize, dt: f64, model: Model, scenario: Scenario, species: usize) -> Config {
    let mut c = Config {
        model,
        roots: [n, 2, 2],
        h: 1. / n as f64,
        dt,
        periodic: !matches!(scenario, Scenario::Hydrostatic),
        capacity: 800_000,
        ..Default::default()
    };
    if model == Model::AdaptiveLbm {
        c.sound_speed = c.h / c.dt / 3f64.sqrt();
    }
    if matches!(scenario, Scenario::Hydrostatic) {
        c.gravity = [0., -0.0025, 0.];
    }
    for i in 0..species {
        c.species.push(Species {
            material: format!("authored-probe-{i}"),
            radius_m: 1e-5,
            density_kg_m3: 20.,
            diffusivity_m2_s: 0.001,
        });
    }
    c
}
pub fn initial(c: &Config, scenario: Scenario, g: mesh::Geometry) -> (f64, V, [f64; 4]) {
    let x = g.centre()[0];
    let mut rho = c.density;
    let mut u = [0.; 3];
    let mut scalar = [0.; 4];
    match scenario {
        Scenario::Equilibrium => {}
        Scenario::Diffusion | Scenario::Advection => {
            if matches!(scenario, Scenario::Advection) {
                u[0] = 0.05;
            }
            for (k, s) in scalar.iter_mut().enumerate().take(c.species.len()) {
                *s = 1e-4 * (k + 1) as f64 * (1. + 0.5 * (2. * PI * x).cos());
            }
        }
        Scenario::Shear => u[1] = 0.01 * (2. * PI * x).sin(),
        Scenario::Wave => {
            rho *= 1. + 1e-4 * (2. * PI * x).cos();
            u[0] = c.sound_speed * 1e-4 * (2. * PI * x).cos();
        }
        Scenario::Hydrostatic => {
            let y = g.centre()[1];
            rho *= ((c.gravity[1] * (y - c.lengths()[1] / 2.)) / c.sound_speed.powi(2)).exp();
        }
        Scenario::Disturbance | Scenario::Interacting => {
            let bump = |xc: f64| (-((x - xc) / 0.06).powi(2)).exp();
            let b = bump(0.3)
                + if matches!(scenario, Scenario::Interacting) {
                    bump(0.7)
                } else {
                    0.
                };
            rho *= 1. + 1e-4 * b;
            u[0] = 0.05;
        }
    }
    (rho, u, scalar)
}
pub fn chamber(config: Config, scenario: Scenario, refined: bool) -> Result<Chamber, MorphError> {
    let c = config.clone();
    let mut chamber = Chamber::new(config, |g| initial(&c, scenario, g))?;
    if refined {
        let roots = (0..c.count())
            .filter(|r| {
                let x = mesh::geometry(&c, (*r as u64) << 4).centre()[0];
                (0.375..0.625).contains(&x)
            })
            .collect::<Vec<_>>();
        chamber.refine(&roots)?;
    }
    Ok(chamber)
}
pub fn errors(s: &State, scenario: Scenario) -> serde_json::Value {
    let t = s.cycle as f64 * s.config.dt;
    let mut absolute = 0.;
    let mut scale = 0.;
    let mut amplitude = 0.;
    let mut wave_l2 = 0.;
    let mut wave_scale = 0.;
    let mut scalar_first = [0.; 2];
    let k = 2. * PI;
    for &key in s.cells.keys() {
        let g = mesh::geometry(&s.config, key);
        let x = g.centre()[0];
        let (rho, u, sc) = s.primitive(key);
        let volume = g.volume();
        match scenario {
            Scenario::Diffusion | Scenario::Advection => {
                let vel = if matches!(scenario, Scenario::Advection) {
                    0.05
                } else {
                    0.
                };
                let exact = 1e-4
                    * (1.
                        + 0.5
                            * (-s.config.species[0].diffusivity_m2_s * k * k * t).exp()
                            * (k * (x - vel * t)).cos());
                absolute += (sc[0] - exact).abs() * volume;
                scale += 1e-4 * 0.5 * volume;
                scalar_first[0] += sc[0] * (k * x).cos() * volume;
                scalar_first[1] += sc[0] * (k * x).sin() * volume;
            }
            Scenario::Shear => {
                let exact = 0.01
                    * (-s.config.viscosity / s.config.density * k * k * t).exp()
                    * (k * x).sin();
                absolute += (u[1] - exact).abs() * volume;
                scale += 0.01 * (k * x).sin().abs() * volume;
                amplitude += 2. * u[1] * (k * x).sin() * volume;
            }
            Scenario::Wave => {
                let exact = 1. + 1e-4 * (k * (x - s.config.sound_speed * t)).cos();
                wave_l2 += (rho / s.config.density - exact).powi(2) * volume;
                wave_scale += (1e-4f64).powi(2) * volume;
            }
            _ => {}
        }
    }
    let volume = s.config.lengths().iter().product::<f64>();
    let centroid_phase = scalar_first[1].atan2(scalar_first[0]) / k;
    let expected_phase = if matches!(scenario, Scenario::Advection) {
        0.05 * t
    } else {
        0.
    };
    let phase_error = (centroid_phase - expected_phase + 0.5).rem_euclid(1.) - 0.5;
    serde_json::json!({"normalized_l1":if scale>0.{Some(absolute/scale)}else{None},"normalized_wave_l2":if wave_scale>0.{Some((wave_l2/wave_scale).sqrt())}else{None},"shear_amplitude":amplitude/volume,"shear_exact_amplitude":0.01*(-s.config.viscosity/s.config.density*k*k*t).exp(),"centroid_phase_error_m":phase_error,"max_speed_m_s":s.diagnostics().max_speed})
}

pub mod fidelity;
