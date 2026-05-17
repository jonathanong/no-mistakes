---
name: no-mistakes
description: Answer structural TS/JS and app-graph questions deterministically. Use for imports, dependents, named exports/imports, test impact, React traits, Playwright route/selector coverage, queue producer/worker hops, server routes, and global no-mistakes checks.
allowed-tools: Bash(no-mistakes:*) Bash(playwright-ast-coverage:*) Bash(rg:*) Read Glob
---

# No Mistakes

Use `no-mistakes` before `rg` when the question is structural: what a TS/JS file
imports, who imports it, what it exports, which tests are related, whether a
queue job is connected, which server route owns an endpoint, whether a route or
selector is covered by Playwright, or what React traits a component has.

## Command Selection

| Question | Tool |
| --- | --- |
| What does this file transitively import? | `no-mistakes dependencies <file>` |
| Which files are affected by touching this file? | `no-mistakes dependents <file>` |
| Which files import this named export? | `no-mistakes dependents <file>#SYMBOL` |
| Which tests should rerun? | `no-mistakes dependents <file> --test vitest --format paths` |
| What does this module export/import? | `no-mistakes symbols <file> --include both` |
| What React traits does this component have? | `no-mistakes react analyze <glob>` |
| Does this component tree call fetch? | `no-mistakes react check <glob> --assert-no-fetch` |
| Is this App Router route or selector covered? | `playwright-ast-coverage check --json` |
| Which Playwright tests cover this page/component? | `playwright-ast-coverage related <file> --json` |
| What routes, fetches, and selectors does a test cover? | `playwright-ast-coverage tests <test-file> --json` |
| Which queue producer/worker files are connected? | `no-mistakes queues related <file>` |
| Are queue producers/workers unmatched? | `no-mistakes queues check` |
| What server routes exist? | `no-mistakes server routes` |
| Which server route files are related? | `no-mistakes server related <file>` |
| Run configured project checks | `no-mistakes check` |
| Plain text, comments, log messages, exact call lines | `rg` |

## Quick Workflow

```bash
# Machine-readable graph query
no-mistakes dependents src/utils.mts --root /path/to/project --format json

# Shell-friendly affected test list
no-mistakes dependents src/utils.mts --test vitest --format paths

# Public API and imports
no-mistakes symbols src/api.mts --include both --format json

# Queue and server graph checks
no-mistakes queues check --format json
no-mistakes server routes --format json

# Playwright route and selector coverage
playwright-ast-coverage check --json
playwright-ast-coverage related 'web/app/users/[id]/page.tsx' --json
playwright-ast-coverage tests tests/e2e/users.spec.ts --json
```

Prefer `--format json` for agent parsing and `--format paths` for command
substitution. Use `--timings` on graph, queue, and server commands when you need
to explain cost.

## Graph Options

`dependencies`, `dependents`, and `related` support:

- `--root <PATH>` for the project root.
- `--tsconfig <FILE>` for path aliases; pass this explicitly in monorepos.
- `--depth <N>` or `--max-depth <N>` to limit traversal.
- `--filter <GLOB>` to include only matching files; repeatable.
- `--test vitest|playwright|cargo` to filter to test files.
- `--relationship import|workspace|test|route|queue|md|ci|http|process|all`.
- `--format json|md|yml|paths|human`, `--json`, `--timings`, and `--jobs`.

`FILE#SYMBOL` works only for `dependents`/`related`, not `dependencies`.
Namespace imports match all symbols; use `rg` on returned files to confirm exact
member usage.

## Playwright Coverage

Inspect `.playwright-ast-coverage.{yaml,yml,json,jsonc}` and Playwright config
before choosing coverage commands.

- Use `playwright-ast-coverage check --json` before finishing Next.js App Router
  or Playwright test-hook selector work.
- Treat uncovered routes, uncovered selectors, and duplicate exact selectors as
  actionable failures unless project instructions explicitly allow them.
- Fix uncovered routes by adding Playwright navigation or URL assertions for a
  URL that matches the reported route pattern.
- Fix uncovered selectors by asserting the hook with `getByTestId(...)` or a
  supported CSS attribute selector.
- Replace unsupported dynamic selectors with literals or static templates when
  stable test coverage is expected.
- Use `--assert-conditional-tests` to require active tests only, or
  `--allow-skipped-tests` when skipped tests should count.

## When To Read References

- `references/decision-tree.md`: choosing commands, relationships, filters, and
  outputs.
- `references/dependencies.md`: full `dependencies` reference.
- `references/dependents.md`: full `dependents`/`related` reference and
  `FILE#SYMBOL` behavior.
- `references/symbols.md`: full `symbols` reference.
- `references/monorepo-resolution.md`: tsconfig paths and workspace packages.
- `references/limits-and-fallbacks.md`: unsupported forms and `rg` fallbacks.

## Hard Limits

- `baseUrl`-only imports are not resolved; use `compilerOptions.paths`.
- Dynamic `import()` and `require()` are tracked only with string literals.
- Bare external specifiers such as `react` and `node:path` are ignored.
- Graph tools answer file/symbol relationships, not exact call locations.
- Dynamic queue names, route paths, fetch URLs, and selectors should be made
  static when agent-readable analysis is required.
