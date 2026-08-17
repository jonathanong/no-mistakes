# Tests And Selectors

`tests` config describes runner configs, project policies, and Playwright
selector extraction.

```yaml
tests:
  playwright:
    configs: tests/playwright.config.ts
    testIdAttribute: data-pw
    selectors:
      testIds: [data-testid, data-pw]
      htmlIds: true
      componentTestIds:
        testId: data-testid
    selectorRoots: ["web"]
    selectorExclude: ["web/generated/**"]
  vitest:
    configs: vitest.config.mts
  swift:
    packages:
      - swift-clients/core
      - swift-clients/ui
    projects:
      swift-core:
        include:
          - swift-clients/core/Tests/**/*.swift
```

Selector settings feed Playwright coverage, route impact, and graph edges.

Language frontends are explicit. Empty lists disable analysis:

```yaml
tests:
  python:
    packages: [backend]
  go:
    modules: [services/worker]
  rust:
    packages: [crates/api]
  rails:
    apps: [apps/web]
  php:
    framework: laravel
    apps: [services/api]
```

When `tests.playwright.configs` and `--playwright-config` are both omitted,
`no-mistakes` automatically discovers Git-visible `playwright*.config.*` files
directly under `--root`. Outside a Git checkout, `.gitignore` and `.ignore`
files are still applied. Explicit config paths remain authoritative and may
refer to ignored files.

`tests.vitest.configs` explicitly accepts `vitest.workspace.*` and
`vitest.projects.*` for the extensions `ts`, `mts`, `cts`, `js`, `mjs`, `cjs`,
and `json`. When omitted, default discovery includes Git-visible root
`vitest.config.*`, `vitest.workspace.*`, and `vitest.projects.*` files,
including JSON project arrays. Project-array sources export project arrays
directly; JSON arrays support static inline project objects and string project
paths/globs and are parsed as JSON, not JavaScript. When a root
`vitest.workspace.*` or `vitest.projects.*` source exists, it takes precedence
over sibling default `vitest.config.*` files; list a root config in that array
when it must also run as a project. Explicit `tests.vitest.configs` remains
authoritative.

Dotnet and Swift test plans use explicit config for source-graph targeting.
`tests.dotnet.projects` or `tests.dotnet.solutions`, and
`tests.swift.packages`, are the explicit inputs; `no-mistakes` does not infer
repository-wide project or package scans. When `tests plan dotnet` or
`tests plan swift` can discover native tests but cannot trace the native
source/project change, the plan falls back to framework-scoped discovered tests
and sets `fallback_triggered`/`fallback_reason`.

Language test plans follow the same native shape. Configure
`tests.python.packages`, `tests.go.modules`, `tests.rust.packages`,
`tests.rails.apps`, or `tests.php.apps`. Empty lists disable that frontend.
`tests plan python|go|cargo|rails|php` then emits `pytest` /
`python -m unittest`, `go test`, `cargo test -p`, `bin/rails test` / `rspec`,
or `phpunit` / `php artisan test` targets. Untraceable source under those
roots falls back to discovered tests in the owning package, module, or app.

## Explicit Vitest projects

`tests.vitest.projects` can declare project ownership directly when a Vitest
config is too dynamic to parse statically:

```yaml
tests:
  vitest:
    configs: vitest.config.mts
    projects:
      backend:
        include: [backend/**/*.test.ts]
      web:
        include: [web/**/*.test.ts]
        exclude: [web/**/*.generated.test.ts]
```

These policies are also used by `vitest-project-mapping` when that rule sets
`explicitProjectsOnly: true`.

Recovered Vitest/Playwright config projects and explicit project policies are
authoritative test universes. Generic filename fallback is used only when that
runner has no recovered projects; it does not add tests outside configured
`include`/`exclude` globs. Vitest and Playwright also reserve each other's owned
files before applying that fallback, while an explicit overlapping policy for
the requested runner remains authoritative. Dotnet and Swift keep their
documented explicit native full-suite fallback behavior.

## Dotnet

`tests.dotnet.projects` lists explicit .NET project mappings used by
`tests plan dotnet` for source-graph targeting. `tests.dotnet.solutions` can
add the projects listed in a solution, but `no-mistakes` does not infer
repository-wide `.csproj` or `.sln` scans.

Use `tests.dotnet.projects` when a project needs named include/exclude policies
or a stable mapping from source changes to test projects. When native tests are
discoverable but the source/project change cannot be traced, `tests plan dotnet`
falls back to framework-scoped discovered tests and sets
`fallback_triggered`/`fallback_reason`.

## Multiple configs

`configs` accepts a single path or a list. When several configs are listed,
`tests plan` builds runner targets per config:

```yaml
tests:
  playwright:
    configs:
      - playwright.config.mts
      - playwright.credentialed.config.mts
```

Ownership is **config-scoped by `testDir`**. When two configs' `testDir`s
overlap — for example a broad `./playwright` and a nested
`./playwright/credentialed` that share a project name like `chromium` — a spec is
attributed to the config with the deepest (most specific) `testDir`. The spec
gets a single target carrying that config's `--config` path, instead of a
duplicate target for the broader config. Configs with sibling or identical
`testDir`s, and explicit `projects` policies, still emit a target each.

## Multiple frontend apps

`playwright-coverage` and `playwright-unique-test-ids` resolve a Playwright
project's route root (`frontendRoot`, defaulting to `<root>/src/app` then
`<root>/app`, whichever exists) and selector root (`selectorRoots`,
defaulting to the whole app package, not just the route directory — so
sibling directories like `src/components` keep selector coverage) from the
repository's `type: nextjs` projects.

With exactly one `type: nextjs` project configured (or none, with a unique
`next.config.*` discoverable at the repository root), this happens
automatically. With more than one `type: nextjs` project, each Playwright
project needs an explicit binding to a specific app; leaving one unbound is a
configuration error rather than a fallback to whichever project happened to
sort first.

When there is no `type: nextjs` project *and* no discoverable `next.config.*`
at all — no frontend-app signal whatsoever — no app can be resolved, so none
of the above applies. `frontendRoot` falls back to the bare `app` literal (or
`<root>/app` when it exists), and `selectorRoots` falls back to matching
`frontendRoot` exactly rather than the whole package — the same defaults
`no-mistakes` used before per-app resolution existed. Configure a
`type: nextjs` project (with an explicit `root:` if it can't be inferred) to
get the wider, decoupled default described above.

Bind via the rule's own `projects:` list — the default mechanism:

```yaml
projects:
  control-web:
    type: nextjs
    root: services/web
  agent-web:
    type: nextjs
    root: services/agent-web

rules:
  - rule: playwright-coverage
    projects: [control-web]
    tests:
      playwright: [control]
  - rule: playwright-coverage
    projects: [agent-web]
    tests:
      playwright: [agent]
```

Or bind per Playwright project directly under `tests.playwright.apps`, which
does not require `rules[].projects` at all:

```yaml
tests:
  playwright:
    apps:
      control:
        project: control-web
      agent:
        project: agent-web
```

Each entry under `tests.playwright.apps.<name>` accepts:

| Field | Meaning |
| --- | --- |
| `project` | The `.no-mistakes.yml` `projects:` key this Playwright project exercises. |
| `frontendRoot` | Overrides the resolved app's route root for this Playwright project only. |
| `selectorRoots` | Overrides the resolved app's selector roots for this Playwright project only. |
| `rewrites` | Overrides the resolved app's rewrites for this Playwright project only. |
| `ignoreRoutes` | Overrides `tests.playwright.ignoreRoutes` for this Playwright project only. |

`tests.playwright.apps.<name>.project` takes precedence over `rules[].projects`
when both are set. `frontendRoot`/`selectorRoots`/`rewrites`/`ignoreRoutes`
set here take precedence over both the resolved app's defaults and the
top-level `tests.playwright.frontendRoot`/`selectorRoots`/`ignoreRoutes`. A
Playwright project can exercise at most one app; binding it to two (via
conflicting `rules[].projects` lists, or a rule naming more than one app) is
an error.

`tests.playwright.projects` (the map of named Playwright-project test-file
policies) and `tests.playwright.apps` (this frontend-app binding) are
independent: the former scopes which test files belong to a Playwright
project, the latter answers which frontend app that project exercises.

## `testIdAttribute`

The attribute that `page.getByTestId(...)` resolves to when matching selector
coverage. Resolution order:

1. `tests.playwright.testIdAttribute`, if set.
2. The `use.testIdAttribute` read statically from the Playwright config.
3. Otherwise, the configured `selectors.testIds`.

Set this when your Playwright config's `testIdAttribute` is not statically
readable — for example when the config object is built by a helper function:

```ts
// playwright.config.ts — testIdAttribute is hidden inside the helper body
export default defineConfig(createPlaywrightConfig({ testDir: './e2e' }))
```

In that case `no-mistakes` cannot see the real attribute and would otherwise
report every `getByTestId` selector as uncovered. Declaring
`testIdAttribute: data-pw` (or relying on the `selectors.testIds` fallback) makes
coverage match `getByTestId('x')` against `data-pw="x"`. See
[`playwright-coverage`](../rules/playwright-coverage.md).

## Selector wrappers

Declare a statically imported helper when one of its arguments carries the same
test ID as `getByTestId(...)`:

```yaml
tests:
  playwright:
    selectors:
      wrappers:
        - module: "@app/playwright-locators"
          export: getAsideLocator
          testIdArgument: 1 # zero-based: the second argument
```

With that configuration, an imported call such as
`getAsideLocator(page, 'save')` covers the same selector as
`page.getByTestId('save')`. `module` is a normal JavaScript module specifier.
The configured module and static import are compared through the request's
TypeScript and workspace resolver, including relative and NodeNext paths,
`baseUrl`/`paths`, package `imports`, and workspace package exports. This is
independent of npm, pnpm, Yarn, or Bun. External packages remain terminal
module identities; `no-mistakes` does not scan `node_modules` or inspect helper
bodies.

If two declarations resolve to the same imported export but disagree on
`testIdArgument`, that binding is ambiguous and does not create coverage.

Only static ESM named, aliased, default, and namespace imports are recognized.
The configured argument uses the same supported forms and interpolation
behavior as `getByTestId`: a string, a template literal, or a regular
expression. Identifier and call-expression values, CommonJS `require`, dynamic
imports, and shadowed local bindings do not create coverage.

## `tests.impact`

Opt-in knobs for the [`tests impact`](../cli/tests-impact.md) query. Both lists
default to empty, so without configuration `tests impact` is unchanged.

```yaml
tests:
  impact:
    alwaysIncludeTests:
      - "**/*.mock.test.*"
    registries:
      - "**/auth-gated-code-splitting.mts"
      - "**/*-registry.mts"
```

- `alwaysIncludeTests` — glob patterns for stub/mock test files that `tests
  impact` must always surface when they transitively import a changed file, even
  when a test-suite `exclude` glob would normally drop them from discovery. Use
  this for mock stubs (e.g. `*.mock.test.*`) that run in a separate project but
  still need updating whenever the file they stub changes. Keep the globs
  test-shaped — a broad pattern like `**/*` would make every transitively
  imported file look like a test.
- `registries` — glob patterns for hand-maintained registry files (lazy-import
  maps, code-splitting tables). When a changed file is imported by a file
  matching one of these globs, `tests impact` emits a `registry-hint` warning so
  you verify the registry entry is up to date. Prefix patterns with `**/` to
  match at any depth. The hint follows the dependency graph, so it fires when the
  registry's mapping is reachable — for example an exported map
  (`export const registry = { foo: () => import('./foo') }`). A fully private map
  reached only through dynamic key indexing
  (`const registry = {…}; export const load = k => registry[k]`) is pruned by
  reachability analysis and may not trigger the hint; export the map (or the
  loader's entries) for reliable detection.

## Swift

`tests.swift.packages` lists SwiftPM package roots explicitly. `no-mistakes` does
not infer repository-wide Swift packages. Swift test discovery reads each
configured `Package.swift`, discovers `.testTarget(...)` targets under
`Tests/<target>/**/*.swift`, and emits `swift test --package-path <package>
--filter <test-target>` execution targets.

Use `tests.swift.projects` when a package needs named include/exclude policies.
Project aliases affect discovery, while runnable Swift filters remain SwiftPM
test targets derived from the selected test file.

When native tests are discoverable but the source/project change cannot be
traced, `tests plan swift` falls back to framework-scoped discovered tests and
sets `fallback_triggered`/`fallback_reason`.
