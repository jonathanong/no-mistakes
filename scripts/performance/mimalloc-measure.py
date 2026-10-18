#!/usr/bin/env python3
"""Run balanced allocator comparisons and preserve per-child wait4 evidence."""
import argparse
import hashlib
import json
import os
import platform
import signal
import subprocess
import sys
import time
from pathlib import Path


def rss_bytes(raw):
    if sys.platform == "darwin":
        return raw
    if sys.platform.startswith("linux"):
        return raw * 1024
    raise RuntimeError(f"unsupported ru_maxrss units on {sys.platform}")


def cpu_model():
    if sys.platform == "darwin":
        return subprocess.check_output(["sysctl", "-n", "machdep.cpu.brand_string"], text=True).strip()
    try:
        lines = Path("/proc/cpuinfo").read_text().splitlines()
        for key in ("model name\t:", "Hardware\t:", "Model\t:"):
            for line in lines:
                if line.startswith(key):
                    return line.split(":", 1)[1].strip()
    except OSError:
        pass
    return platform.processor() or f"unknown ({platform.machine()}; runner model unavailable)"


def wait_child(child, timeout, grace=5):
    deadline = time.monotonic() + timeout
    timed_out = False
    term_at = None
    while True:
        pid, status, usage = os.wait4(child.pid, os.WNOHANG)
        if pid:
            child.returncode = os.waitstatus_to_exitcode(status)
            return usage, timed_out
        now = time.monotonic()
        if not timed_out and now >= deadline:
            timed_out = True
            term_at = now
            try:
                os.killpg(child.pid, signal.SIGTERM)
            except ProcessLookupError:
                pass
        elif timed_out and now - term_at >= grace:
            try:
                os.killpg(child.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
        time.sleep(0.05)


def measure(argv, cwd, output, sample, metadata, timeout):
    output = Path(output)
    output.mkdir(parents=True, exist_ok=True)
    stdout_path, stderr_path = output / f"{sample}.stdout", output / f"{sample}.stderr"
    started = time.perf_counter()
    with stdout_path.open("wb") as stdout, stderr_path.open("wb") as stderr:
        child = subprocess.Popen(argv, cwd=cwd, stdout=stdout, stderr=stderr, start_new_session=True)
        usage, timed_out = wait_child(child, timeout)
    executable = Path(argv[0]).resolve()
    available = len(os.sched_getaffinity(0)) if hasattr(os, "sched_getaffinity") else os.cpu_count()
    row = {
        "sample": sample, "metadata": metadata, "argv": argv, "cwd": str(Path(cwd).resolve()),
        "exit": child.returncode, "signal": -child.returncode if child.returncode < 0 else None,
        "timed_out": timed_out, "wall_seconds": time.perf_counter() - started,
        "user_seconds": usage.ru_utime, "system_seconds": usage.ru_stime,
        "peak_rss_bytes": rss_bytes(usage.ru_maxrss), "ru_maxrss_raw": usage.ru_maxrss,
        "ru_maxrss_units": "bytes" if sys.platform == "darwin" else "KiB",
        "stdout_sha256": hashlib.sha256(stdout_path.read_bytes()).hexdigest(),
        "stderr_sha256": hashlib.sha256(stderr_path.read_bytes()).hexdigest(),
        "platform": platform.platform(), "machine": platform.machine(), "cpu_model": cpu_model(),
        "logical_cpus": os.cpu_count(), "available_cpus": available,
        "executable": str(executable), "executable_sha256": hashlib.sha256(executable.read_bytes()).hexdigest(),
    }
    (output / f"{sample}.json").write_text(json.dumps(row, indent=2) + "\n")


def command(binary, workload, root, jobs, diagnostics):
    args = [binary]
    if workload.startswith("hosted-"):
        args += ["--timeout", "0", "--lock-timeout", "0"]
    args += ["check", "--root", root, "--format", "json"]
    if jobs != "auto":
        args += ["--jobs", jobs]
    if diagnostics:
        args += ["--progress", "--verbose-timings"]
    return args


def main():
    parser = argparse.ArgumentParser()
    for name in ("system", "mimalloc", "consumer", "hosted-acceptance", "hosted-passing-failing", "output"):
        parser.add_argument(f"--{name}", required=True)
    parser.add_argument("--timeout", type=int, default=600)
    args = parser.parse_args()
    workloads = [("consumer", args.consumer), ("hosted-acceptance", args.hosted_acceptance),
                 ("hosted-passing-failing", args.hosted_passing_failing)]
    # Adjacent ABBA then BAAB supplies four samples per variant with reversed order.
    order = [("system", 0), ("mimalloc", 0), ("mimalloc", 1), ("system", 1),
             ("mimalloc", 2), ("system", 2), ("system", 3), ("mimalloc", 3)]
    for workload, root in workloads:
        for diagnostics in (False, True):
            for jobs in ("1", "4", "auto"):
                cell = f"{workload}-diag{int(diagnostics)}-jobs{jobs}"
                for variant, repetition in order:
                    metadata = {"variant": variant, "workload": workload, "diagnostics": diagnostics,
                                "jobs": jobs, "repetition": repetition, "timeout_seconds": args.timeout}
                    sample = f"{cell}-{variant}-{repetition}"
                    measure(command(getattr(args, variant), workload, root, jobs, diagnostics), root,
                            args.output, sample, metadata, args.timeout)
                    print(f"measured {sample}", flush=True)


if __name__ == "__main__":
    main()
