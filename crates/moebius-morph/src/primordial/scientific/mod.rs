//! HABITAT-003: explicitly authorized alternative numerical models, no ship consumer.
//! One selected model owns nonoverlapping leaf inventories. See003 numerical contract.
use super::*;
pub mod experiments;
mod fv;
mod lbm;
pub mod mesh;
pub mod reconstruction;
pub mod topology;
pub const NF: usize = 8;
pub type U = [f64; NF];
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Model {
    FiniteVolume,
    AdaptiveLbm,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Reconstruction {
    #[default]
    Constant,
    Minmod,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Integrator {
    #[default]
    Euler,
    Ssp2,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VortexForcing {
    pub amplitude: f64,
    pub wavenumber: f64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Indicators {
    pub density: f64,
    pub velocity: f64,
    pub scalar: f64,
    pub nonequilibrium: f64,
    pub coarsen_fraction: f64,
    pub quiet_cycles: u32,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub model: Model,
    pub origin: Position,
    pub roots: [usize; 3],
    pub h: f64,
    pub dt: f64,
    pub density: f64,
    pub viscosity: f64,
    pub sound_speed: f64,
    pub gravity: V,
    pub species: Vec<Species>,
    pub periodic: bool,
    pub capacity: usize,
    pub indicators: Option<Indicators>,
    #[serde(default)]
    pub reconstruction: Reconstruction,
    #[serde(default)]
    pub integrator: Integrator,
    #[serde(default)]
    pub vortex_forcing: Option<VortexForcing>,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            model: Model::FiniteVolume,
            origin: Position::from_absolute([0; 3]).unwrap(),
            roots: [8, 2, 2],
            h: 0.1,
            dt: 0.001,
            density: 20.,
            viscosity: 0.1,
            sound_speed: 3f64.sqrt() * 2.,
            gravity: [0.; 3],
            species: Vec::new(),
            periodic: false,
            capacity: 100_000,
            indicators: None,
            reconstruction: Reconstruction::Constant,
            integrator: Integrator::Euler,
            vortex_forcing: None,
        }
    }
}
impl Config {
    pub fn count(&self) -> usize {
        self.roots.iter().product()
    }
    pub fn lengths(&self) -> V {
        std::array::from_fn(|k| self.roots[k] as f64 * self.h)
    }
    pub fn tau(&self, level: u8) -> f64 {
        let h = self.h / (1usize << level) as f64;
        let dt = self.dt / (1usize << level) as f64;
        0.5 + 3. * self.viscosity / self.density * dt / (h * h)
    }
    pub fn validate(&self) -> Result<(), MorphError> {
        self.origin.absolute()?;
        if self.model == Model::AdaptiveLbm {
            require(
                self.reconstruction == Reconstruction::Constant
                    && self.integrator == Integrator::Euler
                    && self.vortex_forcing.is_none(),
                "H004 LBM candidate unchanged",
            )?;
        }
        if let Some(f) = &self.vortex_forcing {
            require(
                self.model == Model::FiniteVolume
                    && self.periodic
                    && self.gravity == [0.; 3]
                    && f.amplitude.is_finite()
                    && (0. ..=0.02).contains(&f.amplitude)
                    && f.wavenumber.is_finite()
                    && (0.1..=100.).contains(&f.wavenumber),
                "H004 manufactured source envelope",
            )?;
        }
        require(
            self.roots.iter().all(|&v| v > 0 && v <= 65536)
                && self.count() <= 100_000
                && self.capacity >= self.count()
                && self.capacity <= 800_000,
            "H003 root/capacity envelope",
        )?;
        require(
            self.h.is_finite()
                && (1e-4..=100.).contains(&self.h)
                && self.dt.is_finite()
                && (1e-8..=0.1).contains(&self.dt)
                && self.density.is_finite()
                && (0.01..=10000.).contains(&self.density)
                && self.viscosity.is_finite()
                && (1e-8..=100.).contains(&self.viscosity)
                && self.sound_speed.is_finite()
                && (0.01..=10000.).contains(&self.sound_speed)
                && norm(self.gravity) <= 0.05
                && self.gravity.iter().all(|v| v.is_finite())
                && self.species.len() <= 4,
            "H003 finite SI configuration",
        )?;
        for s in &self.species {
            s.validate()?;
        }
        if self.model == Model::AdaptiveLbm {
            require(
                (self.sound_speed - self.h / self.dt / 3f64.sqrt()).abs() < 1e-10
                    && (0.501..=2.).contains(&self.tau(0))
                    && (0.501..=2.).contains(&self.tau(1))
                    && self.species.is_empty()
                    && self.gravity == [0.; 3],
                "H003 LBM acoustic/closure envelope",
            )?;
        }
        if let Some(i) = &self.indicators {
            require(
                [i.density, i.velocity, i.scalar, i.nonequilibrium]
                    .iter()
                    .all(|v| v.is_finite() && *v > 0.)
                    && i.coarsen_fraction > 0.
                    && i.coarsen_fraction < 1.
                    && i.quiet_cycles > 0,
                "H003 versioned indicator envelope",
            )?;
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Payload {
    Fv(U),
    Lbm([f64; 19]),
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ledger {
    pub initial: U,
    pub gravity_impulse: V,
    pub walls_impulse: V,
    pub hydrostatic_support: V,
    pub wall_scalar: [[f64; 6]; 4],
    pub drag_loss: f64,
    pub body_wall_loss: f64,
    #[serde(default)]
    pub manufactured_impulse: V,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub config: Config,
    pub cycle: u64,
    pub cells: BTreeMap<u64, Payload>,
    pub bodies: Vec<Sphere>,
    pub ledger: Ledger,
    pub quiet: BTreeMap<usize, u32>,
    pub refinements: u64,
    pub coarsenings: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pending {
    pub work: State,
    pub predictor: BTreeMap<u64, U>,
    pub stage: u8,
    pub interface: BTreeMap<u64, U>,
}
#[derive(Clone, Debug, Serialize)]
pub struct Diagnostics {
    pub total: U,
    pub residual: U,
    pub leaves: usize,
    pub refined_roots: usize,
    pub physical_time: f64,
    pub kinetic_j: f64,
    pub max_speed: f64,
    pub minimum_density: f64,
}
#[derive(Clone, Debug, Default, Serialize)]
pub struct Profile {
    pub total_ns: u128,
    pub clone_ns: u128,
    pub topology_ns: u128,
    pub numerical_ns: u128,
    pub validation_ns: u128,
    pub leaves: usize,
    pub faces: usize,
}
#[derive(Debug)]
pub struct Chamber {
    state: State,
    pending: Option<Pending>,
    pub fail_before_commit: bool,
    rejected: Option<State>,
}
impl Payload {
    pub fn conserved(&self, volume: f64, c: f64) -> U {
        match self {
            Self::Fv(u) => *u,
            Self::Lbm(f) => {
                let (rho, u) = moments(f);
                let mut q = [0.; NF];
                q[0] = rho * volume;
                for k in 0..3 {
                    q[k + 1] = rho * u[k] * c * volume;
                }
                q
            }
        }
    }
}
impl State {
    pub fn primitive(&self, key: u64) -> (f64, V, [f64; 4]) {
        let g = mesh::geometry(&self.config, key);
        let u = self.cells[&key].conserved(g.volume(), self.config.h / self.config.dt);
        (
            u[0] / g.volume(),
            std::array::from_fn(|k| u[k + 1] / u[0]),
            std::array::from_fn(|k| u[k + 4] / g.volume()),
        )
    }
    pub fn diagnostics(&self) -> Diagnostics {
        let mut total = [0.; NF];
        let mut kinetic = 0.;
        let mut speed = 0f64;
        let mut minrho = f64::INFINITY;
        for (&key, p) in &self.cells {
            let g = mesh::geometry(&self.config, key);
            let u = p.conserved(g.volume(), self.config.h / self.config.dt);
            for k in 0..NF {
                total[k] += u[k];
            }
            let v: V = std::array::from_fn(|k| u[k + 1] / u[0]);
            kinetic += 0.5 * u[0] * dot(v, v);
            speed = speed.max(norm(v));
            minrho = minrho.min(u[0] / g.volume());
        }
        for b in &self.bodies {
            for k in 0..3 {
                total[k + 1] += b.mass() * b.velocity_m_s[k];
            }
            kinetic += 0.5 * b.mass() * dot(b.velocity_m_s, b.velocity_m_s);
        }
        for s in 0..4 {
            total[s + 4] += self.ledger.wall_scalar[s].iter().sum::<f64>();
        }
        let mut residual = std::array::from_fn(|k| total[k] - self.ledger.initial[k]);
        for k in 0..3 {
            residual[k + 1] -= self.ledger.gravity_impulse[k]
                + self.ledger.walls_impulse[k]
                + self.ledger.hydrostatic_support[k]
                + self.ledger.manufactured_impulse[k];
        }
        Diagnostics {
            total,
            residual,
            leaves: self.cells.len(),
            refined_roots: self.cells.keys().filter(|k| **k & 15 == 1).count(),
            physical_time: self.cycle as f64 * self.config.dt,
            kinetic_j: kinetic,
            max_speed: speed,
            minimum_density: minrho,
        }
    }
    pub fn validate(&self) -> Result<(), MorphError> {
        self.config.validate()?;
        mesh::validate(self)?;
        require(
            self.quiet.is_empty()
                || self
                    .config
                    .indicators
                    .as_ref()
                    .is_some_and(|i| self.quiet.values().all(|v| *v < i.quiet_cycles)),
            "H003 invalid quiet counters",
        )?;
        require(
            self.quiet
                .keys()
                .all(|r| *r < self.config.count() && !self.cells.contains_key(&((*r as u64) << 4))),
            "H003 quiet history ownership",
        )?;
        require(
            self.ledger
                .initial
                .iter()
                .chain(self.ledger.gravity_impulse.iter())
                .chain(self.ledger.walls_impulse.iter())
                .chain(self.ledger.hydrostatic_support.iter())
                .chain(self.ledger.manufactured_impulse.iter())
                .all(|v| v.is_finite())
                && self.ledger.drag_loss.is_finite()
                && self.ledger.body_wall_loss.is_finite(),
            "H003 nonfinite ledger",
        )?;
        require(
            self.cycle <= 10_000_000 && self.bodies.is_empty(),
            "H003 clock/body envelope",
        )?;
        for (&key, p) in &self.cells {
            match p {
                Payload::Fv(u) => {
                    require(
                        self.config.model == Model::FiniteVolume
                            && u.iter().all(|v| v.is_finite())
                            && u[0] > 0.
                            && u[4..].iter().all(|v| *v >= 0.)
                            && u[4 + self.config.species.len()..].iter().all(|v| *v == 0.),
                        "H003 invalid extensive inventory",
                    )?;
                }
                Payload::Lbm(f) => {
                    require(
                        self.config.model == Model::AdaptiveLbm
                            && f.iter().all(|v| v.is_finite() && *v >= 0.),
                        "H003 invalid populations",
                    )?;
                }
            }
            let (rho, u, _) = self.primitive(key);
            require(
                (rho / self.config.density - 1.).abs() <= 0.05
                    && norm(u) / self.config.sound_speed <= 0.1,
                "H003 density/Mach validity",
            )?;
        }
        let d = self.diagnostics();
        require(
            d.total.iter().all(|v| v.is_finite())
                && d.residual[0].abs() <= 1e-14 + 1e-10 * self.ledger.initial[0].abs(),
            "H003 carrier inventory",
        )?;
        for k in 4..NF {
            require(
                d.residual[k].abs() <= 1e-14 + 1e-10 * self.ledger.initial[k].abs(),
                "H003 scalar inventory",
            )?;
        }
        for k in 1..4 {
            require(
                d.residual[k].abs()
                    <= 1e-10
                        + 1e-9
                            * (self.ledger.initial[k].abs()
                                + norm(self.ledger.gravity_impulse)
                                + norm(self.ledger.walls_impulse)
                                + norm(self.ledger.hydrostatic_support)
                                + norm(self.ledger.manufactured_impulse)),
                "H003 momentum inventory",
            )?;
        }
        require(
            self.ledger
                .wall_scalar
                .iter()
                .flatten()
                .all(|v| v.is_finite() && *v >= 0.)
                && self.ledger.drag_loss >= 0.
                && self.ledger.body_wall_loss >= 0.,
            "H003 invalid sink ledger",
        )?;
        Ok(())
    }
    pub fn sample(&self, p: V) -> Result<(f64, V, [f64; 4]), MorphError> {
        let key = mesh::locate(self, p)?;
        Ok(self.primitive(key))
    }
}
impl Chamber {
    pub fn new(
        config: Config,
        initial: impl Fn(mesh::Geometry) -> (f64, V, [f64; 4]),
    ) -> Result<Self, MorphError> {
        config.validate()?;
        let mut state = State {
            config,
            cycle: 0,
            cells: BTreeMap::new(),
            bodies: Vec::new(),
            ledger: Ledger::default(),
            quiet: BTreeMap::new(),
            refinements: 0,
            coarsenings: 0,
        };
        for root in 0..state.config.count() {
            let key = (root as u64) << 4;
            let g = mesh::geometry(&state.config, key);
            let (rho, u, s) = initial(g);
            let payload = match state.config.model {
                Model::FiniteVolume => {
                    let mut q = [0.; NF];
                    q[0] = rho * g.volume();
                    for k in 0..3 {
                        q[k + 1] = q[0] * u[k];
                    }
                    for k in 0..4 {
                        q[k + 4] = s[k] * g.volume();
                    }
                    Payload::Fv(q)
                }
                Model::AdaptiveLbm => {
                    Payload::Lbm(equilibrium(rho, mul(u, state.config.dt / state.config.h)))
                }
            };
            state.cells.insert(key, payload);
        }
        state.ledger.initial = state.diagnostics().total;
        state.validate()?;
        Ok(Self {
            state,
            pending: None,
            fail_before_commit: false,
            rejected: None,
        })
    }
    pub fn state(&self) -> &State {
        &self.state
    }
    pub fn begin_subcycle(&mut self) -> Result<(), MorphError> {
        require(self.pending.is_none(), "H003 already pending")?;
        self.pending = Some(fv::begin(&self.state)?);
        Ok(())
    }
    pub fn finish_subcycle(&mut self) -> Result<(), MorphError> {
        let p = self
            .pending
            .as_ref()
            .ok_or_else(|| error("H003 no pending cycle"))?;
        let mut next = fv::finish(&self.state, p)?;
        if next.config.indicators.is_some() {
            let faces = mesh::faces(&next)?;
            mesh::adapt(&mut next, &faces)?;
        }
        next.validate()?;
        require(!self.fail_before_commit, "H003 injected subcycle refusal")?;
        self.state = next;
        self.pending = None;
        Ok(())
    }
    pub fn pending(&self) -> Option<&Pending> {
        self.pending.as_ref()
    }
    pub fn rejected(&self) -> Option<&State> {
        self.rejected.as_ref()
    }
    pub fn refine(&mut self, roots: &[usize]) -> Result<(), MorphError> {
        require(self.pending.is_none(), "H003 topology during pending cycle")?;
        let mut next = self.state.clone();
        mesh::refine(&mut next, roots)?;
        next.validate()?;
        self.state = next;
        Ok(())
    }
    pub fn coarsen(&mut self, roots: &[usize]) -> Result<(), MorphError> {
        require(self.pending.is_none(), "H003 topology during pending cycle")?;
        let mut next = self.state.clone();
        mesh::coarsen(&mut next, roots)?;
        next.validate()?;
        self.state = next;
        Ok(())
    }
    pub fn step(&mut self) -> Result<Profile, MorphError> {
        self.step_faces(&[], None)
    }
    pub fn step_cached(
        &mut self,
        workspace: &mut topology::Workspace,
    ) -> Result<Profile, MorphError> {
        let start = Instant::now();
        let faces = workspace.faces(&self.state)?;
        self.step_faces(faces, Some(start.elapsed().as_nanos()))
    }
    fn step_faces(
        &mut self,
        provided: &[mesh::Face],
        cache_ns: Option<u128>,
    ) -> Result<Profile, MorphError> {
        require(self.pending.is_none(), "H003 complete pending cycle first")?;
        let start = Instant::now();
        let t = Instant::now();
        let mut next = self.state.clone();
        let mut p = Profile {
            clone_ns: t.elapsed().as_nanos(),
            ..Default::default()
        };
        let t = Instant::now();
        let built;
        let faces = if cache_ns.is_some() {
            provided
        } else {
            built = mesh::faces(&next)?;
            &built
        };
        p.topology_ns = cache_ns.unwrap_or_else(|| t.elapsed().as_nanos());
        p.faces = faces.len();
        let t = Instant::now();
        let result = match next.config.model {
            Model::FiniteVolume => fv::advance(&mut next, faces, self.state.config.dt),
            Model::AdaptiveLbm => lbm::advance(&mut next),
        };
        p.numerical_ns = t.elapsed().as_nanos();
        if let Err(e) = result {
            self.rejected = Some(next);
            return Err(e);
        }
        next.cycle += 1;
        let t = Instant::now();
        if next.config.indicators.is_some()
            && let Err(e) = mesh::adapt(&mut next, faces)
        {
            self.rejected = Some(next);
            return Err(e);
        }
        if let Err(e) = next.validate().and_then(|_| {
            require(
                !self.fail_before_commit,
                "H003 injected transaction refusal",
            )
        }) {
            self.rejected = Some(next);
            return Err(e);
        }
        p.validation_ns = t.elapsed().as_nanos();
        p.leaves = next.cells.len();
        self.state = next;
        self.rejected = None;
        p.total_ns = start.elapsed().as_nanos() + cache_ns.unwrap_or(0);
        Ok(p)
    }
    pub fn digest(&self) -> Result<String, MorphError> {
        let v = serde_json::json!({"state":self.state,"pending":self.pending});
        Ok(
            blake3::hash(&serde_json::to_vec(&encode_bits(v)).map_err(error)?)
                .to_hex()
                .to_string(),
        )
    }
    pub fn save(&self, path: &Path) -> Result<(), MorphError> {
        self.state.validate()?;
        if let Some(p) = &self.pending {
            fv::validate_pending(&self.state, p)?;
        }
        let data = serde_json::json!({"schema":"HABITAT-004/native-f64-v2","source":crate::BENCHMARK_BUILD_IDENTITY,"backend":"CPU-f64","digest":self.digest()?,"state":encode_bits(serde_json::to_value(&self.state).map_err(error)?),"pending":encode_bits(serde_json::to_value(&self.pending).map_err(error)?)});
        crate::write_atomic(path, &serde_json::to_string(&data).map_err(error)?)
    }
    pub fn load(path: &Path) -> Result<Self, MorphError> {
        require(
            std::fs::metadata(path).map_err(error)?.len() <= 256 * 1024 * 1024,
            "H003 checkpoint byte capacity",
        )?;
        let mut v: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path).map_err(error)?).map_err(error)?;
        require(
            v["schema"] == "HABITAT-004/native-f64-v2"
                && v["source"] == crate::BENCHMARK_BUILD_IDENTITY
                && v["backend"] == "CPU-f64",
            "H003 source/backend/schema mismatch",
        )?;
        let expected = v["digest"]
            .as_str()
            .ok_or_else(|| error("H003 missing digest"))?
            .to_string();
        let state: State =
            serde_json::from_value(decode_bits(v["state"].take())?).map_err(error)?;
        let pending: Option<Pending> =
            serde_json::from_value(decode_bits(v["pending"].take())?).map_err(error)?;
        state.validate()?;
        let result = Self {
            state,
            pending,
            fail_before_commit: false,
            rejected: None,
        };
        require(result.digest()? == expected, "H003 payload mismatch")?;
        if let Some(p) = &result.pending {
            fv::validate_pending(&result.state, p)?;
        }
        Ok(result)
    }
}
#[cfg(test)]
mod tests;
