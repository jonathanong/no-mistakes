# Eval methodology

How to measure a description change on this suite without fooling yourself:
what the instrument can resolve, how to write a gate, and how to build the arms
being compared. Every rule here was paid for by a mistake recorded in
[HISTORY.md](HISTORY.md). Results and how to run the suite are in the
[README](README.md).

## How large an effect can a gate here actually resolve? (#1025)

Measured, not assumed. [`evals/power.py`](power.py) answers this from any
run's `aggregate-result.json`; the numbers below are from the
[2026-09-21 re-baseline](HISTORY.md#the-full-re-baseline) plus a deep `signature` run
(`--runs 15`, 75 runs, $12.77, zero errored runs) done specifically to check
the model against reality.

**The question a gate asks** is not "what is this description's rate" but "are
these two measurements different". Under the null both measure the *same*
description, so each case has one unknown rate `p_i` shared by both arms, and
uncertainty about `p_i` **cancels in the difference** instead of adding to it:

    Var(X - Y) = 2 * m * sum_i E[p_i (1 - p_i)]

with `E[p(1-p)]` taken over each case's Jeffreys posterior.

**The threshold is a one-sided simulated discrete quantile**, not `1.96 * sd`.
Two corrections, pulling in opposite directions:

- **Discrete, not normal.** These are small bounded counts and the normal
  approximation is anti-conservative on them: `1.96 * sd = 3.9` on `signature`
  rounds to 4, but `P(D >= 4) = 7.1%` two-sided.
- **One-sided, not two.** Every gate here rejects in one direction only — a
  should-fire flow rejects a *drop*, an over-trigger guard rejects a *rise*.
  Scoring `|D|` spends half the 5% budget on a tail no gate reads.

`allowed` below is the largest wrong-way gap a gate may tolerate. Write the
gate as `candidate >= control - allowed`: it passes on equality and first
fails one count later, which is exactly where the tail was simulated.

| flow | dir | n at `runs: 3` | **allowed** | tail | **a\|obs** | tail |
| --- | --- | --- | --- | --- | --- | --- |
| `before-edit` | fire | 18 | **4** | 2.7% | **5** | 3.3% |
| `signature` | fire | 12 | **3** | 2.6% | **3** | 3.2% |
| `after-edit` | fire | 12 | **3** | 1.8% | **4** | 1.7% |
| `neg-hard` | neg | 12 | **2** | 4.1% | **4** | 2.9% |

**The shipped gates carried margins of 1.** Against an allowed gap of 4 on
`before-edit`, a margin of 1 rejects on noise roughly a third of the time.
That is [the variance
finding](HISTORY.md#the-gate-cannot-resolve-the-difference-it-was-built-on) with a
number on it.

**`a|obs` is the column to use for the documented workflow**, and it is
usually the *looser* one. The two-arm `allowed` assumes both arms are future
draws. But the [gate procedure](#gating-a-description-change) measures the shipped description once and compares a
later candidate against *that realized number* — and a control that came in
high regresses down, which looks exactly like the candidate failing.
Conditioning on the realized control accounts for it. Ignoring the
distinction is anti-conservative: with four cases at 3/3, a two-arm cutoff
carries a 2.1% tail while conditioning on the realized 12/12 gives **8.6%**.
Use `allowed` only when both arms will be re-run.

`neg-hard` is in that table because it is a **gated** flow that an earlier
revision could not size at all — every negative case was filtered out by
name, so the documented procedure printed "no clean should-fire cases" for
the over-trigger guard.

### Checked against resampling — after the first check turned out circular

~~The model is 8% conservative, which is the direction to err in.~~

> [!WARNING]
> **Retracted.** The first version of this check resampled `runs: 3` draws
> from the deep run **without replacement**, which imposes a
> finite-population correction of `sqrt((15-3)/(15-1)) = 0.9258`. And
> `1.96 × 0.9258 = 1.815` against the 1.81 that was reported as agreement:
> the check was measuring its own resampler. Re-done by drawing *fresh*
> Bernoulli runs from each case's observed rate:

| | sd of the difference |
| --- | --- |
| model | **1.96** |
| observed, 20 000 fresh-draw pairs | **1.96** |

The model is neither conservative nor optimistic; it is right. Per-case rates
on the deep run were 13/15, 10/15, 6/15 and 1/15. The blunt version of the
same resampling: **two runs of the identical description differ by ≥2 in 44%
of pairs, ≥3 in 20%, ≥4 in 7%.** The 9-vs-7 gap that rejected the queue
clause is a 44% event.

### What a real gate costs

A gate whose critical gap merely *equals* the effect you care about catches
that effect about **half** the time. `power.py` sizes for 80% power instead,
which roughly doubles the count a critical-value calculation gives.
`--runs` is *per case*, and the flow's negative cases run too:

| flow at 10% | `runs` | allowed | total runs | one arm | both arms |
| --- | --- | --- | --- | --- | --- |
| `before-edit` | 35 | 13 | 8 × 35 = 280 | $48 | **$95** |
| `signature` | 58 | 13 | 5 × 58 = 290 | $49 | **$99** |
| `neg-hard` | 35 | 8 | 4 × 35 = 140 | $24 | **$48** |

At 20% the same flows need `runs: 10`, `17` and `10` — 3–6× the current
depth.

**Direction is a parameter, not a detail.** The default sizes for catching a
*regression*. A target gate such as `queues >= current + 3/12` is an
**improvement** gate, and on a low-rate flow the two are not symmetric,
because a downward shift clips at zero. On `queues` — three cases at 0/3, one
at 3/3 — a nominal 20% drop lands as 12% and needs `runs: 26`, while the
upward alternative the gate actually cares about lands as 17% and needs
`runs: 13`. Pass `--improve` for a target gate, or size twice the runs you
need.

**The `neg-hard` gate has two halves and both need sizing.** `skill-fired`
0/12 is one; fabrication ≤ 1/12 is the other, and it is a different outcome
computed from the non-firing runs. `power.py --fabrication` sizes it: the
re-baseline observed **4/12** fabricated with a tolerated gap of **3**,
against a shipped threshold of 1. Classification comes from
[`summarize.py`](summarize.py)'s classifier rather than a second copy, so
the two cannot drift. At $0.17/run, **one properly-powered gated flow at 10% costs about
what [the entire eleven-flow re-baseline](HISTORY.md#the-full-re-baseline) did**
($52.60). That is the real reason the old gates were written the way they
were, and wanting it otherwise does not change it: either pay for the runs,
or gate on effects large enough to see at `runs: 3`, which means **allowing
2–4 counts of slack**, not 1.

The `real` column `power.py` prints matters on low-rate flows. The modelled
alternative shifts every case's rate by the target amount, clipped at 0, so
on `signature` — where one case sits at 1/15 — a nominal 10% shift lands as
9%. Sizing against the nominal figure would understate the cost.

### Two limits on these numbers

- **Resampling within one session cannot see between-session drift.** The four
  measurements of this description on `signature` span 10/12, 9/12, 7/12 and
  6/12 (the deep run). A same-day comparison of the last two groups is *not*
  significant (16/24 vs 30/60, z = 1.39, p = 0.17), so there is no drift claim
  here — but the deep run sits at the bottom of the range, and an unmodelled
  between-session component would make the margin **larger**, never smaller. Treat
  these as lower bounds and keep the control arm in the same session as the
  candidate.
- **The margin is per flow, and the model is validated on one.** It was checked
  against resampling on `signature` only; `before-edit` and `after-edit` are
  the same model applied to their own per-case rates. That is a reasonable
  extrapolation — the model has one moving part — but it is an extrapolation.

## Gating a description change

A gate is a comparison against a **control arm measured in the same session**,
never against a number stored in a table. The suite's scores move between days,
so a candidate can miss a fixed floor either because it is worse or because the
day is ([why](HISTORY.md#but-the-floors-are-cross-time-and-that-is-a-defect-in-the-gate)).

For the next candidate:

1. **Re-measure the shipped description today**, on every flow the candidate is
   gated on, as the control arm.
2. **Build the control from today's skill, never from a stored variant.** See
   [below](#what-the-control-cost-to-get-right-and-the-trap-it-exposed).
3. **Size the margin with `power.py` and state it before the candidate runs:**

   ```sh
   # 1. measure the shipped description today, on each gated flow
   pnpm run evals -- --tag <flow> --ablation none --judge-model sonnet -j 4 \
     --no-publish --trust-plugin --json "$OUT/<flow>-control.json"

   # 2. ask what that run can resolve, and what more runs would buy
   python3 evals/power.py "$OUT/<flow>-control.json"
   ```

   Then pick one, in the open:

   - **Accept the coarse gate.** At `runs: 3` the gate must tolerate the
     simulated wrong-way gap — 2–4 counts on the gated flows, or up to 5 when
     conditioning on a realized control. Read the `a|obs` column, not
     `allowed`, whenever the control arm is a run you already have. A gate this
     coarse only catches large effects, and saying so up front is the point.
   - **Or pay for resolution.** `power.py` prints the `runs` for a 20/15/10%
     target **at 80% power**. `--runs` is per case, so 10% on one flow is
     **$48–112 both arms**; 20% is 3–6× `runs: 3`.

   A gate whose margin is below the measured one is not a weak gate, it is a
   coin flip with a number next to it. Do not write one.
4. **Budget the cost, not just the win.** Gate on the flows the candidate does
   *not* target too, and on every flow whose subject the diff *removes* — that
   is the check that would have caught
   [#981's `after-edit` regression](HISTORY.md#the-after-edit-regression-981-shipped).
   Treat a flow the current description wins as something to hold, not spend.
5. **Gate `neg-hard` on both halves**: `skill-fired`, and fabricated command
   forms in the non-firing runs (`power.py --fabrication` sizes the second).
   `skill-fired` alone reports every description as identical
   ([evidence](HISTORY.md#what-a-non-firing-run-actually-produces)).
6. **Judge the holdout last, and only if the gates pass.** A candidate that
   fails its gates must not be run against held-out cases — a spent case cannot
   be bought back.

Screen at most two candidates per round: screening N and taking the best biases
the winner's number upward, and the holdout has few live cases to confirm with.

## Running a measurement

- **Run flow by flow, not as one job.** A session limit then costs the flow in
  flight rather than the whole suite. The single-job re-baseline
  [failed once](HISTORY.md#the-two-attempts-before-it) and has no advantage.
- **Check each flow's `aggregate-result.json` before writing a number down**:
  zero errored runs, `partial: false`, and cost within $0.16–0.19 per run. An
  errored run scores 0 in *both* arms and renders as a plausible `Δ +0.00`.
  `summarize.py` refuses such runs, but a table of zeros at a fraction of the
  expected cost is the tell.
- **Write `--json` outside the worktree.** `evals/results/` is gitignored, so
  results produced in a throwaway worktree die with it. Copy the numbers that
  matter into the docs.
- **Compare like with like.** Case-matched, same `runs`, same session. Never
  compare a 1-run pilot against a `runs: 3` measurement — that is how the
  [phantom `signature` regression](HISTORY.md#signature-did-not-regress--the-35-vs-215-above-was-a-1-run-artifact)
  happened.

## Comparing trigger counts across ablation modes

Trigger counts from an `--ablation none` run are directly comparable with those
from an `--ablation with-without` run: both are counts over the **with-arm**
runs, and the number of with-arm runs per case is the same either way. Only the
*scores* differ in scale, because `skill-fired` is scored under `none` and
display-only under `with-without` — which is why `evals/summarize.py` drops that
grader from every score it prints.

## What the control cost to get right, and the trap it exposed

The obvious way to run this control is to point it at
`evals/variants/register-plus-validate/`, the stored variant holding the
pre-clause description. **That would have measured the wrong thing.** The
stored variants are frozen copies of `SKILL.md` *and* its `references/`, so
every one of them rots the moment the skill body changes — and the body had
changed under them (#1001, #1007, #1008 edited `SKILL.md` and
`references/dependencies.md`). A control run against a stale variant differs
from the shipped skill in the description **and** the body, and reports the sum
as if it were the clause.

Build a control from the shipped skill instead, never from a stored variant:

```sh
mkdir -p evals/variants/<name>
cp -R skills/no-mistakes/references evals/variants/<name>/references
cp skills/no-mistakes/SKILL.md evals/variants/<name>/SKILL.md
# replace ONLY the description: line, then prove it:
diff skills/no-mistakes/SKILL.md evals/variants/<name>/SKILL.md   # line 3 only
diff -r skills/no-mistakes/references evals/variants/<name>/references  # silent
python3 evals/generate.py --variant <name>
```

The two `diff`s are the point: [screening
discipline](HISTORY.md#candidate-screening) says "verify only line 3 differs", and that
check is what catches the rot. The variants committed in this repo are correct
*as of the run that produced them* and must not be reused for a later
comparison without re-verifying both diffs.

## Adding or rebuilding a variant

To add another variant: create `evals/variants/<name>/` (copy the skill, change
only the frontmatter), then `python3 evals/generate.py --variant <name>` and run
with `--eval-dir evals-variants/<name>`. The plugin under `skills/` is never
modified to run a comparison.

Screened candidates are kept under `evals/variants/` so their comparison can be
re-run. Replicas of *past shipped* descriptions are not — they live in git
history, and checking each one in permanently would double the diff of every
description change for no added information. To rebuild one (this is exactly
how the `#979` and `#981` columns in [HISTORY](HISTORY.md#trigger-rate-by-description) were produced):

```sh
name=shipped-pre-981
sha=<the commit that shipped that description>

mkdir -p "evals/variants/$name"
cp -R skills/no-mistakes/references "evals/variants/$name/"
cp skills/no-mistakes/SKILL.md "evals/variants/$name/SKILL.md"

# Overwrite the `description:` line in the COPY with the historical one. Reading
# the old line without writing it is the whole trap: the copy keeps the current
# description, the variant run looks fine, and it silently measures the arm you
# already have.
python3 - "$name" "$sha" <<'PY'
import pathlib, subprocess, sys
name, sha = sys.argv[1], sys.argv[2]
old = subprocess.run(
    ["git", "show", f"{sha}:skills/no-mistakes/SKILL.md"],
    capture_output=True, text=True, check=True,
).stdout
desc = next(l for l in old.splitlines() if l.startswith("description:"))
p = pathlib.Path(f"evals/variants/{name}/SKILL.md")
lines = p.read_text().splitlines(keepends=True)
for i, l in enumerate(lines):
    if l.startswith("description:"):
        assert lines[i] != desc + "\n", "copy already has the historical line — wrong sha?"
        lines[i] = desc + "\n"
        break
else:
    raise SystemExit("no description: line found")
p.write_text("".join(lines))
print("wrote:", desc[:80])
PY

python3 evals/generate.py --variant "$name"
```

**Verify before spending**, or the run measures something other than the
description:

```sh
sed '3d' skills/no-mistakes/SKILL.md > /tmp/a.md
sed '3d' "evals/variants/$name/SKILL.md" > /tmp/b.md
diff /tmp/a.md /tmp/b.md && echo "only line 3 differs"
diff -r skills/no-mistakes/references "evals/variants/$name/references" && echo "references identical"
```

Both must be clean, and the two `description:` lines must actually differ —
that is the check the assertion above enforces.
