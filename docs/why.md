# Why no-mistakes exists

Coding agents miss structure that a text search cannot see. They recreate an
API that already exists in another file, ship a Next.js page with no
Playwright coverage, run every suite because they cannot tell which tests
reach a change, or write a dynamic `fetch` / test ID that drops the
relationship from the graph.

`no-mistakes` answers those questions from the current checkout. It parses
each requested source once per required semantic mode, builds one in-memory
graph, and returns a small structured result. It does not run the app, call
a model, or keep a database.

The model is:

```text
facts (one parse per required semantic mode) → one graph → commands and checks
```

Static literals produce edges. Dynamic values are skipped rather than
guessed. That is why the bundled rules exist: they keep the shapes the graph
can trust.

When the analyzer cannot follow a pattern, the default fix is the code, not
the analyzer. A rule bans the pattern and names the static rewrite: an
`as const` literal, a finite union, or a value declared once instead of a
shared registry mutated at runtime. Recovery is worth teaching only for common
patterns that are sound to recover and have no reasonable static rewrite.

Rules apply only when confident. A rule judges the facts it can soundly judge,
such as an exact value, a finite union, or a known statement shape, and
abstains on the rest. An abstention is reported as unanalyzable, never as a
pass or a guessed finding.

## Narrowing

Analysis runs where the configuration says the behavior lives, not wherever a
pattern happens to match.

- **Locations.** A repository already knows where code of a given kind may
  live. For example, an import-boundary rule (dependency-cruiser, or
  [`forbidden-dependencies`](rules/forbidden-dependencies.md)) may allow only `backend/services`, `backend/data-stores`,
  and `backend/test-helpers` to import the database module. Rules then scan
  only those directories (project `root`, `include`, `sqlInclude`). Repository-
  wide scanning is an explicit opt-in, not the default.
- **Targets.** Inside those locations, a rule checks only calls it can tie to
  configuration: the database module's imports, configured executor factories
  and types, trusted SQL tags, queue factories, route roots. A string that
  merely looks like SQL, or a method that is merely named `query`, is not an
  executor.
- **Declare once.** One project-level declaration of a domain's locations and
  targets is shared by every rule and command that needs it. Repeating the same
  `include` and executor list in each rule lets the copies drift apart.
- **No inferred conventions.** No hardcoded `src/`, `backend/`, or framework
  folklore. Full-suite, repository-wide, and global fallbacks need an explicit
  flag or configuration knob.

Narrowing does not trade away recall inside the declared scope: there, the
rules above about soundness and unanalyzable reporting still apply. It keeps
noise from code that cannot be the thing being checked, and it keeps runs fast
on large repositories.

### Unprovable selections

Recall inside a declared framework still comes first. When a Vitest setup
declaration is dynamic, unresolved, or an unavailable inline `extends` target,
or when a Dotnet or Swift change cannot be traced to a project, the plan
widens only within that framework. Owner scope is selected first. The
framework's discovered tests are used only when ownership is unknown. The
widening never spills into another framework or the rest of the repository.
The plan sets `fallback_triggered` and `fallback_reason`, and it emits a
warning that names the declaration. This does not require
`--global-config-fallback`. Repository-wide and full-suite selection still
need that explicit opt-in. Dotnet and Swift stay inside that framework in
`tests_plan_json_reports_configured_dotnet_source_impact` and
`tests_plan_json_reports_configured_swift_source_impact`
(`crates/no-mistakes/src/napi_api/tests/tests_native_fallback.rs`).

## Recall over precision

A missed mistake is worse than a noisy report. An over-report costs a reviewer
or agent a moment to dismiss or suppress; a silent omission cannot be detected
at all. So `no-mistakes` never treats an input it cannot prove as clean:
uncertain inputs (dynamic or unproven shapes) are surfaced through the
`unanalyzable` policy, while definite violations are always findings. When a
report says an input is unanalyzable, rewrite the code to a statically provable
shape, raise `effort`, or suppress it with a reason. See
[how analysis works](../README.md#how-analysis-works).

## Three jobs

1. **Scope the change.** Callers, routes, queues, workflows, and named
   exports — resolved through tsconfig aliases and workspace packages, not
   a string match.
2. **Select the tests.** Vitest, Playwright, and configured native plans
   for the files that actually reach the change, including in PR CI.
3. **Keep the graph honest.** Repository checks and ESLint/Oxlint rules
   stop agents from hiding relationships behind aliases, dynamic URLs, or
   duplicate public names.

Graph queries such as `dependents`, `tests plan vitest`, and
`playwright check` work without a custom config block when the checkout
already uses the framework's own imports and file conventions. Next.js is
recognized from `next.config.*` and the App Router directories `app/` and
`src/app`. Express, Hono, Fastify, Koa, NestJS, and BullMQ / glide-mq graphs
are recognized from those packages' imports. Built-in targets are those
imports and file conventions. Nothing in that set is inferred from naming.
Configuration adds custom Playwright roots, wrappers, and multi-app bindings,
plus custom queue factories, Remix and language HTTP routes, language
frontends, and repository rules.

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
| Change a signature and miss import-based callers | Callers the graph can see; confirm exact lines with `rg` | [`symbols --mode signature-impact`](cli/symbols.md), [`call-sites`](cli/call-sites.md) |
| Rename a queue job and miss the worker | Producers with no processor | [`queues check`](cli/queues-check.md), [`queues related`](cli/queues-related.md) |
| Hoist a uniqueness token above re-entrant `beforeAll` | Duplicate-key failures that depend on the worker | [`playwright-no-hoisted-unique-token`](eslint-rules/playwright-no-hoisted-unique-token.md) |
| One raw `scrollTo` racing cursor pagination | A wait that burns its timeout | [`playwright-no-raw-scroll-pagination`](eslint-rules/playwright-no-raw-scroll-pagination.md) |
| Catch `isVisible()` and return a boolean | A broken locator looks like a normal absence | [`playwright-no-swallowed-visibility`](eslint-rules/playwright-no-swallowed-visibility.md) |
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
