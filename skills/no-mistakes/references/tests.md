# `tests` command reference

## When to use

Use `tests plan` for all test selection — it is the preferred replacement for
`dependents --test` in all repos, with or without a `testPlan:` block in
`.no-mistakes.yml`. Without a config block it uses default direct + dependencies
groups; with a config block it adds custom environments, limits, coverage groups
(Playwright only), and global-config triggers. It also handles diff/deleted-file
and lockfile cases that `dependents --test` misses.

Use `tests why` when you need to explain which dependency path connects a
changed file to a selected test. Use `tests impact` when you have a fixed set
of changed files and only need impacted test paths with no environment or group
config applied.

## `tests plan`

Select tests to run from changed files, diffs, and configured environments.

```sh
# Changed-file selection (preferred)
no-mistakes tests plan vitest --changed-file src/utils.mts --format paths
no-mistakes tests plan playwright --changed-file web/app/users/page.tsx --format paths

# Diff-based (from git)
no-mistakes tests plan vitest --base origin/main --format json
no-mistakes tests plan vitest --from-git-diff origin/main...HEAD --format json

# Named environment (from .no-mistakes.yml testPlan)
no-mistakes tests plan vitest --environment prePush --changed-file src/api.mts --format paths
```

Key flags:
- `--changed-file <FILE>` — explicit changed file path; repeatable.
- `--changed-files <FILE>` — path to a file containing one changed path per line.
- `--base <REF>` / `--head <REF>` — compute changed files from a git diff.
- `--from-git-diff <BASE...HEAD>` — single-argument sugar for `--base`/`--head`;
  desugars to the same `git diff --relative --name-status <base>...<head>` lookup
  (three-dot only — bare `<base>` and `<base>...` both default head to `HEAD`;
  two-dot `<base>..<head>` is rejected since it compares a different baseline).
  Conflicts with `--base`/`--head`.
- `--diff <FILE>` / `--diff-stdin` / `--diff-command <CMD>` — supply a diff
  directly.
- `--entrypoint <FILE>` — treat a file as the root regardless of changes.
- `--environment <NAME>` — pick an env group from `testPlan.environments`.
- `--limit-percent <N>` / `--limit-files <N>` — override `testPlan` limits.
- `--global-config-fallback true|false` — run the full suite when a global
  config file changes (package.json, tsconfig.json, etc.) or when a lockfile
  diff cannot be parsed; does not trigger for ordinary source files that happen
  to have no test dependents.
- `--format paths|json` — `paths` for shell substitution, `json` for agents.

`fullSuiteTriggers.projects` can scope a configured trigger to runner projects:

```yaml
testPlan:
  vitest:
    fullSuiteTriggers:
      projects:
        database-resources:
          paths: ["migrations/**/*.sql", "!migrations/archive/**"]
          targets: [database]
```

Here `database` is a Vitest project name. A match selects only tests owned by
that target, reports `configured-trigger`, and does not mark the plan as a
fallback. Environment filters and limits run afterward. Legacy `true` and path
list entries remain broad fallbacks. Trigger paths are ordered: later `!`
patterns exclude earlier matches and later positive patterns can re-include.

Revision and inline-diff plans compare `.no-mistakes.yml`/`.yaml` semantically
per framework, so formatting-only edits and unrelated framework changes do not
invalidate the selected framework. Changed-file-only input and unreadable old
configuration fail open to the normal global-config fallback.

For Playwright, a changed Next.js page selects specs that navigate to it — including
specs whose navigation path interpolates an unresolvable value (e.g.
`` `/user/${userId}` `` or `'/user/' + id`), which matches the page's dynamic
`[param]` segment.

Node API: `testsPlan(options)`.

In a TypeScript/JavaScript workspace, omit `tsconfig` so test impact follows
the config owning each importing file. Passing `tsconfig` deliberately forces a
single config for the whole plan.
### Vitest setup dependency tracing

`tests plan vitest` and `testsPlan()` statically trace each project's effective
`setupFiles` and `globalSetup`, including their ordinary import/re-export
closure. A changed setup dependency selects only tests owned by that Vitest
project. Inline project fields inherit a root field only with `extends: true`;
the default and `extends: false` keep the project independent. A string config
in `test.projects` is likewise independent of its referencing config. Explicit
values replace inherited fields and `[]` clears them.
For supported inline objects, nested `test` owns `setupFiles` and
`globalSetup`; same-named outer fields are ignored regardless of direct or
static-spread declaration order.
Workspace configs may export projects directly or through
`defineWorkspace([...])`. Without `tests.vitest.configs`, root
`vitest.workspace.*` and `vitest.projects.*` files, including `.json`, are
discovered by default. Config globs include suffixes such as
`vitest.config.unit.ts` and `vite.config.e2e.js`.
`defineWorkspace` is static through named ESM imports, ESM namespaces, or a
direct `require('vitest/config')` namespace; ESM defaults and CommonJS
`.default` members remain unsupported dynamic forms.

Dynamic or unresolved setup declarations emit `vitest-setup-dynamic` or
`vitest-setup-unresolved` warnings. If relevant, planning safely selects the
known owner scope (or the discovered Vitest framework set if no owner is known)
and sets `fallback_triggered` without relying on `globalConfigFallback`. Its
bounded helper closure follows ordinary static imports/re-exports and literal
CommonJS `require(...)` or `require.resolve(...)` dependencies, retaining
edits and deletions as owner triggers. Static CommonJS bindings support direct
members, destructured aliases, and named `module.exports = { ... }` values;
computed or non-literal forms are not followed.

For `tests impact`, a malformed or unavailable optional Vitest config does not
block unrelated native test impact. A successfully prepared Vitest config is
still strict: discovery errors such as invalid include patterns are returned.

Resolved setup edges use `via: ["vitest-setup"]`; optional aligned
`via_details` records `{ type: "vitest-setup", field: "setupFiles" |
"globalSetup" }`. `tests why` and `tests graph` expose the same structured
`detail`.

## `tests why`

Explain the dependency path from a changed file to a selected test.

```sh
no-mistakes tests why tests/users.test.mts --plan plan.json
no-mistakes tests why tests/users.test.mts --changed src/api.mts --format json
```

Key flags:
- `--plan <FILE>` — path to a previously generated `tests plan` JSON file.
- `--changed <FILE>` — changed file to compute the path from (without a prior plan).
- `--format text|json`.

Node API: `testsWhy(options)`.

## `tests impact`

Impacted tests for specific changed files (no `testPlan` config required).

```sh
no-mistakes tests impact src/utils.mts --format paths
```

Traversal follows `next/dynamic(() => import('./Foo'))` boundaries (Foo's tests
surface at `medium` confidence). Two opt-in `tests.impact` config knobs refine
output: `alwaysIncludeTests` surfaces suite-excluded stub tests (e.g.
`**/*.mock.test.*`), and `registries` emits a `registry-hint` warning when the
changed file is imported by a registry file (e.g. `auth-gated-code-splitting.mts`).

Node API: `testsImpact(options)`.

Literal runtime filesystem resources (`fs` reads/directories and supported
static glob calls) are part of ordinary test impact. A plan JSON reason with
`via: ["resource"]` may carry edge-aligned `via_details` containing the
structured `{ type: "resource", consumer_file, call_sites: [{ call_kind,
line }] }` detail. Dynamic paths, patterns, or cwd values are warnings, not
guessed dependencies or implicit fallback triggers.

## `tests comment`

Render a plan JSON as a Markdown PR comment.

```sh
no-mistakes tests comment plan.json
no-mistakes tests comment plan.json --out comment.md
```

Node API: `testsComment(options)`.

## `testPlan` configuration

In `.no-mistakes.yml`:

```yaml
testPlan:
  vitest:
    environments:
      pre-push:
        groups:
          - type: direct
          - type: dependencies
        limit:
          percent: 20
          files: 30
        globalConfigFallback: false
      pull-request:
        groups:
          - type: direct
          - type: sample
        limit:
          files: 50
  playwright:
    environments:
      pre-push:
        groups:
          - type: direct
          - type: coverage
        limit:
          percent: 30
```

`fullSuiteTriggers` and `environments` are nested under the framework key
(`vitest` or `playwright`). Environment names default to `pre-push`.

Group types: `direct`, `dependencies`, `sample` — for `vitest`;
`direct`, `dependencies`, `coverage`, `sample` — for `playwright`.
(`coverage` is a Playwright-only group type; vitest does not support it.)
Consult https://github.com/jonathanong/no-mistakes/blob/main/docs/configuration/test-plan.md
for the full schema.
