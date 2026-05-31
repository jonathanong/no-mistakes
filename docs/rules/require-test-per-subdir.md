# `require-test-per-subdir`

Requires at least one matching test file in each first-level subdirectory.

```yaml
rules:
  - rule: require-test-per-subdir
    scope: repository
    options:
      roots: [src]
      testGlob: "**/*.test.ts"
```

Counterexample: `src/payments/` with source files but no tests.

Fix: add a test file or exclude the directory.
