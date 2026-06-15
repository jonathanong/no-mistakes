# `no-mistakes ci impact`

Given changed file(s), list every workflow whose trigger `paths:` /
`paths-ignore:` filters match, along with each job and its resolved permissions.

```sh
no-mistakes ci impact src/api/handler.ts --format json
no-mistakes ci impact .github/workflows/ci.yml --format paths
```

## Options

| Flag | Description |
|------|-------------|
| `--root` | Project root directory (default: current directory). |
| `--config` | Path to config file. |
| `--format` | Output format: `json`, `md`, `yml`, `paths`, `human`. |
| `--json` | Shorthand for `--format json`. |

Pass one or more changed files as positional arguments (relative to `--root` or
absolute).

## Trigger semantics (heuristics)

- Path-filterable events are `push`, `pull_request`, and `pull_request_target`.
- An event with no `paths`/`paths-ignore` runs on any change → reported as
  `always`.
- `paths` matches if any include glob matches; `!`-negations apply
  gitignore-style (last match wins). `paths-ignore` excludes matching files.
- Matching approximates GitHub's filter patterns with `globset`
  (`*` does not cross `/`, `**` does). `+()`/`?()` extglob forms are not
  supported. Matching is case-sensitive.
- Workflows with only `workflow_dispatch`/`schedule`/`workflow_call` are not
  file-triggered and are omitted.
- A job that calls a local reusable workflow (`uses: ./.github/workflows/x.yml`)
  is reported with its `uses` target, but the called workflow's own jobs and
  permissions are not recursively resolved (a documented v1 limitation).
- If an event declares both `paths` and `paths-ignore` (which GitHub disallows),
  a warning is emitted and `paths` is honored.

## Permission semantics (heuristics)

- A job-level `permissions:` block fully overrides the workflow default (GitHub
  does not merge them). The `source` field reports `job`, `workflow`, or
  `default`.
- `read-all` / `write-all` expand to every scope; `{}` grants none.
- When no `permissions:` is set anywhere, the documented restricted default is
  reported with `assumed_default: true` (the real default depends on repository
  settings).

## Output (json)

```json
{
  "changed_files": ["src/api/handler.ts"],
  "workflows": [
    {
      "path": ".github/workflows/ci.yml",
      "name": "CI",
      "trigger": "matched",
      "reusable": false,
      "matched_filters": [{ "event": "push", "pattern": "src/**" }],
      "jobs": [
        {
          "id": "test",
          "permissions": {
            "source": "workflow",
            "scopes": { "contents": "read" },
            "assumed_default": false
          }
        }
      ]
    }
  ],
  "warnings": []
}
```

`paths` format prints one workflow path per line. `human`/`md` render a
`workflow → job → permissions` tree, marking unfiltered triggers `(always)`.

Node API: `ciImpact(options)`.
