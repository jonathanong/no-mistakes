# `github-actions-workflow-script-paths`

## Why and when

A path-filtered workflow that executes a repository script outside its own
`on.<event>.paths` (or inside `on.<event>.paths-ignore`) will not run when that
script is edited. A broken change to the script then merges without exercising
the workflow that depends on it.

Enable this opt-in rule to verify that every repository script invoked by a
path-filtered workflow falls inside that workflow's path filters. Configure
`options.scriptDirs` with the exact repository-relative script directories to
track; no directory names are hardcoded.

## What it catches

The rule scans direct workflow files under `.github/workflows/*.yml` and
`.github/workflows/*.yaml` that declare `paths` or `paths-ignore` on any
checked event (`push`, `pull_request`, or `pull_request_target`). Workflows
without path filters stay silent.

For each path-filtered workflow, the rule parses every job `run` step with the
shared shell parser and follows local composite actions (`uses: ./...`) up to
four levels deep. A deeper composite, or a shell command nested more than four
levels, is unanalyzable. It collects repository script invocations inside the
configured `scriptDirs`:

- Direct path execution (for example `./scripts/check.sh` or `tools/ci/lint.sh`),
  resolved against workflow, job, and step `working-directory` as well as
  literal `cd` commands in the same step.
- Script interpreter and runner invocations (`bash`, `sh`, `zsh`, `dash`, `ash`,
  `ksh`, `source`, `.`, `node`, `tsx`, `ts-node`, `bun`, `deno`, `python`,
  `python3`, `ruby`, `perl`, `pwsh`, `npx`, `pnpx`, `pnpm exec`, `npm exec`,
  `yarn exec`, and `uv run`), plus nested `bash -c` / `sh -c` / `zsh -c` literal
  scripts and `$(...)` / backtick command substitutions.

Each resolved script path is evaluated against every checked path-filtered
event on that workflow using ordered gitignore-style glob matching (including
`!` negations on `paths`). GitHub's `?` and `+` quantifiers are not that
matcher: a pattern that uses either is unanalyzable. `paths-ignore` is
exclusion-only, so the remediation tells you to narrow the ignore or switch to
`paths`. When any path-filtered event fails to cover the script, the
rule reports the invocation line (or the `uses:` line for a script run inside a
local composite action) and names the missing event filters.

When a workflow cannot be proven statically—invalid YAML, unreadable bytes,
YAML merge keys or unrecognized `on:` / `paths:` shapes, dynamic `paths`
expressions, unsupported extended globs, GitHub `?` or `+` quantifiers, dynamic
command names, `eval`, dynamic or clustered `bash -c`, dynamic script paths
under `scriptDirs`, dynamic `working-directory`, `cd` or package-runner
directory changes this check cannot model, unterminated shell syntax, nesting
past four levels, unreadable local composite actions, a non-string `run` inside
a local composite action, or a nested `./` action that is dynamic or cannot be
resolved—the rule abstains from reporting a pass and records an `unanalyzable`
diagnostic governed by `options.unanalyzable` (default `advise`).

## Options and defaults

`options.scriptDirs` defaults to `[]`, so the rule reports nothing until
configured. Invalid entries (blank strings, absolute paths, backslashes, `..`,
glob wildcards, or duplicates), invalid `include` globs, or unknown
`events` fail at configuration load.

| Option | Default | Behavior |
| --- | --- | --- |
| `scriptDirs` | `[]` | Repository-relative literal script directories (for example `["scripts", "tools/ci"]`). Empty list disables reporting. |
| `include` | `[]` | Optional repo-relative globs narrowing direct workflow files under `.github/workflows/*.{yml,yaml}`. |
| `events` | `[]` | Optional subset of `push`, `pull_request`, `pull_request_target` to check. Empty checks all three when path-filtered. |
| `unanalyzable` | `advise` | Controls diagnostics for unprovable triggers or scripts (`advise`, `report`, or `ignore`). |

```yaml
rules:
  - rule: github-actions-workflow-script-paths
    scope: repository
    options:
      scriptDirs:
        - scripts
        - tools/ci
```

## Valid example

Every script invoked from `scripts/` and `tools/ci/` matches `on.pull_request.paths`.
Commands that only print a path without executing it stay silent:

```yaml
# .github/workflows/ci.yml
name: CI
on:
  pull_request:
    paths:
      - "apps/**"
      - "scripts/**"
      - "tools/ci/**"
jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: |
          ./scripts/check.sh
          node scripts/verify.mjs
          pnpm exec tsx tools/ci/lint.ts
```

## Counterexample

`on.pull_request.paths` covers only `apps/**`, so editing `scripts/check.sh` or
`tools/ci/lint.ts` does not trigger the workflow:

```yaml
# .github/workflows/ci.yml
name: CI
on:
  pull_request:
    paths:
      - "apps/**"
jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: |
          ./scripts/check.sh
          pnpm exec tsx tools/ci/lint.ts
```

## Fix

Add the script path or its script directory to every path-filtered event on the
workflow (or remove the exclusion from `paths-ignore`):

```yaml
# .github/workflows/ci.yml
name: CI
on:
  pull_request:
    paths:
      - "apps/**"
      - "scripts/**"
      - "tools/ci/**"
jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: |
          ./scripts/check.sh
          pnpm exec tsx tools/ci/lint.ts
```

## Suppression

Use a YAML comment on or above the reported invocation or `uses:` line, or at
the top of the workflow file:

- `# no-mistakes-disable-next-line github-actions-workflow-script-paths`
- `# no-mistakes-disable-line github-actions-workflow-script-paths`
- `# no-mistakes-disable-file github-actions-workflow-script-paths`

Directives apply to both uncovered-script findings and `unanalyzable`
advisories.

```yaml
# .github/workflows/ci.yml
steps:
  - run: |
      # no-mistakes-disable-next-line github-actions-workflow-script-paths
      ./scripts/manual-helper.sh
```

## Analysis

### Effort

- Runs at `max` only. Configuring `effort: low` records a warning and runs at
  `max`.

### Unanalyzable inputs

- Invalid YAML, non-UTF-8 bytes, YAML merge or non-string keys in root or `on:`,
  unrecognized `on:` / `paths:` / `paths-ignore:` shapes, dynamic `paths`
  expressions or unsupported extended globs, dynamic commands, `eval`, dynamic
  `bash -c` (including clustered forms such as `bash -ce`), dynamic script paths
  under `scriptDirs` (including an expansion in the middle of a path segment),
  dynamic `working-directory`, `cd -` when the previous directory is unknown,
  `cd` options this check cannot model, package-runner directory flags such as
  `pnpm --dir`, unquoted shell filename patterns such as `scripts/*.sh`,
  a `cd` inside `if` / `while` / `for` / `case`, a `run` step whose `shell`
  this check cannot parse, embedded `**` such as `pre**post`, unterminated
  shell syntax, composite or shell nesting past four
  levels, and unreadable or malformed local composite actions
  never pass silently; they emit a diagnostic governed by `options.unanalyzable`
  (default `advise`).

## Related rules

- [`github-actions-run-command-policy`](github-actions-run-command-policy.md)
  shares the same workflow shell parser to check command flags and arguments.
- [`vitest-ci-path-coverage`](vitest-ci-path-coverage.md) checks Vitest
  project globs against path-filter action maps.
- [`github-actions-local-action-checkout`](github-actions-local-action-checkout.md)
  verifies local composite actions are preceded by a covering checkout.

## Invalid UTF-8

Workflow bytes that are not valid UTF-8 cannot be parsed safely and produce an
`unanalyzable` diagnostic on the line of the first invalid byte.
