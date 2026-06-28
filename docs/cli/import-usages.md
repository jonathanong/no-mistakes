# `no-mistakes import-usages`

Report direct string-literal import usages in TS/JS files.

```sh
no-mistakes import-usages --root . --filter 'src/**' --format json
no-mistakes import-usages src/index.mts --format json
```

Use this when checking package dependency declarations from actual source
imports. The report includes static imports, re-exports, dynamic `import()`,
TypeScript `import("pkg")` type references, `require()`, and
`require.resolve()`.

Usage:

```sh
no-mistakes import-usages [FILE]... [--root <PATH>] [--scan-root <PATH>]...
  [--filter <GLOB>]... [--format json|yml|md|paths|human] [--json]
```

JSON output:

```json
{
  "roots": ["."],
  "files": [
    {
      "path": "src/index.mts",
      "imports": [
        {
          "specifier": "react/jsx-runtime",
          "packageName": "react",
          "kind": "static",
          "line": 1,
          "sideEffectOnly": false,
          "reExport": false
        }
      ]
    }
  ]
}
```

`packageName` is derived from the raw specifier only. Relative imports,
absolute imports, package `#imports`, Node built-ins such as `node:fs`, and
incomplete scoped names report `null`.

Node API: `importUsages(options)`, or `analyzeProject({ reports: [{ type:
"importUsages" }] })`.
