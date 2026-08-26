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
      srcMax: 200
      testMax: 500
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
