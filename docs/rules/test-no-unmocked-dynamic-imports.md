# `test-no-unmocked-dynamic-imports`

Requires dynamic imports reachable from tests to be mocked.

```yaml
rules:
  - rule: test-no-unmocked-dynamic-imports
    tests:
      vitest: [unit]
```

Counterexample: a test reaches `await import("external-lib")` without a manual
mock.

Fix: add a manual mock, make the dependency static, or narrow the rule target.
