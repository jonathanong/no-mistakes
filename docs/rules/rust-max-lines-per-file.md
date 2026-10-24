# `rust-max-lines-per-file`

Caps Rust source and test file length in **code lines**, not physical lines.

```yaml
rules:
  - rule: rust-max-lines-per-file
    scope: repository
    options:
      srcMax: 3
      testMax: 10
```

Counterexample: a 900-line Rust source file. Blank lines and comments are ignored, but Rust string and raw-string literal contents still count as code lines because they are part of the file body.

Fix: extract cohesive modules or move test fixtures out of inline code.

## Why and when

Use this rule when Rust files need a reviewable size budget and tests should
not crowd production modules.

## What it catches/requires

Non-test Rust files must stay within `srcMax`; test files must stay within
`testMax`. Blank lines and comment-only lines (including documentation and
nested block comments) are ignored. A line containing code and a trailing comment counts once.
String and raw-string literal body lines containing text count even when they
contain comment markers; entirely blank lines are skipped, including inside literals.

A test file has basename `tests.rs` or a repository-relative path containing a
`tests/` directory. Names such as `options_tests.rs` outside that directory use
`srcMax`; the rule does not inspect `#[cfg(test)]` to classify files.

The repository's contributor guideline is stricter: 200 physical lines for
source and 500 for tests. Audit tracked files with the command below and split cohesive modules when they
exceed that guideline:

```sh
git ls-files -z '*.rs' | xargs -0 wc -l
```

A passing check proves only the configured code-line limits, scopes, exclusions,
and suppressions. For example, a nine-line file with six blank/comment-only
lines and three code lines passes `srcMax: 3`, but fails `srcMax: 2`.
File suppressions can also hide code-line overages;
`check --include-suppressed --format json` exposes suppression accounting. This distinction does not change
the rule's thresholds or counting behavior.

## Options and defaults

`srcMax` defaults to `200` code lines and `testMax` defaults to `500`.
`excludes` skips files whose repository-relative path contains an entry, and
`roots` defaults to the rule application's target roots. There are no other
rule-local options.

## Valid example

```rust
pub fn parse(input: &str) -> Result<(), Error> { todo!() }
```

## Counterexample

One source file contains unrelated parsing, HTTP, storage, and hundreds of
inline fixture lines beyond `srcMax`.

```rust
pub fn too_long() {
    let a = 1;
    let b = 2;
    let c = 3;
    let d = 4;
}
```


## Fix

Split cohesive modules, move fixtures to `fixtures/` or `test-cases/`, and keep
the production file focused.

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

This rule counts code lines without parsing a Rust AST. Malformed Rust that
is readable remains subject to the line limit; requesting this rule alone
does not trigger the inline-attribute rules or a syntax parse.

## Suppression

Adjust the configured limit for a deliberate generated file or use a file
directive only when the exception has a documented ownership reason.

## Related rules

[`rust-no-inline-tests`](rust-no-inline-tests.md) moves tests out of production
files; [`rust-no-inline-allows`](rust-no-inline-allows.md) keeps lint policy
visible.
