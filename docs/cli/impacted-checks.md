# `no-mistakes impacted-checks`

Return the minimal set of local validation commands to run for a set of changed
files: test commands derived from the [`tests plan`](tests-plan.md) engine plus
generic checks (lint, typecheck, …) from the [`checks`](../configuration/checks.md)
config block.

```sh
no-mistakes impacted-checks src/api/handler.ts --format paths
no-mistakes impacted-checks --base origin/main --json
no-mistakes impacted-checks src/api/handler.ts --json --timings
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
| `--timings` | Emit analysis phase durations to stderr. |
| `--verbose-timings` | Also emit deterministic one-pass work counters; implies timings. |

Changed files may also be passed as positional arguments.

## How it works

- Changed files, repository files, parsed facts, and the dependency graph are
  prepared once per invocation and reused across the
  configured frameworks (`dotnet`, `vitest`, `playwright`, and `swift`). Each
  selected `TestExecutionTarget` becomes a `test` check; emitted commands match
  `tests plan` exactly.
- Each `checks.commands` entry whose `include` globs match a changed file
  produces a `generic` check. `fileArgs: append` adds the matched files as
  trailing arguments; `fileArgs: none` runs the command once.
- Commands are deduped and sorted. If the test-plan engine triggers a
  full-suite fallback (e.g. a global config change), `fallback_triggered` is set.

`--timings` emits one deterministic diagnostics block after analysis. Stable
phase names include `prepare`,
`discover.<framework>`, `select.<framework>`, `generic-checks`, and `total`, plus
`graph` when dependency analysis is needed. Phase durations exclude nested work,
so the first selection phase does not double-count the lazy graph build.
Diagnostics go only to stderr, so stdout remains byte-compatible and safe to
parse or pipe. If a phase fails, stderr reports the phase and elapsed time before
the normal actionable error. See [Performance diagnostics](diagnostics.md) for
the shared stderr contract and verbose work counters.

`--diff-stdin` / `--diff-command` are not supported by this command; use a
reusable `--diff <file>` input instead.

Known limitation: an explicit changed path that is a symlink is canonicalized to
its target before glob matching (the shared change-collection step resolves
existing paths for the dependency graph), so a `checks` glob written against the
symlink path may not match. Match against the resolved target path instead.

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

Node API: `impactedChecks(options)`. Pass `timings: true` to receive ordered
`{ phase, duration_ms }` entries in the returned report; Node timing collection
does not write progress to stderr.
