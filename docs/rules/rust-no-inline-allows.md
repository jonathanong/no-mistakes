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

## Suppression

This rule intentionally rejects inline `allow` as an escape hatch. Use the
repository lint configuration or a documented rule-level scope instead.

## Related rules

[`rust-max-lines-per-file`](rust-max-lines-per-file.md) controls module size;
[`rust-no-inline-tests`](rust-no-inline-tests.md) controls test placement.
