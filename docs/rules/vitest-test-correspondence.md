# `vitest-test-correspondence`

Enforces source/test correspondence for Vitest projects.

```yaml
rules:
  - rule: vitest-test-correspondence
    tests:
      vitest: [unit]
    options:
      duplicateStemGroup: exact
```

Counterexample: a source file selected by the rule has no corresponding Vitest
test.

Fix: add a corresponding test or adjust include/exclude policy.

Set `duplicateStemGroup: first-dot-segment` when sibling files such as
`index.test.mts` and `index.edge.test.mts` should count as duplicate stem tests
that must move under the configured `testsDir`.
