use crate::config::{MAX_AXIS, MAX_VOLUME, SimConfig, config_hash_bytes};
use crate::error::Error;
use crate::geom::{Dims, Site, site_index};
use crate::hashutil::{hex32, parse_hex32};
use crate::world::{Cell, DeathRecord, World};

pub const SNAP_MAGIC: &[u8; 8] = b"MAS1SNAP";
pub const SNAP_VERSION: u32 = 1;
pub const STATE_FORMAT: &str = "canonical-le-v1";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub tick: u64,
    pub seed: u64,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub reservoir: u64,
    pub waste: u64,
    pub external_input: u64,
    pub external_output: u64,
    pub energy_credited: u64,
    pub energy_maintenance: u64,
    pub energy_disposed: u64,
    pub material_initial: u64,
    pub next_cell_id: u64,
    pub inject_cursor: u64,
    pub cells: Vec<Cell>,
    pub deaths: Vec<DeathRecord>,
    pub nutrient: Vec<u64>,
}

impl Snapshot {
    pub fn dims(&self) -> Dims {
        Dims {
            width: self.width,
            height: self.height,
            depth: self.depth,
        }
    }

    pub fn try_from_parts(mut parts: Snapshot) -> Result<Self, Error> {
        if parts.width == 0
            || parts.height == 0
            || parts.depth == 0
            || parts.width > MAX_AXIS
            || parts.height > MAX_AXIS
            || parts.depth > MAX_AXIS
        {
            return Err(Error::Snapshot(format!(
                "snapshot grid {}×{}×{} is outside 1..={MAX_AXIS}",
                parts.width, parts.height, parts.depth
            )));
        }
        let volume = parts.dims().volume()?;
        if volume > MAX_VOLUME {
            return Err(Error::Snapshot(format!(
                "snapshot volume {volume} exceeds {MAX_VOLUME}"
            )));
        }
        if parts.nutrient.len() as u64 != volume {
            return Err(Error::Snapshot(format!(
                "nutrient entries {} != volume {volume}",
                parts.nutrient.len()
            )));
        }
        if parts.inject_cursor >= volume {
            return Err(Error::Snapshot(format!(
                "inject cursor {} is outside volume {volume}",
                parts.inject_cursor
            )));
        }
        parts.cells.sort_by_key(|cell| cell.id);
        parts.deaths.sort_by_key(|death| death.id);
        let mut used_sites = std::collections::BTreeSet::new();
        let mut max_id = 0u64;
        for cell in &parts.cells {
            if cell.id == 0 || !used_sites.insert(cell.site) {
                return Err(Error::Snapshot(format!(
                    "illegal live cell {} at {:?}",
                    cell.id, cell.site
                )));
            }
            site_index(cell.site, parts.dims())?;
            max_id = max_id.max(cell.id);
        }
        for pair in parts.cells.windows(2) {
            if pair[0].id == pair[1].id {
                return Err(Error::Snapshot(format!("duplicate cell id {}", pair[0].id)));
            }
        }
        for death in &parts.deaths {
            if death.id == 0 {
                return Err(Error::Snapshot("death id 0 is illegal".into()));
            }
            if parts.cells.iter().any(|cell| cell.id == death.id) {
                return Err(Error::Snapshot(format!(
                    "id {} is both live and dead",
                    death.id
                )));
            }
            site_index(death.site, parts.dims())?;
            max_id = max_id.max(death.id);
        }
        for pair in parts.deaths.windows(2) {
            if pair[0].id == pair[1].id {
                return Err(Error::Snapshot(format!(
                    "duplicate death id {}",
                    pair[0].id
                )));
            }
        }
        if parts.next_cell_id <= max_id {
            return Err(Error::Snapshot(format!(
                "next_cell_id {} is not above max id {max_id}",
                parts.next_cell_id
            )));
        }
        Ok(parts)
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        push_u64(&mut out, self.tick);
        push_u64(&mut out, self.seed);
        push_u32(&mut out, self.width);
        push_u32(&mut out, self.height);
        push_u32(&mut out, self.depth);
        push_u64(&mut out, self.reservoir);
        push_u64(&mut out, self.waste);
        push_u64(&mut out, self.external_input);
        push_u64(&mut out, self.external_output);
        push_u64(&mut out, self.energy_credited);
        push_u64(&mut out, self.energy_maintenance);
        push_u64(&mut out, self.energy_disposed);
        push_u64(&mut out, self.material_initial);
        push_u64(&mut out, self.next_cell_id);
        push_u64(&mut out, self.inject_cursor);
        push_u32(&mut out, self.cells.len() as u32);
        for cell in &self.cells {
            push_u64(&mut out, cell.id);
            push_u32(&mut out, cell.site.x);
            push_u32(&mut out, cell.site.y);
            push_u32(&mut out, cell.site.z);
            push_u64(&mut out, cell.energy);
            push_u64(&mut out, cell.age);
            out.extend_from_slice(&cell.genome);
        }
        push_u32(&mut out, self.deaths.len() as u32);
        for death in &self.deaths {
            push_u64(&mut out, death.id);
            push_u32(&mut out, death.site.x);
            push_u32(&mut out, death.site.y);
            push_u32(&mut out, death.site.z);
            push_u64(&mut out, death.energy_disposed);
            push_u64(&mut out, death.age);
            push_u64(&mut out, death.during_tick);
            out.extend_from_slice(&death.genome);
        }
        push_u32(&mut out, self.nutrient.len() as u32);
        for amount in &self.nutrient {
            push_u64(&mut out, *amount);
        }
        out
    }

    pub fn state_hash_bytes(&self) -> [u8; 32] {
        *blake3::hash(&self.canonical_bytes()).as_bytes()
    }

    pub fn state_hash(&self) -> String {
        hex32(&self.state_hash_bytes())
    }
}

pub struct SnapshotFile {
    pub config_hash: [u8; 32],
    pub snapshot: Snapshot,
}

impl SnapshotFile {
    pub fn encode(config: &SimConfig, world: &World) -> Result<Vec<u8>, Error> {
        let snapshot = world.snapshot();
        let canonical = snapshot.canonical_bytes();
        let state_hash = *blake3::hash(&canonical).as_bytes();
        let config_hash = config_hash_bytes(config)?;
        let mut out = Vec::with_capacity(8 + 4 + 32 + 32 + 4 + canonical.len());
        out.extend_from_slice(SNAP_MAGIC);
        push_u32(&mut out, SNAP_VERSION);
        out.extend_from_slice(&state_hash);
        out.extend_from_slice(&config_hash);
        push_u32(&mut out, canonical.len() as u32);
        out.extend_from_slice(&canonical);
        Ok(out)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() < 8 + 4 + 32 + 32 + 4 {
            return Err(Error::Snapshot("file is shorter than the header".into()));
        }
        if &bytes[..8] != SNAP_MAGIC.as_slice() {
            return Err(Error::Snapshot("bad snapshot magic".into()));
        }
        let mut cursor = 8usize;
        let version = read_u32(bytes, &mut cursor)?;
        if version != SNAP_VERSION {
            return Err(Error::Snapshot(format!(
                "unsupported snapshot version {version}"
            )));
        }
        let mut stated_hash = [0u8; 32];
        stated_hash.copy_from_slice(&bytes[cursor..cursor + 32]);
        cursor += 32;
        let mut config_hash = [0u8; 32];
        config_hash.copy_from_slice(&bytes[cursor..cursor + 32]);
        cursor += 32;
        let length = read_u32(bytes, &mut cursor)? as usize;
        if bytes.len() != cursor + length {
            return Err(Error::Snapshot(format!(
                "snapshot length {length} does not match trailing bytes {}",
                bytes.len().saturating_sub(cursor)
            )));
        }
        let payload = &bytes[cursor..];
        let actual = *blake3::hash(payload).as_bytes();
        if actual != stated_hash {
            return Err(Error::Snapshot(
                "stored state hash does not match payload".into(),
            ));
        }
        let snapshot = decode_canonical(payload)?;
        if snapshot.state_hash_bytes() != stated_hash {
            return Err(Error::Snapshot(
                "re-encoded snapshot hash does not match payload".into(),
            ));
        }
        Ok(Self {
            config_hash,
            snapshot,
        })
    }

    pub fn config_hash_hex(&self) -> String {
        hex32(&self.config_hash)
    }
}

pub fn audit_with_config(config: &SimConfig, file: &SnapshotFile) -> Result<(), Error> {
    let expected = config_hash_bytes(config)?;
    if file.config_hash != expected {
        return Err(Error::Snapshot(format!(
            "snapshot config hash {} != supplied config {}",
            hex32(&file.config_hash),
            hex32(&expected)
        )));
    }
    let _world = World::restore(config, &file.snapshot)?;
    Ok(())
}

pub fn state_hash_of(world: &World) -> String {
    world.snapshot().state_hash()
}

fn decode_canonical(bytes: &[u8]) -> Result<Snapshot, Error> {
    let mut cursor = 0usize;
    let tick = read_u64(bytes, &mut cursor)?;
    let seed = read_u64(bytes, &mut cursor)?;
    let width = read_u32(bytes, &mut cursor)?;
    let height = read_u32(bytes, &mut cursor)?;
    let depth = read_u32(bytes, &mut cursor)?;
    let reservoir = read_u64(bytes, &mut cursor)?;
    let waste = read_u64(bytes, &mut cursor)?;
    let external_input = read_u64(bytes, &mut cursor)?;
    let external_output = read_u64(bytes, &mut cursor)?;
    let energy_credited = read_u64(bytes, &mut cursor)?;
    let energy_maintenance = read_u64(bytes, &mut cursor)?;
    let energy_disposed = read_u64(bytes, &mut cursor)?;
    let material_initial = read_u64(bytes, &mut cursor)?;
    let next_cell_id = read_u64(bytes, &mut cursor)?;
    let inject_cursor = read_u64(bytes, &mut cursor)?;
    let cell_count = read_u32(bytes, &mut cursor)? as usize;
    if cell_count as u64 > MAX_VOLUME {
        return Err(Error::Snapshot(format!(
            "cell count {cell_count} exceeds volume cap {MAX_VOLUME}"
        )));
    }
    let mut cells = Vec::with_capacity(cell_count);
    for _ in 0..cell_count {
        let id = read_u64(bytes, &mut cursor)?;
        let site = Site {
            x: read_u32(bytes, &mut cursor)?,
            y: read_u32(bytes, &mut cursor)?,
            z: read_u32(bytes, &mut cursor)?,
        };
        let energy = read_u64(bytes, &mut cursor)?;
        let age = read_u64(bytes, &mut cursor)?;
        let genome = read_bytes(bytes, &mut cursor, 8)?;
        cells.push(Cell {
            id,
            site,
            energy,
            age,
            genome,
        });
    }
    let death_count = read_u32(bytes, &mut cursor)? as usize;
    if death_count as u64 > MAX_VOLUME {
        return Err(Error::Snapshot(format!(
            "death count {death_count} exceeds volume cap {MAX_VOLUME}"
        )));
    }
    let mut deaths = Vec::with_capacity(death_count);
    for _ in 0..death_count {
        let id = read_u64(bytes, &mut cursor)?;
        let site = Site {
            x: read_u32(bytes, &mut cursor)?,
            y: read_u32(bytes, &mut cursor)?,
            z: read_u32(bytes, &mut cursor)?,
        };
        let energy_disposed = read_u64(bytes, &mut cursor)?;
        let age = read_u64(bytes, &mut cursor)?;
        let during_tick = read_u64(bytes, &mut cursor)?;
        let genome = read_bytes(bytes, &mut cursor, 8)?;
        deaths.push(DeathRecord {
            id,
            site,
            energy_disposed,
            age,
            during_tick,
            genome,
        });
    }
    let nutrient_count = read_u32(bytes, &mut cursor)?;
    if u64::from(nutrient_count) > MAX_VOLUME {
        return Err(Error::Snapshot(format!(
            "nutrient count {nutrient_count} exceeds volume cap {MAX_VOLUME}"
        )));
    }
    let mut nutrient = Vec::with_capacity(nutrient_count as usize);
    for _ in 0..nutrient_count {
        nutrient.push(read_u64(bytes, &mut cursor)?);
    }
    if cursor != bytes.len() {
        return Err(Error::Snapshot(
            "canonical payload has trailing bytes".into(),
        ));
    }
    Snapshot::try_from_parts(Snapshot {
        tick,
        seed,
        width,
        height,
        depth,
        reservoir,
        waste,
        external_input,
        external_output,
        energy_credited,
        energy_maintenance,
        energy_disposed,
        material_initial,
        next_cell_id,
        inject_cursor,
        cells,
        deaths,
        nutrient,
    })
}

fn push_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn push_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn read_u32(bytes: &[u8], cursor: &mut usize) -> Result<u32, Error> {
    let raw = read_array(bytes, cursor)?;
    Ok(u32::from_le_bytes(raw))
}

fn read_u64(bytes: &[u8], cursor: &mut usize) -> Result<u64, Error> {
    let raw = read_array(bytes, cursor)?;
    Ok(u64::from_le_bytes(raw))
}

fn read_array<const N: usize>(bytes: &[u8], cursor: &mut usize) -> Result<[u8; N], Error> {
    if *cursor + N > bytes.len() {
        return Err(Error::Snapshot("truncated canonical field".into()));
    }
    let mut raw = [0u8; N];
    raw.copy_from_slice(&bytes[*cursor..*cursor + N]);
    *cursor += N;
    Ok(raw)
}

fn read_bytes(bytes: &[u8], cursor: &mut usize, len: usize) -> Result<[u8; 8], Error> {
    if len != 8 {
        return Err(Error::Snapshot("genome width is not 8".into()));
    }
    read_array(bytes, cursor)
}

/// Used by manifests that store the hex form and need the raw bytes.
pub fn hex_to_hash(text: &str) -> Result<[u8; 32], Error> {
    parse_hex32(text)
}
