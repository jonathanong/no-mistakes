# `no-mistakes symbols`

Print named exports and imports for TS/JS files.

```sh
no-mistakes symbols src/api.mts --include both --format json
```

Use this before editing public APIs, replacing imports, or asking who consumes a
symbol with `dependents FILE#SYMBOL`.

Key options: `--root`, `--tsconfig`, repeatable `--kind`, `--include
exports|imports|both`, `--format`, `--json`, and `--timings`.

Node API: `symbols(options)`.
