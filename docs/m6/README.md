# M6 motor nursery

Read [REPORT.md](REPORT.md), [AUDIT.md](AUDIT.md), [PROTOCOL.json](PROTOCOL.json), [ANALYSIS.json](ANALYSIS.json), [VALIDATION.json](VALIDATION.json), and [ARTIFACT_STORAGE.json](ARTIFACT_STORAGE.json).

Run the native viewer from the repository root:

```sh
cargo run --release -p moebius-viewer -- --motor-nursery docs/m6/DEMO.json --motor-control compare
```

ID1 is genetically selected; ID2 is the untrained factory genome. `--motor-control selected` and `--motor-control untrained` open solo modes. `N` opens the read-only neural inspector; `Tab` switches its pages, Space pauses. The demo is explicitly food-free with4096 initial energy per organism and frozen plastic weights. It does not alter ordinary startup. Selection improved locomotion; useful lifetime motor learning and steering remain unproven.

Capture an actual window frame with `--capture-frame NEW.png --capture-tick 128`; add `--inspect` for the inspector. Very early captures can precede mesh/material readiness (the blank early trial is preserved). Requested tick is a lower bound, not an exact render frame; capture metadata and HUD show committed/rendered ticks.

Build diagnostic examples and restore evidence:

```sh
cargo build --release -p moebius-core --example m6_nursery --example m6_replay
python3 python/m6_artifacts.py verify
python3 python/m6_artifacts.py unpack /path/to/NEW_RESTORE
python3 python/m6_analysis.py /path/to/NEW_RESTORE /path/to/recomputed-analysis.json
```

`--archive-root COPY_OF_ARCHIVE_ROOT` permits a portable verified archive copy on another machine. The default verifies Mermogon UUID before access. Unpacking refuses existing destinations and verifies every restored file.

Each stage refuses existing output directories. Example commands below use NEW directories outside the repository:

```sh
target/release/examples/m6_nursery capability /cache/NEW_CAP
target/release/examples/m6_nursery babble /cache/NEW_BABBLE
target/release/examples/m6_nursery optimize /cache/NEW_OPT /cache/NEW_CAP/measurements.json
target/release/examples/m6_nursery assess /cache/NEW_HELD /cache/NEW_OPT/selected_genome.json
target/release/examples/m6_nursery lifetime /cache/NEW_LIFE /cache/NEW_OPT/selected_genome.json
target/release/examples/m6_nursery direction /cache/NEW_DIR /cache/NEW_OPT/selected_genome.json /cache/NEW_HELD/measurements.json
target/release/examples/m6_nursery direction_assess /cache/NEW_DIR_HELD /cache/NEW_DIR/selected_genome.json /cache/NEW_OPT/selected_genome.json
target/release/examples/m6_nursery falsify /cache/NEW_FALSIFY /cache/NEW_OPT/selected_genome.json
target/release/examples/m6_replay /path/to/NEW_RESTORE/heldout-v1/episodes.jsonl.gz
```

Current cue constructor reproduces v2. Invalid v1 is replayed from its recorded field arrays, not silently regenerated using v2. Wall times may differ; scores, genotypes, commands, trajectories and state hashes are deterministic. Gate checks do not replace reading the protocol: do not enlarge the budgets or advance blocked curriculum stages without a new explicit experiment.

Mechanism decomposition: `cargo run --release -p moebius-core --example m6_mechanism -- RESTORE/heldout-v1/episodes.jsonl.gz`. See MECHANISM.json and the report correction; the superseded lowercase-axis analysis is preserved. The current archive index spans immutable v1 raw archives and v2 correction/source addenda. Copy the entire `M6/8e351e2` directory for portable verification.
