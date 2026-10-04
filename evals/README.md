# `no-mistakes` skill evals

Eval suite for the [`skills/no-mistakes`](../skills/no-mistakes/SKILL.md)
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

**Description:** the `description:` line in `skills/no-mistakes/SKILL.md`
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
well without the skill. Opus writes a *good* impact-scoping plan either way —
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
pnpm run evals -- --tag before-edit --ablation with-without --judge-model sonnet -j 4

# Every routine flow — everything EXCEPT the holdout, which only means
# something while it stays unseen.
pnpm run evals -- --ablation with-without --judge-model sonnet -j 4 \
  --tag before-edit queues after-edit signature ci usage safety duplication \
        neg-hard lang-graph napi

# The holdout: only for a candidate that has passed its gates.
pnpm run evals -- --tag heldout --ablation none --judge-model sonnet -j 4
```

`pnpm run evals` wraps `claude plugin eval .` via
[`scripts/run-evals.sh`](../scripts/run-evals.sh). Prefer it over the raw
command: pnpm forwards its own `--` into the script's argv, and the eval CLI
reads that as end-of-options — it silently discards every flag after it and
launches an unfiltered full-suite run. The wrapper strips the `--` and refuses
to launch unscoped; pass `--all` when an unfiltered run (62 cases × 3 runs × 2
arms = 372 runs) is what you actually want.

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
python3 evals/summarize.py "$OUT"/*.json   # trigger, per-arm score, Δ; refuses errored runs
python3 evals/power.py "$OUT"/<flow>.json  # what this run can resolve
```

**Cost:** ~$0.16–0.19 per case × run × arm at `-j 4`. A 17-case single-arm
screen is ~$9.50 and ~20 minutes; the full 53-case both-arm re-baseline is
~320 runs, ~$50 and ~2 hours. A total far below the estimate means runs
errored — see [METHODOLOGY](METHODOLOGY.md#running-a-measurement).

## Flows

Every case carries a `tags:` field naming its flow. 62 generated cases across
12 flows; 53 excluding the holdout.

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
| `napi` | 5 | programmatic API — `SKILL.md` gives it one sentence, so Δ ≈ 0 is expected | skill surface |
| `heldout` | 9 | trigger-only cases in unseen wording: 01–03 spent, 04–06 live, 07–09 never run | real register |

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

## Conventions

- `plugins: [../../skills/no-mistakes]` in each case — the skill is not at the
  repo root, so auto-detection does not find it.
- `model: claude-opus-5` for the agent, `--judge-model sonnet` for LLM graders.
  The judge must be sonnet-tier or larger and must not be the agent model.
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
