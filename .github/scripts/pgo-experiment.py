"""Temporary balanced comparison of matched release and PGO binaries."""

import hashlib
import json
import os
from pathlib import Path
import subprocess


ROOT = Path.cwd()
BASE = Path(os.environ["RUNNER_TEMP"]) / "no-mistakes-pgo"
OUTPUT = BASE / "result" / "measurements"
OUTPUT.mkdir(parents=True, exist_ok=True)
BINARIES = {
    "baseline": BASE / "baseline" / "no-mistakes",
    "pgo": BASE / "result" / "no-mistakes-pgo",
}
WORKLOADS = {
    "self-check": ["check", "--root", str(ROOT), "--format", "json"],
    "graph-gates": ["check", "--root", str(ROOT / "fixtures/performance/graph-gates"), "--format", "json"],
}


def digest(data):
    return hashlib.sha256(data).hexdigest()


records = []
for workload, args in WORKLOADS.items():
    for trial, variant in enumerate(("baseline", "pgo", "pgo", "baseline"), 1):
        binary = BINARIES[variant]
        binary.chmod(0o755)
        prefix = OUTPUT / f"{workload}-{variant}-{trial}"
        result = subprocess.run(
            ["/usr/bin/time", "-f", "elapsed=%e\nuser=%U\nsystem=%S\nrss_kib=%M\nexit=%x",
             "-o", str(prefix) + ".time", "timeout", "--kill-after=5s", "240s", str(binary), *args],
            cwd=ROOT, capture_output=True, check=False,
        )
        (Path(str(prefix) + ".stdout")).write_bytes(result.stdout)
        (Path(str(prefix) + ".stderr")).write_bytes(result.stderr)
        metrics = dict(line.split("=", 1) for line in Path(str(prefix) + ".time").read_text().splitlines() if "=" in line)
        record = {
            "variant": variant, "workload": workload, "trial": trial, "command": args,
            "exit": result.returncode, "stdout_sha256": digest(result.stdout),
            "stderr_sha256": digest(result.stderr), "binary_sha256": digest(binary.read_bytes()),
            **{key: float(metrics[key]) for key in ("elapsed", "user", "system", "rss_kib")},
        }
        records.append(record)
        print(json.dumps(record), flush=True)
    parity = {(r["exit"], r["stdout_sha256"], r["stderr_sha256"]) for r in records if r["workload"] == workload}
    if len(parity) != 1:
        raise SystemExit(f"Output parity failure: {workload}: {parity}")

(OUTPUT / "samples.json").write_text(json.dumps(records, indent=2))
