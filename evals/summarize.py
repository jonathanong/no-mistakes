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

2.  **The should-fire aggregate is the `openai.yaml` gate.** It counts trigger
    across every case except the `-neg-` slugs and the `neg-hard` flow, which
    are the cases where firing is the failure. That is the single number
    standing for "the description carries Claude on its own".

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


def _fired(runs) -> int:
    return sum(
        1
        for r in runs
        for g in r.get("graders", [])
        if g["name"] == "skill-fired" and g.get("passed")
    )


def _mean_score(runs) -> float:
    """Mean weighted grader score per run, with `skill-fired` left out."""
    per_run = []
    for r in runs:
        num = den = 0.0
        for g in r.get("graders", []):
            if g["name"] == "skill-fired":
                continue
            weight = g.get("weight", 1)
            den += weight
            if g.get("passed"):
                num += weight
        if den:
            per_run.append(num / den)
    return sum(per_run) / len(per_run) if per_run else 0.0


def summarize(path: str) -> None:
    report = json.loads(pathlib.Path(path).read_text())
    suite = report.get("suite", {})
    plugins = ", ".join(p.get("path", "?") for p in suite.get("plugins", []))
    print(f"\n=== {path}")
    print(
        f"    ablation={suite.get('ablation')}  cost=${report.get('costUsd', 0):.2f}"
        f"  partial={report.get('partial')} {report.get('partialReason') or ''}"
    )
    print(f"    plugin: {plugins}")
    print(f"    {'case':<38} {'fired':>7} {'with':>6} {'without':>8} {'delta':>7}")

    fired_total = runs_total = 0
    for case in report["cases"]:
        with_arm = case["arms"].get("with", [])
        without_arm = case["arms"].get("without", [])
        fired = _fired(with_arm)
        with_score = _mean_score(with_arm)
        row = f"    {case['name']:<38} {fired:>3}/{len(with_arm):<3} {with_score:>6.2f}"
        if without_arm:
            without_score = _mean_score(without_arm)
            row += f" {without_score:>8.2f} {with_score - without_score:>+7.2f}"
        else:
            row += f" {'-':>8} {'-':>7}"
        print(row)
        if _is_should_fire(case["name"]):
            fired_total += fired
            runs_total += len(with_arm)

    if runs_total:
        pct = 100 * fired_total / runs_total
        print(
            f"    should-fire aggregate trigger: "
            f"{fired_total}/{runs_total} ({pct:.0f}%)"
        )


def _evidence(run) -> str:
    for g in run.get("graders", []):
        if g.get("evidence"):
            return g["evidence"]
    return ""


def _refers_as(text: str) -> str:
    if re.search(r"/no-mistakes\b", text):
        return "invented /no-mistakes form"
    if re.search(r"\bno-mistakes\s+[a-z-]+", text):
        return "real CLI command"
    if "no-mistakes" in text:
        return "named, no command"
    return "no mention"


def mentions(path: str) -> None:
    """Classify how the runs that did NOT fire refer to the tool."""
    report = json.loads(pathlib.Path(path).read_text())
    groups: dict = {}
    for case in report["cases"]:
        group = "should-fire" if _is_should_fire(case["name"]) else "negative"
        for run in case["arms"].get("with", []):
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


def main() -> None:
    paths = [a for a in sys.argv[1:] if a != "--mentions"]
    want_mentions = "--mentions" in sys.argv[1:]
    if not paths:
        raise SystemExit(__doc__)
    for path in paths:
        summarize(path)
        if want_mentions:
            mentions(path)


if __name__ == "__main__":
    main()
