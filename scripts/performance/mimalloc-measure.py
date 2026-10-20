#!/usr/bin/env python3
"""Run balanced allocator comparisons and preserve per-child wait4 evidence."""
import argparse
import json
import os
import platform
import signal
import subprocess
import sys
import time
from pathlib import Path


def mimalloc_environment(environment=None):
    environment = os.environ if environment is None else environment
    return {key: value for key, value in environment.items() if key.startswith("MIMALLOC_")}


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


def measure_child(argv, cwd, output, sample, metadata, timeout, env=None):
    output = Path(output)
    output.mkdir(parents=True, exist_ok=True)
    stdout_path, stderr_path = output / f"{sample}.stdout", output / f"{sample}.stderr"
    started = time.perf_counter()
    with stdout_path.open("wb") as stdout, stderr_path.open("wb") as stderr:
        child = subprocess.Popen(argv, cwd=cwd, stdout=stdout, stderr=stderr, start_new_session=True, env=env)
        usage, timed_out = wait_child(child, timeout)
    executable = Path(argv[0]).resolve()
    available = len(os.sched_getaffinity(0)) if hasattr(os, "sched_getaffinity") else os.cpu_count()
    row = {
        "sample": sample, "metadata": metadata, "host_id": metadata["host_id"], "argv": argv,
        "cwd": str(Path(cwd).resolve()),
        "exit": child.returncode, "signal": -child.returncode if child.returncode < 0 else None,
        "timed_out": timed_out, "wall_seconds": time.perf_counter() - started,
        "user_seconds": usage.ru_utime, "system_seconds": usage.ru_stime,
        "peak_rss_bytes": rss_bytes(usage.ru_maxrss), "ru_maxrss_raw": usage.ru_maxrss,
        "ru_maxrss_units": "bytes" if sys.platform == "darwin" else "KiB",
        "stdout_sha256": ARTIFACT["digest"](stdout_path),
        "stderr_sha256": ARTIFACT["digest"](stderr_path),
        "platform": platform.platform(), "machine": platform.machine(), "cpu_model": cpu_model(),
        "logical_cpus": os.cpu_count(), "available_cpus": available,
        "rayon_environment": os.environ.get("RAYON_NUM_THREADS"),
        "mimalloc_environment": mimalloc_environment(env),
        "executable": str(executable), "executable_sha256": ARTIFACT["digest"](executable),
    }
    (output / f"{sample}.json").write_text(json.dumps(row, indent=2) + "\n")


def measure(argv, cwd, output, sample, metadata, timeout, env=None):
    # A fresh interpreter prevents the controller's earlier buffers from becoming
    # the native child's pre-exec RSS floor on the next fork.
    payload = dict(argv=argv, cwd=str(cwd), output=str(output), sample=sample,
                   metadata=metadata, timeout=timeout, env=env)
    worker = Path(__file__).with_name("mimalloc-sample-worker.py")
    subprocess.run([sys.executable, str(worker)], input=json.dumps(payload), text=True, check=True)


def command(binary, workload, root, jobs, diagnostics):
    args = [binary, "--timeout", "0", "--lock-timeout", "0"]
    args += ["check", "--root", root, "--format", "json"]
    if jobs != "max":
        args += ["--jobs", jobs]
    if diagnostics:
        args += ["--progress", "--verbose-timings"]
    return args


ARTIFACT = __import__("runpy").run_path(str(Path(__file__).with_name("mimalloc-artifact.py")))
verify_artifact = ARTIFACT["verify_artifact"]
verify_pair = ARTIFACT["verify_pair"]
verify_probe = ARTIFACT["verify_probe"]


def main():
    parser = argparse.ArgumentParser()
    for name in ("system", "mimalloc", "consumer", "hosted-acceptance", "hosted-passing-failing", "output"):
        parser.add_argument(f"--{name}", required=True)
    parser.add_argument("--measurement-id", help="unique collection ID; defaults to host-id")
    parser.add_argument("--timeout", type=int, default=600)
    parser.add_argument("--workload", choices=("consumer", "hosted-acceptance", "hosted-passing-failing"), required=True)
    parser.add_argument("--jobs", choices=("1", "4", "16", "max"), required=True)
    parser.add_argument("--diagnostics", choices=("on", "off"), required=True)
    parser.add_argument("--pair", type=int, choices=range(4), required=True)
    parser.add_argument("--host-id", required=True)
    parser.add_argument("--platform", choices=("linux-arm64", "linux-x64", "macos-arm64"), required=True)
    parser.add_argument("--source", required=True)
    parser.add_argument("--system-probe", required=True)
    parser.add_argument("--mimalloc-probe", required=True)
    parser.add_argument("--workload-commit", required=True)
    parser.add_argument("--workload-tree", required=True)
    args = parser.parse_args()
    workload = args.workload
    root = getattr(args, workload.replace("-", "_"))
    artifacts = {variant: verify_artifact(getattr(args, variant), variant, args.source, args.platform)
                 for variant in ("system", "mimalloc")}
    verify_pair(artifacts)
    probe_jobs = {}
    for variant in ("system", "mimalloc"):
        probe = verify_probe(getattr(args, f"{variant}_probe"), artifacts[variant])
        result = json.loads(subprocess.check_output([probe], text=True))
        expected = variant == "mimalloc"
        if result.get("expected_mimalloc") is not expected or result.get("is_mimalloc_heap") is not expected:
            raise ValueError(f"{variant} allocator probe disagrees with the expected registration")
        probe_jobs[variant] = result["default_rayon_jobs"]
    if probe_jobs["system"] != probe_jobs["mimalloc"]:
        raise ValueError("system and mimalloc variants report different automatic Rayon thread counts")
    raw_rayon_override = os.environ.get("RAYON_NUM_THREADS")
    try:
        rayon_override = int(raw_rayon_override) if raw_rayon_override is not None else 0
    except ValueError:
        rayon_override = 0
    automatic_jobs = rayon_override if rayon_override > 0 else probe_jobs["system"]
    diagnostics = args.diagnostics == "on"
    # Adjacent ABBA then BAAB is encoded as four matched pairs: AB, BA, BA, AB.
    order = ("system", "mimalloc") if args.pair in (0, 3) else ("mimalloc", "system")
    cell = f"{workload}-diag{int(diagnostics)}-jobs{args.jobs}"
    for position, variant in enumerate(order):
        metadata = {"variant": variant, "workload": workload, "platform": args.platform,
                    "diagnostics": diagnostics,
                    "jobs": args.jobs, "pair": args.pair, "order": "AB" if order[0] == "system" else "BA",
                    "position": position, "host_id": args.host_id, "expected_exit": 0,
                    "measurement_id": args.measurement_id or args.host_id, "tree": artifacts[variant]["tree"],
                    "source": args.source, "lock": artifacts[variant]["lock"],
                    "rustc": artifacts[variant]["rustc"], "cargo": artifacts[variant]["cargo"],
                    "target": artifacts[variant]["target"], "build_flags": artifacts[variant]["build_flags"],
                    "artifact_sha256": artifacts[variant]["no-mistakes"]["sha256"],
                    "workload_commit": args.workload_commit, "workload_tree": args.workload_tree,
                    "lto": artifacts[variant]["lto"], "codegenUnits": artifacts[variant]["codegenUnits"],
                    "rayon_jobs": int(args.jobs) if args.jobs != "max" else automatic_jobs,
                    "automatic_rayon_jobs": automatic_jobs, "num_cpus_default": probe_jobs[variant],
                    "rayon_environment": raw_rayon_override,
                    "mimalloc_environment": mimalloc_environment(),
                    "timeout_seconds": args.timeout}
        sample = f"{args.platform}-{cell}-pair{args.pair}-{variant}"
        measure(command(getattr(args, variant), workload, root, args.jobs, diagnostics), root,
                args.output, sample, metadata, args.timeout)
        print(f"measured {sample}", flush=True)


if __name__ == "__main__":
    main()
