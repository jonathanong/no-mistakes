# `playwright-coverage`

Runs Playwright route and selector coverage from `no-mistakes check`.

```yaml
rules:
  - rule: playwright-coverage
    tests:
      playwright: [web]
```

Counterexample: a page or selector is reachable in the app but has no matching
Playwright coverage.

Fix: add coverage, adjust selector config, or exclude intentional gaps.
