#!/usr/bin/env python3
"""Measure fresh Node processes using the same wait4 sampler as the CLI cells."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import runpy
import shutil
import subprocess

SHARED = runpy.run_path(str(Path(__file__).with_name("mimalloc-measure.py")))
CONSUMER_COMMIT = "f38a048f5eeff0438fc59a15e9eb742def575a36"


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def verify_addon(directory, provenance):
    addon = Path(directory).resolve() / "no-mistakes.node"
    if provenance.get("no-mistakes.node", {}).get("sha256") != digest(addon):
        raise ValueError("native addon hash does not match build provenance")
    return addon


def verify_workload(prepared, platform_name):
    prepared = Path(prepared)
    record = json.loads((prepared / "workload-provenance.json").read_text())
    expected = {"platform": platform_name, "workload": "consumer", "commit": CONSUMER_COMMIT}
    if any(record.get(key) != value for key, value in expected.items()):
        raise ValueError("prepared N-API consumer provenance mismatch")
    if record.get("archive_sha256") != digest(prepared / "workload.tar.gz"):
        raise ValueError("prepared N-API consumer archive hash mismatch")
    if not isinstance(record.get("tree"), str) or len(record["tree"]) != 40:
        raise ValueError("prepared consumer tree identity missing")
    return record


def main():
    parser = argparse.ArgumentParser()
    for name in ("system", "mimalloc", "consumer", "prepared", "output", "host-id", "source"):
        parser.add_argument(f"--{name}", required=True)
    parser.add_argument("--jobs", choices=("1", "max"), required=True)
    parser.add_argument("--pair", type=int, choices=range(4), required=True)
    parser.add_argument("--platform", choices=("linux-arm64", "macos-arm64"), required=True)
    args = parser.parse_args()
    if SHARED["mimalloc_environment"]():
        raise ValueError("MIMALLOC_ environment overrides are forbidden")
    workload = verify_workload(args.prepared, args.platform)
    artifacts = {variant: SHARED["verify_artifact"](
        str(Path(getattr(args, variant)) / "no-mistakes"), variant, args.source, args.platform)
        for variant in ("system", "mimalloc")}
    SHARED["verify_pair"](artifacts)
    addons = {variant: verify_addon(getattr(args, variant), artifacts[variant]) for variant in artifacts}
    if digest(addons["system"]) == digest(addons["mimalloc"]):
        raise ValueError("system and mimalloc native addons must be distinct")
    probes = {}
    for variant in artifacts:
        probe = Path(getattr(args, variant)).resolve() / "rayon-probe"
        record = json.loads(subprocess.check_output([str(probe)], text=True))
        expected = variant == "mimalloc"
        if record.get("expected_mimalloc") is not expected or record.get("is_mimalloc_heap") is not expected:
            raise ValueError("allocator probe registration mismatch")
        probes[variant] = record["default_rayon_jobs"]
    if probes["system"] != probes["mimalloc"]:
        raise ValueError("allocator variants disagree on automatic Rayon jobs")
    automatic_jobs = probes["system"]
    raw_override = os.environ.get("RAYON_NUM_THREADS")
    try:
        automatic_jobs = max(0, int(raw_override or "0")) or automatic_jobs
    except ValueError:
        pass
    node = Path(shutil.which("node")).resolve()
    node_version = subprocess.check_output([str(node), "--version"], text=True).strip()
    runner = str(Path(__file__).with_name("mimalloc-napi-runner.js").resolve())
    order = ("system", "mimalloc") if args.pair in (0, 3) else ("mimalloc", "system")
    for position, variant in enumerate(order):
        build = artifacts[variant]
        metadata = {key: build[key] for key in ("source", "tree", "lock", "lto", "codegenUnits", "rustc", "cargo", "target", "build_flags")}
        metadata.update({"variant": variant, "workload": "consumer-napi", "platform": args.platform,
                         "diagnostics": False, "jobs": args.jobs, "pair": args.pair,
                         "order": "AB" if order[0] == "system" else "BA", "position": position,
                         "host_id": args.host_id,
                         "measurement_id": f"{os.environ.get('GITHUB_RUN_ID', args.host_id)}-{os.environ.get('GITHUB_RUN_ATTEMPT', '1')}",
                         "expected_exit": 0, "timeout_seconds": 600,
                         "artifact_sha256": digest(addons[variant]), "node_version": node_version,
                         "node_sha256": digest(node), "workload_commit": workload["commit"],
                         "workload_tree": workload["tree"], "num_cpus_default": probes[variant],
                         "automatic_rayon_jobs": automatic_jobs,
                         "rayon_jobs": 1 if args.jobs == "1" else automatic_jobs,
                         "rayon_environment": raw_override, "mimalloc_environment": {}})
        env = {**os.environ, "NO_MISTAKES_TEST_NAPI_ADDON_PATH": str(addons[variant])}
        sample = f"{args.platform}-consumer-napi-diag0-jobs{args.jobs}-pair{args.pair}-{variant}"
        SHARED["measure"]([str(node), runner, str(Path(args.consumer).resolve()), args.jobs],
                          args.consumer, args.output, sample, metadata, 600, env=env)
        print(f"measured {sample}", flush=True)


if __name__ == "__main__":
    main()
