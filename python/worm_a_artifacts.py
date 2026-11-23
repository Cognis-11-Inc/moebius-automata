"""Verify/restore local Worm_A rigging evidence from its immutable archive."""
from pathlib import Path
import m6_artifacts
m6_artifacts.__doc__ = __doc__
m6_artifacts.STORAGE = Path(__file__).resolve().parents[1] / 'docs/worm_a/ARTIFACT_STORAGE.json'
if __name__ == '__main__':
    m6_artifacts.main()
