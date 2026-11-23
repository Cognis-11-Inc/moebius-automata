//! Native controller jaw audit; no diagnostic commands or gait injection.
#[path = "../src/motor_demo.rs"]
mod motor_demo;
use serde_json::json;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::Path::new("experiments/motor_land_demo.json");
    for control in ["selected", "untrained"] {
        let mut w = motor_demo::load(path, control)?;
        for t in 0..256 {
            let (_, rows) = w.tick_observed()?;
            let n = w.brains()[&1].neural.as_ref().unwrap();
            {
                println!(
                    "{}",
                    json!({"control":control,"tick":t+1,"state_hash":w.snapshot().state_hash(),"angles":&n.body.angles[24..26],"rates":&n.body.rates[24..26],"commands":&n.commands[24..26],"drive":&n.actuator_drive()[24..26],"intake":rows[0].intake})
                );
            }
        }
    }
    Ok(())
}
