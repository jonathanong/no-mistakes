# Performance diagnostics

Every CLI leaf inherits two opt-in diagnostics flags:

```sh
no-mistakes --timings dependencies src/app.ts --format json
no-mistakes tests plan vitest --verbose-timings --changed-file src/app.ts
```

`--timings` emits phase durations. `--verbose-timings` implies `--timings` and
also emits deterministic work counts for discovery, source reads, parsing,
resolution, graph construction, traversal, and output. Diagnostics go only to
stderr; stdout, structured output order, exit status, and actionable error text
are unchanged.

```text
[timing] graph: 12.411ms
[timing] analysis.rules: 7.034ms (parallel; non-additive)
[timing] total: 21.882ms
[work] discovery.roots: 1
[work] workspace.builds: 1
[work] parse.files: 42
[work] source.reads: 42
```

Parallel phase durations can overlap their enclosing phase and siblings. Never
add lines marked `parallel; non-additive` to estimate total time; use `total`
for wall time.

For graph-finalization regressions, verbose `check` output separates
`graph.forward_adjacency_normalization`, `graph.reverse_adjacency_normalization`,
`graph.canonical_flatten`, and `graph.ordinal_construction`. Playwright-enabled
graphs additionally report `graph.playwright_selector_merge`. These labels
identify deterministic finalization work; they do not change graph output or
canonical edge order.

PostgreSQL annotation checks report `postgres.annotation.project` separately
from parsing, with `postgres.annotation.reachability_walks` counting the
branch-state liveness traversals. This helps distinguish source extraction
from helper and callback evaluation. `postgres.annotation.captured_reader_rebuilds`
counts reverse capture-index reconstruction; unchanged compaction retains the
existing index.

File-scoped forbidden call checks report `forbidden_calls.path_filter_checks`
once per selected file instead of once per call occurrence.

Production dependency checks report `production_dependencies.resolutions` for
distinct source/specifier pairs resolved within each rule application, including
unresolved targets. `production_dependencies.legacy_import_projections` counts
historical TypeScript/TSX import projections prepared for sources whose ordinary
parser mode differs. Those projections share the request's parser cache; their
count does not imply an additional physical parse for every projection.

Companion import checks report `companion_imports.candidate_universes`,
`companion_imports.selections`, `companion_imports.literal_projections`, and
`companion_imports.literal_unions` for shared candidate selection and literal
import membership preparation.

Forbidden workspace closure checks report
`forbidden_workspace_closure.lockfile_importer_constructions` for distinct
lockfile importer projections initialized during the request. The count
includes cached read and parse failures, so repeated rule applications using
the same lockfile still contribute one construction.
`forbidden_workspace_closure.workspace_selections` and
`forbidden_workspace_closure.manifest_projections` count immutable workspace and
dependency-field projections shared by equivalent applications.

Instrumentation is invocation-scoped and disabled by default. A disabled run
does not construct an observer, start clocks, or retain work ledgers. Enabled
runs memoize successful and failed discovery/read/parse operations in memory
for the duration of that invocation only.

The async Node API remains stderr-free. `impactedChecks({ timings: true })`
retains its structured `{ phase, duration_ms }[]` response; other Node methods
do not add timing fields.
