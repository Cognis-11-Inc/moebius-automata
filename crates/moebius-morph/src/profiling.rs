//! Feature-gated observer counters. No simulation input or mutation rights.
use std::{
    cell::Cell,
    sync::atomic::{AtomicU64, Ordering::Relaxed},
    time::Instant,
};
pub const LABELS: [&str; 18] = [
    "outside",
    "tick",
    "state_clone",
    "policy",
    "resolve",
    "recovery",
    "placement",
    "deposit_admission",
    "emission_admission",
    "root_admission",
    "nozzle_frames",
    "book_step",
    "finish",
    "event_commit",
    "geometry",
    "clearance",
    "overlap",
    "containment",
];
thread_local! { static CURRENT: Cell<usize> = const {Cell::new(0)}; static CHILD: Cell<u64> = const {Cell::new(0)}; }
struct Counters {
    values: [AtomicU64; 7],
}
static COUNTERS: [Counters; 18] = [const {
    Counters {
        values: [const { AtomicU64::new(0) }; 7],
    }
}; 18];
pub struct Scope {
    parent: usize,
    parent_child: u64,
    start: Instant,
    id: usize,
}
pub fn enter(id: usize) -> Scope {
    let parent = CURRENT.with(|c| c.replace(id));
    let parent_child = CHILD.with(|c| c.replace(0));
    COUNTERS[id].values[0].fetch_add(1, Relaxed);
    Scope {
        parent,
        parent_child,
        start: Instant::now(),
        id,
    }
}
impl Drop for Scope {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed().as_nanos() as u64;
        let nested = CHILD.with(Cell::get);
        COUNTERS[self.id].values[1].fetch_add(elapsed.saturating_sub(nested), Relaxed);
        CURRENT.with(|c| c.set(self.parent));
        CHILD.with(|c| c.set(self.parent_child.saturating_add(elapsed)));
    }
}
/// Allocation/reallocation requests and frees are attributed to execution scope,
/// not allocation ownership. Requested bytes are cumulative, not retained bytes.
pub fn allocation(kind: usize, bytes: usize) {
    let id = CURRENT.try_with(Cell::get).unwrap_or(0);
    COUNTERS[id].values[kind].fetch_add(1, Relaxed);
    COUNTERS[id].values[if kind == 4 { 6 } else { 5 }].fetch_add(bytes as u64, Relaxed);
}
pub fn reset() {
    for v in &WORK {
        v.store(0, Relaxed);
    }
    for c in &COUNTERS {
        for a in &c.values {
            a.store(0, Relaxed);
        }
    }
}
/// calls, exclusive ns, alloc, realloc, dealloc, requested bytes, freed bytes.
pub fn snapshot() -> [[u64; 7]; 18] {
    std::array::from_fn(|i| std::array::from_fn(|k| COUNTERS[i].values[k].load(Relaxed)))
}

thread_local! { static SAMPLING: Cell<bool> = const {Cell::new(false)}; static SAMPLE_COUNT: Cell<u64> = const {Cell::new(0)}; }
/// Optional allocation-stack sampling pass. Never mix its timings/counts with
/// the ordinary attribution pass: stack capture allocates observer storage.
pub fn sample_stack() {
    if SAMPLING.try_with(Cell::get).unwrap_or(true) {
        return;
    }
    SAMPLING.with(|v| v.set(true));
    let count = SAMPLE_COUNT.with(|v| {
        let n = v.get() + 1;
        v.set(n);
        n
    });
    if count.is_multiple_of(10000)
        && let Ok(dir) = std::env::var("MOEBIUS_ALLOCATION_STACK_DIR")
    {
        let stack = std::backtrace::Backtrace::force_capture();
        let _ = std::fs::write(
            std::path::Path::new(&dir).join(format!("allocation-{count}.txt")),
            format!("{stack}"),
        );
    }
    SAMPLING.with(|v| v.set(false));
}

// Work counters count observable query operations, not hidden Parry loop bodies.
pub const WORK_LABELS: [&str; 10] = [
    "support_calls",
    "support_vertex_scan_candidates",
    "gjk_contact_calls",
    "epa_calls",
    "actual_hull_vertex_dots",
    "support_index_nodes",
    "support_planes_tested",
    "support_plane_rejects",
    "support_bound_evaluations",
    "support_bound_rejections",
];
static WORK: [AtomicU64; 10] = [const { AtomicU64::new(0) }; 10];
pub fn work(id: usize, amount: u64) {
    WORK[id].fetch_add(amount, Relaxed);
}
pub fn work_snapshot() -> [u64; 10] {
    std::array::from_fn(|i| WORK[i].load(Relaxed))
}
