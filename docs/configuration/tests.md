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

Dotnet and Swift test plans use explicit config for source-graph targeting.
`tests.dotnet.projects` or `tests.dotnet.solutions`, and
`tests.swift.packages`, are the explicit inputs; `no-mistakes` does not infer
repository-wide project or package scans. When `tests plan dotnet` or
`tests plan swift` can discover native tests but cannot trace the native
source/project change, the plan falls back to framework-scoped discovered tests
and sets `fallback_triggered`/`fallback_reason`.

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
