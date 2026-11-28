# Reproduction

Experimental feature only, no canonical integration. Preserve source/backend/schema
and explicit integrator recipe. New checkpoint schema HABITAT-004/native-f64-v2
refuses003 headers; use old003 captured binary for historical data. Default004
constant/Euler physical state is compatibility-tested against003 after removal
of declared new metadata/zero-ledger fields. No hidden state migration exists.

Focused: cargo test --offline -p moebius-morph --all-features --lib scientific.
Hardware tests: cargo test --offline -p moebius-morph --features habitat-experiment
--example habitat_hardware. Static: cargo fmt --all -- --check;
cargo clippy --offline --workspace --all-targets --all-features -- -D warnings.
Python: python3 -m unittest discover -s python -p test_habitat_004_integrity.py.
Broad certification requires clean recorded source:
cargo test --offline --workspace --all-targets --all-features.

Reuse benchmark_integrity.build_capture for habitat_fidelity ordinary/count-attribution
and habitat_hardware ordinary binaries. All compiler/source identities are embedded.
The capture contains Rust/config; separate snapshots retain Python/reports. Offline
Cargo registry/toolchain are external prerequisites. Do not claim a complete rebuild
without them. Plan: python3 python/habitat_004_qualification.py plan (121 recipes).
Run expects RAW_ROOT/{ordinary,counted}-certified/habitat_fidelity.binary; new raw
output directories only. Analyze independently recomputes endpoint budgets, analytic
errors and characteristic scattering. Process exit0 is not PASS: inspect completed,
failure, scientific criteria and dependency matrix.

Recorder: habitat_fidelity.binary NEW_OUT SPEC_JSON. Replay:
habitat_fidelity.binary replay INITIAL NEW_FINAL CYCLES mono|cached|subcycle.
Integrator/configuration are persisted; explicit synchronization command remains
part of the recipe. Caches are derived/rebuilt, not persisted material. Rejected
candidate evidence is raw and committed state stays unchanged. Hardware executable:
habitat_hardware.binary NEW_JSON [workers-only]. GPU probe:
python3 python/habitat_004_hardware_gpu.py NEW_OUT. Neither microbenchmark is FV.

Restore: reverify Mermogon UUID D0A02EEBA02ED82C on this run, archive/manifest hashes
and every restored member hash/size; execute matching binary/checkpoint. Old003 fresh
working restore is excluded as duplicate cache; ENTRY identifies its authoritative
prior archive and760 verified members. No archive overwritten or prior copy removed.
Figures are offline standard plots; no native Witness capture is implied.

Final build directories are selected by RAW_ROOT/BUILD_SELECTION.json (v2 capsules).
Version1 source/binaries and implementation archive remain retained after a reference
correction before qualification. Full workspace source archive includes compile-time
non-Rust inputs; selected numerical-target rebuild/restored execution is verified
separately from the full-workspace suite on the live clean source.

Final certified clean source: d9834e609f220d25e85cb937f69b186dbb94c1a7. Full suite304 passed, then121 cases and hardware probes ran with Git clean before/after. To reproduce from a restored capsule, create a new working root, copy the three selected build directories and BUILD_SELECTION.json into it, then run `python3 python/habitat_004_qualification.py run NEW_ROOT`. Do not overwrite the archived qualification directory. Run `analyze` on closed completed traces; a live writer may have an incomplete trailing line. Independent audits use Python stdlib; figures use the separately recorded Matplotlib environment. Final ordinary versus allocation-instrumented identities are explicit per case.
