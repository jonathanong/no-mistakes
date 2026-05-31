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
