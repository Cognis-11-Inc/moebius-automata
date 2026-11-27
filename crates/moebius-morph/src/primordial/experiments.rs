//! Headless physical experiments, shared setup with the environmental Witness.
use super::*;
pub fn demo(config: Config) -> Result<Chamber, MorphError> {
    let mut c = Chamber::new(config)?;
    let length = c.state.config.length_m;
    let h = c.state.config.h();
    c.initialize_velocity(|p| {
        let r2 = (p[0] - length * 0.4).powi(2)
            + (p[1] - length * 0.5).powi(2)
            + (p[2] - length * 0.5).powi(2);
        [0.015 * (-r2 / 0.02).exp(), 0., 0.]
    })?;
    for (material, radius, density, offset) in [
        ("fine mineral", 0.004, 100., 0.35),
        ("heavy aerosol", 0.012, 250., 0.62),
    ] {
        c.add_species(
            Species {
                material: material.into(),
                radius_m: radius,
                density_kg_m3: density,
                diffusivity_m2_s: 0.00002,
            },
            |p| {
                let r2 = (p[0] - length * offset).powi(2)
                    + (p[1] - length * 0.6).powi(2)
                    + (p[2] - length * 0.5).powi(2);
                1e-6 * (-r2 / 0.005).exp() * h.powi(3)
            },
        )?;
    }
    c.add_sphere(Sphere {
        id: 1,
        radius_m: 0.01_f64.min(h / 5.),
        density_kg_m3: 80.,
        position_m: [length * 0.5, length * 0.65, length * 0.5],
        velocity_m_s: [0.01, 0., 0.],
    })?;
    for i in 0..8 {
        c.add_sphere(Sphere {
            id: i + 2,
            radius_m: 0.002,
            density_kg_m3: 100. + 20. * i as f64,
            position_m: [
                length * (0.2 + 0.07 * i as f64),
                length * 0.75,
                length * 0.65,
            ],
            velocity_m_s: [0.; 3],
        })?;
    }
    c.seal()?;
    Ok(c)
}

use serde_json::{Value, json};
use std::fs;
fn write(path: &Path, value: &impl Serialize) -> Result<(), MorphError> {
    fs::write(path, serde_json::to_vec_pretty(value).map_err(error)?).map_err(error)
}
fn scalar_moments(state: &State) -> Vec<Value> {
    state.fields.iter().map(|f|{let total=f.mass.total();let mut centre=[0.;3];let mut second=[0.;3];for (i,m) in f.mass.nonzero() {let p=state.config.centre(i);for k in 0..3 {centre[k]+=m*p[k];second[k]+=m*p[k]*p[k];}}if total>0. {for k in 0..3 {centre[k]/=total;second[k]=second[k]/total-centre[k]*centre[k];}}json!({"material":f.species.material,"suspended_kg":total,"wall_kg":f.wall_kg,"centre_m":centre,"variance_m2":second,"slip_m_s":f.species.slip(&state.config)})}).collect()
}
fn observe(state: &State) -> Value {
    let n = state.config.cells;
    json!({"tick":state.tick,"diagnostics":state.diagnostics(),"spheres":state.spheres,"species":scalar_moments(state),"rss_kib":rss(),"flow_samples":(0..n).map(|i|{let cell=state.config.index([i,n/2,n/2]);json!({"at":state.config.centre(cell),"sample":state.sample_index(cell)})}).collect::<Vec<_>>()})
}
fn run_case(
    root: &Path,
    name: &str,
    mut world: Chamber,
    ticks: usize,
    memory: fn() -> [usize; 3],
) -> Result<Value, MorphError> {
    let out = root.join(name);
    fs::create_dir(&out).map_err(error)?;
    world.seal()?;
    world.save(&out.join("initial.json"))?;
    write(&out.join("config-si.json"), &world.state.config)?;
    let mut rows = Vec::new();
    let mut digests = Vec::new();
    for _ in 0..ticks {
        let before = memory();
        let profile = world.step()?;
        let after = memory();
        let mut row = observe(world.state());
        row["profile"] = serde_json::to_value(profile).map_err(error)?;
        row["requested_bytes"] = json!(after[0] - before[0]);
        row["live_bytes"] = json!(after[1]);
        row["peak_live_bytes"] = json!(after[2]);
        digests.push(world.digest()?);
        rows.push(row);
    }
    world.save(&out.join("final.json"))?;
    write(&out.join("rows.json"), &rows)?;
    let mut replay = Chamber::load(&out.join("initial.json"))?;
    for digest in &digests {
        replay.step()?;
        require(
            replay.digest()? == *digest,
            "H001 exact every-tick replay mismatch",
        )?;
    }
    require(
        replay.state == world.state,
        "H001 final state replay mismatch",
    )?;
    let summary = json!({"experiment":name,"ticks":ticks,"initial":"initial.json","final":"final.json","final_digest":world.digest()?,"every_tick_replay_exact":true,"initial_config":world.state.config,"last_observation":rows.last(),"tick_ns":rows.iter().map(|r|r["profile"]["total_ns"].as_u64().unwrap()).collect::<Vec<_>>(),"total_requested_bytes":rows.iter().map(|r|r["requested_bytes"].as_u64().unwrap()).sum::<u64>(),"retained_allocated_bytes":rows.last().map(|r|&r["live_bytes"])});
    write(&out.join("summary.json"), &summary)?;
    Ok(summary)
}
fn base() -> Config {
    Config {
        cells: 8,
        gravity_m_s2: [0.; 3],
        ..Default::default()
    }
}
fn sphere(density: f64) -> Sphere {
    Sphere {
        id: 1,
        radius_m: 0.005,
        density_kg_m3: density,
        position_m: [0.425; 3],
        velocity_m_s: [0.; 3],
    }
}
fn tracer(material: &str, radius: f64, density: f64, diffusivity: f64) -> Species {
    Species {
        material: material.into(),
        radius_m: radius,
        density_kg_m3: density,
        diffusivity_m2_s: diffusivity,
    }
}
fn shear(config: Config, amplitude: f64) -> Result<Chamber, MorphError> {
    let length = config.length_m;
    let mut c = Chamber::new(config)?;
    c.initialize_velocity(|p| {
        [
            amplitude * (std::f64::consts::TAU * p[1] / length).sin(),
            0.,
            0.,
        ]
    })?;
    c.seal()?;
    Ok(c)
}
fn sine_amplitude(state: &State) -> f64 {
    let length = state.config.length_m;
    let mut sum = 0.;
    for i in 0..state.config.count() {
        sum += state.sample_index(i).velocity_m_s[0]
            * (std::f64::consts::TAU * state.config.centre(i)[1] / length).sin();
    }
    2. * sum / state.config.count() as f64
}

/// Each experiment records controls and actual verdicts, not scripted success.
pub fn suite(out: &Path, memory: fn() -> [usize; 3]) -> Result<(), MorphError> {
    fs::create_dir(out).map_err(error)?;
    let mut results = Vec::new();
    results.push(run_case(
        out,
        "E01-equilibrium",
        Chamber::new(base())?,
        120,
        memory,
    )?);
    let mut vacuum = Chamber::new(Config {
        density_kg_m3: 0.,
        viscosity_pa_s: 0.,
        ..base()
    })?;
    let mut b = sphere(80.);
    b.velocity_m_s = [0.01, 0., 0.];
    vacuum.add_sphere(b)?;
    results.push(run_case(out, "E02-vacuum", vacuum, 120, memory)?);
    for (index, g) in [[0., -0.01, 0.], [0.005, 0., 0.], [0., 0., -0.0025]]
        .into_iter()
        .enumerate()
    {
        let mut c = Chamber::new(Config {
            density_kg_m3: 0.,
            viscosity_pa_s: 0.,
            gravity_m_s2: g,
            ..base()
        })?;
        c.add_sphere(sphere(80.))?;
        results.push(run_case(
            out,
            &format!("E03-gravity-{index}"),
            c,
            120,
            memory,
        )?);
    }
    let mut drag = Chamber::new(base())?;
    let mut b = sphere(8000.);
    b.radius_m = 0.01;
    b.velocity_m_s = [0.01, 0., 0.];
    drag.add_sphere(b)?;
    results.push(run_case(out, "E04-drag", drag, 120, memory)?);
    for density in [10., 20., 40.] {
        let mut c = Chamber::new(Config {
            gravity_m_s2: [0., -0.0025, 0.],
            ..base()
        })?;
        c.add_sphere(sphere(density))?;
        results.push(run_case(
            out,
            &format!("E05-buoyancy-{}", density as usize),
            c,
            120,
            memory,
        )?);
    }
    let mut settling = Chamber::new(Config {
        gravity_m_s2: [0., -0.01, 0.],
        ..base()
    })?;
    for (name, rho, r) in [
        ("small", 200., 0.005),
        ("large", 200., 0.02),
        ("light", 50., 0.02),
    ] {
        settling.add_species(tracer(name, r, rho, 0.), |p| {
            if p.iter().all(|p| (*p - 0.45).abs() < 0.01) {
                1e-6
            } else {
                0.
            }
        })?;
    }
    results.push(run_case(out, "E06-settling", settling, 240, memory)?);
    let mut wall = Chamber::new(Config {
        gravity_m_s2: [0., -0.01, 0.],
        ..base()
    })?;
    wall.add_species(tracer("wall settling", 0.02, 200., 0.), |p| {
        if p[0] < 0.15 && p[1] < 0.15 && p[2] < 0.15 {
            1e-6
        } else {
            0.
        }
    })?;
    results.push(run_case(out, "E06-wall-deposition", wall, 120, memory)?);
    let mut convergence = Vec::new();
    for cells in [8, 16, 32] {
        for substeps in [1, 2, 4] {
            let config = Config {
                cells,
                substeps,
                boundary: Boundary::PeriodicReference,
                ..base()
            };
            let h = config.h();
            let length = config.length_m;
            let k = std::f64::consts::TAU / length;
            let speed = 0.05;
            let diffusion = 0.0005;
            let volume = h.powi(3);
            let average = (k * h * 0.5).sin() / (k * h * 0.5);
            let mut c = Chamber::new(config.clone())?;
            c.initialize_velocity(|_| [speed, 0., 0.])?;
            c.add_species(
                tracer("sinusoidal passive material", 0.001, 20., diffusion),
                |p| (1e-5 + 2e-6 * average * (k * p[0]).sin()) * volume,
            )?;
            c.seal()?;
            let initial = c.state.fields[0].mass.total();
            let mut timings = Vec::new();
            for _ in 0..30 {
                timings.push(c.step()?.total_ns);
            }
            let t = 30. * TICK_NANOS as f64 / 1e9;
            let mut error2 = 0.;
            for i in 0..config.count() {
                let p = config.centre(i);
                let exact = 1e-5
                    + 2e-6
                        * average
                        * (-diffusion * k * k * t).exp()
                        * (k * (p[0] - speed * t)).sin();
                error2 += (c.state.fields[0].mass.get(i) / volume - exact).powi(2);
            }
            let mut wave = shear(config, 0.01)?;
            let initial_amp = sine_amplitude(wave.state());
            let initial_ke = wave.state.diagnostics().kinetic_j;
            wave.advance(30)?;
            let expected_amp = initial_amp
                * (-c.state.config.viscosity_pa_s / c.state.config.density_kg_m3 * k * k * t).exp();
            let measured_amp = sine_amplitude(wave.state());
            convergence.push(json!({"cells":cells,"substeps":substeps,"h_m":h,"dt_s":c.state.config.dt(),"time_s":t,"scalar_l2_kg_m3":(error2/c.state.config.count() as f64).sqrt(),"species_mass_error_kg":c.state.fields[0].mass.total()-initial,"shear_reference_amplitude_m_s":expected_amp,"shear_measured_amplitude_m_s":measured_amp,"shear_relative_error":(measured_amp/expected_amp-1.).abs(),"shear_initial_kinetic_j":initial_ke,"shear_final_kinetic_j":wave.state.diagnostics().kinetic_j,"tick_ns":timings}));
            write(&out.join("E07-E10-convergence.json"), &convergence)?;
        }
    }
    for (name, centres) in [
        ("E08-local-disturbance", vec![0.3]),
        ("E08-second-disturbance", vec![0.55]),
        ("E08-two-disturbances", vec![0.3, 0.55]),
    ] {
        let config = Config {
            cells: 16,
            ..base()
        };
        let mut c = Chamber::new(config)?;
        c.initialize_velocity(|p| {
            let mut u = [0.; 3];
            for &x in &centres {
                let dy = p[1] - 0.4;
                let dx = p[0] - x;
                let gaussian = (-(dx * dx + dy * dy + (p[2] - 0.4).powi(2)) / 0.004).exp();
                u[0] += -dy * 0.1 * gaussian;
                u[1] += dx * 0.1 * gaussian;
            }
            u
        })?;
        results.push(run_case(out, name, c, 120, memory)?);
    }
    let mut coupling = Chamber::new(base())?;
    let mut b = sphere(8000.);
    b.radius_m = 0.01;
    b.velocity_m_s = [0.01, 0., 0.];
    coupling.add_sphere(b)?;
    results.push(run_case(out, "E09-momentum", coupling, 120, memory)?);
    results.push(run_case(
        out,
        "E11-demo-replay",
        demo(Config::default())?,
        120,
        memory,
    )?);
    write(&out.join("experiments.json"), &results)?;
    let mut gravity_convergence = Vec::new();
    for substeps in [1, 2, 4] {
        let g = [0., -0.01, 0.];
        let mut c = Chamber::new(Config {
            density_kg_m3: 0.,
            viscosity_pa_s: 0.,
            gravity_m_s2: g,
            substeps,
            ..base()
        })?;
        c.add_sphere(sphere(80.))?;
        c.seal()?;
        c.advance(120)?;
        let t = 120. * TICK_NANOS as f64 / 1e9;
        gravity_convergence.push(json!({"substeps":substeps,"dt_s":c.state.config.dt(),"position_error_m":norm(sub(c.state.spheres[0].position_m,add([0.425;3],mul(g,0.5*t*t)))),"velocity_error_m_s":norm(sub(c.state.spheres[0].velocity_m_s,mul(g,t)))}));
    }
    write(&out.join("E10-gravity-dt.json"), &gravity_convergence)?;
    // E12 uses factorials: same grid/physics except exactly the named factor.
    let mut scaling = Vec::new();
    for (factor, values) in [
        ("resolution", vec![8, 12, 16, 24, 32]),
        ("active-regions", vec![0, 1, 4, 16]),
        ("species-count", vec![0, 1, 2, 4]),
        ("explicit-objects", vec![0, 1, 16, 64, 256]),
    ] {
        for value in values {
            for backend in [ScalarBackend::Dense, ScalarBackend::SparseActiveFaces] {
                let cells = if factor == "resolution" { value } else { 16 };
                let config = Config {
                    cells,
                    scalar_backend: backend,
                    ..base()
                };
                let mut c = Chamber::new(config)?;
                let species = if factor == "species-count" { value } else { 1 };
                let regions = if factor == "active-regions" { value } else { 1 };
                let n = c.state.config.cells;
                for s in 0..species {
                    c.add_species(
                        tracer(&format!("factorial material{s}"), 0.001, 100., 0.00001),
                        |p| {
                            let cell = p.map(|x| (x / (0.8 / n as f64)) as usize);
                            if factor == "resolution" {
                                if p.iter().all(|x| *x >= 0.2 && *x < 0.4) {
                                    1e-6 * (0.8 / n as f64).powi(3)
                                } else {
                                    0.
                                }
                            } else if cell[1] == n / 2
                                && (0..regions).any(|r| {
                                    cell[0] == (r % 4) * (n / 4) + n / 8
                                        && cell[2] == (r / 4) * (n / 4) + n / 8
                                })
                            {
                                1e-8
                            } else {
                                0.
                            }
                        },
                    )?;
                }
                let objects = if factor == "explicit-objects" {
                    value
                } else {
                    0
                };
                for id in 0..objects {
                    c.add_sphere(Sphere {
                        id: id as u64 + 1,
                        radius_m: 0.001,
                        density_kg_m3: 100.,
                        position_m: [
                            0.15 + 0.06 * (id % 8) as f64,
                            0.15 + 0.06 * ((id / 8) % 8) as f64,
                            0.15 + 0.06 * (id / 64) as f64,
                        ],
                        velocity_m_s: [0.001, 0., 0.],
                    })?;
                }
                c.seal()?;
                let mut profiles = Vec::new();
                let before = memory();
                for _ in 0..16 {
                    profiles.push(c.step()?);
                }
                let after = memory();
                let sampling = Instant::now();
                let mut sample_sum = 0.;
                for i in 0..10000 {
                    sample_sum +=
                        std::hint::black_box(c.state.sample_index(i % c.state.config.count()))
                            .density_kg_m3;
                }
                let sample_ns = sampling.elapsed().as_nanos();
                std::hint::black_box(sample_sum);
                let mut neutral = c.state.clone();
                neutral.config.scalar_backend = ScalarBackend::Dense;
                let physical_signature = Chamber { state: neutral }.digest()?;
                scaling.push(json!({"storage_neutral_physical_digest":physical_signature,"factor":factor,"value":value,"cells":cells,"backend":backend,"nominal_active_regions":regions,"species_count":species,"explicit_objects":objects,"profiles":profiles,"requested_bytes":after[0]-before[0],"live_bytes":after[1],"peak_live_bytes":after[2],"field_samples":10000,"field_sampling_ns":sample_ns,"final_diagnostics":c.state.diagnostics()}));
                write(&out.join("E12-scaling.json"), &scaling)?;
            }
        }
    }
    Ok(())
}

fn rss() -> Option<u64> {
    fs::read_to_string("/proc/self/status")
        .ok()?
        .lines()
        .find_map(|line| {
            line.strip_prefix("VmRSS:")?
                .split_whitespace()
                .next()?
                .parse()
                .ok()
        })
}

pub fn record_demo(
    out: &Path,
    config: Config,
    ticks: usize,
    memory: fn() -> [usize; 3],
) -> Result<Value, MorphError> {
    require((1..=16384).contains(&ticks), "H001 recording tick budget")?;
    fs::create_dir(out).map_err(error)?;
    run_case(out, "demo", demo(config)?, ticks, memory)
}
