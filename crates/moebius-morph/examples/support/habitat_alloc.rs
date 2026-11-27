#[cfg(feature = "allocation-attribution")]
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
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

pub fn memory() -> [usize; 3] {
    [BYTES.load(Relaxed), LIVE.load(Relaxed), PEAK.load(Relaxed)]
}
