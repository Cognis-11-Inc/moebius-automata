//! Explicit jaw exercise audit; outputs imposed commands, energy and replay hashes.
#[path = "../src/jaw_exercise.rs"]
mod jaw_exercise;
#[path = "../src/motor_demo.rs"]
mod motor_demo;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut world = motor_demo::load(
        std::path::Path::new("experiments/motor_land_demo.json"),
        "selected",
    )?;
    for _ in 0..256 {
        let controls = jaw_exercise::controls(&world);
        world.tick_controlled(&controls)?;
        let n = world.brains()[&1].neural.as_ref().unwrap();
        let a = world.accounts()?;
        println!(
            "{}",
            serde_json::json!({"tick":world.tick_index(),"state_hash":world.snapshot().state_hash(),"angles":&n.body.angles[24..26],"commands":controls[&1],"energy":world.cells().next().unwrap().energy,"energy_residual":a.energy_residual().to_string(),"material_residual":a.material_residual().to_string(),"externally_imposed_motor_commands":true})
        );
    }
    Ok(())
}
