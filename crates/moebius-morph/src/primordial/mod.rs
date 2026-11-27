//! HABITAT-001: bounded experimental chamber, independent of ShipRuntime.
//! See docs/habitat_001/ENVIRONMENTAL_PHYSICS_CONTRACT.md for actual closures.
use crate::{
    MorphError,
    emission::SCALE,
    require,
    runtime::{Position, TICK_NANOS},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    time::Instant,
};
pub mod experiments;
mod transport;

pub type V = [f64; 3];
pub(crate) fn add(a: V, b: V) -> V {
    std::array::from_fn(|k| a[k] + b[k])
}
pub(crate) fn sub(a: V, b: V) -> V {
    std::array::from_fn(|k| a[k] - b[k])
}
pub(crate) fn mul(a: V, b: f64) -> V {
    a.map(|a| a * b)
}
pub(crate) fn dot(a: V, b: V) -> f64 {
    (0..3).map(|k| a[k] * b[k]).sum()
}
pub fn norm(a: V) -> f64 {
    dot(a, a).sqrt()
}
fn error(e: impl std::fmt::Display) -> MorphError {
    MorphError(e.to_string())
}
const C: [[i32; 3]; 19] = [
    [0, 0, 0],
    [1, 0, 0],
    [-1, 0, 0],
    [0, 1, 0],
    [0, -1, 0],
    [0, 0, 1],
    [0, 0, -1],
    [1, 1, 0],
    [-1, -1, 0],
    [1, -1, 0],
    [-1, 1, 0],
    [1, 0, 1],
    [-1, 0, -1],
    [1, 0, -1],
    [-1, 0, 1],
    [0, 1, 1],
    [0, -1, -1],
    [0, 1, -1],
    [0, -1, 1],
];
const OPP: [usize; 19] = [
    0, 2, 1, 4, 3, 6, 5, 8, 7, 10, 9, 12, 11, 14, 13, 16, 15, 18, 17,
];
fn weight(q: usize) -> f64 {
    if q == 0 {
        1. / 3.
    } else if q < 7 {
        1. / 18.
    } else {
        1. / 36.
    }
}
fn equilibrium(rho: f64, u: V) -> [f64; 19] {
    std::array::from_fn(|q| {
        let cu = dot(C[q].map(f64::from), u);
        weight(q) * rho * (1. + 3. * cu + 4.5 * cu * cu - 1.5 * dot(u, u))
    })
}
fn moments(f: &[f64; 19]) -> (f64, V) {
    let rho = f.iter().sum::<f64>();
    let mut p = [0.; 3];
    for q in 0..19 {
        for (k, p) in p.iter_mut().enumerate() {
            *p += f[q] * f64::from(C[q][k]);
        }
    }
    (rho, mul(p, 1. / rho))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Boundary {
    Sealed,
    PeriodicReference,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScalarBackend {
    Dense,
    SparseActiveFaces,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub origin: Position,
    pub cells: usize,
    pub length_m: f64,
    pub density_kg_m3: f64,
    pub viscosity_pa_s: f64,
    pub gravity_m_s2: V,
    pub substeps: usize,
    pub boundary: Boundary,
    pub scalar_backend: ScalarBackend,
    pub wall_restitution: f64,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            origin: Position::from_absolute([0; 3]).unwrap(),
            cells: 16,
            length_m: 0.8,
            density_kg_m3: 20.,
            viscosity_pa_s: 0.1,
            gravity_m_s2: [0., -0.0025, 0.],
            substeps: 1,
            boundary: Boundary::Sealed,
            scalar_backend: ScalarBackend::Dense,
            wall_restitution: 0.,
        }
    }
}
impl Config {
    pub fn h(&self) -> f64 {
        self.length_m / self.cells as f64
    }
    pub fn dt(&self) -> f64 {
        TICK_NANOS as f64 / 1e9 / self.substeps as f64
    }
    pub fn tau(&self) -> f64 {
        0.5 + 3. * self.viscosity_pa_s / self.density_kg_m3 * self.dt() / self.h().powi(2)
    }
    pub fn count(&self) -> usize {
        self.cells.pow(3)
    }
    pub fn validate(&self) -> Result<(), MorphError> {
        self.origin.absolute()?;
        require(
            (4..=32).contains(&self.cells) && (1..=8).contains(&self.substeps),
            "H001 grid/substep budget",
        )?;
        require(
            self.length_m.is_finite()
                && (0.1..=10.).contains(&self.length_m)
                && self.density_kg_m3.is_finite()
                && (0. ..=10000.).contains(&self.density_kg_m3)
                && self.viscosity_pa_s.is_finite()
                && (0. ..=100.).contains(&self.viscosity_pa_s),
            "H001 finite SI configuration",
        )?;
        require(
            self.gravity_m_s2.iter().all(|v| v.is_finite())
                && norm(self.gravity_m_s2) <= 0.05
                && self.wall_restitution.is_finite()
                && (0. ..=1.).contains(&self.wall_restitution),
            "H001 gravity/restitution envelope",
        )?;
        if self.density_kg_m3 == 0. {
            require(
                self.viscosity_pa_s == 0.,
                "H001 vacuum requires zero viscosity",
            )?;
        } else {
            require(
                self.viscosity_pa_s > 0. && (0.501..=2.).contains(&self.tau()),
                "H001 BGK relaxation stability envelope",
            )?;
        }
        Ok(())
    }
    pub fn coordinates(&self, i: usize) -> [usize; 3] {
        [
            i % self.cells,
            (i / self.cells) % self.cells,
            i / (self.cells * self.cells),
        ]
    }
    pub fn index(&self, p: [usize; 3]) -> usize {
        p[0] + self.cells * (p[1] + self.cells * p[2])
    }
    pub fn centre(&self, i: usize) -> V {
        self.coordinates(i).map(|x| (x as f64 + 0.5) * self.h())
    }
    pub fn cell_at(&self, p: V) -> Result<usize, MorphError> {
        require(
            p.iter()
                .all(|p| p.is_finite() && *p >= 0. && *p < self.length_m),
            "H001 sample outside chamber",
        )?;
        Ok(self.index(p.map(|p| ((p / self.h()) as usize).min(self.cells - 1))))
    }
    pub fn metric_position(&self, p: V) -> Result<Position, MorphError> {
        require(
            p.iter().all(|x| x.is_finite() && x.abs() <= 10.),
            "H001 observer offset envelope",
        )?;
        self.origin
            .offset(p.map(|x| (x * SCALE as f64).round() as i64))
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Species {
    pub material: String,
    pub radius_m: f64,
    pub density_kg_m3: f64,
    pub diffusivity_m2_s: f64,
}
impl Species {
    pub fn slip(&self, config: &Config) -> V {
        mul(
            config.gravity_m_s2,
            2. * self.radius_m.powi(2) * (self.density_kg_m3 - config.density_kg_m3)
                / (9. * config.viscosity_pa_s),
        )
    }
    fn validate(&self) -> Result<(), MorphError> {
        require(
            !self.material.is_empty()
                && self.material.len() <= 64
                && self.radius_m.is_finite()
                && (1e-7..=0.025).contains(&self.radius_m)
                && self.density_kg_m3.is_finite()
                && (0.01..=10000.).contains(&self.density_kg_m3)
                && self.diffusivity_m2_s.is_finite()
                && (0. ..=0.01).contains(&self.diffusivity_m2_s),
            "H001 species physical envelope",
        )
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Mass {
    Dense(Vec<f64>),
    Sparse(BTreeMap<usize, f64>),
}
impl Mass {
    pub fn get(&self, i: usize) -> f64 {
        match self {
            Self::Dense(v) => v[i],
            Self::Sparse(v) => v.get(&i).copied().unwrap_or(0.),
        }
    }
    pub fn nonzero(&self) -> Vec<(usize, f64)> {
        match self {
            Self::Dense(v) => v
                .iter()
                .enumerate()
                .filter_map(|(i, &v)| (v != 0.).then_some((i, v)))
                .collect(),
            Self::Sparse(v) => v.iter().map(|(&i, &v)| (i, v)).collect(),
        }
    }
    pub fn total(&self) -> f64 {
        match self {
            Self::Dense(v) => v.iter().sum(),
            Self::Sparse(v) => v.values().sum(),
        }
    }
    pub(super) fn accumulate(&mut self, i: usize, delta: f64) {
        if delta == 0. {
            return;
        }
        match self {
            Self::Dense(v) => v[i] += delta,
            Self::Sparse(v) => *v.entry(i).or_default() += delta,
        }
    }
    pub(super) fn coefficients_valid(&self) -> bool {
        match self {
            Self::Dense(v) => v.iter().all(|v| v.is_finite() && *v <= 0.9),
            Self::Sparse(v) => v.values().all(|v| v.is_finite() && *v <= 0.9),
        }
    }
    pub fn stored(&self) -> usize {
        match self {
            Self::Dense(v) => v.len(),
            Self::Sparse(v) => v.len(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpeciesField {
    pub species: Species,
    pub mass: Mass,
    pub wall_kg: [f64; 6],
    pub initial_kg: f64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sphere {
    pub id: u64,
    pub radius_m: f64,
    pub density_kg_m3: f64,
    pub position_m: V,
    pub velocity_m_s: V,
}
impl Sphere {
    pub fn volume(&self) -> f64 {
        4. / 3. * std::f64::consts::PI * self.radius_m.powi(3)
    }
    pub fn mass(&self) -> f64 {
        self.density_kg_m3 * self.volume()
    }
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Impulses {
    pub gravity_kg_m_s: V,
    /// Prescribed hydrostatic support; not resolved fluid/body internal exchange.
    pub hydrostatic_support_kg_m_s: V,
    pub walls_kg_m_s: V,
    pub drag_loss_j: f64,
    pub body_wall_loss_j: f64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub config: Config,
    pub tick: u64,
    pub populations: Vec<[f64; 19]>,
    pub fields: Vec<SpeciesField>,
    pub spheres: Vec<Sphere>,
    pub impulses: Impulses,
    pub carrier_initial_kg: f64,
    pub initial_momentum: V,
    pub sealed: bool,
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct Sample {
    pub density_kg_m3: f64,
    pub velocity_m_s: V,
    pub viscosity_pa_s: f64,
    pub dynamic_pressure_pa: f64,
}
#[derive(Clone, Debug, Serialize)]
pub struct Diagnostics {
    pub carrier_kg: f64,
    pub carrier_residual_kg: f64,
    pub species_residual_kg: Vec<f64>,
    pub momentum: V,
    pub momentum_residual: V,
    pub kinetic_j: f64,
    pub max_mach: f64,
    pub max_density_relative_error: f64,
    pub stored_scalar_cells: usize,
}
#[derive(Clone, Debug, Default, Serialize)]
pub struct StepProfile {
    pub total_ns: u128,
    pub clone_ns: u128,
    pub coupling_ns: u128,
    pub carrier_ns: u128,
    pub transport_ns: u128,
    pub validation_ns: u128,
    pub transported_faces: usize,
}
impl State {
    pub fn sample_index(&self, i: usize) -> Sample {
        if self.populations.is_empty() {
            return Sample {
                density_kg_m3: 0.,
                velocity_m_s: [0.; 3],
                viscosity_pa_s: 0.,
                dynamic_pressure_pa: 0.,
            };
        }
        let (rho, u) = moments(&self.populations[i]);
        let c = self.config.h() / self.config.dt();
        Sample {
            density_kg_m3: rho,
            velocity_m_s: mul(u, c),
            viscosity_pa_s: self.config.viscosity_pa_s,
            dynamic_pressure_pa: (rho - self.config.density_kg_m3) * c * c / 3.,
        }
    }
    pub fn sample(&self, p: V) -> Result<Sample, MorphError> {
        Ok(self.sample_index(self.config.cell_at(p)?))
    }
    pub fn diagnostics(&self) -> Diagnostics {
        let volume = self.config.h().powi(3);
        let mut mass = 0.;
        let mut momentum = [0.; 3];
        let mut energy = 0.;
        let mut mach = 0f64;
        let mut density_error = 0f64;
        for i in 0..self.populations.len() {
            let s = self.sample_index(i);
            mass += s.density_kg_m3 * volume;
            momentum = add(momentum, mul(s.velocity_m_s, s.density_kg_m3 * volume));
            energy += 0.5 * s.density_kg_m3 * volume * dot(s.velocity_m_s, s.velocity_m_s);
            mach =
                mach.max(norm(s.velocity_m_s) * self.config.dt() / self.config.h() * 3f64.sqrt());
            density_error =
                density_error.max((s.density_kg_m3 / self.config.density_kg_m3 - 1.).abs());
        }
        for b in &self.spheres {
            momentum = add(momentum, mul(b.velocity_m_s, b.mass()));
            energy += 0.5 * b.mass() * dot(b.velocity_m_s, b.velocity_m_s);
        }
        let expected = add(
            self.initial_momentum,
            add(
                self.impulses.gravity_kg_m_s,
                add(
                    self.impulses.hydrostatic_support_kg_m_s,
                    self.impulses.walls_kg_m_s,
                ),
            ),
        );
        Diagnostics {
            carrier_kg: mass,
            carrier_residual_kg: mass - self.carrier_initial_kg,
            species_residual_kg: self
                .fields
                .iter()
                .map(|f| f.mass.total() + f.wall_kg.iter().sum::<f64>() - f.initial_kg)
                .collect(),
            momentum,
            momentum_residual: sub(momentum, expected),
            kinetic_j: energy,
            max_mach: mach,
            max_density_relative_error: density_error,
            stored_scalar_cells: self.fields.iter().map(|f| f.mass.stored()).sum(),
        }
    }
    pub fn validate(&self) -> Result<(), MorphError> {
        self.config.validate()?;
        require(
            self.tick <= 10_000_000 && self.fields.len() <= 4 && self.spheres.len() <= 256,
            "H001 state budget",
        )?;
        require(
            self.populations.len()
                == if self.config.density_kg_m3 == 0. {
                    0
                } else {
                    self.config.count()
                },
            "H001 carrier storage mismatch",
        )?;
        require(
            self.populations
                .iter()
                .flatten()
                .all(|f| f.is_finite() && *f >= 0.),
            "H001 nonfinite/negative population",
        )?;
        let mut ids = BTreeSet::new();
        let mut volumes = 0.;
        for b in &self.spheres {
            require(
                ids.insert(b.id)
                    && b.id > 0
                    && b.radius_m.is_finite()
                    && b.radius_m > 0.
                    && b.mass().is_finite()
                    && b.mass() > 0.
                    && b.radius_m <= self.config.h() / 4.
                    && b.density_kg_m3.is_finite()
                    && (0.01..=10000.).contains(&b.density_kg_m3)
                    && b.velocity_m_s.iter().all(|v| v.is_finite())
                    && b.position_m.iter().all(|p| {
                        p.is_finite() && *p >= b.radius_m && *p <= self.config.length_m - b.radius_m
                    }),
                "H001 invalid inert sphere",
            )?;
            volumes += b.volume();
            if self.config.density_kg_m3 > 0. {
                let s = self.sample(b.position_m)?;
                require(
                    2. * b.radius_m * s.density_kg_m3 * norm(sub(b.velocity_m_s, s.velocity_m_s))
                        / self.config.viscosity_pa_s
                        <= 0.1,
                    "H001 Stokes Reynolds envelope",
                )?;
            }
        }
        require(
            volumes <= 0.01 * self.config.length_m.powi(3),
            "H001 unresolved body volume fraction",
        )?;
        let mut names = BTreeSet::new();
        for field in &self.fields {
            field.species.validate()?;
            require(
                2. * field.species.radius_m
                    * self.config.density_kg_m3
                    * norm(field.species.slip(&self.config))
                    / self.config.viscosity_pa_s
                    <= 0.1,
                "H001 statistical Stokes slip Reynolds envelope",
            )?;
            require(
                names.insert(&field.species.material) && self.config.density_kg_m3 > 0.,
                "H001 species identity/carrier mismatch",
            )?;
            match &field.mass {
                Mass::Dense(v) => require(
                    v.len() == self.config.count() && v.iter().all(|v| v.is_finite() && *v >= 0.),
                    "H001 dense concentration",
                )?,
                Mass::Sparse(v) => require(
                    v.iter()
                        .all(|(i, v)| *i < self.config.count() && v.is_finite() && *v > 0.),
                    "H001 sparse concentration",
                )?,
            }
            require(
                field.wall_kg.iter().all(|v| v.is_finite() && *v >= 0.)
                    && field.initial_kg.is_finite()
                    && field.initial_kg >= 0.,
                "H001 wall/species inventory",
            )?;
        }
        require(
            self.fields.iter().map(|f| f.initial_kg).sum::<f64>()
                <= self.carrier_initial_kg * 0.001,
            "H001 passive loading approximation",
        )?;
        let d = self.diagnostics();
        require(
            d.carrier_kg.is_finite()
                && d.carrier_residual_kg.abs() <= 1e-12 + 1e-10 * self.carrier_initial_kg
                && d.species_residual_kg
                    .iter()
                    .zip(&self.fields)
                    .all(|(r, f)| r.abs() <= 1e-16 + 1e-10 * f.initial_kg),
            "H001 material conservation",
        )?;
        let scale = norm(self.initial_momentum)
            + norm(self.impulses.gravity_kg_m_s)
            + norm(self.impulses.hydrostatic_support_kg_m_s)
            + norm(self.impulses.walls_kg_m_s);
        require(
            norm(d.momentum_residual) <= 1e-10 + scale * 1e-9
                && self.impulses.drag_loss_j.is_finite()
                && self.impulses.drag_loss_j >= 0.
                && self.impulses.body_wall_loss_j.is_finite()
                && self.impulses.body_wall_loss_j >= 0.,
            "H001 momentum/dissipation accounting",
        )?;
        require(
            d.max_mach <= 0.1 && d.max_density_relative_error <= 0.05,
            "H001 low-Mach/density validity envelope",
        )?;
        Ok(())
    }
}
#[derive(Debug)]
pub struct Chamber {
    state: State,
}
impl Chamber {
    pub fn new(config: Config) -> Result<Self, MorphError> {
        config.validate()?;
        let populations = if config.density_kg_m3 == 0. {
            Vec::new()
        } else {
            vec![equilibrium(config.density_kg_m3, [0.; 3]); config.count()]
        };
        let initial = config.density_kg_m3 * config.length_m.powi(3);
        Ok(Self {
            state: State {
                config,
                tick: 0,
                populations,
                fields: Vec::new(),
                spheres: Vec::new(),
                impulses: Impulses::default(),
                carrier_initial_kg: initial,
                initial_momentum: [0.; 3],
                sealed: false,
            },
        })
    }
    pub fn state(&self) -> &State {
        &self.state
    }
    pub fn initialize_velocity(&mut self, velocity: impl Fn(V) -> V) -> Result<(), MorphError> {
        require(
            !self.state.sealed && self.state.tick == 0,
            "H001 initialization already sealed",
        )?;
        let c = self.state.config.dt() / self.state.config.h();
        for i in 0..self.state.populations.len() {
            let u = velocity(self.state.config.centre(i));
            require(
                u.iter().all(|v| v.is_finite()) && norm(u) * c * 3f64.sqrt() <= 0.1,
                "H001 initial velocity envelope",
            )?;
            self.state.populations[i] = equilibrium(self.state.config.density_kg_m3, mul(u, c));
        }
        Ok(())
    }
    pub fn add_species(
        &mut self,
        species: Species,
        mass_kg: impl Fn(V) -> f64,
    ) -> Result<(), MorphError> {
        require(
            !self.state.sealed
                && self.state.fields.len() < 4
                && self.state.config.density_kg_m3 > 0.,
            "H001 species initialization budget",
        )?;
        species.validate()?;
        let masses = (0..self.state.config.count())
            .map(|i| mass_kg(self.state.config.centre(i)))
            .collect::<Vec<_>>();
        require(
            masses.iter().all(|v| v.is_finite() && *v >= 0.),
            "H001 initialized material must be finite/nonnegative",
        )?;
        let initial_kg = masses.iter().sum();
        let mass = match self.state.config.scalar_backend {
            ScalarBackend::Dense => Mass::Dense(masses),
            ScalarBackend::SparseActiveFaces => Mass::Sparse(
                masses
                    .into_iter()
                    .enumerate()
                    .filter(|(_, v)| *v != 0.)
                    .collect(),
            ),
        };
        self.state.fields.push(SpeciesField {
            species,
            mass,
            wall_kg: [0.; 6],
            initial_kg,
        });
        Ok(())
    }
    pub fn add_sphere(&mut self, sphere: Sphere) -> Result<(), MorphError> {
        require(
            !self.state.sealed && self.state.spheres.len() < 256,
            "H001 sphere initialization budget",
        )?;
        self.state.spheres.push(sphere);
        self.state.spheres.sort_by_key(|b| b.id);
        Ok(())
    }
    pub fn seal(&mut self) -> Result<(), MorphError> {
        if !self.state.sealed {
            let initial = self.state.diagnostics().momentum;
            self.state.initial_momentum = initial;
            self.state.sealed = true;
        }
        self.state.validate()
    }
    pub fn step(&mut self) -> Result<StepProfile, MorphError> {
        let started = Instant::now();
        let clone = Instant::now();
        let mut next = self.state.clone();
        let clone_ns = clone.elapsed().as_nanos();
        let validation = Instant::now();
        if !next.sealed {
            next.initial_momentum = next.diagnostics().momentum;
            next.sealed = true;
        }
        next.validate()?;
        let mut profile = StepProfile {
            clone_ns,
            validation_ns: validation.elapsed().as_nanos(),
            ..Default::default()
        };
        for _ in 0..next.config.substeps {
            let t = Instant::now();
            couple(&mut next)?;
            profile.coupling_ns += t.elapsed().as_nanos();
            let t = Instant::now();
            carrier_step(&mut next)?;
            profile.carrier_ns += t.elapsed().as_nanos();
            let t = Instant::now();
            profile.transported_faces += transport::step(&mut next)?;
            profile.transport_ns += t.elapsed().as_nanos();
        }
        next.tick = next
            .tick
            .checked_add(1)
            .ok_or_else(|| error("H001 clock overflow"))?;
        let t = Instant::now();
        next.validate()?;
        profile.validation_ns += t.elapsed().as_nanos();
        self.state = next;
        profile.total_ns = started.elapsed().as_nanos();
        Ok(profile)
    }
    pub fn advance(&mut self, ticks: usize) -> Result<(), MorphError> {
        require(ticks <= 16384, "H001 advance budget")?;
        for _ in 0..ticks {
            self.step()?;
        }
        Ok(())
    }
    pub fn digest(&self) -> Result<String, MorphError> {
        let state = &self.state;
        let fields = state
            .fields
            .iter()
            .map(|f| (&f.species, f.initial_kg, f.wall_kg))
            .collect::<Vec<_>>();
        let metadata = serde_json::json!({"schema":"H001-physical-bits-v1","config":state.config,"tick":state.tick,"sealed":state.sealed,"spheres":state.spheres,"impulses":state.impulses,"carrier_initial":state.carrier_initial_kg,"initial_momentum":state.initial_momentum,"fields":fields,"populations":state.populations.len()});
        let mut hash = blake3::Hasher::new();
        hash.update(&serde_json::to_vec(&encode_bits(metadata)).map_err(error)?);
        for cell in &state.populations {
            for value in cell {
                hash.update(&value.to_bits().to_le_bytes());
            }
        }
        for field in &state.fields {
            for i in 0..state.config.count() {
                hash.update(&field.mass.get(i).to_bits().to_le_bytes());
            }
        }
        Ok(hash.finalize().to_hex().to_string())
    }
    pub fn save(&self, path: &Path) -> Result<(), MorphError> {
        self.state.validate()?;
        require(
            self.state.sealed,
            "H001 checkpoint requires sealed initialization",
        )?;
        let data = serde_json::json!({"schema":"HABITAT-001/f64-bits-v1","source_identity":crate::BENCHMARK_BUILD_IDENTITY,"digest":self.digest()?,"state":encode_bits(serde_json::to_value(&self.state).map_err(error)?)});
        crate::write_atomic(path, &serde_json::to_string(&data).map_err(error)?)
    }
    pub fn load(path: &Path) -> Result<Self, MorphError> {
        require(
            std::fs::metadata(path).map_err(error)?.len() <= 64 * 1024 * 1024,
            "H001 checkpoint byte budget",
        )?;
        let mut value: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path).map_err(error)?).map_err(error)?;
        require(
            value["schema"] == "HABITAT-001/f64-bits-v1"
                && value["source_identity"] == crate::BENCHMARK_BUILD_IDENTITY,
            "H001 snapshot schema/source mismatch",
        )?;
        let expected = value["digest"]
            .as_str()
            .ok_or_else(|| error("H001 missing digest"))?
            .to_string();
        let state: State =
            serde_json::from_value(decode_bits(value["state"].take())?).map_err(error)?;
        state.validate()?;
        require(state.sealed, "H001 unsealed snapshot")?;
        let chamber = Self { state };
        require(
            chamber.digest()? == expected,
            "H001 snapshot digest mismatch",
        )?;
        Ok(chamber)
    }
}
fn encode_bits(v: serde_json::Value) -> serde_json::Value {
    use serde_json::Value;
    match v {
        Value::Number(n) if n.is_f64() => {
            serde_json::json!({"$f64":format!("{:016x}",n.as_f64().unwrap().to_bits())})
        }
        Value::Array(v) => Value::Array(v.into_iter().map(encode_bits).collect()),
        Value::Object(v) => {
            Value::Object(v.into_iter().map(|(k, v)| (k, encode_bits(v))).collect())
        }
        v => v,
    }
}
fn decode_bits(v: serde_json::Value) -> Result<serde_json::Value, MorphError> {
    use serde_json::Value;
    Ok(match v {
        Value::Object(mut v) if v.len() == 1 && v.contains_key("$f64") => {
            let value = v.remove("$f64").unwrap();
            let s = value
                .as_str()
                .ok_or_else(|| error("H001 floating encoding"))?;
            let f = f64::from_bits(u64::from_str_radix(s, 16).map_err(error)?);
            require(f.is_finite(), "H001 nonfinite snapshot float")?;
            serde_json::json!(f)
        }
        Value::Array(v) => Value::Array(v.into_iter().map(decode_bits).collect::<Result<_, _>>()?),
        Value::Object(v) => Value::Object(
            v.into_iter()
                .map(|(k, v)| Ok((k, decode_bits(v)?)))
                .collect::<Result<_, MorphError>>()?,
        ),
        v => v,
    })
}

fn inject(state: &mut State, i: usize, impulse: V) -> Result<(), MorphError> {
    let (rho, u) = moments(&state.populations[i]);
    let new = add(
        u,
        mul(
            impulse,
            state.config.dt() / state.config.h() / (rho * state.config.h().powi(3)),
        ),
    );
    let old_eq = equilibrium(rho, u);
    let new_eq = equilibrium(rho, new);
    for q in 0..19 {
        state.populations[i][q] += new_eq[q] - old_eq[q];
    }
    require(
        state.populations[i]
            .iter()
            .all(|v| v.is_finite() && *v >= 0.),
        "H001 impulse exceeds population positivity",
    )
}
fn couple(state: &mut State) -> Result<(), MorphError> {
    let dt = state.config.dt();
    for index in 0..state.spheres.len() {
        let mut b = state.spheres[index].clone();
        let mass = b.mass();
        let gravity = mul(state.config.gravity_m_s2, mass);
        let mut buoyancy = [0.; 3];
        if state.populations.is_empty() {
            b.position_m = add(
                b.position_m,
                add(
                    mul(b.velocity_m_s, dt),
                    mul(state.config.gravity_m_s2, 0.5 * dt * dt),
                ),
            );
            b.velocity_m_s = add(b.velocity_m_s, mul(state.config.gravity_m_s2, dt));
        } else {
            let cell = state.config.cell_at(b.position_m)?;
            let sample = state.sample_index(cell);
            buoyancy = mul(
                state.config.gravity_m_s2,
                -sample.density_kg_m3 * b.volume(),
            );
            let acceleration = mul(add(gravity, buoyancy), 1. / mass);
            let mf = sample.density_kg_m3 * state.config.h().powi(3);
            let total = mass + mf;
            let reduced = mass * mf / total;
            let gamma = 6. * std::f64::consts::PI * state.config.viscosity_pa_s * b.radius_m;
            let lambda = gamma / reduced;
            let e = (-lambda * dt).exp();
            let phi = -(-lambda * dt).exp_m1() / lambda;
            let relative = sub(b.velocity_m_s, sample.velocity_m_s);
            let terminal = mul(acceleration, 1. / lambda);
            let deviation = sub(relative, terminal);
            let cm = mul(
                add(mul(b.velocity_m_s, mass), mul(sample.velocity_m_s, mf)),
                1. / total,
            );
            let relative_new = add(terminal, mul(deviation, e));
            let cm_new = add(cm, mul(acceleration, mass / total * dt));
            let velocity_new = add(cm_new, mul(relative_new, mf / total));
            let fluid_new = sub(cm_new, mul(relative_new, mass / total));
            let displacement = add(
                add(mul(cm, dt), mul(acceleration, 0.5 * mass / total * dt * dt)),
                mul(add(mul(terminal, dt), mul(deviation, phi)), mf / total),
            );
            b.position_m = add(b.position_m, displacement);
            b.velocity_m_s = velocity_new;
            inject(state, cell, mul(sub(fluid_new, sample.velocity_m_s), mf))?;
            let loss = gamma
                * (dot(terminal, terminal) * dt
                    + 2. * dot(terminal, deviation) * phi
                    + dot(deviation, deviation) * (-(-2. * lambda * dt).exp_m1()) / (2. * lambda));
            require(loss >= -1e-20, "H001 negative pair drag loss")?;
            state.impulses.drag_loss_j += loss.max(0.);
        }
        state.impulses.gravity_kg_m_s = add(state.impulses.gravity_kg_m_s, mul(gravity, dt));
        state.impulses.hydrostatic_support_kg_m_s =
            add(state.impulses.hydrostatic_support_kg_m_s, mul(buoyancy, dt));
        for axis in 0..3 {
            if b.position_m[axis] < b.radius_m
                || b.position_m[axis] > state.config.length_m - b.radius_m
            {
                let wall_velocity = b.velocity_m_s[axis];
                b.position_m[axis] =
                    b.position_m[axis].clamp(b.radius_m, state.config.length_m - b.radius_m);
                b.velocity_m_s[axis] = -state.config.wall_restitution * wall_velocity;
                state.impulses.walls_kg_m_s[axis] += mass * (b.velocity_m_s[axis] - wall_velocity);
                state.impulses.body_wall_loss_j +=
                    0.5 * mass * (wall_velocity.powi(2) - b.velocity_m_s[axis].powi(2));
            }
        }
        state.spheres[index] = b;
    }
    Ok(())
}
fn carrier_step(state: &mut State) -> Result<(), MorphError> {
    if state.populations.is_empty() {
        return Ok(());
    }
    let mut streamed = vec![[0.; 19]; state.config.count()];
    let tau = state.config.tau();
    let scale = state.config.h().powi(4) / state.config.dt();
    let n = state.config.cells as i32;
    for (i, f) in state.populations.iter().enumerate() {
        let (rho, u) = moments(f);
        let eq = equilibrium(rho, u);
        let p = state.config.coordinates(i);
        for q in 0..19 {
            let post = f[q] + (eq[q] - f[q]) / tau;
            require(
                post.is_finite() && post >= 0.,
                "H001 unstable BGK collision",
            )?;
            let at: [i32; 3] = std::array::from_fn(|k| p[k] as i32 + C[q][k]);
            let outside = at.iter().any(|x| *x < 0 || *x >= n);
            if outside && state.config.boundary == Boundary::Sealed {
                streamed[i][OPP[q]] += post;
                state.impulses.walls_kg_m_s = sub(
                    state.impulses.walls_kg_m_s,
                    mul(C[q].map(f64::from), 2. * post * scale),
                );
            } else {
                let dest = state.config.index(at.map(|x| x.rem_euclid(n) as usize));
                streamed[dest][q] += post;
            }
        }
    }
    state.populations = streamed;
    Ok(())
}

#[cfg(test)]
mod tests;
