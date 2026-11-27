# Reproduce the bounded HABITAT-002 evidence

Source checkpoint and toolchain: HABITAT-002_VALIDATION.json and captured build.json.
Clean source is0d2bd8990860e6f1fee6166fa4c7bdc42958551f; final publication changes
only metadata/reports. Compiler/feature/source identity is enforced, not bypassed.

1. Verify Mermogon mount/UUID D0A02EEBA02ED82C. Read STORAGE and QUALIFICATION_STORAGE
   receipts; hash each archive/manifest. Extract each into a separate NEW directory
   and verify every member size/hash. Do not overwrite an immutable archive.
2. Implementation-v1 contains original and certified Rust executables/source
   archives, prequalification failures/pilots and Python probe source snapshots.
   Qualification-v1 is a delta with final datasets/scripts/figures/Atlas evidence.
   If needed, assemble a third working directory by copying base then supplement;
   keep both verified restores separate. Cargo targets/Python caches are excluded.
3. FINAL_PLAN.json and FINAL_RUNS.json retain168 exact argument lists and183 initial
   commands/results. Extra original001 repeat, corrected render overlap and GPU
   program capture have separate receipts. The initial log-wait contention protocol
   is INVALID as overlap evidence; use VERIFIED_RENDER_OVERLAP.json and its script.
4. Execute a matching restored CPU checkpoint, e.g.:

```sh
ordinary-certified/habitat_scaling.binary replay demo-reference-Buffered-e4-w1-ordinary/initial.json NEW_FINAL.json 120 Buffered 1
```

   Compare complete JSON checkpoint with its saved final.json. Lane and worker
   arguments must match the record. Wrong source/backend/lane/worker records must
   refuse. No old001-to002 checkpoint migration is provided.
5. GPU invocation (fixture source/build identity checked; detached carrier only):

```sh
python3 habitat_002_gpu.py GPU_CASE/initial.json GPU_CASE/final.json ordinary-certified/build.json NEW_GPU_OUTPUT
```

   Use the archived matching Python source (implementation-docs) and installed
   OpenCL device/driver/FP64 support. Kernel/runner/device/compiler identity and
   payload hashes define the GPU restoration contract. A different driver is a
   new qualification, not an exact-repeat assertion. Never reinterpret the carrier
   fixture as a full atmospheric checkpoint.
6. Analyze without the viewer: all rows.json, config.json, initial/final f64-bit
   checkpoints and GPU reports are plain JSON/binary. `analyze.py` performs168-case
   statistics and133 paired tick-digest/decoded-state comparisons; then
   `enrich_analysis.py` adds fresh original and corrected-overlap evidence;
   `plot_results.py` emits standalone SVG/PNG/PDF. On a different machine, change
   only each analysis script's evidence-root variable R to the assembled working
   copy and the explicit legacy001 archive root if rechecking historical state.
   These are analysis-path edits, not permission to change physical inputs or guards.
   plot-requirements.txt records plotting dependencies. Do not execute publication
   helper scripts against an unrelated source repository.
7. Preserve nulls: C/D reference budget failures, slower worker controls, support
   saturation, exact-deactivation limits, scalar-interface fidelity error, missing
   cache counters/total NVIDIA VRAM/OOM/same-GPU contention, and failed initial
   overlap protocol are part of the result. The default CPU step remains A;
   qualification does not imply canonical adoption.
