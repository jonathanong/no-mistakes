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

## Why and when

Use this rule when source files should have a predictable nearby Vitest test and
new modules must not silently enter the suite without regression coverage.

## What it catches/requires

Each source selected by the configured Vitest application must have a matching
test stem under the test policy; duplicate stem handling follows
`duplicateStemGroup`.

## Options and defaults

`scopes` limits source/test paths; an empty list uses the rule application.
`testExtensions` defaults to `.test.mts`, `.test.ts`, and `.test.tsx`.
`testsDir` defaults to `__tests__`. `direction` defaults to `both`:
`source-to-test` reports selected sources without a test, `test-to-source`
reports tests without a source, and `both` performs both checks.
`stemSuffixesToStrip` removes configured source-name suffixes before matching.
`duplicateStemGroup` defaults to `exact`; set `first-dot-segment` when variants
such as `index.edge.test.mts` should share a stem and move under `testsDir`.

## Valid example

```text
src/users.ts
src/users.test.ts
```

## Counterexample

```text
src/users.ts
```

## Fix

Add the corresponding test, adjust include/exclude ownership, or choose the
duplicate stem policy that matches the repository naming convention.

## Suppression

Use a narrow exclude for generated or adapter-only sources. Use the file
directive only when another test framework provides the coverage.

## Related rules

[`vitest-project-mapping`](vitest-project-mapping.md) ensures each test has one
owner; [`require-test-per-subdir`](require-test-per-subdir.md) enforces a
coarser directory-level test presence policy.
