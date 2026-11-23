"""Measure saved check fixtures with an idle build lock and no timing observer."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import signal
import statistics
import subprocess
import tempfile
import time


def digest(path):
    value = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            value.update(chunk)
    return value.hexdigest()


def sample(binary, root, jobs, accounting, config):
    args = [str(binary), "check", "--root", str(root), "--effort", "max",
            "--timeout", "0", "--format", "json"]
    if jobs != "default":
        args.extend(["--jobs", jobs])
    if accounting:
        args.append("--include-suppressed")
    if config:
        args.extend(["--config", config])
    env = dict(os.environ)
    for key in ("RAYON_NUM_THREADS", "NO_MISTAKES_TRACE", "CARGO_BUILD_JOBS"):
        env.pop(key, None)
    with tempfile.TemporaryFile() as stdout, tempfile.TemporaryFile() as stderr:
        started = time.monotonic()
        process = subprocess.Popen(args, stdout=stdout, stderr=stderr, env=env,
                                   start_new_session=True)
        def expired(_signal, _frame):
            os.killpg(process.pid, signal.SIGKILL)
        previous = signal.signal(signal.SIGALRM, expired)
        signal.alarm(120)
        try:
            _, status, usage = os.wait4(process.pid, 0)
            process.returncode = os.waitstatus_to_exitcode(status)
        except BaseException:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait()
            raise
        finally:
            signal.alarm(0)
            signal.signal(signal.SIGALRM, previous)
        wall = time.monotonic() - started
        stdout.seek(0)
        stderr.seek(0)
        report, errors = stdout.read(), stderr.read()
    if process.returncode not in (0, 1):
        raise RuntimeError(f"{root.name}: exit {process.returncode}: {errors.decode()}")
    if b"[timing]" in errors or b"[work]" in errors:
        raise RuntimeError("timing observer must remain disabled")
    parsed = json.loads(report)
    return {"wallSeconds": wall, "cpuSeconds": usage.ru_utime + usage.ru_stime,
            "userSeconds": usage.ru_utime, "systemSeconds": usage.ru_stime,
            "peakRssKiB": usage.ru_maxrss / (1024 if platform.system() == "Darwin" else 1),
            "exit": process.returncode, "reported": len(parsed.get("rules", [])),
            "suppressed": len(parsed.get("suppressed", [])),
            "stdoutSha256": hashlib.sha256(report).hexdigest(),
            "stderrSha256": hashlib.sha256(errors).hexdigest()}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--binary", action="append", required=True, help="label=executable")
    parser.add_argument("--revision", action="append", required=True, help="label=git-revision")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--blocks", type=int, default=3)
    parser.add_argument("--jobs", nargs="+", default=["1", "4", "default"])
    parser.add_argument("--preflight-only", action="store_true")
    parser.add_argument("--case", action="append", help="measure only these fixture names")
    options = parser.parse_args()
    binaries = {name: Path(path).resolve() for name, path in
                (entry.split("=", 1) for entry in options.binary)}
    revisions = dict(entry.split("=", 1) for entry in options.revision)
    if set(binaries) != set(revisions):
        parser.error("each binary requires its exact source revision")
    hashes = {name: digest(path) for name, path in binaries.items()}
    if len(set(hashes.values())) != len(hashes):
        parser.error("variants must be separately verified executables")
    root = options.root.resolve()
    cases = [(mode, root / "fixtures/performance/suppression-index" / mode,
              account, None)
             for mode in ("dense", "no-directives", "single", "file-disable",
                          "long-header", "early-quoted-marker")
             for account in (False, True)]
    cases.extend((f"ast-grep-{mode}", root / "fixtures/performance/ast-grep-source-reuse" / mode,
                  False, None) for mode in ("single", "repeated-12", "repeated-20"))
    cases.extend([
        ("compatibility", root / "test-cases/rules/forbidden-calls/suppression-index/fixture",
         True, None),
        ("core-analysis", root / "fixtures/performance/core-analysis", False, None),
        ("nextjs-mixed", root / "fixtures/performance/nextjs-raw-source-scope",
         False, "mixed.yml"),
    ])
    result = {"system": platform.system(), "machine": platform.machine(),
              "logicalCpus": os.cpu_count(), "timingObserver": "off",
              "profile": {"name": "release", "lto": "thin", "codegenUnits": 16,
                          "features": "default", "cargoBuildJobs": 8},
              "rustc": subprocess.check_output(["rustc", "--version"], text=True).strip(),
              "variants": {name: {"revision": revisions[name], "binarySha256": hashes[name]}
                           for name in binaries}, "cases": []}
    labels = list(binaries)
    for mode, fixture, accounting, config in cases:
        if options.case and mode not in options.case:
            continue
        for jobs in options.jobs:
            name = f"{mode}/accounting-{int(accounting)}/jobs-{jobs}"
            values = {label: [] for label in labels}
            reference = None
            def collect(label, retain):
                nonlocal reference
                value = sample(binaries[label], fixture, jobs, accounting, config)
                identity = (value["exit"], value["stdoutSha256"], value["stderrSha256"])
                if reference is None:
                    reference = identity
                if identity != reference:
                    raise RuntimeError(f"report/status/stderr parity failed: {name} {label}")
                if mode in ("dense", "no-directives", "single", "file-disable",
                            "long-header", "early-quoted-marker"):
                    reported, suppressed = {"dense": (2800, 200), "no-directives": (3000, 0),
                                            "single": (0, 1), "file-disable": (0, 3000),
                                            "long-header": (0, 1), "early-quoted-marker": (1, 0)}[mode]
                    if (value["reported"], value["suppressed"]) != (
                            reported, suppressed if accounting else 0):
                        raise RuntimeError(f"fixture preflight failed: {name} {value}")
                if mode.startswith("ast-grep-") and (value["reported"] or value["suppressed"]):
                    raise RuntimeError(f"YAML preflight failed: {name} {value}")
                if mode == "compatibility" and (value["reported"], value["suppressed"]) != (2, 3):
                    raise RuntimeError(f"compatibility preflight failed: {name} {value}")
                if retain:
                    values[label].append(value)
            for label in labels:
                collect(label, False)
            if not options.preflight_only:
                for _ in range(options.blocks):
                    for label in labels + labels[::-1]:
                        collect(label, True)
            summary = {}
            for label, rows in values.items():
                summary[label] = {key: {"median": statistics.median(row[key] for row in rows),
                                        "min": min(row[key] for row in rows),
                                        "max": max(row[key] for row in rows)}
                                  for key in ("wallSeconds", "cpuSeconds", "peakRssKiB")} if rows else {}
            result["cases"].append({"name": name, "samples": values, "summary": summary,
                                    "parity": {"exit": reference[0], "stdoutSha256": reference[1],
                                               "stderrSha256": reference[2]}})
            options.output.parent.mkdir(parents=True, exist_ok=True)
            options.output.write_text(json.dumps(result, indent=2) + "\n")
            print(name, "parity ok", flush=True)


if __name__ == "__main__":
    main()
