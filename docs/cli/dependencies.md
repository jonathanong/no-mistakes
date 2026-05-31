# `no-mistakes dependencies`

Find files and module nodes that the input files depend on.

```sh
no-mistakes dependencies src/api.mts --root . --format json
```

Use this when an agent needs upstream context before editing: imports,
workspace/package edges, routes, queues, tests, markdown links, CI, HTTP,
process, asset, and React edges can all be included.

Key options: `--tsconfig`, `--depth`/`--max-depth`, repeatable `--filter`,
repeatable `--target-module`, repeatable `--relationship`, repeatable `--test`,
`--format`, `--json`, and `--timings`.

`FILE#SYMBOL` is not meaningful for dependencies; symbol filtering is for
[`dependents`](dependents.md) and [`related`](related.md).

Node API: `dependencies(options)`.
