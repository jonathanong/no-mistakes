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

The reverse scan adds `partial` and `incompleteFiles` to structured output.
Each incomplete file reports a root-relative `file`, a `reason`
(`operational-error`, `parse-error`, `missing-facts`, or `missing-symbols`), and a
`message`. The list covers the prepared TS/JS candidate inventory: a failed file
may contain an import that cannot be recovered, so its relevance can be unknown.
Recovered imports may still contribute results when a parser diagnostic exists.
`partial: false` means no recorded collection gaps; it does not guarantee that
all runtime references or unsupported syntax were resolved. When `partial` is
true, treat empty lists and zero counts as known results, not proof of absence.
Human output prints the same warnings. `--format paths` retains the path list
without diagnostics; use JSON before making decisions that depend on absence.

With `--no-importers`, these reverse-scan fields are omitted.
