"""Verify or restore immutable MORPH-005 evidence on Mermogon."""
from pathlib import Path
import m6_artifacts

m6_artifacts.STORAGE = Path(__file__).resolve().parents[1] / 'docs/morph_005/ARTIFACT_STORAGE.json'
if __name__ == '__main__':
    m6_artifacts.main()
