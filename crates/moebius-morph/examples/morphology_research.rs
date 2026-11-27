//! Explicit research runner; no runtime integration or canonical anatomy mutation.
use moebius_morph::{
    collision::fixtures::{PipeCastFixture, research_vii},
    runtime::WorldSnapshot,
};
#[cfg(feature = "allocation-attribution")]
use std::alloc::{GlobalAlloc, Layout, System};
use std::{
    path::Path,
    sync::atomic::{AtomicUsize, Ordering::Relaxed},
};
#[cfg(feature = "allocation-attribution")]
struct Counted;
#[cfg(feature = "allocation-attribution")]
static ALLOCS: AtomicUsize = AtomicUsize::new(0);
static BYTES: AtomicUsize = AtomicUsize::new(0);
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
// Instrument only this executable. Requested bytes exclude allocator metadata.
#[cfg(feature = "allocation-attribution")]
unsafe impl GlobalAlloc for Counted {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(layout) };
        if !p.is_null() {
            #[cfg(feature = "allocation-attribution")]
            {
                moebius_morph::profiling::allocation(2, layout.size());
                moebius_morph::profiling::sample_stack();
            }
            ALLOCS.fetch_add(1, Relaxed);
            BYTES.fetch_add(layout.size(), Relaxed);
            let live = LIVE.fetch_add(layout.size(), Relaxed) + layout.size();
            PEAK.fetch_max(live, Relaxed);
        }
        p
    }
    unsafe fn realloc(&self, p: *mut u8, old: Layout, new: usize) -> *mut u8 {
        let result = unsafe { System.realloc(p, old, new) };
        if !result.is_null() {
            #[cfg(feature = "allocation-attribution")]
            moebius_morph::profiling::allocation(3, new);
            ALLOCS.fetch_add(1, Relaxed);
            BYTES.fetch_add(new, Relaxed);
            let live = if new >= old.size() {
                LIVE.fetch_add(new - old.size(), Relaxed) + new - old.size()
            } else {
                LIVE.fetch_sub(old.size() - new, Relaxed) - (old.size() - new)
            };
            PEAK.fetch_max(live, Relaxed);
        }
        result
    }
    unsafe fn dealloc(&self, p: *mut u8, layout: Layout) {
        #[cfg(feature = "allocation-attribution")]
        moebius_morph::profiling::allocation(4, layout.size());
        LIVE.fetch_sub(layout.size(), Relaxed);
        unsafe { System.dealloc(p, layout) };
    }
}
#[cfg(feature = "allocation-attribution")]
#[global_allocator]
static ALLOCATOR: Counted = Counted;

fn memory() -> [usize; 3] {
    [BYTES.load(Relaxed), LIVE.load(Relaxed), PEAK.load(Relaxed)]
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.first().is_some_and(|s| s == "--build-identity") {
        println!("{}", moebius_morph::BENCHMARK_BUILD_IDENTITY);
        return Ok(());
    }
    match args.first().map(String::as_str) {
        Some("behaviour") => research_vii::behaviour(
            Path::new(args.get(1).ok_or("new output required")?),
            args.get(2).map_or(Ok(24), |s| s.parse())?,
            memory,
        )?,
        Some("escape-routes") => {
            let world: WorldSnapshot = serde_json::from_slice(&std::fs::read(
                args.get(1).ok_or("resolver state required")?,
            )?)?;
            let value = research_vii::escape_routes(&world)?;
            std::fs::write(
                args.get(2).ok_or("output required")?,
                serde_json::to_vec_pretty(&value)?,
            )?;
        }
        Some("escape") => {
            // Experimental resolver output, not a validated full-runtime restore.
            let world: WorldSnapshot = serde_json::from_slice(&std::fs::read(
                args.get(1).ok_or("resolver state required")?,
            )?)?;
            let value = research_vii::escape_intervention(&world)?;
            std::fs::write(
                args.get(2).ok_or("output required")?,
                serde_json::to_vec_pretty(&value)?,
            )?;
        }
        Some("economy") => {
            let world = WorldSnapshot::load(Path::new(args.get(1).ok_or("world required")?))?;
            research_vii::geometry_economy(
                &world,
                Path::new(args.get(2).ok_or("new output required")?),
                memory,
            )?;
        }
        Some("procedural") => {
            let world = WorldSnapshot::load(Path::new(args.get(1).ok_or("world required")?))?;
            research_vii::procedural(
                &world,
                Path::new(args.get(2).ok_or("new output required")?),
                memory,
            )?;
        }
        Some("components") => {
            let world = WorldSnapshot::load(Path::new(args.get(1).ok_or("world required")?))?;
            let fixture: PipeCastFixture =
                serde_json::from_slice(&std::fs::read(args.get(2).ok_or("fixture required")?)?)?;
            let value = research_vii::components(
                &world,
                &fixture,
                args.get(4).map_or(Ok(4), |s| s.parse())?,
                args.get(5).map_or(Ok(5), |s| s.parse())?,
            )?;
            std::fs::write(
                args.get(3).ok_or("output required")?,
                serde_json::to_vec_pretty(&value)?,
            )?;
        }
        _ => return Err("expected behaviour or components".into()),
    }
    Ok(())
}
