"""Temporary same-runner release-codegen experiment."""

import hashlib
import json
import os
from pathlib import Path
import subprocess


ROOT = Path.cwd()
ARTIFACTS = Path(os.environ["RUNNER_TEMP"]) / "codegen-experiment"
OUTPUT = ARTIFACTS / "measurements"
OUTPUT.mkdir(parents=True, exist_ok=True)
WORKLOADS = {
    "self-check": ["check", "--root", str(ROOT), "--format", "json"],
    "graph-gates": ["check", "--root", str(ROOT / "fixtures/performance/graph-gates"), "--format", "json"],
}
ORDER = ["baseline", "candidate", "candidate", "baseline"]


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def run(variant, workload, args, trial):
    binary = ARTIFACTS / f"codegen-{variant}" / "no-mistakes"
    binary.chmod(0o755)
    prefix = OUTPUT / f"{workload}-{variant}-{trial}"
    command = [
        "/usr/bin/time", "-f", "elapsed=%e\nuser=%U\nsystem=%S\nrss_kib=%M\nexit=%x",
        "-o", str(prefix) + ".time", "timeout", "--kill-after=5s", "240s", str(binary), *args,
    ]
    result = subprocess.run(command, cwd=ROOT, capture_output=True, check=False)
    (Path(str(prefix) + ".stdout")).write_bytes(result.stdout)
    (Path(str(prefix) + ".stderr")).write_bytes(result.stderr)
    metrics = dict(line.split("=", 1) for line in Path(str(prefix) + ".time").read_text().splitlines() if "=" in line)
    return {
        "variant": variant, "workload": workload, "trial": trial,
        "command": args, "exit": result.returncode,
        "stdout_sha256": sha256(result.stdout), "stderr_sha256": sha256(result.stderr),
        "binary_sha256": sha256(binary.read_bytes()),
        **{key: float(metrics[key]) for key in ("elapsed", "user", "system", "rss_kib")},
    }


records = []
for workload, args in WORKLOADS.items():
    for candidate in ("size", "thin"):
        for trial, selected in enumerate(ORDER, 1):
            variant = "baseline" if selected == "baseline" else candidate
            sample = run(variant, workload, args, trial)
            records.append(sample)
            print(json.dumps(sample), flush=True)
        pair = [r for r in records if r["workload"] == workload and r["variant"] in ("baseline", candidate)]
        outputs = {(r["exit"], r["stdout_sha256"], r["stderr_sha256"]) for r in pair}
        if len(outputs) != 1:
            raise SystemExit(f"Output parity failure: {workload}/{candidate}: {outputs}")

(OUTPUT / "samples.json").write_text(json.dumps(records, indent=2))
(OUTPUT / "source-commit.txt").write_text(subprocess.check_output(["git", "rev-parse", "HEAD"], text=True))
(OUTPUT / "cpu.txt").write_text(subprocess.check_output(["lscpu"], text=True))
(OUTPUT / "image.txt").write_text(json.dumps({key: os.environ.get(key) for key in ("ImageOS", "ImageVersion", "RUNNER_ARCH")}, indent=2))
