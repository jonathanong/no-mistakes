"""Status and resource summaries for completed or censored child processes."""
import math
import statistics
from collections import Counter

METRICS = ("wall_seconds", "user_seconds", "system_seconds", "cpu_seconds", "peak_rss_bytes")
STATUSES = ("success", "expected_failure", "unexpected_exit", "timeout", "signal")


def classify(row):
    expected = row.get("metadata", {}).get("expected_exit", 0)
    control = row.get("metadata", {}).get("expected_failure_control") is True
    if type(expected) is not int or expected not in (0, 1) or expected == 1 and not control:
        raise ValueError("nonzero expected exit requires an explicit expected-failure control")
    exit_code = row.get("exit")
    deadline = row.get("metadata", {}).get("timeout_seconds", 600)
    if not valid_metric(deadline) or deadline <= 0:
        raise ValueError("collection deadline must be positive and finite")
    elapsed = row.get("wall_seconds")
    expired = valid_metric(elapsed) and elapsed >= deadline
    if row.get("timed_out") or exit_code in (124, 137) or expired:
        return "timeout"
    if row.get("signal") or isinstance(exit_code, int) and exit_code < 0:
        return "signal"
    if exit_code == expected and type(exit_code) is int:
        return "success" if expected == 0 else "expected_failure"
    return "unexpected_exit"


def eligible(row):
    code = row.get("exit")
    return (classify(row) not in ("timeout", "signal") and type(code) is int
            and code >= 0 and code not in (124, 137))


def valid_metric(value):
    return type(value) in (int, float) and math.isfinite(value) and value >= 0


def describe(values):
    values = sorted(values)
    return {"n": len(values), "median": statistics.median(values) if values else None,
            "max": values[-1] if values else None,
            "p95": values[math.ceil(len(values) * 0.95) - 1] if values else None}


def summarize_resources(rows):
    counts = Counter(classify(row) for row in rows)
    completed = [row for row in rows if eligible(row)]
    return {"sample_count": len(rows), "statuses": {key: counts[key] for key in STATUSES},
            "exits": dict(Counter(str(row.get("exit")) for row in rows)),
            "resources_eligible_count": len(completed),
            "metrics": {key: describe(row[key] for row in completed if valid_metric(row.get(key)))
                        for key in METRICS}}


def cpu_seconds(row):
    values = [row.get(key) for key in ("user_seconds", "system_seconds")]
    return sum(values) if all(valid_metric(value) for value in values) else None
