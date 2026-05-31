# `no-mistakes dependents`

Find files, tests, or module nodes that depend on the input files.

```sh
no-mistakes dependents src/utils.mts --test vitest --format paths
no-mistakes dependents src/api.mts#handler --format json
```

Use this for change impact, targeted test selection, and named-export usage.
`FILE#SYMBOL` narrows to dependents of a named export. Namespace imports match
all symbols, so use `rg` on returned files when exact member usage matters.

Key options match [`dependencies`](dependencies.md): `--root`, `--tsconfig`,
`--depth`, `--filter`, `--target-module`, `--relationship`, `--test`,
`--format`, `--json`, and `--timings`.

Node API: `dependents(options)`.
