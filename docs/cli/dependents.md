# `no-mistakes dependents`

Find files, tests, or module nodes that depend on the input files.

```sh
no-mistakes dependents src/utils.mts --test vitest --format paths
no-mistakes dependents src/api.mts#handler --format json
```

Use this for change impact, targeted test selection, and named-export usage.
`FILE#SYMBOL` narrows to dependents of a named export. Namespace imports match
all symbols, so use `rg` on returned files when exact member usage matters.

Use `--relationship route-import` to find files that conservatively reach a
module through runtime static imports/re-exports or literal dynamic imports,
including imports inside functions whose call reachability is unknown. It
excludes type-only imports and `require()` and is distinct from the URL-routing
edges selected by `--relationship route`.

Use `--relationship resource` to find runtime consumers of a tracked resource.
Only literal supported filesystem and static glob calls become resource edges;
dynamic paths are reported by test-impact diagnostics rather than guessed here.

Key options match [`dependencies`](dependencies.md): `--root`, `--tsconfig`,
`--depth`, `--filter`, `--target-module`, `--relationship`, `--test`,
`--format`, `--json`, and `--timings`.

Node API: `dependents(options)`.
