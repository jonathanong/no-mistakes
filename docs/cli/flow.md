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

`--relationship workflow` includes GitHub Actions virtual job and step nodes.
Their IDs are `workflow.yml#job:<job>` and
`workflow.yml#job:<job>/step:<zero-based-index>`; JSON Flow nodes identify
them with `kind: "workflow-job"` or `kind: "workflow-step"` plus
`workflowFile`, `job`, and (for steps) `step`. Use the precise workflow filters
when only one semantic is needed; each retains only the structural job/step
bridges required to traverse it. `ci` remains the separate legacy
workflow-file-to-Rust-binary relationship.

Node API: `flow(options)`.
