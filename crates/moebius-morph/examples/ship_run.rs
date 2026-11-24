//! General headless ship runner. OUT TICKS [INITIAL_SNAPSHOT_JSON] [INTERVENTIONS_JSON].
//! Without initial state use the recorded16-instance fixture. No renderer or wall-clock input.
use moebius_morph::runtime::*;
use std::path::PathBuf;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if !(2..=4).contains(&args.len()) {
        return Err(
            "usage: ship_run NEW_OUTPUT TICKS [INITIAL_SNAPSHOT_JSON] [INTERVENTIONS_JSON]".into(),
        );
    }
    let out = PathBuf::from(&args[0]);
    let ticks = args[1].parse::<u64>()?;
    let mut world = if let Some(path) = args.get(2) {
        ShipRuntime::restore(WorldSnapshot::load(&PathBuf::from(path))?)?
    } else {
        fixture(16, 20261123)?
    };
    let commands: Vec<Intervention> = if let Some(path) = args.get(3) {
        serde_json::from_slice(&std::fs::read(path)?)?
    } else {
        vec![]
    };
    std::fs::create_dir(&out)?;
    world.snapshot().save(&out.join("initial.json"))?;
    std::fs::write(
        out.join("interventions.json"),
        serde_json::to_vec_pretty(&commands)?,
    )?;
    for c in commands {
        world.submit(c)?;
    }
    let start = world.state().tick;
    let initial_digest = world.digest()?;
    world.advance(ticks)?;
    world.validate()?;
    world.snapshot().save(&out.join("final.json"))?;
    let report = serde_json::json!({"start_tick":start,"steps":ticks,"final_tick":world.state().tick,"initial_admitted_digest":initial_digest,"final_digest":world.digest()?,"tick_nanos":TICK_NANOS,"definition":"initial.json","interventions":"interventions.json","snapshot":"final.json","owner":"MORPH-004 canonical free-space runtime; nursery is a separate experiment"});
    std::fs::write(
        out.join("manifest.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    println!("{report}");
    Ok(())
}
