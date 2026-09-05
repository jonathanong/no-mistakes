# Why no-mistakes exists

Coding agents miss structure that a text search cannot see. They recreate an
API that already exists in another file, ship a Next.js page with no
Playwright coverage, run every suite because they cannot tell which tests
reach a change, or write a dynamic `fetch` / test ID that drops the
relationship from the graph.

`no-mistakes` answers those questions from the current checkout. It parses
source once, builds one in-memory graph, and returns a small structured
result. It does not run the app, call a model, or keep a database.

The model is:

```
facts (one parse) → one in-memory graph → commands and checks project that graph
```

Static literals produce edges. Dynamic values are skipped rather than
guessed. That is why the bundled rules exist: they keep the shapes the graph
can trust.

## Three jobs

1. **Scope the change.** Callers, routes, queues, workflows, and named
   exports — resolved through tsconfig aliases and workspace packages, not
   a string match.
2. **Select the tests.** Vitest, Playwright, and configured native plans
   for the files that actually reach the change, including in PR CI.
3. **Keep the graph honest.** Repository checks and ESLint/Oxlint rules
   stop agents from hiding relationships behind aliases, dynamic URLs, or
   duplicate public names.

Graph queries such as `dependents` and `tests plan vitest` work without a
custom `testPlan` block. Configuration unlocks Playwright coverage, queues,
server routes, language frontends, and repository rules.

## Mistakes this prevents

| If you skip this | You get | Use |
| --- | --- | --- |
| Recreate an export grep found in one file | A second `getCurrentUser` in another path | [`unique-exports`](rules/unique-exports.md), [`dead-exports`](cli/dead-exports.md), [`symbols`](cli/symbols.md) |
| Run every Vitest or Playwright suite | Extra CI minutes and a noisy local loop | [`tests plan`](cli/tests-plan.md), [`playwright related`](cli/playwright-related.md) |
| Ship an App Router page with no test | A route nothing navigates to in CI | [`playwright check`](cli/playwright-check.md), [`playwright-coverage`](rules/playwright-coverage.md) |
| Treat an empty plan as “nothing to run” | Missed suites after a fallback or warning | inspect `warnings` and `fallback_triggered` |
| Dynamic `fetch(\`/api/${id}\`)` or test IDs | Edges the graph cannot emit | [`nextjs-static-fetch-url`](eslint-rules/nextjs-static-fetch-url.md), [`playwright-literals`](eslint-rules/playwright-literals.md) |
| Miss callers behind aliases or workspaces | Edits that compile in one package and break another | [`dependents`](cli/dependents.md) (omit `--tsconfig` in monorepos) |
| Move a file and leave broken imports | A green agent session with unresolved specifiers | [`resolve-check`](cli/resolve-check.md) |
| Change a signature and miss typed call sites | Callers that still type-check until runtime | [`symbols --mode signature-impact`](cli/symbols.md), [`call-sites`](cli/call-sites.md) |
| Rename a queue job and miss the worker | Producers with no processor | [`queues check`](cli/queues-check.md), [`queues related`](cli/queues-related.md) |
| Hoist a uniqueness token above re-entrant `beforeAll` | Duplicate-key failures that depend on the worker | [`playwright-no-hoisted-unique-token`](eslint-rules/playwright-no-hoisted-unique-token.md) |
| One raw `scrollTo` racing cursor pagination | A wait that burns its timeout | [`playwright-no-raw-scroll-pagination`](eslint-rules/playwright-no-raw-scroll-pagination.md) |
| Alias a function or rename an export | A public name the graph cannot follow | [`ts-no-function-aliases`](eslint-rules/ts-no-function-aliases.md), [`ts-no-export-renaming`](eslint-rules/ts-no-export-renaming.md) |
| Parse human CLI output or `eval` command text | Broken paths and unsafe shell | `--format json` and JSON `command` arrays |
| Shell out once per question | Repeated discovery, parse, and graph builds | `analyzeProject()` in the [Node API](node-api.md) |
| Infer route or queue roots from folklore | Edges that appear in one repo layout and vanish in another | explicit [`.no-mistakes.yml`](configuration/README.md) |

CI and humans hit a shorter list of the same failures: paying for full-suite
PR CI, trusting embeddings to pick tests, and expecting ESLint to catch
cross-file uniqueness.

## What it will not do

- Guess dynamic imports, route paths, fetch URLs, queue names, or selectors.
- Replace `rg` for exact call lines, comments, or strings.
- Walk `importers` over non-TS/JS graphs (`importers` is the fast TS/JS
  reverse static-import scan). Use `dependents --relationship <lang>`.
- Parse non-npm lockfiles in `lockfile diff`. SwiftPM and NuGet pins
  participate in test planning; `poetry.lock`, `go.mod`, `Cargo.lock`, and
  similar diffs are later work.

See the [agent guide](agent-guide.md) for the change lifecycle, the
[CLI index](cli/README.md) for commands, and
[feature parity](feature-parity.md) for language coverage.
