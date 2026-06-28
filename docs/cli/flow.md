# `no-mistakes flow`

Print a compact dependency or dependent flow around one file or exported symbol.

```sh
no-mistakes flow src/api/users.mts --direction deps --depth 2 --format json
no-mistakes flow src/api/users.mts#handler --direction dependents --depth 1 --format md
```

Targets use the same `file#symbol` syntax as dependency graph commands. The
report includes the target node, visited nodes, and canonical dependency edges.

Key options: `--root`, `--config`, `--tsconfig`, `--direction`,
`--depth`, repeatable `--relationship`, `--format`, and `--json`.

Use `--direction deps` to inspect what a file or symbol consumes,
`--direction dependents` to inspect callers, and `--direction both` for a small
bidirectional slice.

Node API: `flow(options)`.
