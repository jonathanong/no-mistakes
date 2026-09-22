#!/usr/bin/env python3
"""How large an effect can a gate on this suite actually resolve?

    python3 evals/power.py evals/results/*/aggregate-result.json
    python3 evals/power.py --runs 9 <report.json>     # size a future gate
    python3 evals/power.py --subsample <deep-report.json>

Exists because #1025: the same description scored `signature` 9/12 and 7/12
hours apart, and every gate in `evals/README.md` was written in two-run units.
A gate is only meaningful when its margin exceeds the spread of the statistic
it reads, so that spread has to be a number rather than an assumption.

**The question a gate asks** is not "what is this description's rate" but
"are these two measurements different". Under the null they measure the *same*
description, so each case has one unknown rate `p_i` shared by both arms, and
a `runs: m` measurement of it is Binomial(m, p_i).

That sharing matters. Given `p`, the difference of two independent
measurements has variance `2 * m * sum_i p_i (1 - p_i)`. The `p_i` are unknown,
but they are the *same* unknown on both sides, so uncertainty about them
cancels out of the difference rather than adding to it. Averaging over the
Jeffreys posterior Beta(x+0.5, n-x+0.5) for each case:

    E[p (1-p)] = mu (1-mu) * s / (s + 1),    mu = a/(a+b),  s = a+b
    Var(X - Y) = 2 * m * sum_i E[p_i (1-p_i)]

(An earlier draft used the Beta-Binomial *predictive* variance of a single
measurement here. That is the right answer to a different question, and it is
wrong for this one: it charges parameter uncertainty to both arms without
letting it cancel, so the implied gap grows linearly in `m` and no amount of
extra runs ever closes it. The bug is worth naming because its output looked
like a finding — "no feasible `runs` fixes this" — rather than like a bug.)

**Reading the output.** `sd(diff)` is the spread of the difference between two
measurements of one description. `gap95 = 1.96 * sd(diff)` is the smallest
count difference that re-running the identical description does **not**
routinely produce. A gate margin below `gap95` cannot resolve what it gates
on. `as rate` restates it as a fraction of `n`, so flows of different sizes
are comparable.

`--subsample` skips the model entirely: given a report with many runs per case,
it resamples `runs: 3` measurements directly and reports the observed spread.
Use it to check the model rather than to replace it — it needs a deep run.
"""
import json
import math
import pathlib
import random
import sys

JEFFREYS = 0.5


def _is_should_fire(name: str) -> bool:
    return "-neg-" not in name and not name.startswith("neg-hard")


def _fired(runs) -> int:
    return sum(
        1
        for r in runs
        for g in r.get("graders", [])
        if g["name"] == "skill-fired" and g.get("passed")
    )


def _cases(report) -> list:
    """(name, fired, n) for should-fire cases with no errored with-arm run."""
    out = []
    for case in report["cases"]:
        arm = case["arms"].get("with", [])
        if not arm or any(r.get("error") for r in arm):
            continue
        if _is_should_fire(case["name"]):
            out.append((case["name"], _fired(arm), len(arm)))
    return out


def _e_p_q(x: int, n: int) -> float:
    """E[p (1-p)] for one case under its Jeffreys posterior."""
    a, b = x + JEFFREYS, n - x + JEFFREYS
    s = a + b
    mu = a / s
    return mu * (1 - mu) * s / (s + 1)


def _diff_sd(cases, m: int) -> float:
    """SD of the difference between two `runs: m` measurements of one flow."""
    return math.sqrt(2 * m * sum(_e_p_q(x, n) for _, x, n in cases))


def _flow(report, m: int) -> tuple:
    cases = _cases(report)
    if not cases:
        return None
    fired = sum(x for _, x, _ in cases)
    runs = sum(n for _, _, n in cases)
    return cases, fired, runs, _diff_sd(cases, m)


def analyse(path: str, m: int) -> None:
    report = json.loads(pathlib.Path(path).read_text())
    got = _flow(report, m)
    name = pathlib.Path(path).stem
    if not got:
        print(f"{name:<16} no clean should-fire cases")
        return
    cases, fired, runs, sd = got
    n_m = len(cases) * m
    gap = 1.96 * sd
    print(
        f"{name:<14}{f'{fired}/{runs}':>10}{len(cases):>7}{n_m:>5}"
        f"{sd:>10.2f}{gap:>8.1f}{gap / n_m:>9.0%}"
    )


def size(path: str, targets) -> None:
    """Smallest `runs` whose min-detectable gap is under each target rate."""
    report = json.loads(pathlib.Path(path).read_text())
    cases = _cases(report)
    if not cases:
        return
    print(f"\n=== {pathlib.Path(path).stem}: runs needed per case")
    print(f"    {'target':>8}{'runs':>7}{'n':>7}{'gap95':>10}{'cost':>9}")
    for t in targets:
        for m in range(3, 500):
            gap = 1.96 * _diff_sd(cases, m)
            if gap / (len(cases) * m) <= t:
                print(
                    f"    {t:>8.0%}{m:>7}{len(cases) * m:>7}"
                    f"{gap:>9.1f}{m / 3:>8.0f}x"
                )
                break
        else:
            print(f"    {t:>8.0%}   not reached below runs: 500")


def subsample(path: str, m: int = 3, trials: int = 20000) -> None:
    """Observed spread of a `runs: m` statistic, resampled from a deep run."""
    report = json.loads(pathlib.Path(path).read_text())
    cases = [
        [
            1 if _fired([r]) else 0
            for r in case["arms"].get("with", [])
        ]
        for case in report["cases"]
        if _is_should_fire(case["name"])
        and case["arms"].get("with")
        and not any(r.get("error") for r in case["arms"]["with"])
    ]
    depth = min(len(c) for c in cases)
    if depth < m * 2:
        print(f"    needs >= {m * 2} runs per case; deepest common is {depth}")
        return
    rng = random.Random(0)
    totals = []
    for _ in range(trials):
        totals.append(sum(sum(rng.sample(c, m)) for c in cases))
    mean = sum(totals) / len(totals)
    sd = math.sqrt(sum((t - mean) ** 2 for t in totals) / (len(totals) - 1))
    lo, hi = min(totals), max(totals)
    n_m = len(cases) * m
    print(f"\n=== {pathlib.Path(path).stem}: observed runs:{m} spread")
    print(f"    depth {depth} runs/case, {len(cases)} cases, resampled {trials}x")
    print(f"    mean {mean:.2f}/{n_m}   sd {sd:.2f}   range {lo}-{hi}")
    pairs = [
        abs(a - b)
        for a, b in zip(totals[::2], totals[1::2])
    ]
    print(
        f"    two runs of the SAME description differ by "
        f">= 2 in {sum(1 for d in pairs if d >= 2) / len(pairs):.0%} of pairs, "
        f">= 3 in {sum(1 for d in pairs if d >= 3) / len(pairs):.0%}"
    )


#: Known-answer checks. The scaling one is the regression guard: an earlier
#: draft used the single-measurement predictive variance here, which makes the
#: gap grow with `runs` so no run count ever resolves anything. If `gap95` as a
#: RATE stops shrinking when `runs` rises, that bug is back.
def _self_test() -> None:
    bad = []

    # E[p(1-p)] for 3/3 under Jeffreys: mu=0.875, s=4 -> .875*.125*4/5
    got = _e_p_q(3, 3)
    if abs(got - 0.0875) > 1e-9:
        bad.append(f"_e_p_q(3,3)={got!r}, want 0.0875")

    # A case seen 3/3 must still carry spread; p=1 would give zero.
    if _e_p_q(3, 3) <= 0:
        bad.append("a 3/3 case must not have zero variance")

    cases = [("a", 3, 3), ("b", 2, 3), ("c", 2, 3), ("d", 0, 3)]
    # sd(diff) grows as sqrt(m)...
    r = _diff_sd(cases, 12) / _diff_sd(cases, 3)
    if abs(r - 2.0) > 1e-9:
        bad.append(f"sd(diff) should double from runs 3->12, got {r:.3f}x")
    # ...so the same gap expressed as a RATE must halve. This is the bug guard.
    rate3 = 1.96 * _diff_sd(cases, 3) / (len(cases) * 3)
    rate12 = 1.96 * _diff_sd(cases, 12) / (len(cases) * 12)
    if not rate12 < rate3 / 1.99:
        bad.append(f"rate must shrink ~1/sqrt(m): {rate3:.4f} -> {rate12:.4f}")

    for line in bad:
        print(f"    FAIL {line}")
    if bad:
        raise SystemExit(f"{len(bad)} power-model checks failed")
    print("power self-test: 4/4 pass")


def main() -> None:
    args = sys.argv[1:]
    if not args:
        raise SystemExit(__doc__)
    if "--self-test" in args:
        _self_test()
        return
    if "--subsample" in args:
        for p in (a for a in args if a != "--subsample"):
            subsample(p)
        return
    m = 3
    if "--runs" in args:
        i = args.index("--runs")
        m = int(args[i + 1])
        args = args[:i] + args[i + 2:]
    print(
        f"spread between two runs:{m} measurements of the SAME description\n"
    )
    print(
        f"{'flow':<14}{'observed':>10}{'cases':>7}{'n':>5}"
        f"{'sd(diff)':>10}{'gap95':>8}{'as rate':>9}"
    )
    for p in args:
        analyse(p, m)
    print(
        "\ngap95 = 1.96 * sd(diff): the smallest count difference between two\n"
        "measurements that re-running ONE description does NOT routinely\n"
        "produce. A gate margin below it cannot resolve what it gates on."
    )
    for p in args:
        size(p, (0.20, 0.15, 0.10))


if __name__ == "__main__":
    main()
