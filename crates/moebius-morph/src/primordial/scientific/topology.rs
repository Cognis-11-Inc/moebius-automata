//! Derived sealed topology, never physical state. Geometry-key equality is exact.
use super::*;
#[derive(Clone, Debug, PartialEq)]
struct Identity {
    keys: Vec<u64>,
    roots: [usize; 3],
    h_bits: u64,
    periodic: bool,
}
impl Identity {
    fn of(s: &State) -> Self {
        Self {
            keys: s.cells.keys().copied().collect(),
            roots: s.config.roots,
            h_bits: s.config.h.to_bits(),
            periodic: s.config.periodic,
        }
    }
}
#[derive(Default, Debug)]
pub struct Workspace {
    identity: Option<Identity>,
    faces: Vec<mesh::Face>,
    pub rebuilds: u64,
    pub hits: u64,
}
impl Workspace {
    pub fn faces<'a>(&'a mut self, s: &State) -> Result<&'a [mesh::Face], MorphError> {
        let key = Identity::of(s);
        if self.identity.as_ref() != Some(&key) {
            let faces = mesh::faces(s)?;
            self.faces = faces;
            self.identity = Some(key);
            self.rebuilds += 1;
        } else {
            self.hits += 1;
        }
        Ok(&self.faces)
    }
    pub fn bytes_estimate(&self) -> usize {
        self.faces.capacity() * std::mem::size_of::<mesh::Face>()
            + self
                .identity
                .as_ref()
                .map_or(0, |k| k.keys.capacity() * std::mem::size_of::<u64>())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn injected_missing_cached_neighbor_refuses_commit() {
        let cfg = Config::default();
        let mut c = Chamber::new(cfg, |_| (20., [0.; 3], [0.; 4])).unwrap();
        let mut w = Workspace::default();
        w.faces(c.state()).unwrap();
        w.faces[0].left = Some(u64::MAX);
        let before = c.digest().unwrap();
        assert!(c.step_cached(&mut w).is_err());
        assert_eq!(c.digest().unwrap(), before);
    }
}
