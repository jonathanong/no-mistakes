# Eval roadmap

Evals worth adding next, and ones already considered and turned down. Current
results are in the [README](README.md); how to gate whatever these measure is
in [METHODOLOGY.md](METHODOLOGY.md).

## Next evals to consider

Ranked by what they would let the suite decide. Proposals only — none is
built. Each new flow starts with a `--runs 1` rubric pilot and is not quoted
until it has a `runs: 3` measurement
([Writing new cases](README.md#writing-new-cases)).

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

The skill's [quick command table](../skills/no-mistakes/SKILL.md) routes to
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
| `postgres catalog` | "how do I give the PostgreSQL rules a schema catalog?" | added to `SKILL.md` in #1139, after the last re-baseline |
| `infra resource-refs`, `swift importers` | "what references `aws_s3_bucket.assets`?" | real language/infra graphs, unlike `lang-graph`'s synthetic one |

These are **skill surface, not demonstrated demand**: no mined-prompt evidence
backs them, and the corpus is not in this repository to check. Before building
any of them, search real session history for the phrasing, the way the
existing flows were sourced. A surface with no traffic belongs in a pilot at
most, and its results carry less weight ([how demand is
labelled](README.md#flows)).

Every new should-fire flow also needs a **negative twin** — a question in the
same vocabulary that the graph cannot answer — because widening the
description to reach a new subject is the change most likely to over-trigger.

### 3. Repeat the zero flows

`duplication` 0/9, `safety` 0/6 and `napi` 0/12 are first observations with
95% upper bounds of 22–39%. One more `runs: 3` pass each (~$12 single-arm)
would show whether they are structurally dead or unlucky, which decides whether
a clause is worth screening for them.

### 4. Re-measure the negative-case Δ

`ci`, `safety` and `signature` each show Δ −0.33 on one negative case — the
plugin arm scoring *worse* than no plugin on a question the graph cannot
answer. At n=3 that is one run flipping. A deeper run on just those three cases
(`--runs 10`, ~$10 both arms) would settle whether the skill distracts the
agent there.

## Considered and not built

Recorded so the reasoning is not rediscovered. Each of these was evaluated and
judged not worth the cost.

| not built | why |
| --- | --- |
| **Real-checkout suite** (scaffold a repo + staged native binary into the sandbox) | The only way to measure real-session trigger rate, but unportable to CI and machine-specific. It also has an inverting failure mode: with a repo present and no working binary, the with-plugin arm tries the CLI, fails, and falls back to grep while the without-plugin arm greps successfully — Δ goes negative and the plugin looks actively harmful. Do not attempt without solving the binary staging first. Any `Bash`-enabled case will also hit the [Docker symlink issue](README.md#known-environment-issue). |
| **Cost/latency vs a broad search agent** | The original motivating hypothesis. Needs a large real repo the agent actually traverses; both arms here answer from an empty directory, so any cost number would be meaningless. |
| **More `napi` cases** | `SKILL.md`'s programmatic-API surface is about one sentence. The existing 5 already return Δ ≈ 0; more would add cost without discrimination. |
| **More `lang-graph` cases** | The fixture is synthetic — `auto-harness` is TypeScript-only. Additional cases would grade plan shape against an imagined repository. |
| **Engine correctness** | Covered by `test-cases/**` and the Rust suite. These evals test routing and guidance, not whether the graph is right. |
| **Sub-skill variant** (splitting into intent-scoped skills) | Designed, then not built — but the reasoning has weakened. It rested on one description reaching 94% tuned and 100% held-out; against a clean holdout #981's description manages [5/9 and #979's 3/9](HISTORY.md#held-out-confirmation), and coverage turns out to track [which subjects the description names](HISTORY.md#naming-a-subject-reliably-reaches-it-not-naming-one-is-a-coin-flip) rather than how general its framing is. That is the argument *for* splitting, not against it. Still not built, because the cheaper move — naming more subjects in one description — has not been exhausted. |
| **A lifecycle case spanning before-edit → after-edit → handoff** | Multi-step flows are graded on a single final message here, so a long chain collapses into one hard-to-attribute verdict. The three phases are tested separately instead. |
| **Performance / scale behaviour** | No case exercises a large repository, a cold graph build, or concurrency. |
