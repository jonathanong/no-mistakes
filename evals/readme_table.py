#!/usr/bin/env python3
"""Print the README results tables from one run's reports.

    python3 evals/readme_table.py rules  <topic.json>...
    python3 evals/readme_table.py skill  <flow.json>...

`rules` prints the per-topic and per-case tables: loaded, passed with the
plugin, passed when loaded, and passed without it, each with its count.
`skill` prints one row per flow: should-fire trigger, should-fire Δ, the part
of it that content graders carry (`names-graph-command` left out), and the
negatives' Δ. The output replaces the README's results tables whole; do not
edit its numbers by hand. Notes on single cases go in prose under the table,
not in an extra column.
"""
import json
import math
import pathlib
import sys

from summarize import USAGE_GRADERS, _errors, _fired, _is_should_fire, _mean_score


def wilson(k: int, n: int) -> str:
    if n == 0:
        return "n/a"
    z = 1.96
    p = k / n
    d = 1 + z * z / n
    c = (p + z * z / (2 * n)) / d
    h = z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n)) / d
    return f"{round(100 * max(0.0, c - h))}-{round(100 * min(1.0, c + h))}%"


def _passed(run) -> bool:
    graders = [g for g in run.get("graders", []) if g["name"] != "skill-fired"]
    return bool(graders) and all(g.get("passed") for g in graders)


def _loaded(run) -> bool:
    return _fired([run]) == 1


def _load(paths):
    cases = []
    for path in paths:
        report = json.loads(pathlib.Path(path).read_text())
        if report.get("partial"):
            raise SystemExit(f"{path}: partial run, not a result")
        for case in report["cases"]:
            if any(_errors(runs) for runs in case["arms"].values()):
                raise SystemExit(f"{path}: {case['name']} has an errored run")
            cases.append(case)
    return cases


def _count(runs, pred) -> int:
    return sum(1 for r in runs if pred(r))


def rules(paths) -> None:
    cases = _load(paths)
    topics = {}
    rows = []
    for case in cases:
        name = case["name"].removeprefix("rules-")
        topic = "off-topic" if name.startswith("offtopic-") else name.split("-write-")[0]
        topic = topic.split("-review-")[0].split("-exception-")[0]
        w = case["arms"].get("with", [])
        wo = case["arms"].get("without", [])
        loaded = [r for r in w if _loaded(r)]
        t = topics.setdefault(topic, [0, 0, 0, 0, 0, 0, 0])
        for i, v in enumerate(
            (len(w), _count(w, _loaded), _count(w, _passed), len(loaded),
             _count(loaded, _passed), len(wo), _count(wo, _passed))
        ):
            t[i] += v
        rows.append(
            f"| `{name}` | {len(loaded)}/{len(w)} | {_count(w, _passed)}/{len(w)} | "
            f"{_count(loaded, _passed)}/{len(loaded)} | {_count(wo, _passed)}/{len(wo)} |"
        )
    print("| topic | loaded (with) | passed overall (with) | passed when loaded | passed (without plugin) |")
    print("| --- | --- | --- | --- | --- |")
    for topic, (n, ld, ps, nl, pl, nwo, pwo) in sorted(topics.items()):
        print(
            f"| `{topic}` | {ld}/{n} ({wilson(ld, n)}) | {ps}/{n} ({wilson(ps, n)}) | "
            f"{pl}/{nl} | {pwo}/{nwo} ({wilson(pwo, nwo)}) |"
        )
    print()
    print("| case | loaded | passed (with) | passed when loaded | passed (without) |")
    print("| --- | --- | --- | --- | --- |")
    print("\n".join(sorted(rows)))


def _delta(cases, skip) -> str:
    deltas = [
        _mean_score(c["arms"]["with"], skip) - _mean_score(c["arms"]["without"], skip)
        for c in cases
        if c["arms"].get("with") and c["arms"].get("without")
    ]
    return f"{sum(deltas) / len(deltas):+.2f}" if deltas else "—"


def skill(paths) -> None:
    print("| flow | trigger (with-arm) | should-fire Δ | content-only Δ | negatives Δ |")
    print("| --- | --- | --- | --- | --- |")
    for path in paths:
        cases = _load([path])
        tags = json.loads(pathlib.Path(path).read_text())["suite"].get("tagFilters") or ["?"]
        fire = [c for c in cases if _is_should_fire(c["name"])]
        neg = [c for c in cases if not _is_should_fire(c["name"])]
        if not fire:
            fired = sum(_fired(c["arms"]["with"]) for c in neg)
            n = sum(len(c["arms"]["with"]) for c in neg)
            print(f"| `{tags[0]}` (guard) | {fired}/{n} | n/a | n/a | {_delta(neg, ('skill-fired',))} |")
            continue
        fired = sum(_fired(c["arms"]["with"]) for c in fire)
        n = sum(len(c["arms"]["with"]) for c in fire)
        print(
            f"| `{tags[0]}` | {fired}/{n} ({wilson(fired, n)}) | {_delta(fire, ('skill-fired',))} | "
            f"{_delta(fire, USAGE_GRADERS)} | {_delta(neg, ('skill-fired',))} |"
        )


if __name__ == "__main__":
    if len(sys.argv) < 3 or sys.argv[1] not in ("rules", "skill"):
        raise SystemExit(__doc__)
    {"rules": rules, "skill": skill}[sys.argv[1]](sys.argv[2:])
