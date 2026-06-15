# `no-mistakes impacted-checks`

Return the minimal set of local validation commands to run for a set of changed
files: test commands derived from the [`tests plan`](tests-plan.md) engine plus
generic checks (lint, typecheck, …) from the [`checks`](../configuration/checks.md)
config block.

```sh
no-mistakes impacted-checks src/api/handler.ts --format paths
no-mistakes impacted-checks --base origin/main --json
```

## Options

| Flag | Description |
|------|-------------|
| `--root` | Project root directory (default: current directory). |
| `--config` | Path to config file. |
| `--tsconfig` | Path to tsconfig.json for alias resolution. |
| `--base` / `--head` | Git refs to diff for changed files. |
| `--changed-file` | Specific changed file (repeatable). |
| `--changed-files` | File listing changed files, one per line. |
| `--diff` | Unified diff file. |
| `--format` | Output format: `json`, `md`, `yml`, `paths`, `human`. |
| `--json` | Shorthand for `--format json`. |

Changed files may also be passed as positional arguments.

## How it works

- For each configured framework (`vitest`, `playwright`, `swift`), the
  `tests plan` engine selects impacted tests and attaches concrete
  `TestExecutionTarget` commands. Each target becomes a `test` check; the
  emitted commands match `tests plan` exactly.
- Each `checks.commands` entry whose `include` globs match a changed file
  produces a `generic` check. `fileArgs: append` adds the matched files as
  trailing arguments; `fileArgs: none` runs the command once.
- Commands are deduped and sorted. If the test-plan engine triggers a
  full-suite fallback (e.g. a global config change), `fallback_triggered` is set.

`--diff-stdin` / `--diff-command` are intentionally unsupported here because the
inputs are read more than once; use `--diff <file>` instead.

## Output

`paths` format prints each command joined by spaces, one per line — ready to
pipe into a shell loop:

```sh
no-mistakes impacted-checks src/foo.ts --format paths | while read -r cmd; do eval "$cmd"; done
```

```json
{
  "changed_files": ["src/foo.ts"],
  "checks": [
    { "name": "eslint", "kind": "generic", "command": ["pnpm", "exec", "eslint", "src/foo.ts"], "files": ["src/foo.ts"] },
    { "name": "vitest", "kind": "test", "command": ["vitest", "--project", "unit", "src/foo.test.ts"], "files": ["src/foo.test.ts"] }
  ],
  "warnings": [],
  "fallback_triggered": false
}
```

Node API: `impactedChecks(options)`.
