"""Run the registered SIMULATION-0001 observations through the existing recorder."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import shutil

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(__doc__)
    parser.add_argument("output", type=Path)
    parser.add_argument("--protocol", type=Path, default=ROOT / "docs/morph_005/SIMULATION_0001_EXPERIMENT.json")
    args = parser.parse_args()
    plan = json.loads(args.protocol.read_text())
    binary = ROOT / "target/release/examples/habitat_probe"
    if not binary.is_file():
        raise SystemExit("Build first: cargo build --release --offline -p moebius-morph --example habitat_probe")
    args.output.mkdir(parents=True, exist_ok=False)
    captured = args.output / "habitat_probe.binary"
    shutil.copy2(binary, captured)
    binary = captured.resolve()
    receipt = {
        "protocol": plan,
        "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
        "head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        "git_status": subprocess.check_output(["git", "status", "--short"], cwd=ROOT, text=True),
        "cases": [],
    }
    cases = []
    for phase in ["longitudinal", "scaling"]:
        config = plan[phase]
        for founders in config["founders"]:
            for seed in plan["seeds"]:
                cases.append((phase, founders, seed, config["ticks"], config["wall_seconds"]))
    for founders in plan["refused_targets"]:
        cases.append(("refusal", founders, plan["seeds"][0], 1, 20))
    for phase, founders, seed, ticks, wall in cases:
        name = f"{phase}-f{founders}-s{seed}"
        out = args.output / name
        command = [str(binary), str(out), str(ticks), "--founders", str(founders), "--seed", str(seed),
                   "--abundance", str(plan["abundance"]), "--wall-seconds", str(wall)]
        print(f"START {name}", flush=True)
        with (args.output / f"{name}.log").open("w") as log:
            result = subprocess.run(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, check=False)
        report_path = out / "report.json"
        report = json.loads(report_path.read_text()) if report_path.is_file() else None
        row = {"name": name, "command": command, "exit_code": result.returncode, "report": report,
               "expected_refusal": phase == "refusal"}
        receipt["cases"].append(row)
        (args.output / "summary.json").write_text(json.dumps(receipt, indent=2) + "\n")
        print(f"END {name} exit={result.returncode} ticks={report.get('ticks') if report else None}", flush=True)


if __name__ == "__main__":
    main()
