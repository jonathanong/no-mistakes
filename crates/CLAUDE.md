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

### Diagnosing a performance regression

Use `no-mistakes check --timings --verbose-timings` (`perf_trace.rs`) instead of
a special instrumented build: it prints `[timing] <label>: <ms>` per hot path
wrapped in `crate::perf_trace::trace(label, || { ... })`. Wrap new hot paths
the same way, not a temporary `eprintln!` timer. `cargo bench` in CI is
diagnostic-only — `ast-grep` plus tests proving buggy vs. fixed paths
disagree is the real regression-prevention layer.

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
rule failing only its own findings. `git_visible_files`/`git ls-files` is
equally undeduped (a process spawn per call) — thread one fetch through a
path that discovers files more than once per invocation, e.g. the additive
`_from_git_files` variants of `discover_files`/`discover_files_preserving_roots`.

### Shared state in parallel loops

Avoid `Mutex<HashMap<K, V>>` for caches accessed from rayon `par_iter()`. The
lock serialises every lookup and insert across all threads, eliminating most
parallel speedup. Use `DashMap<K, V>` instead:

```rust
// Bad – contended lock dominates runtime at high thread counts
let cache: Mutex<HashMap<PathBuf, Arc<Vec<PathBuf>>>> = Mutex::new(HashMap::new());

// Good – concurrent map with sharded locks; or_insert_with runs the closure only once per key
let cache: DashMap<PathBuf, Arc<Vec<PathBuf>>> = DashMap::new();
let deps = cache
    .entry(key.clone())
    .or_insert_with(|| Arc::new(expensive_compute(&key)))
    .clone();
```

### Verify a builder method doesn't silently disable an existing cache

A builder method that configures one thing (e.g. a visible-file set) can also
flip an unrelated flag (e.g. disabling a cache) if the two were bundled together
for a reason that no longer applies. Easy to miss: results stay correct, only
performance regresses. A "cache reuses result" test doesn't catch this either —
it only asserts the same value comes back twice, which holds regardless of
whether caching happened; assert on the cache's own state (length, hit counter)
instead.

```rust
// Bad – bundles an unrelated cache_enabled=false into an unrelated setter.
pub fn with_visible(mut self, visible: &'a HashSet<PathBuf>) -> Self {
    self.visible = Some(visible);
    self.cache_enabled = false;
    self
}

// Good – caching stays on; an opt-out gets its own method (`without_cache()`).
pub fn with_visible(mut self, visible: &'a HashSet<PathBuf>) -> Self {
    self.visible = Some(visible);
    self
}
```

### Hoist per-iteration I/O and parsing out of hot loops

Never read from disk, spawn processes, or parse files inside a loop that runs
once per test file (or per any other O(N) entity). Instead, compute the
invariant data once before the loop and pass it in:

```rust
// Bad – reads and parses config on every iteration
for file in test_files.par_iter() {
    let setup = config::setup_files_for_test(root, config, rel_path)?;
    // ...
}

// Good – compute once, reuse across all iterations
let setup_data = config::precompute_setup_data(root, config)?;
for file in test_files.par_iter() {
    let setup = config::setup_files_for_test_precomputed(&rel_path, &setup_data);
    // ...
}
```

Common violations to watch for:
- Calling `discover_files` (which runs `git ls-files`) per test file
- Reading and parsing config files per test file
- Building `GlobSet`/`Regex` per test file
- Parsing TS/JS again inside a graph edge producer when `TsFactMap` already has
  the required facts

### Guard expensive discovery behind an early return

`discover_files` runs `git ls-files` (two child processes). Only call it when
you actually need the file list. Guard with an early return for the empty-input
case:

```rust
fn expand_config_patterns(root: &Path, patterns: Vec<String>) -> Vec<ConfigFile> {
    if patterns.is_empty() {
        return Vec::new();  // avoid git ls-files when nothing to expand
    }
    let files = discover_files(root, &[]);
    // ...
}
```

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

```rust
// Bad – .gitignore-blind; visits every entry, including large ignored dirs.
fn find_dirs_matching(base: &Path, name: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    collect_recursive(base, name, &mut out); // raw std::fs::read_dir recursion
    out
}

// Good – derive from the git-visible list; non-git fallback uses `ignore`.
fn find_dirs_matching(base: &Path, name: &str, git_files: Option<&[String]>) -> Vec<PathBuf> {
    match git_files {
        Some(files) => dirs_matching_from_files(base, name, files),
        None => walk_with_ignore_crate(base, name),
    }
}
```

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
expensive computation scale linearly:

```rust
// Pre-populate cache for all test files before the per-test loop
test_files.par_iter().for_each(|file| {
    dependency_cache
        .entry(file.clone())
        .or_insert_with(|| Arc::new(runtime_deps(&graph, file.clone())));
});

// Now every per-test reachable check is a cache hit
test_files.into_par_iter().map(|file| {
    reachable::check(/* ... uses dependency_cache ... */)?;
    // ...
})
```
