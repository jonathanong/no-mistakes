#!/usr/bin/env python3
"""Balanced baseline/candidate CLI benchmark for no-mistakes check."""
import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import time
from pathlib import Path

DEFAULT_FIXTURES = [
    "annotation-excluded-callers-large",
    "annotation-excluded-callers-unrelated",
    "annotation-excluded-shared-initials",
    "postgres-annotations",
    "postgres-branch-copies",
    "annotation-timings",
]
SOURCE_SUFFIXES = {".js", ".jsx", ".mjs", ".cjs", ".ts", ".tsx", ".mts", ".cts"}
TIMING_KEYS = {"timing", "timings", "elapsed", "duration", "durationMs", "elapsedMs", "wallMs", "cpuMs"}


def parse_args():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("root", type=Path, help="repository root containing fixtures/performance")
    p.add_argument("baseline", type=Path, help="frozen baseline CLI binary")
    p.add_argument("candidate", type=Path, help="candidate CLI binary")
    p.add_argument("output_directory", type=Path, help="directory for raw outputs and summaries")
    p.add_argument("--fixtures", action="append", help="fixture directory name (repeat or comma-separate); defaults to requested set")
    p.add_argument("--jobs", default="1,4,default", help="comma-separated jobs values")
    p.add_argument("--samples", type=int, default=8, help="total runs per fixture/job group; half per binary; multiple of 4")
    return p.parse_args()


def fixture_names(args):
    if not args.fixtures:
        return DEFAULT_FIXTURES
    return [name for item in args.fixtures for name in item.split(",") if name]


def strip_timings(value):
    if isinstance(value, dict):
        return {k: strip_timings(v) for k, v in value.items() if k not in TIMING_KEYS and not re.search(r"(?:^|_)(?:timing|duration|elapsed)(?:$|_)", k, re.I)}
    if isinstance(value, list):
        return [strip_timings(v) for v in value]
    return value


def normalize(value, fixture_root):
    value = strip_timings(value)
    root = str(fixture_root.resolve())
    if isinstance(value, dict):
        return {k: normalize(v, fixture_root) for k, v in value.items()}
    if isinstance(value, list):
        values = [normalize(v, fixture_root) for v in value]
        # Report collections are order-independent; canonical sorting avoids noisy diffs.
        return sorted(values, key=lambda x: json.dumps(x, sort_keys=True, ensure_ascii=False))
    if isinstance(value, str):
        return value.replace(root, "<root>").replace(root.replace("/", "\\"), "<root>").replace("\\", "/")
    return value


def source_stats(path):
    files = [f for f in path.rglob("*") if f.is_file() and f.suffix.lower() in SOURCE_SUFFIXES]
    size = lines = jsx_files = 0
    for f in files:
        data = f.read_bytes()
        size += len(data)
        text = data.decode("utf-8", errors="replace")
        lines += len(text.splitlines())
        if re.search(r"<\s*[A-Za-z][\w.:-]*(?:\s|/|>)", text):
            jsx_files += 1
    return {"source_files": len(files), "source_bytes": size, "source_lines": lines,
            "jsx_files": jsx_files, "jsx_density_percent": round(100 * jsx_files / len(files), 2) if files else 0.0}


def report_projection(value):
    if not isinstance(value, dict):
        return value
    keys = ("findings", "warnings", "advisories")
    return {key: value.get(key, []) for key in keys if key in value}


def main():
    a = parse_args()
    root = a.root.resolve()
    base, cand = a.baseline.resolve(), a.candidate.resolve()
    out = a.output_directory.resolve()
    if base == cand:
        raise SystemExit("baseline and candidate must be distinct binary paths")
    digests = [hashlib.sha256(binary.read_bytes()).hexdigest() for binary in (base, cand)]
    if len(set(digests)) != 2:
        raise SystemExit("baseline and candidate binary bytes must differ; discard stale shared-target artifacts")
    if not shutil.which("/usr/bin/time"):
        raise SystemExit("GNU time is required at /usr/bin/time")
    out.mkdir(parents=True, exist_ok=True)
    runtime = Path("/tmp/nm-3085-runtime")
    runtime.mkdir(mode=0o700, parents=True, exist_ok=True)
    jobs = [j.strip() for j in a.jobs.split(",") if j.strip()]
    if a.samples < 4 or a.samples % 4:
        raise SystemExit("--samples must be a positive multiple of 4 to retain balanced B/C/C/B order")
    fixtures, missing, records = [], [], []
    for name in fixture_names(a):
        fixture = (root / "fixtures" / "performance" / name).resolve()
        if not fixture.is_dir():
            missing.append(name)
            continue
        fixtures.append((name, fixture))
    for name, fixture in fixtures:
        stats = source_stats(fixture)
        for job in jobs:
            order = ["baseline", "candidate", "candidate", "baseline"] * (a.samples // 4)
            parsed = {}
            for idx, variant in enumerate(order, 1):
                binary = base if variant == "baseline" else cand
                label = f"{Path(name).name}.jobs-{job}.{variant}.{idx}"
                raw_json = out / f"{label}.stdout.json"
                raw_stderr = out / f"{label}.stderr.txt"
                time_file = out / f"{label}.time.txt"
                cmd = [str(binary), "check", "--root", str(fixture), "--effort", "max", "--format", "json", "--timeout", "0"]
                if job != "default":
                    cmd += ["--jobs", job]
                env = os.environ.copy()
                env["XDG_RUNTIME_DIR"] = str(runtime)
                # No diagnostics flags are passed: observer, timings, and progress remain disabled.
                with raw_json.open("wb") as stdout, raw_stderr.open("wb") as stderr:
                    started = time.perf_counter()
                    measured = subprocess.Popen(["/usr/bin/time", "-q", "-f", "%e %U %S %M", "-o", str(time_file), "--"] + cmd,
                                                cwd=fixture, env=env, stdout=stdout, stderr=stderr)
                    _, status, usage = os.wait4(measured.pid, 0)
                    wall = time.perf_counter() - started
                    measured.returncode = os.waitstatus_to_exitcode(status)
                try:
                    obj = json.loads(raw_json.read_text(encoding="utf-8"))
                except Exception as exc:
                    raise RuntimeError(f"{label}: CLI stdout was not JSON (status {measured.returncode}): {exc}; see raw outputs")
                normalized = normalize(obj, fixture)
                stdout_sha256 = hashlib.sha256(raw_json.read_bytes()).hexdigest()
                parsed.setdefault(variant, []).append((normalized, measured.returncode, stdout_sha256))
                metric = time_file.read_text().strip().split()
                if len(metric) != 4:
                    raise RuntimeError(f"{label}: malformed GNU time output: {metric}")
                records.append({"fixture": name, **stats, "jobs": job, "variant": variant, "sample": idx,
                                "exit_status": measured.returncode, "stdout_sha256": stdout_sha256, "wall_seconds": wall,
                                "user_seconds": usage.ru_utime, "sys_seconds": usage.ru_stime,
                                "peak_rss_kib": int(metric[3])})
            baseline_reports = parsed.get("baseline", [])
            candidate_reports = parsed.get("candidate", [])
            parity = all(item == baseline_reports[0] for item in baseline_reports + candidate_reports) if baseline_reports else False
            if not parity:
                raise RuntimeError(f"normalized report, raw stdout hash, or exit status mismatch for fixture={name}, jobs={job}; inspect raw JSON")
    summary = {"binary_sha256": {"baseline": digests[0], "candidate": digests[1]}, "fixtures": [{"fixture": name, **source_stats(path)} for name, path in fixtures],
               "missing_fixtures": missing, "jobs": jobs, "balanced_runs_per_job": a.samples,
               "records": records, "parity": "equal for all measured fixtures/jobs"}
    (out / "summary.json").write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n")
    lines = ["# CLI benchmark", "", "Observer OFF; monotonic wall, wait4 CPU and GNU time peak RSS; balanced B/C/C/B; report and exit-status parity verified.", "",
             "| Fixture | Jobs | Variant | n | Wall s (mean) | User s (mean) | Sys s (mean) | Peak RSS KiB (max) | Bytes | Lines | Sources | JSX density |",
             "|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|"]
    for name, _ in fixtures:
        stats = source_stats(root / "fixtures" / "performance" / name)
        for job in jobs:
            for variant in ("baseline", "candidate"):
                subset = [r for r in records if r["fixture"] == name and r["jobs"] == job and r["variant"] == variant]
                if not subset:
                    continue
                mean = lambda k: sum(x[k] for x in subset) / len(subset)
                lines.append(f"| {name} | {job} | {variant} | {len(subset)} | {mean('wall_seconds'):.4f} | {mean('user_seconds'):.4f} | {mean('sys_seconds'):.4f} | {max(x['peak_rss_kib'] for x in subset)} | {stats['source_bytes']} | {stats['source_lines']} | {stats['source_files']} | {stats['jsx_density_percent']:.2f}% |")
    if missing:
        lines += ["", "Missing fixtures (skipped until present): " + ", ".join(missing)]
    (out / "summary.md").write_text("\n".join(lines) + "\n")
    print(json.dumps({"fixtures": [n for n, _ in fixtures], "missing_fixtures": missing,
                      "jobs": jobs, "balanced_runs_per_job": a.samples, "parity": summary["parity"],
                      "summary": "summary.json", "metrics": "summary.md"}, sort_keys=True))


if __name__ == "__main__":
    main()
