#!/usr/bin/env python3
"""Reject incomplete/failed checks, compare semantics, and summarize raw runs."""
import hashlib
import json
import statistics
import sys
from collections import defaultdict
from pathlib import Path

REQUIRED_ARRAYS = ("react", "queues", "rules", "integration", "codebase", "warnings", "advisories")
def project(value):
    normalized = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    warning_data = json.dumps({key: value[key] for key in ("warnings", "advisories")},
                              sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(normalized).hexdigest(), hashlib.sha256(warning_data).hexdigest()


def read_run(path):
    row = json.loads(path.read_text())
    if row.get("timed_out") or row.get("exit") not in (0, 1):
        raise ValueError(f"timeout or unexpected exit status in {path}")
    stdout = path.with_suffix(".stdout").read_text()
    if not stdout.strip():
        raise ValueError(f"empty analyzer output in {path}")
    report = json.loads(stdout)
    if not isinstance(report, dict) or any(not isinstance(report.get(k), list) for k in REQUIRED_ARRAYS):
        raise ValueError(f"incomplete check JSON shape in {path}")
    row["normalized_report_sha256"], row["warnings_sha256"] = project(report)
    stderr = path.with_suffix(".stderr").read_text()
    semantic_lines = [line for line in stderr.splitlines()
                      if not line.startswith(("[progress] ", "[timing] ", "[work] "))]
    row["stderr_semantics_sha256"] = hashlib.sha256("\n".join(semantic_lines).encode()).hexdigest()
    return row


def main():
    if len(sys.argv) != 3 or sys.argv[1] != "summarize":
        raise SystemExit("usage: mimalloc-report.py summarize MEASUREMENTS_DIR")
    root = Path(sys.argv[2])
    groups = defaultdict(lambda: defaultdict(list))
    for path in sorted(path for path in root.glob("*.json") if path.name != "summary.json"):
        row = read_run(path)
        meta = row["metadata"]
        key = (meta["workload"], meta["diagnostics"], meta["jobs"])
        groups[key][meta["variant"]].append(row)
    summary = []
    for key, variants in sorted(groups.items()):
        if set(variants) != {"system", "mimalloc"} or any(len(rows) != 4 for rows in variants.values()):
            raise ValueError(f"missing four samples per variant for {key}")
        left = {row["metadata"]["repetition"]: row for row in variants["system"]}
        right = {row["metadata"]["repetition"]: row for row in variants["mimalloc"]}
        if set(left) != {0, 1, 2, 3} or set(right) != {0, 1, 2, 3}:
            raise ValueError(f"duplicate or missing repetition index for {key}")
        for repetition in left:
            a, b = left[repetition], right[repetition]
            if (a["exit"], a["normalized_report_sha256"], a["warnings_sha256"], a["stderr_semantics_sha256"]) != \
               (b["exit"], b["normalized_report_sha256"], b["warnings_sha256"], b["stderr_semantics_sha256"]):
                raise ValueError(f"report, warning, or status mismatch for {key} repetition {repetition}")
        metrics = {}
        for variant, rows in variants.items():
            metrics[variant] = {name: statistics.median(row[name] for row in rows)
                                for name in ("wall_seconds", "user_seconds", "system_seconds", "peak_rss_bytes")}
        summary.append({"workload": key[0], "diagnostics": key[1], "jobs": key[2], "samples_per_variant": 4,
                        "system": metrics["system"], "mimalloc": metrics["mimalloc"],
                        "candidate_minus_system": {k: metrics["mimalloc"][k] - metrics["system"][k]
                                                   for k in metrics["system"]}})
    if len(summary) != 18:
        raise ValueError(f"expected 18 workload/diagnostic/jobs cells, found {len(summary)}")
    (root / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
