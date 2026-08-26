# Agent Guide

Use these tools throughout a change when static codebase intelligence can
reduce missed tests, hidden dependencies, or fragile dynamic patterns.

## Change lifecycle

1. Before editing, identify the owning package and run a focused dependency or
   test-plan query for the files you expect to touch. `tests plan` works without
   a custom `testPlan` block; configuration adds project-specific groups and
   safety policy.
2. After editing, rerun the relevant query against the changed files, then run
   `no-mistakes impacted-checks <files> --format json` to review both selected
   tests and generic checks. Execute only commands you trust and inspect the
   JSON when a result is empty or surprising.
3. Before handoff, run `no-mistakes resolve-check` for moved or renamed modules,
   `no-mistakes check` for configured repository rules, and the Playwright
   coverage check when routes, selectors, or pages changed. Record any
   intentional analyzer limitation for the next agent.

## Command Selection

| Agent question | Command |
| --- | --- |
| What files does this file import? | `no-mistakes dependencies <file> --format json` |
| What files are affected by this change? | `no-mistakes dependents <file> --format paths` |
| What tests should run? | `no-mistakes tests plan <vitest\|jest\|playwright\|python\|go\|cargo\|...> --changed-file <file> --format paths`, or the lower-level `dependents --test` / `playwright related` commands |
| What should I validate before handoff? | `no-mistakes impacted-checks <file...> --format json` |
| What public API does this file expose? | `no-mistakes symbols <file> --include both --format json` |
| Who calls this exported function? | `no-mistakes call-sites <file> <symbol> --format json` |
| Is this export still used? | `no-mistakes dead-exports <file> <symbol> --format json` |
| Do imports still resolve after a move? | `no-mistakes resolve-check <file...> --format json` |
| Is this App Router route tested? | `no-mistakes playwright check --json` |
| Which Playwright tests assert this page/component? | `no-mistakes playwright related <file> --json` |
| Which test IDs/routes/fetches does a test cover? | `no-mistakes playwright tests <test-file> --json` |
| Which API calls can this Next.js route make? | `no-mistakes fetches <route-or-file> --format json` |
| Does this queue job have a worker? | `no-mistakes queues check --format json` |
| What server route file owns this endpoint? | `no-mistakes server routes --format json` |
| Does this component tree call fetch? | `no-mistakes react check <glob> --assert-no-fetch --format json` |
| Which workflows or jobs can this change trigger? | `no-mistakes ci topology-impact --base <ref> --head HEAD --format json` |
| Which Terraform resources depend on this one? | `no-mistakes infra resource-refs <type.name> --format json` |
| Which packages changed in the lockfile? | `no-mistakes lockfile diff --base <ref> --format json` |

## Recommended Agent Instructions

Add project-specific versions of these instructions to `AGENTS.md`, `CLAUDE.md`,
or the repository's agent guide:

```md
Use no-mistakes for structural TS/JS questions before falling back to grep.
Run no-mistakes dependents <changed-file> --format json to inspect impact.
Use no-mistakes tests plan <framework> --changed-file <file> --format json before editing and after the change.
Use no-mistakes impacted-checks <changed-files> --format json before handoff and inspect warnings and fallback_triggered.
Run no-mistakes playwright check --json before finishing Next.js App Router or Playwright work.
Use no-mistakes playwright related <file> to identify Playwright tests for changed pages or selector-bearing components.
Keep test IDs and fetch URLs static unless the project explicitly accepts that the AST tools cannot reason about them.
```

## Pre-Finish Workflows

### TS/JS Module Change

```sh
changed=src/utils.mts
no-mistakes symbols "$changed" --include both --format json
no-mistakes dependents "$changed" --format paths
no-mistakes tests plan vitest --changed-file "$changed" --format paths
no-mistakes impacted-checks "$changed" --format json
```

Use `rg` after `no-mistakes dependents` when you need exact call lines inside
the affected files.

### Next.js App Router Or Playwright Change

```sh
no-mistakes playwright check --json
no-mistakes tests plan playwright --changed-file 'web/app/users/[id]/page.tsx' --format paths
no-mistakes playwright related 'web/app/users/[id]/page.tsx'
no-mistakes playwright tests --json
```

Fix uncovered routes by adding navigation or URL assertions. Fix uncovered
selectors by asserting a stable test hook with `getByTestId(...)` or a supported
CSS selector.

### API Or Fetch Change

```sh
no-mistakes fetches --format json
```

If `no-mistakes fetches` reports dynamic paths, prefer static route strings or small
static wrappers so agents can reason about route-to-API relationships.
When the project uses `eslint-plugin-no-mistakes`, run the project's own
ESLint command so local config, ignores, and package boundaries are respected.

### Queue Or Server Route Change

```sh
no-mistakes queues check --format json
no-mistakes queues related backend/jobs/email.ts --format paths
no-mistakes server routes --format json
no-mistakes server related backend/api/users.ts --format paths
```

Root-scoped `edges` commands default to direct edges. Pass a larger `--depth`
when you want more transitive hops, or omit roots when you want the full edge
list.

### Several Questions In One Node Process

Use `analyzeProject()` when an agent or integration needs several related
reports. The request discovers files, collects facts, and builds each effective
graph once instead of repeating that work in separate CLI processes.

```js
import { analyzeProject } from "no-mistakes";

const report = await analyzeProject({
  root: process.cwd(),
  reports: [
    { type: "dependents", files: ["src/api.ts"] },
    { type: "symbols", files: ["src/api.ts"], include: "both" },
    { type: "testsPlan", framework: "vitest", changedFiles: ["src/api.ts"] },
  ],
});
```

See the [Node/N-API guide](node-api.md) for report types and dedicated APIs.

## Failure Handling

- Empty or surprising dependency results usually mean the wrong `--tsconfig`,
  dynamic imports, unsupported aliases, or external package boundaries.
- Dynamic selectors, fetch URLs, route paths, queue names, and job names should
  be made static when the project expects agent-readable behavior.
- In monorepos, pass `--root` explicitly. Omit `--tsconfig` for normal
  per-workspace ownership resolution; force one package tsconfig only when the
  whole request intentionally uses that resolver scope.
- Treat parse errors as real blockers unless the file is intentionally outside
  the analyzer's supported language set.

## Output Guidance

- Use `--format json` when another tool or agent needs structured data.
- CLI JSON uses `snake_case`; Node results use `camelCase`.
- Use `--format paths` to review paths or trusted command text. Prefer JSON
  command arrays for execution so spaces and shell syntax stay data.
- Use `--format human` for interactive debugging.
- Timings are written only to stderr. Use `--timings` when investigating slow
  graph queries without changing machine-readable stdout.
- Inspect `warnings`, `fallback_triggered`, `fallback_reason`, and
  `empty_result` before concluding that a narrow or empty result is safe.

For a compact operational version of this guide, use the
[packaged agent skill](../skills/no-mistakes/SKILL.md).
