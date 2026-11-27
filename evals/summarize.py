#!/usr/bin/env python3
"""Summarize one or more eval runs.

    python3 evals/summarize.py evals/results/*/aggregate-result.json

Reads the `aggregate-result.json` a run writes (or the file `--json <path>`
produced) and prints, per case: how many runs fired the skill, the mean score
per arm, and Δ where both arms exist.

Two things it does that reading the HTML report does not:

1.  **Scores exclude `skill-fired`.** Under `--ablation with-without` the runner
    already treats that grader as a display-only indicator, but under
    `--ablation none` it is scored — which silently makes single-arm scores
    incomparable with two-arm ones. Dropping it everywhere keeps every number
    in this file on one scale.

2.  **A should-fire aggregate.** It counts trigger across every case except the
    `-neg-` slugs and the `neg-hard` flow, which are the cases where firing is
    the failure. That is the single number standing for "the description
    carries the agent on its own". Keep reports grouped by runner because
    Claude and Codex use different agents and trigger checks.

It also lists every case whose plugin arm scores below its baseline arm. The
plugin must never make an answer worse, so `with < without` is a defect to
explain even when it is within run-to-run noise, and the script exits 1 when
one exists. Each row says how many with-arm runs fired the skill: an unfired
loss is description or grader noise, a fired one is the skill's content.

It also refuses to let a failed run pass as a result. A run that errors — a
session limit, a timeout — scores 0 in **both** arms, so the case renders as a
tidy `Δ +0.00` that is indistinguishable from "the skill made no difference".
The header counts errored runs, names the cause, and drops those cases from the
aggregate.

`--mentions` adds a second table classifying how the runs that did NOT fire
refer to the tool. `skill-fired` is binary and hides the difference between a
run that never reached for the skill and one that reached for it and invented a
command form that does not exist — and on the `neg-hard` guard, between a
description that stays silent and one that leaks into questions the graph
cannot answer while still scoring 0 fired.
"""
import json
import pathlib
import re
import sys


def _is_should_fire(name: str) -> bool:
    return "-neg-" not in name and not name.startswith("neg-hard")


def _errors(runs) -> list:
    errors = []
    for run in runs:
        if run.get("error"):
            errors.append(run["error"])
        elif run.get("isolationFindings"):
            errors.append("attempt flagged by the isolation audit")
        elif run.get("judgeWithheld"):
            errors.append("attempt awaiting a judge verdict")
        else:
            grader_errors = [grader["error"] for grader in run.get("graders", []) if "error" in grader]
            if grader_errors:
                errors.append("judge error: " + ", ".join(grader_errors))
    return errors


def _fired(runs) -> int:
    return sum(
        1
        for r in runs
        for g in r.get("graders", [])
        if g["name"] == "skill-fired" and g.get("passed")
    )


#: Graders that record which tool the answer used rather than how good it is.
#: `content` scores leave them out so a gain from naming a command shows apart.
USAGE_GRADERS = ("skill-fired", "names-graph-command")


def _mean_score(runs, skip=("skill-fired",)) -> float:
    """Mean weighted grader score per run, with `skip` graders left out."""
    per_run = []
    for r in runs:
        num = den = 0.0
        for g in r.get("graders", []):
            if g["name"] in skip:
                continue
            weight = g.get("weight", 1)
            den += weight
            if g.get("passed"):
                num += weight
        if den:
            per_run.append(num / den)
    return sum(per_run) / len(per_run) if per_run else 0.0


def _regressions(report) -> list:
    """`(name, with, without, fired, runs)` for each case scoring below baseline.

    Cases with an errored run are skipped; the table already marks them FAILED.
    """
    rows = []
    for case in report["cases"]:
        with_arm = case["arms"].get("with", [])
        without_arm = case["arms"].get("without", [])
        if not with_arm or not without_arm or _errors(with_arm) or _errors(without_arm):
            continue
        with_score, without_score = _mean_score(with_arm), _mean_score(without_arm)
        if with_score < without_score:
            rows.append(
                (case["name"], with_score, without_score, _fired(with_arm), len(with_arm))
            )
    return rows


def summarize(path: str) -> bool:
    """Print the table; return True when some case scored below its baseline."""
    report = json.loads(pathlib.Path(path).read_text())
    suite = report.get("suite", {})
    plugins = ", ".join(p.get("path", "?") for p in suite.get("plugins", []))
    judge_cost = sum(
        r.get("judgeCostUsd") or 0
        for case in report["cases"]
        for runs in case["arms"].values()
        for r in runs
    )
    print(f"\n=== {path}")
    print(
        f"    ablation={suite.get('ablation')}  agent=${report.get('costUsd', 0):.2f}"
        f"  judge=${judge_cost:.2f}"
        f"  partial={report.get('partial')} {report.get('partialReason') or ''}"
    )
    print(f"    plugin: {plugins}")
    errors = [
        e
        for case in report["cases"]
        for runs in case["arms"].values()
        for e in _errors(runs)
    ]
    if errors:
        seen = {}
        for e in errors:
            seen[str(e)[:100]] = seen.get(str(e)[:100], 0) + 1
        print()
        print(f"    !!  {len(errors)} RUNS FAILED — these numbers are NOT a result.")
        for message, count in sorted(seen.items(), key=lambda kv: -kv[1]):
            print(f"    !!  {count:>4}x {message}")
        print("    !!  The affected cases have no numeric content score below.")
        print()

    partial = bool(report.get("partial"))
    if partial:
        print(f"    !!  PARTIAL RUN — {report.get('partialReason') or 'no reason given'}")
        print("    !!  Per-case scores are shown where available. The aggregate is suppressed.")
        print()

    print(
        f"    {'case':<38} {'fired':>7} {'with':>6} {'without':>8} {'delta':>7}"
        f" {'content':>8}"
    )

    fired_total = runs_total = 0
    for case in report["cases"]:
        with_arm = case["arms"].get("with", [])
        without_arm = case["arms"].get("without", [])
        with_errors = _errors(with_arm)
        case_errors = with_errors + _errors(without_arm)
        if case_errors:
            # Never render a numeric row for a case with a failed run: the
            # failed run scores 0 in both arms, so the row prints as an
            # ordinary low score with a clean delta instead of missing data.
            n = len(case_errors)
            print(
                f"    {case['name']:<38} {'--':>7} "
                f"{('%d run(s) FAILED' % n):>25}"
            )
            # Trigger is defined on the with arm alone, so a baseline-only
            # failure invalidates the score and the delta but not the trigger
            # observations. Dropping them too would thin the headline rate on
            # whichever cases happened to lose a baseline run.
            if not with_errors and _is_should_fire(case["name"]):
                fired_total += _fired(with_arm)
                runs_total += len(with_arm)
            continue
        fired = _fired(with_arm)
        with_score = _mean_score(with_arm)
        row = f"    {case['name']:<38} {fired:>3}/{len(with_arm):<3} {with_score:>6.2f}"
        if without_arm:
            without_score = _mean_score(without_arm)
            content = _mean_score(with_arm, USAGE_GRADERS) - _mean_score(
                without_arm, USAGE_GRADERS
            )
            row += (
                f" {without_score:>8.2f} {with_score - without_score:>+7.2f}"
                f" {content:>+8.2f}"
            )
        else:
            row += f" {'-':>8} {'-':>7}"
        print(row)
        if _is_should_fire(case["name"]):
            fired_total += fired
            runs_total += len(with_arm)

    if partial:
        print(
            "    should-fire aggregate trigger: SUPPRESSED (partial run)"
        )
    elif runs_total:
        pct = 100 * fired_total / runs_total
        suffix = "  (errored runs excluded)" if errors else ""
        print(
            f"    should-fire aggregate trigger: "
            f"{fired_total}/{runs_total} ({pct:.0f}%){suffix}"
        )

    regressions = _regressions(report)
    for name, with_score, without_score, fired, runs in regressions:
        print(
            f"    !!  REGRESSION {name}: with {with_score:.2f} < without "
            f"{without_score:.2f} (fired {fired}/{runs})"
        )
    return bool(regressions)


def _evidence(run) -> str:
    """Every grader's evidence, joined.

    A grader's `evidence` is the run's final response text, so taking only the
    first non-empty one drops nothing when the graders agree — but a case whose
    graders captured different turns would be classified off a partial view.
    Joining is safe precisely because evidence is model output, never rubric
    text: concatenating cannot invent a command the model did not write.
    """
    return "\n".join(g["evidence"] for g in run.get("graders", []) if g.get("evidence"))


#: `[`tests plan`](tests-plan.md)` — the docs write each command's real
#: invocation path as the link text, so the path is read from there rather than
#: from the filename. `docs_coverage.rs` keeps this index in sync with the clap
#: enums, so the set cannot drift from the CLI.
_DOC_LINK = re.compile(r"\[`([a-z0-9][a-z0-9 -]*)`\]\([a-z0-9-]+\.md\)")

#: "`no-mistakes test` is an alias for `no-mistakes tests`." — clap declares the
#: alias (`#[command(alias = "test")]`) and the docs state it in this fixed form,
#: so an aliased invocation is real and must not read as a fabrication.
_DOC_ALIAS = re.compile(
    r"`no-mistakes ([a-z0-9][a-z0-9 -]*)` is an alias for `no-mistakes ([a-z0-9][a-z0-9 -]*)`"
)


def _load_commands() -> tuple:
    """Every complete `no-mistakes` invocation path, from `docs/cli/`.

    Two earlier attempts were wrong in opposite directions. A hand-listed tuple
    held 20 of the real commands, so `no-mistakes lockfile` scored as a
    fabrication. Filename stems then over-corrected: they treat `graph.md` and
    `diagnostics.md` as commands when both are concept pages, and they accept
    `no-mistakes tests-plan` for a command that is really `tests plan`.

    So: take the link text, which is the invocation path; drop a hyphenated
    spelling whenever the spaced path exists (`README.md` writes `ci-env` in
    prose, `ci.md` writes the real `ci env`); and drop bare group names, since
    `no-mistakes tests` without a subcommand is not a runnable command.
    """
    cli_docs = pathlib.Path(__file__).resolve().parent.parent / "docs" / "cli"
    texts = [p.read_text() for p in cli_docs.glob("*.md")]
    paths = {m.group(1) for t in texts for m in _DOC_LINK.finditer(t)}
    paths -= {p.replace(" ", "-") for p in paths if " " in p}
    groups = {p.split()[0] for p in paths if " " in p}
    paths -= groups
    # `no-mistakes test plan` is as real as `tests plan`; mirror every path that
    # starts with an aliased name onto the alias.
    for text in texts:
        for alias, canonical in _DOC_ALIAS.findall(text):
            paths |= {
                f"{alias}{p[len(canonical):]}"
                for p in paths
                if p == canonical or p.startswith(f"{canonical} ")
            }
    if not paths:
        # An empty set would classify every real command as invented and quietly
        # turn the fabrication tables into noise. Fail instead.
        raise SystemExit(f"summarize: no CLI commands found under {cli_docs}")
    # Longest first, so `tests plan` is preferred over any shorter prefix.
    return tuple(sorted(paths, key=lambda n: (-len(n), n)))


COMMANDS = _load_commands()

#: `(?![-\w])` rather than `\b`, so `no-mistakes tests-plan` does not match the
#: real `tests` — a hyphen is a word boundary, which is what made stems wrong.
_REAL_COMMAND = re.compile(
    r"\bno-mistakes\s+(?:%s)(?![-\w])" % "|".join(re.escape(s) for s in COMMANDS)
)


#: The slash INVOCATION form (`/no-mistakes check`), which does not exist as a
#: command. The leading boundary is the whole point: `crates/no-mistakes`,
#: `packages/no-mistakes` and `github.com/jonathanong/no-mistakes` are ordinary
#: references to this repository and appear in most responses. Matching every
#: `/no-mistakes` scored any run that merely named a path as a fabrication, and
#: because this check runs BEFORE the real-command one it also hid genuine
#: invocations sitting in the same response. Require the slash to start a token.
_SLASH_COMMAND = re.compile(r"(?:^|[\s`'\"(\[])/no-mistakes(?![-\w])")


def _refers_as(text: str) -> str:
    # Slash form first. `\b` in `_REAL_COMMAND` also matches between `/` and `no`,
    # so testing real commands first would score the nonexistent
    # `/no-mistakes check` as a real CLI command and hide it from the tables.
    if _SLASH_COMMAND.search(text):
        return "invented command form"
    if _REAL_COMMAND.search(text):
        return "real CLI command"
    # `(?<![/\w-])` for the same reason as `_SLASH_COMMAND`: after a path
    # separator this is a repo reference, and the next prose word is not a
    # subcommand -- `packages/no-mistakes for the binding` is not a fabricated
    # `no-mistakes for`.
    if re.search(r"(?<![/\w-])no-mistakes\s+[A-Za-z_][A-Za-z0-9_]*", text):
        return "invented command form"
    if "no-mistakes" in text:
        return "named, no command"
    return "no mention"


def mentions(path: str) -> None:
    """Classify how the runs that did NOT fire refer to the tool."""
    report = json.loads(pathlib.Path(path).read_text())
    if report.get("partial"):
        # Same reasoning as the suppressed trigger aggregate: these buckets are
        # comparable across descriptions only if they cover the same cases, and
        # a partial run covers whichever prefix happened to finish.
        print("    non-firing runs: SUPPRESSED (partial run)")
        return
    groups: dict = {}
    for case in report["cases"]:
        group = "should-fire" if _is_should_fire(case["name"]) else "negative"
        for run in case["arms"].get("with", []):
            if run.get("error"):
                # An errored run has no response to classify; counting it as
                # "no mention" would read a session limit as model silence —
                # the same misleading-zero the numeric table refuses to print.
                continue
            flags = {g["name"]: g.get("passed") for g in run.get("graders", [])}
            if "skill-fired" not in flags or flags["skill-fired"]:
                continue
            bucket = groups.setdefault(group, {})
            kind = _refers_as(_evidence(run))
            bucket[kind] = bucket.get(kind, 0) + 1
    print("    non-firing runs, by how they refer to the tool:")
    for group, bucket in sorted(groups.items()):
        total = sum(bucket.values())
        print(f"      [{group}] {total} runs")
        for kind, n in sorted(bucket.items(), key=lambda kv: -kv[1]):
            print(f"        {kind:<26} {n}")


#: Regression cases for `_refers_as`. The repo-path rows are the ones that
#: matter: a response naming `crates/no-mistakes` used to be filed as a
#: fabrication, which inflates the very fabrication count a shipping gate reads.
_REFERS_AS_CASES = (
    ("/no-mistakes check", "invented command form"),
    ("run `/no-mistakes roleHas` next", "invented command form"),
    ("no-mistakes check", "real CLI command"),
    ("no-mistakes tests plan vitest", "real CLI command"),
    # A path plus a genuine invocation: the path must not win.
    ("edit crates/no-mistakes/src/lib.rs then run no-mistakes check",
     "real CLI command"),
    ("see packages/no-mistakes for the binding", "named, no command"),
    ("https://github.com/jonathanong/no-mistakes", "named, no command"),
    ("no-mistakes roleHas", "invented command form"),
    ("just grep for it", "no mention"),
)


def _run(score: float, fired: bool) -> dict:
    return {
        "graders": [
            {"name": "skill-fired", "passed": fired},
            {"name": "answer", "passed": score >= 1},
        ]
    }


#: `(arms, want)` regression cases. Ties and wins never flag; an errored run
#: never flags, since its 0 is missing data rather than a loss.
_REGRESSION_CASES = (
    ({"with": [_run(1, True)], "without": [_run(1, False)]}, []),
    ({"with": [_run(1, False)], "without": [_run(0, False)]}, []),
    ({"with": [_run(0, False)], "without": [_run(1, False)]}, [("c", 0.0, 1.0, 0, 1)]),
    ({"with": [{"error": "timeout", "graders": []}], "without": [_run(1, False)]}, []),
    ({"with": [_run(0, False)]}, []),
)


def _self_test() -> None:
    for arms, want in _REGRESSION_CASES:
        got = _regressions({"cases": [{"name": "c", "arms": arms}]})
        if got != want:
            raise SystemExit(f"regression case failed: {arms!r}: want {want!r}, got {got!r}")
    print(f"regression self-test: {len(_REGRESSION_CASES)}/{len(_REGRESSION_CASES)} pass")
    bad = [
        (text, want, got)
        for text, want in _REFERS_AS_CASES
        if (got := _refers_as(text)) != want
    ]
    for text, want, got in bad:
        print(f"    FAIL {text!r}\n         want {want!r}, got {got!r}")
    if bad:
        raise SystemExit(f"{len(bad)}/{len(_REFERS_AS_CASES)} classifier cases failed")
    print(f"classifier self-test: {len(_REFERS_AS_CASES)}/{len(_REFERS_AS_CASES)} pass")


def main() -> None:
    if "--self-test" in sys.argv[1:]:
        _self_test()
        return
    paths = [a for a in sys.argv[1:] if a != "--mentions"]
    want_mentions = "--mentions" in sys.argv[1:]
    if not paths:
        raise SystemExit(__doc__)
    regressed = False
    for path in paths:
        regressed |= summarize(path)
        if want_mentions:
            mentions(path)
    if regressed:
        raise SystemExit("summarize: the plugin arm scored below baseline; see REGRESSION rows")


if __name__ == "__main__":
    main()
