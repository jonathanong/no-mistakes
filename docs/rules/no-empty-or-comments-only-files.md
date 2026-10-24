# `no-empty-or-comments-only-files`

## Why and when

Use this rule when every tracked source or documentation file should have a
real purpose instead of acting as a placeholder or hidden marker.

## What it catches

It reports empty files and files whose selected language syntax contains only
comments or whitespace.

## Options

`extensions` defaults to TypeScript, JavaScript, SQL, Rust, and CSS source
extensions. `intentionallyEmpty` defaults to an empty list of exceptions. Use
the common rule `include` and `exclude` fields to narrow the file scope.

## Valid example

A file containing one meaningful declaration or Markdown paragraph passes.

```ts
export const ready = true;
```

## Counterexample

A placeholder file containing only `// TODO`.

```ts
// TODO
```

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` is accepted: `check` emits an analysis warning (requested effort low; effective effort max because this rule has no low implementation) and runs the rule at `max`.

### Unanalyzable inputs

Following recall over precision, nothing is silently treated as clean ([how analysis works](README.md#how-analysis-works)); this rule has no unanalyzable inputs because it judges file content that is fully read and tokenized, so `options.unanalyzable` has no effect on it.

## Related rules

[`agents-md-max-size`](agents-md-max-size.md) constrains instruction-file size;
[`required-local-docs`](required-local-docs.md) requires meaningful local docs.

Bans tracked files that contain no executable or meaningful content.

```yaml
rules:
  - rule: no-empty-or-comments-only-files
    scope: repository
```

Counterexample: a placeholder file containing only `// TODO`.

Compliant example: a README with project-specific notes, or a source file with
an exported placeholder implementation that callers can import and test.

Fix: delete the file or add real implementation/docs content.

Suppression caveat: suppress only temporary placeholders with a `no-mistakes`
directive and a reason that names the follow-up owner or removal condition.
