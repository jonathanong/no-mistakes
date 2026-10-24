<!-- cspell:ignore usize -->

# `rust-no-inline-allows`

Bans inline Rust `allow` attributes.

```yaml
rules:
  - rule: rust-no-inline-allows
    scope: repository
```

Counterexample: `#[allow(dead_code)]` above a function.

Fix: remove the allow by addressing the lint or use a documented broader policy
where appropriate.

## Why and when

Use this rule when lint exceptions must be reviewed centrally instead of being
hidden beside the code they silence.

## What it catches/requires

Rust `#[allow(...)]` attributes in analyzed source are findings, including
function-, module-, and item-level attributes.

## Options and defaults

`roots` optionally replaces the rule's target roots; relative paths are
resolved from the repository root. `excludes` omits files whose root-relative
path contains one of its strings. By default, `roots` uses the rule's target
roots and `excludes` is empty. Test files are not scanned by this rule.

## Valid example

```rust
pub fn used_api() -> usize { 1 }
```

## Counterexample

```rust
#[allow(dead_code)]
fn unfinished_api() {}
```

## Fix

Remove the unused code, correct the lint, or move a deliberately broad lint
policy to the repository's approved configuration.

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` is accepted: `check` emits an analysis warning (requested effort low; effective effort max because this rule has no low implementation) and runs the rule at `max`.

### Unanalyzable inputs

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

A Rust parse failure also uses this policy when the selected file contains
the `allow` marker. Marker-negative files skip AST parsing because the
forbidden attribute cannot be present. Combined Rust checks share one source
read and one parse; a parse failure affects only requested AST checks, while
read failures affect every requested check for that file.

AST analysis requires the full Rust file grammar (`syn::File`). An `include!`
expression or statement fragment is not a standalone Rust module and reports
uncertainty when the requested markers require parsing. Prefer extracting it
into a function or an out-of-line module. For an intentional fragment, narrow
the rule with an exact file `exclude`; excluded files are not read or parsed.

## Suppression

This rule intentionally rejects inline `allow` as an escape hatch. Use the
repository lint configuration or a documented rule-level scope instead.

## Related rules

[`rust-max-lines-per-file`](rust-max-lines-per-file.md) controls module size;
[`rust-no-inline-tests`](rust-no-inline-tests.md) controls test placement.
