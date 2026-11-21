# `no-mistakes` skill evals

Eval suite for the [`plugins/no-mistakes/skills/no-mistakes`](../plugins/no-mistakes/skills/no-mistakes/SKILL.md)
skill. It measures two things about the skill's `description:` — whether an
agent **reaches for the skill** on the questions it answers (and stays away
from the ones it does not), and whether the **plan** it writes is better for
having loaded it.

| doc | read it for |
| --- | --- |
| this README | current results, what they mean, how to run the suite, how cases are built |
| [METHODOLOGY.md](METHODOLOGY.md) | what a gate can resolve, how to gate a description change, how to build a control or variant |
| [HISTORY.md](HISTORY.md) | how the description and the suite got here — every screen, regression and retraction |
| [ROADMAP.md](ROADMAP.md) | evals worth adding next, and what was considered and not built |

## Current results

> **Opus-era numbers.** Everything in this section was measured with an
> Opus agent and a Sonnet judge. The suite now runs a Sonnet agent
> (`claude-sonnet-5-5`) under an Opus judge (`claude-opus-5-5`), so none of
> these counts, Δs or noise margins carry over: gate only against a control
> arm measured under the current models
> ([why](METHODOLOGY.md#changing-the-agent-or-judge-model)).

**Description:** the `description:` line in `plugins/no-mistakes/skills/no-mistakes/SKILL.md`
today. **Measured:** 2026-09-21, all eleven routine flows, `runs: 3`, both
arms, Opus agent / Sonnet judge, 318 runs, zero errored runs. The `heldout`
flow is not measured for this description; cases 07-09 are reserved, unrun,
for the next candidate.

| flow | trigger (with-arm) | should-fire Δ | negatives Δ |
| --- | --- | --- | --- |
| `before-edit` | **14/18 (78%)** | +0.14 | +0.00 |
| `after-edit` | **10/12 (83%)** | +0.19 | +0.00 |
| `lang-graph` ⚠️ synthetic fixture | **10/12 (83%)** | +0.50 | +0.00 |
| `signature` | **7/12 (58%)** | +0.31 | −0.33 |
| `usage` | **5/12 (42%)** | +0.28 | — |
| `queues` | **3/12 (25%)** | +0.08 | +0.00 |
| `ci` | **2/12 (17%)** | +0.00 | −0.33 |
| `duplication` | **0/9** | +0.00 | +0.00 |
| `safety` | **0/6** | +0.00 | −0.33 |
| `napi` | **0/12** | +0.06 | +0.00 |
| `neg-hard` — over-trigger guard, lower is better | **0/12** | n/a | +0.00 |

Δ is the mean weighted grader score per run, with-skill minus without-skill,
`skill-fired` excluded. On `neg-hard` the skill never fired, but 4 of the 12
non-firing plans invented a `no-mistakes` command that does not exist.

The skill body has changed since this run (the PostgreSQL catalog paragraph in
`SKILL.md`, six lines in `references/playwright.md`); the `description:` line
has not. Build any new control arm from today's skill
([why](METHODOLOGY.md#what-the-control-cost-to-get-right-and-the-trap-it-exposed)).
The skill body and the `names-graph-command` graders changed again in the
drift-sweep PR; the suite was not re-run after that change.

Earlier descriptions, their scores, and how this one was chosen are in
[HISTORY.md](HISTORY.md).

## How to read these numbers

- **Differences under ~4 counts are not results.** At `runs: 3` two runs of
  the *identical* description differ by ≥2 in 44% of pairs. A gate must
  tolerate a wrong-way gap of **4** on `before-edit`, **3** on `signature` and
  `after-edit`, **2** on `neg-hard`
  ([derivation](METHODOLOGY.md#how-large-an-effect-can-a-gate-here-actually-resolve-1025)).
  Run [`power.py`](power.py) before writing any gate.
- **A zero is not certain.** The exact one-sided 95% upper bounds on
  `duplication` 0/9, `safety` 0/6 and `napi` 0/12 are 28%, 39% and 22%. A zero
  rules out a high rate, nothing more.
- **Tuned flows overstate generalization.** The description was written
  against these flows; only a held-out case measures how it generalizes, and
  none is current.
- **Trigger counts are an upper bound on real-session behaviour.** Cases run
  in an empty sandbox, so the agent cannot grep instead — compare counts
  *between descriptions*, never read them as absolute rates
  ([below](#what-this-suite-does-not-measure)).
- **Δ is gated on triggering.** Where the skill rarely fires, Δ ≈ 0 says
  nothing about the skill's guidance. Decompose per grader before claiming
  uplift: if Δ is carried by the `names-graph-command` regex, the honest
  reading is "the skill got used", not "the plan got better".
- **Negative-case Δ of −0.33** (`ci`, `safety`, `signature`) is one run
  flipping on one case at n=3 — a thing to re-measure, not a finding.
- **Every number is Claude's.** Codex reads the same `description:`, but the
  suite has no Codex arm.

## What the measurements support

Rules for changing the description. The evidence for each is in
[HISTORY.md](HISTORY.md).

- **Name the subjects that matter.** A subject the description names is
  reached reliably; one it does not name may or may not be, and nothing
  predicts which.
- **Treat the description as a budget.** Rewording reallocates coverage, so
  measure every flow whose subject a change removes, not only the flows it
  targets.
- **Prefer concrete question forms over general framing.**
- **Gate against a same-day control arm, never a stored number**
  ([procedure](METHODOLOGY.md#gating-a-description-change)).
- **Read `neg-hard` as two numbers:** `skill-fired`, plus invented commands in
  the non-firing runs. A non-firing plan has never named a real subcommand.

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
failure.

The headline number is **Δ** — the with-plugin score minus the without-plugin
score. A high absolute score with Δ ≈ 0 means the model would have done just as
well without the skill. Opus wrote a *good* impact-scoping plan either way —
it reasons about barrels and re-export chains unprompted — so where the arms
genuinely differ is whether the plan is exhaustive **by construction** or only
**if executed carefully**. The `exhaustive-by-construction` grader on the
recall-critical cases (01, 02, 05) targets that difference; a text-search plan
written to be genuinely exhaustive can and should pass it.

## What this suite does NOT measure

- **Trigger rate in a real session.** Cases run in an empty sandbox — no
  checkout, no `no-mistakes` binary, no network — so the prompts say the
  repository is unavailable and ask for a plan. That removes the choice the
  trigger question is about: in a real session the agent *can* grep, and that
  is when it skips the skill. Measuring the real rate needs a checkout and a
  staged native binary; see [ROADMAP](ROADMAP.md#considered-and-not-built) for
  the inverting failure mode that makes this harder than it looks.
- **Whether the CLI returns correct answers.** No case runs `no-mistakes`.
  Engine correctness is covered by `test-cases/**` and the Rust suite.
- **Cost or latency versus an alternative.** Per-run cost is recorded, but
  both arms answer from an empty directory, so it says nothing about the tool
  against a search agent on a real repo.
- **Engine behaviour under load** — no large repository, cold graph build or
  concurrency.

## Running

```sh
# One flow.
pnpm run evals -- --tag before-edit --ablation with-without -j 4

# Targeted: one case-name glob. Prefer this to whole flows while iterating.
# `--case` takes `*` only; `[...]` and `{a,b}` match nothing.
pnpm run evals -- --case '01-*' --runs 1 -j 4

# Every routine flow — everything EXCEPT the holdout, which only means
# something while it stays unseen.
pnpm run evals -- --ablation with-without -j 4 \
  --tag before-edit queues after-edit signature ci usage safety duplication \
        neg-hard lang-graph napi

# The holdout: only for a candidate that has passed its gates.
pnpm run evals -- --tag heldout --ablation none -j 4
```

`pnpm run evals` wraps `claude plugin eval .` via
[`scripts/run-evals.sh`](../scripts/run-evals.sh). Prefer it over the raw
command: pnpm forwards its own `--` into the script's argv, and the eval CLI
reads that as end-of-options — it silently discards every flag after it and
launches an unfiltered full-suite run. The wrapper strips the `--`, refuses
to launch without `--tag` or `--case` (pass `--all` when an unfiltered run is
what you actually want), adds `--judge-model claude-opus-5-5` when no judge is
given, and refuses a judge from the same model family as the agent.

Other flags the numbers depend on:

- `-j 4` — concurrency **defaults to 1**, and every duration here assumes 4.
- `--ablation none` — single arm; enough for trigger counts, which are
  [comparable across modes](METHODOLOGY.md#comparing-trigger-counts-across-ablation-modes).
- `--runs <n>` overrides the per-case `runs: 3`; `--max-cost-usd <n>` is a hard
  ceiling; `--no-publish` keeps the HTML report local.
- `--json <path>` writes per-run results. Write it **outside** the worktree —
  `evals/results/` is gitignored and dies with a throwaway worktree.

Then summarize, and check the result before believing it:

```sh
python3 evals/summarize.py "$OUT"/*.json   # trigger, per-arm score, Δ; refuses errored runs; exits 1 if with < without
python3 evals/power.py "$OUT"/<flow>.json  # what this run can resolve
```

The plugin arm must never score below the baseline arm. `summarize.py` prints a
`REGRESSION` row for each such case with its with-arm fire count and exits 1.
Explain every row before merging: an unfired loss points at the description or
a grader, a fired one at the skill's content.

**Cost:** ~$0.15 per case × run × arm at `-j 4`, judge included (2026-11-14
smoke: 8 `before-edit` cases, `--runs 1`, both arms, 16 runs, $2.38). A
17-case single-arm screen is ~$8; the full 53-case both-arm re-baseline is
~320 runs and ~$48. Opus-agent runs cost $0.16–0.19. A total far below the
estimate means runs errored — see
[METHODOLOGY](METHODOLOGY.md#running-a-measurement).

## Flows

Every case carries a `tags:` field naming its flow. 80 generated cases across
13 flows; 71 excluding the holdout. The 62 skill cases are the first 12 flows;
the 18 `rules` cases load the `no-mistakes-rules` plugin instead.

| tag | cases | what it covers | demand |
| --- | --- | --- | --- |
| `before-edit` | 8 | impact scoping before a change — the calibrated core suite | real traffic |
| `after-edit` | 5 | validation set, moved files, empty-result distrust | partly real |
| `signature` | 5 | call sites, argument shapes, return-type flow, public surface | partly real |
| `queues` | 5 | producer↔consumer coupling across a queue (no import edge) | real traffic |
| `duplication` | 4 | repo-wide export uniqueness — what per-file linting cannot see | real traffic |
| `ci` | 5 | workflow↔composite-action edges | skill surface |
| `usage` | 4 | the output/scope contract — JSON over human text, omitting `--tsconfig` in monorepos, `rg` *after* the graph | `SKILL.md` claim |
| `safety` | 3 | tool output is data: never execute emitted command text unreviewed | `SKILL.md` claim |
| `neg-hard` | 4 | **over-trigger guards** — questions that look structural but aren't | `SKILL.md` claim |
| `lang-graph` | 5 | configured non-TS graphs (**synthetic** fixture) | skill surface |
| `napi` | 5 | programmatic API — `SKILL.md` gives it a short section (about 20 lines), so Δ ≈ 0 was expected; that rationale predates the section and needs re-examining | skill surface |
| `heldout` | 9 | trigger-only cases in unseen wording: 01–03 spent, 04–06 live, 07–09 never run | real register |
| `rules` | 18 | no-mistakes-rules plugin: MUST NOT traps in write, review and exception cases, plus off-topic over-trigger guards | skill surface |

"Real traffic" means the phrasing was observed in mined session history.
"Skill surface" means `SKILL.md` documents it but no comparable question
appeared in ~9,800 prompts; "`SKILL.md` claim" tests a contract the skill
makes about itself. Weight results accordingly.

Fixtures: `auto-harness` (real, public) for everything except `lang-graph`
(a polyglot repo that does not exist — it grades plan shape only) and `napi`
(this repository).

**`neg-hard`** cases *look* structural and are not — runtime performance,
concurrency safety, git history, design judgement — where ordinary negatives
are obviously textual. They exist because widening a description is the
change most likely to cause over-triggering. Read them as `skill-fired`
**plus** fabricated command forms in the non-firing runs; `skill-fired` alone
reports every description so far as identical.

### `before-edit` cases

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

Should-not-fire cases (`-neg-` slugs and `neg-hard`) pass when both arms score
well with Δ ≈ 0. They do not assert the skill is never mentioned; a response
that says "use `rg` here, the graph won't help" is ideal.

## Rules-plugin cases

The `rules` flow covers `plugins/no-mistakes-rules`. Definitions live in
`rules_cases.py`; `generate.py` emits each as `rules-<topic>-<slug>/`. Four
topics (`testing`, `postgresql`, `queue`, `github-actions`) with 14 cases
between them (`testing` and `github-actions` have three, the others four),
plus four off-topic guards. Every case
exposes all installed plugin skills.

- **write** (one or two per topic, slug `write-*`): ordinary work (a function, query,
  test, worker, workflow). The prompt never names the rule; breaking one
  specific rule is the path of least resistance. The rubric checks that one
  thing in the code.
- **review** (1 per topic, `review-*`): an inline 30–50 line diff with one
  planted defect and one decoy that looks wrong but is allowed. Pass needs both:
  the defect is named at its location, and the decoy is not flagged.
- **exception** (1 per topic, `exception-*`): same vocabulary, but the code is
  correct because of a written exception or a reasoned SHOULD NOT. Pass when
  the response leaves it alone. Over-flagging is the failure. The topic skill
  should still fire, so these count as should-fire.
- **off-topic** (`rules-offtopic-neg-*`, tag `rules-offtopic`): questions no
  rules skill covers. `skill-fired` matches any `*-rules` skill, so a green row
  here is over-triggering. The `-neg-` slug makes `summarize.py` count them as
  negatives.

Case selection does not require the without-plugin arm to fail. Keep ceiling
cases: they measure triggering and check that the rules do no harm, although
they cannot establish a content-score gain. Do not replace cases or add scale
pressure just to make an unprimed model fail. Keep the same task pressure in
both arms and retain the content graders as regression checks.

`skill-fired` here checks the specific topic skill among the installed skills, not whether the
plugin fired at all.

### Current trigger comparison

The results below measure the earlier 16-skill inventory. The current plugin
contains 15 skills after #3587; it has not been re-measured. Cases and graders
are unchanged. See [history](HISTORY.md).

Measured 2026-11-17 on `77413ee9411801db8de00cb94d2cb348d1e30283`
(release 0.109.78), Claude plugin eval 2.1.325, Sonnet 5.5 agent and Opus 5.5
judge. One `--runs 3` pass per variant: 54 with-plugin attempts each, 108 total,
$10.0481 ($5.0976 control; $4.9505 candidate). All attempts completed without
errors or skipped graders. These are control/candidate variants, not a
with/without-plugin content comparison.

| Measure | Control | Candidate |
| --- | --- | --- |
| Expected topic skill fired | 36/42 | 33/42 |
| Queue skill fired (primary group) | 7/12 | 7/12 |
| Any rules skill fired off-topic | 0/12 | 0/12 |
| Content grader passed, excluding `skill-fired` | 51/54 | 48/54 |

| Case | Control fired | Candidate fired | Control content | Candidate content |
| --- | --- | --- | --- | --- |
| `github-actions-exception-runner-reasons` | 3/3 | 3/3 | 3/3 | 3/3 |
| `github-actions-review-path-filter-required` | 3/3 | 3/3 | 3/3 | 3/3 |
| `github-actions-write-persist-credentials` | 3/3 | 3/3 | 3/3 | 3/3 |
| `postgresql-exception-index-timeout` | 3/3 | 2/3 | 3/3 | 3/3 |
| `postgresql-review-long-identifier` | 3/3 | 3/3 | 3/3 | 3/3 |
| `postgresql-write-advisory-lock` | 3/3 | 2/3 | 3/3 | 3/3 |
| `postgresql-write-offset-pagination` | 3/3 | 3/3 | 3/3 | 3/3 |
| `queue-exception-enqueue-in-tx-receipt` | 0/3 | 0/3 | 0/3 | 0/3 |
| `queue-review-entity-only-dedup-id` | 1/3 | 1/3 | 3/3 | 3/3 |
| `queue-write-bounded-sweep` | 3/3 | 3/3 | 3/3 | 2/3 |
| `queue-write-self-requeue` | 3/3 | 3/3 | 3/3 | 3/3 |
| `testing-exception-parsed-usage-text` | 3/3 | 3/3 | 3/3 | 3/3 |
| `testing-review-real-email-fixture` | 2/3 | 1/3 | 3/3 | 1/3 |
| `testing-write-error-text` | 3/3 | 3/3 | 3/3 | 3/3 |

The candidate changes only the queue description in `rules_variants.py`.
It leads with enqueueing from request/event handlers and database transaction
handoffs, rather than "background work" and queue library names. This is a
routing hypothesis: the receipt prompt centers on a webhook transaction, and
historical replications repeatedly missed it. The fresh control also missed
two dedup triggers and one testing-fixture trigger. PostgreSQL and GitHub
Actions were perfect in this control; their older misses are not a current
reason to broaden those descriptions. The testing prompt already names a test
file and Vitest, so a single missed trigger does not justify another candidate.

**Decision: reject the candidate; retain the shipped descriptions.** Queue
triggering did not improve, and the receipt case remained 0/3 in both variants.
The `describe` variant retains the rejected wording for reproducibility; it is
not shipped by the plugin. No prompts, graders, or rule bodies changed.

The gate was sized before the candidate: `power.py --runs 3` on the fresh
control gives a same-description predictive margin of 3 queue triggers and 5
global triggers. Queue would need at least 11/12 versus 7/12 to exceed that
simulated variation; the observed gain is zero. The global loss of three is
also within its margin. These are noise checks, not proof of equivalence or
content-score significance. For queue drops, the modeled 80%-power depths
are 15 runs for a nominal 20% drop, 25 for 15%, and 50 for 10%; concentrated
single-case changes can need more. No additional paid depth was run.

Every failed content verdict was inspected. All six receipt answers skipped
`Skill` and proposed after-commit enqueueing or an outbox, contrary to the
exception rubric. They also raised retention and retry concerns whose validity
depends on assumptions absent from the snippet; this does not establish that
routing alone caused the content failures. The candidate's bounded-sweep
answer enforced batch/time caps but aimed to finish the promo backlog in one
run, which conflicts with the rubric's leave-work-for-later wording. One
failed testing review loaded the skill and raised the real-domain issue but
criticized the empty-list decoy; the other missed the domain issue. Retain all
recorded verdicts: ambiguous content grading is a limitation, not a reason to
loosen a grader or relabel a failure.

This suite measures synthetic skill-surface requests, not consumer traffic,
production reliability, or the model's internal reason for skipping a skill.
The current comparison does not re-measure without-plugin scores or claim
that every triggered answer follows every rule.

### Trigger variants

`python3 evals/generate.py --rules-variant <name>` builds a copy of the plugin
under `evals-variants/_plugins/<name>/` (`rules_variants.py`) and emits the 18
rules cases under `evals-variants/rules-<name>/`, pointed at it. Rule bodies
are copied unchanged. Compare trigger counts separately from content scores;
sampling can move either result even when the inputs match.

| variant | change |
| --- | --- |
| `control` | unchanged copy, run the same day as the others |
| `describe` | description overrides from `rules_variants.py`; bodies stay unchanged |
| `entry` | one `coding-rules` skill that indexes the topics; each topic's rules move to `references/<topic>-rules.md`. `skill-fired` expects `coding-rules`. Whether the topic reference was read is in the trace, not a grader: a `tool_used` grader other than `skill-fired` is scored |

Compare `control` and `describe` single-arm (`--ablation none --runs 3`) on the same day:

```bash
python3 evals/generate.py --rules-variant control
bash scripts/run-evals.sh --eval-dir evals-variants/rules-control --tag rules \
  --runs 3 --ablation none --model claude-sonnet-5-5 \
  --judge-model claude-opus-5-5 -j 4 --no-publish --no-scaffold --trust-plugin \
  --keep-temp --max-cost-usd 12 --output-dir evals/results/trigger-control \
  --json evals/results/trigger-control.json
# Size both the global gate and the declared queue group before the candidate.
python3 - <<'PYTHON'
import json
from pathlib import Path
source = Path("evals/results/trigger-control.json")
report = json.loads(source.read_text())
report["derived_from"] = str(source)
report["cases"] = [case for case in report["cases"] if
                   case["name"].startswith(("rules-queue-", "rules-offtopic-neg-"))]
report.pop("aggregates", None)
Path("evals/results/trigger-control-queue-derived.json").write_text(json.dumps(report))
PYTHON
python3 evals/power.py --runs 3 evals/results/trigger-control.json
python3 evals/power.py --runs 3 evals/results/trigger-control-queue-derived.json
# Report actual run count and cost before another --runs 3 pass.
python3 evals/generate.py --rules-variant describe
bash scripts/run-evals.sh --eval-dir evals-variants/rules-describe --tag rules \
  --runs 3 --ablation none --model claude-sonnet-5-5 \
  --judge-model claude-opus-5-5 -j 4 --no-publish --no-scaffold --trust-plugin \
  --keep-temp --max-cost-usd 12 --output-dir evals/results/trigger-candidate \
  --json evals/results/trigger-candidate.json
python3 evals/power.py --runs 3 evals/results/trigger-control.json \
  evals/results/trigger-candidate.json
```

Budget roughly $5 per 54-attempt variant at the measured prices; the $12 cap is
a safety bound, not an estimate. Paid runs require authorization. Preserve the
raw JSON, HTML report, exact variant inputs, and `power.py` output before
cleaning the worktree; `--keep-temp` preserves traces for failure inspection.
The 2026-11-17 artifacts are archived outside the worktree at
`~/.cache/no-mistakes/evals/3379-2026-11-17/`. Only trace files were copied out
of kept temporary agent directories, not agent configuration. Large reports
stay out of git. See [HISTORY.md](HISTORY.md) for prior replications and the
abandoned pilot's separate cost; [AGENTS.md](AGENTS.md) owns the editing rules.

Demand is "skill surface": these traps come from the rules
text, not from mined traffic. `summarize.py --mentions` and
`power.py --fabrication` do not apply, since there are no graph command forms
to fabricate.

## Codex rules runner

Status: not yet run. No Codex score is recorded in this file or in
[HISTORY.md](HISTORY.md). The Claude results above stay the primary measure.
Codex trigger counts use a different agent, so they are not comparable with
the Claude table.

`python3 evals/codex_rules.py` prints `cases x arms x runs` and exits. It does
not call a model. Dollars are unknown until a paid run; the preview says so.
A full rules pass at the case default is 18 cases x 2 arms x 3 runs = 108
attempts. Pilot with one case before that:

```sh
python3 evals/codex_rules.py
python3 evals/codex_rules.py --case rules-testing-write-error-text --runs 1
CODEX_AUTH_FILE=/path/to/spare-auth.json \
  python3 evals/codex_rules.py --run --case rules-testing-write-error-text --runs 1 \
  --output "$HOME/.cache/no-mistakes/evals/codex-rules-pilot"
CODEX_AUTH_FILE=/path/to/spare-auth.json \
  python3 evals/codex_rules.py --run --tag rules --runs 3 \
  --output "$HOME/.cache/no-mistakes/evals/codex-rules"
python3 evals/codex_rules.py --regrade \
  --output "$HOME/.cache/no-mistakes/evals/codex-rules"
```

Read the pilot `transcript.jsonl` before starting the group run.

`--output` is a directory. The runner writes `aggregate-result.json` there, in
the shape `summarize.py` and `power.py` already read (`arms.with` and
`arms.without`). It also writes `attempts/<case>/<arm>/<n>/transcript.jsonl`
as soon as that attempt ends, then updates the aggregate. Run the same command
again to resume; an attempt with `result.json` is skipped. `--runs` must be 1
or greater. Before any paid attempt, `codex plugin list --json` must show
`no-mistakes-rules` installed and enabled on the with arm and no plugins on
the without arm. `skill-fired` on each arm is the primary measure: a finished
command that reads `<skill>-rules/SKILL.md` from that attempt's installed
plugin cache or its marketplace copy, including a relative path after `cd`
and a canonical path after the attempt directory is gone.
`--regrade` recomputes that bit from the saved transcripts and does not call
a model. The pass rate is a regression check: `summarize.py` exits 1 when the
with-arm score is below the without-arm score. A judge failure is recorded on
that grader and leaves `skill-fired` in place.

Authenticate as the ChatGPT Codex account that has spare budget, not a shared
or capped login. Pass `CODEX_AUTH_FILE` (a copy of that account's `auth.json`
stored outside `~/.codex`) or `CODEX_API_KEY`. The runner refuses both
`~/.codex` and `~/.claude`, and it refuses an `--output` directory inside the
repository. Each attempt gets a fresh `CODEX_HOME` and `HOME`.
The with arm installs `plugins/no-mistakes-rules` and no other plugin. The
without arm installs nothing. Model, reasoning effort, sandbox, prompt,
workspace, and timeout are the pins in `codex_rules_env.py`, shared by both
arms. `costUsd` is 0 because the runner does not invent a dollar figure.

Details and the transcript shapes the trigger check accepts are in
[METHODOLOGY.md](METHODOLOGY.md#codex-rules-runner).

## Conventions

- `plugins: [../../plugins/no-mistakes/skills/no-mistakes]` in each case — the skill is not at the
  repo root, so auto-detection does not find it.
- `model: claude-sonnet-5-5` for the agent (`AGENT_MODEL` in `generate.py`),
  `--judge-model claude-opus-5-5` for LLM graders (set by `run-evals.sh`). The
  judge must be sonnet-tier or larger and never the agent's model family.
- `skill-fired` (`tool_used: Skill`) is on every case. Under `with-without` it
  is a **display-only indicator** excluded from the score, so it can never
  move Δ; under `none` it is scored, which is why `summarize.py` drops it
  everywhere. On a should-not-fire case a red indicator is how
  over-triggering becomes visible.
- `names-graph-command` regex graders are weight `0.5` secondaries. They match
  a `SKILL.md` literal, so they never stand alone as a case's only evidence.
- Results land in `results/` and are gitignored.
- Description candidates live in [`variants/`](variants/) as a full copy of
  the skill with only the `description:` line changed; the plugin under
  `skills/` is never modified to run a comparison. See
  [METHODOLOGY](METHODOLOGY.md#adding-or-rebuilding-a-variant).

## Writing new cases

Cases are generated by [`generate.py`](generate.py), the single source of
truth for prompts, the shared preamble, the plan-mode system prompt, and every
rubric:

```sh
python3 evals/generate.py                    # rewrites every case in place
python3 evals/generate.py --variant <name>   # same cases against variants/<name>
```

Edit the generator, never the generated `prompt.md` / `graders/*.md` — a hand
edit is lost on the next run.

Two rules learned the hard way, both of which produced silently meaningless
scores:

1. **Grade the plan, never a repo-derived fact.** A rubric phrased "the
   response states whether coverage exists" fails in *both* arms, because the
   agent has been told the checkout is unavailable and correctly refuses to
   guess. Phrase rubrics as "the plan would …", and say explicitly that
   declining to invent an answer is correct.
2. **Make the prompt well-posed.** An early version omitted any instruction
   about the missing checkout. Every run in both arms spent 6–12 turns proving
   the directory was empty and then refused to answer. The
   `append_system_prompt` block that states the repository is unavailable is
   load-bearing.

Pilot a new flow at `--runs 1` to calibrate its rubrics, then measure it at
`runs: 3` before quoting a number — single-run pilots have produced at least
one phantom result
([history](HISTORY.md#signature-did-not-regress--the-35-vs-215-above-was-a-1-run-artifact)).
New held-out cases must be committed **before** the description they judge is
screened. Candidate flows to add are in [ROADMAP.md](ROADMAP.md).

## Known environment issue

Granting `Bash` to a case fails on a machine whose `~/.docker` contains
symlinks (Docker Desktop puts them in `bin/` and `cli-plugins/`):

```
the Docker (~/.docker, DOCKER_CONFIG) credential store on this machine holds a
symbolic link inside it, so the Bash sandbox cannot reliably exclude it
```

`DOCKER_CONFIG` does not work around it. No case here needs `Bash`, so the
suite is unaffected — but any future real-checkout suite will hit this.
