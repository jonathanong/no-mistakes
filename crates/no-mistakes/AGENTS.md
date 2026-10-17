## Performance regression rules

These rules supplement the root and `crates/AGENTS.md` ownership/architecture
requirements. Apply them when optimizing core analysis or reporting performance.

### Prepared demand and shared check inputs

- Declare the exact annotation executor-profile subset at the boundary,
  separately from all embedded-SQL profiles. Union annotation demand when plans
  merge; unspecified direct plans retain all-profile compatibility. Never skip
  executor-free importers whose literals flow into configured executors. Guard
  mixed rules/include scopes and cyclic helpers. Parse options once while
  preserving error precedence.
- Production imports use the declared union fact plan and prepared sources.
  Preserve historical JS/module/MDX/metadata/unsupported-extension extraction
  and resolver behavior. Resolve each runtime `(source, specifier)` once per
  applicable rule scope; group external package seeds before package traversal.
- Cache parsed pnpm importers per lockfile within the invocation, including
  malformed/read-failure results and concurrent compute-once behavior. Keep
  application-specific dependency options outside the parsed-lockfile cache.
- Share ordered workspace selections and manifest dependency-field projections
  only when their selection and field keys agree. Preserve order and options;
  assert selection/projection construction counts, not just equal findings.
- Evaluate file-scoped call policy once per distinct selected file, before its
  occurrences; file traversal must stay within the selected catalog/file set.
- Prepare distinct companion glob selections and membership unions once.
  Project each distinct companion's historical literal imports once, retaining
  type/computed/commented-import and parser recovery/failure semantics. Reuse
  the filesystem candidate universe, source store, and declared fact pass.
  Keep existing production demand when it already covers companion imports;
  AST-only/no-companion requests must not initialize this filesystem rule.
  Avoid membership copies when the primary scope is already fully requested.
  Guard suppression/includes, reopened/skipped projects, external roots,
  duplicate findings, and configuration errors.
- Normalize paths without building an intermediate component vector. Preserve
  clean/dot/parent/root/prefix behavior; use FxHash for interned resolver keys
  under the parent guide's public/untrusted-key distinction.

- Share annotation import/export indexes and lazy resolved targets across
  evaluators and executor profiles only within one resolver/catalog projection.
  Memoize unresolved targets too; borrow hot bindings and paths, and guard
  construction/resolution counts plus ESM and mutable-module semantics.

### Evaluator snapshots, joins, and derived indexes

- Pruning heap mutations and builder updates shares one liveness walk; skip
  empty tables and reuse compaction's live argument/builder IDs for snapshots.
  Immutable value summaries may skip scalar-only containers only when cache
  invalidation preserves mixed reference trees, function roots, and cycles.
- Identity frame compaction must avoid scope/value copy-on-write remapping,
  while still pruning unreachable state. Skip equal captured-value writes so
  shared scopes and their indexes do not detach unnecessarily.
- Restore primary state before derived indexes, once. Keep exact reverse readers
  when ownership is unchanged. Entry frames retain capture ownership; update only
  arm-created readers, including module initials. Clear the reverse index only
  when every reader belongs to the arm. On compaction, rebuild or remap using its
  injective frame mapping; bound sparse storage. Guard full-rebuild equality,
  retained prefixes, private origins, nested closures, full-clear/re-add and
  module-initial restoration.
- Reuse final-arm liveness only across unchanged graph/roots; keep the original
  prune fallback. Guard full-state equality and actual walk counts.
- Borrow unchanged incoming dense argument slots across branch joins; clone
  only new objects or differing incoming slots. Remap returned values directly
  with initial argument IDs as separate roots, without copying the return
  slice or adding synthetic values. Count incoming slots actually cloned,
  rather than all evaluator copies. Fixtures must make callbacks read joined
  slots and verify the specific annotation outcome so Unknown cannot pass;
  preserve cyclic argument objects and sparse mutations.

### Workspace closure paths

- Apply the start-package source filter once before traversal. Queue borrowed
  package identities and record the first predecessor when enqueuing; build
  chains only for matching findings. Preserve FIFO/dependency order, first
  shortest paths, aliases, finding locations, cycles, and diamond behavior.
- Assert path-reconstruction counts before final finding deduplication. Saved
  benchmark preflight checks must prove the intended starts and unique findings;
  use supported exact package selectors instead of assuming glob semantics.

### Required regression gates

- Reject reproduced slowdowns in representative consuming or non-consuming
  configurations; a focused win does not excuse a slower control path.
- Retaining extra ranges, contexts, or caches for speed must earn measured benefit.
  Include cold construction and peak RSS; reject flat work with higher memory.
- Save controls with realistic source/JSX density and configured include/filter
  payloads. Add fast-path count or concurrency tests that fail on the old code.
- Measure diagnostics off/on separately, including CPU and RSS. Use raw elapsed
  time: never pause clocks or subtract reporting overhead to improve results.
- Keep expensive diagnostics opt-in; with diagnostics disabled, add no clocks,
  spans, maps, heartbeat/registry work, source reads, or AST walks.
- Active-span locks capture owned metadata/counters briefly; aggregate
  history, sort, format, and write after release. Test stalled aggregation while
  real workers enter and finish spans. Keep legitimate compute-once ownership.

### Measurement and PR reporting

- Every performance PR reports before/after results, exact baseline/candidate
  revisions, sample count, and scope. Include wall/CPU time and peak RSS when
  measured. State flat results and regressions; benchmark-only changes claim no runtime
  improvement. Do not add percentages from different comparisons.
- Match machine, toolchain, features, release flags, benchmark mode, thread/job
  counts, config/tsconfig, dependencies, and physical fixtures. Use fresh
  invocation caches and idle builds/tests. Check the final production source,
  not an obsolete head or mismatched release; label documentation-only updates.
- Verify distinct baseline/candidate binaries and their source provenance.
  Isolate Cargo targets or freeze verified artifacts: shared targets can reuse
  the wrong worktree executable. Discard comparisons that ran the same binary.
- Label test, documentation, and version-only rebases with the exact measured
  revisions and versions. Carry measurements forward only after proving analyzer
  source and third-party dependency versions unchanged; never describe an
  unmeasured version as measured. Runtime changes require rebuilds and new samples.
- Use balanced baseline/candidate/candidate/baseline order and adjacent reverse
  rechecks for surprising deltas. Compare report hashes, findings, warnings,
  advisories, and exit status; existing findings may legitimately exit 1.
- Separate preparation/projection, focused benchmarks, parallel phase timings,
  and end-to-end runtime. Overlapping parallel phases are not additive.
  Report Criterion estimates, intervals, noise/p-values, samples, and warm-up/
  measurement settings. Two-run means are observations, not guarantees; claim
  no workload gain when direction changes across matched rechecks.
- Work-count tests prove sharing, construction/copy/traversal reductions, and
  ownership invariants; equal reports alone do not prove the fast path. Fewer
  operations or a profile's CPU share do not establish measured speedup.
- Describe representative fixtures by source size and lines, JSX density, and
  configured payloads, not file count alone. Default fixtures can omit configured
  include/filter payloads; measure those paths explicitly.
- Separate fixed invocation startup from focused, baseline-compatible fresh
  analysis. Keep fixture inputs identical and reset analysis state per sample.
- Immutable context hoists must demonstrate workload benefit and quantify their
  memory tradeoff; amortized initialization alone does not prove runtime gain.
- Before adding scope-summary caches, measure cold construction and shared-DAG
  traversal against the reuse benefit. Mutations, captured frames, and remaps
  constrain reuse; invalidate summaries explicitly, and keep builder-only
  membership checks from eagerly collecting unrelated roots. A lower work
  count alone is not a performance gain.
- CodSpeed comparisons require the same `ubuntu-24.04-arm` runner and expected
  base commit. Never change code for architecture/fallback mismatches. Inspect
  system call limitations and rerun unexpected unrelated outliers before assigning
  a cause; preserve coverage floors and semantic guards throughout.

### Active diagnostics ownership

- Active progress belongs to the request observer and tracks a set of uniquely
  identified live spans, including parallel siblings. Start before the closure;
  remove with a guard on success, returned error, and unwinding. Disabled requests
  must not create clocks, IDs, active maps, heartbeat threads, or registry entries.
- Capture active spans, counters, and their clock under a short lock; release it
  before aggregating timing history. Stalled aggregation must not stop Rayon
  span entry and cleanup. Keep optional telemetry routing distinct from delivery:
  dropped progress must not trigger blocking warning or watchdog fallbacks.
- Rayon workers do not inherit the caller's thread-local observer. Capture it
  before scheduling and install/restore it per task so counters and spans are
  retained by the owning request. Guard concurrent requests against dropped
  observations and cross-request contamination.
- CLI progress is opt-in and flushes stderr only. Keep stdout and signal exit
  behavior intact; use the existing child-group forwarding listener for Unix
  termination snapshots, including lock waiting and disabled deadlines. Progress
  output uses a bounded best-effort queue and a detached writer with its own
  stderr descriptor; cleanup and signal/timeout exits must never join blocked I/O. SIGKILL
  diagnostics rely on already-flushed heartbeats.
- Preserve returned command errors through the normal CLI stderr boundary;
  best-effort progress delivery must never consume an authoritative error or
  convert it to `Ok`. Queue acceptance does not prove bytes were flushed.
  Preformat complete error lines before writing to avoid interleaving formatter
  fragments with independent diagnostic output; long writes need not be atomic.
- Verify actual CommonJS/ESM export parity and forward every declared invocation
  option through wrapper whitelists; new diagnostic fields must reach the native
  boundary, including artifact-writing helpers.
- N-API progress polling stays asynchronous and skips the invocation file lock.
  Use a declarative independent deferred-promise JSON binding with a bounded
  polling worker outside the libuv analysis pool, and a live-only Weak request
  registry; copy the small polling input into owned Rust bytes on the JS thread
  before queueing, and cover UTF-8/JSON/run/overload logic outside native-only
  coverage gates. Reject duplicate active IDs and remove entries on every
  completion/failure path. Never
  install CLI heartbeat/signal behavior or retain completed request payloads.
