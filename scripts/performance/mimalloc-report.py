#!/usr/bin/env python3
"""Compare complete semantics and aggregate resources within physical host pairs."""
import argparse
import hashlib
import json
import re
import runpy
from pathlib import Path

STATS = runpy.run_path(str(Path(__file__).with_name("mimalloc-report-stats.py")))
REQUIRED_ARRAYS = ("react", "queues", "rules", "integration", "codebase", "warnings", "advisories")
SEMANTICS = ("normalized_report_sha256", "warnings_sha256", "stderr_semantics_sha256")
HOST_FIELDS = ("host_id", "cwd", "platform", "machine", "cpu_model", "available_cpus")
CELL_FIELDS = ("platform", "workload", "diagnostics", "jobs")
BUILD_FIELDS = ("source", "lock", "lto", "codegenUnits", "rayon_jobs", "automatic_rayon_jobs",
                "rayon_environment", "workload_commit", "workload_tree", "rustc", "cargo", "target",
                "build_flags", "num_cpus_default", "tree", "measurement_id")
MATRIX = runpy.run_path(str(Path(__file__).with_name("mimalloc-report-matrix.py")))
PLATFORMS = MATRIX["PLATFORMS"]
ADOPTION_RULE = runpy.run_path(str(Path(__file__).with_name("mimalloc-adoption-rule.py")))["ADOPTION_RULE"]


def mimalloc_environment(row):
    if "mimalloc_environment" in row:
        value = row["mimalloc_environment"]
    else:
        value = row.get("metadata", {}).get("mimalloc_environment", {})
    if not isinstance(value, dict):
        return None
    return value


def project(value):
    normalized = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    warnings = json.dumps({key: value[key] for key in ("warnings", "advisories")},
                          sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(normalized).hexdigest(), hashlib.sha256(warnings).hexdigest()


def read_run(path):
    row = json.loads(path.read_text())
    row["cpu_seconds"] = STATS["cpu_seconds"](row)
    row["status"] = STATS["classify"](row)
    row["resources_eligible"] = STATS["eligible"](row)
    if row["status"] in ("timeout", "signal"):
        return row
    stdout = path.with_suffix(".stdout").read_text()
    if not stdout.strip():
        raise ValueError(f"empty analyzer output in {path}")
    report = json.loads(stdout)
    if not isinstance(report, dict) or any(not isinstance(report.get(k), list) for k in REQUIRED_ARRAYS):
        raise ValueError(f"incomplete check JSON shape in {path}")
    row["normalized_report_sha256"], row["warnings_sha256"] = project(report)
    stderr = path.with_suffix(".stderr").read_text()
    lines = [line for line in stderr.splitlines()
             if not line.startswith(("[progress] ", "[timing] ", "[work] "))
             and not re.fullmatch(r"\[memory\] peak_rss_kib: [0-9]+", line)]
    row["stderr_semantics_sha256"] = hashlib.sha256("\n".join(lines).encode()).hexdigest()
    return row


def summarize_pair(a, b, pair):
    order = "AB" if pair in (0, 3) else "BA"
    errors = []
    if any(a.get(key) in (None, "") or a.get(key) != b.get(key) for key in HOST_FIELDS):
        errors.append("missing or mismatched physical host facts")
    if any(a["metadata"].get(key) != b["metadata"].get(key) for key in CELL_FIELDS):
        errors.append("mismatched cell metadata")
    if any(key not in a["metadata"] or key not in b["metadata"]
           or key != "rayon_environment" and a["metadata"].get(key) in (None, "")
           or a["metadata"].get(key) != b["metadata"].get(key)
           for key in BUILD_FIELDS):
        errors.append("missing or mismatched source, lock, profile, or Rayon facts")
    for row in (a, b):
        meta = row["metadata"]
        actual = meta.get("rayon_jobs")
        automatic = meta.get("automatic_rayon_jobs")
        expected = automatic if meta["jobs"] == "max" else int(meta["jobs"])
        if type(actual) is not int or actual <= 0 or actual != expected:
            errors.append("invalid effective Rayon thread count")
    if any(row["metadata"].get("order") != order for row in (a, b)):
        errors.append("mismatched balanced order")
    if any(row["metadata"].get("position") != (0 if variant == order[0] else 1)
           for row, variant in ((a, "A"), (b, "B"))):
        errors.append("mismatched execution position")
    if not a.get("argv") or not b.get("argv") or a["argv"][1:] != b["argv"][1:]:
        errors.append("mismatched workload command")
    if a.get("exit") != b.get("exit") or any(not a.get(k) or a.get(k) != b.get(k) for k in SEMANTICS):
        errors.append("report, warning, or status mismatch")
    if any(STATS["classify"](row) not in ("success", "expected_failure") for row in (a, b)):
        errors.append("unexpected or censored process result")
    environments = [mimalloc_environment(row) for row in (a, b)]
    if (any(environment is None for environment in environments)
            or environments[0] != environments[1] or any(environments)):
        errors.append("non-empty or mismatched MIMALLOC_ environment")
    if a["metadata"]["workload"] == "consumer-napi":
        for key in ("node_version", "node_sha256"):
            if not a["metadata"].get(key) or a["metadata"].get(key) != b["metadata"].get(key):
                errors.append("missing or mismatched Node identity")
        if any(row.get("executable_sha256") != row["metadata"].get("node_sha256") for row in (a, b)):
            errors.append("Node executable identity mismatch")
    if a["metadata"]["workload"] != "consumer-napi":
        if any(row.get("executable_sha256") != row["metadata"].get("artifact_sha256") for row in (a, b)):
            errors.append("CLI executable does not match allocator artifact identity")
    deltas, ratios = {}, {}
    completed = all(STATS["eligible"](row) for row in (a, b))
    for key in STATS["METRICS"]:
        values = completed and all(STATS["valid_metric"](row.get(key)) for row in (a, b))
        if not values:
            errors.append(f"ineligible paired metric: {key}")
        deltas[key] = b[key] - a[key] if values else None
        ratios[key] = b[key] / a[key] if values and a[key] else None
    return {"pair": pair, "host_id": a.get("host_id"), "measurement_id": a["metadata"].get("measurement_id"), "order": order,
            "hostFacts": {key: a.get(key) for key in HOST_FIELDS},
            "node": {key: a["metadata"].get(key) for key in ("node_version", "node_sha256")}, "valid": not errors,
            "statuses": {name: STATS["classify"](row) for name, row in (("system", a), ("mimalloc", b))},
            "exits": {name: row.get("exit") for name, row in (("system", a), ("mimalloc", b))},
            "provenance": {key: a["metadata"].get(key) for key in BUILD_FIELDS},
            "errors": errors, "delta": deltas, "ratio": ratios}


def summarize_group(rows):
    rows = [dict(row, cpu_seconds=STATS["cpu_seconds"](row)) for row in rows]
    variants = {name: [row for row in rows if row["metadata"]["variant"] == name]
                for name in ("system", "mimalloc")}
    indexed = {name: {row["metadata"].get("pair"): row for row in entries}
               for name, entries in variants.items()}
    if len(rows) != 8 or any(len(entries) != 4 or set(indexed[name]) != {0, 1, 2, 3}
                             for name, entries in variants.items()):
        raise ValueError("duplicate or missing four physical pairs per variant")
    for name, entries in variants.items():
        hashes = {row["metadata"].get("artifact_sha256") for row in entries}
        if None in hashes or len(hashes) != 1:
            raise ValueError(f"missing or changed executable identity for {name}")
    if indexed["system"][0]["metadata"]["artifact_sha256"] == indexed["mimalloc"][0]["metadata"]["artifact_sha256"]:
        raise ValueError("system and mimalloc require distinct allocator artifacts")
    pairs = [summarize_pair(indexed["system"][i], indexed["mimalloc"][i], i) for i in range(4)]
    result = {key: rows[0]["metadata"][key] for key in CELL_FIELDS}
    result.update({name: STATS["summarize_resources"](entries) for name, entries in variants.items()})
    result["artifact_sha256"] = {name: entries[0]["metadata"].get("artifact_sha256") for name, entries in variants.items()}
    result.update({"valid": all(pair["valid"] for pair in pairs), "pairs": pairs,
                   "samples_per_variant": 4,
                   "tail_note": "Four samples: nearest-rank p95 equals max; no guaranteed tail bound."})
    for label in ("delta", "ratio"):
        # Hosts can differ across pairs. Compare within each host first.
        result[f"paired_{label}"] = {
            key: STATS["describe"](pair[label][key] for pair in pairs
                                   if pair["valid"] and pair[label][key] is not None)
            for key in STATS["METRICS"]}
    return result


def summarize(rows, matrix, platform=None):
    return MATRIX["summarize"](rows, matrix, platform, summarize_group)


def main():
    parser = argparse.ArgumentParser(prog="mimalloc-report.py")
    parser.add_argument("command", choices=("summarize",))
    parser.add_argument("--matrix", choices=("hosted", "many-core", "napi"), default="hosted")
    parser.add_argument("--platform", choices=PLATFORMS)
    parser.add_argument("directory")
    args = parser.parse_args()
    if args.matrix == "many-core" and args.platform is None:
        parser.error("--platform is required for the many-core matrix")
    root = Path(args.directory)
    paths = sorted(path for path in root.rglob("*.json") if path.name != "summary.json")
    summary = summarize((read_run(path) for path in paths), args.matrix, platform=args.platform)
    (root / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))
    if not all(group["valid"] for group in summary):
        raise SystemExit("invalid or failed candidate samples; inspect summary.json")


if __name__ == "__main__":
    main()
