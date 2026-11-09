#!/usr/bin/env python3
"""Measure the frozen call-site assembly fixture with exact report parity."""

from __future__ import annotations

import argparse
import fcntl
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time
from typing import Any

LOCK_PATH = Path("/tmp/nm-heavy.lock")
TIME_FORMAT = "%e %U %S %M"
MODES = ("on", "off", "imports")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--baseline", required=True, help="baseline no-mistakes binary")
    parser.add_argument("--candidate", help="candidate no-mistakes binary")
    parser.add_argument("--baseline-revision", required=True)
    parser.add_argument("--candidate-revision")
    parser.add_argument(
        "--fixture",
        type=Path,
        default=Path(__file__).resolve().parent,
        help="fixture root (defaults to this script's directory)",
    )
    parser.add_argument("--out", required=True, type=Path, help="output directory")
    parser.add_argument("--rounds", type=int, default=1, help="balanced blocks per mode/jobs")
    parser.add_argument("--timings", action="store_true", help="measure diagnostics separately")
    parser.add_argument(
        "--lock-held",
        action="store_true",
        help="skip acquiring the lock because the caller already holds it",
    )
    args = parser.parse_args()
    if args.candidate and not args.candidate_revision:
        parser.error("--candidate-revision is required with --candidate")
    if args.rounds < 1:
        parser.error("--rounds must be at least 1")
    return args


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def clean_environment(output_dir: Path) -> dict[str, str]:
    environment = os.environ.copy()
    for key in ("RAYON_NUM_THREADS", "MALLOC_ARENA_MAX", "LD_PRELOAD", "CARGO_TARGET_DIR"):
        environment.pop(key, None)

    runtime_dir = output_dir / "runtime"
    runtime_dir.mkdir(exist_ok=True, mode=0o700)
    environment["XDG_RUNTIME_DIR"] = str(runtime_dir.resolve())
    return environment


def job_counts() -> list[int]:
    available_cpus = len(os.sched_getaffinity(0))
    return [1, 4, available_cpus]


def execution_order(
    binary_names: list[str], round_index: int
) -> list[str]:
    if round_index == -1:
        return binary_names.copy()
    if len(binary_names) == 1:
        return binary_names.copy()
    if round_index % 2 == 0:
        return ["baseline", "candidate", "candidate", "baseline"]
    return ["candidate", "baseline", "baseline", "candidate"]


def expected_result(mode: str) -> tuple[int, int]:
    if mode == "on":
        return 1, 1
    return 0, 0


def make_command(
    binary: str, fixture: Path, mode: str, jobs: int, output_dir: Path, label: str, diagnostics: bool
) -> tuple[list[str], Path, Path, Path]:
    report_path = output_dir / f"{label}.json"
    timing_path = output_dir / f"{label}.time"
    stderr_path = output_dir / f"{label}.stderr"
    cores = sorted(os.sched_getaffinity(0))[:jobs]
    command = [
        "/usr/bin/taskset", "--cpu-list", ",".join(map(str, cores)),
        "/usr/bin/time",
        "-f",
        TIME_FORMAT,
        "-o",
        str(timing_path),
        binary,
        "check",
        "--root",
        str(fixture),
        "--config",
        str(fixture / f"{mode}.yml"),
        "--format",
        "json",
        "--timeout",
        "0",
        "--lock-timeout",
        "0",
        "--fail-on-lock",
        "--jobs",
        str(jobs),
    ]
    if diagnostics:
        command.extend(["--timings", "--verbose-timings"])
    return command, report_path, timing_path, stderr_path


def run_sample(
    *,
    binary_name: str,
    binary: str,
    fixture: Path,
    output_dir: Path,
    mode: str,
    jobs: int,
    round_index: int,
    order_index: int,
    environment: dict[str, str],
    diagnostics: bool,
) -> dict[str, Any]:
    warmup = round_index == -1
    label = f"{mode}-j{jobs}-r{round_index}-{order_index}-{binary_name}"
    command, report_path, timing_path, stderr_path = make_command(
        binary, fixture, mode, jobs, output_dir, label, diagnostics
    )

    with report_path.open("wb") as stdout_file, stderr_path.open("wb") as stderr_file:
        completed = subprocess.run(
            command,
            stdout=stdout_file,
            stderr=stderr_file,
            env=environment,
            check=False,
        )

    timing_fields = timing_path.read_text(encoding="utf-8").splitlines()[-1].split()
    wall_seconds, user_seconds, system_seconds, peak_rss_kib = map(float, timing_fields)
    report_bytes = report_path.read_bytes()
    report = json.loads(report_bytes)
    finding_count = len(report.get("rules", []))
    expected_exit, expected_findings = expected_result(mode)
    if completed.returncode != expected_exit:
        raise RuntimeError(
            f"{label}: expected exit {expected_exit}, got {completed.returncode}; "
            f"stderr: {stderr_path.read_text(encoding='utf-8')}"
        )
    if finding_count != expected_findings:
        raise RuntimeError(
            f"{label}: expected {expected_findings} rule findings, got {finding_count}"
        )

    return {
        "warmup": warmup,
        "label": label,
        "arm": binary_name,
        "mode": mode,
        "jobs": jobs,
        "command": command,
        "wallSeconds": wall_seconds,
        "userSeconds": user_seconds,
        "systemSeconds": system_seconds,
        "peakRssKiB": int(peak_rss_kib),
        "exitCode": completed.returncode,
        "ruleFindings": finding_count,
        "reportSha256": hashlib.sha256(report_bytes).hexdigest(),
    }


def main() -> None:
    args = parse_args()
    fixture = args.fixture.resolve()
    output_dir = args.out.resolve()
    output_dir.mkdir(parents=True, exist_ok=True)

    binaries = {"baseline": str(Path(args.baseline).resolve())}
    if args.candidate:
        binaries["candidate"] = str(Path(args.candidate).resolve())
    binary_hashes = {
        name: sha256_file(Path(binary)) for name, binary in binaries.items()
    }
    if len(set(binary_hashes.values())) != len(binary_hashes):
        raise ValueError("baseline and candidate binary hashes must be distinct")

    environment = clean_environment(output_dir)
    samples: list[dict[str, Any]] = []
    lock_file = LOCK_PATH.open("w", encoding="utf-8")
    try:
        if not args.lock_held:
            fcntl.flock(lock_file, fcntl.LOCK_EX)

        for mode in MODES:
            for jobs in job_counts():
                for round_index in range(-1, args.rounds):
                    order = execution_order(list(binaries), round_index)
                    for order_index, binary_name in enumerate(order):
                        sample = run_sample(
                            binary_name=binary_name,
                            binary=binaries[binary_name],
                            fixture=fixture,
                            output_dir=output_dir,
                            mode=mode,
                            jobs=jobs,
                            round_index=round_index,
                            order_index=order_index,
                            environment=environment,
                            diagnostics=args.timings,
                        )
                        samples.append(sample)
                        print(json.dumps(sample, sort_keys=True), flush=True)
                        write_results(
                            output_dir,
                            args,
                            fixture,
                            binaries,
                            binary_hashes,
                            samples,
                        )
    finally:
        lock_file.close()

    for mode in MODES:
        report_hashes = {
            sample["reportSha256"] for sample in samples if sample["mode"] == mode
        }
        if len(report_hashes) != 1:
            raise AssertionError(f"report hash drift in mode {mode}: {report_hashes}")

    write_results(output_dir, args, fixture, binaries, binary_hashes, samples)


def write_results(
    output_dir: Path,
    args: argparse.Namespace,
    fixture: Path,
    binaries: dict[str, str],
    binary_hashes: dict[str, str],
    samples: list[dict[str, Any]],
) -> None:
    provenance = {
        "fixture": str(fixture),
        "fixtureFiles": sum(path.is_file() for path in fixture.rglob("*")),
        "fixtureBytes": sum(path.stat().st_size for path in fixture.rglob("*") if path.is_file()),
        "baselineAndCandidate": binaries,
        "binarySha256": binary_hashes,
        "baselineRevision": args.baseline_revision,
        "candidateRevision": args.candidate_revision,
        "rounds": args.rounds,
        "jobs": job_counts(),
        "modes": list(MODES),
        "timeFormat": TIME_FORMAT,
        "lockPath": str(LOCK_PATH),
        "lockHeldByCaller": args.lock_held,
        "observerAndTimings": "timings and verbose timings" if args.timings else "disabled",
        "availableAffinity": sorted(os.sched_getaffinity(0)),
        "sampleAffinity": "first jobs CPUs from the available affinity",
        "capturedAtUnixSeconds": time.time(),
    }
    document = {"provenance": provenance, "samples": samples}
    temporary_path = output_dir / "samples.json.tmp"
    temporary_path.write_text(json.dumps(document, indent=2, sort_keys=True) + "\n")
    temporary_path.replace(output_dir / "samples.json")


if __name__ == "__main__":
    main()
