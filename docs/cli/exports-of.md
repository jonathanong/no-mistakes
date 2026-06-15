# `no-mistakes exports-of`

List a file's named exports and who imports each one.

```sh
no-mistakes exports-of src/components/Tabs.tsx --format json
no-mistakes exports-of src/components/Tabs.tsx --no-importers --format json
```

Use this to see a file's public API alongside the consumers of each export in a
single call. The export list is parsed from the target file; the per-export
`importers` come from one reverse import scan (no full graph build).

`--no-importers` skips the reverse scan and returns only the export list
(instant). Re-exports include their resolved source as `resolved`. Namespace
imports (`import * as ns`) are reported at file granularity by
[`importers`](importers.md), not per-export here.

For just the export/import symbol list without consumers, use
[`symbols`](symbols.md).

Key options: `--root`, `--tsconfig`, `--no-importers`, `--format`, and `--json`.

Node API: `exportsOf(options)`.
