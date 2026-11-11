# `no-mistakes symbols`

Print named exports and imports for TS/JS files.

```sh
no-mistakes symbols src/api.mts --include both --format json
no-mistakes symbols src/api.mts --mode signature-impact --symbol handler --format json
```

Use this before editing public APIs, replacing imports, or asking who consumes a
symbol with `dependents FILE#SYMBOL`.

Usage:

```sh
no-mistakes symbols <FILE>... [--root <PATH>] [--tsconfig <FILE>] [--config <FILE>]
  [--kind <KIND>]... [--include exports|imports|both]
  [--mode list|signature-impact] [--symbol <SYMBOL>]
```

Key options: `--root`, `--tsconfig`, `--config`, repeatable `--kind`, `--include
exports|imports|both`, `--mode`, `--symbol`, `--format`, `--json`, and
`--timings`.

| Option | Default | Description |
| --- | --- | --- |
| `--config <FILE>` | auto-detected | Path to `.no-mistakes.yml` config. |

## Signature impact

Use signature-impact mode before changing a function export signature:

```sh
no-mistakes symbols src/api.mts --mode signature-impact --symbol handler --format json
```

This report groups the symbol definition, export/re-export paths, production
callers, test callers, and suggested focused tests. Signature-impact results
include `unanalyzed`, an array of `{ file, reason }` records. Unrelated failures
are reported there with an `unanalyzed-files` warning. Source text is decoded
lossily before analysis. A failure is an error when it could change the answer:
the target file, transitive importers of the target and their transitive
dependencies, or a raw occurrence of the requested symbol in retained source.
Imports or re-exports of the requested name through an opaque failed module
also fail conservatively, even when that module's link to the target is unknown.
The default `symbols` output is unchanged unless `--mode signature-impact` is set.
Unrelated failures return exit 0; potentially relevant failures return exit 2.
Use JSON or YAML to inspect completeness before consuming `--format paths`.

Node API: `symbols(options)`.
