"""Compare saved Postgres configurations and worker determinism from the repo root."""
import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
import os
from pathlib import Path
import subprocess

cli = argparse.ArgumentParser()
cli.add_argument("baseline", type=Path)
cli.add_argument("candidate", type=Path)
cli.add_argument("--out", type=Path, required=True)
args = cli.parse_args()
args.out.mkdir(parents=True, exist_ok=True)
here = Path(__file__).resolve().parent
manifests = [here.parent / "postgres-ddl-release-blocker-2734/configs.tsv",
             here / "configs-extra.tsv"]
inputs = sorted({tuple(line.split("\t")) for manifest in manifests
                 for line in manifest.read_text().splitlines()})


def inspect(row):
    root, config = row
    key = hashlib.sha256((root + "|" + config).encode()).hexdigest()[:16]
    runtime = args.out / ("runtime-" + key)
    runtime.mkdir(exist_ok=True)
    observations = []
    rules = set()
    for phase, binary, jobs in [("before", args.baseline, None),
                                ("after", args.candidate, None),
                                ("jobs1", args.candidate, "1"),
                                ("jobs4", args.candidate, "4")]:
        command = [str(binary.resolve()), "check", "--root", str(Path(root).resolve()),
                   "--config", config, "--format", "json"]
        if jobs:
            command.extend(["--jobs", jobs])
        result = subprocess.run(command, capture_output=True,
                                env=dict(os.environ, XDG_RUNTIME_DIR=str(runtime.resolve())))
        for stream in ["stdout", "stderr"]:
            (args.out / (phase + "-" + key + "." + stream)).write_bytes(getattr(result, stream))
        observations.append((result.returncode, result.stdout, result.stderr))
        if phase == "after" and result.returncode in [0, 1]:
            report = json.loads(result.stdout)
            rules.update(f["rule"] for f in report["rules"] + report["advisories"]
                         if f.get("rule", "").startswith("postgres-"))
    return {"root": root, "config": config, "status": observations[1][0],
            "baselineIdentical": observations[0] == observations[1],
            "workersIdentical": observations[1] == observations[2] == observations[3],
            "rules": sorted(rules)}


with ThreadPoolExecutor(max_workers=4) as pool:
    results = list(pool.map(inspect, inputs))
(args.out / "summary.json").write_text(json.dumps(results, indent=2) + "\n")
print(json.dumps({"configurations": len(results),
                  "baselineChanges": sum(not r["baselineIdentical"] for r in results),
                  "workerDifferences": sum(not r["workersIdentical"] for r in results),
                  "rulesObserved": sorted({rule for r in results for rule in r["rules"]})}, indent=2))
assert all(row["workersIdentical"] for row in results)
