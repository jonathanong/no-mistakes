# no-mistakes

<!-- cspell:ignore uuidv -->

[![CodSpeed](https://img.shields.io/endpoint?url=https://codspeed.io/badge.json)](https://app.codspeed.io/jonathanong/no-mistakes?utm_source=badge)

`no-mistakes` is about **deterministically not making mistakes**. It tells
coding agents and CI which files, tests, and routes a change can affect, then
enforces the static shapes that keep those answers true and the PostgreSQL
patterns that are expensive to fix later. It parses the current checkout
locally. It does not run the app, call a model, or keep a database.

## Four pillars

### 1. Scope the change

Find what a change can reach by walking one canonical dependency graph (typed
edges for imports, routes, queues, workflows, and more), not by text search.
The traversal is deterministic and crosses apps in a workspace, so a backend
edit surfaces the frontend pages and tests that depend on it.

```sh
no-mistakes dependents packages/api/src/posts.mts --format paths
no-mistakes symbols packages/api/src/posts.mts --mode signature-impact --symbol getPost --format json
no-mistakes impacted-checks packages/api/src/posts.mts --root . --format json
```

See [`dependents`](docs/cli/dependents.md), [`symbols`](docs/cli/symbols.md),
[`impacted-checks`](docs/cli/impacted-checks.md), and
[graph edges](docs/graph-edges.md).

### 2. Select the tests

Run the Playwright and Vitest tests that reach the change instead of every
suite, including tests in other apps that exercise a changed backend route.

```sh
no-mistakes tests plan playwright --changed-file packages/api/src/posts.mts --format json
no-mistakes tests plan vitest --from-git-diff origin/main...HEAD --format json
```

See [`tests plan`](docs/cli/tests-plan.md) and
[`tests why`](docs/cli/tests-why.md).

### 3. Keep the graph honest

Dynamic imports, computed selectors, and ambiguous ids make the graph guess.
Rules ban that code and name a static rewrite. `no-mistakes check` runs the
global, cross-file rules; the ESLint/Oxlint plugin `eslint-plugin-no-mistakes`
runs the per-file ones. For example,
[`playwright-unique-test-ids`](docs/rules/playwright-unique-test-ids.md) fails
when two components share `data-testid="save"`, and
[`no-mistakes/playwright-literals`](docs/eslint-rules/playwright-literals.md)
rejects ``data-pw={`save-${user.id}`}`` so selectors and test ids stay literal
strings the graph can match to tests.

See [rules](docs/rules/README.md), [ESLint rules](docs/eslint-rules/README.md),
and [why](docs/why.md).

### 4. PostgreSQL correctness

Schema and query mistakes are hard to undo in production: the fix is a
migration, a backfill, or a lock-heavy rewrite, and it may not be trivially
fixable afterwards. So bad patterns are banned up front, in schema SQL and in
the queries your code executes. For example,
[`postgres-no-offset`](docs/rules/postgres-no-offset.md) bans `OFFSET`
pagination (use cursor pagination), and
[`postgres-no-test-state-mutation`](docs/rules/postgres-no-test-state-mutation.md)
bans test-executed schema and server-state changes, plus `pg_sleep`, within
explicitly selected tests and helpers.
[`postgres-sql-shape-policy`](docs/rules/postgres-sql-shape-policy-banned-functions.md)
with `banned-function-call` can ban `uuidv7()` calls inside `WHERE`, `JOIN ON`,
and `HAVING` predicates, so the key is compared against a stable bound computed
once per statement.
[`postgres-generated-column-predicates`](docs/rules/postgres-generated-column-predicates.md)
also bans filtering or sorting on a generated column derived from the primary
key (such as `uuid_extract_timestamp(id)`); compare the key column itself. UUIDv7
cast and index-loss coverage is coming in #2652.

See the [PostgreSQL rules](docs/rules/README.md) and
[`postgres`](docs/cli/postgres.md).

## How analysis works

**Recall over precision** governs both `effort` and `unanalyzable`. A missed mistake is worse than a
noisy report: an over-report costs a moment to dismiss or suppress, while a
silent omission cannot be detected. So `no-mistakes` never treats an input it
cannot prove as clean (see [why](docs/why.md#recall-over-precision)).

- `effort` follows from it: `max` spends more analysis to prove more inputs
  (helpers, aliases, builders), shrinking the uncertain set. `low` proves less
  and is cheaper, but does not clear what it skips; skipped shapes become
  unanalyzable instead of passing, which is why `low` can report more.
- `unanalyzable` follows from it: uncertain inputs are always surfaced through
  the policy. `report` (default) fails, `advise` notes them, and `ignore` is an
  explicit, visible opt-out. Definite violations stay findings under every policy.
- When you get an unanalyzable report: rewrite the code to a statically provable
  shape (change the code, not the analyzer), raise `effort`, or suppress it with
  a reason.

Each rule application has two analysis dimensions. Both are described in
[rule applications](docs/configuration/rules.md) and
[configuration](docs/configuration/README.md).

**`effort`** is `low` or `max`. `max` is the default and the most complete
analysis. `low` is opt-in per rule application and only exists for rules that
adopt it: `declared-payload-compatibility`, `github-actions-download-integrity`,
`github-actions-job-timeouts`, `nextjs-no-caching`,
`nextjs-redirect-destinations`, `playwright-coverage`,
`playwright-unique-html-ids`, `playwright-unique-test-ids`,
`postgres-no-mutable-sql-registry`, `postgres-no-offset`,
`postgres-no-unbounded-sql-identifier`, `postgres-require-query-annotation`,
`require-storybook-stories`, and `test-no-unmocked-dynamic-imports`. Resolution
order, highest first: CLI `--effort` / N-API `effort`, then `options.effort`,
then repo-level `effort`, then `max`. `--effort` overrides every rule, so use
it deliberately. If a partner-backed low rule's partner check is missing or out
of scope, it warns and falls back to `max`. A rule outside this list
configured with `effort: low` warns "requested effort low; effective effort
max because this rule has no low implementation" and runs at `max`.

```yaml
effort: max # repo default
rules:
  - rule: postgres-require-query-annotation
    options:
      effort: low # fast literal-only proof for this rule
  - rule: playwright-unique-test-ids # stays at max
```

**`unanalyzable`** is `report`, `advise`, or `ignore`: what to do with inputs
the analyzer cannot soundly resolve (dynamic or unproven shapes). `report`
fails the run and is the default; `advise` emits a non-failing advisory;
`ignore` omits it. Set it repo-wide or per rule with `options.unanalyzable`.
The legacy `unanalyzableSql: fail` alias means `report`. `low` can report more uncertainty than `max` resolves, so compare definite
findings separately.

## How it works

Grep has no resolver. Embeddings are probabilistic and cost a model call.
Persistent indexes go stale across worktrees. ESLint cannot see cross-file
uniqueness. `no-mistakes` builds one in-memory graph from this checkout and
discards it after the request. [Why it exists](docs/why.md) names the agent
mistakes that graph is for.

The canonical graph covers TypeScript/JavaScript, React, Next.js, Playwright,
queues, server routes, GitHub Actions, Terraform/OpenTofu, Swift, .NET, and
explicitly configured Python, Go, Rust, Ruby, PHP, Java, Kotlin, Elixir, and
Dart projects. Results are small, structured, and designed to feed the next
edit or validation step.

For example, consider this dependency chain:

> Backend `getPost(id)` -> Backend GET `/posts/:id` -> Next.js Fetch GET `/posts/:id` -> Next.js Page `/post/[id]` -> Playwright Test on `/post/[id]`

One graph query can expose the whole chain while planning. In CI, a change to
`getPost()` can select the relevant Playwright tests instead of running every
suite. The answer is derived locally from syntax and configuration.

## Install and first queries

```sh
npm install --save-dev no-mistakes eslint-plugin-no-mistakes
```

The npm package installs a matching platform-specific optional package that
contains both the executable and N-API addon. Installation has no downloader
or lifecycle binary build; if optional dependencies are disabled, reinstall
with optional dependencies enabled.

`dependents`, `tests plan vitest`, and `playwright check` work without a
custom config block on a conventional Next.js app with root `app/` and
`*.spec.ts` tests. Express, Hono, Fastify, Koa, and NestJS routes, and
BullMQ / glide-mq queue graphs, also run without config. Configuration adds
custom Playwright roots, wrappers, and multi-app bindings, plus custom queue
factories, Remix and language HTTP routes, language frontends, and
repository rules.

```sh
npx no-mistakes dependents src/utils.mts --format json
npx no-mistakes tests plan vitest --changed-file src/utils.mts --format json
npx no-mistakes tests plan vitest --from-git-diff origin/main...HEAD --format json
```

Use the git-range form in PR CI so the job runs the tests that reach the
diff instead of every suite. For several related reports in one process, call
`analyzeProject()` from the [Node API](docs/node-api.md).

Local development from this repository:

```sh
cargo run -p no-mistakes -- dependents src/utils.mts --format paths
```

## Why AST-based?

Many codebase-intelligence tools build persistent indexes, vector embeddings,
or an LLM layer. Those approaches add cost, operational state, and awkwardness
when several worktrees are active. `no-mistakes` parses the current checkout on
demand in Rust, builds one in-memory graph, and discards it after the request.

There are a few trade-offs with this approach:

1. Dynamic imports, selectors, routes, and queue names cannot always be resolved.
   The bundled rules help keep important relationships static and analyzable.
   That is the intended workflow: when `no-mistakes` cannot soundly follow a
   pattern that has a reasonable static rewrite, a rule points to that rewrite,
   and the code changes to fit the analyzer rather than the analyzer guessing.
2. Some relationships are heuristic. The engine favors recall, so agents should
   confirm exact call sites with `rg` after the graph narrows the file set.
3. On-demand parsing is CPU-intensive. A machine-wide lock prevents competing
   worktrees from running large analyses concurrently, and the async Node API
   can batch related reports into one prepared request.
4. Small modules and direct, static bindings produce the most useful graph.

## Agent Workflows

| Agent question | Use |
| --- | --- |
| What does this file import? | `no-mistakes dependencies <file> --format json` |
| What can this change affect? | `no-mistakes dependents <file> --format paths` |
| Who uses this export? | `no-mistakes dependents <file>#Symbol --format json` |
| What does a signature change affect? | `no-mistakes symbols <file> --mode signature-impact --symbol Symbol --format json` |
| What does this module export/import? | `no-mistakes symbols <file> --include both --format json` |
| Which packages do source files import directly? | `no-mistakes import-usages --root . --filter 'src/**' --format json` |
| Which tests should run? | `no-mistakes tests plan <playwright\|vitest> --changed-file <file> --format json` |
| Why was a test selected? | `no-mistakes tests why <test> --plan plan.json` |
| Which Playwright tests cover this page? | `no-mistakes playwright related <file> --json` |
| Which queue/server files are connected? | `no-mistakes queues related <file> --json`; `no-mistakes server related <file> --json` |
| Are configured repository rules passing? | `no-mistakes check --format json` |

Use `--format json` when an agent will parse the answer, `--format paths` when
the output feeds another shell command, and `--timings` when explaining analysis
cost. For repeated in-process queries, prefer the async Node API so one agent
workflow can avoid subprocess overhead.

### Example recipes

| Goal | Command |
|---|---|
| Check if a named export is still used (static imports) | `no-mistakes dead-exports <file> [NAME...]` |
| Find all Vitest tests covering a component | `no-mistakes tests plan vitest --changed-file <file> --format paths` |
| Select tests from a git range | `no-mistakes tests plan vitest --from-git-diff origin/main...HEAD --format json` |
| Find all Playwright tests covering a route/page | `no-mistakes tests plan playwright --changed-file <file> --format paths` |
| Find direct importers before renaming a module | `no-mistakes dependents <file> --depth 1 --relationship import --relationship workspace --format paths` |
| Count static-import callers of a file | `no-mistakes importers <file>` |

## Documentation

- [Why no-mistakes exists](docs/why.md)
- [Documentation index](docs/README.md)
- [CLI commands](docs/cli/README.md)
- [Node/N-API guide](docs/node-api.md)
- [Configuration](docs/configuration/README.md)
- [Graph edge types](docs/graph-edges.md)
- [Feature parity](docs/feature-parity.md)
- [no-mistakes rules](docs/rules/README.md)
- [ESLint rules](docs/eslint-rules/README.md)
- [Agent guide](docs/agent-guide.md)
- [Packaged agent skill](skills/no-mistakes/SKILL.md)
- [AST analysis behavior](docs/ast-analysis.md)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Support

| Language/Framework/Tool | Status |
| -- | -- |
| TypeScript | Mature |
| Next.js / React / Playwright | Mature |
| `pnpm`, `npm`, `yarn`, `bun` | Supported, primarily tested using `pnpm` |
| `bullmq`, `glide-mq` | Supported, primarily tested for `glide-mq` |
| Vitest / Jest | Supported |
| Queue/server/CI/Terraform/OpenTofu | Supported with explicit configuration |
| Swift / .NET | Shipped, narrower language frontends |
| Python / Go / Rust / Ruby / PHP / Java / Kotlin / Elixir / Dart | Shipped v1 frontends and test plans; configure package or module roots explicitly |

## Design Constraints

- Local and deterministic: no services, databases, remote AI calls, or
  persistent filesystem caches.
- One pass per invocation: discover files once, read each requested source
  once, parse it once per required semantic mode, and reuse shared fact maps
  across checks.
- Programmatic parity: stable CLI capabilities also expose async N-API
  functions for Node callers.
- Explicit configuration: route roots, queue factories, test projects, and
  global fallback behavior are opt-in configuration, not inferred conventions.

## Harness Ecosystem

This is part of the following harness ecosystem:

- [auto-harness](https://github.com/jonathanong/auto-harness) - non-interactive agent CLI orchestration across sandboxes
- [agent-blackboard](https://github.com/jonathanong/agent-blackboard) - session-scoped telemetry for autonomous agents
- [pr-shepherd](https://github.com/jonathanong/pr-shepherd) - autonomous pull request shepherd
- [no-mistakes](https://github.com/jonathanong/no-mistakes) - deterministic AST-based codebase intelligence, test selection, and linting for agents
