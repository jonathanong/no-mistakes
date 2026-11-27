# `no-mistakes` evals

Two suites share this directory. They answer one question each:

- **Tool skill** (`plugins/no-mistakes/skills/no-mistakes`, 62 cases): does an
  agent reach for the skill on the questions it answers (and stay away from the
  ones it does not), and is the **plan** it writes better for having loaded it?
- **Rules plugin** (`plugins/no-mistakes-rules`, 26 cases): does the topic skill
  load for ordinary work in its area, and does the answer follow the rule
  (write, review, and exception cases) when it does?

Every case runs the agent in an empty sandbox and grades the final answer.
Nothing here measures real-session behavior (see
[what the suite does not measure](#what-this-suite-does-not-measure)).

| doc | read it for |
| --- | --- |
| this README | current results, what each case checks, how to read and run them |
| [HISTORY.md](HISTORY.md) | every measured run and every change to what is measured, newest first |
| [AGENTS.md](AGENTS.md) | the rules for adding, changing, running, and recording evals |
| [METHODOLOGY.md](METHODOLOGY.md) | what a gate can resolve, how to gate a description change, how to build a control or variant |
| [ROADMAP.md](ROADMAP.md) | evals worth adding next, gaps against the eval-suite conventions, and what was considered and not built |

## Current results

Two tables from two runs on the same day. Each names its run. A row with no
run in [HISTORY.md](HISTORY.md) is marked "not re-measured".

### Tool skill (`no-mistakes`)

**Description:** the `description:` line in `plugins/no-mistakes/skills/no-mistakes/SKILL.md`
today. **Measured:** 2026-11-26 on main `24768fa692` plus #4742's reporting
scripts (skill and cases are main's), Claude CLI 2.1.332, agent `claude-sonnet-5-5`, judge
`claude-opus-5-5`, all eleven routine flows, `runs: 3`, `--ablation
with-without -j 4`, one invocation per flow, 318 attempts, zero errored and
`partial: false` on every flow. Cost `$49.83`: `$31.36` agent and `$18.46`
judge. Reports outside git at `~/.cache/no-mistakes/evals/2026-11-26-skill-control/`.
Run entry: [tool-skill control](HISTORY.md#2026-11-26--tool-skill-control-all-eleven-flows-re-measured).
Built with `python3 evals/readme_table.py skill <reports>`. The `heldout` flow
is not measured for this description; cases 07-09 are reserved, unrun, for
the next candidate.

| flow | trigger (with-arm) | should-fire Δ | content-only Δ | negatives Δ |
| --- | --- | --- | --- | --- |
| `before-edit` | 12/18 (44-84%) | +0.23 | +0.15 | +0.00 |
| `after-edit` | 7/12 (32-81%) | +0.11 | +0.00 | +0.00 |
| `signature` | 0/12 (0-24%) | +0.06 | +0.08 | +0.00 |
| `lang-graph` | 7/12 (32-81%) | +0.25 | +0.08 | +0.00 |
| `usage` | 5/12 (19-68%) | +0.31 | +0.33 | — |
| `queues` | 0/12 (0-24%) | +0.11 | +0.17 | +0.00 |
| `ci` | 1/12 (1-35%) | +0.03 | +0.00 | +0.00 |
| `duplication` | 0/9 (0-30%) | +0.00 | +0.00 | +0.00 |
| `safety` | 0/6 (0-39%) | -0.17 | -0.17 | +0.00 |
| `napi` | 0/12 (0-24%) | +0.11 | +0.17 | +0.00 |
| `neg-hard` (guard) | 0/12 | n/a | n/a | +0.00 |

Trigger rates count should-fire cases only (negative cases excluded), so
12/18 and the like are not out of the 318 attempts. Ranges are 95% Wilson
intervals.

Δ is the mean weighted grader score per run, with-skill minus without-skill,
`skill-fired` excluded. "Content-only Δ" also leaves out the
`names-graph-command` regex, which passes when the plan names a command from
`SKILL.md`: it records that the skill was used, not that the plan was better.
On `neg-hard` the skill never fired, and 2 of the 12 non-firing plans used an
invented `no-mistakes` command form (`power.py --fabrication`).

### Rules plugin (`no-mistakes-rules`)

**Measured:** 2026-11-26 on `af27b588f5` (rules plugin and rules cases
unchanged since), Claude CLI 2.1.332, agent `claude-sonnet-5-5`, judge
`claude-opus-5-5`, `runs: 3` per arm, `--ablation with-without -j 4
--no-scaffold`, one invocation per topic (`github-actions` by `--case
'rules-*-actions-*'`), 156 attempts, zero errored and `partial: false` on
all five. Cost `$19.31`: `$13.10` agent and `$6.21` judge. Reports are
outside git at `~/.cache/no-mistakes/evals/2026-11-26-rules-control/` and
`~/.cache/no-mistakes/evals/2026-11-26-step1/`. Run entries:
[rules control](HISTORY.md#2026-11-26--rules-control-postgresql-queue-testing-re-measured),
[`github-actions` and off-topic](HISTORY.md#2026-11-26--github-actions-and-off-topic-guards-measured).
Built with `python3 evals/readme_table.py rules <reports>`.

| topic | loaded (with) | passed overall (with) | passed when loaded | passed (without plugin) |
| --- | --- | --- | --- | --- |
| `github-actions` | 9/9 (70-100%) | 9/9 (70-100%) | 9/9 | 7/9 (45-94%) |
| `off-topic` | 0/12 (0-24%) | 12/12 (76-100%) | 0/0 | 12/12 (76-100%) |
| `postgresql` | 20/24 (64-93%) | 24/24 (86-100%) | 20/20 | 21/24 (69-96%) |
| `queue` | 9/15 (36-80%) | 11/15 (48-89%) | 8/9 | 6/15 (20-64%) |
| `testing` | 9/18 (29-71%) | 16/18 (67-97%) | 7/9 | 9/18 (29-71%) |

| case | loaded | passed (with) | passed when loaded | passed (without) |
| --- | --- | --- | --- | --- |
| `github-actions-exception-runner-reasons` | 3/3 | 3/3 | 3/3 | 3/3 |
| `github-actions-review-path-filter-required` | 3/3 | 3/3 | 3/3 | 3/3 |
| `github-actions-write-persist-credentials` | 3/3 | 3/3 | 3/3 | 1/3 |
| `offtopic-neg-big-o` | 0/3 | 3/3 | 0/0 | 3/3 |
| `offtopic-neg-mutex-semaphore` | 0/3 | 3/3 | 0/0 | 3/3 |
| `offtopic-neg-regex-explain` | 0/3 | 3/3 | 0/0 | 3/3 |
| `offtopic-neg-release-note` | 0/3 | 3/3 | 0/0 | 3/3 |
| `postgresql-exception-index-timeout` | 3/3 | 3/3 | 3/3 | 3/3 |
| `postgresql-exception-view-uses-joined-columns` | 3/3 | 3/3 | 3/3 | 3/3 |
| `postgresql-review-duplicate-row-lookup` | 3/3 | 3/3 | 3/3 | 3/3 |
| `postgresql-review-long-identifier` | 3/3 | 3/3 | 3/3 | 3/3 |
| `postgresql-review-narrow-view-lookup` | 3/3 | 3/3 | 3/3 | 0/3 |
| `postgresql-review-parent-key-update` | 2/3 | 3/3 | 2/2 | 3/3 |
| `postgresql-write-advisory-lock` | 0/3 | 3/3 | 0/0 | 3/3 |
| `postgresql-write-offset-pagination` | 3/3 | 3/3 | 3/3 | 3/3 |
| `queue-exception-enqueue-in-tx-receipt` | 0/3 | 0/3 | 0/0 | 0/3 |
| `queue-review-entity-only-dedup-id` | 2/3 | 2/3 | 1/2 | 2/3 |
| `queue-review-job-self-overlap` | 1/3 | 3/3 | 1/1 | 3/3 |
| `queue-write-bounded-sweep` | 3/3 | 3/3 | 3/3 | 0/3 |
| `queue-write-self-requeue` | 3/3 | 3/3 | 3/3 | 1/3 |
| `testing-exception-parsed-usage-text` | 3/3 | 1/3 | 1/3 | 0/3 |
| `testing-exception-raised-coverage-floor` | 0/3 | 3/3 | 0/0 | 3/3 |
| `testing-review-lowered-coverage-floor` | 0/3 | 3/3 | 0/0 | 3/3 |
| `testing-review-real-email-fixture` | 3/3 | 3/3 | 3/3 | 0/3 |
| `testing-write-error-text` | 3/3 | 3/3 | 3/3 | 0/3 |
| `testing-write-flaky-response-wait` | 0/3 | 3/3 | 0/0 | 3/3 |

Ranges are 95% Wilson intervals. "Passed when loaded" counts attempts that
called the topic skill and passed the content grader, out of those that did.
An attempt that never loaded the skill is still graded on content, and counts
in "passed overall". A case whose without-plugin arm passed 3/3 is a ceiling
case: it shows the skill fires and does no harm, not that it helps.

Readings: the skill helps on `github-actions-write-persist-credentials`,
`postgresql-review-narrow-view-lookup`, `queue-write-bounded-sweep`,
`queue-write-self-requeue`, `testing-review-real-email-fixture` and
`testing-write-error-text` (with arm 3/3 against 0/3 or 1/3 without). Every
other case is at the ceiling except three. `queue-exception-enqueue-in-tx-receipt`
fails in both arms and no skill loads. `queue-review-entity-only-dedup-id`
passes 2/3 in both. `testing-exception-parsed-usage-text` passes 1/3 with and
0/3 without: the loaded failures kept the exact-text assertions and then
advised replacing them. `testing-write-flaky-response-wait` loaded
`playwright-rules`, not `testing-rules`, on all three attempts and passed;
`testing-rules` also holds the no-retries rule the case checks, so its 0/3 is
a real loading gap on browser tests.

## What each case checks

### What the tool-skill cases grade

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
well without the skill. The agent wrote a *good* impact-scoping plan either way
(the without-arm content graders mostly pass), so where the arms
genuinely differ is whether the plan is exhaustive **by construction** or only
**if executed carefully**. The `exhaustive-by-construction` grader on the
recall-critical cases (01, 02, 05) targets that difference; a text-search plan
written to be genuinely exhaustive can and should pass it.

### Tool-skill flows

Every case carries a `tags:` field naming its flow. 88 generated cases across
13 flows; 79 excluding the holdout. The 62 skill cases are the first 12 flows;
the 26 `rules` cases load the `no-mistakes-rules` plugin instead.

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
| `rules` | 26 | no-mistakes-rules plugin: MUST NOT traps in write, review and exception cases, plus off-topic over-trigger guards | skill surface |

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

### Rules-plugin cases

The `rules` flow covers `plugins/no-mistakes-rules`. Definitions live in
`rules_cases.py`; `generate.py` emits each as `rules-<topic>-<slug>/`. Four
topics (`testing`, `postgresql`, `queue`, `github-actions`) with 22 cases
between them (`postgresql` has eight, `testing` has six, `queue` has five, and
`github-actions` has three), plus four off-topic guards. Every case exposes all
installed plugin skills.

- **write** (one or two per topic, slug `write-*`): ordinary work (a function, query,
  test, worker, workflow). The prompt never names the rule; breaking one
  specific rule is the path of least resistance. The rubric checks that one
  thing in the code.
- **review** (`review-*`) gives an inline code sample or diff with a planted
  defect. Passing requires identifying the defect at its location and leaving
  any deliberate decoy alone.
- **exception** (one or two per topic, `exception-*`): same vocabulary, but the code is
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

One line per rules case:

| case | checks |
| --- | --- |
| `testing-write-error-text` | the test asserts an error code, not message text |
| `testing-write-flaky-response-wait` | registers the response wait before the click instead of adding retries |
| `testing-review-real-email-fixture` | flags fixtures that use real domains |
| `testing-review-lowered-coverage-floor` | flags a lower coverage floor that hides an uncovered path |
| `testing-exception-parsed-usage-text` | leaves exact usage text that a docs generator parses |
| `testing-exception-raised-coverage-floor` | accepts a higher floor backed by measured coverage |
| `postgresql-write-offset-pagination` | keyset pagination instead of `OFFSET` |
| `postgresql-write-advisory-lock` | transaction-level advisory lock, not session level |
| `postgresql-review-long-identifier` | flags identifiers that truncate at 63 bytes |
| `postgresql-review-parent-key-update` | traces the implicit parent lock to the child insert and the deadlock, recommends the stronger parent lock first |
| `postgresql-review-duplicate-row-lookup` | flags two scalar subqueries on the same row by `t.assignee_id`, recommends one join, leaves the distinct `reviewer_id` join alone |
| `postgresql-review-narrow-view-lookup` | flags joining a composite view only to read `handle`, recommends joining `users` with `deleted_at IS NULL` |
| `postgresql-exception-index-timeout` | leaves a deliberate statement-timeout raise |
| `postgresql-exception-view-uses-joined-columns` | does not flag a view read that uses its joined columns |
| `queue-write-bounded-sweep` | caps work per run |
| `queue-write-self-requeue` | a no-progress guard on self-requeue |
| `queue-review-entity-only-dedup-id` | flags a dedup id that is only the entity id |
| `queue-review-job-self-overlap` | overlap of scheduled and manual runs across replicas |
| `queue-exception-enqueue-in-tx-receipt` | leaves an enqueue inside the transaction that holds the receipt |
| `github-actions-write-persist-credentials` | checkout drops persisted credentials |
| `github-actions-review-path-filter-required` | flags a path filter on a required check |
| `github-actions-exception-runner-reasons` | leaves runners that have written reasons |
| `offtopic-neg-*` (4) | big-O, mutex versus semaphore, regex explanation, release note: no rules skill should load |

## How to read these numbers

- **Differences of 2-4 counts are not results.** At `runs: 3`, `power.py`
  `a|obs` on the 2026-11-26 control gives the wrong-way gap a gate comparing a
  candidate with that run must tolerate: **4** on `before-edit`; **3** on
  `after-edit` and `lang-graph`; **2** on `usage`; **4** on `neg-hard`
  loading and on its invented command forms. The flows at 0 have no room to
  drop. Summed over the 39 should-fire cases, a rise must reach **16** counts
  to clear noise. Run [`power.py`](power.py) before writing any gate
  ([derivation](METHODOLOGY.md#how-large-an-effect-can-a-gate-here-actually-resolve-1025)).
- **A zero is not certain.** The exact one-sided 95% upper bounds on 0/12
  (`signature`, `queues`, `ci`, `napi`, `neg-hard`), `duplication` 0/9 and
  `safety` 0/6 are 22%, 28% and 39%. A zero rules out a high rate, nothing
  more.
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
- **`safety` reads −0.17** from one with-arm attempt on
  `safety-01-no-blind-exec` that never loaded the skill; it says nothing
  about the skill's text.
- **Every number is Claude's.** Codex reads the same `description:`, but the
  suite has no Codex arm.

- **Rules rows report loading and passing apart.** Unloaded attempts are
  graded on content too and often pass on the model's own judgment, so read
  "passed (without plugin)" as the content baseline and "loaded" as the
  description's job. Fix low loading (on 2026-11-26:
  `queue-exception-enqueue-in-tx-receipt`, the two coverage-floor cases,
  `testing-write-flaky-response-wait`, `postgresql-write-advisory-lock`) in
  the `description:`, not the body.
- **Ceiling cases show the skill fires and does no harm.** A rules case with
  3/3 in the without-plugin arm cannot show the skill helps.
- **Rules rows are 3 attempts per arm.** One attempt is a third of a row, and
  the 95% interval on 3 of 3 runs from 44% to 100%.

**Where Δ comes from.** The content-only column is lower than should-fire
Δ on `before-edit` (+0.15 of +0.23), `after-edit` (+0.00 of +0.11) and
`lang-graph` (+0.08 of +0.25): there, much of the gain is the regex. On
`usage` (loaded 5/12) the content graders carry it. `queues`, `signature` and
`napi` show content gains with the skill never loaded in the with arm, so
those are noise between two arms that ran the same way. The
per-grader table for the 2026-11-21 run is in
[HISTORY.md](HISTORY.md#2026-11-21--re-baseline-under-the-sonnet-agent-and-opus-judge).

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
- **Rules content on real code.** The rules cases are synthetic snippets at 3
  attempts. A pass shows one answer followed one rule on one snippet.
- **Loading in a real session.** Rules cases install every plugin skill in an
  empty sandbox; real sessions carry more skills and more context.

## How to run it

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
python3 evals/readme_table.py skill "$OUT"/<flow>.json...   # README tool-skill rows
python3 evals/readme_table.py rules "$OUT"/<topic>.json...  # README rules tables
```

`summarize.py` prints agent and judge cost apart, and a `content` column: Δ
with `names-graph-command` left out as well as `skill-fired`. That regex only
records that the plan named a command, so `content` is the part of Δ a content
grader carries. Build the README tables with `readme_table.py` rather than by
hand; it refuses partial and errored runs.

The plugin arm must never score below the baseline arm. `summarize.py` prints a
`REGRESSION` row for each such case with its with-arm fire count and exits 1.
Explain every row before merging: an unfired loss points at the description or
a grader, a fired one at the skill's content.

**Cost:** the 2026-11-26 control (318 attempts, Sonnet agent, Opus judge)
reported $31.36 in the per-flow `costUsd` fields, $0.099 per attempt, plus
$18.46 of judge cost in the per-run `judgeCostUsd` fields: $49.83 and $0.157
per attempt together. Plan with ~$0.16 per case × run × arm at `-j 4`, judge
included (2026-11-14 smoke: 8 `before-edit` cases, `--runs 1`, both arms, 16
runs, $2.38); budget `--max-cost-usd` against the agent figure unless you have
checked whether the cap counts judge cost. A 17-case single-arm screen is ~$7;
the full 53-case both-arm re-baseline is ~320 runs and ~$50. Opus-agent runs
cost $0.16-0.19. A total far below the estimate means runs errored — see
[METHODOLOGY](METHODOLOGY.md#running-a-measurement).

### Rules plugin and trigger variants

`python3 evals/generate.py --rules-variant <name>` builds a copy of the plugin
under `evals-variants/_plugins/<name>/` (`rules_variants.py`) and emits the
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

### Codex rules runner

Status: not yet run. No Codex score is recorded in this file or in
[HISTORY.md](HISTORY.md). The Claude results above stay the primary measure.
Codex trigger counts use a different agent, so they are not comparable with
the Claude table.

`python3 evals/codex_rules.py` prints `cases x arms x runs` and exits. It does
not call a model. Dollars are unknown until a paid run; the preview says so.
A full rules pass at the case default is 26 cases x 2 arms x 3 runs = 156
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
as soon as that attempt ends, then updates the aggregate. Resume with the
same inputs and effective run counts. `run-inputs.json` binds the directory to
its plugin contents, agent and judge settings, prompts, and graders. Use a
fresh directory for changed inputs or to run again from a legacy directory
without a fingerprint. `--runs` must be 1 or greater.

Before a paid agent attempt, the runner verifies that only the with arm has
the rules plugin. The primary measure is `skill-fired`: a completed command
must read the skill from the attempt's installed plugin or marketplace copy.
`--regrade` updates trigger and isolation findings from saved transcripts
without model calls, including for legacy reports. The runner reports judge
failures as incomplete grading and retries failed verdicts on resume using
the saved transcript and final answer. The summary leaves affected content
scores blank and exits 1 on a scored with-arm regression.

Both arms use a read-only sandbox with ambient read access. The transcript
audit flags some accesses outside the attempt and to its credential file.
Flagged attempts leave the report incomplete and withhold new judge calls.

Run the credential-free isolation and runner tests locally with the same
command CI uses:

```sh
python3 -m unittest discover -s evals
```

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

### Known environment issue

Granting `Bash` to a case fails on a machine whose `~/.docker` contains
symlinks (Docker Desktop puts them in `bin/` and `cli-plugins/`):

```
the Docker (~/.docker, DOCKER_CONFIG) credential store on this machine holds a
symbolic link inside it, so the Bash sandbox cannot reliably exclude it
```

`DOCKER_CONFIG` does not work around it. No case here needs `Bash`, so the
suite is unaffected — but any future real-checkout suite will hit this.
