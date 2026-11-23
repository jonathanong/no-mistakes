# Shared source and suppression indexing

This change reuses request-scoped source and parsed data where the current
analysis path permits it, and indexes disable comments once per source file
before matching findings. It does not change finding or report semantics.

## Original source-read candidates

| Original candidate | Current path | Status |
| --- | --- | --- |
| `rules/nextjs_no_caching.rs` standalone `check_files` | Reads and extracts source itself. The aggregate entrypoint separately consumes `CheckFactMap` source and Next.js facts. | Active standalone candidate; it is not the aggregate prepared-facts path. |
| `rules/nextjs_no_api_routes/aggregate.rs` | Aggregate path takes source from `CheckFactMap` and passes borrowed source items to the checker. | Already shared; no remaining aggregate source read to optimize. |
| `ts_routes/defs_backend/types.rs` | Uses prepared symbol facts when available and has a guarded fallback for missing facts. | Facts-covered on the normal prepared path; retain fallback for optional/missing symbol facts. |
| `dependencies/graph/edge_queue_processors.rs` | Uses prepared TS symbol facts when present; otherwise reads and extracts the processor file. | Active optional symbol-facts fallback candidate. |
| `ts_queues/factory/reachability.rs` | `bfs_reachable` reads each reached file and parses imports during traversal. | Test-only helper; not an active production analysis path. |
| `rules/ast_grep_ignore_freshness/scan.rs` | Loads selected YAML through `SourceStore::parse_structured_yaml_path`, sharing source and structured YAML documents. | Now shared. Existing entry selection, parse-error handling, and finding semantics remain in the rule. |

The suppression path also now obtains a per-file `DisableCommentIndex` and
looks up each finding against that index. With a request `SourceStore`, source
text and the index are request-scoped; the raw-source path builds the same
index locally for the suppression pass.

## Measurement status

No before/after measurements are recorded here yet. **Parent PR: fill in exact
wall time, CPU time, peak RSS, sample count, baseline and candidate revisions,
and benchmark scope after the required release-build comparison.** Do not
infer performance from fixture counts or index construction counts. Keep only
optimizations with measured benefit under the repository performance guidance.

| Comparison | Wall time | CPU time | Peak RSS | Samples / scope |
| --- | --- | --- | --- | --- |
| Before → after | Pending parent measurements | Pending parent measurements | Pending parent measurements | Pending parent measurements |

The saved `fixtures/performance/ast-grep-source-reuse/` corpus has one-app,
12-app, and 20-app cases with ten rule YAML files per app (plus its app config).
It establishes input
shape and semantic correctness only; it is not a timing result.
