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

Upgrade note: in-scope files that cannot be read as UTF-8 now fail `check`
under the default `unanalyzable: report` instead of silently passing. Fix or exclude those inputs, or set
`options.unanalyzable: advise` or `ignore` to restore the previous successful
exit status for uncertainty alone. Independent definite findings still fail
under every policy.

Selected files that cannot be read as UTF-8 produce an `unanalyzable`
diagnostic instead of silently passing. Restore read access or repair the
encoding so the rule can verify the file. The shared `options.unanalyzable`
setting inherits the repository policy: `report` (the default) fails,
`advise` emits non-failing advice, and `ignore` omits uncertainty. Definite
violations remain failures under every policy, including when `effort: low`
falls back to `max`.

Configured scope, includes, excludes, and rule-specific exemptions still
select the inputs. Suppressions apply to both channels, including directives
retained in an invalid UTF-8 read error; source bytes are never reinterpreted
as analyzable code. Rule-level custom messages apply to these diagnostics.

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
