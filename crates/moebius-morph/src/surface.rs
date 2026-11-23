use crate::{MorphError, math::*, require};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Surface {
    pub name: String,
    pub positions: Vec<V>,
    pub normals: Vec<V>,
    pub uvs: Vec<[f32; 2]>,
    pub colors: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeshAudit {
    pub vertices: usize,
    pub triangles: usize,
    pub edges: usize,
    pub minimum_double_area: f32,
    pub signed_volume: f64,
}
impl Surface {
    pub(crate) fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            positions: vec![],
            normals: vec![],
            uvs: vec![],
            colors: vec![],
            indices: vec![],
        }
    }
    pub(crate) fn vertex(&mut self, p: V, uv: [f32; 2], color: V) -> u32 {
        let i = self.positions.len() as u32;
        self.positions.push(p);
        self.uvs.push(uv);
        self.colors.push([color[0], color[1], color[2], 1.]);
        i
    }
    pub(crate) fn triangle(&mut self, a: u32, b: u32, c: u32) {
        self.indices.extend([a, b, c]);
    }
    pub(crate) fn bridge(&mut self, a: u32, b: u32, sides: usize) {
        for j in 0..sides as u32 {
            let k = (j + 1) % sides as u32;
            self.triangle(a + j, a + k, b + j);
            self.triangle(a + k, b + k, b + j);
        }
    }
    pub(crate) fn normals(&mut self) {
        self.normals = vec![[0.; 3]; self.positions.len()];
        for t in self.indices.as_chunks::<3>().0 {
            let a = t[0] as usize;
            let b = t[1] as usize;
            let c = t[2] as usize;
            let n = cross(
                sub(self.positions[b], self.positions[a]),
                sub(self.positions[c], self.positions[a]),
            );
            for i in [a, b, c] {
                self.normals[i] = add(self.normals[i], n);
            }
        }
        for n in &mut self.normals {
            *n = unit(*n);
        }
    }
    /// Every generated component is intended to be a consistently wound closed manifold.
    pub fn audit(&self) -> Result<MeshAudit, MorphError> {
        let n = self.positions.len();
        require(
            n > 3 && self.normals.len() == n && self.uvs.len() == n && self.colors.len() == n,
            "surface attribute lengths",
        )?;
        require(
            self.positions
                .iter()
                .flatten()
                .chain(self.normals.iter().flatten())
                .chain(self.uvs.iter().flatten())
                .chain(self.colors.iter().flatten())
                .all(|v| v.is_finite()),
            "nonfinite surface data",
        )?;
        require(
            self.normals.iter().all(|&v| (length(v) - 1.).abs() < 1e-4),
            "non-unit surface normal",
        )?;
        require(
            self.indices.len().is_multiple_of(3) && self.indices.iter().all(|&i| (i as usize) < n),
            "invalid surface indices",
        )?;
        let mut edges = BTreeMap::<(u32, u32), (usize, i32)>::new();
        let mut minimum = f32::MAX;
        let mut volume = 0f64;
        for tri in self.indices.as_chunks::<3>().0 {
            let [a, b, c] = [
                self.positions[tri[0] as usize],
                self.positions[tri[1] as usize],
                self.positions[tri[2] as usize],
            ];
            let cross_n = cross(sub(b, a), sub(c, a));
            let area = length(cross_n);
            minimum = minimum.min(area);
            require(area > 1e-10, "degenerate triangle")?;
            let average = add(
                add(self.normals[tri[0] as usize], self.normals[tri[1] as usize]),
                self.normals[tri[2] as usize],
            );
            require(
                dot(cross_n, average) > 0.,
                "vertex normal opposes face winding",
            )?;
            volume += dot(a, cross(b, c)) as f64 / 6.;
            for (a, b) in [(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])] {
                let value = edges.entry((a.min(b), a.max(b))).or_default();
                value.0 += 1;
                value.1 += if a < b { 1 } else { -1 };
            }
        }
        require(
            edges
                .values()
                .all(|&(count, winding)| count == 2 && winding == 0),
            "open edge, nonmanifold edge or inconsistent winding",
        )?;
        require(volume > 1e-10, "nonpositive enclosed signed volume")?;
        Ok(MeshAudit {
            vertices: n,
            triangles: self.indices.len() / 3,
            edges: edges.len(),
            minimum_double_area: minimum,
            signed_volume: volume,
        })
    }
    /// Nearest two-sided triangle hit along a normalized ray, useful for audit/picking.
    pub fn ray_distance(&self, origin: V, direction: V) -> Option<f32> {
        let dir = unit(direction);
        let mut nearest = f32::INFINITY;
        for t in self.indices.as_chunks::<3>().0 {
            let a = self.positions[t[0] as usize];
            let e1 = sub(self.positions[t[1] as usize], a);
            let e2 = sub(self.positions[t[2] as usize], a);
            let h = cross(dir, e2);
            let det = dot(e1, h);
            if det.abs() < 1e-9 {
                continue;
            }
            let offset = sub(origin, a);
            let u = dot(offset, h) / det;
            if !(0. ..=1.).contains(&u) {
                continue;
            }
            let q = cross(offset, e1);
            let v = dot(dir, q) / det;
            if v < 0. || u + v > 1.000001 {
                continue;
            }
            let distance = dot(e2, q) / det;
            if distance >= 0. {
                nearest = nearest.min(distance);
            }
        }
        nearest.is_finite().then_some(nearest)
    }
    pub fn digest(&self) -> String {
        let mut h = blake3::Hasher::new();
        h.update(self.name.as_bytes());
        for v in self
            .positions
            .iter()
            .flatten()
            .chain(self.normals.iter().flatten())
            .chain(self.uvs.iter().flatten())
            .chain(self.colors.iter().flatten())
        {
            h.update(&v.to_bits().to_le_bytes());
        }
        for i in &self.indices {
            h.update(&i.to_le_bytes());
        }
        h.finalize().to_hex().to_string()
    }
}
