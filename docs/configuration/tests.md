# Tests And Selectors

`tests` config describes runner configs, project policies, and Playwright
selector extraction.

```yaml
tests:
  playwright:
    configs: tests/playwright.config.ts
    selectors:
      testIds: [data-testid, data-pw]
      htmlIds: true
      componentTestIds:
        testId: data-testid
    selectorRoots: ["web"]
    selectorExclude: ["web/generated/**"]
  vitest:
    configs: vitest.config.mts
```

Selector settings feed Playwright coverage, route impact, and graph edges.

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
