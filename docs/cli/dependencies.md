# `no-mistakes dependencies`

Find files and module nodes that the input files depend on.

```sh
no-mistakes dependencies src/api.mts --root . --format json
```

Use this when an agent needs upstream context before editing: imports,
workspace/package edges, routes, queues, tests, markdown links, CI, HTTP,
process, asset, and React edges can all be included.

Use `--relationship route-import` when you need the conservative runtime module
closure used by Playwright route analysis. It follows runtime static
imports/re-exports and literal dynamic imports without function-reachability
pruning; it excludes type-only imports and `require()`. This is distinct from
`--relationship route`, which follows URL route references, Playwright route
tests, and Next.js layouts.

Key options: `--tsconfig`, `--depth`/`--max-depth`, repeatable `--filter`,
repeatable `--target-module`, repeatable `--relationship`, repeatable `--test`,
`--format`, `--json`, and `--timings`.

`FILE#SYMBOL` is not meaningful for dependencies; symbol filtering is for
[`dependents`](dependents.md) and [`related`](related.md).

Node API: `dependencies(options)`.
