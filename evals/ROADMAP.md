# Eval roadmap

Evals worth adding next, and ones already considered and turned down. Current
results are in the [README](README.md); how to gate whatever these measure is
in [METHODOLOGY.md](METHODOLOGY.md).

## Next evals to consider

Ranked by what they would let the suite decide. Proposals only — none is
built. Each new flow starts with a `--runs 1` rubric pilot and is not quoted
until it has a `runs: 3` measurement
([Writing new cases](AGENTS.md#writing-new-cases)).

### 1. Fresh held-out cases — needed before any description change

The holdout is the only number that generalizes, and it has
[not been measured](README.md#current-results) for the current description.
Cases 01–03 are spent, 04–06 were seen by #981's screen, and `after-edit`
became tuning-visible when #985 was screened on it. Cases 07–09 are unspent
but cover only three subjects.

Write a new held-out batch **before** the next candidate is drafted, in the
real register, worded so it reuses no clause of any variant description, and
covering at least one subject the description names (to check reach) and one
it does not (to check generalization). Commit them before the screen runs — a
case read while tuning is no longer held out.

### 2. Skill surface no case exercises

The skill's [quick command table](../plugins/no-mistakes/skills/no-mistakes/SKILL.md) routes to
commands that no prompt or grader mentions today:

| surface | example question | why it may be worth a flow |
| --- | --- | --- |
| `react usages` / `react analyze` | "where is `<Button>` rendered, and does any caller pass `asChild`?" | component callers resolve through JSX, which text search handles badly |
| `fetches` | "which API routes does this page call?" | page↔API coupling has no import edge — the same shape as `queues` |
| `server routes` / tRPC relationships | "what serves `POST /api/orders`, and who calls that procedure?" | route-to-handler is configuration, not an import |
| `tests why` | "why did this test get selected for my change?" | the planner's explain path; the counterpart of `03-full-suite` |
| `tests plan` with a deleted file | "I deleted this test file — what lost coverage?" | `references/impact-recipes.md` has a recipe; nothing measures whether it is reached |
| `playwright tests` / selector coverage | "is anything still asserting on `data-testid=checkout-total`?" | no case asks from the assertion side |
| `lockfile diff` | "which packages actually changed in this lockfile bump?" | named in the decision tree only |
| `postgres catalog` | "how do I give the PostgreSQL rules a schema catalog?" | added to `SKILL.md` in #1139; still no case exercises it after the 2026-11-21 re-baseline |
| `infra resource-refs`, `swift importers` | "what references `aws_s3_bucket.assets`?" | real language/infra graphs, unlike `lang-graph`'s synthetic one |

These are **skill surface, not demonstrated demand**: no mined-prompt evidence
backs them, and the corpus is not in this repository to check. Before building
any of them, search real session history for the phrasing, the way the
existing flows were sourced. A surface with no traffic belongs in a pilot at
most, and its results carry less weight ([how demand is
labelled](README.md#tool-skill-flows)).

Every new should-fire flow also needs a **negative twin** — a question in the
same vocabulary that the graph cannot answer — because widening the
description to reach a new subject is the change most likely to over-trigger.

### 3. Name the zero flows in the description

`duplication`, `safety` and `napi` have now loaded 0 times in three runs: the
2026-11-21 re-baseline, the
[2026-11-26 control](HISTORY.md#2026-11-26--tool-skill-control-all-eleven-flows-re-measured)
(0/9, 0/6, 0/12), and under the "Load before answering" description (0/9,
0/6, 1/12). They are not unlucky, so no further repeat is needed. In the
2026-11-26 runs the agent named the skill and deferred it on `napi`,
`queues` and most `duplication` attempts, and the description names none of
those questions (does a helper already exist, where a job is enqueued or
processed, calling the analyses from Node). A clause per flow is the next
screen. It needs new held-out cases first, because all nine are spent, and
for queues it needs the file-lookup framing that kept the earlier queue
clause off `neg-hard-02` ([HISTORY](HISTORY.md#gates-for-the-queue-clause-984-item-6)).

### 4. Re-measure the negative-case Δ

`ci`, `safety` and `signature` each show Δ −0.33 on one negative case — the
plugin arm scoring *worse* than no plugin on a question the graph cannot
answer. At n=3 that is one run flipping. A deeper run on just those three cases
(`--runs 10`, ~$10 both arms) would settle whether the skill distracts the
agent there. The 2026-11-21 re-baseline (Sonnet agent / Opus judge) read 0.00
on `ci` and `signature` and +0.33 on `safety`, so the −0.33 did not recur at
`runs: 3`; the models changed, so this is not a like-for-like re-measure.

## Learnings from the 2026-11-26 pass

What the [2026-11-26 runs](HISTORY.md#2026-11-26--tool-skill-control-all-eleven-flows-re-measured)
showed about where the next work is.

- **The tool skill was named, then deferred.** Most non-firing should-fire
  attempts in the control named the `no-mistakes` skill in their plan and did
  not load it, because the case runs in plan mode in an empty sandbox. The
  imperative "Load before answering" opening took summed loading from 32/117
  to 69/117 with no negative firing; the remaining misses are concentrated in
  `duplication`, `safety`, `napi` and `queues`, which need their own wording
  or cases.
- **Telling the agent to plan makes answers stop at the plan.** Candidate B's
  "or planning how to answer" loaded more (79/117), but two of its
  three `before-edit-06-route-coverage` answers listed commands and gave no
  coverage verdict, where A and the control gave one 3/3. Wording that names an action
  can move content as well as loading, so read the content column too.
- **Read failing attempts before blaming the description.** Both arms of
  `queue-exception-enqueue-in-tx-receipt` failed for the same, correct reason:
  the snippet never showed the queue's job retention, so the `jobId`
  collision it called harmless could drop a redelivery. The case was wrong, not the skill, and one fixture
  fix exposed the next hole, so the case needs a redesign rather than patches.
- **A loading grader can be right when another skill held the rule.**
  `testing-write-flaky-response-wait` loaded `playwright-rules` and passed,
  but `testing-rules` holds the no-retries rule the case also checks, so the
  `testing-rules` expectation stays.
- **The coverage-floor cases are at the ceiling.** They pass 3/3 without the
  plugin, so a description that makes them load is a loading result only.

## Gaps against the eval-suite conventions

Process gaps, in priority order. Current scores and the runs behind them are
in the [README](README.md) and [HISTORY](HISTORY.md); this list holds no
counts.

1. **Per-case priority and target pass rate.** Add `priority:` to each case
   (critical 80%, high 70%, medium 60%, low 50%) in `generate.py` and
   `rules_cases.py`, so a row of `2/3` can be called pass or fail.
2. **95% intervals in every table.** Add a Wilson interval to `summarize.py`
   output; the README rules table already carries them.
3. **Loading and passing reported apart for every on-demand skill.** The rules
   table does this by hand. Make `summarize.py` print "loaded n/N" and "passed
   when loaded k/n", hold loading to 80% from six or more attempts, and fix low
   loading in the `description:`.
4. **Ceiling cases marked in the tables.** Many rules cases score the same
   without the plugin; mark them, and add a harder variant per topic or run a
   weaker agent model on them.
5. **A baseline arm and an old-rule control for every quoted delta.** Single-arm
   runs (the trigger variants, the Codex runner) have no without-plugin arm,
   and nothing compares a changed rule's old text with its new text for the
   queue and testing rules (#4187 and #4623 ask for it).
6. **Early stopping.** Stop a case once its remaining attempts cannot change its
   verdict against the target, and record the attempts that ran ("2 of 2").
7. **A tracking CSV** (`evals/results.csv`: date, commit, models, case, arm,
   attempts, loaded, passed, cost) that `summarize.py` appends to. Until it
   exists, build the README tables with `python3 evals/readme_table.py` from
   one run's reports.
8. **Rules-plugin loading gaps.** Several cases load the skill in at most one of
   three attempts. Try `describe` variants for those topics at six attempts or
   more per case.
9. **Rows without a current run.** Re-run the groups the README marks "not
   re-measured".
10. **Record abandoned and failed pilots** in HISTORY.md so the next person does
    not repeat them.

Grader weaknesses seen in failing attempts:

- `rules-queue-exception-enqueue-in-tx-receipt` fails in both arms and the
  skill never loads: answers call the enqueue a defect and raise a hazard the
  snippet does not close. Making the snippet airtight did not work
  ([pilot](HISTORY.md#2026-11-26--receipt-case-fixture-fix-piloted-and-abandoned)):
  the snippet enqueues on every delivery, so enqueue-after-commit is already
  safe and the exception never applies. Redesign it so the code enqueues only
  when the insert inserted, and give it a reconciler for exhausted retries, or
  remove the case.
- `rules-testing-exception-raised-coverage-floor`: both arms raise a
  merge-order warning. Grade only whether the answer accepts the higher floor.
- `names-graph-command` records that the skill was used, not that the plan was
  better. Pair it with a content grader per case, or report it on its own line.
- Rules review cases check the finding and the repair in one rubric. Split
  them so a failing attempt says which part failed.
- `skill-fired` on rules cases accepts any `*-rules` skill for off-topic
  guards and only the topic skill otherwise. Record which one a case expects.
- Exception cases pass when the response leaves the code alone, including an
  empty response. Add a check that the response answers the prompt.

## Considered and not built

Recorded so the reasoning is not rediscovered. Each of these was evaluated and
judged not worth the cost.

| not built | why |
| --- | --- |
| **Real-checkout suite** (scaffold a repo + staged native binary into the sandbox) | The only way to measure real-session trigger rate, but unportable to CI and machine-specific. It also has an inverting failure mode: with a repo present and no working binary, the with-plugin arm tries the CLI, fails, and falls back to grep while the without-plugin arm greps successfully — Δ goes negative and the plugin looks actively harmful. Do not attempt without solving the binary staging first. Any `Bash`-enabled case will also hit the [Docker symlink issue](README.md#known-environment-issue). |
| **Cost/latency vs a broad search agent** | The original motivating hypothesis. Needs a large real repo the agent actually traverses; both arms here answer from an empty directory, so any cost number would be meaningless. |
| **More `napi` cases** | `SKILL.md`'s programmatic-API text is now about 20 lines, not one sentence. The existing 5 returned Δ ≈ 0 against the older, shorter text; re-examine this rationale before relying on it to skip more cases. |
| **More `lang-graph` cases** | The fixture is synthetic — `auto-harness` is TypeScript-only. Additional cases would grade plan shape against an imagined repository. |
| **Engine correctness** | Covered by `test-cases/**` and the Rust suite. These evals test routing and guidance, not whether the graph is right. |
| **Sub-skill variant** (splitting into intent-scoped skills) | Designed, then not built — but the reasoning has weakened. It rested on one description reaching 94% tuned and 100% held-out; against a clean holdout #981's description manages [5/9 and #979's 3/9](HISTORY.md#held-out-confirmation), and coverage turns out to track [which subjects the description names](HISTORY.md#naming-a-subject-reliably-reaches-it-not-naming-one-is-a-coin-flip) rather than how general its framing is. That is the argument *for* splitting, not against it. Still not built, because the cheaper move — naming more subjects in one description — has not been exhausted. |
| **A lifecycle case spanning before-edit → after-edit → handoff** | Multi-step flows are graded on a single final message here, so a long chain collapses into one hard-to-attribute verdict. The three phases are tested separately instead. |
| **Performance / scale behaviour** | No case exercises a large repository, a cold graph build, or concurrency. |
