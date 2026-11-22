//! Controlled actuator interventions; these are assays, never controller gaits.
use moebius_core::neural::{
    Q,
    body::Body,
    schema::{Anatomy, Axis},
};
use serde_json::json;
fn main() {
    let anatomy = Anatomy::reference();
    let strengths = vec![Q; anatomy.motors()];
    let mut results = Vec::new();
    for version in [1, 2] {
        for drag in [0, 8, 128] {
            for assay in ["bend_positive", "bend_negative", "twist", "jaw", "coast"] {
                let mut body = Body::new(&anatomy);
                let mut environment = [0i64; 3];
                if assay == "coast" {
                    body.impulse([2048, 0, 0]);
                    environment[0] = -2048;
                }
                let mut position = [0f64; 3];
                let mut path = 0f64;
                let mut peak_speed = 0f64;
                let mut max_angular = 0;
                let mut speed_at_release = 0f64;
                let mut ledger_max = 0i64;
                let mut observations = Vec::new();
                for tick in 0..128 {
                    let mut command = vec![0; anatomy.motors()];
                    if tick < 16 {
                        for (i, b) in anatomy.axes.iter().enumerate() {
                            command[i] = match (assay, b.axis) {
                                ("bend_positive", Axis::Bend)
                                | ("twist", Axis::Twist)
                                | ("jaw", Axis::Jaw) => Q,
                                ("bend_negative", Axis::Bend) => -Q,
                                _ => 0,
                            };
                        }
                    }
                    let reaction = body.step_with_law(
                        &anatomy,
                        &command,
                        &strengths,
                        drag,
                        &[],
                        6,
                        &[],
                        version,
                    );
                    for a in 0..3 {
                        environment[a] += reaction[a];
                        ledger_max = ledger_max.max((environment[a] + body.momentum[a]).abs());
                        position[a] += f64::from(body.velocity[a]) / 1024.;
                    }
                    let speed = body
                        .velocity
                        .iter()
                        .map(|v| f64::from(*v).powi(2))
                        .sum::<f64>()
                        .sqrt()
                        / 1024.;
                    path += speed;
                    peak_speed = peak_speed.max(speed);
                    max_angular =
                        max_angular.max(body.angular_velocity.iter().map(|v| v.abs()).sum::<i32>());
                    if tick == 15 {
                        speed_at_release = speed;
                    }
                    observations.push(
                        json!({"tick":tick,"velocity":body.velocity,"momentum":body.momentum,
                "angles":body.angles,"rates":body.rates,"orientation":body.orientation}),
                    );
                }
                results.push(json!({"version":version,"drag":drag,"assay":assay,"path":path,"position":position,
            "peak_speed_sites_per_tick":peak_speed,"speed_at_release":speed_at_release,
            "speed_final":body.velocity.iter().map(|v|f64::from(*v).powi(2)).sum::<f64>().sqrt()/1024.,
            "maximum_angular_rate":max_angular,"maximum_momentum_residual":ledger_max,"observations":observations}));
            }
        }
    }
    println!("{}",serde_json::to_string_pretty(&json!({"description":"16 ticks imposed command then release; coast begins with an explicit external impulse. No metabolic or navigation test.","results":results})).unwrap());
}
