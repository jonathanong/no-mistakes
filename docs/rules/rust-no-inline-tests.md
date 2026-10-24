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

Upgrade note: in-scope files that cannot be read, or parsed where this rule
requires parsing, now fail `check` under the default `unanalyzable: report`
instead of silently passing. Fix or exclude those inputs, or set
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

A Rust parse failure also uses this policy when the selected file contains
both substrings `cfg` and `test`. Other Rust syntax errors are not reported
by this rule. Marker-negative files skip AST parsing because the
forbidden attribute cannot be present. Combined Rust checks share one source
read and one parse; a parse failure affects only requested AST checks, while
read failures affect every requested check for that file.

AST analysis requires the full Rust file grammar (`syn::File`). An `include!`
expression or statement fragment is not a standalone Rust module and reports
uncertainty when the requested markers require parsing. Prefer extracting it
into a function or an out-of-line module. For an intentional fragment, narrow
the rule with an exact file `exclude`; excluded files are not read or parsed.

## Suppression

Prefer moving the tests. Use a file directive only for generated or external
Rust source that cannot be relocated.

## Related rules

[`rust-max-lines-per-file`](rust-max-lines-per-file.md) enforces the resulting
file budgets; [`rust-no-inline-allows`](rust-no-inline-allows.md) keeps test
lint exceptions explicit.
