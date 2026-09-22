#!/usr/bin/env python3
"""How large an effect can a gate on this suite actually resolve?

    python3 evals/power.py evals/results/*/aggregate-result.json
    python3 evals/power.py --runs 9 <report.json>     # spread at other depths
    python3 evals/power.py --subsample <deep-report.json>

Exists because #1025: the same description scored `signature` 9/12 and 7/12
hours apart, and every gate in `evals/README.md` was written in two-run units.
A gate only means something when its margin exceeds the spread of the statistic
it reads, so that spread has to be a number rather than an assumption.

**The question a gate asks** is not "what is this description's rate" but "are
these two measurements different". Under the null both measure the *same*
description, so each case has one unknown rate `p_i` shared by both arms. That
sharing matters: uncertainty about `p_i` cancels out of the difference instead
of adding to it. Each case carries a Jeffreys posterior Beta(x+0.5, n-x+0.5),
and the null distribution of `X - Y` is simulated by drawing `p_i` from it once
and then two independent Binomial(m, p_i) draws from it.

**`margin` is a discrete quantile, not `1.96 * sd`.** These are small bounded
counts and the normal approximation is anti-conservative on them: on the deep
`signature` run `1.96 * sd = 3.9` rounds to a margin of 4, but
`P(|X-Y| >= 4) = 7.1%` — a gate at 4 rejects a correct description 7% of the
time, not 5%. `margin` is the smallest integer whose exceedance is genuinely
<= ALPHA.

**Sizing targets power, not just the critical value.** A gate whose critical
gap equals the effect you care about catches that effect about half the time.
`runs` is chosen so a shift of the target size is caught with POWER
probability, which roughly doubles what a critical-value-only calculation
gives. The alternative modelled is a uniform absolute shift of every case's
`p_i`, clipped at zero -- the `real` column reports the mean drop that
actually lands, which is below the target on flows with cases already near
zero. An effect concentrated in one case needs more runs than this reports.

**Negative flows are sized too.** `neg-hard` is a gated flow, so it gets its
own row: the event counted is *firing*, which there is the failure. The
difference calculation does not care which direction is the good one.
"""
import json
import math
import pathlib
import random
import sys

JEFFREYS = 0.5
ALPHA = 0.05
POWER = 0.80
TRIALS = 60_000
SEED = 0
#: Only used to pick a starting `runs` before the simulation checks it:
#: z(1 - ALPHA/2) and z(POWER).
Z_ALPHA, Z_POWER = 1.959964, 0.841621
GENERIC_STEMS = {"aggregate-result", "result", "results"}


def _label(path: str) -> str:
    """`evals/results/<run>/aggregate-result.json` -> `<run>`.

    The documented invocation globs `*/aggregate-result.json`, so every input
    shares a stem and every row would otherwise carry the same name.
    """
    p = pathlib.Path(path)
    if p.stem in GENERIC_STEMS and p.parent.name:
        return p.parent.name
    return p.stem


def _is_negative(name: str) -> bool:
    return "-neg-" in name or name.startswith("neg-hard")


def _fired(runs) -> int:
    return sum(
        1
        for r in runs
        for g in r.get("graders", [])
        if g["name"] == "skill-fired" and g.get("passed")
    )


def _reject(report, path: str) -> bool:
    """Refuse anything that is not a complete, clean measurement.

    A partial report is an order-dependent prefix of the suite; a report with
    errored runs is a thinned one. Sizing either silently changes the case set
    the margin describes, so a control and a candidate stop being comparable —
    which is the entire point of computing the margin.
    """
    name = _label(path)
    if report.get("partial"):
        why = report.get("partialReason") or "no reason given"
        print(f"{name:<14} PARTIAL RUN ({why}) — not sized")
        return True
    errored = [
        c["name"]
        for c in report["cases"]
        for arm in c["arms"].values()
        if any(r.get("error") for r in arm)
    ]
    if errored:
        print(
            f"{name:<14} {len(errored)} case(s) with errored runs "
            f"({errored[0]}) — not sized"
        )
        return True
    return False


def _group(report, negative: bool) -> list:
    """(name, fired, n) for one direction's cases, read from the with-arm."""
    return [
        (c["name"], _fired(c["arms"]["with"]), len(c["arms"]["with"]))
        for c in report["cases"]
        if c["arms"].get("with") and _is_negative(c["name"]) == negative
    ]


def _e_p_q(x: int, n: int) -> float:
    """E[p (1-p)] for one case under its Jeffreys posterior."""
    a, b = x + JEFFREYS, n - x + JEFFREYS
    s = a + b
    mu = a / s
    return mu * (1 - mu) * s / (s + 1)


def _diff_sd(cases, m: int) -> float:
    """Normal-approximation SD of the difference of two `runs: m` measures."""
    return math.sqrt(2 * m * sum(_e_p_q(x, n) for _, x, n in cases))


def _draw(cases, m: int, rng, shift: float = 0.0, up: bool = False) -> int:
    """One X - Y draw. `shift` moves the candidate arm's rate.

    Direction matters and is not cosmetic. On should-fire cases the
    regression is firing *less*, so the candidate shifts down. On an
    over-trigger guard the regression is firing *more*, and those cases sit
    at 0 — shifting them down is a no-op, which made an earlier revision
    report that `before-edit [negatives]` needed 137x the runs to detect a
    10% effect. It needs 2x; the effect was simply being clipped away.
    """
    total = 0
    for _, x, n in cases:
        p = rng.betavariate(x + JEFFREYS, n - x + JEFFREYS)
        q = min(1.0, max(0.0, p + shift if up else p - shift))
        total += rng.binomialvariate(m, p) - rng.binomialvariate(m, q)
    return total


def _null_margin(cases, m: int, alpha=ALPHA, trials=TRIALS) -> tuple:
    """Smallest integer margin whose null exceedance is <= alpha."""
    rng = random.Random(SEED)
    draws = [abs(_draw(cases, m, rng)) for _ in range(trials)]
    for d in range(1, len(cases) * m + 2):
        tail = sum(1 for v in draws if v >= d) / trials
        if tail <= alpha:
            return d, tail
    return len(cases) * m + 1, 0.0


def _power_at(cases, m, shift, margin, trials=TRIALS, seed=SEED + 1,
              up=False) -> float:
    rng = random.Random(seed)
    hit = sum(
        1
        for _ in range(trials)
        if abs(_draw(cases, m, rng, shift, up)) >= margin
    )
    return hit / trials


def _achieved(cases, shift: float, up: bool = False, trials=20_000) -> float:
    """The mean per-case rate drop a nominal `shift` actually produces.

    `_draw` clips the candidate rate at 0, so on a flow with cases already
    near zero -- `signature-04` at 1/15, the three `queues` topology cases at
    0/3 -- a nominal 10% shift moves those cases by less than 10% and the
    real effect is smaller than the target. Reporting the achieved shift
    keeps the sizing table from quietly understating what it costs to detect
    a genuine 10%.
    """
    rng = random.Random(SEED + 2)
    total = 0.0
    per = trials // len(cases)
    for _, x, n in cases:
        for _ in range(per):
            p = rng.betavariate(x + JEFFREYS, n - x + JEFFREYS)
            total += min(1.0 - p, shift) if up else min(p, shift)
    return total / (len(cases) * per)


def analyse(path: str, m: int) -> None:
    report = json.loads(pathlib.Path(path).read_text())
    if _reject(report, path):
        return
    name = _label(path)
    for negative in (False, True):
        cases = _group(report, negative)
        if not cases:
            continue
        fired = sum(x for _, x, _ in cases)
        runs = sum(n for _, _, n in cases)
        margin, tail = _null_margin(cases, m)
        n_m = len(cases) * m
        print(
            f"{name:<14}{'neg' if negative else 'fire':>5}"
            f"{f'{fired}/{runs}':>9}{len(cases):>6}{n_m:>5}"
            f"{_diff_sd(cases, m):>8.2f}{margin:>8}{tail:>8.1%}"
            f"{margin / n_m:>9.0%}"
        )


def size(path: str, targets) -> None:
    """`runs` needed to catch a target-sized shift with POWER probability."""
    report = json.loads(pathlib.Path(path).read_text())
    if _reject(report, path):
        return
    for negative in (False, True):
        cases = _group(report, negative)
        if not cases:
            continue
        kind = "negatives" if negative else "should-fire"
        print(f"\n=== {_label(path)} [{kind}]: runs for {POWER:.0%} power")
        print(
            f"    {'target':>8}{'real':>7}{'runs':>7}{'n':>7}"
            f"{'margin':>9}{'power':>8}{'cost':>8}"
        )
        s = sum(_e_p_q(x, n) for _, x, n in cases)
        for t in targets:
            # The normal approximation picks a starting point; the simulation
            # decides. Sizing on the critical value alone would halve this.
            m0 = max(3, math.ceil(
                2 * s * (Z_ALPHA + Z_POWER) ** 2 / (t * len(cases)) ** 2
            ))
            cap = min(m0 * 8, 800)
            hit = None
            for m in range(m0, cap):
                margin, _ = _null_margin(cases, m, trials=15_000)
                if _power_at(
                    cases, m, t, margin, trials=15_000, up=negative
                ) >= POWER:
                    hit = (m, margin)
                    break
            if hit is None:
                print(f"    {t:>8.0%}   not reached below runs: {cap}")
                continue
            # The scan runs at 15k trials, so its crossing point carries Monte
            # Carlo noise. Re-check at full depth on an independent seed and
            # step up rather than print an m that only just cleared by luck.
            m, margin = hit
            while m < cap:
                margin, _ = _null_margin(cases, m)
                got = _power_at(cases, m, t, margin, seed=SEED + 7,
                                up=negative)
                if got >= POWER:
                    break
                m += 1
            print(
                f"    {t:>8.0%}{_achieved(cases, t, negative):>7.0%}{m:>7}"
                f"{len(cases) * m:>7}{margin:>9}{got:>8.0%}{m / 3:>7.0f}x"
            )


def subsample(path: str, m: int = 3, trials: int = 20_000) -> None:
    """Observed spread of a `runs: m` statistic, from a deep run.

    Draws *fresh* Bernoulli runs from each case's observed rate rather than
    resampling the recorded runs without replacement. Without replacement is
    wrong here: taking 3 of 15 imposes a finite-population correction of
    `sqrt((15-3)/(15-1)) = 0.926`, shrinking the spread 7% for reasons that
    have nothing to do with the description. An earlier revision did exactly
    that and read the shrinkage back as the model being 8% conservative.
    """
    report = json.loads(pathlib.Path(path).read_text())
    if _reject(report, path):
        return
    cases = _group(report, negative=False)
    if not cases:
        print(f"{_label(path):<14} no should-fire cases")
        return
    depth = min(n for _, _, n in cases)
    if depth < m:
        print(f"    needs >= {m} runs per case; deepest common is {depth}")
        return
    rng = random.Random(SEED)
    rates = [x / n for _, x, n in cases]
    totals, diffs = [], []
    for _ in range(trials):
        a = sum(rng.binomialvariate(m, p) for p in rates)
        b = sum(rng.binomialvariate(m, p) for p in rates)
        totals.append(a)
        diffs.append(abs(a - b))
    mean = sum(totals) / len(totals)
    sd = math.sqrt(sum((t - mean) ** 2 for t in totals) / (len(totals) - 1))
    n_m = len(cases) * m
    print(f"\n=== {_label(path)}: observed runs:{m} spread")
    print(f"    depth {depth} runs/case, {len(cases)} cases, {trials} draws")
    print(
        f"    mean {mean:.2f}/{n_m}   sd {sd:.2f}   "
        f"sd(diff) {sd * 2 ** 0.5:.2f}"
    )
    for d in (2, 3, 4):
        share = sum(1 for v in diffs if v >= d) / len(diffs)
        print(
            f"    two runs of the SAME description differ by >= {d} "
            f"in {share:.0%} of pairs"
        )


def _self_test() -> None:
    import tempfile

    bad, checks = [], 0
    cases = [("a", 3, 3), ("b", 2, 3), ("c", 2, 3), ("d", 0, 3)]

    checks += 1
    if abs(_e_p_q(3, 3) - 0.0875) > 1e-9:
        bad.append(f"_e_p_q(3,3)={_e_p_q(3, 3)!r}, want 0.0875")
    checks += 1
    if _e_p_q(3, 3) <= 0:
        bad.append("a 3/3 case must not have zero variance")
    checks += 1
    if abs(_diff_sd(cases, 12) / _diff_sd(cases, 3) - 2.0) > 1e-9:
        bad.append("sd(diff) should double from runs 3->12")

    # The margin as a RATE must shrink with runs. An earlier model used the
    # single-measurement predictive variance, which held it constant, so no
    # run count ever resolved anything -- output that read like a finding.
    checks += 1
    r3 = _null_margin(cases, 3, trials=8000)[0] / 12
    r24 = _null_margin(cases, 24, trials=8000)[0] / 96
    if not r24 < r3:
        bad.append(f"margin rate must shrink with runs: {r3:.3f} -> {r24:.3f}")

    # The margin must be a real quantile. 1.96*sd is anti-conservative here.
    checks += 1
    margin, tail = _null_margin(cases, 3, trials=40_000)
    if tail > ALPHA:
        bad.append(f"margin {margin} has null tail {tail:.3f} > {ALPHA}")
    checks += 1
    if margin < math.ceil(1.96 * _diff_sd(cases, 3)):
        bad.append(
            f"margin {margin} undercuts the normal approximation "
            f"{1.96 * _diff_sd(cases, 3):.2f}, which is already too small"
        )

    with tempfile.TemporaryDirectory() as d:
        base = pathlib.Path(d)
        ok = {"graders": [{"name": "skill-fired", "passed": True}]}
        part = base / "partial.json"
        part.write_text(json.dumps({
            "partial": True, "partialReason": "self-test",
            "cases": [{"name": "x-one", "arms": {"with": [ok]}}],
        }))
        err = base / "errored.json"
        err.write_text(json.dumps({
            "cases": [{"name": "x-one", "arms": {"with": [{"error": "boom"}]}}],
        }))
        negonly = base / "neg.json"
        negonly.write_text(json.dumps({
            "cases": [{"name": "neg-hard-01", "arms": {"with": [ok]}}],
        }))
        for label, target in (("partial", part), ("errored", err)):
            checks += 1
            if not _reject(json.loads(target.read_text()), str(target)):
                bad.append(f"a {label} report must be refused")
        checks += 1
        try:
            subsample(str(negonly))
        except Exception as exc:  # noqa: BLE001 - any escape is the bug
            bad.append(f"subsample() raised with no should-fire cases: {exc!r}")
        checks += 1
        run = base / "2026-01-01T00-00-00Z"
        run.mkdir()
        if _label(str(run / "aggregate-result.json")) != run.name:
            bad.append("a generic stem must fall back to the run directory")

    # An over-trigger guard sits at 0, so a DOWNWARD shift is a no-op there
    # and sizing reports absurd run counts (137x was the observed symptom).
    # Shifting up must move it; shifting down must not.
    guard = [("neg-hard-01", 0, 3), ("neg-hard-02", 0, 3)]
    checks += 1
    up_hit, down_hit = _achieved(guard, 0.10, up=True), _achieved(guard, 0.10)
    # Upward lands in full (there is headroom above 0); downward is clipped.
    if not (abs(up_hit - 0.10) < 0.005 and down_hit < 0.08):
        bad.append(
            f"guard shift should land up={up_hit:.3f} (~0.10) vs clipped "
            f"down={down_hit:.3f} (<0.08)"
        )
    checks += 1
    m_up = _power_at(guard, 6, 0.20, 2, trials=8000, up=True)
    m_down = _power_at(guard, 6, 0.20, 2, trials=8000)
    if not m_up > m_down:
        bad.append(f"guard power up {m_up:.3f} must exceed down {m_down:.3f}")

    # `neg-hard` is a gated flow and must be sizeable, not filtered out by name.
    checks += 1
    negatives = _group(
        {"cases": [{"name": "neg-hard-01", "arms": {"with": [{"graders": []}]}}]},
        negative=True,
    )
    if not negatives:
        bad.append("negative cases must be available for sizing")

    for line in bad:
        print(f"    FAIL {line}")
    if bad:
        raise SystemExit(f"{len(bad)} power checks failed")
    print(f"power self-test: {checks}/{checks} pass")


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
    print(f"spread between two runs:{m} measurements of the SAME description\n")
    print(
        f"{'run':<14}{'dir':>5}{'observed':>9}{'cases':>6}{'n':>5}"
        f"{'sd':>8}{'margin':>8}{'tail':>8}{'as rate':>9}"
    )
    for p in args:
        analyse(p, m)
    print(
        f"\nmargin = smallest integer count difference whose null exceedance is\n"
        f"<= {ALPHA:.0%}, simulated. NOT 1.96*sd, which is anti-conservative on\n"
        f"counts this small. A gate margin below it cannot resolve what it\n"
        f"gates on. `dir` is which direction the counted event points: `fire`\n"
        f"for should-fire cases, `neg` for over-trigger guards."
    )
    for p in args:
        size(p, (0.20, 0.15, 0.10))


if __name__ == "__main__":
    main()
