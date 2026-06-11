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
callers, test callers, and suggested focused tests. The default `symbols`
output is unchanged unless `--mode signature-impact` is set.

Node API: `symbols(options)`.
