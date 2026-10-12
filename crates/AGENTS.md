<!-- cspell:ignore rustc -->
## Performance Guidelines

Core check/evaluator optimizations must also follow [the crate rules](no-mistakes/AGENTS.md).

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

The public check, query, N-API, or integration runner owns one request-scoped
`AnalysisSession`. Declare the complete `TsFactPlan` and graph demand, then
prepare the visible-file inventory, `SourceStore`, resolver/catalog, fact map,
workspace data, and canonical graph once. Lower layers borrow those inputs;
never open an independent source/read/parse/resolution or graph pipeline.

`SourceStore` owns physical I/O for the request. Every source consumer uses
the supplied store, including supplemental normalized paths, memoizing source
text and read failures under one identity and accounting path. Thread prepared
sources to helpers; never replace a missing store with `std::fs::read_to_string`.

Keep layer responsibilities declarative:

1. Extract syntactic imports, exports, routes, selectors, queues, HTTP calls,
   and other per-file facts for the plan; do not choose report traversals.
2. Resolve facts through the prepared resolver/catalog and workspace using the
   request's file universe. This layer owns path/config variation; commands
   must not rebuild resolvers or indexes.
3. Project relationships into one typed `DepGraph`/`EdgeIndex`. Queries and
   rules filter/traverse it; never create parallel forward/reverse maps.
4. Renderers shape computed results. Renderer/CLI leaves must not discover
   files, read sources, parse programs, or call extractors.

Explicit fact plans, shared input identities, and canonical relationships
keep results deterministic as well as avoiding repeated work.

Keep facts in final ownership form: `Arc` is for retained sharing, never an owned
value recovered with `as_deref().cloned()` by its sole consumer, copying nested data.

Cache immutable selection work at the coarsest stable request key. A scoped
resolver selects once per importing file, including negative results; assert
the selection-computation count in tests.

Reverse symbol queries consume the owner's symbol catalog/reverse projection.
`importers`, `exports-of`, `dead-exports`, and `call-sites` must not rediscover
graph files, collect TS facts, or build `SymbolIndex` ad hoc. Domain edge
producers request facts through the plan and never call `collect_domain_facts`.

Do not add a blanket ast-grep guard for a future `PreparedSymbolCatalog` yet:
this branch has no stable post-refactor module/type/call shape to scope without
either missing aliases or banning its legitimate constructor. Protect that
invariant with fixture-backed semantic architecture tests that assert one
prepared catalog is reused, then add a narrow structural rule only once its
owner and public API are stable.

N-API JSON bindings must use `json_binding!`, never a handwritten
`AsyncTask::new(JsonTask::new(...))`. JS facades must use `callJson`/
`createJsonApis`. ast-grep guards boilerplate; parity tests guard aliases/re-exports.

### Canonical edge finalization

Flatten already-normalized forward adjacency in deterministic source order,
assigning ordinals in that pass. Never rebuild, sort, or deduplicate a global
`Vec<CanonicalEdge>`: that repeats local normalization with O(E log E) work.
New batches use `EdgeIndex` keyed/per-source deduplication; sort only affected
adjacency lists when public order requires it.

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
`git ls-files -z -t --stage --cached --others --deleted --exclude-standard` process — thread
that snapshot through any path that would rediscover files, e.g. the
`_from_git_files` variants of `discover_files`/`discover_files_preserving_roots`.

### Interned analysis keys use FxHash

Interned local keys (paths, `NodeId`, `CallableId`, lexical `(scope, name)`)
use `crate::fx` maps. Nest `(scope, name)` tables so lookups take `&str`;
do not `get(&(scope, name.to_string()))`. SipHash is for public/untrusted
keys. rustc-hash 2 aliases have no `new()`. ast-grep covers
`no-std-hashmap-call-indexes` and `no-std-hashmap-extract-collector`.

### Index related tables before per-entity queries

After collecting a per-file `Vec`, build a membership or group index before
any per-file/per-name helper. Nested `table_a.iter()` × `table_b.filter`
joins must be one grouping pass. Do not linear-scan a repo-wide `Vec`
(`resolved_call_sites`) inside a per-file helper. Lock construction work,
not output equality.

### Catalog-rooted graph policy stays in the selected files

Graph rules rooted on a Vitest catalog or file set must not BFS or scan
repo-wide call records for `traversal: file`; filter selected files first.

### Shared state in parallel loops

Avoid `Mutex<HashMap<K, V>>` for caches accessed from rayon `par_iter()`.
Use `DashMap` so `entry(...).or_insert_with(...)` does not serialize
unrelated keys.

### Verify a builder method doesn't silently disable an existing cache

A builder that configures one thing can also flip an unrelated cache flag.
Results stay correct and only performance regresses. Assert on cache state
(length, hit counter), not value equality.

### Hoist per-iteration I/O and parsing out of hot loops

Never read from disk, spawn processes, or parse files inside a loop that runs
once per test file (or per any other O(N) entity). Instead, compute the
invariant data once before the loop and pass it in.

Prepare `TsFactContext` once per stable `(root, plan/variant)` collection
scope and borrow it through per-file work. Keep semantically distinct graph and
runner variants on their own contexts, and assert context construction counts.

Common violations to watch for:
- Calling `discover_files` (which runs `git ls-files`) per test file
- Reading and parsing config files per test file
- Building `GlobSet`/`Regex` per test file
- Parsing TS/JS again inside a graph edge producer when `TsFactMap` already has
  the required facts

### Guard expensive discovery behind an early return

Call `discover_files` only when its file list is needed. Early-return on empty
inputs so pattern expansion never spawns Git when there is nothing to expand.

### Never walk the tree without `.gitignore` awareness

A raw recursive `std::fs::read_dir`/`WalkDir` walk has no `.gitignore`
awareness beyond hardcoded denylists, so it can visit huge generated trees
(`node_modules`, `.next`) that `git ls-files` would skip.

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

**Regression guard:** prove the fast path is taken. Construct a case where a
`.gitignore`-blind walk and a git-aware one disagree (gitignored nested match).

### Pre-compute BFS before per-entity loops

Cache shared BFS results up front in one `par_iter()` before per-entity work.
