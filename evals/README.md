# `no-mistakes` skill evals

Eval suite for the `skills/no-mistakes` skill.

## Results

The shipped `description:` was reworked in #981 and corrected in #985. All at
`runs: 3`, should-fire cases only, from files with **zero** errored runs. The
**#979**, **#981** and **current, 09-13** columns are `--ablation none`; the
**current, 09-21** column is `--ablation with-without`, and its figures are
trigger counts taken from the **with** arm. Those are [directly
comparable](#comparing-trigger-counts-across-ablation-modes) — both are counts
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
cases](#writing-new-cases) written first.

**The `after-edit` row is why this table has a column per description.** #981 reported two
wins and listed `after-edit` as merely unmeasured. It was not neutral: it fell
from the best measured flow to the worst, 75% → 25%, and $3 of eval would have
caught it before merge. See [the regression](#the-after-edit-regression-981-shipped).

### What the runs established

- **[The instrument cannot resolve the differences these gates are built
  on.](#the-gate-cannot-resolve-the-difference-it-was-built-on)** The shipped
  description scored `signature` 9/12 and 7/12 on the same day; the queue
  clause rejected for scoring 7/12 scored exactly what the description
  reproduces against itself. `runs: 3` at n=12 cannot see a two-run effect,
  and every gate here is written in two-run units. This retracted a price
  claim made earlier the same day, and it outranks every trigger number below.
- **A named subject is reached, but its cost is unmeasured.** The queue clause
  did what the naming model predicts — `queues` 3/12 → 12/12 — and was
  rejected on a `signature` gate the instrument could not resolve. Whether a
  twelfth clause is affordable is **still an open question**; this suite has
  not answered it in either direction.
- **[Naming a subject reliably reaches it; not naming one is a coin
  flip.](#naming-a-subject-reliably-reaches-it-not-naming-one-is-a-coin-flip)**
  Now measured in both directions. *Adding* a name lifts its flow: signatures
  25–33% → 83%, post-edit validation 25% → 100%, and queues 25% → 100%.
  *Removing* one drops it: #981 deleted "after editing to validate" and
  `after-edit` fell 75% → 25%.
  Unnamed subjects are unpredictable rather than dead — the queue-shaped
  held-out case fires 0/3 and the duplication-shaped one 2/3 under the shipped
  description, which names neither. So name what matters, treat any deletion
  as a change to be measured, and do not read every low flow as merely
  unnamed. Whether naming one is *cheap* is a separate question this suite
  has not answered — the queue clause was the attempt, and its cost [could
  not be
  resolved](#the-gate-cannot-resolve-the-difference-it-was-built-on).
- **[A description is a budget, not a
  bag.](#the-after-edit-regression-981-shipped)** #981's rework was framed as
  replacing a vague framing with a concrete one. What it actually did was
  reallocate: two subjects gained roughly what one lost. The fix was not a
  better framing but more named subjects — the current description is #981 plus
  two clauses, and it holds every flow #981 won while restoring the one it
  broke, at 684 characters against a 1536 cap. Whether a *twelfth* clause is
  affordable is untested: the [queue
  clause](#the-gate-cannot-resolve-the-difference-it-was-built-on) was the
  attempt, and the instrument could not resolve its cost either way.
- **[Absolute gate floors expire.](#but-the-floors-are-cross-time-and-that-is-a-defect-in-the-gate)**
  Gate 2 pre-registered floors as fixed counts against eight-day-old
  measurements. That is a cross-time comparison — the exact error that
  manufactured the [phantom `signature`
  regression](#signature-did-not-regress--the-35-vs-215-above-was-a-1-run-artifact).
  Pre-register the floors as a **same-day control arm**, not as a number.
- **[`signature` never
  regressed.](#signature-did-not-regress--the-35-vs-215-above-was-a-1-run-artifact)**
  The 3/5 → 2/15 drop that motivated the rework compared a *single-run* pilot
  against a three-run variant. Re-measured, both descriptions sit at 25–33%.
- **[Keeping the general framing is
  worse.](#candidate-screening)** The candidate that kept it and added the real
  register lost on every axis to the one that replaced it.
- **[A non-firing run is worse than a silent
  one.](#what-a-non-firing-run-actually-produces)** The old description's
  common failure is not "forgot the tool exists" — it is confidently writing
  `/no-mistakes roleHas`, which does not exist. It does that 10 times across 20
  non-firing runs; the current description, twice across 14 — **both counts
  scoped to the two-flow screen** (`signature` and `neg-hard`) that produced
  them. The [2026-09-21 re-baseline](#the-full-re-baseline) is a wider sample
  and a worse one: on `neg-hard` alone the current description invents a
  command form in **4 of 12** non-firing runs. Treat "twice across 14" as a
  historical screen figure, not the current rate. Across every description and
  flow, a non-firing run named a real subcommand **zero** times.
- **[Codex reads the same description Claude
  does.](#codex-reads-the-same-description--the-openaiyaml-gate-was-never-real)**
  An earlier revision of this file claimed `agents/openai.yaml` gives Codex an
  always-on imperative. It does not — the description is the whole of what
  either agent gets. That makes the description Codex's trigger surface too,
  but the **rates above are Claude's**: every case runs `claude-opus-5`, and
  the suite has no Codex arm.

### Caveats

- **[This suite cannot resolve a difference smaller than about four
  counts.](#how-large-an-effect-can-a-gate-here-actually-resolve-1025)** The
  caveat that governs every other number in this file, and it is now
  quantified rather than asserted: at `runs: 3` the 95% same-description
  difference threshold is **6** counts on `before-edit`, **5** on
  `signature`, **4** on `after-edit` and **4** on the `neg-hard` guard — a
  simulated discrete quantile, because `1.96 * sd` is anti-conservative on
  counts this small. Two runs of the *identical* description differ by ≥2 in
  **44%** of pairs. Treat any gap below the margin — between descriptions,
  dates, or flows — as unresolved, not as a finding, and run
  [`evals/power.py`](power.py) before writing a gate.
- **Δ is now measured for all eleven flows**
  ([re-baseline](#the-full-re-baseline), 2026-09-21, 318 runs, zero errored
  runs). [Baseline](#baseline-before-edit-flow-shipped-description-as-of-pr-979)
  is kept as the #979 historical record and is **not** the current description.
- **Every flow in the routine suite now has a `runs: 3` measurement under the
  current description** — the eleven non-holdout flows, which is what the
  previous "six flows are still unmeasured" caveat asked for. `heldout` is
  *deliberately* excluded and remains unmeasured under this description;
  running it would spend cases 07-09. Three of them came back at **zero** — `duplication` 0/9, `safety` 0/6,
  `napi` 0/12. A zero count is **not** thereby certain: the exact one-sided
  95% upper bounds are 39% (0/6), 28% (0/9) and 22% (0/12), so the true
  trigger rate could still be substantial. What a zero does rule out is a
  *high* rate, which is more than a two-run gap rules out — but these are
  first observations needing repeated measurement, not settled facts.
- **A low flow is not automatically a bug to fix.** `queues` is still the
  worked example, though for a different reason than this file claimed
  earlier: naming the subject does lift it, the attempt to price that lift
  [could not be resolved by the
  instrument](#the-gate-cannot-resolve-the-difference-it-was-built-on), and
  the flow stays at 3/12. Before opening the next "flow X is low" issue,
  check that the suite can actually measure the fix.

---

Run one flow with:

```sh
pnpm run evals -- --tag before-edit --ablation with-without --judge-model sonnet
```

`pnpm run evals` wraps `claude plugin eval .` via
[`scripts/run-evals.sh`](../scripts/run-evals.sh). Prefer it over the raw
command: pnpm forwards its own `--` into the script's argv, and the eval CLI
reads that as end-of-options — it silently discards every flag after it and
launches an unfiltered full-suite run. The wrapper strips the `--`, and refuses
to launch unscoped, so a mistyped flag cannot cost $65 unintentionally. Pass
`--all` when an unfiltered run is what you actually want. (Unfiltered is 62
cases x 3 runs x 2 arms = 372 runs: a path target resolves a plugin, and the
ablation default is then `with-without`, not `none`.)

See [Flows](#flows) for the full-suite command — it deliberately excludes the
`heldout` tag, which only means anything while those cases stay unseen.

Add `--no-publish` to keep the HTML report local. Other flags the numbers below
depend on: `-j 4` (concurrency defaults to **1**, so every cost and duration
figure here assumes `-j 4`), `--runs <n>` to override the per-case `runs: 3`,
`--max-cost-usd <n>` as a hard ceiling, and `--json <path>` for machine-readable
per-run results alongside the HTML report.

The headline number is **Δ** — the with-plugin score minus the without-plugin
score. A high absolute score with Δ ≈ 0 means the model would have done just as
well without the skill.

## What this suite measures

**The quality of the impact-scoping plan the agent produces**, in plan mode,
for a question about a codebase it has been told it cannot read.

Each case is a question phrased the way the questions actually get asked —
sourced from ~9,800 real prompts across Claude Code, Codex, Cursor and Grok
histories, then re-pointed at public `auto-harness` symbols. None of them use
the vocabulary `SKILL.md` is written in ("blast radius", "who calls", "which
tests should I run"); across all four corpora those phrasings appear zero
times. The cases deliberately use the real register instead: *is this dead*,
*where is X used*, *are any of these unused*, *make sure all references are
updated*, *why is the whole suite running*.

Graders are outcome-shaped: they score whether the **plan** would find what
needs finding. The design bias throughout is **recall over precision** —
over-inclusive results are explicitly not penalised, a missed category is the
failure. Several rubrics state this in so many words.

### How to read Δ on this suite

Piloting established that Opus writes a *good* impact-scoping plan with or
without the skill — it reasons about barrels and re-export chains unprompted.
The rubrics that ask "would this plan find the consumers" therefore pass in
both arms.

Where the arms genuinely differ is whether the plan is exhaustive **by
construction** or merely exhaustive **if executed carefully** — a grep plan's
completeness depends on the operator correctly following `export *` chains and
knowing when to stop. The `exhaustive-by-construction` grader on the
recall-critical cases (01, 02, 05) is the one that targets that difference; a
text-search plan written to be genuinely exhaustive can and should pass it.

The `names-graph-command` regex secondaries measure **tool adoption**, not plan
quality. If Δ is carried entirely by those, the honest reading is "the skill got
used", not "the plan got better". Decompose per-grader before claiming uplift.

## What this suite does NOT measure

### Trigger rate in a real session

This is the big one, and it is the question that prompted the suite.

Cases run in an **empty sandbox working directory** — no checkout, no
`node_modules`, no `no-mistakes` binary, and no network to fetch one. The
prompts therefore tell the agent the repository is unavailable and ask for a
plan.

That framing removes the choice the trigger question is about. In a real
session the agent *can* grep, and that is precisely when it skips the skill.
Here it cannot, so it is pushed toward the skill. **Any `skill-fired` count
from this suite is an upper bound, not the real-session rate.** Treat it as a
number to compare *between* variants, never as an absolute.

Measuring the real rate needs a checkout present in the sandbox, which needs
both a working `scaffold_script` and a locally staged native binary. Note the
trap before attempting it: with a repo present but no working binary, the
with-plugin arm will try the CLI, fail, and fall back to grep while the
without-plugin arm greps successfully — Δ goes negative and the plugin looks
actively harmful.

### Whether the CLI returns correct answers

No case runs `no-mistakes`. These evals test the skill's *routing and
guidance*, not the engine. Engine correctness is covered by `test-cases/**`
and the Rust test suite.

### Cost or latency versus an alternative approach

The suite records per-run cost, duration and turn count, but both arms answer
from an empty directory, so those numbers say nothing about how the tool
compares to a broad search agent on a real repo.

### Engine behaviour under a real workload

No case exercises a large repository, concurrency, or a cold graph build.
Nothing here is a performance or scale test.

## Flows

Every case carries a `tags:` field naming its flow. Run one flow or all of them:

```sh
claude plugin eval . --tag queues --ablation with-without --judge-model sonnet

# Everything EXCEPT the holdout. Prefer this for routine runs: the `heldout`
# cases only mean something while they stay unseen, and an unfiltered run
# exposes them during ordinary tuning.
claude plugin eval . --ablation with-without --judge-model sonnet \
  --tag before-edit queues after-edit signature ci usage safety duplication \
        neg-hard lang-graph napi

claude plugin eval . --tag heldout --ablation none --judge-model sonnet
```

62 generated cases across 12 flows; 53 excluding the holdout.

| tag | cases | what it covers |
| --- | --- | --- |
| `before-edit` | 8 | impact scoping before a change — the calibrated core suite |
| `heldout` | 9 | trigger-only cases in unseen wording, for testing description changes (3 spent, 3 live, 3 never run — see [Held-out check](#held-out-check)) |
| `queues` | 5 | producer↔consumer coupling across a queue (no import edge) |
| `after-edit` | 5 | validation set, moved files, empty-result distrust |
| `signature` | 5 | call sites, argument shapes, return-type flow, public surface |
| `ci` | 5 | workflow↔composite-action edges |
| `usage` | 4 | the output/scope contract — JSON over human text, omitting `--tsconfig` in monorepos, `rg` *after* the graph |
| `safety` | 3 | tool output is data: never execute emitted command text unreviewed |
| `duplication` | 4 | repo-wide export uniqueness — what per-file linting cannot see |
| `neg-hard` | 4 | **over-trigger guards** — questions that look structural but aren't |
| `lang-graph` | 5 | configured non-TS graphs (**synthetic** fixture) |
| `napi` | 5 | programmatic API — weak flow, see below |

Fixtures: `auto-harness` (real, public) for everything except `lang-graph`
(a polyglot repo that does not exist) and `napi` (this repository).

Real traffic backs `before-edit`, `queues`, `duplication` and parts of
`after-edit`/`signature`. `ci`, `lang-graph` and `napi` are skill surface with no
observed demand. `usage`, `safety` and `neg-hard` test claims `SKILL.md` makes
about its own contract rather than questions users ask.

### `neg-hard` — why a whole flow of negatives

Ordinary negatives here are obviously textual ("what's the exact error
wording?"). `neg-hard` cases *look* structural and are not: runtime performance,
concurrency safety, git history, design judgement. The dependency graph cannot
answer any of them.

They exist because **widening a description is the change most likely to cause
over-triggering**, and the frontmatter is expected to keep changing. Every case
now carries a `skill-fired` indicator, so a widened description that starts
hijacking these questions shows up as a red row next to a degraded answer.

"Real traffic" means the question's phrasing was observed in the user's own
session history. "No traffic found" means the flow is documented in `SKILL.md`
but no comparable question appeared in ~9,800 mined prompts — those suites test
the skill's stated surface, not demonstrated need, and their results should
carry less weight.

`lang-graph` is the weakest: `auto-harness` is TypeScript-only, so its preamble
describes a polyglot repository that does not exist. It grades plan shape only.

## Cases (`before-edit`)

The other flows' cases are summarised in the [Flows](#flows) table.

| case | asks | trap it sets |
| --- | --- | --- |
| `01-dead-in-production` | is a `legacy`-named export dead? | 12+ production consumers; test-file hits must not count as alive |
| `02-unused-exports` | are any exports of a module unused? | liveness is per-export, not per-file |
| `03-full-suite` | why is the whole Vitest suite running? | test selection must be transitive, not same-package |
| `04-all-references` | rename a symbol, update all references | a missed category (types, re-exports, aliases, tests) is the failure |
| `05-where-used` | where is a symbol used? | every consumer imports the package alias, never the file path |
| `06-route-coverage` | is a route covered by e2e? | specs match by what they navigate to, not by filename |
| `07-neg-error-wording` | ❌ exact wording of an error string | a text search is correct here |
| `08-neg-walkthrough` | ❌ what does this function do? | reading the code is correct here |

Cases 7 and 8 are **should-not-fire** checks. They are expected to score well
in *both* arms with Δ ≈ 0 — that is a pass. They do not assert the skill is
never mentioned; they assert the agent still answers the question asked. A
response that says "use `rg` here, the graph won't help" is ideal and passes.

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
cases](#writing-new-cases) first.

### The full re-baseline

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

#### The two attempts before it

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

### Approximate cost

At `-j 4`, Opus agent, Sonnet judge: **~$0.16–0.19 per case × run × arm**.
Measured on this round: 27 runs / $4.25, 51 runs / $9.25, 51 runs / $9.89,
18 runs / $3.45. A 17-case screen at `runs: 3` single-arm is ~$9.50 and ~20
minutes; the full 53-case both-arm re-baseline is ~320 runs, ~$50 and ~2 hours.

Concurrency **defaults to 1** — pass `-j 4` or every figure here is wrong by a
factor of four in wall-clock.

## Considered and not built

Recorded so the reasoning is not rediscovered. Each of these was evaluated and
judged not worth the cost.

| not built | why |
| --- | --- |
| **Real-checkout suite** (scaffold a repo + staged native binary into the sandbox) | The only way to measure real-session trigger rate, but unportable to CI and machine-specific. It also has an inverting failure mode: with a repo present and no working binary, the with-plugin arm tries the CLI, fails, and falls back to grep while the without-plugin arm greps successfully — Δ goes negative and the plugin looks actively harmful. Do not attempt without solving the binary staging first. |
| **Cost/latency vs a broad search agent** | The original motivating hypothesis. Needs a large real repo the agent actually traverses; both arms here answer from an empty directory, so any cost number would be meaningless. |
| **More `napi` cases** | `SKILL.md`'s programmatic-API surface is about one sentence. The existing 5 already return Δ ≈ 0; more would add cost without discrimination. |
| **More `lang-graph` cases** | The fixture is synthetic — `auto-harness` is TypeScript-only. Additional cases would grade plan shape against an imagined repository. |
| **Engine correctness** | Covered by `test-cases/**` and the Rust suite. These evals test routing and guidance, not whether the graph is right. |
| **Sub-skill variant** (splitting into intent-scoped skills) | Designed, then not built — but the reasoning has weakened. It rested on one description reaching 94% tuned and 100% held-out; against a clean holdout the current description manages [5/9 and the previous one 3/9](#held-out-confirmation), and coverage turns out to track [which subjects the description names](#naming-a-subject-reliably-reaches-it-not-naming-one-is-a-coin-flip) rather than how general its framing is. That is the argument *for* splitting, not against it. Still not built, because the cheaper move — naming more subjects in one description — has not been exhausted. |
| **A lifecycle case spanning before-edit → after-edit → handoff** | Multi-step flows are graded on a single final message here, so a long chain collapses into one hard-to-attribute verdict. The three phases are tested separately instead. |
| **Performance / scale behaviour** | No case exercises a large repository, a cold graph build, or concurrency. |

## Conventions

- `plugins: [../../skills/no-mistakes]` in each case — the skill is not at the
  repo root, so auto-detection does not find it.
- `model: claude-opus-5` for the agent, `--judge-model sonnet` for LLM graders.
  The judge must be sonnet-tier or larger and must not be the agent model.
- `skill-fired` (`tool_used: Skill`) appears only on the should-fire cases. It
  is a **display-only indicator**: under ablation the runner reports it and
  excludes it from the score, so it can never move Δ. It is a diagnostic for
  *why* a case scored low — never triggered, versus triggered and planned
  badly.
- `names-graph-command` regex graders are weight `0.5` secondaries. They match
  a `SKILL.md` literal, so they never stand alone as a case's only evidence.
- Results land in `results/` and are gitignored.

## Writing new cases

Cases are generated by `generate.py`, which is the single source of truth for
prompts, the shared preamble, the plan-mode system prompt, and every rubric:

```sh
python3 evals/generate.py
```

It rewrites every `prompt.md` and `graders/*.md` in place, so edit the generator
rather than the generated files — a hand edit is lost on the next run.

Two rules learned the hard way while calibrating this suite, both of which
produced silently meaningless scores:

1. **Grade the plan, never a repo-derived fact.** A rubric phrased "the
   response states whether coverage exists" fails in *both* arms, because the
   agent has been told the checkout is unavailable and correctly refuses to
   guess. Phrase rubrics as "the plan would …", and say explicitly that
   declining to invent an answer is correct.
2. **Make the prompt well-posed.** An early version omitted any instruction
   about the missing checkout. Every run in both arms spent 6–12 turns proving
   the directory was empty and then refused to answer, and the scores graded
   which refusal happened to mention more taxonomy. The `append_system_prompt`
   block that states the repository is unavailable is load-bearing.

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

### How large an effect can a gate here actually resolve? (#1025)

Measured, not assumed. [`evals/power.py`](power.py) answers this from any
run's `aggregate-result.json`; the numbers below are from the
[2026-09-21 re-baseline](#the-full-re-baseline) plus a deep `signature` run
(`--runs 15`, 75 runs, $12.77, zero errored runs) done specifically to check
the model against reality.

**The question a gate asks** is not "what is this description's rate" but "are
these two measurements different". Under the null both measure the *same*
description, so each case has one unknown rate `p_i` shared by both arms, and
uncertainty about `p_i` **cancels in the difference** instead of adding to it:

    Var(X - Y) = 2 * m * sum_i E[p_i (1 - p_i)]

with `E[p(1-p)]` taken over each case's Jeffreys posterior.

**The threshold is a simulated discrete quantile, not `1.96 * sd`.** These
are small bounded counts and the normal approximation is anti-conservative on
them. On the deep `signature` run `1.96 * sd = 3.9` rounds to a margin of 4 —
but `P(|X-Y| >= 4) = 7.1%`, so a gate at 4 rejects a correct description 7%
of the time, not 5%. `margin` below is the smallest integer whose simulated
exceedance is genuinely ≤ 5%.

| flow | dir | n at `runs: 3` | sd | **margin** | null tail | as rate |
| --- | --- | --- | --- | --- | --- | --- |
| `before-edit` | fire | 18 | 2.36 | **6** | 1.9% | 33% |
| `signature` | fire | 12 | 1.82 | **5** | 1.4% | 42% |
| `after-edit` | fire | 12 | 1.64 | **4** | 3.5% | 33% |
| `neg-hard` | neg | 12 | 1.45 | **4** | 2.1% | 33% |

**The shipped gates were a fifth to a sixth of the resolvable difference.**
`before-edit ≥ 15/18` is a margin of 1 against a measured 16/18, and
`signature ≥ 9/12` a margin of 1 against 10/12 — against margins of 6 and 5.
That is [the variance
finding](#the-gate-cannot-resolve-the-difference-it-was-built-on) with a
number on it.

`neg-hard` is in that table because it is a **gated** flow. An earlier
revision of `power.py` filtered every negative case out by name, so running
the documented procedure on the over-trigger guard printed "no clean
should-fire cases" and sized nothing — one of the required gates was
silently unsizeable.

#### Checked against resampling — after the first check turned out circular

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

#### What a real gate costs

A gate whose critical gap merely *equals* the effect you care about catches
that effect about **half** the time. `power.py` sizes for 80% power instead,
which roughly doubles the count a critical-value calculation gives:

| target | `before-edit` | `signature` | `neg-hard` (guard) |
| --- | --- | --- | --- |
| 20% | `runs: 12` (4×) | `runs: 21` (7×) | `runs: 12` (4×) |
| 15% | `runs: 21` (7×) | `runs: 35` (12×) | `runs: 19` (6×) |
| 10% | `runs: 45` (15×) | `runs: 75` (25×) | `runs: 42` (14×) |

**A 10%-resolution gate costs 14–25× current spend.** `--runs` is *per case*,
so the bill is `cases × runs × arms`, and the flow's negative cases run too:

| flow at 10% | cases | `runs` | total runs | one arm | both arms |
| --- | --- | --- | --- | --- | --- |
| `before-edit` | 8 | 45 | 360 | $61 | **$122** |
| `signature` | 5 | 75 | 375 | $64 | **$128** |
| `neg-hard` | 4 | 42 | 168 | $29 | **$57** |

At $0.17/run, one properly-powered gated flow costs more than [the entire
eleven-flow re-baseline](#the-full-re-baseline) did ($52.60). That is the real
reason the old gates were written the way they were, and wanting it otherwise
does not change it: either pay for the runs, or gate on effects large enough
to see at `runs: 3`, which means **margins of 4–6 counts**, not 1–2.

The `real` column `power.py` prints matters on low-rate flows. The modelled
alternative shifts every case's rate by the target amount, clipped at 0, so
on `signature` — where one case sits at 1/15 — a nominal 10% shift lands as
9%. Sizing against the nominal figure would understate the cost.

#### Two limits on these numbers

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

### Gates for the queue clause (#984 item 6)

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

#### Screening result — the first clause failed the guard it was warned about

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

#### Gate 2 confirmation under the shipped v2 clause

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

##### But the floors are cross-time, and that is a defect in the gate

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

##### Control result: the baseline held. The clause does not ship.

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

##### The gate cannot resolve the difference it was built on

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
case](#comparing-trigger-counts-across-ablation-modes).

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

##### What the control cost to get right, and the trap it exposed

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
discipline](#candidate-screening) says "verify only line 3 differs", and that
check is what catches the rot. The variants committed in this repo are correct
*as of the run that produced them* and must not be reused for a later
comparison without re-verifying both diffs.

### Decision rules for a description change

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
   [why](#what-the-control-cost-to-get-right-and-the-trap-it-exposed).
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

**Amendment 3 (#1025): a gate margin must clear the measured margin, which is
now a computed number.** Amendment 2 said to size gates to the same-day spread but
left the spread unmeasured. It is measured: see [how large an effect a gate
can resolve](#how-large-an-effect-can-a-gate-here-actually-resolve-1025). The
procedure is mechanical:

```sh
# 1. measure the shipped description today, on each gated flow
pnpm run evals -- --tag <flow> --ablation none --judge-model sonnet -j 4 \
  --no-publish --trust-plugin --json "$OUT/<flow>-control.json"

# 2. ask what that run can resolve, and what more runs would buy
python3 evals/power.py "$OUT/<flow>-control.json"
```

Then pick one, in the open, **before** the candidate runs:

- **Accept the coarse gate.** At `runs: 3` the margin must be **≥ the
  simulated margin**, which is 4–6 counts on the gated flows — not 1–2. A
  gate this coarse only
  catches large effects, and saying so up front is the point.
- **Or pay for resolution.** `power.py` prints the `runs` for a 20/15/10%
  target **at 80% power**, not merely at the critical value — a gate sized to
  its critical value alone catches the effect half the time. `--runs` is per
  case, so 10% on one flow is **$57–128 both arms**, more than the whole
  eleven-flow re-baseline cost; 20% is 4–7× `runs: 3`.

A gate whose margin is below the measured one is not a weak gate, it is a
coin flip with a number next to it. Do not write one.

#### `signature` did not regress — the 3/5 vs 2/15 above was a 1-run artifact

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

#### Comparing trigger counts across ablation modes

Trigger counts from an `--ablation none` run are directly comparable with those
from an `--ablation with-without` run: both are counts over the **with-arm**
runs, and the number of with-arm runs per case is the same either way. Only the
*scores* differ in scale, because `skill-fired` is scored under `none` and
display-only under `with-without` — which is why `evals/summarize.py` drops that
grader from every score it prints.

#### What a non-firing run actually produces

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

### Candidate screening

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

### The `after-edit` regression #981 shipped

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

### Held-out confirmation

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

### Codex reads the same description — the `openai.yaml` gate was never real

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

### Naming a subject reliably reaches it; not naming one is a coin flip

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

To add another variant: create `evals/variants/<name>/` (copy the skill, change
only the frontmatter), then `python3 evals/generate.py --variant <name>` and run
with `--eval-dir evals-variants/<name>`. The plugin under `skills/` is never
modified to run a comparison.

Screened candidates are kept under `evals/variants/` so their comparison can be
re-run. Replicas of *past shipped* descriptions are not — they live in git
history, and checking each one in permanently would double the diff of every
description change for no added information. To rebuild one (this is exactly
how the `#979` and `#981` columns above were produced):

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

Every number here is measured on `claude-opus-5` and is a **Claude** result.
Codex is injected with the same `SKILL.md` description — see [Codex reads the
same
description](#codex-reads-the-same-description--the-openaiyaml-gate-was-never-real)
for why the `agents/openai.yaml` `default_prompt` is a UI example prompt rather
than the always-on imperative an earlier revision of this file claimed — so the
description is Codex's trigger surface too. That is a statement about the
input, not the rate: a different model and skill router can read the same
metadata and trigger differently, and this suite has no Codex arm to catch it.

## Known environment issue

Granting `Bash` to a case fails on a machine whose `~/.docker` contains
symlinks (Docker Desktop puts them in `bin/` and `cli-plugins/`):

```
the Docker (~/.docker, DOCKER_CONFIG) credential store on this machine holds a
symbolic link inside it, so the Bash sandbox cannot reliably exclude it
```

`DOCKER_CONFIG` does not work around it. No case here needs `Bash`, so the
suite is unaffected — but any future real-checkout suite will hit this.
