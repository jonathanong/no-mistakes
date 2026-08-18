# `csharp-max-lines-per-file`

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
