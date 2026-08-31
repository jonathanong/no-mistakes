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

## Suppression

Use `no-mistakes-disable-next-line no-test-git-sha` for a single documented
exception, or `no-mistakes-disable-file` only for an intentional snapshot.

## Related rules

[`test-no-dependency-pins`](test-no-dependency-pins.md) similarly prevents
tests from asserting volatile external version pins.
