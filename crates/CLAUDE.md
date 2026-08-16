## Performance Guidelines

### Core architecture invariants

- One pass per invocation: discover the file universe once, parse each TS/JS
  file once for the requested `TsFactPlan`, and reuse `TsFactMap` everywhere.
- In-memory only: use invocation-scoped fact maps, resolver/traversal caches,
  and dependency maps. No disk caches, daemons, databases, or cross-run state.
- Canonical graph: project-level relationships belong in `DepGraph` as typed
  `EdgeKind` edges unless they are purely file-local lint rules.
- Fully parallel: per-file extraction and independent checks run through
  rayon or concurrent maps, with deterministic sorting before output.
- No hardcoded domain defaults: route roots, HTTP prefixes, queue factories, and
  worker locations must be configured instead of inferred from repo conventions.

### Prepared analysis, source-session ownership, and relationship projection

The public check, query, N-API, or integration-runner boundary owns one
`AnalysisSession`/prepared analysis for the invocation. That owner declares the
complete `TsFactPlan` and graph demand, then prepares the visible-file inventory,
`SourceStore`, resolver/catalog, fact map, workspace data, and canonical graph
once. Lower layers receive the prepared facts and session-owned stores by
reference; they must not open an independent source/read/parse/resolution or
graph-building pipeline.

`SourceStore` is specifically a *session-owned* physical-I/O boundary. A
source consumer must use the store supplied by its prepared request, including
for a supplemental normalized path. This memoizes both source text and read
failures and makes read accounting meaningful. Do not compensate for a missing
prepared source parameter by calling `std::fs::read_to_string`; thread the
prepared store down instead.

Each layer has a deliberately narrow, declarative responsibility:

1. Fact extraction records syntactic facts (imports, exports, routes,
   selectors, queues, HTTP calls, and similar per-file occurrences) for the
   declared plan. It does not decide a report-specific traversal.
2. The prepared resolver/catalog and workspace layer resolves those facts with
   the request's visible-file universe. It owns path/config variation rather
   than commands rebuilding their own resolver or index.
3. Graph builders project the resolved facts into the one typed `DepGraph`/
   `EdgeIndex`. Query and rule layers filter or traverse that prepared
   relationship projection; they do not create parallel forward/reverse maps.
4. Renderers only shape already-computed results. A renderer/CLI leaf that
   discovers files, reads sources, parses programs, or calls an extractor has
   crossed the boundary in the wrong direction.

This split is important for determinism as well as speed: a fact plan makes
what is collected explicit, the source session gives every input one identity,
and a canonical relationship projection gives every consumer the same answer.

Keep extracted facts in their final ownership form. Do not wrap an owned value
in `Arc` only to recover it with `as_deref().cloned()` for its sole consumer;
that deep-copies nested data. Use `Arc` only for retained shared ownership.

Cache immutable selection work at the coarsest stable request key. A scoped
resolver selects once per importing file, including negative results; assert
the selection-computation count in tests.

For reverse symbol queries, the prepared analysis owner supplies the symbol
catalog/reverse projection. `importers`, `exports-of`, `dead-exports`, and
`call-sites` may project it differently, but none may discover graph files,
collect TS facts, or build a `SymbolIndex` ad hoc. Similarly, a domain edge
producer requests its domain facts through the plan and projects typed edges;
it never invokes `collect_domain_facts` itself.

Do not add a blanket ast-grep guard for a future `PreparedSymbolCatalog` yet:
this branch has no stable post-refactor module/type/call shape to scope without
either missing aliases or banning its legitimate constructor. Protect that
invariant with fixture-backed semantic architecture tests that assert one
prepared catalog is reused, then add a narrow structural rule only once its
owner and public API are stable.

The N-API JSON bindings have a stable, narrower rule: the binding-only
`napi_api/*_bindings.rs`/`wrappers_query.rs` includes and the root registration
module must call `json_binding!`, never hand-write an
`AsyncTask::new(JsonTask::new(...))` function. Likewise, the two JS facades
must route direct native JSON conversion through `callJson`/`createJsonApis`
rather than add another static or computed `native.*` parse/stringify wrapper.
The ast-grep guards cover that exact boilerplate. They cannot prove that an
alias, re-export, or wrapper chain ultimately delegates to the right prepared
operation, so fixture-backed N-API/JS parity tests remain the semantic guard.

### Canonical edge finalization

When relationship producers have already normalized each source adjacency,
flatten the forward map in deterministic source order and assign ordinals in
that pass. Do **not** rebuild one repository-wide `Vec<CanonicalEdge>`, sort
it, and deduplicate it again: that adds a global O(E log E) pass after local
normalization and can become the graph's dominant allocation/sort cost. New
edge batches should use `EdgeIndex`'s keyed/per-source deduplication, then sort
only the affected adjacency lists when their public order requires it.

### Diagnosing a performance regression

Use `no-mistakes check --timings --verbose-timings` (`perf_trace.rs`) instead of
a special instrumented build: it prints `[timing] <label>: <ms>` per hot path
wrapped in `crate::perf_trace::trace(label, || { ... })`. Wrap new hot paths
the same way, not a temporary `eprintln!` timer. `cargo bench` in CI is
diagnostic-only — `ast-grep` plus tests proving buggy vs. fixed paths
disagree is the real regression-prevention layer.

Benchmark preparation separately from projection. When optimizing construction,
add a batched constructor benchmark alongside lookup/projection benchmarks.

### Duplicate full-repo work across independent call paths

When two independent call paths in one invocation (a standalone rule, a
`DepGraph` edge collector) need the same repo-wide computation, share one
result instead of paying twice, even if neither path knows about the other.
Most of these need no cache key — see `get_or_compute_route_reachable_files`
(`graph/fact_lookup.rs`, a `OnceLock` on the shared fact map): the largest win
found this way, ~8s dropping to ~0 on a real monorepo. Only key a cache when
an input genuinely varies between callers (`get_or_compute_app_selector_occurrences`
in the same file needs one) — grep the callee for anything caller-specific
(e.g. a Playwright project) before assuming "compute once" is safe; a wrong
key returns silently wrong data. **Regression guard:** assert on a call count,
not value equality — a non-caching implementation returns the same value too.

**Edge producer smell:** a `builder_edges.rs` producer missing `facts:
Option<&dyn TsFactLookup>` while siblings have it may duplicate a rule's
`TsFactLookup`-routed scan, or hand-roll a sequential loop where a shared
parallel helper exists — keep per-file error tolerance when wiring facts in
regardless, since an edge producer failing aborts the shared graph, unlike a
rule failing only its own findings. `git_visible_files` is one
`git ls-files -z -t --cached --others --exclude-standard` process — thread
that snapshot through any path that would rediscover files, e.g. the
`_from_git_files` variants of `discover_files`/`discover_files_preserving_roots`.

### Shared state in parallel loops

Avoid `Mutex<HashMap<K, V>>` for caches accessed from rayon `par_iter()`. The
lock serialises every lookup and insert across all threads, eliminating most
parallel speedup. Use `DashMap<K, V>` instead; its sharded
`entry(...).or_insert_with(...)` keeps per-key computation single-shot without
serializing unrelated keys.

### Verify a builder method doesn't silently disable an existing cache

A builder method that configures one thing (e.g. a visible-file set) can also
flip an unrelated flag (e.g. disabling a cache) if the two were bundled together
for a reason that no longer applies. Easy to miss: results stay correct, only
performance regresses. A "cache reuses result" test doesn't catch this either —
it only asserts the same value comes back twice, which holds regardless of
whether caching happened; assert on the cache's own state (length, hit counter)
instead.

### Hoist per-iteration I/O and parsing out of hot loops

Never read from disk, spawn processes, or parse files inside a loop that runs
once per test file (or per any other O(N) entity). Instead, compute the
invariant data once before the loop and pass it in.

Common violations to watch for:
- Calling `discover_files` (which runs `git ls-files`) per test file
- Reading and parsing config files per test file
- Building `GlobSet`/`Regex` per test file
- Parsing TS/JS again inside a graph edge producer when `TsFactMap` already has
  the required facts

### Guard expensive discovery behind an early return

`discover_files` runs `git ls-files` (two child processes). Only call it when
you actually need the file list. Guard with an early return for the empty-input
case so pattern expansion does not spawn Git when there is nothing to expand.

### Never walk the tree without `.gitignore` awareness

A raw recursive `std::fs::read_dir`/`WalkDir` walk has no `.gitignore` awareness beyond
whatever directory names you hardcode into a denylist. Dependency stores, build
caches, and other generated directories are routinely gitignored but not in any
hardcoded skip list, so an unguarded walk can visit hundreds of thousands of entries
per call on a real repo even though the equivalent `git ls-files` call returns
instantly.

Prefer, in order:
1. Derive candidate paths from the already-discovered git-visible file list (tracked
   files plus untracked files not excluded by `.gitignore`) instead of walking the
   filesystem at all — a candidate only matters if it can contain a file that discovery
   would otherwise surface, so this is both correct and touches zero extra I/O.
2. If a walk is unavoidable (e.g. outside a git repository), use the `ignore` crate
   (`WalkBuilder`) so `.gitignore` rules apply, not a hardcoded directory denylist.

Root/prefix expansion (include globs, preserved roots, project roots) must reuse
the single discovered file list, not walk per pattern or per project — compute
once, memoize per `(base, pattern)`, and early-return when nothing to expand.

**Regression guard:** prove the fast path is taken, not just that output is
unchanged — a `.gitignore`-blind walk and a git-aware one often produce the same
final file list while differing enormously in work done. Construct a case where
the two approaches would disagree (e.g. a gitignored directory containing a
nested match) and assert on the disagreement.

### Pre-compute BFS traversals in parallel before the per-entity loop

When every parallel work item needs a BFS traversal of the same graph, run all
BFS traversals up front in a single `par_iter()` pass so the results are cached
before the work loop begins. This avoids redundant traversals and lets the
expensive computation scale linearly. Regression tests must show the traversal
cache is populated before the dependent per-entity loop.
