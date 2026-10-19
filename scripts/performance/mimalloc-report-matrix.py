"""Explicit required allocator matrices and shared run identity checks."""
import itertools
import json
from collections import defaultdict

CELL_FIELDS = ("platform", "workload", "diagnostics", "jobs")
WORKLOADS = ("consumer", "hosted-acceptance", "hosted-passing-failing")
HOSTED_PLATFORMS = ("linux-arm64", "macos-arm64")
HOSTED_JOBS = ("1", "4", "max")
MANY_CORE_JOBS = ("1", "4", "16", "max")
# Same names verify_artifact accepts, including the x64 release target.
PLATFORMS = ("linux-arm64", "linux-x64", "macos-arm64")


def matrix_cells(matrix, platform):
    """Return the required (platform, workload, diagnostics, jobs) tuples."""
    if matrix == "hosted":
        cells = set(itertools.product(HOSTED_PLATFORMS, WORKLOADS, (False, True), HOSTED_JOBS))
        return cells, "all 36 native cells"
    if matrix == "napi":
        return set(itertools.product(HOSTED_PLATFORMS, ("consumer-napi",), (False,), ("1", "max"))), "all 4 N-API cells"
    if matrix == "many-core":
        # One host. Jobs include 16, which the hosted 4-vCPU runners cannot separate from max.
        cells = set(itertools.product((platform,), WORKLOADS, (False, True), MANY_CORE_JOBS))
        return cells, "all 24 many-core cells"
    raise ValueError(f"unknown measurement matrix {matrix!r}")


def many_core_platform(rows, platform):
    found = {row["metadata"].get("platform") for row in rows}
    if platform is None:
        if len(found) != 1 or None in found:
            raise ValueError("many-core matrix requires exactly one platform")
        return next(iter(found))
    if platform not in PLATFORMS:
        raise ValueError(f"unknown platform {platform!r}")
    return platform


def reject_small_many_core_host(rows):
    if len({row.get("host_id") for row in rows}) != 1:
        raise ValueError("many-core matrix requires one physical host_id")
    for row in rows:
        meta = row["metadata"]
        available = row.get("available_cpus")
        automatic, probe = meta.get("automatic_rayon_jobs"), meta.get("num_cpus_default")
        if (type(available) is not int or available < 16 or type(automatic) is not int
                or automatic < 16 or type(probe) is not int or probe < 16
                or automatic != probe or meta.get("rayon_environment")):
            raise ValueError("many-core requires at least 16 available CPUs and unmodified automatic_rayon_jobs")


def summarize(rows, matrix, platform, summarize_group):
    if matrix not in ("hosted", "many-core", "napi"):
        raise ValueError(f"unknown measurement matrix {matrix!r}")
    rows = list(rows)
    if not rows:
        raise ValueError("no measurement samples were produced")
    groups = defaultdict(list)
    identities = defaultdict(set)
    sources, locks = set(), set()
    workloads = defaultdict(set)
    for row in rows:
        groups[tuple(row["metadata"][key] for key in CELL_FIELDS)].append(row)
        identities[(row["metadata"]["platform"], row["metadata"]["variant"])].add(row["metadata"].get("artifact_sha256"))
        sources.add(row["metadata"].get("source"))
        locks.add(json.dumps(row["metadata"].get("lock"), sort_keys=True))
        workloads[row["metadata"]["workload"]].add((row["metadata"].get("workload_commit"),
                                                   row["metadata"].get("workload_tree")))
    if matrix == "many-core":
        platform = many_core_platform(rows, platform)
    expected, label = matrix_cells(matrix, platform)
    if set(groups) != expected:
        raise ValueError(f"expected {label}; missing={expected - set(groups)}, extra={set(groups) - expected}")
    if matrix == "many-core":
        reject_small_many_core_host(rows)
    if any(len(hashes) != 1 or None in hashes for hashes in identities.values()):
        raise ValueError("executable identity changed across native cells")
    if len(sources) != 1 or None in sources or len(locks) != 1 or "null" in locks:
        raise ValueError("source or lock identity changed across native cells")
    if any(len(pins) != 1 or any(not commit or not tree for commit, tree in pins)
           for pins in workloads.values()):
        raise ValueError("workload commit or tree changed across native cells")
    return [summarize_group(entries) for _, entries in sorted(groups.items())]
