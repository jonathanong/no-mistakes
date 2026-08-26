# `rust-max-lines-per-file`

Caps Rust source and test file length.

```yaml
rules:
  - rule: rust-max-lines-per-file
    scope: repository
    options:
      srcMax: 200
      testMax: 500
```

Counterexample: a 900-line Rust source file. Blank lines and comments are ignored, but Rust string and raw-string literal contents still count as code lines because they are part of the file body.

Fix: extract cohesive modules or move test fixtures out of inline code.

## Why and when

Use this rule when Rust files need a reviewable size budget and tests should
not crowd production modules.

## What it catches/requires

Non-test Rust files must stay within `srcMax`; test files must stay within
`testMax`. Blank lines and comments are ignored, while literal body lines count.

## Options and defaults

`srcMax` and `testMax` are required limits for the rule application; the rule
does not choose a project-specific default budget.

## Valid example

```rust
pub fn parse(input: &str) -> Result<(), Error> { todo!() }
```

## Counterexample

One source file contains unrelated parsing, HTTP, storage, and hundreds of
inline fixture lines beyond `srcMax`.

## Fix

Split cohesive modules, move fixtures to `fixtures/` or `test-cases/`, and keep
the production file focused.

## Suppression

Adjust the configured limit for a deliberate generated file or use a file
directive only when the exception has a documented ownership reason.

## Related rules

[`rust-no-inline-tests`](rust-no-inline-tests.md) moves tests out of production
files; [`rust-no-inline-allows`](rust-no-inline-allows.md) keeps lint policy
visible.
