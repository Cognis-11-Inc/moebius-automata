//! Headless HABITAT-001 experiments; never canonical ship admission.
use moebius_morph::primordial::{Chamber, Config, experiments};
use std::path::Path;
#[path = "support/habitat_alloc.rs"]
mod allocation;
use allocation::memory;

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
