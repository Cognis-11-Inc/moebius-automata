#[allow(dead_code)]
#[path = "../examples/support/m6.rs"]
mod motor;
use moebius_core::{
    SnapshotFile, World,
    neural::{ControllerGenome, NeuralRules, NeuralState},
};
#[test]
fn feedback_ablation_preserves_default_encoding_and_keeps_physiological_observations() {
    let original = motor::original();
    let mut on = NeuralState::new(original.clone(), 4096);
    let mut off = on.clone();
    let yes = NeuralRules {
        version: 2,
        ..NeuralRules::default()
    };
    let no = NeuralRules {
        physiological_feedback: false,
        ..yes.clone()
    };
    for i in 1..=64 {
        on.activations.fill(500);
        off.activations.fill(500);
        on.muscle.fill(512);
        off.muscle.fill(512);
        on.feedback(4096 - i * 6, 0, &yes);
        off.feedback(4096 - i * 6, 0, &no);
    }
    assert_eq!(off.reward, 0);
    assert_eq!(off.last_energy, 3712);
    assert!(
        off.plastic
            .iter()
            .chain(&off.plastic_output)
            .all(|w| *w == 0)
    );
    assert!(on.changed_connections > 0);
    let encoded = serde_json::to_value(NeuralRules::default()).unwrap();
    assert!(encoded.get("physiological_feedback").is_none());
    let decoded: NeuralRules = serde_json::from_value(encoded).unwrap();
    assert!(decoded.physiological_feedback);
    let invalid = NeuralRules {
        physiological_feedback: false,
        ..NeuralRules::default()
    };
    assert!(invalid.validate().is_err());
}
fn travel(g: &ControllerGenome) -> (f64, World) {
    let e = motor::Episode::held(601);
    let mut w = motor::world(g, &e, None, false).unwrap();
    let mut total = [0.; 3];
    for _ in 0..256 {
        let before = motor::position(&w);
        w.tick().unwrap();
        let d = motor::delta(before, motor::position(&w));
        for a in 0..3 {
            total[a] += d[a];
        }
        assert_eq!(w.accounts().unwrap().energy_residual(), 0);
        assert_eq!(w.accounts().unwrap().material_residual(), 0);
        assert!(w.jets().is_empty());
    }
    (motor::norm(total), w)
}
#[test]
fn selected_controller_moves_through_native_recurrent_path_without_external_commands() {
    let selected: ControllerGenome = serde_json::from_str(include_str!(
        "../../../docs/m6/checkpoints/selected_genome.json"
    ))
    .unwrap();
    let base = motor::original();
    assert_eq!(selected.anatomy, base.anatomy);
    assert_eq!(selected.strengths, base.strengths);
    assert_eq!(selected.ligament_rest, base.ligament_rest);
    let (actual, w) = travel(&selected);
    let (original, _) = travel(&base);
    assert!(actual > original + 0.25);
    assert_eq!(w.brains()[&1].neural.as_ref().unwrap().steps, 256);
    let file = SnapshotFile::decode(&SnapshotFile::encode(w.config(), &w).unwrap()).unwrap();
    let mut restored = World::restore(w.config(), &file.snapshot).unwrap();
    let mut continued = w.clone();
    for _ in 0..8 {
        restored.tick().unwrap();
        continued.tick().unwrap();
    }
    assert_eq!(
        restored.snapshot().state_hash(),
        continued.snapshot().state_hash()
    );
}
#[test]
fn local_cue_reverses_real_sensor_gradient_without_adding_material_or_impulse() {
    let g = motor::original();
    let mut e = motor::Episode::train(101);
    e.cue = Some(motor::Cue {
        axis: 1,
        first_sign: 1,
        switch_tick: 64,
    });
    let w = motor::world(&g, &e, None, false).unwrap();
    let forward = w.organism_sensors(1).unwrap();
    let reversed = motor::change_cue(&w, 1, -1).unwrap();
    let backward = reversed.organism_sensors(1).unwrap();
    assert!(forward[95] > forward[96]);
    assert!(backward[95] < backward[96]);
    assert_eq!(forward.len(), 107);
    assert_eq!(
        w.accounts().unwrap().field,
        reversed.accounts().unwrap().field
    );
    assert_eq!(motor::position(&w), motor::position(&reversed));
    assert_eq!(
        w.brains()[&1].neural.as_ref().unwrap().body,
        reversed.brains()[&1].neural.as_ref().unwrap().body
    );
    assert_eq!(reversed.accounts().unwrap().material_residual(), 0);
    assert_eq!(reversed.config().uptake_per_tick, 0);
}
#[test]
fn frozen_post_probe_retains_weights_but_resets_fast_state_and_body() {
    let g = motor::original();
    let e = motor::Episode::train(101);
    let mut state = NeuralState::new(g.clone(), 4096);
    state.plastic[0] = 32;
    state.plastic_output[0] = -16;
    state.activations.fill(700);
    state.body.impulse([128, 0, 0]);
    let probe = motor::world(&g, &e, Some(&state), false).unwrap();
    let n = probe.brains()[&1].neural.as_ref().unwrap();
    assert_eq!(n.plastic[0], 32);
    assert_eq!(n.plastic_output[0], -16);
    assert!(n.activations.iter().all(|v| *v == 0));
    assert_eq!(n.body.momentum, [0; 3]);
    assert_eq!(n.steps, 0);
}
