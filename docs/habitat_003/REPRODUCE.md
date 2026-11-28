# Reproduction boundaries

Use the exact captured executable/source/backend/feature identity associated with
the checkpoint. Source changes intentionally refuse old003 checkpoints. Source
capsules contain Rust/config and separate archived Python/report snapshots; Cargo
registry/toolchain are external offline prerequisites. Do not import solver code.

Focused gate: `cargo test --offline -p moebius-morph --all-features --lib scientific`.
Static gate: `cargo fmt --all -- --check` and
`cargo clippy --offline --workspace --all-targets --all-features -- -D warnings`.
Python audit tests: `python3 -m unittest discover -s python -p 'test_habitat_003_integrity.py'`.
Broad gate only on clean recorded source:
`cargo test --offline --workspace --all-targets --all-features`.

Build/capture by reusing `python/benchmark_integrity.py` build_capture with
example `habitat_scientific`, feature habitat-experiment. A second build adds
allocation-attribution; its timing is diagnostic and not an ordinary speed claim.
`python3 python/habitat_003_qualification.py plan` prints the registered74-case plan;
`run RAW_ROOT` expects ordinary-certified and counted-certified captured binaries.
Raw runs never overwrite existing directories. `analyze RAW_ROOT` independently
recomputes endpoint inventories/partition and uniform-fine field comparisons.
All outputs stay external; compact summaries belong in Git.

Example command shape: `habitat_scientific.binary NEW_OUT Diffusion FiniteVolume
32 0.0002 1000 refined subcycle 1`. Optional tenth argument is coarseh in metres.
`replay INITIAL_JSON NEW_FINAL_JSON TICKS` resumes synchronized monorate state;
pending continuation is exercised by the recorded subcycle protocol and unit test.
A rejected experimental step may produce exit0 with a REJECTED summary: consumers
MUST inspect completed_cycles and failure, not process exit alone. Raw summary
includes rejected candidate. `restore_exact` concerns complete payload restoration;
continuation checks are separately recorded. Native Witness/GPU are NOT RUN.

For external artifacts verify Mermogon mount UUID D0A02EEBA02ED82C on this run,
archive/manifest SHA256, and each restored member hash/size; execute the restored
captured binary against its matching checkpoint. STORAGE records authoritative
versions and working-copy paths. Never overwrite a version or discard old evidence.
