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

## 2026-11-26 — receipt case: fixture fix piloted and abandoned

**Run:** a local, uncommitted edit to `rules-queue-exception-enqueue-in-tx-receipt`
in `evals/rules_cases.py`, regenerated, then `--case
rules-queue-exception-enqueue-in-tx-receipt --runs 1 --ablation with-without
--no-scaffold -j 2`. Claude CLI 2.1.332, agent `claude-sonnet-5-5`, judge
`claude-opus-5-5`, 2 attempts, zero errored. Cost `$0.26`: `$0.18` agent and
`$0.07` judge. Report outside git at
`~/.cache/no-mistakes/evals/2026-11-26-receipt/pilot.json`.

The edit added a `src/queues.ts` with `attempts: 10`, exponential backoff,
`removeOnComplete` and `removeOnFail`, and rewrote the comment to say a
redelivery collides only while the first job is queued or retrying. That
closes the gap the [rules control](#2026-11-26--rules-control-postgresql-queue-testing-re-measured)
found: the snippet never showed the queue's options, so its "collides
harmlessly" claim could not be checked.

Both arms failed `leaves-enqueue-inside-tx` again (with 0/1, without 0/1), and
the with arm did not load `queue-rules`. Both found new, real holes. Once
retries run out, `removeOnFail` deletes the job while the provider already has
its 2xx, so with no reconciler the event is lost. And because
`receiveWebhook` enqueues on every delivery, conflicting insert or not,
"commit, then enqueue, return 500 on failure" is already safe. The with arm
said exactly that. The queue-rules exception does not apply to this snippet.

**Abandoned.** Another patch would tune the case until the model agrees, so
the edit was dropped and the arms were not run at 3. The case stays as it
was; the ROADMAP says what a redesign needs.

## 2026-11-26 — candidate B ("or planning how to answer"): passes its gates, not shipped

**Run:** `evals/variants/nm-load-plan/`, set up the same way as candidate A
(`--eval-dir evals-variants/nm-load-plan --ablation none --runs 3 -j 4`, one
invocation per flow, all eleven flows), Claude CLI 2.1.332, agent
`claude-sonnet-5-5`, judge `claude-opus-5-5`, 159 attempts, zero errored and
`partial: false` on every flow. Cost `$26.08`: `$18.03` agent and `$8.05`
judge. Reports outside git at
`~/.cache/no-mistakes/evals/2026-11-26-skill-nm-load-plan/<flow>.json`.

| flow | loaded, control | loaded, A | loaded, B | content-only, control | content-only, B |
| --- | --- | --- | --- | --- | --- |
| `before-edit` | 12/18 | 18/18 | 18/18 | 1.00 | 0.89 |
| `after-edit` | 7/12 | 12/12 | 12/12 | 1.00 | 1.00 |
| `signature` | 0/12 | 12/12 | 12/12 | 1.00 | 1.00 |
| `lang-graph` | 7/12 | 11/12 | 10/12 | 0.92 | 1.00 |
| `usage` | 5/12 | 6/12 | 7/12 | 0.92 | 1.00 |
| `queues` | 0/12 | 3/12 | 8/12 | 1.00 | 1.00 |
| `ci` | 1/12 | 6/12 | 7/12 | 1.00 | 1.00 |
| `duplication` | 0/9 | 0/9 | 3/9 | 0.89 | 0.89 |
| `safety` | 0/6 | 0/6 | 0/6 | 0.83 | 1.00 |
| `napi` | 0/12 | 1/12 | 2/12 | 1.00 | 0.92 |
| **summed should-fire** | 32/117 | 69/117 | **79/117** | | |
| `neg-hard` loaded | 0/12 | 0/12 | 0/12 | | |
| `neg-hard` invented commands | 2/12 | 0/12 | 0/12 | | |
| each flow's negative case | ≤ 0/3 | ≤ 0/3 | ≤ 0/3 | | |

`safety-03-neg-json-flag`, which expects the skill to load, loaded 3/3 in the
control and A and 2/3 in B. G1, G2 and G3 pass. B leads A by 10 on summed
loading, which is inside the tie margin of 16, so **A ships** under the
pre-registered tie rule and B is recorded here only. A's screen and its
holdout are recorded with the description change that ships it.

**Failing attempts read.** B loses content where A does not. On `before-edit`,
`06-route-coverage` failed `states-coverage-verdict` 2/3 (control and A 3/3
passed): both answers lay out the `playwright related` and `tests plan`
commands "to run against the real checkout" and stop without a coverage
verdict. `05-where-used` failed `exhaustive-by-construction` 1/3, and
`03-full-suite` failed `produces-reduced-test-set` 1/3, as A did. The
"planning how to answer" wording seems to push answers further toward a plan
with no conclusion. That is a reason beyond the tie rule not to ship B.

## 2026-11-26 — `testing-rules` config candidate (T1): kept

**Run:** main `24768fa692` plus the T1 gates commit, `python3 evals/generate.py --rules-variant describe`,
then `--eval-dir evals-variants/rules-describe --tag rules --ablation none
--runs 3 -j 4 --no-scaffold`, all 26 rules cases, Claude CLI 2.1.332, agent
`claude-sonnet-5-5`, judge `claude-opus-5-5`, 78 attempts, zero errored and
`partial: false`. Cost `$11.20`: `$7.69` agent and `$3.52` judge. Report
outside git at `~/.cache/no-mistakes/evals/2026-11-26-rules-t1/t1.json`.
Gates: [pre-registered](#2026-11-26--gates-for-the-testing-rules-config-candidate-t1).
Control: the with arm of the
[rules control](#2026-11-26--rules-control-postgresql-queue-testing-re-measured)
and the [`github-actions` and off-topic run](#2026-11-26--github-actions-and-off-topic-guards-measured).

| gate | needed | control | T1 | |
| --- | --- | --- | --- | --- |
| 1. coverage-floor cases load `testing-rules` | ≥ 4/6 | 0/6 | **6/6** | pass |
| 2. `postgresql` loaded | ≥ 14/24 | 20/24 | 23/24 | pass |
| 2. `queue` loaded | ≥ 6/15 | 9/15 | 8/15 | pass |
| 2. `testing` loaded | ≥ 6/18 | 9/18 | 14/18 | pass |
| 2. `github-actions` loaded | ≥ 6/9 | 9/9 | 9/9 | pass |
| 2. summed should-fire loaded | ≥ 38/66 | 47/66 | 54/66 | pass |
| 3. off-topic loaded | 0/12 | 0/12 | 0/12 | pass |
| 4. passed overall, `testing` (read) | — | 16/18 | 15/18 | within noise |
| 4. passed overall, all should-fire (read) | — | 60/66 | 60/66 | same |

The rise is on the targeted cases: both coverage-floor cases went from 0/3 to
3/3 loaded. Of the other `testing` cases, parsed-usage (3/3), error-text (3/3)
and flaky-wait (0/3) held, and real-email fell from 3/3 to 2/3.
`postgresql-write-advisory-lock` went from
0/3 to 3/3 loaded with its description unchanged, the same-day swing the
control entry already shows on that case.

**Failing attempts read.** `testing-review-lowered-coverage-floor` failed once
with the skill loaded, on a 2-1 judge vote: the review flagged the lowered
floor and quoted the rule, then asked to move the unrelated `json-summary`
reporter into its own commit, which the grader counts against it.
`testing-review-real-email-fixture` failed once without loading the skill.
`testing-exception-parsed-usage-text` failed once with it loaded, as in the
control. `queue-write-bounded-sweep` failed once with `queue-rules` loaded;
its description is unchanged. None of these comes from the changed sentence.

**Verdict: kept.** The coverage-floor cases are ceiling cases (3/3 without the
plugin in the control), so this is a loading result only, not evidence that
the rules improve those reviews. The reason to keep it anyway: a coverage
threshold edit is a test-config change the `testing-rules` coverage rule
("MUST NOT lower a coverage threshold … to make a flaky or failing test
pass") is written for, and the shipped description never let it load there.
Shipped in `plugins/no-mistakes-rules/skills/testing-rules/SKILL.md`.

## 2026-11-26 — gates for the `no-mistakes` description candidates A and B

**Pre-registered**, committed before either candidate ran. No run in this entry.

The [tool-skill control](#2026-11-26--tool-skill-control-all-eleven-flows-re-measured)
shows the agent naming the skill and deferring it. Two candidates change only
the `description:` line of `plugins/no-mistakes/skills/no-mistakes/SKILL.md`;
the body and references are byte-identical to the shipped skill.

- **A**, `evals/variants/nm-load-before/`: "Answers questions about how code
  connects:" becomes "Load before answering any question about how code
  connects:". The rules plugin's descriptions use this imperative form.
- **B**, `evals/variants/nm-load-plan/`: A's opening widened to "Load before
  answering, or planning how to answer, any question about how code
  connects:", plus a closing sentence "Lists the exact commands to put in the
  plan." B makes two changes, so a B win cannot say which part did it.

Each runs `--ablation none --runs 3 -j 4` on all eleven flows, one invocation
per flow, compared with the control's with arm (trigger counts are comparable
across ablation modes). Margins are `a|obs` from `power.py`, conditioned on
the realized control.

1. **G1, the target:** summed should-fire loading over the 39 cases reaches
   at least 48/117 (control 32/117, rise margin 16).
2. **G2, no flow drops:** `before-edit` ≥ 8/18, `after-edit` ≥ 4/12,
   `lang-graph` ≥ 4/12, `usage` ≥ 3/12, `ci` ≥ 0/12. The flows at 0 cannot
   drop.
3. **G3, `neg-hard`, both halves:** loading ≤ 4/12 (control 0/12), and
   invented command forms ≤ 6/12 (control 2/12, `power.py --fabrication`).
   Each flow's own negative case stays at or below 2/3 loaded (`before-edit`:
   3/6). `safety-03-neg-json-flag` expects the skill to load and is not a
   guard.
4. **G4, content, read and not gated:** the candidate's with-arm content-only
   score per flow beside the control's with arm. B's gain is judged on this
   column, because its closing sentence can raise `names-graph-command`
   without improving the answer.
5. **Tie rule:** if both pass and B's summed loading is within 16 of A's, A
   ships as the smaller change.
6. **Holdout, last and only for a candidate that passes G1–G3:** `--tag
   heldout`, cases 04–09 (07–09 never run), `--ablation none --runs 3`, shipped
   description and the winner in the same session. The holdout uses the same
   plan-mode, empty-sandbox setup, so it cannot catch a win that comes only
   from that framing.

A candidate that misses G1, G2 or G3 is rejected and recorded here only.

## 2026-11-26 — tool-skill control: all eleven flows re-measured

**Run:** main `24768fa692` plus #4742's reporting scripts; the `no-mistakes`
skill and its cases are main's. Claude CLI 2.1.332, agent `claude-sonnet-5-5`, judge
`claude-opus-5-5`, `runs: 3` per arm, `--ablation with-without -j 4`, one
invocation per flow, 318 attempts, zero errored and `partial: false` on all
eleven. Cost `$49.83`: `$31.36` agent and `$18.46` judge. Reports outside git
at `~/.cache/no-mistakes/evals/2026-11-26-skill-control/<flow>.json`. An
unrelated release `rustc` build pushed host load to about 42 during
`before-edit`; no attempt errored or timed out.

This is the same-session control for the two description candidates above. It
replaces the 2026-11-21 tool-skill numbers in the README.

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

Summed over the 39 should-fire cases the skill loaded on 32/117 with-arm
attempts. `neg-hard` loaded 0/12, and 2/12 of its non-firing answers used an
invented command form (`power.py --fabrication`).

**What the failing attempts say.** Of the with-arm should-fire attempts that
did not load the skill, most still named it: 12/12 on `queues`, 9/12 on
`signature`, 5/5 on `lang-graph`, 4/5 on `after-edit`, 4/7 on `usage`, 3/6 on
`before-edit`. Typical lines: "I'd use the `no-mistakes` skill, which resolves
the import graph", "If I need a precise answer, use the `no-mistakes` skill."
Most open with "I can't see the repo in this environment, so this is the
search I'd run": the cases run in plan mode in an empty sandbox, and the agent
puts loading the skill in the plan for later instead of loading it now. The
description reaches the model and the model defers it. The `safety-01` miss
(with 2/3, without 3/3) answered "You can, but I wouldn't pipe it straight
into `bash` unless you add some safeguards" to piping the planner's output
into a shell; the skill never loaded in that attempt, so it does not reflect
the skill's text.

`names-graph-command` still carries much of the should-fire Δ: the
content-only column is lower than the should-fire column on `before-edit`,
`after-edit` and `lang-graph`.

## 2026-11-26 — gates for the `testing-rules` config candidate (T1)

**Pre-registered**, committed before the candidate ran. No run in this entry.

The two coverage-floor cases (`testing-exception-raised-coverage-floor`,
`testing-review-lowered-coverage-floor`) are `vitest.config.ts` diffs, and the
shipped `testing-rules` description names tests and test helpers but not test
runner config. Neither case loaded `testing-rules` in the control (0/6), and
both pass 3/3 without the plugin, so a gain here is a loading gain on ceiling
cases and cannot show the rules help.

Candidate T1, in `DESCRIBE["testing-rules"]` of `evals/rules_variants.py`,
changes one phrase: "any automated test or test helper" becomes "any automated
test, test helper, or test runner config such as a coverage threshold". Every
other `DESCRIBE` entry equals the shipped description; the entry for
`queue-rules` was reset to the shipped text in this change. The #3379 wording
it held is in `evals/rules_variants.py` at `af27b588f5`, and the
[2026-11-17 entry](#2026-11-17--retain-ceiling-cases-reject-queue-trigger-candidate-3379)
describes it.

Run: `python3 evals/generate.py --rules-variant describe`, then `--eval-dir
evals-variants/rules-describe --tag rules --ablation none --runs 3 -j 4
--no-scaffold`, all 26 rules cases. Control: the with arm of the
[rules control](#2026-11-26--rules-control-postgresql-queue-testing-re-measured)
and the [`github-actions` and off-topic run](#2026-11-26--github-actions-and-off-topic-guards-measured).
Margins from `power.py` on those reports (`a|obs`, conditioned on the realized
control):

1. **Target:** the two coverage-floor cases load `testing-rules` on at least
   4/6 (control 0/6; the null rise tail first drops under 5% at 4).
2. **Hold:** each topic's loading stays at or above control minus its `a|obs`:
   `postgresql` ≥ 14/24, `queue` ≥ 6/15, `testing` ≥ 6/18 (control 9/18;
   the flaky-wait case still expects `testing-rules` in this comparison),
   `github-actions` ≥ 6/9. Summed over the 22 should-fire cases, ≥ 38/66
   (control 47/66, `a|obs` 9).
3. **Guard:** off-topic stays at 0/12. The modeled margin would allow 4; any
   rules skill loading on a question no rule covers is over-triggering, and
   this candidate widens a description.
4. **Content, read not gated:** passed overall on the with arm is reported
   beside the control's. A single-arm candidate has no baseline arm of its own.

Miss any of 1–3 and T1 is rejected and recorded here only.

## 2026-11-26 — rules control: `postgresql`, `queue`, `testing` re-measured

The same-session control for the rules description candidate above. Claude CLI
2.1.332, agent `claude-sonnet-5-5`, judge `claude-opus-5-5`, commit
`af27b588f5` (rules plugin unchanged since), `runs: 3` per arm, `--ablation
with-without -j 4 --no-publish --no-scaffold --trust-plugin --keep-temp`, one
invocation per topic, 114 attempts, zero errored, `partial: false`. Cost:
`postgresql` `$6.29` (`$4.40` agent, `$1.89` judge), `queue` `$4.46` (`$3.00`
agent, `$1.47` judge), `testing` `$4.60` (`$2.99` agent, `$1.61` judge),
`$15.35` in all. Reports at `~/.cache/no-mistakes/evals/2026-11-26-rules-control/`.
No case, grader, or fixture changed.

| topic | loaded | passed (with) | passed when loaded | passed (without) |
| --- | --- | --- | --- | --- |
| `postgresql` | 20/24 | 24/24 | 20/20 | 21/24 |
| `queue` | 9/15 | 11/15 | 8/9 | 6/15 |
| `testing` | 9/18 | 16/18 | 7/9 | 9/18 |

Rows that moved against the 2026-11-26 entries below (same day, earlier):

| Case | Loaded | Passed (with) | Passed (without) | Earlier today |
| --- | --- | --- | --- | --- |
| `postgresql-write-advisory-lock` | 0/3 | 3/3 | 3/3 | loaded 3/3, without 0/3 |
| `postgresql-review-parent-key-update` | 2/3 | 3/3 | 3/3 | loaded 3/3 |
| `postgresql-review-narrow-view-lookup` | 3/3 | 3/3 | 0/3 | without 1/3 |
| `queue-review-entity-only-dedup-id` | 2/3 | 2/3 | 2/3 | loaded 1/3, with 3/3, without 1/3 |
| `queue-review-job-self-overlap` | 1/3 | 3/3 | 3/3 | loaded 2/3 |
| `queue-write-self-requeue` | 3/3 | 3/3 | 1/3 | without 2/3 |
| `testing-exception-parsed-usage-text` | 3/3 | 1/3 | 0/3 | with 2/3 |
| `testing-exception-raised-coverage-floor` | 0/3 | 3/3 | 3/3 | with 2/3 |
| `testing-review-lowered-coverage-floor` | 0/3 | 3/3 | 3/3 | loaded 1/3, without 2/3 |

**Failing attempts read.**

- `queue-exception-enqueue-in-tx-receipt`, 0/6 again. No skill of any kind
  loaded in the with arm. The case's own comment ("a duplicate delivery
  collides harmlessly") is true only if finished jobs are retained, and the
  fixture never shows the queue's options, so the grader asks for a judgment
  the prompt does not give enough to make. A fixture fix was piloted and
  abandoned ([receipt pilot](#2026-11-26--receipt-case-fixture-fix-piloted-and-abandoned)).
- `queue-review-entity-only-dedup-id`: the loaded failure and the unloaded
  one both found the retained-id drop and proposed removing `jobId` or using
  BullMQ's `deduplication` option instead of adding the event to the id. The
  rule says not to key the id by entity alone; the answers did not follow it.
  The grader is right.
- `testing-exception-parsed-usage-text`: both loaded failures kept the
  exact-text assertions but then recommended replacing them with a test
  through the docs generator's parser. The grader fails a recommendation to
  replace them; it is right. This is the rule body under Sonnet, not loading.
- `testing-write-flaky-response-wait` loaded `playwright-rules` on all three
  with-arm attempts and `testing-rules` on none, and all three passed.
  `playwright-rules` holds the response-wait rule; `testing-rules` holds it
  too, plus the no-retries rule the grader also checks, and its description
  says browser tests load `playwright-rules` as well, not instead. The
  `skill-fired` grader (`testing-rules`) is right: this is a `testing-rules`
  loading gap on a browser test, not a grader defect. A first plan to accept
  either skill was dropped because it would count a load that lacks the
  retries rule.

**What it showed.** The with arm passed every `postgresql` case and every
`testing` case but the parsed-usage one. The coverage-floor cases never loaded
`testing-rules` and passed at the ceiling in both arms. `advisory-lock` moved
from 3/3-loaded and 0/3-without earlier today to 0/3 and 3/3: the same day's
draws differ by three on a case, which is the noise a 3-attempt row carries.

## 2026-11-26 — `github-actions` and off-topic guards measured

Closes the two rules rows the docs restructure marked "not re-measured". Claude
CLI 2.1.332, agent `claude-sonnet-5-5`, judge `claude-opus-5-5`, commit
`af27b588f5`, `runs: 3` per arm, `--ablation with-without -j 4 --no-publish
--no-scaffold --trust-plugin --keep-temp`, two invocations (`--case
'rules-*-actions-*'`, `--tag rules-offtopic`), 42 attempts, zero errored,
`partial: false`. Cost: `github-actions` `$2.28` (`$1.54` agent, `$0.75`
judge), off-topic `$1.66` (`$1.17` agent, `$0.49` judge), `$3.94` in all.
Reports at `~/.cache/no-mistakes/evals/2026-11-26-step1/`. No case, grader, or
fixture changed.

| Case | Loaded | Passed (with) | Passed (without) |
| --- | --- | --- | --- |
| `github-actions-exception-runner-reasons` | 3/3 | 3/3 | 3/3 |
| `github-actions-review-path-filter-required` | 3/3 | 3/3 | 3/3 |
| `github-actions-write-persist-credentials` | 3/3 | 3/3 | 1/3 |
| `offtopic-neg-big-o` | 0/3 | 3/3 | 3/3 |
| `offtopic-neg-mutex-semaphore` | 0/3 | 3/3 | 3/3 |
| `offtopic-neg-regex-explain` | 0/3 | 3/3 | 3/3 |
| `offtopic-neg-release-note` | 0/3 | 3/3 | 3/3 |

**Failing attempts read.** The two without-plugin failures on
`write-persist-credentials` wrote `actions/checkout@v4` with no
`persist-credentials: false`; the grader was right. No with-arm attempt failed.

**What it showed.** `github-actions-rules` loads on all three cases (9/9) and
helps on `write-persist-credentials` (3/3 against 1/3). The other two cases are
ceiling cases. No rules skill loaded on any off-topic guard (0/12), which is the
goal, and all twelve answers passed.

## 2026-11-26 — restructure the eval docs

No run. README.md now follows the eval-suite layout (what the suite evaluates,
current results from named runs, what each case checks, how to read the
numbers, what it does not measure, how to run it); AGENTS.md is the short rule
set; the "Conventions" and "Writing new cases" sections moved into AGENTS.md;
"What the measurements support" moved to METHODOLOGY.md; the 2026-11-17
trigger-variant tables and the lock-review and PostgreSQL case narratives left
README.md because this log already holds them. The tool-skill table is the
2026-11-21 run unchanged. `github-actions` and the off-topic guards have no
2026-11-26 run and are marked not re-measured. No case, grader, or fixture
changed.

## 2026-11-26 — queue and testing rules measured (#4187, #4623)

Closes the unmeasured rows left by #3879 and #3963 for the cases they added
(`rules-queue-review-job-self-overlap`, `rules-postgresql-review-parent-key-update`,
and the three `rules-testing` cases). Claude CLI 2.1.332, agent
`claude-sonnet-5-5`, judge `claude-opus-5-5`, commit `7da7f3d663`, `runs: 3`
per arm, `--ablation with-without -j 4 --keep-temp`, one invocation per topic
(`--tag rules-queue`, `--tag rules-testing`), 66 attempts, zero errored,
`partial: false`. Cost: queue `$4.39` (`$2.95` agent, `$1.43` judge), testing
`$4.52` (`$2.95` agent, `$1.56` judge). Reports at
`~/.cache/no-mistakes/evals/parked-2026-11-26/`. Not run: a baseline of the
rule text from before #3879 and #3963, so these rows compare with and without
the plugin only, not old rules against new.

| Case | Loaded | Passed (with) | Passed (without) |
| --- | --- | --- | --- |
| `queue-exception-enqueue-in-tx-receipt` | 0/3 | 0/3 | 0/3 |
| `queue-review-entity-only-dedup-id` | 1/3 | 3/3 | 1/3 |
| `queue-review-job-self-overlap` | 2/3 | 3/3 | 3/3 |
| `queue-write-bounded-sweep` | 3/3 | 3/3 | 0/3 |
| `queue-write-self-requeue` | 3/3 | 3/3 | 2/3 |
| `testing-exception-parsed-usage-text` | 3/3 | 2/3 | 0/3 |
| `testing-exception-raised-coverage-floor` | 0/3 | 2/3 | 3/3 |
| `testing-review-lowered-coverage-floor` | 1/3 | 3/3 | 2/3 |
| `testing-review-real-email-fixture` | 3/3 | 3/3 | 0/3 |
| `testing-write-error-text` | 3/3 | 3/3 | 0/3 |
| `testing-write-flaky-response-wait` | 0/3 | 3/3 | 3/3 |

**Failing attempts read.** (1) `queue-exception-enqueue-in-tx-receipt`: all six
answers (both arms) never loaded the skill and called the enqueue inside the
transaction a defect, as the earlier 2026-11-17 runs also did. Several raise a
real hazard the case text does not close (a commit failing after `add`
succeeds, with `jobId` collisions blocking the redelivery), so the case, not
only the description, needs review before it can judge the rule. (2)
`testing-exception-raised-coverage-floor`: the with-arm failure never loaded
the skill and marked the merge-order concern "blocking", as both arms did;
one attempt of three, noise. (3) `testing-exception-parsed-usage-text`: the
one with-arm failure loaded the skill and told the author the exact option
columns were too tight, the over-flag the case exists to catch; the without arm
did this in 3 of 3. The grader verdicts matched the answers.

**What it showed.** Where the skill loads it helps on `bounded-sweep`,
`real-email-fixture`, `write-error-text`, and `parsed-usage-text`. Loading
is the gap: `testing-write-flaky-response-wait` and
`testing-exception-raised-coverage-floor` loaded 0 of 3, and
`queue-exception-enqueue-in-tx-receipt` 0 of 3. `review-job-self-overlap`
and `review-parent-key-update` are ceiling cases (3/3 without the plugin).

## 2026-11-26 — re-measure the SHOULD NOT wording

Re-ran `rules-postgresql` on the PR #4617 head `41621a7c09`, which has the
SHOULD NOT duplicate-lookup sentence and the narrow-view sentence with its
per-row reason. Claude CLI 2.1.332, agent `claude-sonnet-5-5`, judge
`claude-opus-5-5`, `runs: 3` per arm, `-j 4`. Zero errored attempts, `partial:
false` on both invocations. No case, grader, or fixture changed since the
earlier entry.

- **Updated and without plugin:** `bash scripts/run-evals.sh --tag
  rules-postgresql --runs 3 --ablation with-without -j 4 --no-publish
  --no-scaffold --trust-plugin --keep-temp --max-cost-usd 12` (48 attempts;
  `$4.33` agent + `$1.71` judge = `$6.04`).
- **Base (`origin/main` rules, `2a69aee871`):** the same command with
  `--eval-dir evals-variants/rules-control --ablation none --max-cost-usd 8`,
  control generated from `origin/main`'s `postgresql-rules/SKILL.md` (24
  attempts; `$2.89` agent + `$1.32` judge = `$4.21`).
- Total `$10.25`. Reports are archived outside git at
  `~/.cache/no-mistakes/evals/pr4617-rerun-2026-11-26/`.

| Case | Base fired / content | Updated fired / content | Without plugin |
| --- | --- | --- | --- |
| `postgresql-exception-index-timeout` | 3/3, 3/3 | 3/3, 3/3 | 3/3 |
| `postgresql-exception-view-uses-joined-columns` | 3/3, 3/3 | 3/3, 3/3 | 3/3 |
| `postgresql-review-duplicate-row-lookup` | 3/3, 3/3 | 3/3, 3/3 | 3/3 |
| `postgresql-review-long-identifier` | 3/3, 3/3 | 3/3, 3/3 | 3/3 |
| `postgresql-review-narrow-view-lookup` | 3/3, 1/3 | 3/3, 3/3 | 1/3 |
| `postgresql-review-parent-key-update` | 2/3, 3/3 | 2/3, 3/3 | 3/3 |
| `postgresql-write-advisory-lock` | 3/3, 3/3 | 3/3, 3/3 | 0/3 |
| `postgresql-write-offset-pagination` | 3/3, 3/3 | 3/3, 3/3 | 3/3 |
| Total | 23/24, 22/24 | 23/24, 24/24 | 19/24 |

**Failing attempts read.** The two base and two without-plugin failures on
`review-narrow-view-lookup` approved the query and spent the review on
first-page cursors, indexes, and soft-deleted authors; none mentioned the
view's extra join, so the grader was right. The three without-plugin failures
on `write-advisory-lock` used `pg_try_advisory_lock` (session level) with
cleanup code, which the existing rule bans. The unfired `review-parent-key-update`
attempts (one per arm) passed on content and need no action.

**What it showed.** The new wording scored 24/24 against 22/24 for base and
19/24 without the plugin. The only case that separates the new sentences from
base is `review-narrow-view-lookup` (3/3 vs 1/3 base and 1/3 without); the
earlier run had base at 2/3, so base is 3 of 6 pooled and the difference is
suggestive, not resolved at this sample. `review-duplicate-row-lookup` is a
ceiling case (3/3 without the plugin in both runs), so it supports neither the
duplicate-lookup sentence nor its removal; that sentence rests on the extra
probe per output row.

## 2026-11-26 — Duplicate-lookup rule downgraded to SHOULD NOT

No new run. The duplicate-lookup sentence measured below was later changed from MUST NOT to SHOULD NOT. The only harm it names is another probe per output row, which depends on row count, and a second lookup of the same key can be deliberate. The narrow-view sentence gained the reason that the view's joins and subqueries still run for every output row. Those two sentences were not re-measured here; [the re-run](#2026-11-26--re-measure-the-should-not-wording) did. The table in the next section is the earlier wording. Downgrading a MUST NOT needs no new evidence.

## 2026-11-26 — PostgreSQL duplicate row lookup and narrow view rules

Added two rules to `plugins/no-mistakes-rules/skills/postgresql-rules/SKILL.md`
(`## Queries`) — MUST NOT look up the same row more than once in one statement
through separate joins or scalar subqueries keyed by the same expression, and
SHOULD NOT join or subquery a view containing joins or subqueries only to read
one or two base-table columns — plus three synthetic cases in
`evals/rules_cases.py`: `rules-postgresql-review-duplicate-row-lookup`,
`rules-postgresql-review-narrow-view-lookup`, and
`rules-postgresql-exception-view-uses-joined-columns`.

**What ran.** Base commit `ca2d93697c` (`origin/main`), Claude CLI 2.1.332,
agent `claude-sonnet-5-5`, judge `claude-opus-5-5`. After 1-attempt pilots on
the three new cases (`--runs 1 --ablation with-without`, 6 attempts, `$0.54`
agent cost, `partial: false`), ran two `--runs 3 -j 4` passes across all eight
`rules-postgresql` cases in the same session, with zero errored runs and
`partial: false` on both:

- **Base (`origin/main` rules):** `bash scripts/run-evals.sh --eval-dir
  evals-variants/rules-control --tag rules-postgresql --runs 3 --ablation none
  -j 4 --no-publish --no-scaffold --trust-plugin` (24 attempts; `$2.81` agent +
  `$1.24` judge = `$4.06`).
- **Updated rules (with and without plugin):** `bash scripts/run-evals.sh
  --tag rules-postgresql --runs 3 --ablation with-without -j 4 --no-publish
  --no-scaffold --trust-plugin` (48 attempts; `$4.51` agent + `$1.91` judge =
  `$6.42`).

Raw JSON and HTML reports are archived outside git at
`~/.cache/no-mistakes/evals/pg-skill-2026-11-26/`.

| Case | Base fired | Base content | Updated fired | Updated content | Without plugin | Δ (updated vs without) |
| --- | --- | --- | --- | --- | --- | --- |
| `postgresql-exception-index-timeout` | 3/3 | 3/3 (1.00) | 3/3 | 3/3 (1.00) | 3/3 (1.00) | +0.00 |
| `postgresql-exception-view-uses-joined-columns` | 3/3 | 3/3 (1.00) | 3/3 | 3/3 (1.00) | 3/3 (1.00) | +0.00 |
| `postgresql-review-duplicate-row-lookup` | 3/3 | 3/3 (1.00) | 3/3 | 3/3 (1.00) | 3/3 (1.00) | +0.00 |
| `postgresql-review-long-identifier` | 3/3 | 3/3 (1.00) | 3/3 | 3/3 (1.00) | 3/3 (1.00) | +0.00 |
| `postgresql-review-narrow-view-lookup` | 3/3 | 2/3 (0.67) | 3/3 | 3/3 (1.00) | 1/3 (0.33) | +0.67 |
| `postgresql-review-parent-key-update` | 3/3 | 3/3 (1.00) | 2/3 | 3/3 (1.00) | 3/3 (1.00) | +0.00 |
| `postgresql-write-advisory-lock` | 3/3 | 3/3 (1.00) | 3/3 | 3/3 (1.00) | 1/3 (0.33) | +0.67 |
| `postgresql-write-offset-pagination` | 3/3 | 3/3 (1.00) | 3/3 | 3/3 (1.00) | 3/3 (1.00) | +0.00 |

**What the run showed.** With the updated rules, every `rules-postgresql` case
scored `3/3` (`24/24` content pass rate, zero regressions) and triggered
`23/24` (`96%`; one `review-parent-key-update` run skipped `Skill` and still
passed). On `review-narrow-view-lookup`, the failing base attempt and both
failing without-plugin attempts approved the query without flagging the
composite view join for a single base-table column, while all three updated
runs flagged it and recommended joining `users` with `deleted_at IS NULL`
(`3/3` vs `2/3` base and `1/3` without; at `n = 3`, a 1-count difference is
within small-sample variation). `review-duplicate-row-lookup` and
`exception-view-uses-joined-columns` are ceiling cases (`3/3` in all arms),
confirming trigger firing and no over-flagging on legitimate multi-column view
reads.

## 2026-11-24 — lock review cases (#3879)

Added synthetic review cases for a child insert followed by a parent key
update and for removing a job lease based on per-worker concurrency and
schedule cadence. Both cases grade the failure mechanism and a repair.

On base `3e54e9c50a9cc74f51e74175bd881a53bbea46e5`, Claude CLI 2.1.325
attempted one PostgreSQL pilot with the base rules and one with the updated
rules. The configured agent was `claude-sonnet-5-5` and the judge was
`claude-opus-5-5`. Both reports were partial with `auth_failed`. The agent
reported `Not logged in` and the judge reported an expired OAuth session.
There were zero completed attempts and the reports recorded $0. The
PostgreSQL and queue cases have no model scores.

## 2026-11-24 — Codex rules isolation handles shell arguments

The offline audit distinguishes inline text-processing programs and quoted
operators from file operands, recognizes file redirections with `>&`, and
recognizes plain command-prefix assignments. It can change which Codex
attempts are eligible for scoring. Regrade earlier saved reports with the
same audit version before comparing eligibility. Validation used synthetic
transcripts and made zero model calls.

## 2026-11-24 — testing rules for retries and coverage gates

PR #3963 added three cases. They check that an agent fixes a missed Playwright
response instead of adding retries, flags a lower coverage floor that hides
an uncovered path, and accepts a measured, justified increase to that floor.
The existing cases and graders are unchanged. Results are pending, and the
expanded case set has no comparable aggregate score yet.

## 2026-11-21 — re-baseline under the Sonnet agent and Opus judge

Resets the control arm after the 2026-11-14 move from an Opus agent / Sonnet
judge to a Sonnet agent (`claude-sonnet-5-5`) / Opus judge (`claude-opus-5-5`)
([why the old numbers do not carry](METHODOLOGY.md#changing-the-agent-or-judge-model)).
No description, grader or case changed in this entry.

Trigger counts cover should-fire cases only (negative cases excluded), so they
are not out of the 318 attempts.

**What ran.** `d44efa910e`, Claude CLI 2.1.330, one invocation per flow:
`bash scripts/run-evals.sh --tag <flow> --ablation with-without --runs 3 -j 4
--no-publish --max-cost-usd 15 --json <out>`, all eleven routine flows,
318 attempts. Every run's `error` is null, `partial` is false on every flow, no
paid grader was skipped. Reports are preserved outside git at
`~/.cache/no-mistakes/evals/rebaseline-2026-11-21/`. Cost: $28.29 from the
`costUsd` fields ($0.089 per attempt) plus $16.21 in `judgeCostUsd`, $44.50
in all.

| flow | trigger (with-arm) | should-fire Δ | negatives Δ | was (2026-09-21, Opus/Sonnet) |
| --- | --- | --- | --- | --- |
| `before-edit` | 7/18 | +0.08 | +0.00 | 14/18 |
| `after-edit` | 6/12 | +0.08 | +0.00 | 10/12 |
| `lang-graph` (synthetic) | 6/12 | +0.28 | +0.00 | 10/12 |
| `usage` | 4/12 | +0.11 | — | 5/12 |
| `signature` | 0/12 | +0.00 | +0.00 | 7/12 |
| `queues` | 0/12 | +0.00 | +0.00 | 3/12 |
| `ci` | 0/12 | +0.00 | +0.00 | 2/12 |
| `duplication` | 0/9 | −0.07 | +0.00 | 0/9 |
| `safety` | 0/6 | −0.33 | +0.33 | 0/6 |
| `napi` | 0/12 | +0.06 | +0.00 | 0/12 |
| `neg-hard` (guard) | 0/12 | n/a | +0.00 | 0/12 |

Trigger rates fell on seven flows and held at zero on the rest. Agent and judge
both changed, and the skill body changed between the runs, so the drop is not
attributed to any one of them. The negative-case Δ of −0.33 on `ci`, `safety`
and `signature` did not recur.

**Where the cases fired** (with-arm, 3 runs each): `before-edit` 03 and 05 3/3,
01 1/3, the other five 0/3; `after-edit` 02 and 04 3/3; `lang-graph` 01 3/3,
04 2/3, 02 1/3; `usage` 04 3/3, 03 1/3; `safety-03-neg-json-flag` (a negative
case, so outside the `safety` count) 3/3. Nothing else fired.

**Δ by grader.** Per-run score is the weighted pass rate; the table splits the
should-fire Δ between the `names-graph-command` regex (weight 0.5) and
everything else. Across the five flows with a positive Δ (sum +0.61) the regex
carries +0.35 (57%) and content graders +0.26; on `before-edit` and
`after-edit` it is nearly all of Δ.

| flow | should-fire Δ | `names-graph-command` | content graders |
| --- | --- | --- | --- |
| `before-edit` | +0.08 | +0.07 | +0.01 |
| `after-edit` | +0.08 | +0.08 | +0.00 |
| `lang-graph` | +0.28 | +0.17 | +0.11 |
| `usage` | +0.11 | +0.03 | +0.08 |
| `napi` | +0.06 | +0.00 | +0.06 |
| `duplication` | −0.07 | +0.00 | −0.07 |
| `safety` | −0.33 | +0.00 | −0.33 |
| `signature`, `queues`, `ci` | +0.00 | +0.00 | +0.00 |

On `after-edit` the regex is the whole uplift and on `before-edit` nearly all
of it. Content gains with the skill fired in all three with-arm runs are two:
`before-edit-03` `produces-reduced-test-set` 1/3 vs 0/3 and `usage-04`
`paths-is-not-the-parseable-contract` 3/3 vs 0/3. `lang-graph-02`
`covers-both-language-graphs` gained 2/3 vs 0/3 with the skill firing in only 1
of 3 runs (passes in with-arm runs 0 and 2, fired in run 2 only), so it is not
attributable to the skill. `napi-02` (3/3 vs 2/3) has no firing run. Apart from those and the four regression rows below, every
content grader passed at the same rate in both arms, mostly 3/3 each.

**`neg-hard`.** 0/12 fired. Of the 12 non-firing plans, 0 invented a
`no-mistakes` command (7 no mention, 5 named the tool without a command);
`power.py --fabrication` agrees at 0/12. The Opus-era run had 4/12. This
counts commands that are absent from `docs/cli/`; it does not check that a
named command exists with those flags.

**The four `REGRESSION` rows.** Each case is n=3 per arm, and the single
grader behind the drop is read from the saved judge votes.

| case | grader | with / without | with-arm fired | verdict |
| --- | --- | --- | --- | --- |
| `safety-01-no-blind-exec` | `treats-command-text-as-data` | 1/3 / 3/3 | 0/3 | noise |
| `usage-03-dynamic-import-missing` | `explains-static-limit-and-complements` | 1/3 / 2/3 | 1/3 | noise |
| `duplication-02-agent-recreated` | `systematic-not-ad-hoc` | 2/3 / 3/3 | 0/3 | noise |
| `before-edit-01-dead-in-production` | `exhaustive-by-construction` | 1/3 / 2/3 | 1/3 | noise, one mechanism to re-measure |

- **`safety-01`.** The skill did not fire in any with-arm run, so both arms ran
  the same configuration. All six answers advised against a bare
  `planner | bash` and proposed `planner > plan.sh` then
  `bash plan.sh`. The two failing with-arm runs were unanimous FAIL: #1 ran
  `bash -euo pipefail plan.sh` with no validation step, a fair fail; #0 added an
  allowlist `grep` check that the rubric counts as validating, yet failed 0-3.
  The passing with-arm run and all three without-arm runs (one 2-1) used the
  same shape. One-sided Fisher p = 0.2. Judge variance on one rubric, not a
  skill effect. The without-arm 3/3 is the unusual draw.
- **`usage-03`.** The one with-arm run that loaded the skill passed 3-0; the
  two that did not fail, and the without-arm failed 1 of 3 for the same reason
  the unfired with-arm runs did: they diagnosed a missing dynamic import as a
  config problem and never framed text search as a complement to the static
  graph. Non-firing runs overall passed 2/5. p = 0.5.
- **`duplication-02`.** 0/3 fired; the dip is one 1-2 vote split on a
  with-arm run whose plan (git diff, barrel read, `jscpd`, a ts-morph
  script) matches the passing runs. A without-arm run passed on a 2-1 vote.
  p = 0.5.
- **`before-edit-01`.** The other two content graders pass in every run, and the
  `names-graph-command` regex passes only in the one fired run.
  `exhaustive-by-construction`
  failed unanimously in two with-arm runs that told the reader to "use the
  `no-mistakes` skill" without invoking it or naming a command, and in one
  without-arm run (2-1) that relied on grep. A plan that defers to a skill it
  has not loaded fails a rubric that asks for a method which cannot stop a hop
  short; that is a plausible mechanism from two runs and is not established.
  p = 0.5 against the without arm. Worth a targeted `--runs 10` before the next
  description change touches this case, not a finding now.

None of these is a reason to change a grader, and none was loosened.

## 2026-11-17 — trim the rules plugin (#3587)

Removed the Git process skill and the TypeScript object-property deletion
style rule. Moved the six Evidence rules unchanged into observability and
extended that skill's description to cover production claims and measured
results. The control, describe, and entry variants now use the remaining
15 skills. Cases and graders are unchanged. No paid evals were run; the
recorded trigger comparison measures the earlier 16-skill inventory and
has not been re-measured after this change.

## 2026-11-17 — retain ceiling cases; reject queue trigger candidate (#3379)

Decision superseding the issue's case-replacement plan: case selection does
not require the without-plugin arm to fail. Keep the existing cases, omit
scale framing, and investigate skill triggering with unchanged content
graders. The earlier describe-variant replication was already complete.

Base `77413ee9411801db8de00cb94d2cb348d1e30283` (0.109.78), CLI 2.1.325,
Claude Sonnet 5.5 agent / Opus 5.5 judge, concurrency 4, single with-plugin arm,
18 cases × 3 attempts per variant. Control and candidate ran in the same
session; no prompt, grader, rule-body, model, or attempt-depth changes.
The candidate only rewrote the queue description to name enqueueing in
request/event handlers and database transaction handoffs. The wording is
preserved in `rules_variants.py`; the shipped queue description is unchanged.

| Measure | Control | Candidate |
| --- | --- | --- |
| Expected topic skill fired | 36/42 | 33/42 |
| Queue skill fired | 7/12 | 7/12 |
| Off-topic fired | 0/12 | 0/12 |
| Content passes (excluding `skill-fired`) | 51/54 | 48/54 |
| Cost | $5.0975846 | $4.9505255 |

Both reports are complete: no errored attempts or skipped graders. Total
comparison cost $10.0481101 for 108 agent attempts, including judging. Queue
receipt fired/passed 0/3 in each arm; dedup fired 1/3 and passed 3/3 in each.
Queue bounded-sweep and self-requeue fired 3/3 in each. The global trigger
loss is in unchanged PostgreSQL index-timeout/advisory-lock cases (each
3→2) and testing real-email fixture (2→1). Content losses were bounded-sweep
(3→2) and testing real-email fixture (3→1).

Before the candidate, `power.py --runs 3` set the fresh-pair predictive
margins at 3 for the declared queue group and 5 globally. Queue gain zero
fails the improvement threshold (at least +4); global -3 is within modeled
noise. No over-triggering was observed in 12 off-topic attempts per variant.
These are not content-score confidence bounds or proof of non-regression.
Queue's modeled 80%-power depth is 15/25/50 runs for nominal 20%/15%/10%
drops; the model assumes a distributed shift, not a single-case effect.
No deeper run was purchased.

All nine failed content verdicts across both arms were inspected. Six receipt
answers never called `Skill` and recommended moving the enqueue or adding an
outbox; their retry/retention assumptions limit what this rubric proves.
The bounded-sweep failure had enforced caps but tried to finish the promo
backlog in one run. One testing failure raised the domain issue and criticized
the empty-list decoy; another missed the domain issue. Preserve recorded
verdicts and note the ambiguity instead of changing a grader. The candidate
shows no queue gain and is rejected; no production description change ships.

Artifacts (raw reports, exact inputs, case/plugin hashes, copied candidate
traces, failure answers, protocol, and power output):
`~/.cache/no-mistakes/evals/3379-2026-11-17/`.
Control traces were removed by the runner's default cleanup; its final-answer
evidence remains in the JSON report. The candidate used `--keep-temp`.

## 2026-11-17 — abandoned replacement pilot (#3379)

Ten new candidate attempts, plugin-free, one each, Sonnet 5.5 / Opus 5.5,
CLI 2.1.325, cost $0.9766534 including judging. Nine passed; one review failed
on its decoy. The user superseded replacement and scale-framing work before
any candidate case was adopted. No tracked case edits were made. This pilot
is not a control for the trigger comparison and supplies no claimed gain.
Artifacts: `pilot.json`, `pilot/`, and `pilot-suite/` in the archive above.
Including this abandoned pilot, the session spent $11.0247635 on 118 agent
attempts. Run counts and costs were reported before the second `--runs 3`
pass; no additional paid runs followed.

## 2026-11-16 — rules description replications

Historical results moved from README.md. The first replication shipped in
[#3482](https://github.com/jonathanong/no-mistakes/pull/3482); the second in
[#3509](https://github.com/jonathanong/no-mistakes/pull/3509). Raw reports are
unavailable locally, so these are published historical aggregates, not a
same-session control for a new candidate. The second replication's cost was
not published. Its 216 agent attempts used Sonnet 5.5 and Opus 5.5. Descriptions
were trimmed after measurement; the trimmed catalog was not re-measured then.
The replacement-case selection instruction below was superseded on 2026-11-17.

Replication, 2026-11-16: `control` and `describe`, `--runs 6`, single
arm, Sonnet 5.5 agent, Opus 5.5 judge, same day, 17 cases.

| variant | should-fire trigger | off-topic fired | cost |
| --- | --- | --- | --- |
| `control` | 51/78 (65%) | 0/24 | $8.21 |
| `describe` | 58/78 (74%) | 0/24 | $8.85 |

`power.py --runs 6` allows a wrong-way gap of 6 between two fresh arms, so
+7 clears the noise by one count. It agrees with the 2026-11-14 screen
(+6/48). The gain is concentrated in `postgresql-write-anti-join` (2/6 →
6/6) and `postgresql-write-offset-pagination` (1/6 → 6/6), two cases that
already pass without the plugin. `describe` lost on
`testing-exception-parsed-usage-text` (6/6 → 4/6 fired, score 0.83 →
0.33). Re-measure on the replacement cases before changing any description.
`queue-exception-enqueue-in-tx-receipt` fired 0/12 across both variants.

Second replication, 2026-11-16, 14 should-fire cases after the case-bank
updates: `control` 59/84, `describe` 69/84, allowed gap 7, off-topic 0/24 for
both. The plugin then shipped the `describe` descriptions, trimmed where one
claimed more than its skill body covered. `DESCRIBE` matched the shipped
catalog at that point, so the variants were identical until the next
description experiment.

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
cases](AGENTS.md#writing-new-cases) written first.

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
cases](AGENTS.md#writing-new-cases) first.
