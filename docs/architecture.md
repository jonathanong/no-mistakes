# Architecture

`no-mistakes` is a local, deterministic codebase-intelligence engine. Its
architecture is optimized for AI agents that need reliable project facts while
spending as few tokens and CPU cycles as possible.

This document codifies the performance architecture behind issues `#126`, `#130`, `#132`, `#133`, `#530`, and `#531`.

## Core Decisions

1. One pass per invocation.
2. In-memory caching only.
3. Build one canonical graph.
4. Run independent work fully parallel.

These are constraints, not preferences. New features should fit this model
instead of adding separate scanners, persistent caches, background services, or
serial bottlenecks.

Project-specific relationship discovery must be explicit. Route definitions,
HTTP prefixes, queue factories, workers, and similar domain roots should come
from configuration rather than hardcoded repository conventions.

## Invocation Boundary

Each CLI invocation is self-contained:

1. Resolve root, tsconfig, config, entrypoints, and requested relationships.
2. Create a request-scoped `AnalysisDataset`, discover visible project files once,
   and assign stable lexical file identities.
3. Read each requested file once and plan the union of facts required by the
   invocation.
4. Parse each file once per required semantic parse mode, then build graph
   edges and symbol indexes once per normalized effective plan and
   file universe.
5. Query the graph or shared fact maps.
6. Emit deterministic output.

Grammar, module kind, and declaration interpretation are fixed when OXC
constructs a parser. A mixed `analyzeProject()` request that needs both normal
extension-based semantics and the legacy list-symbols TypeScript interpretation
therefore performs one cached parse in each distinct required mode. Ordinary
`.ts` and `.tsx` inputs share a physical parse when their modes are equivalent;
JavaScript-family, explicit module-kind, and declaration inputs do not. Legacy
results remain isolated to list-symbols output, while graph and check facts
retain extension-based semantics.

No state is trusted across invocations. Persistent graph caches, daemons,
databases, and filesystem cache directories are intentionally outside the
architecture. If a future feature needs speedups, prefer reducing the per-run
work or sharing more in-memory facts during that run.

### Analysis session and observability

`AnalysisSession` owns the invocation observer, root datasets, parser gateway,
and resolver caches. Each root's `AnalysisDataset` owns the canonical visible
inventory and `SourceStore`; its config and tsconfig result caches are keyed by
normalized effective paths so distinct same-root report scopes cannot reuse the
first scope's manifest. Automatic and explicit aliases for the same effective
path share one entry. Source reads and manifest loads memoize both successes and
failures; graph and traversal caches use normalized effective plans.

The session optionally carries one `InvocationObserver`. When diagnostics are
disabled the observer is `None`: hot paths branch before reading the clock and
do not allocate work ledgers. With `--verbose-timings`, the same gateways expose
deterministic aggregate counts and keyed test snapshots. Independent Rayon
durations are labeled non-additive because they overlap.

One-pass fixture tests enforce these ceilings:

1. One discovery per normalized root.
2. At most one physical source read per requested path and one parse attempt
   per `(normalized path, semantic mode)` key.
3. One config or tsconfig parse per normalized effective manifest path,
   including cached failures.
4. One resolver computation per normalized resolution key, including misses.
5. At most one graph/index build per effective request plan.
6. One traversal computation per roots/direction/edge-set/depth/symbol-mode/candidate-inventory key.

Lazy import-only traversal does not eagerly prepare the full indexable universe
or build the canonical multi-domain graph. A single import-only query walks the
reachable frontier and reuses any prepared per-file facts already supplied by
an enclosing request.

`analyzeProject` with import-only `dependencies` reports still follows
facts → one graph → commands: one lazy walk of the union of those report roots
builds one reachable import adjacency graph; each report projects `deps_of`
from that graph. That graph is the reachable import subgraph for the request,
not a full-universe index.

Bounded import-closure reports add a second, report-scoped universe:

- The session `VisiblePathSnapshot` remaps symlink spellings back to the
  request-root namespace.
- `candidateInclude` / `candidateExclude` select the initial GraphFiles
  inventory for that report. Shared GraphFiles stay full unless every graph
  consumer in the request is import-only `dependencies` with the same bounds.
- Escape admission uses the pre-filter GraphFiles universe, not every
  git-visible snapshot path, so skipped directories such as `dist` and
  `fixtures` stay invisible. A local/workspace source in that universe can
  still leave the candidate inventory; the walk then escapes and parses it.
- `projection: "paths"` is presentation only and is not part of the
  traversal cache key.

## Current Pipeline Shape

The main graph pipeline is centered in `no-mistakes`:

| Stage             | Current type/module                      | Role                                                                                               |
| ----------------- | ---------------------------------------- | -------------------------------------------------------------------------------------------------- |
| Request dataset   | `AnalysisDataset`                        | Owns the immutable request-scoped inventory, sources, and parsed configuration/workspace metadata. |
| Request analysis  | `SharedTraversalContext`                 | Owns shared immutable facts, the canonical resolver, and normalized graph/symbol caches.           |
| File universe     | `FileInventory`, `GraphFiles`            | Assigns stable lexical file identities and exposes the selected visible/indexable views.           |
| Source text       | `SourceStore`                            | Lazily memoizes successful and failed reads without changing consumer-specific error policy.       |
| TS/JS facts       | `TsFactPlan`, `TsFileFacts`, `TsFactMap` | Selects and stores facts extracted from one OXC parse per required semantic mode.                  |
| Import resolution | `ImportResolver`                         | Resolves relative imports and tsconfig aliases using an invocation-scoped cache.                   |
| Graph build       | `DepGraph`, `GraphBuildPlan`             | Builds forward and reverse adjacency maps once per normalized plan and file universe.              |
| Traversal         | `deps_of`, `dependents_of`, `related`    | Runs BFS over the canonical graph with optional edge and path filters.                             |

The top-level `check` command also shares precomputed facts across domain
checks and runs those checks through `rayon::join`.

## Single-Pass Fact Extraction

The effective file fact plan is the contract for source parsing. It merges the
`TsFactPlan` and check/report demands before any source file is parsed.

Required direction:

1. Add fields to `TsFactPlan` for every fact family needed by graph edges or
   project checks.
2. Add corresponding fields to `TsFileFacts`.
3. Extract all requested TS/JS facts inside the same OXC parse in
   `collect_file_facts`.
4. Pass `TsFactMap` into graph/check builders instead of letting domains read
   and parse files independently.

The parser allocator and full OXC AST are discarded as soon as compact owned
facts have been extracted. Domain modules such as routes, queues, HTTP calls, symbols, imports, process
spawns, and future extractors should be visitors/fact producers or graph edge
producers. They should not be independent full-codebase scanners when their
input can come from the shared fact pass.

All production OXC parsing goes through the observable parser entrypoint in
`ast`. Check-only analyses may store compact owned fields in `CheckFileFacts`;
for example, the server-route client-boundary rule derives its route-shape bit
and client-call lines from the same `Program` used by graph and React facts.
Compatibility wrappers may still accept source text, but must delegate to that
parser entrypoint and then to the corresponding Program-based extractor.

Acceptable exceptions:

1. Non-TS/JS inputs such as Markdown, package manifests, and CI YAML may have
   their own lightweight readers.
2. A narrowly requested lazy query may read only the reachable frontier if it
   avoids a full graph build and does not duplicate work inside that query.
3. Shared source text may be read once into an invocation-local collection when
   more than one non-AST extractor needs raw text.

## In-Memory Caching

Caches are scoped to one request dataset and never survive the invocation.

Allowed cache shapes:

1. `SourceStore`: file identity to a memoized read result, including failures.
2. Shared immutable file facts and compatibility views such as `TsFactMap`.
3. Pre-classified filesystem-rule candidates keyed by rule ID.
4. Parsed manifest, workspace, tsconfig, and requested configuration metadata.
5. Canonical import classifications, including unresolved imports.
6. `EdgeIndex`: canonical typed edges plus forward and reverse adjacency for dependency, queue, and server-route traversal.
7. `GraphFiles.visible`: path membership for resolver and graph checks.
8. Graph and symbol-index results keyed by normalized plan and file universe.
9. Local traversal caches for expensive per-root searches.
10. Request-scoped canonical `tsconfig.json` / `jsconfig.json` path map used
    when rendering `tsconfig_provenance`. The map is filled once from those
    config files only. Report projection must not canonicalize every visible
    path to recover a relative config spelling.

Disallowed cache shapes:

1. Disk-backed graph caches.
2. Daemons that keep project state between CLI runs.
3. Databases or services.
4. Global mutable caches that survive unrelated invocations.

Parallel caches must not serialize the hot path. Use concurrent maps such as
`DashMap`, per-thread collections followed by deterministic merge, or immutable
shared maps. Do not put `Mutex<HashMap<_, _>>` around a high-frequency cache
used from `rayon` workers.

## Canonical Graph

`DepGraph` is the canonical relationship substrate.

Nodes are:

1. Source files.
2. Virtual nodes when the relationship is real but not a file, such as queue
   jobs.

Edges are typed with `EdgeKind`. The current graph supports imports, type
imports, dynamic imports, requires, workspace imports, test relationships,
routes, queues, Playwright route tests, Markdown links, CI invocations, HTTP
calls, process spawns, asset imports, and React render relationships.

Every relationship feature should produce edges into this graph unless there is
a strong reason it cannot be represented as a source-to-target relationship.
Queries should prefer graph traversal over bespoke recursive search.

The graph stores one request-scoped `EdgeIndex`. It owns and validates canonical
typed edges while retaining both adjacency directions:

1. `forward`: node to dependencies.
2. `reverse`: node to dependents.

This double-indexed shape is required. It makes `dependencies`, `dependents`,
`related`, and focused test selection cheap after the build phase.

Queue and server-route commands project their public DTOs from typed
relationships and use the same index implementation for traversal. Queue
Project analysis and dependency-graph Dashboard analysis remain explicit modes:
they share resolver and relationship infrastructure without silently adopting
each other's matching rules.

## Parallel Execution Model

Parallelism is used where measurement shows it helps:

1. Per-file fact collection uses `rayon` over files.
2. Edge producers may use `par_iter` or `into_par_iter` when each file can be
   analyzed independently and a representative benchmark clears the speed and
   peak-memory gates.
3. Top-level domain checks run concurrently when they consume shared facts.
4. Traversal pre-computation can run per root when roots are independent.
5. Expensive caches must be concurrent or thread-local plus merged.

The CLI initializes the global rayon pool from `--jobs`,
`NO_MISTAKES_JOBS`, or CPU defaults. New commands should use the shared
initialization path instead of creating ad hoc thread pools.

Visitor fusion requires at least a 5% representative end-to-end improvement.
Outer graph parallelism requires at least a 10% improvement, no more than 10%
peak-memory growth, and byte-identical output across thread counts. When a
candidate misses its gate, keep the simpler existing execution model rather
than landing speculative concurrency or a larger combined visitor.

Parallel code must still produce deterministic output:

1. Collect unordered work into vectors or maps.
2. Merge on the main thread when mutation order matters.
3. Sort adjacency lists and output entries before rendering.
4. Keep diagnostics stable and path-based.

## Graph Build Plan

`GraphBuildPlan` prevents unnecessary work. It should mirror the relationship
filters exposed by the CLI:

1. Enable only the edge producers needed by the requested relationships.
2. Reuse the same discovered file universe for every producer.
3. Reuse the dataset's immutable facts when any enabled producer needs TS/JS
   facts, and normalize set-like plan fields before cache lookup.
4. Avoid domain-specific rediscovery.

Adding a relationship kind requires updating:

1. `EdgeKind`.
2. `GraphBuildPlan`.
3. CLI relationship filtering.
4. Edge production from shared facts or shared file contents.
5. Fixture-backed graph tests.
6. Documentation for supported and unsupported static forms.

## Extension Checklist

When adding a new analyzer:

1. Decide whether it is file-local, fact-based, graph-based, or output-only.
2. If it is file-local, prefer an ESLint/Oxlint rule.
3. If it needs project context, put shared logic in `no-mistakes`.
4. Extend `TsFactPlan` and `TsFileFacts` instead of adding a second TS/JS parse.
5. Add a graph edge when the result is a relationship.
6. Run extraction over files in parallel.
7. Use in-memory caches only.
8. Sort outputs for determinism.
9. Add fixture-based regression tests.
10. Route physical work through `AnalysisSession` and add an exact work-count
    assertion for any new gateway category.

## Performance regression suite

`crates/no-mistakes/benches/core_analysis.rs` is the single checked-in
Criterion-compatible harness. It uses `fixtures/performance/core-analysis`
for the existing small in-process APIs and
`fixtures/performance/graph-gates` for a larger checked-in synthetic graph.
The 14-file core-analysis corpus is too small for honest visitor-fusion and
outer-graph-parallelism gates: CodSpeed noise can dominate
`graph/forward_reverse_query` there. The graph-gates fixture is a step up
(~75 TypeScript sources, configured queue/HTTP/route rules, and a
`GraphBuildPlan::all().with_symbols(true)` end-to-end build) rather than a
10x blowup of `test-cases/codebase-analysis/large-graph-monorepo`, so fusion
(≥5% e2e) and outer graph parallelism (≥10% speed, ≤10% peak memory,
byte-identical output) can be measured without OOMing CI bench jobs.

The harness measures only in-process APIs: lazy traversal, fact extraction,
canonical graph build/query, workspace load/resolution, symbols, aggregate
checks, reused multi-report analysis, impacted checks,
disabled/timings/verbose observer overhead, composed language-frontend
extract/edges/queue-glob matching on `fixtures/lang-frontends`, full-domain
fact extract on graph-gates, and an aggregate `check` of that same fixture.
Every workload runs a preflight that validates stable, fixture-specific
output invariants before the measured loop.

CI builds the harness once, then runs each semantic shard in both CodSpeed CPU
simulation and memory modes: `check`, `observer`, `tests-plan`, `graph-core`,
`graph-gates`, `language-frontends`, `native-frontends`, `graph-finalization`,
`graph-production`, and `query`. Production node and selector finalization use
separate filtered jobs. `NO_MISTAKES_BENCH_SHARD` skips unused setup and
preflight work; unset or `general-memory` still runs every non-production
workload locally. Swift/.NET fixture-only changes select `native-frontends`,
while `fixtures/lang-frontends` changes select `language-frontends`, so the two
CodSpeed histories do not perturb one another. Shared code and configuration
changes conservatively run every shard. Unknown shard names fail fast. New
workloads must use checked-in fixtures,
`BenchmarkId` for meaningful variants, `Throughput` where a stable unit exists,
and must not generate repositories or launch the CLI as a subprocess.

The `bench` and `benchmark-shards` jobs both run on GitHub-hosted
`ubuntu-24.04-arm` so the uploaded harness matches the machine that executes
it. That label is one ARM64 CPU generation, so glibc dispatch and CodSpeed's
simulated cache sizes stay aligned between base and head. `ubuntu-latest`
mixes Intel and AMD on x86_64 and is not used for these jobs. Regular test
and lint jobs stay on GitHub-hosted x86_64 runners. CodSpeed Macro Runners
are not used: they register an organization-level runner, and this repository
lives on a personal account. A CodSpeed report is not an implementation
regression until both the base and the PR head ran on `ubuntu-24.04-arm`.

Each shard's benchmark step allows six minutes so a cold install of CodSpeed's
instruments (valgrind and `libc6-dbg`) can finish. The action's instrument
cache key omits libc, so after a runner image updates libc6 an exact-key hit
restores debug symbols that no longer match and every run reinstalls. The
workflow keys the cache directory by the installed libc6 version, so the first
run after an update saves a fresh entry. That entry matches only while the
Ubuntu archive's `libc6-dbg` is the image's libc6 version; until images catch
up to a newer archive release, runs may still reinstall.

### Interpreting CodSpeed results

`ubuntu-24.04-arm` is one ARM64 CPU generation, so a base and a head that both
ran there share glibc dispatch and simulated cache sizes. CodSpeed still falls
back to an older base when the PR's exact base has no successful benchmark
run. A report that names a different runtime environment, including any
x86_64 base from before this runner, is not evidence of a regression.

Treat a CodSpeed failure as actionable only when the report compares
`ubuntu-24.04-arm` with the expected base commit. If CodSpeed reports different
runtime environments or an unexpected base, inspect the changed files first. A
docs-only or otherwise unrelated change should record the mismatch in the PR's
Shepherd Journal and acknowledge the result; code changes should be rerun on
`ubuntu-24.04-arm` before performance work begins. Local before/after
measurements must use the same machine, toolchain, benchmark mode, thread
count, and fixture.

### Control runs for unrelated benchmark changes

An expected base and matching runner establish that a CodSpeed comparison is
eligible for investigation. Establish causality before changing an unrelated
implementation: inspect the measured source and fixtures, keep fixture preflight
invariants, and repeat matched measurements in both base/head orders. Include a
repeat of the same binary as a control for variation between runs.

[Investigation #1171](https://github.com/jonathanong/no-mistakes/issues/1171)
examined the language-frontend report on
[the synthetic resolve-check fixture cleanup](https://github.com/jonathanong/no-mistakes/pull/1168#issuecomment-5969771786).
The report compared the expected base `ffc6bb12` with head `14df367b`, and both
language jobs used `ubuntu-24.04-arm`. The measured language sources, benchmark
adapter, and `fixtures/lang-frontends` corpus were unchanged. Every local run
passed the existing preflight checks: 117 files, 69 parsed files, and 125 edges.

Matched local Criterion runs used one x86_64 Linux host, Rust 1.96.0, four Rayon
threads, the `language-frontends` shard, and the same bench profile (LTO off,
16 code generation units, debug information and incremental compilation off).
Initial and repeated comparisons used 30 samples, a one-second warmup, and three-second
measurement; the adjacent reverse-order comparison used 50 samples, a
two-second warmup, and five-second measurement for both binaries.

| Comparison | Extract time change | Edges time change |
| --- | --- | --- |
| Original base → head | +7.60%, p < 0.05 | +3.63%, p = 0.23 |
| Original base → same base again | +2.76%, p = 0.24 | +5.72%, within Criterion's noise threshold |
| Original base → head again | +7.61%, p < 0.05 | +6.53%, p < 0.05 |
| Adjacent head → base | −0.45%, p = 0.72 | +11.08%, p < 0.05 |

Positive values mean the second binary was slower. The adjacent extract
comparison's 95% interval was −2.83% to +2.11%; the older base was slower for
edges in that comparison, reversing the reported head regression. These
controls did not establish a consistent head-specific wall-clock regression.
The initial statistically significant results remain part of the evidence.

Criterion wall-clock timings on x86_64 do not validate CodSpeed's ARM64 CPU
simulation or its memory measurements. Local validation did not include memory
mode, so the reported allocation increase has not been validated locally.
Keep that limitation explicit when recording the investigation; a matched wall-clock result does not establish
that a different measurement mode is noise.

Use an isolated checkout and one dedicated `CARGO_TARGET_DIR` for both builds.
Keep the saved baseline until the comparison finishes, then remove build
artifacts. A saved Criterion executable must receive `--bench` for Criterion measurement;
without it, test mode runs each selected benchmark once without measurement.

## Anti-Patterns

Avoid these patterns:

1. A new command that independently walks the repository when `GraphFiles` can
   be shared.
2. A domain module that parses every TS/JS file again after `TsFactMap` exists.
3. A high-contention `Mutex<HashMap<_, _>>` inside a `par_iter` loop.
4. A persistent cache that makes results depend on previous runs.
5. A graph-like feature that keeps its own untyped adjacency structure instead
   of adding `EdgeKind` edges.
6. Parallel collection without deterministic sorting before output.

## Implementation Status

Already aligned:

1. `ImportResolver` uses `DashMap` instead of a single locked `HashMap`.
2. `TsFactMap` exists for imports and symbols.
3. `DepGraph` stores forward and reverse maps.
4. Many file and edge collectors use `rayon`.
5. The top-level `check` command shares facts and parallelizes domain checks.
6. `TsFactPlan` and `TsFileFacts` cover route, queue, HTTP, and process facts,
   and aggregate consumers reuse them through the unified `TsFactMap`.

Still to converge:

1. Keep lazy query paths explicit so they do not become hidden duplicate parse
   passes.
2. Continue replacing serial shared mutation with concurrent caches or
   thread-local collection plus deterministic merge.
