//! Isolated invocation of the canonical cast; never an alternate world model.
use moebius_morph::{
    collision::fixtures::{self, PipeCastFixture},
    runtime::WorldSnapshot,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.first().is_some_and(|s| s == "--build-identity") {
        println!("{}", moebius_morph::BENCHMARK_BUILD_IDENTITY);
        return Ok(());
    }
    if args.first().is_some_and(|s| s == "--extract") {
        let initial = WorldSnapshot::load(std::path::Path::new(
            args.get(1).ok_or("initial snapshot required")?,
        ))?;
        let requests: Vec<fixtures::CaptureRequest> =
            serde_json::from_slice(&std::fs::read(args.get(2).ok_or("requests required")?)?)?;
        let end = requests
            .iter()
            .map(|r| r.tick + 1)
            .max()
            .ok_or("empty capture request")?;
        let out = std::path::Path::new(args.get(3).ok_or("new output required")?);
        std::fs::create_dir(out)?;
        let initial_tick = initial.tick;
        let mut world = moebius_morph::runtime::ShipRuntime::restore(initial)?;
        fixtures::request(requests);
        world.advance(
            end.checked_sub(initial_tick)
                .ok_or("capture precedes initial state")?,
        )?;
        world.validate()?;
        let count = fixtures::write_captures(&out.join("fixtures"))?;
        world.snapshot().save(&out.join("final.json"))?;
        std::fs::write(
            out.join("report.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "purpose":"fixture extraction only, not performance measurement", "initial_tick":initial_tick,
                "final_tick":world.state().tick,"digest":world.digest()?,"material_gap":world.state().condensate_gap(),"fixtures":count
            }))?,
        )?;
        return Ok(());
    }
    let world = WorldSnapshot::load(std::path::Path::new(
        args.first().ok_or("world snapshot required")?,
    ))?;
    let fixture: PipeCastFixture =
        serde_json::from_slice(&std::fs::read(args.get(1).ok_or("fixture required")?)?)?;
    let out = std::path::Path::new(args.get(2).ok_or("new output required")?);
    std::fs::create_dir(out)?;
    let repeats = args.get(3).map_or(Ok(3usize), |s| s.parse())?;
    let trace = args.iter().any(|s| s == "--trace");
    let mut rows = vec![];
    for _ in 0..repeats {
        rows.push(fixtures::probe(&world, &fixture, trace)?);
    }
    std::fs::write(
        out.join("report.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "build_identity":serde_json::from_str::<serde_json::Value>(moebius_morph::BENCHMARK_BUILD_IDENTITY)?,
            "fixture":fixture,"repeats":repeats,"detailed_trace":trace,"rows":rows,"exact":true
        }))?,
    )?;
    Ok(())
}
