# `finite-set-consistency`

Compares named finite string sets extracted from source files and file paths.

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
      comparisons:
        - left: routeType
          right: routeFiles
```

Supported set kinds are `ts-string-union`, `ts-const-object-keys`,
`ts-const-object-property`, `sql-enum`, and `path-regex-capture`.

Counterexample: a TypeScript union includes `"settings"` but no matching route
file exists.

Fix: add the missing value to the other set, remove stale values, or narrow the
configured extraction.
