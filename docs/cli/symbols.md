# `no-mistakes symbols`

Print named exports and imports for TS/JS files.

```sh
no-mistakes symbols src/api.mts --include both --format json
no-mistakes symbols src/api.mts --mode signature-impact --symbol handler --format json
```

Use this before editing public APIs, replacing imports, or asking who consumes a
symbol with `dependents FILE#SYMBOL`.

Key options: `--root`, `--tsconfig`, repeatable `--kind`, `--include
exports|imports|both`, `--mode`, `--symbol`, `--format`, `--json`, and
`--timings`.

## Signature impact

Use signature-impact mode before changing a function export signature:

```sh
no-mistakes symbols src/api.mts --mode signature-impact --symbol handler --format json
```

This report groups the symbol definition, export/re-export paths, production
callers, test callers, and suggested focused tests. The default `symbols`
output is unchanged unless `--mode signature-impact` is set.

Node API: `symbols(options)`.
