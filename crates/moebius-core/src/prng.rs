use rand::RngCore;
use rand::SeedableRng;
use rand_chacha::ChaCha12Rng;

use crate::error::Error;
use crate::geom::{Dims, Dir, Site, step_site};

/// Named generator. The crate requirement is rand_chacha 0.9; Cargo.lock holds the patch.
pub const PRNG: &str = "ChaCha12 (12-round IETF ChaCha via rand_chacha 0.9); domain separator blake3 'moebius-automata/prng/v1'";

pub const PLACEMENT_STREAM: &str = "one ChaCha12 stream seeded by blake3(domain || master_seed || label=placement || tick=0 || cell_id=0); Fisher-Yates then optional genome bytes";

pub const MOVEMENT_STREAM: &str = "independent ChaCha12 stream per (master_seed, tick, cell_id) labeled 'movement'; first unbiased residue modulo 7; call order cannot change another cell's draw";

pub fn domain_seed(master: u64, label: &[u8], tick: u64, cell_id: u64) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"moebius-automata/prng/v1");
    hasher.update(&master.to_le_bytes());
    hasher.update(&(label.len() as u64).to_le_bytes());
    hasher.update(label);
    hasher.update(&tick.to_le_bytes());
    hasher.update(&cell_id.to_le_bytes());
    *hasher.finalize().as_bytes()
}

/// Uniform in `0..n` by rejection sampling. `n` must be non-zero.
pub fn unbiased_below(rng: &mut ChaCha12Rng, n: u32) -> u32 {
    debug_assert!(n > 0);
    let bound = u32::MAX - (u32::MAX % n);
    loop {
        let draw = rng.next_u32();
        if draw < bound {
            return draw % n;
        }
    }
}

pub fn movement_dir(master_seed: u64, tick: u64, cell_id: u64) -> Result<Dir, Error> {
    let mut rng = ChaCha12Rng::from_seed(domain_seed(master_seed, b"movement", tick, cell_id));
    Dir::from_roll(unbiased_below(&mut rng, 7))
}

pub fn movement_target(
    master_seed: u64,
    tick: u64,
    cell_id: u64,
    from: Site,
    dims: Dims,
) -> Result<(Dir, Site), Error> {
    let dir = movement_dir(master_seed, tick, cell_id)?;
    let to = step_site(from, dir, dims)?;
    Ok((dir, to))
}

pub fn placement_rng(master_seed: u64) -> ChaCha12Rng {
    ChaCha12Rng::from_seed(domain_seed(master_seed, b"placement", 0, 0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn movement_streams_do_not_depend_on_call_order() {
        let first = movement_dir(9, 3, 1).unwrap();
        let second = movement_dir(9, 3, 2).unwrap();
        let second_again = movement_dir(9, 3, 2).unwrap();
        let first_again = movement_dir(9, 3, 1).unwrap();
        assert_eq!(first, first_again);
        assert_eq!(second, second_again);
    }
}
