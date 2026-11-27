//! Sparse block addresses and contiguous local scalar arrays; no material threshold.
use super::*;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockMass {
    pub cells: usize,
    pub edge: usize,
    pub blocks: BTreeMap<usize, Vec<f64>>,
}
impl BlockMass {
    pub fn new(cells: usize, edge: usize) -> Self {
        Self {
            cells,
            edge,
            blocks: BTreeMap::new(),
        }
    }
    pub fn address(&self, i: usize) -> (usize, usize) {
        let n = self.cells;
        let b = self.edge;
        let nb = n.div_ceil(b);
        let p = [i % n, i / n % n, i / (n * n)];
        (
            p[0] / b + nb * (p[1] / b + nb * (p[2] / b)),
            p[0] % b + b * (p[1] % b + b * (p[2] % b)),
        )
    }
    pub fn get(&self, i: usize) -> f64 {
        let (block, offset) = self.address(i);
        self.blocks.get(&block).map_or(0., |v| v[offset])
    }
    pub fn add(&mut self, i: usize, v: f64) {
        if v == 0. {
            return;
        }
        let (block, offset) = self.address(i);
        self.blocks
            .entry(block)
            .or_insert_with(|| vec![0.; self.edge.pow(3)])[offset] += v;
    }
    pub fn nonzero(&self) -> Vec<(usize, f64)> {
        let b = self.edge;
        let nb = self.cells.div_ceil(b);
        let mut out = Vec::new();
        for (&block, v) in &self.blocks {
            let base = [block % nb * b, block / nb % nb * b, block / (nb * nb) * b];
            for (i, &mass) in v.iter().enumerate() {
                if mass != 0. {
                    let p = [base[0] + i % b, base[1] + i / b % b, base[2] + i / (b * b)];
                    if p.iter().all(|&v| v < self.cells) {
                        out.push((p[0] + self.cells * (p[1] + self.cells * p[2]), mass));
                    }
                }
            }
        }
        out.sort_unstable_by_key(|v| v.0);
        out
    }
    pub fn retire_zero(&mut self) -> usize {
        let n = self.blocks.len();
        self.blocks.retain(|_, v| v.iter().any(|v| *v != 0.));
        n - self.blocks.len()
    }
    pub fn validate(&self) -> bool {
        if ![4, 8, 16, 32].contains(&self.edge) || self.cells < 4 || self.cells > 32 {
            return false;
        }
        let b = self.edge;
        let nb = self.cells.div_ceil(b);
        self.blocks.iter().all(|(&key, v)| {
            key < nb.pow(3)
                && v.len() == b.pow(3)
                && v.iter().enumerate().all(|(i, &x)| {
                    let p = [
                        key % nb * b + i % b,
                        key / nb % nb * b + i / b % b,
                        key / (nb * nb) * b + i / (b * b),
                    ];
                    x.is_finite() && x >= 0. && (p.iter().all(|&v| v < self.cells) || x == 0.)
                })
        })
    }
}
