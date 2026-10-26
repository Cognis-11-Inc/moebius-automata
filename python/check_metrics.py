#!/usr/bin/env python3
"""Recompute M0 accounting identities from a run directory.

This is not a simulator. It reads manifest.json and metrics.csv written by
the Rust batch runner and checks those integers. It does not advance a world.
"""

from __future__ import annotations

import csv
import json
import sys
from pathlib import Path


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print("usage: check_metrics.py RUN_DIR", file=sys.stderr)
        return 2
    run = Path(argv[1])
    manifest = json.loads((run / "manifest.json").read_text(encoding="utf-8"))
    if manifest.get("action_energy_is_conserved_physical_quantity") is not False:
        print(
            "manifest must record that action-energy is not a conserved physical quantity",
            file=sys.stderr,
        )
        return 1
    initial = int(manifest["material_initial"])
    ticks = int(manifest["ticks"])
    with (run / "metrics.csv").open(encoding="utf-8", newline="") as handle:
        rows = list(csv.DictReader(handle))
    if len(rows) != ticks + 1:
        print(f"metrics rows {len(rows)} != ticks+1 ({ticks + 1})", file=sys.stderr)
        return 1
    for index, row in enumerate(rows):
        if int(row["tick"]) != index:
            print(f"metrics tick {row['tick']} is not row {index}", file=sys.stderr)
            return 1
        material = (
            initial
            + int(row["external_input"])
            - int(row["field"])
            - int(row["reservoir"])
            - int(row["waste"])
            - int(row["external_output"])
        )
        if material != 0 or int(row["material_residual"]) != 0:
            print(f"tick {index} material residual {material}", file=sys.stderr)
            return 1
        energy = (
            int(row["energy_credited"])
            - int(row["energy_live"])
            - int(row["energy_maintenance"])
            - int(row["energy_disposed"])
        )
        if energy != 0 or int(row["energy_residual"]) != 0:
            print(f"tick {index} action-energy residual {energy}", file=sys.stderr)
            return 1
    last = rows[-1]
    if last["state_hash"] != manifest["final_state_hash"]:
        print("final state hash does not match the last metrics row", file=sys.stderr)
        return 1
    if int(last["live"]) != int(manifest["final_live_count"]):
        print("final live count does not match the manifest", file=sys.stderr)
        return 1
    if int(last["dead"]) != int(manifest["final_dead_count"]):
        print("final dead count does not match the manifest", file=sys.stderr)
        return 1
    extinct = int(last["live"]) == 0
    if bool(manifest["extinct"]) != extinct:
        print("extinct flag does not match the final live count", file=sys.stderr)
        return 1
    print(
        json.dumps(
            {
                "ok": True,
                "ticks": ticks,
                "final_state_hash": manifest["final_state_hash"],
                "material_residual": 0,
                "energy_residual": 0,
                "extinct": extinct,
                "live": int(last["live"]),
            }
        )
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
