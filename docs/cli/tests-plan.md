# `no-mistakes tests plan`

Select tests to run from changed files, diffs, configured environments, and
dependency graph analysis.

```sh
no-mistakes tests plan vitest --base origin/main --format json
no-mistakes tests plan playwright --changed-file web/app/users/page.tsx --format paths
no-mistakes tests plan swift --changed-file backend/api/feeds.mts --format paths
```

Use this for agent test selection before running expensive suites. Inputs can
come from `--base/--head`, `--changed-file`, `--changed-files`, `--diff`,
`--diff-stdin`, `--diff-command`, or repeatable `--entrypoint`.

Key options: `--root`, `--config`, `--tsconfig`, `--environment`,
`--limit-percent`, `--limit-files`, `--global-config-fallback`, `--format`, and
`--json`.

Node API: `testsPlan(options)`.


`swift` plans require `tests.swift.packages` config. They select changed Swift
tests directly and select dependent Swift tests through Swift graph edges and
HTTP route edges.
