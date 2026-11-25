//! Matched-state coordinate-envelope audit. No alternate simulation authority.
use moebius_morph::{emission::SCALE, filament::StructuralIndex, runtime::*, witness::*};
use serde_json::json;
use std::{
    alloc::{GlobalAlloc, Layout, System},
    hint::black_box,
    path::Path,
    sync::atomic::{AtomicUsize, Ordering::Relaxed},
    time::Instant,
};

struct Counted;
static ALLOCS: AtomicUsize = AtomicUsize::new(0);
static BYTES: AtomicUsize = AtomicUsize::new(0);
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
// Instrument only this executable. Requested bytes exclude allocator metadata.
unsafe impl GlobalAlloc for Counted {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(layout) };
        if !p.is_null() {
            ALLOCS.fetch_add(1, Relaxed);
            BYTES.fetch_add(layout.size(), Relaxed);
            let live = LIVE.fetch_add(layout.size(), Relaxed) + layout.size();
            PEAK.fetch_max(live, Relaxed);
        }
        p
    }
    unsafe fn dealloc(&self, p: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size(), Relaxed);
        unsafe { System.dealloc(p, layout) };
    }
}
#[global_allocator]
static ALLOCATOR: Counted = Counted;

fn rss() -> Option<u64> {
    std::fs::read_to_string("/proc/self/status")
        .ok()?
        .lines()
        .find_map(|line| {
            line.strip_prefix("VmHWM:")?
                .split_whitespace()
                .next()?
                .parse()
                .ok()
        })
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let initial = WorldSnapshot::load(Path::new(args.first().ok_or("snapshot required")?))?;
    let ticks: usize = args.get(1).ok_or("tick count required")?.parse()?;
    let output = Path::new(args.get(2).ok_or("new output directory required")?);
    std::fs::create_dir(output)?;
    let restore_started = Instant::now();
    let mut world = ShipRuntime::restore(initial)?;
    let restore_ns = restore_started.elapsed().as_nanos();
    world.tick()?; // Identical cache warmup in every process, counted separately.
    let start_tick = world.state().tick;
    let profile = world.collision_profile();
    let mut rows = Vec::with_capacity(ticks);
    for _ in 0..ticks {
        let live = LIVE.load(Relaxed);
        PEAK.store(live, Relaxed);
        let allocations = ALLOCS.load(Relaxed);
        let bytes = BYTES.load(Relaxed);
        let start = Instant::now();
        world.tick()?;
        let ns = start.elapsed().as_nanos();
        let allocated = ALLOCS.load(Relaxed) - allocations;
        let requested = BYTES.load(Relaxed) - bytes;
        let peak = PEAK.load(Relaxed);
        let digest = world.digest()?;
        rows.push(
            json!({"tick":world.state().tick,"ns":ns,"allocations":allocated,
            "allocated_bytes":requested,"peak_live_bytes":peak,"digest":digest}),
        );
    }
    world.validate()?;
    let p = world.collision_profile();
    let state = world.snapshot();
    state.save(&output.join("final.json"))?;
    let start = Instant::now();
    let index = StructuralIndex::build(state.assembly.as_ref().ok_or("assembly missing")?)?;
    let index_ns = start.elapsed().as_nanos();
    let mut visibility = vec![];
    for metres in [20, 1_000, 4_000, 30_000_000, 999_000_000] {
        if metres * SCALE > MAX_POSITION {
            continue;
        }
        let sphere = WitnessSphere {
            center: Position::from_absolute([0; 3])?,
            radius: metres * SCALE,
        };
        let start = Instant::now();
        let mut found = 0;
        for _ in 0..100 {
            found = black_box(index.visible(&sphere)?).len();
        }
        visibility.push(
            json!({"radius_m":metres,"queries":100,"chunks":found,"ns":start.elapsed().as_nanos()}),
        );
    }
    let mut positions = vec![];
    for metres in [0, 15_000_000, 900_000_000] {
        if metres * SCALE + 1_000 > MAX_POSITION {
            continue;
        }
        let origin = Position::from_absolute([metres * SCALE; 3])?;
        let start = Instant::now();
        let mut checksum = 0i64;
        for k in 0..200_000 {
            let q = black_box(origin).offset([black_box(k % 100), -1, 1])?;
            checksum ^= black_box(q.relative(origin)?)[0];
        }
        positions.push(json!({"origin_m":metres,"operations":200_000,"ns":start.elapsed().as_nanos(),"checksum":checksum}));
    }
    let report = json!({"max_position_m":MAX_POSITION / SCALE,"region_m":REGION / SCALE,
        "source_blake3": {
            "collision.rs":blake3::hash(include_bytes!("../src/collision.rs")).to_hex().to_string(),
            "runtime.rs":blake3::hash(include_bytes!("../src/runtime.rs")).to_hex().to_string(),
            "anatomy.rs":blake3::hash(include_bytes!("../src/anatomy.rs")).to_hex().to_string(),
            "compiler.rs":blake3::hash(include_bytes!("../src/compiler.rs")).to_hex().to_string()
        },
        "start_tick":start_tick,"ticks":ticks,"restore_ns":restore_ns,"tick_rows":rows,
        "queries":p.queries-profile.queries,"distance_queries":p.distance_queries-profile.distance_queries,
        "pipe_builds":p.pipe_builds-profile.pipe_builds,"body_builds":p.body_builds-profile.body_builds,
        "resolve_ns":p.resolve_ns-profile.resolve_ns,"distance_ns":p.distance_ns-profile.distance_ns,
        "contact_ns":p.contact_ns-profile.contact_ns,"instances":state.instances.len(),
        "agents":state.habitat.as_ref().unwrap().agents.len(),"nodes":state.assembly.as_ref().unwrap().nodes(),
        "material_gap":state.condensate_gap(),"index_chunks":index.chunks.len(),"index_build_ns":index_ns,
        "visibility":visibility,"positions":positions,"vm_hwm_kib":rss(),"digest":world.digest()?});
    std::fs::write(
        output.join("report.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    Ok(())
}
