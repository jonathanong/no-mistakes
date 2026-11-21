#!/usr/bin/env python3
"""On-disk attempts for the Codex rules runner.

Each attempt writes its JSONL transcript before grading. `result.json` is the
completion marker. A later invocation loads that marker and does not exec again.
`aggregate-result.json` is rewritten from the markers so a crash keeps the
attempts that already finished.
"""
import json
from pathlib import Path

from codex_rules_grade import aggregate_report


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
        },
    )


def save_result(path: Path, row: dict) -> None:
    _atomic_json(path / "result.json", row)


def write_aggregate(output: Path, cases: list, *, partial: bool) -> dict:
    report = aggregate_report(cases)
    report["partial"] = partial
    report["partialReason"] = "codex rules run still in progress" if partial else None
    output.mkdir(parents=True, exist_ok=True)
    _atomic_json(output / "aggregate-result.json", report)
    return report
