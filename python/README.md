# Python analysis stack

This environment does not run the simulation. The tick lives in `moebius-core`.
Python reads committed manifests and metrics, or holds later analysis tools.

The virtual environment is CPython 3.13 because TensorFlow does not publish a
wheel for the host's Python 3.14. Create it from the repository root:

```bash
uv python install 3.13
uv venv --python 3.13 .venv
uv pip install --python .venv/bin/python --only-binary :all: -r python/requirements.txt
```

`.venv/` is gitignored. `python/requirements.lock.txt` is the set that installed.
Packages include NumPy, SciPy, pandas, PyTorch, and TensorFlow. None of them is
an authority for a state hash.

Check a finished run:

```bash
.venv/bin/python python/check_metrics.py runs/m0_64_a
```

The checker recomputes two identities with Python integers:

- `material_initial + external_input = field + flesh + in_flight + reservoir + waste + external_output`
- `energy_credited = energy_live + energy_maintenance + energy_disposed`

It also checks the row count, the final hash, the live and dead counts, and the
extinction flag. A nonzero residual is a failure.

`flesh` and `in_flight` default to zero for historical metrics.

`analyze_neural_study.py RUN --dest DEST` preserves study data, computes matched
seed comparisons and exploratory uncertainty, independently parses the three
saved final snapshots for material/energy/linear momentum, and plots observed
neural/joint/velocity traces. It does not independently recompute BLAKE3 or prove
learning. Matplotlib is optional; numerical analysis uses the standard library.

`audit_neural_replay.py MAIN REPLAY DEST` checks instrumentation-only replay
against all trial results and independently compares exported parent-child
genomes and fresh-state command probes. It archives compressed lineage records;
these records show inherited divergence, not adaptive improvement.
