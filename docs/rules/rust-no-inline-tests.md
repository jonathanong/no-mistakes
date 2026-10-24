# `rust-no-inline-tests`

Bans inline Rust `#[cfg(test)] mod tests` modules.

```yaml
rules:
  - rule: rust-no-inline-tests
    scope: repository
```

Counterexample: tests embedded at the bottom of `src/lib.rs`.

Fix: move tests into sibling `tests.rs` or integration tests, using fixture
files under `fixtures/` or `test-cases/`.

## Why and when

Use this rule in shared Rust libraries where production modules should remain
small and integration fixtures should be visible to reviewers.

## What it catches/requires

Inline `#[cfg(test)] mod tests` modules are findings. Sibling test modules and
integration tests are the supported locations.

## Options and defaults

`roots` optionally replaces the rule's target roots; relative paths are
resolved from the repository root. `excludes` omits files whose root-relative
path contains one of its strings. By default, `roots` uses the rule's target
roots and `excludes` is empty. The rule recognizes Rust `cfg(test)` forms in
non-test source files.

## Valid example

```rust
// src/lib.rs
pub fn add(a: i32, b: i32) -> i32 { a + b }

#[cfg(test)]
mod tests;
```

## Counterexample

```rust
#[cfg(test)]
mod tests { #[test] fn adds() {} }
```

## Fix

Move the module body into `src/tests.rs` or an integration test under `tests/`,
and put reusable fixtures under `fixtures/` or `test-cases/`.

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` is accepted: `check` emits an analysis warning (requested effort low; effective effort max because this rule has no low implementation) and runs the rule at `max`.

### Unanalyzable inputs

Following recall over precision, nothing is silently treated as clean ([how analysis works](README.md#how-analysis-works)); this rule has no unanalyzable inputs because it matches inline test modules in parsed Rust source, so `options.unanalyzable` has no effect on it.

## Suppression

Prefer moving the tests. Use a file directive only for generated or external
Rust source that cannot be relocated.

## Related rules

[`rust-max-lines-per-file`](rust-max-lines-per-file.md) enforces the resulting
file budgets; [`rust-no-inline-allows`](rust-no-inline-allows.md) keeps test
lint exceptions explicit.
