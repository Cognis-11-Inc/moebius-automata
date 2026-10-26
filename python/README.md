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

- `material_initial + external_input = field + reservoir + waste + external_output`
- `energy_credited = energy_live + energy_maintenance + energy_disposed`

It also checks the row count, the final hash, the live and dead counts, and the
extinction flag. A nonzero residual is a failure.
