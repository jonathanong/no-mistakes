# CI workflow analysis

The `ci` block configures the [`no-mistakes ci`](../cli/ci.md) workflow-graph
commands.

```yaml
ci:
  # Directories (relative to root) scanned for GitHub Actions workflow YAML.
  # Defaults to [.github/workflows].
  workflowDirs:
    - .github/workflows
  # Directories holding local action descriptors used by workflow dependency
  # edges. Action internals are not inlined. Defaults to [].
  actionDirs: []
```

| Key | Default | Description |
| --- | --- | --- |
| `workflowDirs` | `[.github/workflows]` | Where to find `*.yml` / `*.yaml` workflows. |
| `actionDirs` | `[]` | Roots for local action descriptors resolved by workflow dependency edges. |

Workflow discovery examines direct children of each configured directory and
uses Git visibility: tracked workflows and untracked workflows that are not
ignored. Outside a Git checkout, `.gitignore` and `.ignore` files are still
applied.

In a monorepo, point `--root` at the git root (where `.github/workflows` lives)
or set `workflowDirs` to the correct relative path. Changed-file paths in
`ci impact` are compared against repo-root-relative workflow path filters.
