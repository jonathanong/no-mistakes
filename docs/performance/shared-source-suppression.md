# Shared-source reads and suppression indexes

The retained changes share selected structured YAML source/documents between
freshness-rule applications and index disable comments for finding lookup.
All caches belong to one request and disappear when its `SourceStore` drops.
There is no filesystem cache, new configuration, or report-schema change.

## Candidate audit

| Original candidate | Current production path | Decision |
| --- | --- | --- |
| `rules/nextjs_no_caching.rs` | Aggregate checks already borrow source and Next.js facts from `CheckFactMap`; the direct read belongs to the standalone compatibility entrypoint. | Leave unchanged: the standalone pass reads each selected file once and has no shared request session. |
| `rules/nextjs_no_api_routes/aggregate.rs` | The prepared aggregate borrows `CheckFactMap` sources; `check_files` is the separate standalone loader. | Leave unchanged: adding a second local store would not remove repeated prepared reads. |
| `ts_routes/defs_backend/types.rs` | `dependencies/graph/edge_route_defs.rs` uses prepared facts when they cover `backend_routes`, then falls back to the standalone collector otherwise. | Leave the guarded compatibility fallback unchanged. |
| `dependencies/graph/edge_queue_processors.rs` | Prepared symbol facts supply exports. The optional fallback is already memoized by processor path in the graph build. | Leave unchanged; no per-edge repeated read remains in the prepared path. |
| `ts_queues/factory/reachability.rs` | The reading BFS helper has test callers only. | Drop the stale production-performance candidate. |
| `rules/ast_grep_ignore_freshness/scan.rs` | Every configured application formerly read and parsed the selected YAML again. | Retain shared source/documents: the no-findings repeated-application workloads isolate this gain. |
| `rules/suppression.rs` | Formerly cached text but scanned it again for each finding and again for accounting. | Retain indexed lookup: flooding-rule workloads measure the gain, with sparse controls below. |

The smaller SQL-option and ESLint candidates are unchanged: this comparison
provides no profile evidence to retain an optimization there. The existing
mixed Next.js benchmark remains a scope/control workload, not a claim of
improvement in the unchanged standalone loaders.

## Compatibility and work gates

The suppression index shares the existing directive grammar and lexer. A file
header is indexed once; line directives are lazy and indexed once per quote
mode, advancing only through the latest requested line and retaining lexer
state for later queries. Out-of-order queries reuse already indexed lines. A file directive avoids building the line index. A marker-free source
avoids the lexical line walk. Markdown prose and MDX preserve their different
quote modes. File, next-line, and same-line precedence and directive provenance
remain unchanged. Filtering and domain accounting borrow the same index.

Saved-fixture conformance tests compare indexed matching with the canonical
parser, including absent/zero lines, malformed rule identifiers, strings,
block comments, a BOM, and Markdown apostrophes. Concurrent filtering/accounting
assert one source read, one header construction and at most one line-index
construction; the dense source has exactly 3,105 lexical line visits rather
than a per-finding walk. An early finding with a quoted marker at the end of
a 3,006-line source visits only its first six lines. The file-disable control has zero line-index builds.

The structured YAML view deliberately retains the existing merge-key and
partial-document parser rather than substituting the generic YAML view.
Strict UTF-8 and I/O diagnostics keep their previous text. Concurrent tests
assert ten source reads and ten parses for one, twelve, and twenty applications,
including normalized-path reuse and cached failure results. Rule-specific
selection/project inheritance remains outside the cache.

The saved compatibility fixture has the same two reported locations
(`worker.ts:2`, `worker.ts:7`) and three suppressed locations (`file.ts:3`,
`worker.ts:4`, `worker.ts:5`) before and after. The suppressed directives remain
file line 1, next-line line 3, and same-line line 5 respectively. Its new JSON
snapshot runs at jobs 1 and 2; async N-API `check` and `analyzeProject` parity is
also pinned. No existing rule snapshot is changed.

## Native release comparison

The [raw samples](shared-source-suppression/native-samples.csv) contain all
972 measured invocations without rounding. [Metadata](shared-source-suppression/native-metadata.json)
records executable hashes, revisions, dependencies, toolchain, and profile.

The published-release baseline was `v0.109.115` at
`939e89bbf9d46e14f7af9c6bc418fee339b0b609`, selected before compilation.
The unmodified-main control is `fe94a73d2812658e10792edd0d0cced2bade135d`;
the measured candidate is `81b96255431eb312ef4f611127b34a6583cc302e`.
Later evidence/documentation-only commits do not change that production source.
The release/control project versions differ; third-party package versions are
identical, and the main/candidate lockfiles are byte-identical.

All three executables used stable Rust 1.100.0 on the same Linux x86-64 host
with 24 logical CPUs: release profile, default features, thin LTO, 16 code generation
units, eight build jobs, and no incremental compilation. Independent targets
or frozen copies prevented accidental executable reuse. Timing observers were
off. Measurements held the global heavy-job lock with no concurrent Cargo job.
Fresh processes include CLI startup and fresh request construction; filesystem
page caches were warmed, with no clock subtraction or persisted analysis cache.

Each workload warms each executable once, then uses three balanced blocks of
release/main/candidate/candidate/main/release (six samples per variant).
Jobs 1, jobs 4, and default jobs are separate cases. The script uses
`check --effort max --timeout 0 --format json`; accounting adds
`--include-suppressed`, and the mixed Next.js control selects `mixed.yml`.
Every sample verifies exact stdout, stderr, and exit-status equality, including
warnings and advisories; finding counts are checked before recording. All 54 cases pass.

The suppression sources have 3,000 static member calls and no JSX, with exact
configured targets, file traversal, and include filters; see the fixture README
for 3,005–5,006-line source sizes and directive density. YAML applications share
ten files totaling 690 lines and 18,127 bytes of rule payload. Their reports
are empty, so that comparison isolates YAML reuse from suppression filtering.
Existing core-analysis and mixed Next.js fixtures are broader scope controls.

Cells below are median **wall ms / CPU ms / peak RSS MiB**. Full ranges and
individual values remain in the CSV. Flooding-rule cases improve substantially;
repeated YAML applications save CPU, while default-job YAML wall time is mostly
flat because preparation and thread startup dominate. Sparse, file-disable,
long-header, early-finding, and broader controls are treated as flat; no speedup
is claimed for them. Whole-process RSS is higher than the release baseline
and lower than the unmodified-main control in these samples; these values
do not isolate cache allocations.


| Workload / accounting / jobs | Release | Main | Candidate |
| --- | ---: | ---: | ---: |
| `dense/accounting-0/jobs-1` | 1114.5 / 1113.4 / 51.9 | 1111.4 / 1109.8 / 62.6 | 54.8 / 53.4 / 57.1 |
| `dense/accounting-0/jobs-4` | 1118.7 / 1118.2 / 52.0 | 1122.3 / 1121.0 / 62.0 | 55.8 / 56.1 / 57.1 |
| `dense/accounting-0/jobs-default` | 1113.8 / 1117.1 / 52.3 | 1112.7 / 1116.2 / 63.5 | 58.1 / 62.3 / 57.1 |
| `dense/accounting-1/jobs-1` | 591.0 / 589.6 / 51.7 | 589.7 / 588.7 / 62.7 | 54.3 / 53.3 / 57.1 |
| `dense/accounting-1/jobs-4` | 589.1 / 587.0 / 51.9 | 585.7 / 585.7 / 63.2 | 55.3 / 55.4 / 57.6 |
| `dense/accounting-1/jobs-default` | 589.9 / 592.9 / 52.0 | 588.0 / 592.3 / 63.2 | 56.0 / 59.2 / 57.3 |
| `no-directives/accounting-0/jobs-1` | 1122.0 / 1120.5 / 51.8 | 1128.8 / 1127.2 / 62.3 | 53.5 / 52.5 / 57.0 |
| `no-directives/accounting-0/jobs-4` | 1125.5 / 1124.9 / 52.3 | 1129.4 / 1128.3 / 62.8 | 53.5 / 53.3 / 57.4 |
| `no-directives/accounting-0/jobs-default` | 1131.2 / 1132.1 / 52.1 | 1130.2 / 1134.5 / 62.8 | 55.3 / 59.4 / 57.8 |
| `no-directives/accounting-1/jobs-1` | 589.7 / 587.0 / 52.1 | 584.3 / 583.6 / 62.1 | 53.9 / 53.0 / 57.5 |
| `no-directives/accounting-1/jobs-4` | 592.3 / 591.9 / 52.7 | 583.8 / 582.9 / 62.5 | 54.2 / 54.5 / 58.4 |
| `no-directives/accounting-1/jobs-default` | 593.1 / 596.5 / 53.3 | 585.5 / 588.6 / 64.3 | 54.7 / 58.8 / 57.4 |
| `single/accounting-0/jobs-1` | 46.7 / 46.3 / 50.1 | 45.4 / 44.6 / 60.7 | 45.7 / 45.2 / 55.4 |
| `single/accounting-0/jobs-4` | 46.8 / 47.2 / 50.4 | 45.2 / 45.2 / 60.8 | 45.1 / 45.3 / 55.9 |
| `single/accounting-0/jobs-default` | 48.3 / 54.6 / 50.3 | 46.3 / 51.9 / 60.8 | 46.6 / 52.7 / 55.5 |
| `single/accounting-1/jobs-1` | 46.8 / 46.0 / 50.3 | 46.1 / 45.5 / 60.7 | 45.7 / 45.0 / 55.3 |
| `single/accounting-1/jobs-4` | 46.2 / 46.8 / 50.2 | 45.5 / 45.4 / 60.6 | 45.0 / 45.2 / 55.4 |
| `single/accounting-1/jobs-default` | 48.4 / 54.0 / 50.3 | 46.7 / 52.3 / 60.5 | 46.9 / 52.7 / 55.5 |
| `file-disable/accounting-0/jobs-1` | 49.4 / 48.8 / 51.7 | 48.4 / 47.7 / 62.4 | 48.5 / 47.5 / 57.0 |
| `file-disable/accounting-0/jobs-4` | 50.1 / 50.6 / 51.8 | 49.0 / 49.3 / 63.2 | 47.7 / 48.2 / 57.3 |
| `file-disable/accounting-0/jobs-default` | 52.0 / 57.3 / 53.8 | 51.1 / 55.8 / 64.1 | 50.2 / 56.4 / 58.0 |
| `file-disable/accounting-1/jobs-1` | 56.9 / 55.9 / 52.4 | 55.0 / 54.0 / 62.7 | 55.3 / 54.3 / 57.9 |
| `file-disable/accounting-1/jobs-4` | 56.8 / 56.3 / 52.9 | 55.3 / 55.6 / 63.2 | 56.3 / 56.0 / 57.5 |
| `file-disable/accounting-1/jobs-default` | 59.0 / 62.4 / 53.5 | 57.8 / 61.0 / 63.2 | 57.7 / 62.9 / 60.3 |
| `long-header/accounting-0/jobs-1` | 47.0 / 46.6 / 51.3 | 45.1 / 44.7 / 61.5 | 45.8 / 45.2 / 56.3 |
| `long-header/accounting-0/jobs-4` | 46.7 / 47.3 / 51.0 | 46.1 / 46.8 / 61.9 | 46.6 / 47.5 / 56.5 |
| `long-header/accounting-0/jobs-default` | 48.4 / 53.3 / 51.1 | 47.0 / 52.0 / 61.6 | 47.9 / 52.0 / 56.8 |
| `long-header/accounting-1/jobs-1` | 47.0 / 46.4 / 51.0 | 45.1 / 44.5 / 62.1 | 45.5 / 44.9 / 56.2 |
| `long-header/accounting-1/jobs-4` | 47.1 / 47.5 / 51.2 | 45.2 / 45.5 / 61.7 | 45.1 / 45.6 / 56.5 |
| `long-header/accounting-1/jobs-default` | 48.5 / 53.9 / 51.4 | 47.6 / 52.1 / 61.1 | 46.8 / 52.3 / 56.5 |
| `early-quoted-marker/accounting-0/jobs-1` | 47.1 / 46.6 / 50.3 | 45.5 / 45.0 / 61.1 | 45.4 / 44.7 / 55.6 |
| `early-quoted-marker/accounting-0/jobs-4` | 47.9 / 47.8 / 50.5 | 44.9 / 45.4 / 61.2 | 45.6 / 45.5 / 55.9 |
| `early-quoted-marker/accounting-0/jobs-default` | 49.7 / 54.2 / 50.6 | 46.3 / 51.6 / 60.4 | 46.9 / 52.1 / 56.1 |
| `early-quoted-marker/accounting-1/jobs-1` | 49.3 / 48.7 / 50.6 | 46.4 / 46.1 / 60.8 | 47.2 / 46.7 / 55.9 |
| `early-quoted-marker/accounting-1/jobs-4` | 48.4 / 48.1 / 50.8 | 46.0 / 46.3 / 61.0 | 47.0 / 46.9 / 55.4 |
| `early-quoted-marker/accounting-1/jobs-default` | 49.6 / 55.1 / 50.5 | 47.7 / 53.3 / 60.5 | 47.0 / 51.9 / 56.0 |
| `ast-grep-single/accounting-0/jobs-1` | 14.7 / 14.0 / 33.3 | 13.8 / 13.3 / 44.6 | 14.0 / 13.5 / 40.2 |
| `ast-grep-single/accounting-0/jobs-4` | 14.7 / 14.5 / 33.8 | 14.1 / 14.1 / 44.8 | 13.8 / 13.7 / 39.8 |
| `ast-grep-single/accounting-0/jobs-default` | 14.6 / 19.2 / 33.5 | 13.8 / 18.2 / 45.0 | 14.1 / 18.3 / 39.8 |
| `ast-grep-repeated-12/accounting-0/jobs-1` | 24.3 / 23.9 / 33.6 | 23.5 / 23.0 / 44.9 | 17.5 / 16.9 / 39.9 |
| `ast-grep-repeated-12/accounting-0/jobs-4` | 16.4 / 25.3 / 35.0 | 15.5 / 24.5 / 45.4 | 14.1 / 17.7 / 40.1 |
| `ast-grep-repeated-12/accounting-0/jobs-default` | 15.2 / 34.3 / 36.4 | 14.4 / 32.1 / 47.5 | 14.7 / 25.2 / 40.1 |
| `ast-grep-repeated-20/accounting-0/jobs-1` | 31.4 / 30.7 / 33.7 | 31.0 / 30.5 / 45.3 | 20.2 / 19.6 / 40.0 |
| `ast-grep-repeated-20/accounting-0/jobs-4` | 18.6 / 32.8 / 34.7 | 17.9 / 31.8 / 46.2 | 15.0 / 20.5 / 40.1 |
| `ast-grep-repeated-20/accounting-0/jobs-default` | 15.1 / 43.3 / 38.8 | 14.8 / 41.4 / 49.7 | 14.8 / 28.4 / 39.6 |
| `compatibility/accounting-1/jobs-1` | 31.7 / 31.4 / 42.5 | 29.8 / 29.5 / 52.9 | 30.0 / 29.6 / 47.9 |
| `compatibility/accounting-1/jobs-4` | 31.0 / 31.7 / 42.5 | 29.9 / 30.4 / 52.6 | 30.6 / 31.3 / 48.0 |
| `compatibility/accounting-1/jobs-default` | 31.3 / 37.6 / 42.5 | 30.1 / 36.0 / 52.9 | 30.2 / 36.0 / 48.0 |
| `core-analysis/accounting-0/jobs-1` | 38.0 / 37.1 / 50.1 | 35.7 / 34.9 / 57.5 | 36.5 / 35.5 / 53.0 |
| `core-analysis/accounting-0/jobs-4` | 36.8 / 39.3 / 50.5 | 35.0 / 37.0 / 58.3 | 35.0 / 37.4 / 53.3 |
| `core-analysis/accounting-0/jobs-default` | 37.8 / 56.3 / 50.5 | 35.6 / 52.6 / 58.9 | 36.5 / 53.7 / 53.8 |
| `nextjs-mixed/accounting-0/jobs-1` | 14.5 / 14.5 / 35.6 | 13.4 / 13.3 / 46.5 | 13.3 / 13.2 / 40.5 |
| `nextjs-mixed/accounting-0/jobs-4` | 14.1 / 15.6 / 35.7 | 13.1 / 14.1 / 46.6 | 13.5 / 14.5 / 40.6 |
| `nextjs-mixed/accounting-0/jobs-default` | 14.7 / 23.8 / 35.7 | 13.6 / 22.0 / 47.1 | 14.0 / 23.2 / 40.5 |

Reproduce with the [native harness](../../.github/scripts/suppression-index-performance/README.md).
The committed Criterion cases cover the same suppression and YAML workload
families; these tables are end-to-end CLI measurements, not Criterion estimates.
