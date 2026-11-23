"""Verify/restore immutable MORPH-003 evidence on Mermogon."""
from pathlib import Path
import m6_artifacts

m6_artifacts.STORAGE = Path(__file__).resolve().parents[1] / "docs/morph_003/ARTIFACT_STORAGE.json"
if __name__ == "__main__":
    m6_artifacts.main()
