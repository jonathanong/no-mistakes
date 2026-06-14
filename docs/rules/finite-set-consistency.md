# `finite-set-consistency`

Compares named finite string sets extracted from source files, structured
config, docs, and file paths.

```yaml
rules:
  - rule: finite-set-consistency
    scope: repository
    options:
      sets:
        - name: routeType
          file: src/routes/types.ts
          kind: ts-string-union
          target: RouteName
        - name: routeFiles
          kind: path-regex-capture
          pattern: "^src/routes/(?<value>[^/]+)\\.ts$"
        - name: workspaceExcludes
          file: pnpm-workspace.yaml
          kind: yaml-sequence
          key: minimumReleaseAgeExclude
        - name: dependabotGlobs
          file: .github/dependabot.yml
          kind: yaml-sequence
          key: updates.0.cooldown.exclude
      comparisons:
        - left: routeType
          right: routeFiles
        - left: workspaceExcludes
          right: dependabotGlobs
          mode: glob-coverage
```

Supported set kinds are `ts-string-union`, `ts-const-object-keys`,
`ts-const-object-property`, `ts-array-literal`, `ts-const-array-property`,
`yaml-sequence`, `markdown-table-code-cells`, `sql-enum`, and
`path-regex-capture`.

Comparison modes:

- `equal-set` is the default and requires both sets to contain the same values.
- `glob-coverage` requires every left value to be matched by at least one glob
  string from the right set.
- `mention` requires every left value to appear in the right extracted mention
  set, such as markdown table code cells.

Counterexample: a TypeScript union includes `"settings"` but no matching route
file exists, a workspace YAML allowlist names a package missing from a TS
registry, a registry package is not covered by any dependabot glob, or a package
is missing from a markdown policy table.

Fix: add the missing value to the other set, remove stale values, or narrow the
configured extraction.

Suppression: use `no-mistakes` suppression directives. Findings currently report
line 1 for finite set mismatches.
