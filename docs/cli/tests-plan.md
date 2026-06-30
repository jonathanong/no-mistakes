# `no-mistakes tests plan`

Select tests to run from changed files, diffs, configured environments, and
dependency graph analysis.

```sh
no-mistakes tests plan vitest --base origin/main --format json
no-mistakes tests plan playwright --changed-file web/app/users/page.tsx --format paths
no-mistakes tests plan dotnet --changed-file dotnet-clients/src/App/FeedService.cs --format paths
no-mistakes tests plan vitest --changed-file web/app/users/users.test.ts --format commands
no-mistakes tests plan swift --changed-file backend/api/feeds.mts --format paths
```

Use this for agent test selection before running expensive suites. Inputs can
come from `--base/--head`, `--changed-file`, `--changed-files`, `--diff`,
`--diff-stdin`, `--diff-command`, or repeatable `--entrypoint`.

Key options: `--root`, `--config`, `--tsconfig`, `--environment`,
`--limit-percent`, `--limit-files`, `--global-config-fallback`, `--format`, and
`--json`.

`--format commands` prints the exact runner commands for selected execution
targets. Use it when an agent needs runnable commands instead of test paths or a
structured plan.

Node API: `testsPlan(options)`.

`dotnet` plans require `tests.dotnet.projects` config. They select changed C#
test files directly and select dependent C# tests through namespace imports,
type references, and `.csproj` `ProjectReference` edges. Command output uses
`dotnet test <project.csproj> --no-restore` plus a class-name filter when the
test file maps cleanly to a fully qualified test class.

`swift` plans require `tests.swift.packages` config. They select changed Swift
tests directly and select dependent Swift tests through Swift graph edges and
HTTP route edges.
