# Native mouth motion correction — 2026-11-22

The close-up initialized by pausing the native world at tick16. Keep-open screenshots independently paused it. These viewer paths now leave ticking running; Space controls pause. The skin and neural controller were unchanged. A food-free demo can also die after exhausting energy; the HUD identifies that state instead of suggesting an active mouth.

## Evidence

The selected native controller was sampled every tick for256 ticks, alongside an untrained control. An independent check uses1172 real GLB vertices whose summed jaw weights exceed0.05, with spinal/body motion removed. Selected left/right jaw ranges were[0,768] and[-256,698] fixed-point units. Maximum lip displacement was0.128440 mesh units; maximum RMS displacement0.057975. The untrained control also moved initially (maximum0.062672), then held against limits. This is native actuation, not an idle chewing loop.

Sixteen internal Bevy screenshots at ticks16,32,…256 visibly show changing mouth geometry. Every associated committed state hash and jaw angle matches the headless native replay (16/16). Render interpolation angles are recorded separately. The short GIF is a witness assembled from these frames, not a real-time video or proof of feeding. Intake remained0 in this food-free run. No learning, target reaching or new ingestion result is claimed.

Earlier window-recording attempts failed: Wayland lookup could not find the window, and X11 window capture returned stale/background images or the later dead demo. All attempts and logs remain in the v2 archive as failed witnesses. Internal screenshots resolve the capture ambiguity; actual weighted-vertex analysis isolates jaw movement from body/camera motion.

## Reproduce

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
cargo run --release -p moebius-viewer --example jaw_probe > native.jsonl
python3 scripts/check_worm_jaws.py native.jsonl
cargo run --release -p moebius-viewer -- --motor-nursery experiments/motor_land_demo.json --motor-control selected --mouth-view --record-mouth NEW_DIRECTORY
python3 python/worm_a_artifacts.py verify
python3 python/worm_a_artifacts.py unpack NEW_RESTORE_DIRECTORY
```

106 workspace tests pass, including regressions for native deformation of real asset vertices, entering the close-up without pausing, and keeping captures open without pausing. Strict Clippy and formatting pass. Source identity, per-artifact sizes/purposes and archive/member hashes are in the adjacent JSON metadata. Immutable v1 remains historical; v2 preserves the correction, raw sequence, native telemetry, source and failures. Production asset hashes are unchanged. Work remains local and uncommitted.
