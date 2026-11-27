//! HABITAT-002 execution candidates. Workspace is derived scratch, never authority.
pub use super::transport::Attribution;
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Lane {
    Reference,
    Buffered,
    Blocks,
    ImplicitCarrier,
    InPlace,
}
#[derive(Clone, Debug, Default, Serialize)]
pub struct Activity {
    pub scalar_nonzero: usize,
    pub scalar_blocks: usize,
    pub scalar_slots: usize,
    pub active_carrier_blocks: usize,
    pub implicit_carrier_cells: usize,
    pub activated: usize,
    pub retired: usize,
    pub scratch_bytes: usize,
}
#[derive(Default, Debug)]
pub struct Workspace {
    tentative: Option<State>,
    streamed: Vec<[f64; 19]>,
    post_buffer: Vec<[f64; 19]>,
    pub workers: usize,
    carrier_keys: Vec<usize>,
    scalar_keys: BTreeSet<(usize, usize)>,
    background: Option<(Config, u64, [f64; 19])>,
    destinations: Vec<[u32; 19]>,
    visited: Vec<bool>,
    config: Option<Config>,
    velocities: Vec<V>,
    delta: Vec<f64>,
    outgoing: Vec<f64>,
    faces: Vec<(usize, usize)>,
    pub activity: Activity,
    /// A bounded experimental refusal, checked before staged allocation/commit.
    pub max_cells: Option<usize>,
    pub fail_before_commit: bool,
}
fn copy_state(dst: &mut State, src: &State) {
    dst.config = src.config.clone();
    dst.tick = src.tick;
    dst.populations.clone_from(&src.populations);
    dst.spheres.clone_from(&src.spheres);
    dst.impulses = src.impulses.clone();
    dst.initial_momentum = src.initial_momentum;
    dst.carrier_initial_kg = src.carrier_initial_kg;
    dst.sealed = src.sealed;
    if dst.fields.len() != src.fields.len() {
        dst.fields = src.fields.clone();
        return;
    }
    for (a, b) in dst.fields.iter_mut().zip(&src.fields) {
        a.species.clone_from(&b.species);
        a.wall_kg = b.wall_kg;
        a.initial_kg = b.initial_kg;
        match (&mut a.mass, &b.mass) {
            (Mass::Dense(a), Mass::Dense(b)) => a.clone_from(b),
            _ => a.mass = b.mass.clone(),
        }
    }
}
impl Chamber {
    pub fn step_candidate(
        &mut self,
        lane: Lane,
        w: &mut Workspace,
    ) -> Result<StepProfile, MorphError> {
        if lane == Lane::Reference {
            return self.step();
        }
        require(w.workers <= 8, "H002 worker budget")?;
        require(
            w.max_cells
                .is_none_or(|cap| self.state.config.count() <= cap),
            "H002 scratch cell capacity",
        )?;
        let started = Instant::now();
        let t = Instant::now();
        let mut next = if let Some(mut next) = w.tentative.take() {
            copy_state(&mut next, &self.state);
            next
        } else {
            self.state.clone()
        };
        let mut p = StepProfile {
            clone_ns: t.elapsed().as_nanos(),
            ..Default::default()
        };
        let result = (|| {
            let t = Instant::now();
            if !next.sealed {
                next.initial_momentum = next.diagnostics().momentum;
                next.sealed = true;
            }
            next.validate()?;
            p.validation_ns += t.elapsed().as_nanos();
            w.prepare(&next.config);
            for substep in 0..next.config.substeps {
                let t = Instant::now();
                couple(&mut next)?;
                p.coupling_ns += t.elapsed().as_nanos();
                let t = Instant::now();
                if lane == Lane::ImplicitCarrier {
                    w.implicit_carrier(&mut next, substep)?;
                } else if lane == Lane::InPlace {
                    w.in_place(&mut next)?;
                } else {
                    w.carrier(&mut next)?;
                }
                p.carrier_ns += t.elapsed().as_nanos();
                let t = Instant::now();
                p.transported_faces += w.transport(&mut next)?;
                p.transport_ns += t.elapsed().as_nanos();
            }
            next.tick = next
                .tick
                .checked_add(1)
                .ok_or_else(|| error("H002 clock overflow"))?;
            let t = Instant::now();
            next.validate()?;
            p.validation_ns += t.elapsed().as_nanos();
            require(!w.fail_before_commit, "H002 injected backend refusal")?;
            Ok(())
        })();
        match result {
            Ok(()) => {
                std::mem::swap(&mut self.state, &mut next);
                w.tentative = Some(next);
                w.observe(&self.state);
                p.total_ns = started.elapsed().as_nanos();
                Ok(p)
            }
            Err(e) => {
                w.tentative = Some(next);
                Err(e)
            }
        }
    }
    /// Read-only decoded physical equivalence; representation tags excluded.
    pub fn physical_eq(&self, other: &Self) -> bool {
        let a = &self.state;
        let b = &other.state;
        let mut cfg = a.config.clone();
        cfg.scalar_backend = b.config.scalar_backend;
        cfg == b.config
            && a.tick == b.tick
            && a.sealed == b.sealed
            && a.populations.iter().flatten().map(|v| v.to_bits()).eq(b
                .populations
                .iter()
                .flatten()
                .map(|v| v.to_bits()))
            && a.spheres == b.spheres
            && a.impulses == b.impulses
            && a.initial_momentum == b.initial_momentum
            && a.carrier_initial_kg.to_bits() == b.carrier_initial_kg.to_bits()
            && a.fields.len() == b.fields.len()
            && a.fields.iter().zip(&b.fields).all(|(a, b)| {
                a.species == b.species
                    && a.wall_kg == b.wall_kg
                    && a.initial_kg.to_bits() == b.initial_kg.to_bits()
                    && (0..cfg.count()).all(|i| a.mass.get(i).to_bits() == b.mass.get(i).to_bits())
            })
    }
    pub fn use_blocks(&mut self, edge: usize) -> Result<(), MorphError> {
        require([4, 8, 16, 32].contains(&edge), "H002 block edge")?;
        let mut next = self.state.clone();
        for f in &mut next.fields {
            let mut b = BlockMass::new(next.config.cells, edge);
            for (i, v) in f.mass.nonzero() {
                b.add(i, v);
            }
            f.mass = Mass::Blocks(b);
        }
        next.config.scalar_backend = ScalarBackend::DenseBlocks { edge };
        next.validate()?;
        self.state = next;
        Ok(())
    }
}
impl Workspace {
    pub fn with_workers(workers: usize) -> Self {
        Self {
            workers,
            ..Default::default()
        }
    }
    fn prepare(&mut self, c: &Config) {
        if self.config.as_ref() == Some(c) {
            return;
        }
        let n = c.cells as i32;
        self.destinations = (0..c.count())
            .map(|i| {
                std::array::from_fn(|q| {
                    let p = c.coordinates(i);
                    let at: [i32; 3] = std::array::from_fn(|k| p[k] as i32 + C[q][k]);
                    let wall =
                        c.boundary == Boundary::Sealed && at.iter().any(|&x| x < 0 || x >= n);
                    if wall {
                        (i as u32) | (1 << 31)
                    } else {
                        c.index(at.map(|v| v.rem_euclid(n) as usize)) as u32
                    }
                })
            })
            .collect();

        self.velocities.resize(c.count(), [0.; 3]);
        self.delta.resize(c.count(), 0.);
        self.outgoing.resize(c.count(), 0.);
        self.config = Some(c.clone());
    }
    fn carrier(&mut self, state: &mut State) -> Result<(), MorphError> {
        if state.populations.is_empty() {
            return Ok(());
        }
        self.streamed.resize(state.config.count(), [0.; 19]);
        self.streamed.fill([0.; 19]);
        let tau = state.config.tau();
        let scale = state.config.h().powi(4) / state.config.dt();
        if self.workers > 1 {
            self.post_buffer.resize(state.config.count(), [0.; 19]);
            let chunk = state.config.count().div_ceil(self.workers);
            std::thread::scope(|scope| -> Result<(), MorphError> {
                let mut handles = Vec::new();
                for (out, input) in self
                    .post_buffer
                    .chunks_mut(chunk)
                    .zip(state.populations.chunks(chunk))
                {
                    handles.push(scope.spawn(move || -> Result<(), MorphError> {
                        for (post, f) in out.iter_mut().zip(input) {
                            let (rho, u) = moments(f);
                            let eq = equilibrium(rho, u);
                            for q in 0..19 {
                                post[q] = f[q] + (eq[q] - f[q]) / tau;
                                require(
                                    post[q].is_finite() && post[q] >= 0.,
                                    "H001 unstable BGK collision",
                                )?;
                            }
                        }
                        Ok(())
                    }));
                }
                for h in handles {
                    h.join().map_err(|_| error("H002 worker failure"))??;
                }
                Ok(())
            })?;
        }
        for (i, f) in state.populations.iter().enumerate() {
            let eq = if self.workers <= 1 {
                let (rho, u) = moments(f);
                equilibrium(rho, u)
            } else {
                [0.; 19]
            };
            for q in 0..19 {
                let post = if self.workers > 1 {
                    self.post_buffer[i][q]
                } else {
                    f[q] + (eq[q] - f[q]) / tau
                };
                require(
                    post.is_finite() && post >= 0.,
                    "H001 unstable BGK collision",
                )?;
                let (dest, wall) = self.destination(i, q);
                if wall {
                    self.streamed[dest][OPP[q]] += post;
                    state.impulses.walls_kg_m_s = sub(
                        state.impulses.walls_kg_m_s,
                        mul(C[q].map(f64::from), 2. * post * scale),
                    );
                } else {
                    self.streamed[dest][q] += post;
                }
            }
        }
        std::mem::swap(&mut state.populations, &mut self.streamed);
        Ok(())
    }
    fn transport(&mut self, state: &mut State) -> Result<usize, MorphError> {
        if state.fields.is_empty() {
            return Ok(0);
        }
        let c = &state.config;
        let dt_h = c.dt() / c.h();
        for (i, v) in self.velocities.iter_mut().enumerate() {
            *v = state.sample_index(i).velocity_m_s;
        }
        let mut count = 0;
        for f in &mut state.fields {
            self.delta.fill(0.);
            self.outgoing.fill(0.);
            self.faces.clear();
            if matches!(f.mass, Mass::Dense(_)) {
                self.faces
                    .extend((0..c.count()).flat_map(|i| (0..3).map(move |a| (i, a))));
            } else {
                for (i, _) in f.mass.nonzero() {
                    for a in 0..3 {
                        self.faces.push((i, a));
                        if let Some(j) = transport::neighbour(c, i, a, -1) {
                            self.faces.push((j, a));
                        }
                    }
                }
                self.faces.sort_unstable();
                self.faces.dedup();
            }
            let slip = f.species.slip(c);
            let diffusion = f.species.diffusivity_m2_s / c.h();
            for &(i, a) in &self.faces {
                if let Some(j) = transport::neighbour(c, i, a, 1) {
                    count += 1;
                    let speed = 0.5 * (self.velocities[i][a] + self.velocities[j][a]) + slip[a];
                    let left = f.mass.get(i);
                    let right = f.mass.get(j);
                    let flux = dt_h
                        * (speed.max(0.) * left
                            + speed.min(0.) * right
                            + diffusion * (left - right));
                    if flux != 0. {
                        self.delta[i] += -flux;
                        self.delta[j] += flux;
                    }
                    if left > 0. {
                        self.outgoing[i] += dt_h * (speed.max(0.) + diffusion);
                    }
                    if right > 0. {
                        self.outgoing[j] += dt_h * ((-speed).max(0.) + diffusion);
                    }
                }
            }
            if c.boundary == Boundary::Sealed {
                for (i, mass) in f.mass.nonzero() {
                    let p = c.coordinates(i);
                    for a in 0..3 {
                        for side in 0..2 {
                            if (side == 0 && p[a] == 0) || (side == 1 && p[a] + 1 == c.cells) {
                                let fraction =
                                    dt_h * (slip[a] * if side == 0 { -1. } else { 1. }).max(0.);
                                let amount = fraction * mass;
                                self.outgoing[i] += fraction;
                                if amount != 0. {
                                    self.delta[i] += -amount;
                                    f.wall_kg[a * 2 + side] += amount;
                                }
                            }
                        }
                    }
                }
            }
            require(
                self.outgoing.iter().all(|v| v.is_finite() && *v <= 0.9),
                "H001 scalar positivity CFL exceeded",
            )?;
            for (i, &d) in self.delta.iter().enumerate() {
                if d != 0. {
                    match &mut f.mass {
                        Mass::Dense(v) => v[i] += d,
                        Mass::Sparse(v) => {
                            let next = v.get(&i).copied().unwrap_or(0.) + d;
                            if next == 0. {
                                v.remove(&i);
                            } else {
                                v.insert(i, next);
                            }
                        }
                        Mass::Blocks(v) => v.add(i, d),
                    }
                }
            }
            if let Mass::Blocks(b) = &mut f.mass {
                self.activity.retired += b.retire_zero();
            }
        }
        Ok(count)
    }
    fn observe(&mut self, state: &State) {
        let keys = state
            .fields
            .iter()
            .enumerate()
            .flat_map(|(i, f)| {
                if let Mass::Blocks(b) = &f.mass {
                    b.blocks.keys().map(|&k| (i, k)).collect::<Vec<_>>()
                } else {
                    Vec::new()
                }
            })
            .collect::<BTreeSet<_>>();
        self.activity.activated = keys.difference(&self.scalar_keys).count();
        self.activity.retired = keys
            .symmetric_difference(&self.scalar_keys)
            .filter(|k| self.scalar_keys.contains(k))
            .count();
        self.scalar_keys = keys;
        self.activity.scalar_nonzero = state.fields.iter().map(|f| f.mass.nonzero().len()).sum();
        self.activity.scalar_blocks = state
            .fields
            .iter()
            .map(|f| {
                if let Mass::Blocks(b) = &f.mass {
                    b.blocks.len()
                } else {
                    0
                }
            })
            .sum();
        self.activity.scalar_slots = state.fields.iter().map(|f| f.mass.stored()).sum();
        self.activity.scratch_bytes = (self.streamed.capacity() + self.post_buffer.capacity())
            * 152
            + self.destinations.capacity() * 19 * std::mem::size_of::<u32>()
            + self.visited.capacity()
            + self.velocities.capacity() * 24
            + (self.delta.capacity() + self.outgoing.capacity()) * 8
            + self.faces.capacity() * 16;
        // Includes carrier+field tentative state separately in reports; this is only named buffers.
    }
}

impl Workspace {
    /// Encoded carrier kernel experiment. Dense State decode/validation remains;
    /// this adapter is NOT a qualified globally sparse atmosphere.
    fn implicit_carrier(&mut self, state: &mut State, substep: usize) -> Result<(), MorphError> {
        if state.populations.is_empty() {
            return Ok(());
        }
        let c = &state.config;
        let edge = 4;
        let layout = BlockMass::new(c.cells, edge);
        let nb = c.cells.div_ceil(edge);
        let collide = |f: &[f64; 19]| -> Result<[f64; 19], MorphError> {
            let (rho, u) = moments(f);
            let eq = equilibrium(rho, u);
            let out = std::array::from_fn(|q| f[q] + (eq[q] - f[q]) / c.tau());
            require(
                out.iter().all(|x| x.is_finite() && *x >= 0.),
                "H001 unstable BGK collision",
            )?;
            Ok(out)
        };
        // The floating-point equilibrium is not a bitwise BGK fixed point.
        // Implicit cells advance the same homogeneous recurrence at every substep.
        let micro = state.tick * c.substeps as u64 + substep as u64;
        let (start, mut bg) = match &self.background {
            Some((config, t, bg)) if config == c && *t <= micro => (*t, *bg),
            _ => (0, equilibrium(c.density_kg_m3, [0.; 3])),
        };
        for _ in start..micro {
            bg = collide(&bg)?;
        }
        let new_bg = collide(&bg)?;
        self.background = Some((c.clone(), micro + 1, new_bg));
        let mut active = BTreeSet::new();
        for (i, f) in state.populations.iter().enumerate() {
            if !f.iter().zip(bg).all(|(a, b)| a.to_bits() == b.to_bits()) {
                active.insert(layout.address(i).0);
            }
        }
        let previous = active.len();
        let mut needed = active.clone();
        // One lattice link per substep. D3Q19 includes face diagonals, no body
        // diagonals. Block-level ±1 stencil is conservative (overactivation).
        for &key in &active {
            let p = [key % nb, key / nb % nb, key / (nb * nb)];
            for v in C {
                let at: [i32; 3] = std::array::from_fn(|k| p[k] as i32 + v[k]);
                if c.boundary == Boundary::Sealed && at.iter().any(|&v| v < 0 || v >= nb as i32) {
                    continue;
                }
                let a = at.map(|v| v.rem_euclid(nb as i32) as usize);
                needed.insert(a[0] + nb * (a[1] + nb * a[2]));
            }
        }
        let mut posts = BTreeMap::new();
        for &key in &active {
            let mut block = vec![new_bg; edge.pow(3)];
            for (offset, f) in block.iter_mut().enumerate() {
                let p = [
                    key % nb * edge + offset % edge,
                    key / nb % nb * edge + offset / edge % edge,
                    key / (nb * nb) * edge + offset / (edge * edge),
                ];
                if p.iter().all(|&x| x < c.cells) {
                    *f = collide(&state.populations[c.index(p)])?;
                }
            }
            posts.insert(key, block);
        }
        let post_at = |i: usize| {
            let (key, offset) = layout.address(i);
            posts.get(&key).map_or(new_bg, |v| v[offset])
        };
        let mut output = BTreeMap::new();
        for &key in &needed {
            let mut block = vec![new_bg; edge.pow(3)];
            for (offset, f) in block.iter_mut().enumerate() {
                let p = [
                    key % nb * edge + offset % edge,
                    key / nb % nb * edge + offset / edge % edge,
                    key / (nb * nb) * edge + offset / (edge * edge),
                ];
                if p.iter().any(|&v| v >= c.cells) {
                    continue;
                }
                let i = c.index(p);
                for q in 0..19 {
                    let src: [i32; 3] = std::array::from_fn(|k| p[k] as i32 - C[q][k]);
                    *f.get_mut(q).unwrap() = if c.boundary == Boundary::Sealed
                        && src.iter().any(|&v| v < 0 || v >= c.cells as i32)
                    {
                        post_at(i)[OPP[q]]
                    } else {
                        post_at(c.index(src.map(|v| v.rem_euclid(c.cells as i32) as usize)))[q]
                    };
                }
            }
            if block
                .iter()
                .flatten()
                .zip(std::iter::repeat_n(new_bg, edge.pow(3)).flatten())
                .any(|(a, b)| a.to_bits() != b.to_bits())
            {
                output.insert(key, block);
            }
        }
        // Preserve the original wall floating-point accumulation order, including
        // implicit equilibrium pressure. Skipping quiescent walls is NOT exact.
        let scale = c.h().powi(4) / c.dt();
        for i in 0..c.count() {
            for (q, direction) in C.iter().enumerate() {
                if self.destination(i, q).1 {
                    state.impulses.walls_kg_m_s = sub(
                        state.impulses.walls_kg_m_s,
                        mul(direction.map(f64::from), 2. * post_at(i)[q] * scale),
                    );
                }
            }
        }
        for (i, f) in state.populations.iter_mut().enumerate() {
            let (key, offset) = layout.address(i);
            *f = output.get(&key).map_or(new_bg, |v| v[offset]);
        }
        self.carrier_keys = output.keys().copied().collect();
        self.activity.active_carrier_blocks = output.len();
        self.activity.implicit_carrier_cells = c.count().saturating_sub(output.len() * edge.pow(3));
        self.activity.retired = previous.saturating_sub(output.len());
        Ok(())
    }
}

impl Chamber {
    pub fn transport_attribution(&self) -> Result<Attribution, MorphError> {
        let mut state = self.state.clone();
        let mut a = Attribution::default();
        transport::attributed(&mut state, &mut a)?;
        Ok(a)
    }
}

impl Workspace {
    pub fn carrier_blocks(&self) -> &[usize] {
        &self.carrier_keys
    }
}
/// Controlled nonbiological compact clouds; initialization shared with Witness.
pub fn cloud(
    config: Config,
    fraction: f64,
    regions: usize,
    bodies: usize,
    diffusion: f64,
    species_count: usize,
    speed: f64,
) -> Result<Chamber, MorphError> {
    require(
        (0. ..=1.).contains(&fraction) && regions <= 16 && bodies <= 256 && species_count <= 4,
        "H002 initialization budget",
    )?;
    let n = config.cells;
    let mut c = Chamber::new(config.clone())?;
    let side = ((n as f64 * fraction.cbrt()).round() as usize).max(usize::from(fraction > 0.));
    let selected = |p: V| {
        if fraction == 0. {
            return false;
        }
        let xyz = p.map(|x| (x / config.h()) as usize);
        (0..regions).any(|r| {
            let offset = (r * 7 + 2) % n;
            xyz.iter().all(|v| (*v + n - offset) % n < side)
        })
    };
    c.initialize_velocity(|p| {
        if selected(p) {
            [speed, speed * 0.5, 0.]
        } else {
            [0.; 3]
        }
    })?;
    for species_index in 0..species_count {
        c.add_species(
            Species {
                material: format!("finite cloud {species_index}"),
                radius_m: 0.001,
                density_kg_m3: 20.,
                diffusivity_m2_s: diffusion,
            },
            |p| {
                if selected(p) {
                    1e-9 * config.h().powi(3)
                } else {
                    0.
                }
            },
        )?;
    }
    for i in 0..bodies {
        c.add_sphere(Sphere {
            id: i as u64 + 1,
            radius_m: config.h() / 16.,
            density_kg_m3: 80.,
            position_m: [config.length_m * (0.2 + 0.002 * i as f64); 3],
            velocity_m_s: [0.001, 0., 0.],
        })?;
    }
    c.seal()?;
    Ok(c)
}

impl Workspace {
    fn destination(&self, i: usize, q: usize) -> (usize, bool) {
        let a = self.destinations[i][q];
        ((a & 0x7fff_ffff) as usize, a >> 31 != 0)
    }
    fn in_place(&mut self, state: &mut State) -> Result<(), MorphError> {
        if state.populations.is_empty() {
            return Ok(());
        }
        let tau = state.config.tau();
        let scale = state.config.h().powi(4) / state.config.dt();
        for (i, f) in state.populations.iter_mut().enumerate() {
            let (rho, u) = moments(f);
            let eq = equilibrium(rho, u);
            for q in 0..19 {
                let post = f[q] + (eq[q] - f[q]) / tau;
                require(
                    post.is_finite() && post >= 0.,
                    "H001 unstable BGK collision",
                )?;
                f[q] = post;
                if self.destination(i, q).1 {
                    state.impulses.walls_kg_m_s = sub(
                        state.impulses.walls_kg_m_s,
                        mul(C[q].map(f64::from), 2. * post * scale),
                    );
                }
            }
        }
        self.visited.resize(state.config.count() * 19, false);
        self.visited.fill(false);
        for start in 0..self.visited.len() {
            if self.visited[start] {
                continue;
            }
            let mut at = start;
            let mut carry = state.populations[start / 19][start % 19];
            loop {
                let i = at / 19;
                let q = at % 19;
                let (dest, wall) = self.destination(i, q);
                let slot = dest * 19 + if wall { OPP[q] } else { q };
                let next = state.populations[slot / 19][slot % 19];
                state.populations[slot / 19][slot % 19] = 0. + carry;
                carry = next;
                self.visited[at] = true;
                at = slot;
                if at == start {
                    break;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "scalable_tests.rs"]
mod tests;
