"""Balanced native wait4 samples; performance numbers are evidence, not a gate."""
import hashlib
import json
import math
import os
import platform
import signal
import statistics
import subprocess
import sys
import tempfile
import time
from pathlib import Path

from common import command, descendant_usage, digest, distinct, parity, refs, rss_kib, save, verbose_metrics

FIXTURES = ("annotation-excluded-callers-large", "annotation-excluded-callers-unrelated",
            "annotation-excluded-shared-initials")


def sample(binary, root, jobs, verbose=False):
    args = [str(binary), "check", "--root", str(root), "--effort", "max",
            "--timeout", "0", "--format", "json"]
    if verbose:
        args.append("--verbose-timings")
    if jobs != "default":
        args.extend(["--jobs", str(jobs)])
    env = dict(os.environ)
    for name in ("RAYON_NUM_THREADS", "NO_MISTAKES_TRACE", "CARGO_BUILD_JOBS"):
        env.pop(name, None)
    with tempfile.TemporaryFile() as stdout, tempfile.TemporaryFile() as stderr:
        started = time.monotonic()
        process = subprocess.Popen(args, stdout=stdout, stderr=stderr, env=env,
                                   start_new_session=True)
        def expired(signum, frame):
            os.kill(process.pid, signal.SIGTERM)
            signal.signal(signal.SIGALRM, lambda *_: os.killpg(process.pid, signal.SIGKILL))
            signal.alarm(5)
        previous = signal.signal(signal.SIGALRM, expired)
        signal.alarm(60)
        try:
            _, status, usage = os.wait4(process.pid, 0)
            process.returncode = os.waitstatus_to_exitcode(status)
        finally:
            signal.alarm(0)
            signal.signal(signal.SIGALRM, previous)
        wall = time.monotonic() - started
        stdout.seek(0)
        stderr.seek(0)
        report, errors = stdout.read(), stderr.read()
        if process.returncode < 0:
            raise RuntimeError("sample was signaled or exceeded its 60-second cap")
        json.loads(report)
        metrics = verbose_metrics(errors) if verbose else {"work": None, "timingsMs": None}
        return {**metrics, "verboseStderr": errors.decode() if verbose else None,
                "wallSeconds": wall, "userSeconds": usage.ru_utime,
                "systemSeconds": usage.ru_stime, "cpuSeconds": usage.ru_utime + usage.ru_stime,
                "rssKiB": rss_kib(usage.ru_maxrss, platform.system()),
                "exit": process.returncode, "reportSha256": hashlib.sha256(report).hexdigest(),
                "stderrSha256": hashlib.sha256(errors).hexdigest()}


def main():
    if platform.system() != "Darwin":
        raise RuntimeError("native macOS is required")
    base, head = refs(os.environ["BASE_SHA"], os.environ["HEAD_SHA"])
    temporary = Path(os.environ["RUNNER_TEMP"])
    output = temporary / "annotation-results"
    output.mkdir()
    binaries, manifests = {}, {}
    for variant, revision in (("base", base), ("candidate", head)):
        folder = temporary / "annotation-artifacts" / f"annotation-{variant}"
        binary = folder / "no-mistakes"
        manifest = json.loads((folder / "manifest.json").read_text())
        for field, expected in (("variant", variant), ("source", revision), ("base", base),
                                ("head", head), ("system", "Darwin"),
                                ("machine", platform.machine()), ("binarySha256", digest(binary))):
            if manifest[field] != expected:
                raise ValueError(f"artifact provenance mismatch: {variant} {field}")
        if manifest["profile"] != {"lto": "thin", "codegenUnits": 16, "features": "default",
                                    "cargoBuildJobs": 6, "rustflags": ""}:
            raise ValueError("unexpected release profile")
        binary.chmod(0o755)
        if command(str(binary), "--version") != manifest["version"]:
            raise ValueError("executable version does not match manifest")
        binaries[variant], manifests[variant] = binary, manifest
    for field in ("compiler", "os", "runnerImage", "version"):
        parity(manifests["base"][field], manifests["candidate"][field], f"build provenance {field}")
    parity(manifests["base"]["runnerImage"], os.environ.get("ImageVersion"), "measurement runner image")
    distinct(manifests["base"]["binarySha256"], manifests["candidate"]["binarySha256"])
    probe = subprocess.Popen([sys.executable, str(Path(__file__).with_name("resource_probe.py"))])
    _, probe_status, probe_usage = os.wait4(probe.pid, 0)
    probe.returncode = os.waitstatus_to_exitcode(probe_status)
    if probe.returncode:
        raise RuntimeError("native resource-accounting calibration failed")
    probe_cpu = probe_usage.ru_utime + probe_usage.ru_stime
    probe_rss = rss_kib(probe_usage.ru_maxrss, platform.system())
    descendant_usage(probe_cpu, probe_rss)
    samples, groups, fixtures, diagnostics = [], [], [], []
    save(output / "provenance.json", manifests)
    with tempfile.TemporaryDirectory(prefix="annotation-measure-") as directory:
        tree = Path(directory) / "source"
        subprocess.run(["git", "worktree", "add", "--detach", str(tree), head], check=True)
        try:
            for fixture in FIXTURES:
                root = tree / "fixtures/performance" / fixture
                sources = [path for path in root.rglob("*") if path.suffix in
                           (".js", ".jsx", ".mjs", ".cjs", ".ts", ".tsx", ".mts", ".cts")]
                fixtures.append({"fixture": fixture, "sourceFiles": len(sources),
                                 "sourceBytes": sum(path.stat().st_size for path in sources),
                                 "sourceLines": sum(len(path.read_text().splitlines()) for path in sources),
                                 "jsxFiles": sum(path.suffix in (".jsx", ".tsx") for path in sources),
                                 "configuration": (root / ".no-mistakes.yml").read_text()})
                expected = None
                for jobs in (1, 4, "default"):
                    # Four balanced blocks provide eight samples per binary/group.
                    for variant in ("base", "candidate", "candidate", "base") * 4:
                        result = sample(binaries[variant], root, jobs)
                        result.update(fixture=fixture, jobs=jobs, variant=variant)
                        samples.append(result)
                        save(output / "samples.json", samples)
                        identity = (result["exit"], result["reportSha256"], result["stderrSha256"])
                        if expected is None:
                            expected = identity
                        parity(expected, identity, f"{fixture} jobs={jobs} report/exit/stderr")
                        if result["exit"] not in (0, 1):
                            raise ValueError("unexpected check exit status")
                    group = {"fixture": fixture, "jobs": jobs}
                    for variant in ("base", "candidate"):
                        selected = [s for s in samples if s["fixture"] == fixture
                                    and s["jobs"] == jobs and s["variant"] == variant]
                        metrics = {}
                        for metric in ("userSeconds", "systemSeconds", "cpuSeconds", "wallSeconds", "rssKiB"):
                            values = sorted(s[metric] for s in selected)
                            metrics[metric] = {"mean": statistics.mean(values), "max": max(values),
                                               "p95": values[math.ceil(len(values) * .95) - 1]}
                        group[variant] = {"count": len(selected), "metrics": metrics}
                    group["changePercent"] = {
                        metric: (group["candidate"]["metrics"][metric]["mean"] /
                                 group["base"]["metrics"][metric]["mean"] - 1) * 100
                        for metric in ("userSeconds", "cpuSeconds", "wallSeconds", "rssKiB")
                        if group["base"]["metrics"][metric]["mean"] != 0}
                    groups.append(group)
                for variant in ("base", "candidate"):
                    observed = sample(binaries[variant], root, "default", verbose=True)
                    parity(expected[:2], (observed["exit"], observed["reportSha256"]),
                           f"{fixture} observer-on report/exit")
                    observed.update(fixture=fixture, variant=variant, observer="on", samples=1)
                    diagnostics.append(observed)
                    save(output / "diagnostics.json", diagnostics)
        finally:
            subprocess.run(["git", "worktree", "remove", "--force", str(tree)], check=True)
    summary = {"groups": groups, "fixtures": fixtures, "provenance": manifests, "measurement": {
        "system": platform.platform(), "machine": platform.machine(), "availableCpus": os.cpu_count(),
        "cpu": command("sysctl", "-n", "machdep.cpu.brand_string"),
        "workflowRun": os.environ.get("GITHUB_RUN_ID"), "runAttempt": os.environ.get("GITHUB_RUN_ATTEMPT"),
        "runnerImage": os.environ.get("ImageVersion"), "order": "B/C/C/B repeated four times",
        "resourceCalibration": {"descendantCpuSeconds": probe_cpu, "descendantRssKiB": probe_rss},
        "warmup": "none; initial samples retained", "observer": "off",
        "config": "unchanged saved fixture effort:max",
        "command": "check --root <fixture> --effort max --timeout 0 --format json [--jobs 1|4]", "parity": "exact stdout SHA256 and exit across all samples/jobs",
        "limitations": "Eight samples per binary/group; empirical p95 equals maximum. Hosted noise and cache warming remain; no CPU tolerance gate."}}
    summary["diagnostics"] = diagnostics
    save(output / "summary.json", summary)
    lines = ["# Native macOS annotation comparison", "", summary["measurement"]["limitations"], "",
             "Order: B/C/C/B × 4. Default features, ThinLTO/16 CGU, observer off; no warmups discarded.", "",
             "| Public fixture | Jobs | User CPU s B→C | Total CPU s B→C | Wall s B→C | Peak RSS KiB B→C |",
             "|---|---|---|---|---|---|"]
    for group in groups:
        pairs = []
        for metric in ("userSeconds", "cpuSeconds", "wallSeconds", "rssKiB"):
            field = "max" if metric == "rssKiB" else "mean"
            values = [group[v]["metrics"][metric][field] for v in ("base", "candidate")]
            pairs.append(f"{values[0]:.6f} → {values[1]:.6f}")
        lines.append(f"| {group['fixture']} | {group['jobs']} | " + " | ".join(pairs) + " |")
    lines.extend(["", "Exact source/compiler/binary provenance, user/system CPU, max/p95 and raw bounded samples are in the JSON artifacts."])
    (output / "summary.md").write_text("\n".join(lines) + "\n")
    with open(os.environ["GITHUB_STEP_SUMMARY"], "a") as stream:
        stream.write((output / "summary.md").read_text())


if __name__ == "__main__":
    main()
