# Eval history

How the `no-mistakes` skill description and this suite got to where they are,
in roughly chronological order. **Nothing here is the current state.** For
that, read the [README](README.md); for the gate procedure that came out of
these runs, read [METHODOLOGY.md](METHODOLOGY.md).

Sections are moved here verbatim from earlier revisions of the README.
Struck-through conclusions and `Retracted` warnings stay visible on purpose:
in each case the reasoning is the trap, and deleting it invites the next
person to rediscover it.

| PR | what changed | where it is recorded |
| --- | --- | --- |
| #979 | suite added; shipped description measured | [Baseline](#baseline-before-edit-flow-shipped-description-as-of-pr-979), [Flow pilots](#flow-pilots-1-run-shipped-description) |
| #981 | description reworded from the suite's findings | [Frontmatter A/B](#frontmatter-ab-result), [Candidate screening](#candidate-screening), [Held-out confirmation](#held-out-confirmation) |
| #985 | `after-edit` coverage #981 dropped restored — **current description** | [The `after-edit` regression](#the-after-edit-regression-981-shipped) |
| #986 | queue clause screened and rejected; cross-time gates found defective | [Gates for the queue clause](#gates-for-the-queue-clause-984-item-6) |
| #1024 | all eleven flows re-baselined; #986's price claim retracted | [The full re-baseline](#the-full-re-baseline), [the variance finding](#the-gate-cannot-resolve-the-difference-it-was-built-on) |
| #1033 | gate resolution measured; `power.py` added | [METHODOLOGY.md](METHODOLOGY.md#how-large-an-effect-can-a-gate-here-actually-resolve-1025) |

## Trigger rate by description

The shipped `description:` was reworked in #981 and corrected in #985. All at
`runs: 3`, should-fire cases only, from files with **zero** errored runs. The
**#979**, **#981** and **current, 09-13** columns are `--ablation none`; the
**current, 09-21** column is `--ablation with-without`, and its figures are
trigger counts taken from the **with** arm. Those are [directly
comparable](METHODOLOGY.md#comparing-trigger-counts-across-ablation-modes) — both are counts
over the with-arm, with the same number of with-arm runs per case — but the
condition differs and is labelled here rather than inferred.

- **#979** — the description PR #979 measured.
- **#981** — the rework. Won `before-edit` and `signature`, and silently lost
  `after-edit`.
- **current** — #981 plus a validation clause, in
  `skills/no-mistakes/SKILL.md` today.

| flow | #979 | #981 | current, 09-13 | current, 09-21 |
| --- | --- | --- | --- | --- |
| [`before-edit`](#candidate-screening) | 8/18 (44%) | 16/18 (89%) | 16/18 (89%) | **14/18 (78%)** |
| [`signature`](#candidate-screening) | 4/12 (33%) | 10/12 (83%) | 10/12 (83%) | **7/12 (58%)** [^sigctl] |
| [`after-edit`](#the-after-edit-regression-981-shipped) | 9/12 (75%) | **3/12 (25%)** | 12/12 (100%) | **10/12 (83%)** |
| [`queues`](#the-full-re-baseline) | — | — | 3/12 (25%) | **3/12 (25%)** |
| [`neg-hard`](#candidate-screening) — over-trigger guard, lower is better | 0/12 | 0/12 | 0/12 | **0/12** |
| [**live holdout**](#held-out-confirmation) — never tuned against | **3/9 (33%)** | **5/9 (56%)** | not re-run | not re-run |

[^sigctl]: A **separate** `--ablation none` control run, earlier the same day
    and not part of the 318-run re-baseline, measured this same description at
    **9/12**. It is kept out of the column because the column is defined as
    the both-arm re-baseline — but the two numbers together are the whole
    [variance
    finding](#the-gate-cannot-resolve-the-difference-it-was-built-on).

> [!WARNING]
> **The last two columns are the same description.** `signature` was measured
> twice on 2026-09-21 — 9/12 in a single-arm control and 7/12 in the
> re-baseline — so the gaps between these columns are mostly the instrument,
> not the description. A two-run difference anywhere in this table is **not**
> a result. See [the variance
> finding](#the-gate-cannot-resolve-the-difference-it-was-built-on), which
> retracted a price claim this file made earlier the same day.

The 09-21 column is the [full re-baseline](#the-full-re-baseline): all eleven
flows, both arms, 318 runs, $52.60, zero errored runs. Six flows are measured
at `runs: 3` for the first time there — `usage` 5/12, `ci` 2/12, `duplication`
0/9, `safety` 0/6, `napi` 0/12, `lang-graph` 10/12 (synthetic fixture).

**The live holdout is still 5/9, and cases 07-09 are still unspent.** They were
committed ahead of the queue clause to judge it, and were deliberately not run
once the clause failed gate 2 — a held-out case spent on a description that
does not ship is spent for nothing. The next candidate inherits them.

**Read the holdout row, not the tuned rows.** 56% is what generalizes; the
90%-ish figures are flows their description was written against. No
"94%"-style claim survives a clean holdout, and none is made here. The holdout
was not re-run for the current description — the validation clause was screened
on `after-edit`, which is now tuning-visible, so a fresh holdout needs [fresh
cases](README.md#writing-new-cases) written first.

**The `after-edit` row is why this table has a column per description.** #981 reported two
wins and listed `after-edit` as merely unmeasured. It was not neutral: it fell
from the best measured flow to the worst, 75% → 25%, and $3 of eval would have
caught it before merge. See [the regression](#the-after-edit-regression-981-shipped).

## Baseline (before-edit flow, shipped description as of PR #979)

`claude plugin eval . --tag before-edit --ablation with-without --judge-model sonnet`
at `runs: 3` — 48 runs, $8.89, ~10 min at `-j 4`.

| case | phrasing | fired | Δ |
| --- | --- | --- | --- |
| `04-all-references` | "make sure all references are updated" | 3/3 | +0.33 |
| `02-unused-exports` | "are any exports unused?" | 2/3 | +0.10 |
| `06-route-coverage` | "is it covered by e2e?" | 2/3 | +0.13 |
| `01-dead-in-production` | "is this dead? not used in production?" | 1/3 | −0.05 |
| `03-full-suite` | "why is the whole suite running?" | 0/3 | 0.00 |
| `05-where-used` | "where is X used?" | 0/3 | −0.13 |
| negatives (07, 08) | — | n/a | 0.00 |

**Trigger rate 8/18 = 44%.** Fire-only Δ +0.063.

This table describes the description that shipped with #979, which is **no
longer the one in `skills/no-mistakes/SKILL.md`** — see
[Candidate screening](#candidate-screening). It is kept as the before-side of
the comparison. The current description's own both-arm baseline was
[completed on 2026-09-21](#the-full-re-baseline).

The split is the finding: the three phrasings drawn from the most common forms
in real history (`01`, `03`, `05`) fire 1/9 combined, while the three that
happen to align with `SKILL.md`'s own vocabulary fire 7/9. `"where is X used?"`
— the single most frequent form in the mined corpora — never fires.

Δ is near zero wherever the skill does not fire, which is the expected shape:
with no invocation there is nothing for the with-arm to gain. Uplift is gated on
triggering, so **trigger rate is the metric to move first**.

## Flow pilots (1 run, shipped description)

Each flow piloted at `--runs 1 --tag <flow>` to calibrate rubrics before
spending three-run money. Trigger counts are against the **shipped** description,
so they are low by construction — see the A/B result above.

| flow | fired | reads |
| --- | --- | --- |
| `signature` | 3/5 | ⚠️ **superseded** — at `runs: 3` this flow is 4/12, and the 1-run 3/5 is what produced the phantom regression discussed in [Decision rules](#decision-rules-for-a-description-change). `signature-01-add-param` scored **1.00 vs 0.00**, the largest single gap measured |
| `lang-graph` | 3/5 | unexpectedly strong (Δ +0.67, +1.00, +0.33) despite the synthetic fixture |
| `ci` | 1/5 | one clear win on `ci-01-action-impact`; rest gated on triggering |
| `queues` | 1/5 | rubrics clean; Δ suppressed by non-triggering |
| `after-edit` | 1/5 | rubrics clean; Δ suppressed by non-triggering |
| `napi` | 0/5 | **weak flow — Δ ≈ 0 throughout.** `SKILL.md`'s N-API surface is about one sentence, so there is little for the skill to add. Treat its numbers as uninformative rather than as evidence the skill fails. |

Rubric bugs found and fixed during these pilots, both the familiar kind:
`napi-01` demanded the agent assert an implementation fact it cannot read
(hedging is correct); `napi-05` failed a with-arm answer that named
`package.json` but also mentioned an N-API version floor — the rubric now states
that additional sources are fine.

Because every flow's Δ is gated on triggering, these numbers mostly measure the
current description's silence. Re-run per flow after adopting a reworded
description to get uplift figures worth acting on.

## Frontmatter A/B result

`python3 evals/generate.py --variant real-register` emits the same cases against
`evals/variants/real-register/`, whose `SKILL.md` differs from the shipped one
by **one line** — the `description`, rewritten to lead with the question forms
that appear in real usage ("is this dead, still used, or safe to delete", "where
is a symbol used", "which tests actually need to run").

| | baseline | real-register |
| --- | --- | --- |
| `01-dead-in-production` | 1/3, Δ −0.05 | **3/3, Δ +0.24** |
| `02-unused-exports` | 2/3, Δ +0.10 | 3/3, Δ +0.05 |
| `03-full-suite` | 0/3, Δ 0.00 | **3/3, Δ +0.20** |
| `04-all-references` | 3/3, Δ +0.33 | 3/3, Δ +0.33 |
| `05-where-used` | 0/3, Δ −0.13 | **3/3, Δ +0.33** |
| `06-route-coverage` | 2/3, Δ +0.13 | 2/3, Δ −0.00 |
| negatives (07, 08) | 1.00 / 1.00 | 1.00 / 1.00 |
| **trigger rate** | **8/18 (44%)** | **17/18 (94%)** |
| **fire-only Δ** | +0.063 | **+0.192** |

### Held-out check

The variant description was written *after* seeing which cases failed, so the
tuned-set result alone cannot separate "matches real usage" from "overfit to
these six prompts". The `heldout` flow exists to settle that: same register,
wording that appears in no variant description.

| held-out case | baseline | real-register |
| --- | --- | --- |
| `heldout-01-can-this-go` | 1/3 | 3/3 |
| `heldout-02-more-than-shared-tests` | 3/3 | 3/3 |
| `heldout-03-still-pointing-at` | 0/3 | 3/3 |
| **trigger** | **4/9 (44%)** | **9/9 (100%)** |

The rewrite generalizes. Note the baseline scoring 44% here and 44% on the main
suite — that looks like a stable property of the shipped description rather than
an artifact of case selection.

Run with `--ablation none` (trigger rate is a direct observation and needs no
baseline arm), which means `skill-fired` is *scored* in those results rather
than display-only. Read the trigger counts, not the scores.

Conclusion: **reword the description; sub-skills are not needed.** A single
description spans every vocabulary tested, including wording it was never tuned
against. Splitting would cost permanent context and add selection ambiguity for
no measured gain.

Whenever a description changes, re-run `--tag heldout` and add new held-out
phrasings — a description tuned against the held-out set stops being held out.

### It does NOT generalize across flows

The A/B above covers `before-edit`. Running the other flows against the same
variant tells a different story:

| flow | shipped (⚠️ n=5) | reworded (n=15) |
| --- | --- | --- |
| `lang-graph` | 3/5 | 10/15 |
| `after-edit` | 1/5 | 5/15 |
| `ci` | 1/5 | 3/15 |
| `queues` | 1/5 | 2/15 |
| **`signature`** | **3/5** | **2/15** |
| `napi` | 0/5 | 0/15 |

The rewrite lifts `before-edit` to 94% but leaves the other flows between 0% and
67%, and **`signature` appears to regress**. The mechanism is plausible: the
variant enumerates specific question forms (dead / used / references / tests /
coverage) and drops the general "impact map … before editing to find callers and
tests" framing that was catching signature-shaped questions. Enumerating helps
the listed forms and can crowd out the unlisted ones.

Sample sizes differ (5 vs 15), so `signature` warrants confirmation at
`runs: 3` before acting on it.

~~**Consequence for the next description: ADD the real-register forms while
KEEPING the general framing, rather than replacing it.** A description that only
enumerates is a description that only fires on what it enumerated.~~

**Superseded — this conclusion was wrong.** It rested on the `signature`
regression, which [did not survive re-measurement](#signature-did-not-regress--the-35-vs-215-above-was-a-1-run-artifact),
and it was tested directly in [Candidate screening](#candidate-screening): the
candidate that keeps the general framing (C1) is worse than the one that drops
it (C2) on every flow measured. Struck through rather than deleted because the
reasoning is the trap, not the typo — "enumerating crowds out the unlisted" is
a plausible mechanism that happened not to be what the numbers said.

The should-not-fire cases were unchanged, so broadening the description did not
degrade text-search questions.

Caveat on how that was measured: at the time of this A/B, cases 07 and 08 carried
no `skill-fired` grader, so the result shows the *answers* did not suffer — not
that the skill stayed silent. Every case now carries the indicator (and the
`neg-hard` flow was added specifically as an over-trigger guard), so a re-run
reports whether a widened description fires on questions it cannot answer.

## `signature` did not regress — the 3/5 vs 2/15 above was a 1-run artifact

Re-measured at `runs: 3`, `--ablation none`, counting the four should-fire cases:

| description | `signature` trigger |
| --- | --- |
| shipped | 4/12 (33%) |
| `real-register` | 3/12 (25%) |

A one-count difference at n=12 is noise. The apparent regression in the table
above came from comparing a **single-run** shipped pilot (3/5) against a
three-run variant (2/15); the shipped number was inflated by the small sample.

The real finding is less convenient and more useful: `signature` sits at 25–33%
under *both* descriptions while `before-edit` reaches 94% under one of them. It
is not a flow one vocabulary wins and the other loses — it is a flow **neither**
vocabulary reaches. Do not build a gate on a one-count difference; it selects on
judge variance rather than on the description.

## Decision rules for a description change

**Pre-registered**: written and committed before the candidate screening numbers
existed, so the rule could not be chosen to fit whichever candidate won.

1. **`before-edit` trigger ≥ 17/18** — the `real-register` number. This is the
   flow with the most real traffic behind it and the only one measured at
   `runs: 3` under two descriptions, so it is the gate.
2. **`neg-hard` trigger ≤ 1/12** — the shipped description's floor is **0/12**
   (measured at `runs: 3`, `--ablation none`). One firing run at n=12 is not
   distinguishable from judge variance; **2 or more is a fail**, and the
   offending phrase must be tightened and that flow re-screened rather than
   traded away against a better `before-edit` number.
3. **`signature` is reported, not gated.** See below.

**Amendment**, written after C1's numbers and before C2's, so it constrains a
decision not yet made: a candidate clearing gates 1 and 2 wins. `signature`
breaks a tie only between candidates that both clear gate 1 or both fail it. It
is not promoted to a tiebreaker against `before-edit`, because `before-edit`
carries real observed traffic and has now been measured at `runs: 3` under
three descriptions, while `signature` has no observed demand at all and n=12
per arm — a 6/12 vs 4/12 gap is two runs.

**Amendment 2 (#986), and it supersedes the form of every rule above.** Each
gate here is written as an absolute count against a number measured on some
earlier day. [That is the defect #986
found](#but-the-floors-are-cross-time-and-that-is-a-defect-in-the-gate): the
suite's scores move between days, so a candidate can miss a fixed floor either
because it is worse or because the day is. The counts above stay as written —
rewriting a pre-registered rule after the fact is the thing this file exists to
prevent — but **from #986 onward a gate is a comparison against a control arm
measured in the same session, not against a stored number.**

Concretely, for the next candidate:

1. **Re-measure the shipped description today**, on every flow the candidate is
   gated on, as the control arm. This is the baseline — not the number in any
   table in this file.
2. **Build the control from today's skill, never from a stored variant.** See
   [why](METHODOLOGY.md#what-the-control-cost-to-get-right-and-the-trap-it-exposed).
3. **State each gate as a delta against that control**, with a margin wider
   than the reproducibility the control itself shows (#986's control moved one
   run at n=12 against its own eight-day-old measurement, so a one-run gap is
   not a result).
4. **Budget the cost, not just the win.** A clause that lifts its own flow can
   [charge another one](#control-result-the-baseline-held-the-clause-does-not-ship);
   gate on the flows the candidate does *not* target, and treat a flow the
   current description wins as something to hold, not something to spend.
5. **Judge the holdout last, and only if the gates pass.** Held-out cases
   07-09 are live and unspent for exactly this. A candidate that fails its
   gates must not be run against them.

**Amendment 3 (#1025)** replaced "a margin wider than the reproducibility" with
a computed margin from `power.py`. It is the current procedure and lives in
[METHODOLOGY.md](METHODOLOGY.md#gating-a-description-change).

## Candidate screening

Two candidates, screened at `runs: 3` with `--ablation none` on the target
(`before-edit`), the flow the issue was about (`signature`), and the guard
(`neg-hard`) — 17 cases, 51 runs, ~$9.50 each. Two rather than three on
purpose: screening N candidates and taking the best biases the winner's number
upward, and there are only three live held-out cases to confirm with.

- **C1** `evals/variants/general-plus-register/` — the shipped general framing
  (*deterministic impact map and test plan … before editing … instead of rg
  when …*) **plus** the real-register question forms.
- **C2** `evals/variants/register-plus-signature/` — the measured
  `real-register` description **plus** one signature clause, general framing
  still dropped.

Both apply the same two hardenings against the guards: `who imports or calls
it` became `which files import or call it`, so it does not reach
`neg-hard-01`'s "why is `roleHas` slow when we **call it** in a tight loop";
and `safe to delete` is kept adjacent so it does not reach `neg-hard-02`'s "is
`OutboundQueue` **safe to use** from two workers".

Trigger counts, should-fire cases only:

| flow | shipped | `real-register` | C1 | C2 |
| --- | --- | --- | --- | --- |
| `before-edit` | 8/18 (44%) | **17/18 (94%)** | 15/18 (83%) | 16/18 (89%) |
| `signature` | 4/12 (33%) | 3/12 (25%) | 6/12 (50%) | **10/12 (83%)** |
| `neg-hard` (lower is better) | 0/12 | not run | 0/12 | 0/12 |
| `07`/`08` should-not-fire | — | — | 0/6 | 0/6 |

**Neither candidate cleared gate 1.** C2 missed the 17/18 bar by a single run
and C1 by three. Under the pre-registered amendment — `signature` breaks a tie
between candidates that both fail gate 1 — **C2 wins**, and not narrowly: it is
ahead of C1 on every flow measured, at 10/12 versus 6/12 on `signature`, and of the three
descriptions actually evaluated on the `neg-hard` guards it fabricates a
command once, against five apiece for the other two. (`real-register` was never
run against that flow, so it cannot be ranked here.)

Read C2's 16/18 against `real-register`'s 17/18 as a tie. They are one run
apart at n=18, measured in different sessions, and C2 is `real-register` plus
one clause — the honest claim is that adding the signature clause cost nothing
on `before-edit` while moving `signature` from 3/12 to 10/12.

C1's result is the more interesting one. Restoring the general framing did not
help: it is *worse* than C2 on both target flows, and it reproduces the shipped
description's habit of fabricating a command form exactly as often (5/12 on
`neg-hard`). Combined with the step-1 finding that `signature` never
regressed, the conclusion the previous section reached — "ADD the real-register
forms while KEEPING the general framing, rather than replacing it" — **is not
supported**. Replacing it is better. What `signature` needed was a clause about
signatures, not the general framing back.

## What a non-firing run actually produces

`skill-fired` is binary, which hides the more interesting question: when the
skill does **not** load, what does the plan say instead? Classifying every
non-firing run, case-matched so each column sees the same questions.

On the four should-fire `signature` cases (12 runs each):

| | shipped | `real-register` | C1 | C2 |
| --- | --- | --- | --- | --- |
| fired | 4 | 3 | 6 | **10** |
| did not fire | 8 | 9 | 6 | 2 |
| … inventing a command form | **5** | 2 | **4** | 1 |
| … naming a real subcommand | 0 | 0 | 0 | 0 |
| … naming the tool, no command | 3 | 7 | 2 | 1 |

On the four `neg-hard` over-trigger guards (12 runs each; `real-register` was
never run against this flow, so it has no column):

| | shipped | C1 | C2 |
| --- | --- | --- | --- |
| fired | 0 | 0 | 0 |
| reached for the tool in prose anyway | 9 | 9 | 8 |
| … inventing a command form | **5** | **5** | **1** |
| … naming the tool, no command | 4 | 4 | 7 |
| stayed silent | 3 | 3 | 4 |

**A non-firing run never produces a working command.** Across all four
descriptions and both flows, the count of non-firing runs that named a real
subcommand is **zero** — the subcommands live in the skill body, so a plan
written without loading it cannot get them right. (The classifier checks the
captured token against the real subcommand set for exactly this reason;
matching any lowercase word would score the invented `no-mistakes roleHas` as
real, since `role` is a lowercase prefix of the symbol. That set is the 50
complete invocation paths read from `docs/cli/` link text at runtime — the
hand-listed tuple this file first shipped with held 20 of them, so a run naming
`no-mistakes lockfile` would have been scored as a fabrication; filename stems
then over-corrected, accepting the concept pages `graph.md` and
`diagnostics.md` as commands and `no-mistakes tests-plan` for the real `tests
plan`. Re-running the classification under each of the three moved none of the
counts in these two tables, so the zero above holds against complete paths.)

**A non-firing run is worse than a silent one.** Under the shipped description
the common outcome is not "the model forgot the tool exists" — it is the model
confidently writing `/no-mistakes roleHas` or `no-mistakes roleHas`, neither of
which exists. The description is good enough to be reached for and not good
enough to be used, so the plan names something that will fail. C2 does this
twice across its 14 non-firing runs on the two flows; the shipped description
does it 10 times across 20.

**The over-trigger guard needs reading in two parts.** All three descriptions
score a clean `skill-fired` 0/12 on `neg-hard`, and all three still reach for
the tool in prose on roughly 8–9 of those 12 runs. Widening the description did
not make that worse. What changes is the form: shipped and C1 fabricate a
command on 5 of them, C2 on 1. Read the guard as `skill-fired` **plus** this
classification — `skill-fired` alone reports all three as identical.

## Held-out confirmation

Run against the winner only, at `runs: 3`, `--ablation none`, and against the
shipped description on the same six cases for a before/after.

| held-out case | shipped | C2 | |
| --- | --- | --- | --- |
| `heldout-01-can-this-go` | 1/3 | 3/3 | spent |
| `heldout-02-more-than-shared-tests` | 2/3 | 3/3 | spent |
| `heldout-03-still-pointing-at` | 1/3 | 1/3 | spent |
| **spent subtotal** | **4/9** | **7/9** | |
| `heldout-04-swap-the-arg` | 1/3 | 3/3 | live |
| `heldout-05-other-side-of-the-outbox` | 0/3 | **0/3** | live |
| `heldout-06-two-copies` | 2/3 | 2/3 | live |
| **live subtotal** | **3/9 (33%)** | **5/9 (56%)** | |

**Read the live subtotal, not the aggregate.** Cases 01–03 are contaminated:
`real-register` was tuned against them and C2 is `real-register` plus one
clause, so their 4/9 → 7/9 measures very little. The number that means
something is **5/9**.

The shipped column reproduces this file's earlier 4/9 on the spent cases
exactly, measured months apart in a different ablation mode — a useful
reproducibility signal for the suite itself.

Against C2's tuned 16/18 (89%), a live holdout of 5/9 (56%) is the honest
generalization estimate, and the gap is large. It is a real improvement over
the shipped description's 3/9, and it is nothing like 89%. **No "94%"-style
claim should be made for any description on the strength of a tuned flow.**

The per-case split says why, and it is the same story as everywhere else in
this file: the signature-shaped held-out case goes 1/3 → 3/3, because C2 added
a clause about signatures; the queue-shaped one stays at 0/3, because it did
not add one about queues.

## Naming a subject reliably reaches it; not naming one is a coin flip

**Naming works.** `signature` sat at 25–33% under both the shipped and the
reworded description, neither of which mentioned signatures, arguments or
return types. Adding one clause about them took it to 83%, and the
signature-shaped held-out case — worded so it reuses none of that clause's
language — went 1/3 → 3/3.

**Not naming a subject is where it stops being predictable.** C2 says nothing
about producers, consumers, queues or jobs, and its queue-shaped held-out case
fires **0/3**, the worst cell measured anywhere in this suite. But it says
nothing about duplicates or repository-wide uniqueness either, and *that*
held-out case fires **2/3**.

So an earlier draft of this section overreached. It claimed the description
reaches what it names "and nothing else", which the holdout contradicts: an
unnamed subject scored 0/3 in one case and 2/3 in another. What the data
supports is asymmetric —

- a named subject is reached reliably (two flows, both lifted, and the lift
  survives held-out wording);
- an unnamed subject may or may not be, and nothing here predicts which.

That is still enough to act on, and it points the same way: if a subject
matters, name it, because leaving it to generalization is a gamble this suite
cannot handicap. What it does **not** support is the inference that every
low-scoring flow is low because it went unnamed — `duplication` shows an
unnamed subject can do fine. Enumeration does not crowd out the unlisted, which
is what the struck-through guidance above assumed; it simply does not reliably
reach it.

The obvious next move — add a queue clause and re-screen — is deliberately
**not** taken here. `heldout-05` is a live held-out case; tuning against it
spends it, which is the exact failure the holdout exists to prevent. It belongs
in a follow-up issue with fresh held-out cases written first.

## The `after-edit` regression #981 shipped

#981 changed the description on the strength of `before-edit`, `signature` and
the holdout, and listed `after-edit` among the flows it had not measured. That
framing was wrong in a specific way worth recording: the flow was not neutral
and unobserved, it was **broken by the change**, and the file's own finding
predicted it.

The old description said "*use … **after editing to validate***" and "*empty
plans that are not actually empty*". The rework dropped both. Measured
afterwards, case-matched at `runs: 3`, `--ablation none`, zero errored runs in
either file:

| case | #979 | #981 | current |
| --- | --- | --- | --- |
| `after-edit-01-what-to-run` | 3/3 | 1/3 | 3/3 |
| `after-edit-02-empty-plan` | 3/3 | **0/3** | 3/3 |
| `after-edit-03-moved-file` | 3/3 | 2/3 | 3/3 |
| `after-edit-04-handoff` | **0/3** | **0/3** | **3/3** |
| **should-fire** | **9/12 (75%)** | **3/12 (25%)** | **12/12 (100%)** |
| `after-edit-05-neg-script-name` (guard) | 0/3 | 0/3 | 0/3 |

`02-empty-plan` is the cleanest single datapoint in the suite. The case asks
"*I ran the test planner and it came back empty — does that mean nothing is
affected?*". The old description named that situation almost literally; the
rework deleted the phrase; the case went 3/3 → 0/3.

`04-handoff` is the control. It asks what to check before handing a branch off,
which **neither** older description named — and both score 0/3. The current
description names it and it goes 3/3. Same mechanism, observed as a null and
then as a fix.

**The correction.** Two clauses appended to #981's description, in its own
register, screened as a candidate rather than assumed:

> … does a route have e2e coverage; **what to run to validate a change before
> pushing or handing it off; does an empty test plan really mean nothing is
> affected.**

Gates were pre-registered before the numbers landed: restore `after-edit` to
≥9/12, hold `before-edit` ≥15/18 and `signature` ≥9/12 so #981's gains are not
spent, and keep `neg-hard` no worse than shipped. Screened on all four flows at
once — 22 cases, 66 runs, $12.61:

| flow | gate | result |
| --- | --- | --- |
| `after-edit` | ≥ 9/12 | **12/12** |
| `before-edit` | ≥ 15/18 | **16/18** |
| `signature` | ≥ 9/12 | **10/12** |
| `neg-hard` trigger | 0/12 | **0/12** |
| `neg-hard` fabrication | ≤ 1/12 | **0/12** |

Every gate passed, and two improved on #981 rather than merely holding: the
guard stopped fabricating a command form entirely, and `after-edit` beat the
old description as well as the new one. All eight should-not-fire cases across
the four flows stayed at 0.

**What this changes about how to read this file.** A flow listed as unmeasured
is not evidence of nothing; it is an absence of evidence, and when the change
under review *deletes* wording that named that flow, the absence is
load-bearing. The cost of being wrong here was $3 and ten minutes against a
merged regression. Any future description change should measure every flow
whose subject the diff removes, not only the flows it targets.

## Codex reads the same description — the `openai.yaml` gate was never real

An earlier revision of this file asserted that Codex consumes this skill
through an **always-on imperative** in `skills/no-mistakes/agents/openai.yaml`
(`interface.default_prompt`), while Claude gets only the description. A
pre-registered gate followed from that: remove the imperative once the
aggregate should-fire trigger reached ≥90%.

**That execution model is wrong, and the gate it justified does not exist.**
Per Codex's own spec (`skill-creator/references/openai_yaml.md`, shipped with
the Codex CLI):

- `agents/openai.yaml` is "an extended, product-specific config intended for
  the machine/harness to read, **not the agent**".
- `interface.default_prompt` is the "default prompt snippet inserted **when
  invoking** the skill" — a one-sentence example starting prompt for the UI,
  which is why the spec requires it to name the skill as `$skill-name`. It
  sits beside `display_name` and `short_description` under `interface:`
  because it is UI presentation.
- The field that actually governs ambient injection is
  `policy.allow_implicit_invocation`, which **defaults to true**. This skill
  declares no `policy` block, so it takes the default.

So Codex is injected with the `SKILL.md` description, implicitly, exactly as
Claude is. There is no Codex-side crutch, no asymmetry, and nothing for a
trigger-rate gate to unlock.

What this does and does not license, since the first draft of this correction
overshot it: the **input** is now known to be the same on both sides — the
description is the whole of what either agent gets, so a change to it is a
change to Codex's trigger surface too, and there is no Codex-side instruction
that would absorb a regression. What stays unmeasured is the **rate**. Every
case here runs `model: claude-opus-5` through `claude plugin eval`, so these
numbers are Claude's decision to invoke the skill. A different model with a
different skill-routing implementation can read identical metadata and trigger
at a different rate. **Do not quote a number in this file as a Codex result**;
a Codex-specific regression would be invisible here. Measuring that needs a
Codex arm the suite does not have.

**Outcome: the `default_prompt` was not *removed*** — not because a gate went
unmet, but because the thing the gate proposed to remove is a UI example prompt
whose deletion would not change any agent's behaviour. The 90% bar is withdrawn
rather than deferred; there is no measurement that would reinstate it.

**It was, however, rewritten in #986.** The defect this section had left for a
follow-up was that `default_prompt` still quoted the **old** description's
register ("*before editing to find callers and tests … instead of rg when …*"),
so the Codex UI suggested a starting prompt written in the vocabulary #981
replaced — three sentences where the spec asks for roughly one. It now reads:

> Use $no-mistakes to find every consumer of this symbol and the tests a change
> to it would need.

**No number in this file measures that string, and none can.** It is a UI
example prompt, not part of the trigger surface: `policy.allow_implicit_invocation`
governs ambient injection and the `description:` is what both agents actually
read. Every case here runs `claude plugin eval`, which never loads
`agents/openai.yaml` at all. It is changed here as a docs-consistency fix — the
shipped example should quote the shipped register — and it is explicitly **not**
covered by the gates the rest of this file describes.

## Gates for the queue clause (#984 item 6)

**Pre-registered**: committed before the screening run started, and before the
`queues` baseline for the current description existed.

The `after-edit` lesson applies directly — a clause is judged against a
case-matched baseline at `runs: 3`, never against a 1-run pilot, and never on
its target flow alone. So the screen covers the target *and* every flow the
current description already wins.

1. **`queues` trigger ≥ current + 3/12.** The current description names nothing
   about queues; the clause must move the flow, not merely fail to hurt it. A
   one- or two-run improvement at n=12 is judge variance.
2. **`before-edit` ≥ 15/18**, **`signature` ≥ 9/12**, **`after-edit` ≥ 10/12** —
   hold what #981 and #985 bought, within one to two runs of the measured
   16/18, 10/12 and 12/12.
3. **`neg-hard` trigger 0/12 and fabrication ≤ 1/12.** A clause about producers
   and consumers is the most plausible thing yet written to reach
   `neg-hard-02` ("is `OutboundQueue` safe to use from two workers"). If it
   does, the clause is retuned or dropped — not traded against the `queues`
   win.

Miss any of these and the clause does not ship; the flow stays unnamed and the
finding is recorded as-is. The holdout is judged separately and after, on cases
07-09, which were committed before this section.

### Screening result — the first clause failed the guard it was warned about

Two clauses were screened. Gate 3 caught the first one, on the exact case the
gate had named in advance.

| | v1 `evals/variants/register-plus-queues/` | v2 `evals/variants/register-plus-queues-v2/` |
| --- | --- | --- |
| clause | "which **producers and consumers** a queue or job connects, when they share no import" | "which **files enqueue** a job and **which files process** it, when producer and consumer share no import" |
| `queues` (gate ≥ 6/12) | 9/12 **PASS** | **12/12 (100%) PASS** |
| `neg-hard` (gate 0/12) | **1/12 FAIL** | **0/12 PASS** |
| … fabricated command forms | — | **0 PASS** |
| `before-edit` (gate ≥ 15/18) | 18/18 PASS | [gate 2 below](#gate-2-confirmation-under-the-shipped-v2-clause) |
| `signature` (gate ≥ 9/12) | 9/12 PASS | [gate 2 below](#gate-2-confirmation-under-the-shipped-v2-clause) |
| `after-edit` (gate ≥ 10/12) | 12/12 PASS | [gate 2 below](#gate-2-confirmation-under-the-shipped-v2-clause) |

v1's single firing run was `neg-hard-02-concurrency` — "is `OutboundQueue` safe
to use from two workers at the same time?" — which gate 3 named in advance,
together with the instruction to retune or drop rather than trade it against a
`queues` win of +6.

**The looser standard was available and was not taken.** The older [decision
rule](#decision-rules-for-a-description-change) reads "one firing run at n=12 is
not distinguishable from judge variance; 2 or more is a fail". Reaching for it
*after* seeing v1's +6 is exactly the post-hoc rule-fitting this suite exists to
prevent, so the clause-specific gate 3 governs and 1/12 fails.

The retune applies the same hardening that kept earlier descriptions off
`neg-hard-01` — "who imports or calls it" became "which files import or call
it". Frame the subject as **locating files**, not as a property of the queue:
"is this queue safe under concurrency" is a question about the queue, and a
clause phrased as a property of the queue reaches it. One phrased as a file
lookup does not.

### Gate 2 confirmation under the shipped v2 clause

Gate 2 was not re-screened standalone. The full both-arm re-baseline covers
`before-edit`, `signature` and `after-edit` under this exact description, and
trigger is read from the with-arm regardless of ablation mode, so it produces
the gate-2 numbers as a by-product.

Measured 2026-09-21, `runs: 3`, `--ablation with-without`, zero errored runs,
`partial: false`, cost per run $0.186 and $0.164 against the expected
$0.16–0.19.

| flow | gate | current (2026-09-13) | v1 (2026-09-13) | **v2 (2026-09-21)** |
| --- | --- | --- | --- | --- |
| `before-edit` | ≥ 15/18 | 16/18 | 18/18 | **15/18 — PASS, at the floor** |
| `signature` | ≥ 9/12 | 10/12 | 9/12 | **7/12 — FAIL** |
| `after-edit` | ≥ 10/12 | 12/12 | 12/12 | held, see below |

`signature` 7/12 misses the floor by two runs and sits three below the current
description. That is outside the "one to two runs" band the gate was written to
allow, so it is a fail and the looser [decision
rule](#decision-rules-for-a-description-change) ("2 or more is a fail") is not
available here — the clause-specific gate governs, exactly as it did for v1.

#### But the floors are cross-time, and that is a defect in the gate

Pre-registering absolute floors assumed the baseline holds still. It may not.
v2 differs from v1 by one clause reworded to be **narrower**, yet it is worse
than v1 on *both* untargeted flows — `before-edit` 18→15 and `signature` 9→7.
"The clause crowds out signatures" explains that. So does "runs on 2026-09-21
score lower than runs on 2026-09-13", and the gate cannot tell them apart,
because every floor in it is anchored to a measurement taken eight days
earlier.

This file's own standard is a **case-matched** baseline, and the rigorous
reading of that is *contemporaneous*. Comparing across time is what
manufactured the [`signature` regression that never
happened](#signature-did-not-regress--the-35-vs-215-above-was-a-1-run-artifact)
in #981. Repeating it here, in the section written to prevent it, would be the
same error with better paperwork.

**Pre-registered, before the control run started** — the control is the
pre-clause description measured today, `--tag signature --ablation none`, 15
runs, on a variant that differs from the shipped skill in the description line
only:

- Control **≥ 9/12** → the baseline held; the drop is the clause. v2 does not
  ship, the flow stays unnamed, the finding is recorded as-is.
- Control **≤ 7/12** → the baseline moved. The absolute floors are unanchored,
  the 2026-09-13 numbers are not comparable with today's, and no verdict on the
  clause is available without a same-day A/B. That is a finding about the
  method, and it outranks the clause.
- Control **= 8/12** → the two explanations are not separable at n=12. The
  pre-registered default stands: v2 does not ship.

`after-edit` and the holdout are deliberately **not** run until this resolves.
Spending held-out cases 07-09 on a description that may not ship would consume
them for nothing, and holdout integrity is the one property this suite cannot
buy back.

#### Control result: the baseline held. The clause does not ship.

Same-day, same machine, `--ablation none`, 15 runs, $2.75 at $0.183 per run,
zero errored runs, `partial: false`.

| `signature`, same four should-fire cases | trigger |
| --- | --- |
| pre-clause description, 2026-09-13 | 10/12 (83%) |
| **pre-clause description, 2026-09-21 (control)** | **9/12 (75%)** |
| **v2 clause, 2026-09-21** | **7/12 (58%)** |

The control lands one run below its own eight-day-old measurement — the
baseline held. Drift does not explain a three-run drop, so the first
pre-registered branch applies and **the queue clause does not ship**. `queues`
stays at 3/12, the subject stays unnamed, and the finding is recorded as-is.

The clause was reverted from `skills/no-mistakes/SKILL.md` in this PR;
`SKILL.md` is byte-identical to `main` again.

~~**This is the budget model's first measured price.** Every earlier run
measured what naming a subject *buys*; this one measures what it *costs*. The
clause bought `queues` 3/12 → 12/12 and charged `signature` 9/12 → 7/12 for
it.~~

> [!WARNING]
> **Retracted the same day, by the re-baseline below.** The shipped
> description was measured on `signature` a *second* time on 2026-09-21 and
> came back **7/12** — the clause's number. The 9/12 control and the 7/12
> clause run are two draws from the same distribution, not a price. See
> [the variance finding](#the-gate-cannot-resolve-the-difference-it-was-built-on).
> No price for the queue clause has been measured, and none of the numbers
> here establish one.

**`before-edit` is not part of that price, and saying it were would repeat the
error this section is about.** v2 measured 15/18 against the current
description's 16/18, but those are eight days apart with no same-day
`before-edit` control, so the one-run difference is not attributable to the
clause — it is the same cross-time comparison the gate defect above describes.
By the gate itself, `before-edit` **passed, within the band**. Splitting drift
from clause on that flow needs its own control arm, which was not run because
`signature` had already decided the verdict.

~~The honest summary is not "the clause is bad". It is that **this suite has no
evidence a clause can be added for free**, and the two flows it charged are the
one with the most real traffic behind it and the one the #984 issue was
about.~~ The honest summary is that **the suite could not tell**, which the
next section establishes directly.

#### The gate cannot resolve the difference it was built on

The [re-baseline](#the-full-re-baseline) measured the shipped description
across all eleven flows on 2026-09-21, hours after the control above. It
measured `signature` again — same description, same four cases, same machine,
same day:

| `signature`, shipped description | trigger | when |
| --- | --- | --- |
| 2026-09-13, `--ablation none` | 10/12 (83%) | eight days earlier |
| 2026-09-21, `--ablation none` (the control) | **9/12 (75%)** | ~20:05 UTC |
| 2026-09-21, `--ablation with-without` (re-baseline) | **7/12 (58%)** | ~21:25 UTC |
| *v2 clause*, 2026-09-21 | *7/12 (58%)* | *~20:25 UTC* |

**The shipped description scored 9/12 and 7/12 on the same day, and the clause
that was rejected scored 7/12.** The rejection rests on a gap the description
reproduces against itself.

This is not two different objects being compared. #986 left
`skills/no-mistakes/SKILL.md` byte-identical — its diff for that file across
the merge commit is empty, and the `description:` line has the same md5 on
both sides. Trigger counts are comparable across ablation modes: [both are
counts over the with-arm, with the same number of with-arm runs per
case](METHODOLOGY.md#comparing-trigger-counts-across-ablation-modes).

The same pattern appears on the other two gated flows:

| flow | shipped, 09-13 | shipped, 09-21 | v2 clause, 09-21 |
| --- | --- | --- | --- |
| `before-edit` | 16/18 | **14/18** | 15/18 — *between the two* |
| `after-edit` | 12/12 | **10/12** | not run |
| `signature` | 10/12 | **9/12 and 7/12** | 7/12 |

On `before-edit` the clause sits *between* two measurements of the description
it was compared against.

**What is and is not established.**

- **Not** that the clause should have shipped. Nothing here shows it is free;
  this is a coin flip, not an acquittal.
- **Not** that the pre-registration was misapplied. The control did land
  ≥9/12, branch one did apply, and the verdict followed the rule as written.
  The rule itself had no power.
- **That `runs: 3` at n=12 cannot resolve a two-run difference**, and every
  gate in this file is written in two-run units. The clause verdict is
  collateral; *this* is the finding.

**Consequence for the next candidate.** A gate margin must exceed the shipped
description's own same-day spread, and that spread is now measured rather than
assumed: it is **at least 2 runs at n=12**, so a margin that *exceeds* it is
**at least 3 runs at n=12** — and 2 runs is the observed floor, not an
estimate of the spread, so 3 is itself a lower bound on a defensible margin.
Either raise `runs` until the standard
error is smaller than the effect being gated, or widen the margin past it. Do
not re-run the queue clause against the current gates and read the answer —
the instrument cannot see a difference that size, whichever way it lands.

## The full re-baseline

**Completed 2026-09-21** (#984 item 5), after two earlier attempts. All eleven
flows, `runs: 3`, `--ablation with-without`, `-j 4`, Opus agent / Sonnet judge,
against the shipped description at `a722b2036`. **318 runs, $52.60, zero
errored runs, `partial: false` on every flow**, $0.165 per run against the
expected $0.16–0.19.

| flow | trigger (with-arm) | n | note |
| --- | --- | --- | --- |
| `before-edit` | **14/18 (78%)** | 18 | was 16/18 on 09-13 — see [the spread](#the-gate-cannot-resolve-the-difference-it-was-built-on) |
| `after-edit` | **10/12 (83%)** | 12 | was 12/12 on 09-13 |
| `lang-graph` | **10/12 (83%)** | 12 | **synthetic fixture** — grades plan shape against a repo that does not exist |
| `signature` | **7/12 (58%)** | 12 | also 9/12 earlier the same day |
| `usage` | **5/12 (42%)** | 12 | first `runs: 3` measurement |
| `queues` | **3/12 (25%)** | 12 | same total as 09-13; per-case shape matches what #986 recorded |
| `ci` | **2/12 (17%)** | 12 | first `runs: 3` measurement |
| `duplication` | **0/9 (0%)** | 9 | first `runs: 3` measurement |
| `safety` | **0/6 (0%)** | 6 | first `runs: 3` measurement |
| `napi` | **0/12 (0%)** | 12 | first `runs: 3` measurement |
| `neg-hard` — guard, lower is better | **0/12** | 12 | guard holds; 4 of 12 non-firing runs still invent a command form |

`heldout` is excluded on purpose: 07-09 stay unspent.

**Read this table with the [variance
finding](#the-gate-cannot-resolve-the-difference-it-was-built-on).** Each cell
is one draw. The same description measured `signature` at 9/12 and 7/12 hours
apart, so a two-run gap between any two cells here is not a result, and
neither is a two-run gap against the 09-13 column.

Four observations that are less fragile than a two-run gap — though none is
beyond re-measurement, since every count here is stochastic:

- **`queues` came back at 3/12 again**, and today's per-case shape —
  `queues-03-payload-change` 3/3, the three pure topology questions 0/3 —
  matches the shape #986 recorded for 09-13. (That earlier run's result file
  is gone, so this is a comparison against a recorded description of it, not
  against data still in hand.) Two independent runs landing on the same total
  *and* the same per-case split is the pattern you would expect if the three
  topology cases were failing for a structural reason — the subject is
  unnamed — and it is a reason to investigate that, **not** evidence that a
  later run cannot come back different. It can: this document records the
  same description at 9/12 and 7/12 on `signature`.
- **Three flows sit at zero**: `duplication` 0/9, `safety` 0/6, `napi` 0/12.
  Do not read these as structurally dead. With 6-12 trials the exact one-sided
  95% upper bounds are 39%, 28% and 22% respectively, so a substantial true
  rate is still consistent with observing none. A zero bounds the rate from
  above more usefully than a two-run gap bounds a difference, and that is the
  whole of the claim. None had a prior `runs: 3` shipped-description
  measurement, so these are first observations, not regressions.
- **`lang-graph` at 10/12 is the highest non-`before-edit` flow and means the
  least.** Its fixture is a polyglot repository that does not exist, so it
  grades plan shape against an imagined codebase.
- **Δ is now measured** for every flow for the first time, which is what this
  section existed to deliver. The values are below rather than only in a
  result file — those files were written outside the worktree and are not
  committed, which is precisely how the 2026-09-13 numbers were lost.

**Δ, mean weighted grader score per run** (`skill-fired` excluded, as
[`summarize.py`](summarize.py) does everywhere). Should-fire cases and
negatives are kept apart because a negative's with-arm is supposed to score
*no better* than its baseline:

| flow | should-fire with | without | **Δ** | negatives with | without | **Δ** |
| --- | --- | --- | --- | --- | --- | --- |
| `lang-graph` | 0.97 | 0.47 | **+0.50** | 1.00 | 1.00 | +0.00 |
| `signature` | 0.81 | 0.50 | **+0.31** | 0.67 | 1.00 | **−0.33** |
| `usage` | 0.64 | 0.36 | **+0.28** | — | — | — |
| `after-edit` | 0.94 | 0.75 | **+0.19** | 1.00 | 1.00 | +0.00 |
| `before-edit` | 0.87 | 0.73 | **+0.14** | 1.00 | 1.00 | +0.00 |
| `queues` | 0.64 | 0.56 | +0.08 | 1.00 | 1.00 | +0.00 |
| `napi` | 0.86 | 0.81 | +0.06 | 1.00 | 1.00 | +0.00 |
| `ci` | 0.61 | 0.61 | +0.00 | 0.33 | 0.67 | **−0.33** |
| `duplication` | 0.67 | 0.67 | +0.00 | 0.67 | 0.67 | +0.00 |
| `safety` | 0.83 | 0.83 | +0.00 | 0.33 | 0.67 | **−0.33** |
| `neg-hard` | n/a — all negatives | | | 1.00 | 1.00 | +0.00 |

Read these with the same caution as the trigger counts: each is one draw, and
the per-case n is 3. Two things stand out and are worth naming rather than
averaging away:

- **`lang-graph` has the largest Δ (+0.50) and the least meaning** — its
  fixture is a repository that does not exist, so a large Δ says the skill
  changes the plan's *shape*, not that the plan is right.
- **Three flows show a negative Δ on their negative cases** (`ci`, `safety`,
  `signature`, all −0.33, each a single case at n=3). That is the direction
  that would matter if it held: the plugin arm scoring *worse* than no plugin
  on a question the graph cannot answer. At n=3 per case it is one run
  flipping, so it is a thing to re-measure, not a finding.

### The two attempts before it

The first attempt is void: it hit a Claude session limit 27 runs into 318, and
the remaining 291 runs errored.

That failure is worth recording rather than just retrying, because of how it
presents. An errored run scores 0 in **both** arms, so 45 of the 53 cases came
back reading `0.00 / 0.00 / Δ +0.00` — indistinguishable from a genuine "the
skill made no difference here", across a suite where Δ ≈ 0 is a common and
expected result. The runner reported `partial: false`. The only immediate tell
was the cost: $6.08 against an expected ~$50.

`evals/summarize.py` now refuses to render such a run as a result. Anyone
re-running this should still check the reported cost against the estimate
below before believing a table of zeros, and should start the run with enough
session budget for ~320 runs.

Every other number in this file comes from runs with **zero** errored runs,
verified per file.

A second attempt, 2026-09-21, ran **flow by flow instead of as one job** —
`before-edit` and `signature` both-arm, then a single-arm control — for exactly
this reason. One invocation per flow means a session limit costs the flow in
flight, not the whole suite, and each flow's `aggregate-result.json` is checked
for errored runs and against `$0.16–0.19` per run before its numbers are
written down. Those three runs came in at $0.186, $0.164 and $0.183 per run
with zero errors. **Run the re-baseline this way**; the monolith has now failed
once and has no advantage.

Write the `--json` file somewhere **outside** the worktree. `evals/results/` is
gitignored, so results produced in a throwaway worktree die with it — which is
how the 2026-09-13 numbers were lost and had to be re-measured here.

## Measurement coverage

Not every flow has been measured to the same depth. Single-run numbers carry
real judge variance — cases have been observed flipping between runs — so treat
anything marked ⚠️ as directional.

Columns are descriptions, oldest first. **C2 is the description #981 shipped —
it is no longer current**; `register-plus-validate` is, and it is the column to
read when planning what still needs measuring.

| flow | #979 shipped | reworded | C1 | C2 = #981 | current (`register-plus-validate`) |
| --- | --- | --- | --- | --- | --- |
| `before-edit` | `runs: 3` | `runs: 3` | `runs: 3` | `runs: 3` | `runs: 3` |
| `signature` | `runs: 3` | `runs: 3` | `runs: 3` | `runs: 3` | `runs: 3` |
| `neg-hard` | `runs: 3` | not run | `runs: 3` | `runs: 3` | `runs: 3` |
| `after-edit` | `runs: 3` | `runs: 3` | not run | `runs: 3` | `runs: 3` |
| `heldout` (spent 01–03) | `runs: 3` | `runs: 3` | not run | `runs: 3` | **not run** |
| `heldout` (live 04–06) | `runs: 3` | n/a | not run | `runs: 3` | **not run** |
| `heldout` (live 07–09, added #986) | n/a | n/a | n/a | n/a | **never run — unspent** |
| `queues` | ⚠️ 1 run | `runs: 3` | not run | **not run** | `runs: 3` |
| `ci`, `lang-graph`, `napi` | ⚠️ 1 run | `runs: 3` | not run | **not run** | `runs: 3` |
| `usage`, `safety`, `duplication` | ⚠️ 1 run | not run | not run | **not run** | `runs: 3` |

The `after-edit` row is `runs: 3` under `#979 shipped`, `C2` and `current`
because [the regression check](#the-after-edit-regression-981-shipped) measured
all three; the ⚠️ 1-run pilot it used to carry has been superseded and must not
be quoted. The `heldout` rows are **not run** for the current description on
purpose — `after-edit` is now tuning-visible, so a clean holdout needs [fresh
cases](README.md#writing-new-cases) first.
