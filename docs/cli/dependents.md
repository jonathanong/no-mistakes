# `no-mistakes dependents`

Find files, tests, or module nodes that depend on the input files.

```sh
no-mistakes dependents src/utils.mts --test vitest --format paths
no-mistakes dependents src/api.mts#handler --format json
```

Use this for change impact, targeted test selection, and named-export usage.
`FILE#SYMBOL` narrows to dependents of a named export. Namespace imports match
all symbols, so use `rg` on returned files when exact member usage matters.

Use `--relationship call` for reverse impact through statically resolved
lexical calls. Direct named imports, static namespace-member imports, and
explicit named re-exports are followed when their target is unambiguous;
`--depth 1` is the direct caller boundary and
`--depth 0` returns no callers. Computed members, dynamic callees, globals, and
ambiguous star re-exports are not connected because a guessed edge would make
impact results unsound. Call edges are opt-in and are excluded by
`--relationship all`.

Use `--relationship workflow` for reverse impact through tracked GitHub Actions
jobs, steps, local reusable workflows/actions, literal run targets, and
same-run artifact handoffs. Workflow virtual IDs are
`workflow.yml#job:<job>` and `workflow.yml#job:<job>/step:<index>`. This is
separate from the legacy `--relationship ci` workflow-file-to-Rust-binary
Cargo-invocation edge.

Use `--relationship route-import` to find files that conservatively reach a
module through runtime static imports/re-exports or literal dynamic imports,
including imports inside functions whose call reachability is unknown. It
excludes type-only imports and `require()` and is distinct from the URL-routing
edges selected by `--relationship route`.

Use `--relationship trpc` for reverse impact through static tRPC procedure
calls and the router files that declared them. Virtual IDs are
`src/router.ts#procedure:user.get`. Empty `projects.*.trpc.routers` lists
disable extraction; `--relationship all` omits these edges.

Use `--relationship resource` to find runtime consumers of a tracked resource.
Only literal supported filesystem and static glob calls become resource edges;
dynamic paths are reported by test-impact diagnostics rather than guessed here.

Key options match [`dependencies`](dependencies.md): `--root`, `--tsconfig`,
`--depth`, `--filter`, `--target-module`, `--relationship`, `--test`,
`--format`, `--json`, and `--timings`.

Node API: `dependents(options)`.
