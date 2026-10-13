"""Measure generic and runner-native binaries on the build host itself."""

import hashlib
import json
import os
from pathlib import Path
import subprocess


ROOT = Path.cwd()
ARTIFACTS = Path(os.environ["RUNNER_TEMP"]) / "native-cpu"
OUTPUT = ARTIFACTS / "measurements"
OUTPUT.mkdir(parents=True, exist_ok=True)
resolve_root = ROOT / "fixtures/performance/resolve-check"
resolve_files = sorted(resolve_root.glob("web/app/routes/closure-*/page.tsx"))
if len(resolve_files) != 2120:
    raise SystemExit(f"Expected 2120 held-out resolve fixtures, found {len(resolve_files)}")
WORKLOADS = {
    "self-check": lambda variant: [str(ARTIFACTS / variant), "check", "--root", str(ROOT), "--format", "json"],
    "graph-gates": lambda variant: [str(ARTIFACTS / variant), "check", "--root", str(ROOT / "fixtures/performance/graph-gates"), "--format", "json"],
    "held-out-resolve": lambda variant: [str(ARTIFACTS / variant), "resolve-check", *(str(path) for path in resolve_files), "--root", str(resolve_root), "--format", "json"],
    "node-check": lambda variant: [
        "node", "-e",
        "const a=require(process.argv[1]);a.checkJson(Buffer.from(JSON.stringify({root:process.argv[2]}))).then(x=>process.stdout.write(x)).catch(e=>{console.error(e);process.exitCode=1})",
        str(ARTIFACTS / f"{variant}.node"), str(ROOT / "fixtures/performance/graph-gates"),
    ],
}


def digest(data):
    return hashlib.sha256(data).hexdigest()


records = []
for workload, command in WORKLOADS.items():
    pair = []
    for trial, variant in enumerate(("generic", "native", "native", "generic"), 1):
        prefix = OUTPUT / f"{workload}-{variant}-{trial}"
        result = subprocess.run(
            ["/usr/bin/time", "-f", "elapsed=%e\nuser=%U\nsystem=%S\nrss_kib=%M\nexit=%x",
             "-o", str(prefix) + ".time", "timeout", "--kill-after=5s", "240s", *command(variant)],
            cwd=ROOT, capture_output=True, check=False,
        )
        (Path(str(prefix) + ".stdout")).write_bytes(result.stdout)
        (Path(str(prefix) + ".stderr")).write_bytes(result.stderr)
        metrics = dict(line.split("=", 1) for line in Path(str(prefix) + ".time").read_text().splitlines() if "=" in line)
        record = {
            "variant": variant, "workload": workload, "trial": trial,
            "command": command(variant), "exit": result.returncode,
            "stdout_sha256": digest(result.stdout), "stderr_sha256": digest(result.stderr),
            "binary_sha256": digest((ARTIFACTS / variant).read_bytes()),
            **{key: float(metrics[key]) for key in ("elapsed", "user", "system", "rss_kib")},
        }
        records.append(record)
        pair.append(record)
        print(json.dumps(record), flush=True)
    parity = {(r["exit"], r["stdout_sha256"], r["stderr_sha256"]) for r in pair}
    if len(parity) != 1 or any(r["exit"] in (124, 137) for r in pair):
        raise SystemExit(f"Invalid output parity or timed out: {workload}: {parity}")

(OUTPUT / "samples.json").write_text(json.dumps(records, indent=2))
