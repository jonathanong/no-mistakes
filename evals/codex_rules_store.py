#!/usr/bin/env python3
"""On-disk attempts for the Codex rules runner.

Each attempt writes its JSONL transcript before grading. `result.json` is the
grading marker. A later invocation uses the saved exec to retry judge errors.
`aggregate-result.json` is rewritten from the markers so a crash keeps the
attempts that already finished.
"""
import hashlib
import json
from pathlib import Path

from codex_rules_env import exec_argv, plugin_files
from codex_rules_grade import aggregate_report, judge_argv, judge_error_count, judge_prompt


def _digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def check_run_inputs(output: Path, cases: list, runs: int | None, plugin_src: Path) -> None:
    """Bind future paid resumes to the inputs that produced this directory."""
    inputs = {
        "agent": exec_argv("codex", Path("workspace"), Path("last-message.txt"), ""),
        "judge": judge_argv() + [judge_prompt("", "")],
        "plugin": {
            path.relative_to(plugin_src).as_posix(): _digest(path.read_bytes())
            for path in plugin_files(plugin_src)
        },
        "cases": {
            case["name"]: {
                "prompt": _digest(case["prompt"].encode("utf-8")),
                "timeoutSeconds": case["timeout_seconds"],
                "runs": case["runs"] if runs is None else runs,
                "graders": {
                    path.name: _digest(path.read_bytes())
                    for path in (case["dir"] / "graders").glob("*.md")
                },
            }
            for case in cases
        },
    }
    manifest = output / "run-inputs.json"
    fingerprint = _digest(json.dumps(inputs, sort_keys=True).encode("utf-8"))
    expected = {"version": 1, "sha256": fingerprint}
    if manifest.is_file():
        if json.loads(manifest.read_text()) != expected:
            raise SystemExit("refusing resume: run inputs changed; use a new --output directory")
    elif (output / "attempts").exists() or (output / "aggregate-result.json").exists():
        raise SystemExit("refusing resume: saved run has no input fingerprint; use a new --output directory")
    else:
        _atomic_json(manifest, expected)


def slot(output: Path, case_name: str, arm: str, index: int) -> Path:
    if arm not in ("with", "without") or index < 1:
        raise ValueError(f"bad attempt {case_name} {arm} {index}")
    return output / "attempts" / case_name / arm / str(index)


def transcript_rel(case_name: str, arm: str, index: int) -> str:
    return f"attempts/{case_name}/{arm}/{index}/transcript.jsonl"


def _atomic_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(text)
    temporary.replace(path)


def _atomic_json(path: Path, payload) -> None:
    _atomic_text(path, json.dumps(payload, indent=2) + "\n")


def read_result(path: Path) -> dict | None:
    result = path / "result.json"
    if not result.is_file():
        return None
    return json.loads(result.read_text())


def read_exec(path: Path) -> dict | None:
    """Return a finished exec, or None when this slot still needs `codex exec`."""
    meta_path = path / "exec.json"
    transcript = path / "transcript.jsonl"
    if not meta_path.is_file() or not transcript.is_file():
        return None
    meta = json.loads(meta_path.read_text())
    last = path / "last-message.txt"
    return {
        "returncode": meta["returncode"],
        "stdout": transcript.read_text(),
        "stderr": meta.get("stderr") or "",
        "last_message": last.read_text() if last.is_file() else "",
        "plugin_root": meta.get("plugin_root"),
        "marketplace_root": meta.get("marketplace_root"),
        "attempt_root": meta.get("attempt_root"),
        "cwd": meta.get("cwd"),
    }


def save_exec(path: Path, result: dict) -> None:
    path.mkdir(parents=True, exist_ok=True)
    transcript = path / "transcript.jsonl"
    if not transcript.is_file():
        text = result.get("stdout") or ""
        if text and not text.endswith("\n"):
            text += "\n"
        _atomic_text(transcript, text)
    _atomic_text(path / "last-message.txt", result.get("last_message") or "")
    _atomic_json(
        path / "exec.json",
        {
            "returncode": result["returncode"],
            "stderr": (result.get("stderr") or "")[-2000:],
            "plugin_root": result.get("plugin_root"),
            "marketplace_root": result.get("marketplace_root"),
            "attempt_root": result.get("attempt_root"),
            "cwd": result.get("cwd"),
        },
    )


def save_result(path: Path, row: dict) -> None:
    _atomic_json(path / "result.json", row)


def in_progress(output: Path) -> bool:
    aggregate = output / "aggregate-result.json"
    if not aggregate.is_file():
        return False
    report = json.loads(aggregate.read_text())
    return report.get("inProgress", report.get("partial", False))


def write_aggregate(output: Path, cases: list, *, partial: bool) -> dict:
    report = aggregate_report(cases)
    rows = [row for case in cases for arm in case["arms"].values() for row in arm]
    judge_errors = sum(judge_error_count(row) for row in rows)
    flagged = sum(bool(row.get("isolationFindings")) for row in rows)
    withheld = sum(bool(row.get("judgeWithheld")) and not row.get("isolationFindings") for row in rows)
    report["inProgress"] = partial
    report["judgeErrorCount"] = judge_errors
    report["isolationFlaggedAttempts"] = flagged
    reasons = []
    if partial:
        reasons.append("codex rules run still in progress")
    if judge_errors:
        reasons.append(f"{judge_errors} judge error(s)")
    if flagged:
        reasons.append(f"{flagged} attempt(s) flagged by the isolation audit")
    if withheld:
        reasons.append(f"{withheld} attempt(s) awaiting a judge verdict")
    report["partial"] = bool(reasons)
    report["partialReason"] = ", ".join(reasons) or None
    output.mkdir(parents=True, exist_ok=True)
    _atomic_json(output / "aggregate-result.json", report)
    return report
