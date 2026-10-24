# `no-test-git-sha`

```yaml
rules:
  - rule: no-test-git-sha
    scope: repository
    include: ["**/*.test.ts"]
```

## Why and when

Enable this rule when test fixtures must not embed a historical Git revision.
Exact revisions make tests depend on unrelated repository history.

## What it catches

The rule reports each 40-character hexadecimal literal in the selected,
Git-visible file inventory. `include` and `exclude` use the common rule-path
selection fields. `options.allowedContexts` is a list of regular expressions
matched against a source line; it explicitly permits generated-ref or
shape-only assertions.

## Options and defaults

`allowedContexts` defaults to an empty list, so every matching SHA literal is
reported unless an allowed context is deliberately configured.

## Valid example

```ts
expect(makeSyntheticSha()).toMatch(/^[0-9a-f]{40}$/)
```

## Counterexample

```ts
expect(ref).toBe('0123456789abcdef0123456789abcdef01234567')
```

## Fix

Generate a SHA-shaped fixture or assert the ref's shape. If a whole line is an
intentional generated-ref assertion, configure its narrowly matching pattern:

```yaml
options:
  allowedContexts: ["makeSyntheticSha\\("]
```

## Analysis

### Effort

- Runs at `max` only. Configuring `effort: low` (on the rule, or as a repository or CLI default) is not an error: `check` records the warning `requested effort low; effective effort max because this rule has no low implementation` and runs the rule at `max`.

### Unanalyzable inputs

Following [recall over precision](../../README.md#how-analysis-works), nothing this rule cannot prove is treated as clean, and it has no such inputs.

- This rule has no unanalyzable inputs: it judges literal SHA values in test files. Nothing is routed through `options.unanalyzable`.

## Suppression

Use `no-mistakes-disable-next-line no-test-git-sha` for a single documented
exception, or `no-mistakes-disable-file` only for an intentional snapshot.

## Related rules

[`test-no-dependency-pins`](test-no-dependency-pins.md) similarly prevents
tests from asserting volatile external version pins.
