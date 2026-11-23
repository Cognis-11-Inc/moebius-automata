"""Read-only MORPH-002 Atlas provenance/link validation. Never substitutes repository storage."""
import hashlib
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]
REFERENCE = ROOT / "docs/morph_002/ATLAS_REFERENCE.json"

def verify():
    ref = json.loads(REFERENCE.read_text())
    mount = subprocess.check_output(["findmnt", "-J", "-T", ref["atlas_path"], "-o", "TARGET,UUID"], text=True)
    volume = json.loads(mount)["filesystems"][0]
    if volume["uuid"].lower() != ref["volume_uuid"].lower():
        raise ValueError("Atlas drive UUID does not match; no substitution permitted")
    vault = Path(ref["atlas_path"])
    notes = sorted(vault.rglob("*.md"))
    errors = []
    links = 0
    for path in notes:
        text = path.read_text()
        if not text.startswith("---\n"):
            errors.append(f"Missing frontmatter: {path}")
        for field in ["status:", "type:", "evidence_status:", "last_verified:"]:
            if field not in text:
                errors.append(f"Missing {field}: {path}")
        for target in re.findall(r"\[\[([^\]]+)\]\]", text):
            target = target.split("|")[0].split("#")[0]
            if target and not (vault / f"{target}.md").is_file():
                errors.append(f"Dangling link {target} in {path.name}")
            links += 1
        if path.parent.name != "_TEMPLATES":
            for target in re.findall(r"\]\((file://[^)]+)\)", text):
                from urllib.parse import unquote, urlparse
                if not Path(unquote(urlparse(target).path)).exists():
                    errors.append(f"Missing repository reference {target}")
    for entry in ref.get("snapshot_members", []):
        path = vault / entry["path"]
        if not path.is_file() or hashlib.sha256(path.read_bytes()).hexdigest() != entry["sha256"]:
            errors.append(f"Snapshot drift: {entry['path']}")
    result = {"status": "FAIL" if errors else "PASS", "atlas_path": str(vault), "notes": len(notes), "internal_links": links, "errors": errors, "volume_uuid": volume["uuid"], "role": "interpretive navigation; live repository and independently versioned evidence authoritative"}
    print(json.dumps(result, indent=2))
    if errors:
        raise SystemExit(1)

if __name__ == "__main__":
    verify()
