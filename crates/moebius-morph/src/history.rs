//! Opt-in segmented history encoding. ShipRuntime remains the only tick owner.
//! Segments are immutable; checkpoint state, event order and checked provenance
//! remain authoritative. Existing physical snapshot versions are unchanged.
use crate::{
    MorphError, require,
    runtime::{Event, WorldSnapshot},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};
const EMPTY: &str = "0000000000000000000000000000000000000000000000000000000000000000";
fn source_digest() -> Result<String, MorphError> {
    let identity: serde_json::Value =
        serde_json::from_str(crate::BENCHMARK_BUILD_IDENTITY).map_err(error)?;
    Ok(
        blake3::hash(&serde_json::to_vec(&identity["sources_blake3"]).map_err(error)?)
            .to_hex()
            .to_string(),
    )
}
fn error(e: impl std::fmt::Display) -> MorphError {
    MorphError(e.to_string())
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub resident_events: usize,
    /// Sum of canonical event JSON bytes. Count also bounds fixed enum overhead.
    pub resident_bytes: u64,
    pub segment_events: usize,
    pub recent_events: usize,
    pub provenance_entries: usize,
}
impl Default for Policy {
    fn default() -> Self {
        Self {
            resident_events: 4096,
            resident_bytes: 4 * 1024 * 1024,
            segment_events: 1024,
            recent_events: 256,
            provenance_entries: 16384,
        }
    }
}
impl Policy {
    pub fn validate(&self) -> Result<(), MorphError> {
        require(
            self.recent_events >= 256
                && self.resident_events > self.recent_events
                && self.resident_events <= 65536
                && self.segment_events > 0
                && self.segment_events <= self.resident_events - self.recent_events
                && (4096..=64 * 1024 * 1024).contains(&self.resident_bytes)
                && (16..=65536).contains(&self.provenance_entries),
            "invalid bounded history policy",
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BirthProof {
    pub parent: u64,
    pub child: u64,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Summary {
    pub fed: BTreeMap<u64, u64>,
    pub births: Vec<BirthProof>,
    pub birth_count: u64,
    pub provenance_counts: [u64; 4],
}
impl Summary {
    fn add(&mut self, event: &Event, limit: usize) -> Result<(), MorphError> {
        if let Some(index) = provenance_index(event) {
            self.provenance_counts[index] = self.provenance_counts[index]
                .checked_add(1)
                .ok_or_else(|| error("history provenance count overflow"))?;
            require(
                self.provenance_counts[index] <= limit as u64,
                "history validation provenance budget",
            )?;
        }
        match event {
            Event::Fed {
                organism, quantity, ..
            } => {
                let value = self.fed.entry(*organism).or_default();
                *value = value
                    .checked_add(*quantity)
                    .ok_or_else(|| error("history intake overflow"))?;
            }
            Event::Reproduced { parent, child, .. } => {
                self.birth_count = self
                    .birth_count
                    .checked_add(1)
                    .ok_or_else(|| error("history birth overflow"))?;
                if !self
                    .births
                    .iter()
                    .any(|p| p.parent == *parent && p.child == *child)
                {
                    self.births.push(BirthProof {
                        parent: *parent,
                        child: *child,
                    });
                }
            }
            _ => {}
        }
        require(
            self.fed.len() <= limit && self.births.len() <= limit,
            "history provenance resident budget",
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Prefix {
    pub segments: u64,
    pub events: u64,
    pub hash: String,
    pub summary: Summary,
}
impl Default for Prefix {
    fn default() -> Self {
        Self {
            segments: 0,
            events: 0,
            hash: EMPTY.into(),
            summary: Default::default(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub encoding: u32,
    pub origin_digest: String,
    pub source_digest: String,
    pub policy: Policy,
    pub prefix: Prefix,
    pub resident_bytes: u64,
    pub resident_provenance_counts: [u64; 4],
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoreMeta {
    encoding: u32,
    origin_digest: String,
    source_digest: String,
    policy: Policy,
}
#[derive(Serialize)]
struct SegmentRef<'a> {
    encoding: u32,
    ordinal: u64,
    first_event: u64,
    previous_hash: &'a str,
    source_digest: &'a str,
    events: &'a [Event],
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Segment {
    encoding: u32,
    ordinal: u64,
    first_event: u64,
    previous_hash: String,
    source_digest: String,
    events: Vec<Event>,
}
/// Explicit backing location. Paths never enter logical world identity.
/// Experimental working copies must still be archived on verified Mermogon.
pub struct Store {
    root: PathBuf,
    meta: StoreMeta,
}
fn provenance_index(event: &Event) -> Option<usize> {
    match event {
        Event::Accepted { .. } => Some(0),
        Event::Emitted { .. } => Some(1),
        Event::Deposited { .. } => Some(2),
        Event::Chewed { .. } => Some(3),
        _ => None,
    }
}
fn provenance_counts(events: &[Event]) -> [u64; 4] {
    let mut counts = [0; 4];
    for event in events {
        if let Some(index) = provenance_index(event) {
            counts[index] += 1;
        }
    }
    counts
}
struct ByteCount(u64);
impl Write for ByteCount {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.0 = self
            .0
            .checked_add(data.len() as u64)
            .ok_or_else(|| std::io::Error::other("event byte overflow"))?;
        Ok(data.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
pub(crate) fn event_bytes(events: &[Event]) -> Result<u64, MorphError> {
    let mut count = ByteCount(0);
    for event in events {
        serde_json::to_writer(&mut count, event).map_err(error)?;
    }
    Ok(count.0)
}
fn hash_file(path: &Path) -> Result<String, MorphError> {
    let mut file = fs::File::open(path).map_err(error)?;
    let mut hash = blake3::Hasher::new();
    let mut buffer = [0u8; 65536];
    loop {
        let size = file.read(&mut buffer).map_err(error)?;
        if size == 0 {
            break;
        }
        hash.update(&buffer[..size]);
    }
    Ok(hash.finalize().to_hex().to_string())
}
fn sync_directory(path: &Path) -> Result<(), MorphError> {
    fs::File::open(path)
        .and_then(|f| f.sync_all())
        .map_err(error)
}
/// No replacement of a sealed file. A matching orphan is reusable after failure.
/// Failed temporary writes remain available for diagnosis; old data are retained.
fn immutable_write(path: &Path, data: &[u8]) -> Result<(), MorphError> {
    let expected = blake3::hash(data).to_hex().to_string();
    if path.exists() {
        return require(
            hash_file(path)? == expected,
            "immutable history conflict; fork the backing store explicitly",
        );
    }
    static SERIAL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let serial = SERIAL.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let temporary = path.with_extension(format!("tmp-{}-{serial}", std::process::id()));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(error)?;
    file.write_all(data).map_err(error)?;
    file.sync_all().map_err(error)?;
    drop(file);
    match fs::hard_link(&temporary, path) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => require(
            hash_file(path)? == expected,
            "immutable history publication conflict",
        )?,
        Err(e) => return Err(error(e)),
    }
    sync_directory(
        path.parent()
            .ok_or_else(|| error("history parent missing"))?,
    )?;
    // This is a verified duplicate link, not deletion of the sole evidence copy.
    fs::remove_file(&temporary).map_err(error)?;
    Ok(())
}
impl Store {
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub(crate) fn create(
        root: &Path,
        origin: &WorldSnapshot,
        policy: Policy,
    ) -> Result<Self, MorphError> {
        policy.validate()?;
        require(
            origin.history.is_none(),
            "migration requires a legacy full-history origin",
        )?;
        let digest = origin.digest()?;
        fs::create_dir(root).map_err(error)?;
        fs::create_dir(root.join("segments")).map_err(error)?;
        let meta = StoreMeta {
            encoding: 2,
            origin_digest: digest,
            source_digest: source_digest()?,
            policy,
        };
        immutable_write(
            &root.join("STORE.json"),
            &serde_json::to_vec(&meta).map_err(error)?,
        )?;
        let data = serde_json::to_vec(origin).map_err(error)?;
        immutable_write(&root.join("LEGACY-ORIGIN.json"), &data)?;
        require(
            hash_file(&root.join("LEGACY-ORIGIN.json"))? == meta.origin_digest,
            "migration origin hash mismatch",
        )?;
        sync_directory(root)?;
        Ok(Self {
            root: root.to_owned(),
            meta,
        })
    }
    pub(crate) fn compatible(
        &self,
        origin: &WorldSnapshot,
        policy: &Policy,
    ) -> Result<(), MorphError> {
        require(
            self.meta.origin_digest == origin.digest()? && self.meta.policy == *policy,
            "migration backing does not match origin/policy",
        )
    }
    pub fn open(root: &Path) -> Result<Self, MorphError> {
        let meta: StoreMeta =
            serde_json::from_slice(&fs::read(root.join("STORE.json")).map_err(error)?)
                .map_err(error)?;
        meta.policy.validate()?;
        require(
            meta.encoding == 2 && meta.source_digest == source_digest()?,
            "unsupported history encoding/source; explicit migration required",
        )?;
        require(
            hash_file(&root.join("LEGACY-ORIGIN.json"))? == meta.origin_digest,
            "history origin missing or corrupt",
        )?;
        Ok(Self {
            root: root.to_owned(),
            meta,
        })
    }
    pub(crate) fn initial(&self, events: &[Event]) -> Result<State, MorphError> {
        Ok(State {
            encoding: 2,
            origin_digest: self.meta.origin_digest.clone(),
            source_digest: self.meta.source_digest.clone(),
            policy: self.meta.policy.clone(),
            prefix: Default::default(),
            resident_bytes: event_bytes(events)?,
            resident_provenance_counts: provenance_counts(events),
        })
    }
    fn path(&self, ordinal: u64) -> PathBuf {
        self.root
            .join("segments")
            .join(format!("{ordinal:020}.json"))
    }
    fn seal(&self, state: &State, prefix: &Prefix, events: &[Event]) -> Result<Prefix, MorphError> {
        require(
            !events.is_empty() && events.len() <= state.policy.segment_events,
            "invalid history segment length",
        )?;
        let mut next = prefix.clone();
        for event in events {
            next.summary.add(event, state.policy.provenance_entries)?;
        }
        let segment = SegmentRef {
            encoding: 2,
            ordinal: prefix.segments,
            first_event: prefix.events,
            previous_hash: &prefix.hash,
            source_digest: &state.source_digest,
            events,
        };
        let data = serde_json::to_vec(&segment).map_err(error)?;
        require(
            event_bytes(events)? <= state.policy.resident_bytes,
            "history segment byte budget",
        )?;
        immutable_write(&self.path(prefix.segments), &data)?;
        next.hash = blake3::hash(&data).to_hex().to_string();
        next.segments = next
            .segments
            .checked_add(1)
            .ok_or_else(|| error("history segment count overflow"))?;
        next.events = next
            .events
            .checked_add(events.len() as u64)
            .ok_or_else(|| error("history event count overflow"))?;
        Ok(next)
    }
    /// All fallible work precedes mutation of resident events or the visible head.
    pub(crate) fn append(
        &self,
        state: &mut State,
        tail: &mut Vec<Event>,
        delta: &mut Vec<Event>,
    ) -> Result<(), MorphError> {
        require(
            state.encoding == 2
                && state.source_digest == self.meta.source_digest
                && state.policy == self.meta.policy
                && state.origin_digest == self.meta.origin_digest,
            "history backing identity mismatch",
        )?;
        let mut counts = state.resident_provenance_counts;
        let added = provenance_counts(delta);
        for index in 0..4 {
            counts[index] = counts[index]
                .checked_add(added[index])
                .ok_or_else(|| error("history provenance overflow"))?;
            require(
                counts[index]
                    .checked_add(state.prefix.summary.provenance_counts[index])
                    .is_some_and(|total| total <= state.policy.provenance_entries as u64),
                "history validation provenance budget",
            )?;
        }
        let mut next = state.clone();
        let mut removed = 0usize;
        let mut bytes = state
            .resident_bytes
            .checked_add(event_bytes(delta)?)
            .ok_or_else(|| error("resident history byte overflow"))?;
        while tail.len() - removed + delta.len() > state.policy.resident_events
            || bytes > state.policy.resident_bytes
        {
            let available = (tail.len() - removed).saturating_sub(state.policy.recent_events);
            require(
                available > 0,
                "HistoryBudgetV2: one transition cannot fit the bounded resident policy",
            )?;
            let mut count = available.min(state.policy.segment_events);
            while count > 1
                && event_bytes(&tail[removed..removed + count])? > state.policy.resident_bytes
            {
                count /= 2;
            }
            let events = &tail[removed..removed + count];
            next.prefix = self.seal(state, &next.prefix, events)?;
            bytes = bytes
                .checked_sub(event_bytes(events)?)
                .ok_or_else(|| error("resident history accounting underflow"))?;
            let sealed = provenance_counts(events);
            for index in 0..4 {
                counts[index] = counts[index]
                    .checked_sub(sealed[index])
                    .ok_or_else(|| error("history provenance underflow"))?;
            }
            removed += count;
        }
        // Reserve before draining so allocator failure cannot remove a prefix.
        let shrink = tail.capacity() > state.policy.resident_events;
        let mut compact = Vec::new();
        if shrink {
            compact
                .try_reserve_exact(tail.len() - removed + delta.len())
                .map_err(error)?;
        } else {
            tail.try_reserve(delta.len().saturating_sub(removed))
                .map_err(error)?;
        }
        tail.drain(..removed);
        if shrink {
            compact.append(tail);
            compact.append(delta);
            *tail = compact;
        } else {
            tail.append(delta);
        }
        next.resident_bytes = bytes;
        next.resident_provenance_counts = counts;
        *state = next;
        Ok(())
    }
    pub fn visit(
        &self,
        state: &State,
        mut consume: impl FnMut(&Event) -> Result<(), MorphError>,
    ) -> Result<(), MorphError> {
        require(
            state.encoding == 2
                && state.source_digest == self.meta.source_digest
                && state.policy == self.meta.policy
                && state.origin_digest == self.meta.origin_digest,
            "history backing identity mismatch",
        )?;
        let mut previous = EMPTY.to_owned();
        let mut count = 0u64;
        let mut summary = Summary::default();
        for ordinal in 0..state.prefix.segments {
            let path = self.path(ordinal);
            let bound = state.policy.resident_bytes + state.policy.segment_events as u64 + 8192;
            require(
                fs::metadata(&path).map_err(error)?.len() <= bound,
                "oversized history segment",
            )?;
            let data = fs::read(&path).map_err(error)?;
            let segment: Segment = serde_json::from_slice(&data).map_err(error)?;
            require(
                segment.encoding == 2
                    && segment.source_digest == state.source_digest
                    && segment.ordinal == ordinal
                    && segment.first_event == count
                    && segment.previous_hash == previous
                    && !segment.events.is_empty()
                    && segment.events.len() <= state.policy.segment_events,
                "history sequence/chain mismatch",
            )?;
            require(
                event_bytes(&segment.events)? <= state.policy.resident_bytes,
                "history segment payload budget",
            )?;
            for event in &segment.events {
                summary.add(event, state.policy.provenance_entries)?;
                consume(event)?;
            }
            count = count
                .checked_add(segment.events.len() as u64)
                .ok_or_else(|| error("history count overflow"))?;
            previous = blake3::hash(&data).to_hex().to_string();
        }
        require(
            count == state.prefix.events
                && previous == state.prefix.hash
                && summary == state.prefix.summary,
            "history head/provenance mismatch",
        )
    }
    pub fn verify(&self, state: &State, tail: &[Event]) -> Result<(), MorphError> {
        self.visit(state, |_| Ok(()))?;
        require(
            tail.len() <= state.policy.resident_events
                && provenance_counts(tail) == state.resident_provenance_counts
                && event_bytes(tail)? == state.resident_bytes
                && state.resident_bytes <= state.policy.resident_bytes,
            "resident history count/byte mismatch",
        )
    }
    pub(crate) fn accepted_sequence(
        &self,
        state: &State,
        sequence: u64,
    ) -> Result<bool, MorphError> {
        let mut found = false;
        self.visit(state, |event| {
            if matches!(event,Event::Accepted{command,..}if command.sequence==sequence) {
                found = true;
            }
            Ok(())
        })?;
        Ok(found)
    }
    /// Lossless portable backing copy; publication remains explicit and immutable.
    pub fn copy_prefix(&self, state: &State, destination: &Path) -> Result<Self, MorphError> {
        self.visit(state, |_| Ok(()))?;
        fs::create_dir(destination).map_err(error)?;
        fs::create_dir(destination.join("segments")).map_err(error)?;
        for name in ["STORE.json", "LEGACY-ORIGIN.json"] {
            fs::copy(self.root.join(name), destination.join(name)).map_err(error)?;
            require(
                hash_file(&self.root.join(name))? == hash_file(&destination.join(name))?,
                "history bundle copy mismatch",
            )?;
        }
        for ordinal in 0..state.prefix.segments {
            let name = format!("{ordinal:020}.json");
            let target = destination.join("segments").join(name);
            fs::copy(self.path(ordinal), &target).map_err(error)?;
            require(
                hash_file(&self.path(ordinal))? == hash_file(&target)?,
                "history segment copy mismatch",
            )?;
        }
        for name in ["STORE.json", "LEGACY-ORIGIN.json"] {
            fs::File::open(destination.join(name))
                .and_then(|f| f.sync_all())
                .map_err(error)?;
        }
        for ordinal in 0..state.prefix.segments {
            fs::File::open(
                destination
                    .join("segments")
                    .join(format!("{ordinal:020}.json")),
            )
            .and_then(|f| f.sync_all())
            .map_err(error)?;
        }
        sync_directory(&destination.join("segments"))?;
        sync_directory(destination)?;
        let copy = Self::open(destination)?;
        copy.visit(state, |_| Ok(()))?;
        Ok(copy)
    }
}
