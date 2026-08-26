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

## Why and when

Use this rule for source trees where every first-level feature directory should
carry at least one local regression test.

## What it catches/requires

Each selected first-level subdirectory containing source must match at least
one file under `testGlob`.

## Options and defaults

`roots` and `testGlob` are required; there are no inferred source roots or test
patterns. Common include/exclude filters still apply.

## Valid example

```text
src/payments/charge.ts
src/payments/charge.test.ts
```

## Counterexample

```text
src/payments/charge.ts
```

## Fix

Add a focused test in the selected subtree or exclude generated/adapter-only
directories from the rule application.

## Suppression

Prefer a narrower `roots`/exclude configuration. Use
`no-mistakes-disable-file require-test-per-subdir` only for a directory whose
test ownership is documented elsewhere.

## Related rules

[`require-storybook-stories`](require-storybook-stories.md) covers component
stories; [`vitest-test-correspondence`](vitest-test-correspondence.md) checks
source/test correspondence for configured Vitest projects.
