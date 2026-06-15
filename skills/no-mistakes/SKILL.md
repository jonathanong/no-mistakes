---
name: no-mistakes
description: Answer structural TS/JS and app-graph questions deterministically. Use for imports, dependents, named exports/imports, test impact, test planning, Playwright coverage, React traits, queue/server graphs, fetches, lockfile diffs, and global no-mistakes checks.
allowed-tools: Bash(no-mistakes:*) Bash(rg:*) Read Glob
---

# No Mistakes

Use `no-mistakes` before `rg` when the question is structural: what a TS/JS file
imports, who imports it, what it exports, which tests are related, whether a
queue job is connected, which server route owns an endpoint, what a Next.js
route fetches, or what React traits a component has.

## Command Selection

| Question | Tool |
| --- | --- |
| What does this file transitively import? | `no-mistakes dependencies <file>` |
| Which files are affected by touching this file? | `no-mistakes dependents <file>` |
| Which files directly import this one file? (fast) | `no-mistakes importers <file>` |
| Which files import this named export? | `no-mistakes dependents <file>#SYMBOL` |
| What does this file export, and who imports each export? | `no-mistakes exports-of <file>` |
| Is this export still used anywhere? (yes/no) | `no-mistakes dead-exports <file> [NAME...]` |
| Where is this function called, and with what argument shapes? | `no-mistakes call-sites <file> SYMBOL` |
| Do all imports in this file resolve? | `no-mistakes resolve-check <file>` |
| What must I update before changing this function signature? | `no-mistakes symbols <file> --mode signature-impact --symbol SYMBOL --format json` |
| Which tests should rerun? | `no-mistakes tests plan vitest --changed-file <file> --format paths` |
| Which tests should rerun? (lower-level fallback) | `no-mistakes dependents <file> --test vitest --format paths` |
| Why was this test selected? | `no-mistakes tests why <test> --plan plan.json` |
| What does this module export/import? | `no-mistakes symbols <file> --include both` |
| What React traits does this component have? | `no-mistakes react analyze <glob>` |
| Does this component tree call fetch? | `no-mistakes react check <glob> --assert-no-fetch` |
| Which callsites render this component, and with what props? | `no-mistakes react usages <file>#SYMBOL` |
| Are all App Router routes/selectors covered by Playwright? | `no-mistakes playwright check` |
| Which Playwright tests cover this page/component? | `no-mistakes playwright related <file>` |
| What does this Playwright test assert? | `no-mistakes playwright tests <test-file>` |
| Which API calls does this Next.js route make? | `no-mistakes fetches <route-or-file>` |
| Which packages changed between two lockfile refs? | `no-mistakes lockfile diff --base <ref>` |
| Which queue producer/worker files are connected? | `no-mistakes queues related <file>` |
| Are queue producers/workers unmatched? | `no-mistakes queues check` |
| What server routes exist? | `no-mistakes server routes` |
| Which server route files are related? | `no-mistakes server related <file>` |
| Raw queue/server edges for debugging | `no-mistakes queues edges [file]` / `no-mistakes server edges [file]` |
| Which Terraform/OpenTofu resources reference this resource? | `no-mistakes infra resource-refs <type>.<name>` |
| What outputs does this Terraform module export and who consumes them? | `no-mistakes infra outputs <module-dir>` |
| Which tests cover this `.tf` file? | `no-mistakes infra test-for <tf-file>` |
| Which Swift files import this file/type? | `no-mistakes swift importers <file>` |
| Which Swift test targets cover this file? | `no-mistakes swift test-targets <file>` |
| Run configured project checks in parallel | `no-mistakes check` |
| What edge kinds are supported? | Read `references/decision-tree.md` or https://github.com/jonathanong/no-mistakes/blob/main/docs/graph-edges.md |
| Plain text, comments, log messages, exact call lines | `rg` |

## Quick Workflow

```bash
# Machine-readable graph query
no-mistakes dependents src/utils.mts --root /path/to/project --format json

# Test selection (preferred over dependents --test)
no-mistakes tests plan vitest --changed-file src/utils.mts --format paths
no-mistakes tests plan playwright --changed-file web/app/users/page.tsx --format paths

# Explain why a test was selected
no-mistakes tests why tests/users.test.mts --plan plan.json

# Public API and imports
no-mistakes symbols src/api.mts --include both --format json
no-mistakes symbols src/api.mts --mode signature-impact --symbol handler --format json

# Playwright coverage gate before finishing Next.js / Playwright work
no-mistakes playwright check --json
no-mistakes playwright related web/app/users/page.tsx --json

# Page-to-API coupling
no-mistakes fetches web/app/users/page.tsx --format json

# Lockfile diff (integrates with tests plan)
no-mistakes lockfile diff --base origin/main --format json

# Queue and server graph checks
no-mistakes queues check --format json
no-mistakes server routes --format json
```

Prefer `--format json` for agent parsing and `--format paths` for command
substitution. `--timings` writes phase timings to stderr on graph, queue, and
server commands.

For repeated graph/symbol/playwright/project queries in the same process,
prefer `analyzeProject({reports:[…]})` from the async Node API documented at
https://github.com/jonathanong/no-mistakes/blob/main/docs/node-api.md — it
shares a single graph build across all requested
reports. Note: `analyzeProject` does not support `testsPlan`, `fetches`, or
`lockfileDiff`; call those dedicated Node API functions directly.

## Graph Options

`dependencies`, `dependents`, and `related` support:

- `--root <PATH>` for the project root.
- `--tsconfig <FILE>` for path aliases; pass this explicitly in monorepos.
- `--depth <N>` to limit traversal depth.
- `--filter <GLOB>` to include only matching files; repeatable.
- `--target-module <GLOB>` to include only matching external module nodes (useful with `--relationship package`).
- `--test vitest|playwright|cargo` to filter to test files.
- `--relationship import|import-static|import-dynamic|import-type|import-require|workspace|package|test|route|queue|md|ci|http|process|asset|react|swift|terraform|all`.
- `--direction deps|dependents|both` for `queues related` and `server related`.
- `--format json|md|yml|paths|human`, `--json`, `--timings` (stderr), and `--jobs`.

`FILE#SYMBOL` works only for `dependents`/`related`, not `dependencies`.
Namespace imports match all symbols; use `rg` on returned files to confirm exact
member usage.

## When To Read References

- `references/decision-tree.md`: choosing commands, relationships, filters, and
  outputs.
- `references/dependencies.md`: full `dependencies` reference.
- `references/dependents.md`: full `dependents`/`related` reference and
  `FILE#SYMBOL` behavior.
- `references/symbols.md`: full `symbols` reference.
- `references/lightweight-queries.md`: full `importers`, `exports-of`,
  `dead-exports`, `call-sites`, and `resolve-check` reference.
- `references/tests.md`: full `tests plan/why` reference and `testPlan` config.
- `references/playwright.md`: full `playwright` command reference.
- `references/monorepo-resolution.md`: tsconfig paths and workspace packages.
- `references/limits-and-fallbacks.md`: unsupported forms and `rg` fallbacks.
- Upstream repository docs (not vendored with this skill): the `docs/` tree at
  https://github.com/jonathanong/no-mistakes/tree/main/docs — see the agent-guide,
  cli, graph-edges, rules, eslint-rules, and configuration pages there.

## Hard Limits

- `baseUrl`-only imports are not resolved; use `compilerOptions.paths`.
- Dynamic `import()` and `require()` are tracked only with string literals.
- Bare external specifiers such as `react` are terminal module nodes; their
  `node_modules` sources are not parsed. Node built-ins such as `node:path`
  remain excluded from module nodes.
- Graph tools answer file/symbol relationships, not exact call locations.
- Dynamic queue names, route paths, fetch URLs, and selectors should be made
  static when agent-readable analysis is required.
- Selector text edges are approximate; exact configured test ID selector edges
  are stronger evidence.
- Non-TS/JS files are not walked for import edges; use `rg` for Go, Rust, CSS, JSON.
- `tests plan` works without `testPlan` in `.no-mistakes.yml` (uses default
  direct + dependencies groups). Configure `testPlan` to add environments,
  custom limits, coverage groups (Playwright only), and global-config triggers.
