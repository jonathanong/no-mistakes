# Decision tree: which tool, which flag?

## Choosing a binary

```
What are you trying to find?
│
├─ Files this file imports (forward graph)
│   └─ no-mistakes dependencies <file> [--depth N] [--relationship import]
│
├─ Runtime modules a Playwright route can conservatively reach
│   └─ no-mistakes dependencies <route-file> --relationship route-import
│
├─ Files that import this file (reverse graph)
│   └─ no-mistakes dependents <file> [--depth N] [--relationship import]
│
├─ Files that import a specific named export
│   └─ no-mistakes dependents <file>#SYMBOL
│
├─ Named exports of a file (public API)
│   └─ no-mistakes symbols <file>
│
├─ Named imports of a file (what it consumes)
│   └─ no-mistakes symbols <file> --include imports
│
├─ React component traits / fetch checks
│   └─ no-mistakes react analyze 'app/components/**/*.tsx'
│   └─ no-mistakes react check 'app/components/**/*.tsx' --assert-no-fetch
│
├─ Queue producer/worker hops
│   └─ no-mistakes queues edges [file] [--depth N]
│   └─ no-mistakes queues related <file> [--direction deps|dependents|both]
│   └─ no-mistakes queues check
│
├─ Server route extraction / related files
│   └─ no-mistakes server routes
│   └─ no-mistakes server related <file> [--direction deps|dependents|both]
│   └─ no-mistakes server edges [file] [--depth N]
│
├─ Tests to run after changing a file (preferred)
│   └─ no-mistakes tests plan vitest --changed-file <file> --format paths
│   └─ no-mistakes tests plan playwright --changed-file <file> --format paths
│
├─ Tests to run after changing a file (lower-level fallback)
│   └─ no-mistakes dependents <file> --test vitest --format paths
│   or no-mistakes dependents <file> --test playwright --format paths
│
├─ Explain why a test was selected
│   └─ no-mistakes tests why <test> --plan plan.json
│
├─ Playwright route/selector coverage gate
│   └─ no-mistakes playwright check --json
│
├─ Playwright tests covering a page/component
│   └─ no-mistakes playwright related <file> --json
│
├─ What a Playwright test asserts (routes, selectors, fetches)
│   └─ no-mistakes playwright tests <test-file> --json
│
├─ API calls made by a Next.js App Router route
│   └─ no-mistakes fetches <route-or-file> --format json
│
├─ Which packages changed between two lockfile refs
│   └─ no-mistakes lockfile diff --base <ref> --format json
│
├─ Which routes or queue jobs reach a file
│   └─ no-mistakes dependents <file> --relationship route
│   └─ no-mistakes dependents <file> --relationship queue
│   (requires .no-mistakes.yml with the relevant project/rule config)
│
└─ Which CI workflows invoke a binary
    └─ no-mistakes dependents src/bin/mybinary.rs --relationship ci
```

## Choosing a --relationship flag

| Flag value | What edges it follows |
|---|---|
| `import` | Static TS/JS imports, `import type`, string-literal dynamic `import()`, and string-literal `require()` |
| `import-static` | Static TS/JS value imports only |
| `import-type` | Type-only imports only |
| `import-dynamic` | String-literal dynamic `import()` only |
| `import-require` | String-literal CommonJS `require()` only |
| `route-import` | Runtime static imports/re-exports and literal dynamic imports, conservatively including function-scoped imports for Playwright route reachability; excludes type-only imports and `require()` |
| `workspace` | Cross-package npm workspace imports |
| `package` | `package.json` dependency declarations to workspace entries or external module nodes |
| `test` | source/test correspondence, Playwright route tests, Next layouts, and selector coverage |
| `route` | route refs, Playwright route tests, and Next layouts |
| `queue` | Queue enqueue/worker relationship → virtual queue job |
| `md` | Markdown link → linked source file |
| `ci` | CI workflow YAML → binary entry point |
| `http` | HTTP client call with a static path → backend route definition |
| `process` | `spawn`/`exec`/Playwright `webServer` → spawned entry file |
| `asset` | Explicit non-code asset import |
| `react` | React component render relationship |
| `dotnet` | C# `using`/type reference/project reference edges |
| `swift` | Swift import/type reference/SwiftPM target dependency edges |
| `terraform` | Terraform/OpenTofu resource, module, and output reference edges |
| `all` | All standard relationships except the opt-in `route-import` relationship (default) |

Repeatable — `--relationship import --relationship workspace` follows both kinds.

## Output format selection

| Format | When to use |
|---|---|
| `--format json` / `--json` | Feeding to another tool, agent parsing |
| `--format paths` | Shell `$()` substitution, xargs |
| `--format md` | Writing to a document |
| `--format human` | Debugging interactively on a TTY |
| `--format yml` | YAML pipelines |

Default: `human` on TTY, `json` when piped.

## Filtering results

```bash
# Only files matching a glob
no-mistakes dependents src/auth.mts --filter 'backend/**/*.mts'

# Collapse to folder level (trailing /)
no-mistakes dependents src/auth.mts --filter 'backend/services/*/'

# Combine multiple globs (OR)
no-mistakes dependents src/auth.mts --filter 'backend/**' --filter 'integration-tests/**'
```

## Edge cases

**When to use rg instead of no-mistakes dependents for callers:**
`no-mistakes dependents` answers "who imports this file/symbol" with resolution-correct graph traversal. Use `rg` when you need the specific line of code where a symbol is called, or when a pattern may appear in non-import contexts (template strings, comments, dynamic lookups).

**Payoff threshold — when does `no-mistakes` beat `rg`?**
- ✅ Use `no-mistakes` when: the question spans >2 workspace directories,
  involves >5 import hops, or requires transitive test-impact across a large
  graph.
- ⚡ Use `rg` when: you need the exact call line, the pattern may appear in
  non-import contexts (comments, strings, dynamic lookups), or the file type
  has no `no-mistakes` graph domain (Go source, prose). For non-TS/JS
  structural questions: `.yml` → `ci` · `.tf` → `infra` · `.swift` →
  `swift` · Rust binary CI impact → `--relationship ci` · CSS/JSON asset
  imports → `--relationship asset`.
- For "what directly imports this one file?" in a single directory,
  `no-mistakes importers <file>` is faster and prints static-import callers.
  For complete impact including dynamic and CommonJS imports, use
  `no-mistakes dependents <file>`.

**When to pass --tsconfig explicitly:**
In a monorepo with per-package tsconfigs and no root `tsconfig.json`, auto-discovery may pick the wrong one. Pass `--tsconfig <pkg>/tsconfig.json` whenever you get empty or wrong results from a file inside a specific package.

**When no-mistakes dependents returns fewer results than expected:**
Check if the import uses a bare external specifier, a non-literal dynamic `import()` / `require()`, or an alias that requires a specific package `tsconfig`. See `limits-and-fallbacks.md` for workarounds.

**Graph edge caveats:**
See https://github.com/jonathanong/no-mistakes/blob/main/docs/graph-edges.md for
every edge kind. Dynamic route paths, fetch URLs,
queue names, process commands, and selector values are not guessed. Text-based
selector coverage is approximate; exact configured test ID edges are stronger.
`route-import` and `route` are different: the former follows the conservative
runtime module closure used by Playwright, while the latter follows URL-route,
route-test, and layout relationships.
