# `no-mistakes` skill evals

Eval suite for the `skills/no-mistakes` skill. Run one flow with:

```sh
claude plugin eval . --tag before-edit --ablation with-without --judge-model sonnet
```

See [Flows](#flows) for the full-suite command — it deliberately excludes the
`heldout` tag, which only means anything while those cases stay unseen.

Add `--no-publish` to keep the HTML report local. The headline number is **Δ**
— the with-plugin score minus the without-plugin score. A high absolute score
with Δ ≈ 0 means the model would have done just as well without the skill.

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

56 generated cases across 12 flows; 53 excluding the holdout.

| tag | cases | what it covers |
| --- | --- | --- |
| `before-edit` | 8 | impact scoping before a change — the calibrated core suite |
| `heldout` | 3 | trigger-only cases in unseen wording, for testing description changes |
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

## Cases

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
| `signature` | 3/5 | strongest of the new flows; `signature-01-add-param` scored **1.00 vs 0.00**, the largest single gap measured |
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

## Baseline (before-edit flow)

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

The should-not-fire cases were unchanged, so broadening the description did not
degrade text-search questions.

Caveat on how that was measured: at the time of this A/B, cases 07 and 08 carried
no `skill-fired` grader, so the result shows the *answers* did not suffer — not
that the skill stayed silent. Every case now carries the indicator (and the
`neg-hard` flow was added specifically as an over-trigger guard), so a re-run
reports whether a widened description fires on questions it cannot answer.

To add another variant: create `evals/variants/<name>/` (copy the skill, change
only the frontmatter), then `python3 evals/generate.py --variant <name>` and run
with `--eval-dir evals-variants/<name>`. The plugin under `skills/` is never
modified to run a comparison.

Relevant asymmetry to keep in mind when interpreting results: Codex consumes
this skill through `skills/no-mistakes/agents/openai.yaml`, whose
`default_prompt` is an **always-on imperative** to use `no-mistakes`. Claude has
only the description to match against. That difference — not model quality — is
the leading explanation for Codex invoking the tool more reliably.

## Known environment issue

Granting `Bash` to a case fails on a machine whose `~/.docker` contains
symlinks (Docker Desktop puts them in `bin/` and `cli-plugins/`):

```
the Docker (~/.docker, DOCKER_CONFIG) credential store on this machine holds a
symbolic link inside it, so the Bash sandbox cannot reliably exclude it
```

`DOCKER_CONFIG` does not work around it. No case here needs `Bash`, so the
suite is unaffected — but any future real-checkout suite will hit this.
