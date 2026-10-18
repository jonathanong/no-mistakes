# Performance diagnostics

Every CLI leaf inherits opt-in diagnostics flags:

```sh
no-mistakes --timings dependencies src/app.ts --format json
no-mistakes tests plan vitest --verbose-timings --changed-file src/app.ts
```

`--timings` emits phase durations. `--verbose-timings` implies `--timings` and
also emits deterministic work counts for discovery, source reads, parsing,
resolution, graph construction, traversal, and output. Diagnostics go only to
stderr; stdout, structured output order, exit status, and actionable error text
are unchanged.

On Unix, `--verbose-timings` also emits `[memory] peak_rss_kib: N` when
process resource sampling succeeds. This is the process-lifetime peak resident
memory in KiB, including startup and earlier command work. It is separate from
request-scoped timing snapshots and cannot attribute memory to concurrent N-API
requests. Other platforms omit the footer. Basic `--timings` and disabled
diagnostics perform no memory sampling. See the
[hosted measurement protocol](../development/hosted-check.md) for external
whole-command wall, CPU, and memory comparisons.

For a command that may be killed before it returns, add `--progress`:

```sh
no-mistakes check --profile ci --progress --progress-interval 1000 --format json
```

`--progress` implies `--timings` and tracks fine-grained active spans without
collecting verbose work ledgers. The interval is a positive integer in
milliseconds and defaults to 1000; `--progress-interval` requires `--progress`.
Each heartbeat attempts to flush one `[progress]` JSON line to stderr. A final
snapshot is also attempted on normal completion or a returned error. Output is bounded
and best-effort: a full or unread stderr pipe can drop diagnostics, but cannot
hold cleanup, a command timeout, or signal termination indefinitely. On Unix,
the existing child process-group signal forwarder, installed before lock
waiting, attempts a snapshot before preserving the default SIGINT/SIGTERM
exit behavior, including with `--timeout 0`. SIGKILL
cannot be handled; previously flushed heartbeats remain available in job logs.

Returned command errors stay authoritative and use the ordinary CLI stderr
boundary, independently of the progress queue. They retain the same error text
and exit status. As with a command without progress, reporting an ordinary error
can wait for a full stderr pipe to be read; progress diagnostics never suppress
that command error.

A snapshot contains `elapsedMs`, traced `startedSpans` and `completedSpans`,
an `active` set, and grouped `completed` timing durations. Each live span has
an invocation-local `id`, hierarchical `phase` labels, `startedMs` measured
from invocation start, `elapsedMs`, and `parallel`. Siblings and nested phases
can overlap; the set reports all live spans instead of replacing one phase with
another. Rayon branches share the dispatch ancestor and retain their own
thread-local nested phases. Completed timings are grouped by label and parallel
status; their durations are non-additive when `parallel` is true. Progress only
covers instrumented spans; `invocation.prepare` and `invocation.dispatch` also
identify lock waiting and gaps between deeper phases.

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
from parsing. `postgres.annotation.roots` counts evaluated roots across executor
profiles: modules whose runtime imports cannot reach an executor are omitted.
`postgres.annotation.parallel_passes` counts shared evaluator batches; all
executor profiles and their demanded roots share one Rayon pool. Each completed
profile releases its prepared file map and reduces its root events to owned
results before the full pass ends. Results are applied in deterministic profile
and source order.
Unresolved, computed, and dynamic imports conservatively retain their modules
and importers, including cycles. Executor-free callers that supply SQL literals
to imported helpers remain roots.

- **Seeds.** Configured executor and trusted-tag modules remain demand seeds.
  An empty `importSpecifier` is not a seed: `executorNames` without a module
  still selects calls, but a non-type import of an unrelated module does not.
- **Dead ends.** A successfully resolved import outside the prepared annotation
  catalog is a dead end. A parse-failed inventory file is absent from that
  catalog, so a resolved import of it is a dead end only while parse failure
  yields no query facts. If recovery later emits partial facts, that edge must
  become conservative again.
- **Workspace names.** pnpm workspace package names resolve through the session
  workspace map when the TypeScript resolver has no target, the same way
  imported SQL helpers do. That fallback lives in relative executor scope
  projection, so embedded SQL and annotation demand share it. A package the
  workspace does not declare stays unresolved and keeps its importer.
- **Conservative.** Resolver targets without an exact prepared inventory
  identity (including ambiguous symlink spellings) stay conservative and are
  not replaced by a workspace target.
`postgres.annotation.reachability_walks` counts the
branch-state liveness traversals. This helps distinguish source extraction
from helper and callback evaluation. `postgres.annotation.captured_reader_rebuilds`
counts reverse capture-index reconstruction; unchanged compaction retains the
existing index. `postgres.annotation.incoming_dense_slots_copied` counts incoming
argument slots cloned into a branch join, including every slot of a new object
and only changed slots of an existing object. It excludes clones of prior
values needed to build candidate sets for changed slots.

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
`forbidden_workspace_closure.path_reconstructions` counts dependency paths
materialized for matching forbidden-package dependencies before final finding
deduplication.

Instrumentation is invocation-scoped and disabled by default. A disabled run
does not construct an observer, start clocks, or retain work ledgers. Enabled
runs memoize successful and failed discovery/read/parse operations in memory
for the duration of that invocation only.

The async Node API remains stderr-free. `impactedChecks({ timings: true })`
retains its structured `{ phase, duration_ms }[]` response; other Node methods
do not add timing fields. Repository-analysis calls accept `diagnosticsId` to
register live progress and `diagnosticsSnapshot({ id })` polls the same structured
snapshot without acquiring the invocation lock. IDs must be unique while active;
completion and failure remove them, and polling then returns `null`. Node calls
install no process signal handler or stderr heartbeat. See [Node API](../node-api.md).
