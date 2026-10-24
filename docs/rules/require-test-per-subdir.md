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
one file under `testGlob`. Roots are normalized before the scan: `./`, empty
segments, and in-repo `..` collapse, so `services/../agents` is `agents`. A
root that leaves the repository, such as `..`, is one finding on the config
file at line 1 (`..: configured root escapes the repository`). The parent
directory is not scanned.

## Options and defaults

`roots` is required; an empty list disables the rule. `testGlob` defaults to
`*.test.*`. `excludeDirs` omits named first-level directories, and
`directChild` defaults to `false`, so nested tests count unless it is enabled.
Common include/exclude filters still apply.

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

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` is accepted: `check` emits an analysis warning (requested effort low; effective effort max because this rule has no low implementation) and runs the rule at `max`.

### Unanalyzable inputs

Following recall over precision, nothing is silently treated as clean ([how analysis works](../../README.md#how-analysis-works)); this rule has no unanalyzable inputs because it checks file presence in the file inventory, so `options.unanalyzable` has no effect on it.

## Suppression

Prefer a narrower `roots`/exclude configuration. Use
`no-mistakes-disable-file require-test-per-subdir` only for a directory whose
test ownership is documented elsewhere.

## Related rules

[`require-storybook-stories`](require-storybook-stories.md) covers component
stories; [`vitest-test-correspondence`](vitest-test-correspondence.md) checks
source/test correspondence for configured Vitest projects.
