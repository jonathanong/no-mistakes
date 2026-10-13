"""Temporary PGO training and same-runner validation."""

import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys


root = Path.cwd()
result = Path(os.environ["PGO_RESULT_DIR"])
result.mkdir(parents=True, exist_ok=True)
resolve_root = root / "fixtures/performance/resolve-check"
resolve_files = sorted(resolve_root.glob("web/app/routes/closure-*/page.tsx"))
if len(resolve_files) != 2120:
    raise SystemExit(f"Expected 2120 resolve fixtures, found {len(resolve_files)}")
checks = {
    "self": ["check", "--root", str(root), "--format", "json"],
    "graph": ["check", "--root", str(root / "fixtures/performance/graph-gates"), "--format", "json"],
    "resolve": ["resolve-check", *(str(path) for path in resolve_files),
                "--root", str(resolve_root), "--format", "json"],
}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def execute(name, variant, command, environment=None):
    prefix = result / f"{name}-{variant}"
    run = subprocess.run(
        ["/usr/bin/time", "-o", str(prefix) + ".time",
         "-f", "elapsed=%e\nuser=%U\nsystem=%S\nrss_kib=%M\nexit=%x",
         "timeout", "--signal=TERM", "--kill-after=5s", "180s", *command],
        cwd=root, capture_output=True, check=False, env=environment,
    )
    (Path(str(prefix) + ".stdout")).write_bytes(run.stdout)
    (Path(str(prefix) + ".stderr")).write_bytes(run.stderr)
    fields = dict(line.split("=", 1) for line in Path(str(prefix) + ".time").read_text().splitlines() if "=" in line)
    return {
        "name": name, "variant": variant, "exit": run.returncode,
        "elapsed": float(fields["elapsed"]), "user": float(fields["user"]),
        "system": float(fields["system"]), "rssKiB": int(fields["rss_kib"]),
        "stdoutSha256": digest(run.stdout), "stderrSha256": digest(run.stderr),
    }


if sys.argv[1] == "train":
    profile_dir = Path(os.environ["PGO_PROFILE_DIR"])
    binary = root / "target/aarch64-unknown-linux-gnu/release/no-mistakes"
    records = []
    for name, args in checks.items():
        environment = dict(os.environ, LLVM_PROFILE_FILE=str(profile_dir / f"{name}-%p-%m.profraw"))
        record = execute(name, "train", [str(binary), *args], environment)
        records.append(record)
        print(json.dumps(record), flush=True)
        if record["exit"] in (124, 137):
            raise SystemExit(f"Training timed out: {name}")
    environment = dict(os.environ, LLVM_PROFILE_FILE=str(profile_dir / "node-%p-%m.profraw"))
    record = execute(
        "node", "train",
        ["node", "-e",
         "const a=require(process.argv[1]);a.checkJson(Buffer.from(JSON.stringify({root:process.argv[2]}))).then(x=>process.stdout.write(x)).catch(e=>{console.error(e);process.exitCode=1})",
         str(result / "instrumented.node"), str(root / "fixtures/performance/graph-gates")],
        environment,
    )
    records.append(record)
    print(json.dumps(record), flush=True)
    if record["exit"] != 0:
        raise SystemExit("Instrumented Node API training failed")
    (result / "training-samples.json").write_text(json.dumps(records, indent=2))
elif sys.argv[1] == "measure":
    workloads = dict(checks)
    workloads["node"] = []
    records = []
    for name, args in workloads.items():
        pair = []
        for trial, variant in enumerate(("baseline", "pgo", "pgo", "baseline"), 1):
            if name == "node":
                command = [
                    "node", "-e",
                    "const a=require(process.argv[1]);a.checkJson(Buffer.from(JSON.stringify({root:process.argv[2]}))).then(x=>process.stdout.write(x)).catch(e=>{console.error(e);process.exitCode=1})",
                    str(result / f"{variant}.node"), str(root / "fixtures/performance/graph-gates"),
                ]
            else:
                command = [str(result / variant), *args]
            record = execute(f"{name}-{trial}", variant, command)
            pair.append(record)
            records.append(record)
            print(json.dumps(record), flush=True)
        parity = {(row["exit"], row["stdoutSha256"], row["stderrSha256"]) for row in pair}
        if len(parity) != 1 or any(row["exit"] in (124, 137) for row in pair):
            raise SystemExit(f"Output parity or deadline failure: {name}: {parity}")
    (result / "measurements.json").write_text(json.dumps(records, indent=2))
else:
    raise SystemExit("Usage: pgo-retry-experiment.py train|measure")
