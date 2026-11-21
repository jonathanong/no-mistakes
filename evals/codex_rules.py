#!/usr/bin/env python3
"""Codex runner for the no-mistakes-rules cases.

    python3 evals/codex_rules.py
        Print cases x arms x runs and exit. This does not call a model.

    python3 evals/codex_rules.py --run --tag rules --runs 3 --output <dir>
        Run both arms. Requires CODEX_AUTH_FILE or CODEX_API_KEY for the
        spare-budget Codex account. Refuses ~/.codex and ~/.claude.

The default command is the preview. --run is what spends money.
"""
import argparse
import fnmatch
import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

from codex_rules_env import (
    APPROVAL,
    MODEL,
    PLUGIN_SRC,
    REASONING,
    SANDBOX,
    build_arm,
    exec_plan,
    reject_output,
    reject_user_path,
)
from codex_rules_grade import (
    JUDGE_MODEL,
    aggregate_report,
    case_prompt,
    format_trigger,
    grade_run,
    judge_argv,
    passed_from_judge_output,
)

_HERE = Path(__file__).resolve().parent


def discover(evals_dir: Path = _HERE) -> list:
    cases = []
    for path in sorted(evals_dir.glob("rules-*")):
        if (path / "prompt.md").is_file() and (path / "graders" / "skill-fired.md").is_file():
            cases.append(case_prompt(path))
    return cases


def filter_cases(cases: list, tags: list, pattern: str | None) -> list:
    if not tags and not pattern:
        return list(cases)
    chosen = []
    for case in cases:
        if pattern and not fnmatch.fnmatch(case["name"], pattern):
            continue
        if tags and not any(tag in case["tags"] for tag in tags):
            continue
        chosen.append(case)
    return chosen


def format_preview(cases: list, runs: int | None) -> str:
    attempts = sum((runs or case["runs"]) * 2 for case in cases)
    uniform = runs
    if uniform is None and cases:
        values = {case["runs"] for case in cases}
        uniform = cases[0]["runs"] if len(values) == 1 else None
    formula = f"{len(cases)} x 2 x {uniform}" if uniform is not None else "per-case runs x 2 arms"
    timeout = cases[0]["timeout_seconds"] if cases else 0
    return (
        "Codex rules eval preview (no model calls)\n"
        f"cases: {len(cases)}\n"
        "arms: 2 (with, without)\n"
        f"runs: {uniform if uniform is not None else 'per case'}\n"
        f"attempts: {attempts} ({formula})\n"
        f"agent model: {MODEL}\n"
        f"reasoning effort: {REASONING}\n"
        f"sandbox: {SANDBOX}\n"
        f"approval: {APPROVAL}\n"
        f"timeout_seconds: {timeout}\n"
        f"judge model: {JUDGE_MODEL}\n"
        "dollars: unknown (not measured)\n"
    )


def exec_codex(plan: dict) -> dict:
    try:
        proc = subprocess.run(
            plan["argv"],
            env=plan["env"],
            cwd=plan["cwd"],
            capture_output=True,
            text=True,
            timeout=plan["timeout_seconds"],
            check=False,
        )
    except subprocess.TimeoutExpired as exc:
        stdout = exc.stdout or ""
        stderr = exc.stderr or "timeout"
        if isinstance(stdout, bytes):
            stdout = stdout.decode("utf-8", errors="replace")
        if isinstance(stderr, bytes):
            stderr = stderr.decode("utf-8", errors="replace")
        return {"returncode": 124, "stdout": stdout, "stderr": stderr, "last_message": ""}
    last = Path(plan["last_message"])
    message = last.read_text() if last.is_file() else ""
    return {
        "returncode": proc.returncode,
        "stdout": proc.stdout,
        "stderr": proc.stderr,
        "last_message": message,
    }


def _attempt_error(result: dict) -> str | None:
    if result["returncode"] != 0:
        message = (result.get("stderr") or "").strip()[-500:]
        return message or f"codex exec exited {result['returncode']}"
    if not (result.get("last_message") or "").strip():
        return "codex exec wrote no final message"
    return None


def exec_judge(prompt: str) -> bool:
    work = Path(tempfile.mkdtemp(prefix="codex-rules-judge-"))
    try:
        proc = subprocess.run(
            judge_argv() + [prompt],
            cwd=work,
            capture_output=True,
            text=True,
            timeout=300,
            check=False,
        )
    finally:
        shutil.rmtree(work, ignore_errors=True)
    if proc.returncode != 0:
        detail = (proc.stderr or proc.stdout or "judge failed")[-500:]
        raise RuntimeError(detail)
    return passed_from_judge_output(proc.stdout)


def run_suite(
    cases: list,
    runs: int | None,
    output: Path,
    run_exec,
    judge,
    user_home: Path,
    auth_file: Path | None,
    api_key: str | None,
    plugin_src: Path = PLUGIN_SRC,
    keep_temp: bool = False,
) -> dict:
    output.mkdir(parents=True, exist_ok=True)
    rows = []
    for case in cases:
        arms = {"with": [], "without": []}
        for _ in range(runs or case["runs"]):
            for arm in ("with", "without"):
                root = Path(tempfile.mkdtemp(prefix="codex-rules-"))
                try:
                    built = build_arm(
                        root,
                        arm,
                        plugin_src=plugin_src,
                        user_home=user_home,
                        auth_file=auth_file,
                        api_key=api_key,
                    )
                    plan = exec_plan(built, case["prompt"], case["timeout_seconds"])
                    result = run_exec(plan)
                    error = _attempt_error(result)
                    if error:
                        arms[arm].append({"error": error, "graders": []})
                        continue
                    arms[arm].append(
                        grade_run(case["dir"], result["stdout"], result["last_message"], judge)
                    )
                except Exception as exc:
                    arms[arm].append({"error": str(exc), "graders": []})
                finally:
                    if not keep_temp:
                        shutil.rmtree(root, ignore_errors=True)
        rows.append({"name": case["name"], "arms": arms})
    report = aggregate_report(rows)
    (output / "aggregate-result.json").write_text(json.dumps(report, indent=2) + "\n")
    return report


def main(argv: list | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run", action="store_true", help="call Codex and the judge")
    parser.add_argument("--all", action="store_true", help="run every rules case")
    parser.add_argument("--tag", action="append", default=[])
    parser.add_argument("--case", help="case directory glob, such as rules-testing-*")
    parser.add_argument("--runs", type=int)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--auth-file", type=Path)
    parser.add_argument("--keep-temp", action="store_true")
    args = parser.parse_args(argv)
    selected = filter_cases(discover(), args.tag, args.case)
    print(format_preview(selected, args.runs), end="")
    if not args.run:
        return 0
    scoped = bool(args.tag or args.case or args.all)
    if not scoped:
        print("refusing an unscoped run; pass --tag, --case, or --all", file=sys.stderr)
        return 2
    if not selected:
        print("refusing: no cases matched", file=sys.stderr)
        return 2
    auth = args.auth_file
    if auth is None and os.environ.get("CODEX_AUTH_FILE"):
        auth = Path(os.environ["CODEX_AUTH_FILE"])
    api_key = os.environ.get("CODEX_API_KEY")
    if auth is None and not api_key:
        print(
            "refusing: set CODEX_AUTH_FILE or CODEX_API_KEY "
            "to the spare-budget Codex account",
            file=sys.stderr,
        )
        return 2
    if auth is not None:
        reject_user_path(auth, Path.home(), "CODEX_AUTH_FILE")
    if args.output is None:
        print("refusing: pass --output for aggregate-result.json", file=sys.stderr)
        return 2
    reject_output(args.output, Path.home(), _HERE.parent)
    report = run_suite(
        selected,
        args.runs,
        args.output,
        exec_codex,
        exec_judge,
        Path.home(),
        auth,
        api_key,
        keep_temp=args.keep_temp,
    )
    print(format_trigger(report))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
