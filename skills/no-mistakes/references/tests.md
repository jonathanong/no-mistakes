# `tests` command reference

## When to use

Use `tests plan` when the project has a `testPlan:` block configured in
`.no-mistakes.yml`. It is the high-level replacement for `dependents --test` —
it respects per-environment test groups, coverage percentages, and global
fallback triggers.

Use `tests why` when you need to explain or debug a selection. Use
`tests impact` when the exact set of changed files is already known and
`testPlan` isn't needed.

## `tests plan`

Select tests to run from changed files, diffs, and configured environments.

```sh
# Changed-file selection (preferred)
no-mistakes tests plan vitest --changed-file src/utils.mts --format paths
no-mistakes tests plan playwright --changed-file web/app/users/page.tsx --format paths

# Diff-based (from git)
no-mistakes tests plan vitest --base origin/main --format json

# Named environment (from .no-mistakes.yml testPlan)
no-mistakes tests plan vitest --environment prePush --changed-file src/api.mts --format paths
```

Key flags:
- `--changed-file <FILE>` / `--changed-files <FILE>…` — explicit changed files.
- `--base <REF>` / `--head <REF>` — compute changed files from a git diff.
- `--diff <FILE>` / `--diff-stdin` / `--diff-command <CMD>` — supply a diff
  directly.
- `--entrypoint <FILE>` — treat a file as the root regardless of changes.
- `--environment <NAME>` — pick an env group from `testPlan.environments`.
- `--limit-percent <N>` / `--limit-files <N>` — override `testPlan` limits.
- `--global-config-fallback` — run the full suite when no targeted tests are
  found instead of returning nothing.
- `--format paths|json` — `paths` for shell substitution, `json` for agents.

Node API: `testsPlan(options)`.

## `tests why`

Explain the dependency path from a changed file to a selected or skipped test.

```sh
no-mistakes tests why tests/users.test.mts --plan plan.json
no-mistakes tests why tests/users.test.mts --changed src/api.mts --format json
```

Key flags:
- `--plan <FILE>` — path to a previously generated `tests plan` JSON file.
- `--changed <FILE>` — changed file to compute the path from (without a prior plan).
- `--format text|json`.

Node API: `testsWhy(options)`.

## `tests impact`

Impacted tests for specific changed files (no `testPlan` config required).

```sh
no-mistakes tests impact src/utils.mts --format paths
```

Node API: `testsImpact(options)`.

## `tests comment`

Render a plan JSON as a Markdown PR comment.

```sh
no-mistakes tests comment plan.json
no-mistakes tests comment plan.json --out comment.md
```

Node API: `testsComment(options)`.

## `testPlan` configuration

In `.no-mistakes.yml`:

```yaml
test_plan:
  fullSuiteTriggers:
    vitest:
      - package.json
      - pnpm-lock.yaml
  environments:
    prePush:
      - name: direct
        type: direct
      - name: dependencies
        type: dependencies
        limit:
          percent: 20
    pullRequest:
      - name: coverage
        type: coverage
        limit:
          files: 50
```

Types: `direct`, `dependencies`, `coverage`, `sample`.
Consult `docs/configuration/test-plan.md` for the full schema.
