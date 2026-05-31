# `vitest-test-correspondence`

Enforces source/test correspondence for Vitest projects.

```yaml
rules:
  - rule: vitest-test-correspondence
    tests:
      vitest: [unit]
```

Counterexample: a source file selected by the rule has no corresponding Vitest
test.

Fix: add a corresponding test or adjust include/exclude policy.
