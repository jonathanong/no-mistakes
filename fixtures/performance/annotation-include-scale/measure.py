"""Compare release CLIs with balanced, fresh observer-off invocations."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import statistics
import subprocess
import time

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("baseline", type=Path)
parser.add_argument("candidate", type=Path)
parser.add_argument("output", type=Path)
parser.add_argument("--rounds", type=int, default=3)
parser.add_argument("--jobs", type=int, default=8)
args = parser.parse_args()
root = Path(__file__).resolve().parent
args.output.mkdir(parents=True, exist_ok=True)
environment = dict(os.environ, XDG_RUNTIME_DIR=str((args.output / "invocation").resolve()))
environment.pop("RAYON_NUM_THREADS", None)
runtime_dir = Path(environment["XDG_RUNTIME_DIR"])
runtime_dir.mkdir(mode=0o700, exist_ok=True)
runtime_dir.chmod(0o700)
binaries = {name: binary.resolve() for name, binary in
            [("baseline", args.baseline), ("candidate", args.candidate)]}
manifest = {name: {"path": str(binary), "sha256": hashlib.sha256(binary.read_bytes()).hexdigest()}
            for name, binary in binaries.items()}
assert manifest["baseline"]["sha256"] != manifest["candidate"]["sha256"], "identical binaries"
samples = []
reports = {}


def run(name, case, label, timings=False):
    prefix = args.output / f"{case}-{name}-{label}"
    command = [str(binaries[name]), "check", "--root", str(root), "--config",
               str(root / "configs" / f"{case}.yml"),
               "--format", "json", "--timeout", "0", "--lock-timeout", "0"]
    if args.jobs:
        command.extend(["--jobs", str(args.jobs)])
    if timings:
        command.append("--timings")
    measured = ["/usr/bin/time", "-f", "%e %U %S %M", "-o", str(prefix) + ".resource"] + command
    started = time.perf_counter()
    with open(str(prefix) + ".json", "wb") as stdout, open(str(prefix) + ".stderr", "wb") as stderr:
        result = subprocess.run(measured, env=environment, stdout=stdout, stderr=stderr, check=False)
    wall = time.perf_counter() - started
    resource = Path(str(prefix) + ".resource").read_text().splitlines()[-1].split()
    _, user, system, rss = map(float, resource)
    stdout = Path(str(prefix) + ".json").read_bytes()
    stderr = Path(str(prefix) + ".stderr").read_bytes()
    if result.returncode not in (0, 1):
        raise RuntimeError(f"{name}/{case}: exit {result.returncode}: {stderr.decode()}")
    report = json.loads(stdout)
    stable = (result.returncode, stdout, None if timings else stderr)
    key = (case, timings)
    if key in reports:
        assert reports[key] == stable, f"changed output for {name}/{case}/{label}"
    reports[key] = stable
    match = re.search(rb"\[timing\] parse: ([0-9.]+)ms", stderr)
    assert not timings or match is not None, f"missing parse timing for {name}/{case}"
    sample = {"binary": name, "case": case, "label": label, "observer": timings,
              "wall": wall, "cpu": user + system, "rss_kib": rss,
              "parse_ms": float(match[1]) if match else None,
              "exit": result.returncode, "stdout_sha256": hashlib.sha256(stdout).hexdigest(),
              "findings": len(report.get("rules", [])), "command": command}
    samples.append(sample)
    print(json.dumps(sample), flush=True)


for case in ["small-max", "full-max", "small-low", "full-low"]:
    for name in binaries:
        run(name, case, "warmup")
    for cycle in range(args.rounds):
        for position, name in enumerate(["baseline", "candidate", "candidate", "baseline"]):
            run(name, case, f"{cycle}-{position}")
    if case.endswith("max"):
        for cycle in range(args.rounds):
            for position, name in enumerate(["baseline", "candidate", "candidate", "baseline"]):
                run(name, case, f"parse-{cycle}-{position}", timings=True)

summary = []
for case in ["small-max", "full-max", "small-low", "full-low"]:
    for name in binaries:
        selected = [sample for sample in samples if sample["binary"] == name
                    and sample["case"] == case and sample["label"] != "warmup"]
        acceptance = [sample for sample in selected if not sample["observer"]]
        attribution = [sample["parse_ms"] for sample in selected if sample["observer"]]
        summary.append({"binary": name, "case": case, "samples": len(acceptance),
                        **{metric: statistics.median(sample[metric] for sample in acceptance)
                           for metric in ["wall", "cpu", "rss_kib"]},
                        "instrumented_parse_ms": statistics.median(attribution) if attribution else None})
(args.output / "results.json").write_text(json.dumps({"binaries": manifest,
    "jobs": args.jobs, "samples": samples, "summary": summary}, indent=2) + "\n")
print(json.dumps(summary, indent=2))
