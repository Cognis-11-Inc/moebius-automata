//! Headless HABITAT-001 experiments; never canonical ship admission.
use moebius_morph::primordial::{Chamber, Config, experiments};
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
        Some("suite") => {
            experiments::suite(Path::new(args.get(1).ok_or("new output required")?), memory)?
        }
        Some("demo") => {
            let ticks = args.get(2).map_or(Ok(120), |s| s.parse())?;
            let cells = args.get(3).map_or(Ok(16), |s| s.parse())?;
            let config=if let Some(path)=args.get(4) {serde_json::from_slice(&std::fs::read(path)?)?} else {Config{cells,..Default::default()}};
            experiments::record_demo(
                Path::new(args.get(1).ok_or("new output required")?),
                config,
                ticks,
                memory,
            )?;
        }
        Some("replay") => {
            let mut c = Chamber::load(Path::new(args.get(1).ok_or("checkpoint required")?))?;
            let ticks = args.get(3).ok_or("ticks required")?.parse()?;
            c.advance(ticks)?;
            c.save(Path::new(args.get(2).ok_or("new checkpoint required")?))?;
            println!("{}", c.digest()?);
        }
        _ => return Err(
            "suite NEW_OUTPUT | demo NEW_OUTPUT [ticks] [cells] [SI_CONFIG.json] | replay INITIAL NEW_FINAL ticks"
                .into(),
        ),
    }
    Ok(())
}
