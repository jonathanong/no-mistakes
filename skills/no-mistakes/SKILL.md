---
name: no-mistakes
description: "Deterministic impact map and test plan. Use before editing to find callers/tests, after editing to validate, and instead of rg when the question crosses packages, aliases, Playwright routes, or configured Python/Go/Rust graphs. Skipping it misses tests, duplicate exports, uncovered App Router pages, and empty plans that are not actually empty."
allowed-tools: Bash(no-mistakes:*) Bash(rg:*) Read Glob
---

# No Mistakes

Use `no-mistakes` for structural codebase questions: import and test impact,
public symbols, static route/fetch/queue relationships, or configured project
checks. Use `rg` after a graph query for exact call lines, comments, strings,
or unsupported/dynamic forms. The mistakes this prevents are catalogued in
[docs/why.md](../../docs/why.md).

## When to activate

Use `no-mistakes` when skipping the graph would miss callers, tests, or
coverage: a change that crosses packages or aliases, transitive test impact,
Playwright/route/queue questions, or configured checks. For one local TS/JS
file's direct static importers, use `no-mistakes importers <file>`; otherwise
use the graph command below. For non-code text and exact syntax use `rg`
directly.

## Change lifecycle

For an existing file, plan before editing; for a new file, rerun after it
exists. Recheck changed files after editing, then validate before handoff.

```bash
# Before editing: scope impact and focused tests.
no-mistakes dependents <file> --format json
no-mistakes tests plan vitest --changed-file <file> --format json

# After editing: request the complete local validation set.
no-mistakes impacted-checks <file...> --format json

# Before handoff: resolve imports and run configured checks.
no-mistakes resolve-check <file...> --format json
no-mistakes check --format json
```

For Next.js routes, pages, selectors, or Playwright tests, also run
`no-mistakes tests plan playwright --changed-file <file> --format json` and
`no-mistakes playwright check --json`. See
[lifecycle recipes](references/lifecycle.md) for package additions, hook
context, option propagation, and safe command execution.

## Output and scope contract

- Use `--format json` (or `--json`) for parsing; CLI JSON is the authoritative
  structured result. Inspect warnings and fallback fields before assuming an
  empty or narrow plan is complete.
- Use `--format paths` only for reviewing or piping trusted path lists. Prefer
  JSON `command` arrays to execute configured validation commands; do not
  `eval` command text from an untrusted source.
- `--timings` and `--verbose-timings` write diagnostics to stderr, leaving
  successful stdout machine-readable.
- Pass `--root <workspace>` explicitly. In normal monorepos omit `--tsconfig`
  so each importing file uses its owning config; pass a package tsconfig only
  to force one resolver for debugging or a deliberately scoped request.
- Static literals produce the strongest results. Dynamic imports, route paths,
  selectors, queue names, fetch URLs, and process commands require inspection
  or `rg`; see [limits and fallbacks](references/limits-and-fallbacks.md).

For several related reports in one Node process, use the async N-API
`analyzeProject({ root, reports: [...] })` rather than repeated CLI calls. It
shares one request-scoped inventory, fact pass, and canonical graph. Dedicated
API calls rebuild analysis; see the upstream Node API documentation linked from
the repository docs.

## Quick command selection

| Need | Command |
| --- | --- |
| File/module dependencies | `no-mistakes dependencies <file> --format json` |
| Files or named-export consumers | `no-mistakes dependents <file>[#SYMBOL] --format json` |
| Direct static importers | `no-mistakes importers <file> --format json` |
| Public API, imports, or signature blast radius | `no-mistakes symbols <file> --include both --format json`; add `--mode signature-impact --symbol NAME` |
| Named exports and their consumers | `no-mistakes exports-of <file> --format json` |
| Is an export unused? | `no-mistakes dead-exports <file> [NAME...]` |
| Function calls and static argument shapes | `no-mistakes call-sites <file> NAME --format json` |
| Tests for a changed file or diff | `no-mistakes tests plan <framework> --changed-file <file> --format json`; use `--from-git-diff base...head` for a diff |
| Explain selected tests | `no-mistakes tests why <test> --plan plan.json --format json` |
| Exact runner commands | `no-mistakes tests plan <framework> --changed-file <file> --format commands` |
| Combined tests, lint, typecheck, and configured checks | `no-mistakes impacted-checks <file...> --format json` |
| Playwright coverage or related tests | `no-mistakes playwright check --json`; `no-mistakes playwright related <file> --json` |
| React callers and component traits | `no-mistakes react usages <file>#Component --format json`; `no-mistakes react analyze <glob> --format json` |
| Next.js page-to-API coupling | `no-mistakes fetches <route-or-file> --format json` |
| Queue/server graph | `no-mistakes queues related <file> --format json`; `no-mistakes server related <file> --format json` |
| CI or workflow impact | `no-mistakes ci impact <file> --format json`; `no-mistakes ci topology --format json` |
| Private CI planning artifacts (npm package) | `no-mistakes planning-impact --changed-files <manifest> --output-dir <directory>` |
| Terraform/OpenTofu or Swift | `no-mistakes infra resource-refs <type.name> --format json`; `no-mistakes swift importers <file> --format json` |
| Configured language graph | `no-mistakes dependents <file> --relationship <lang> --format json` |

`tests plan` supports `vitest`, `playwright`, `jest`, `dotnet`, `swift`,
`python`, `go`, `cargo`, `rails`, `php`, `java`, `kotlin`, `elixir`, and `dart`
when the matching language/test configuration is present. `dependents --test`
is a lower-level fallback; prefer the planner because it includes configured
groups, limits, diffs, and deleted-file behavior.

## Reference routing

- [docs/why.md](../../docs/why.md): mistakes this graph exists to prevent.
- [lifecycle.md](references/lifecycle.md): before-edit, after-edit, and
  handoff recipes.
- [decision-tree.md](references/decision-tree.md): command, relationship, and
  output selection.
- [dependencies.md](references/dependencies.md) and
  [dependents.md](references/dependents.md): graph traversal, filtering, and
  `FILE#SYMBOL` semantics.
- [symbols.md](references/symbols.md) and
  [lightweight-queries.md](references/lightweight-queries.md): symbols,
  importers, exports, call sites, and resolution checks.
- [tests.md](references/tests.md): planners, diffs, environments, fallbacks,
  explain output, and runner commands.
- [playwright.md](references/playwright.md): selector, route, and assertion
  coverage commands.
- [impact-recipes.md](references/impact-recipes.md): selector, API shape,
  package-entrypoint, workflow, test-deletion, and queue recipes.
- [monorepo-resolution.md](references/monorepo-resolution.md): workspace
  aliases and resolver ownership.
- [limits-and-fallbacks.md](references/limits-and-fallbacks.md): unsupported
  forms, confidence limits, and `rg` fallbacks.
