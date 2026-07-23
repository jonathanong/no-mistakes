# `no-mistakes dependents` — full reference

Find every file that transitively imports the given file or named export.

## When to use a different tool

- Need a file's exported/imported **symbol list** (its public API) → `no-mistakes symbols`, not `no-mistakes dependents`.
- Need **line numbers** where a symbol is used → use `rg` on the files `no-mistakes dependents` returns.
- Need cross-package consumers by **package name** → run `no-mistakes dependents` on each entrypoint file.

## Usage

```
no-mistakes dependents <FILE[#SYMBOL]>... [--root <PATH>] [--tsconfig <FILE>] [--depth <N>]
           [--filter <GLOB>]... [--test <FRAMEWORK>]...
           [--target-module <GLOB>]...
           [--relationship <KIND>]...
           [--format <FORMAT>] [--json] [--timings] [-j <N>]
```

## How to invoke

```bash
# Find everything that imports a utility file (JSON — pipe or non-TTY)
no-mistakes dependents src/utils.mts --root /path/to/project

# Find only direct importers
no-mistakes dependents src/utils.mts --root /path/to/project --depth 1

# Symbol-level: find only files that import a specific export
no-mistakes dependents src/queues.mts#sendEmail --root /path/to/project

# Find only test files that depend on this file
no-mistakes dependents src/utils.mts --root /path/to/project --test vitest

# Follow only import edges (skip test/route/queue/md/ci/workspace edges)
no-mistakes dependents src/utils.mts --root /path/to/project --relationship import

# Find conservative Playwright route-module importers
no-mistakes dependents web/components/dialog.tsx --root /path/to/project --relationship route-import

# Find tests that would need to run after changing a file
no-mistakes dependents src/utils.mts --root /path/to/project --test vitest --relationship test

# Explicit tsconfig (required for monorepos without a root tsconfig.json)
no-mistakes dependents src/utils.mts --root /path/to/project --tsconfig backend/tsconfig.json
```

## Flags

| Flag | Default | Description |
|------|---------|-------------|
| `--root <PATH>` | cwd | Project root |
| `--tsconfig <FILE>` | auto-detected | Path to tsconfig.json |
| `--depth <N>` | unlimited | Max traversal depth |
| `--filter <GLOB>` | none | Include only matching files (repeatable, OR) |
| `--target-module <GLOB>` | none | Include only matching external module nodes (repeatable, OR) |
| `--test <FRAMEWORK>` | none | Expand to well-known test globs: `vitest`, `playwright`, `cargo`, `dotnet`, `swift` (repeatable) |
| `--relationship <KIND>` | all | Follow only edges of this kind (repeatable, OR). Values: `import`, `import-static`, `import-dynamic`, `import-type`, `import-require`, `route-import`, `workspace`, `package`, `test`, `route`, `queue`, `md`, `ci`, `workflow`, `workflow-job`, `workflow-step`, `workflow-needs`, `workflow-uses`, `workflow-run`, `workflow-artifact`, `http`, `process`, `asset`, `react`, `dotnet`, `swift`, `terraform`, `all` |
| `--format <FORMAT>` | human (TTY) / json (pipe) | Output format: `json`, `md`, `yml`, `paths`, `human` |
| `--json` | false | Shorthand for `--format json` |
| `--timings` | false | Emit phase timings on stderr |
| `-j / --jobs <N>` | all cores | Worker threads. `0` or omitted = all cores. Honors `RAYON_NUM_THREADS`. |

Discovery is git-aware: tracked files plus untracked non-ignored files are considered, and `.gitignore`d files are skipped. Relationship filters also gate graph construction so unrelated edge producers are not run. Invalid relationship values fail at argument parsing.

## Symbol-level queries (`FILE#SYMBOL`)

Append `#SYMBOL` to a file path to find only files that import that specific named export:

```bash
no-mistakes dependents src/queues.mts#sendEmail --root /path/to/project
```

- Follows re-exports transitively — a file that re-exports `sendEmail` is included, as are its importers
- Namespace imports (`import * as`) are treated as matching all symbols
- `#SYMBOL` syntax is only supported in `no-mistakes dependents`, not `no-mistakes dependencies`

## Output

JSON (`--format json` or `--json` or non-TTY):
```json
{
  "roots": ["src/utils.mts"],
  "files": [
    { "path": "src/main.mts", "depth": 1, "via": ["import"] },
    { "path": "src/main.test.mts", "depth": 1, "via": ["test"] }
  ]
}
```

Paths (piped default, for shell `$()`):
```
src/main.mts
src/other.mts
```

- `path` is relative to `--root`
- `via` lists edge kinds that reached each file (omitted when empty)

## Notes

- Static imports/re-exports, type-only imports/references, string-literal dynamic `import()`, and string-literal `require()` are tracked under `--relationship import`
- `--relationship route-import` follows runtime static imports/re-exports and
  literal dynamic imports without function-reachability pruning. It excludes
  type-only imports and `require()`. Use `route` for URL-route, Playwright
  route-test, and Next.js layout edges.
- `route-import` is excluded from unfiltered traversal and `all`; request it by
  name because its deliberately conservative closure is an alternate view.
- Route/queue edges are only active when `.no-mistakes.yml` defines the relevant config
- `http` edges connect static HTTP client paths, including non-interpolated template literals, to backend route-definition files
- `process` edges connect `spawn`/`exec`/Playwright `webServer` entries to their entry files
- Patterns ending in `/` in `--filter` collapse results to that folder level
