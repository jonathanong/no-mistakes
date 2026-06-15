# CLI Commands

Prefer JSON for agent parsing and paths for command substitution:

```sh
no-mistakes dependents src/utils.mts --format json
no-mistakes dependents src/utils.mts --test vitest --format paths
```

Global `--jobs <N>` controls rayon worker count for commands that parallelize
analysis.

## Command Index

| Command | Purpose |
| --- | --- |
| [`dependencies`](dependencies.md) | Files and modules reachable from changed files. |
| [`dependents`](dependents.md) | Files, tests, and modules affected by changed files. |
| [`related`](related.md) | Alias for `dependents`; useful when agents ask for impact. |
| [`symbols`](symbols.md) | Named exports and imports in TS/JS files. |
| [`fetches`](fetches.md) | Next.js routes mapped to static fetch API calls. |
| [`check`](check.md) | Configured project-wide checks. |
| [`lockfile`](lockfile.md) | Show which packages changed between two lockfile versions. |
| [`tests`](tests.md) | Test plan, impact, explanation, comments, and graphs. |
| [`playwright`](playwright.md) | Playwright route, selector, and assertion coverage. |
| [`react`](react.md) | React component trait analysis and fetch checks. |
| [`queues`](queues.md) | Queue producer/worker graph checks. |
| [`server`](server.md) | Express, Hono, and Koa route graphs. |
| [`ci`](ci.md) | GitHub Actions impact ([`ci-impact`](ci-impact.md)) and env usage ([`ci-env`](ci-env.md)). |
| [`impacted-checks`](impacted-checks.md) | Minimal local validation commands for changed files. |
| [`infra`](infra.md) | Terraform/OpenTofu resource, module, and output relationships. |
| [`swift`](swift.md) | Swift package importers and covering test targets. |

## Shared Output Formats

Most commands accept `--format json|yml|md|paths|human` plus `--json`.
`human` is for reading, `json` is for agents, and `paths` is for follow-up test
or lint commands.

## Examples And Counterexamples

Good agent inputs are rooted, structured, and scoped:

```sh
no-mistakes dependents src/api.mts#handler --root . --tsconfig tsconfig.json --format json
no-mistakes tests plan vitest --base origin/main --format paths
```

Avoid relying on human output when another tool will parse the result:

```sh
# Counterexample: hard for an agent to parse robustly.
no-mistakes dependents src/api.mts
```

Prefer a narrower relationship when the question is narrow:

```sh
no-mistakes dependencies src/api.mts --relationship import --format json
```

Use [`Graph edges`](../graph-edges.md) for the complete supported edge-kind and
relationship-filter list.
