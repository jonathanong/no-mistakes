# `no-mistakes ci env`

Find every workflow definition and `${{ env.VAR }}` reference of an environment
variable across the configured workflow directories.

```sh
no-mistakes ci env GITHUB_TOKEN --format json
no-mistakes ci env CARGO_TERM_COLOR --format paths
```

## Options

| Flag | Description |
|------|-------------|
| `--root` | Project root directory (default: current directory). |
| `--config` | Path to config file. |
| `--format` | Output format: `json`, `md`, `yml`, `paths`, `human`. |
| `--json` | Shorthand for `--format json`. |

The variable name is a positional argument and is matched case-sensitively.

## Semantics (heuristics)

- Definitions are read from structured `env:` blocks at the workflow, job, and
  step scopes.
- References are a textual scan of every string scalar for a
  `${{ … env.VAR … }}` expression, attributed to the nearest enclosing scope.
  Computed expressions (e.g. `env[format(...)]`) are not resolved.
- Exact line numbers are omitted because the YAML parser discards spans. Use
  `rg 'env.VAR' <file>` for line-level locations.

## Output (json)

```json
{
  "variable": "CODECOV_TOKEN",
  "files": [
    {
      "path": ".github/workflows/ci.yml",
      "locations": [
        { "kind": "definition", "scope": "workflow", "value": "${{ secrets.CODECOV_TOKEN }}" },
        { "kind": "reference", "scope": "step", "job": "tests" }
      ]
    }
  ],
  "warnings": []
}
```

`paths` format prints one file path per line. `human`/`md` render a
`file → location` tree.

Node API: `ciEnv(options)`.
