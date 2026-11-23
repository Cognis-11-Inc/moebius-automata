"""Verify/restore cathedral evidence using the shared immutable archive verifier."""
from pathlib import Path
import m6_artifacts

m6_artifacts.__doc__ = __doc__
m6_artifacts.STORAGE = Path(__file__).resolve().parents[1] / 'docs/cathedral/ARTIFACT_STORAGE.json'
if __name__ == '__main__':
    m6_artifacts.main()
