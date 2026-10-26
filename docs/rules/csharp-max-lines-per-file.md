# `csharp-max-lines-per-file`

## Why and when

Use this rule when C# source and tests need a reviewable physical-size budget.

## What it catches

It counts comments and blank lines in selected `.cs` files and applies the
appropriate source or test limit.

## Options

`srcMax`, `testMax`, `roots`, `excludes`, and `testRoots` control limits and
selection. `srcMax` defaults to 200, `testMax` to 500, and `roots` defaults to
the rule's target roots. `excludes` defaults to `**/*.g.cs`; its entries match
path globs or literal path substrings. `testRoots` defaults to `**/tests/**`
and `**/*.Tests/**`; paths under `tests/` also use the test limit.

## Valid example

A selected source file at or below its configured physical-line limit passes.

```cs
class Ok { }
```


## Counterexample

```cs
class TooLong {
    int a;
    int b;
    int c;
    int d;
    int e;
}
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

The line-count check does not require a C# syntax parse. Readable malformed
C# remains subject to the physical line limit.

## Suppression and related rules

Use file suppression for generated or intentionally exceptional files. See
[`rust-max-lines-per-file`](rust-max-lines-per-file.md) for the Rust analogue.

Caps C# source and test file length by physical line count. Blank lines and
comments count; this rule does not ignore them.

```yaml
rules:
  - rule: csharp-max-lines-per-file
    scope: repository
    options:
      srcMax: 3
      testMax: 10
      excludes:
        - "**/*.g.cs"
      testRoots:
        - "**/tests/**"
        - "**/*.Tests/**"
```

`srcMax` applies to production `.cs` files (default 200). `testMax` applies to
test files (default 500). A file uses the test limit when its normalized path
contains `/tests/`, or when it matches `testRoots` (default `**/tests/**` and
`**/*.Tests/**`). Generated files matching `**/*.g.cs` are excluded by default.
`roots` limits discovery to selected directories. Additional `excludes` are path
globs or substrings.

Counterexample: a 900-line C# source file. Blank lines and comments still count because this rule measures physical lines.

Fix: extract cohesive types into separate files, or move generated output and test helpers into the configured test roots.
