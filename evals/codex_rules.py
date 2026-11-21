#!/usr/bin/env python3
"""Codex runner for the no-mistakes-rules cases.

    python3 evals/codex_rules.py
        Print cases x arms x runs and exit. This does not call a model.

    python3 evals/codex_rules.py --run --tag rules --runs 3 --output <dir>
        Run both arms. Requires CODEX_AUTH_FILE or CODEX_API_KEY for the
        spare-budget Codex account. Refuses ~/.codex and ~/.claude.

The default command is the preview. --run is what spends money. Each attempt
writes its JSONL transcript under --output before the next attempt starts.
Running the same command again skips attempts that already have result.json.
"""
import argparse
import fnmatch
import json
import os
import shutil
import subprocess
import sys
import tempfile
import threading
from pathlib import Path

import codex_rules_store as store
from codex_rules_env import (
    APPROVAL,
    MODEL,
    PLUGIN,
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


def run_count(runs: int | None, case: dict) -> int:
    count = case["runs"] if runs is None else runs
    if count < 1:
        raise SystemExit("refusing: --runs must be >= 1")
    return count


def format_preview(cases: list, runs: int | None) -> str:
    if runs is None:
        attempts = sum(case["runs"] * 2 for case in cases)
        values = {case["runs"] for case in cases}
        uniform = cases[0]["runs"] if cases and len(values) == 1 else None
    else:
        uniform = runs
        attempts = len(cases) * 2 * runs
    formula = f"{len(cases)} x 2 x {uniform}" if uniform is not None else "per-case runs x 2 arms"
    timeout = cases[0]["timeout_seconds"] if cases else 0
    shown = uniform if uniform is not None else "per case"
    return (
        "Codex rules eval preview (no model calls)\n"
        f"cases: {len(cases)}\n"
        "arms: 2 (with, without)\n"
        f"runs: {shown}\n"
        f"attempts: {attempts} ({formula})\n"
        f"agent model: {MODEL}\n"
        f"reasoning effort: {REASONING}\n"
        f"sandbox: {SANDBOX}\n"
        f"approval: {APPROVAL}\n"
        f"timeout_seconds: {timeout}\n"
        f"judge model: {JUDGE_MODEL}\n"
        "dollars: unknown (not measured)\n"
    )


def _pump(stream, parts: list, handle) -> None:
    try:
        for line in iter(stream.readline, ""):
            parts.append(line)
            if handle is not None:
                handle.write(line)
                handle.flush()
    finally:
        stream.close()


def _keep_partial(partial: Path) -> None:
    if not partial.exists():
        return
    number = 1
    while True:
        dest = partial.with_name(f"transcript.interrupted-{number}.jsonl")
        if not dest.exists():
            partial.replace(dest)
            return
        number += 1


def exec_codex(plan: dict) -> dict:
    """Run one exec. Stream stdout to plan['transcript'] when that path is set."""
    transcript = plan.get("transcript")
    if not transcript:
        return _exec_capture(plan, None)
    path = Path(transcript)
    path.parent.mkdir(parents=True, exist_ok=True)
    partial = path.with_name("transcript.jsonl.partial")
    _keep_partial(partial)
    return _exec_capture(plan, partial, path)


def _exec_capture(plan: dict, partial: Path | None, final: Path | None = None) -> dict:
    stdout_parts = []
    stderr_parts = []
    handle = partial.open("w") if partial is not None else None
    proc = None
    out = None
    err = None
    returncode = 1
    try:
        proc = subprocess.Popen(
            plan["argv"],
            env=plan["env"],
            cwd=plan["cwd"],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        out = threading.Thread(target=_pump, args=(proc.stdout, stdout_parts, handle))
        err = threading.Thread(target=_pump, args=(proc.stderr, stderr_parts, None))
        out.start()
        err.start()
        try:
            returncode = proc.wait(timeout=plan["timeout_seconds"])
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait()
            returncode = 124
        except KeyboardInterrupt:
            proc.kill()
            proc.wait()
            raise
        finally:
            if out is not None:
                out.join()
            if err is not None:
                err.join()
    except KeyboardInterrupt:
        if proc is not None and proc.poll() is None:
            proc.kill()
            proc.wait()
        raise
    finally:
        if handle is not None:
            handle.close()
    if partial is not None and final is not None and partial.exists():
        partial.replace(final)
    last = Path(plan["last_message"])
    message = last.read_text() if last.is_file() else ""
    return {
        "returncode": returncode,
        "stdout": "".join(stdout_parts),
        "stderr": "".join(stderr_parts),
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


def plugin_list_argv() -> list:
    return ["codex", "plugin", "list", "--json"]


def list_plugins(built: dict) -> dict:
    proc = subprocess.run(
        plugin_list_argv(),
        env=built["env"],
        cwd=built["workspace"],
        capture_output=True,
        text=True,
        timeout=60,
        check=False,
    )
    if proc.returncode != 0:
        detail = (proc.stderr or proc.stdout or "plugin list failed")[-500:]
        raise SystemExit(f"preflight: codex plugin list failed: {detail}")
    try:
        return json.loads(proc.stdout)
    except json.JSONDecodeError as exc:
        raise SystemExit(f"preflight: codex plugin list was not JSON: {exc}") from exc


def preflight_reason(with_list: dict, without_list: dict) -> str | None:
    installed = with_list.get("installed") or []
    if len(installed) != 1:
        return "with arm must list only no-mistakes-rules"
    item = installed[0]
    enabled = item.get("name") == PLUGIN and item.get("installed") is True and item.get("enabled") is True
    if not enabled:
        return "with arm plugin is not installed and enabled"
    if without_list.get("installed"):
        return "without arm must list no plugins"
    return None


def preflight(user_home: Path, auth_file, api_key, plugin_src: Path, list_fn) -> None:
    """Abort before a paid exec when the two homes do not differ by this plugin."""
    root = Path(tempfile.mkdtemp(prefix="codex-rules-preflight-"))
    try:
        homes = {}
        for arm in ("with", "without"):
            homes[arm] = build_arm(
                root / arm,
                arm,
                plugin_src=plugin_src,
                user_home=user_home,
                auth_file=auth_file,
                api_key=api_key,
            )
        reason = preflight_reason(list_fn(homes["with"]), list_fn(homes["without"]))
        if reason:
            raise SystemExit(f"preflight: {reason}")
    finally:
        shutil.rmtree(root, ignore_errors=True)


def _needs_exec(cases: list, runs: int | None, output: Path) -> bool:
    for case in cases:
        for index in range(1, run_count(runs, case) + 1):
            for arm in ("with", "without"):
                path = store.slot(output, case["name"], arm, index)
                if store.read_result(path) is None and store.read_exec(path) is None:
                    return True
    return False


def _grade_result(case: dict, result: dict, judge) -> dict:
    error = _attempt_error(result)
    if error:
        return {"error": error, "graders": []}
    plugin_root = result.get("plugin_root")
    root = Path(plugin_root) if plugin_root else None
    return grade_run(case["dir"], result["stdout"], result["last_message"], judge, root)


def _one_attempt(case, arm, index, output, run_exec, judge, user_home, auth_file, api_key, plugin_src, keep_temp):
    path = store.slot(output, case["name"], arm, index)
    done = store.read_result(path)
    if done is not None:
        return done
    finished = store.read_exec(path)
    if finished is None:
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
            plan["transcript"] = str(path / "transcript.jsonl")
            try:
                result = run_exec(plan)
            except Exception as exc:
                result = {
                    "returncode": 1,
                    "stdout": "",
                    "stderr": str(exc),
                    "last_message": "",
                }
            result["plugin_root"] = plan.get("plugin_root")
            store.save_exec(path, result)
            finished = result
            row = _grade_result(case, finished, judge)
        finally:
            if not keep_temp:
                shutil.rmtree(root, ignore_errors=True)
    else:
        row = _grade_result(case, finished, judge)
    row["transcript"] = store.transcript_rel(case["name"], arm, index)
    store.save_result(path, row)
    return row


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
    check_plugins: bool = True,
    list_fn=list_plugins,
) -> dict:
    if runs is not None and runs < 1:
        raise SystemExit("refusing: --runs must be >= 1")
    output.mkdir(parents=True, exist_ok=True)
    if check_plugins and _needs_exec(cases, runs, output):
        preflight(user_home, auth_file, api_key, plugin_src, list_fn)
    rows = []
    for case in cases:
        arms = {"with": [], "without": []}
        count = run_count(runs, case)
        for index in range(1, count + 1):
            for arm in ("with", "without"):
                arms[arm].append(
                    _one_attempt(
                        case,
                        arm,
                        index,
                        output,
                        run_exec,
                        judge,
                        user_home,
                        auth_file,
                        api_key,
                        plugin_src,
                        keep_temp,
                    )
                )
                store.write_aggregate(output, rows + [{"name": case["name"], "arms": arms}], partial=True)
        rows.append({"name": case["name"], "arms": arms})
    return store.write_aggregate(output, rows, partial=False)


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
    if args.runs is not None and args.runs < 1:
        print("refusing: --runs must be >= 1", file=sys.stderr)
        return 2
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
