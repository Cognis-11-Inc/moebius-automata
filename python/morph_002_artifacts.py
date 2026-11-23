"""Verify/restore immutable MORPH-002 evidence on Mermogon."""
from pathlib import Path
import m6_artifacts
m6_artifacts.STORAGE = Path(__file__).resolve().parents[1] / "docs/morph_002/ARTIFACT_STORAGE.json"
if __name__ == "__main__":
    m6_artifacts.main()
