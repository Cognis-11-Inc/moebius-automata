"""Audit instrumentation replay and parent/offspring records without simulating.

Usage: python audit_neural_replay.py MAIN_RUN REPLAY_RUN ARTIFACT_DEST
The caller must preserve both run directories. State hashes here are compared,
not independently recomputed. Genetic structures and command probes are compared
directly from the exported integer records.
"""
import gzip
import hashlib
import json
from pathlib import Path
import sys


def audit(main, replay, dest):
    before = json.loads((main / "results.json").read_text())
    after = json.loads((replay / "results.json").read_text())
    key = lambda r: (r["environment"], r["condition"], r["seed"])
    originals = {key(r): r for r in before["results"]}
    repeated = {key(r): r for r in after["results"]}
    assert originals.keys() == repeated.keys()
    # Only elapsed wall-clock time may differ. No dynamics changed for logging.
    for k in originals:
        clean = lambda r: {f: v for f, v in r.items() if f != "elapsed_seconds"}
        assert clean(originals[k]) == clean(repeated[k]), k
    summaries = []
    for path in sorted(replay.glob("lineage_evidence_*.jsonl")):
        condition, seed = path.stem.removeprefix("lineage_evidence_").rsplit("_", 1)
        records = [json.loads(line) for line in path.read_text().splitlines()]
        by_id = {r["id"]: r for r in records}
        assert len(by_id) == len(records)
        descendants = [r for r in records if r["parent"] != 0]
        changed_genome = changed_probe = 0
        for child in descendants:
            parent = by_id[child["parent"]]
            assert parent["tick"] <= child["tick"]
            assert child["generation"] == parent["generation"] + 1
            structural_change = child["genome"] != parent["genome"]
            assert structural_change == (child["genotype_id"] != parent["genotype_id"])
            changed_genome += structural_change
            changed_probe += child["fresh_state_command_probe"] != parent["fresh_state_command_probe"]
        expected = originals[("lineage", condition, int(seed))]
        assert len(descendants) == expected["descendants_measured"]
        assert changed_genome == expected["changed_germline_descendants"]
        assert changed_probe == expected["common_garden_different_descendants"]
        if condition == "mutation_off":
            assert changed_genome == changed_probe == 0
        data = path.read_bytes()
        with (dest / (path.name + ".gz")).open("wb") as raw:
            with gzip.GzipFile(fileobj=raw, mode="wb", mtime=0, filename="") as out:
                out.write(data)
        summaries.append({"file": path.name + ".gz", "uncompressed_sha256": hashlib.sha256(data).hexdigest(),
                          "condition": condition, "seed": int(seed), "records": len(records),
                          "descendants": len(descendants), "changed_genotypes": changed_genome,
                          "changed_command_probes": changed_probe})
    assert len(summaries) == 16
    report = {"status": "PASS", "main": str(main), "instrumented_replay": str(replay),
              "matched_trials": len(originals), "comparison": "all result fields except elapsed_seconds identical",
              "scope": "exported parent-child genetic structures and fresh-state command probes independently compared; state hashes compared, not independently recomputed; no adaptation verdict", "lineages": summaries}
    (dest / "REPLAY_AUDIT.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"status": "PASS", "matched_trials": len(originals), "lineage_records": sum(s["records"] for s in summaries)}))


if __name__ == "__main__":
    audit(*(Path(p) for p in sys.argv[1:]))
