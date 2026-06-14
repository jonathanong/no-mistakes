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

## Swift

`tests.swift.packages` lists SwiftPM package roots explicitly. `no-mistakes` does
not infer repository-wide Swift packages. Swift test discovery reads each
configured `Package.swift`, discovers `.testTarget(...)` targets under
`Tests/<target>/**/*.swift`, and emits `swift test --package-path <package>
--filter <test-target>` execution targets.

Use `tests.swift.projects` when a package needs named include/exclude policies.
Project aliases affect discovery, while runnable Swift filters remain SwiftPM
test targets derived from the selected test file.
