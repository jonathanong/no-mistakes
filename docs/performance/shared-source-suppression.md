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
Raw header names use standard hashing for untrusted strings; numeric line
keys retain the fast map.

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

The published-release baseline is `v0.109.116` at
`fe94a73d2812658e10792edd0d0cced2bade135d`.
The unmodified-main control is `cc55270e4cd09b7c47004cc51f095eb4e1f32142`;
the measured candidate is `e31e725e51a4d840d9366c1ff714c62ec6473431`, including
standard hashing for raw header names. Later evidence/documentation-only
commits do not change that production source. All project and third-party
package versions match, and all three lockfiles are byte-identical.

All executables used stable Rust 1.100.0 on the same Linux x86-64 host with
24 logical CPUs: release profile, default features, thin LTO, 16 code generation
units, eight build jobs, and no incremental compilation. Independent targets
and frozen copies prevented accidental executable reuse. Timing observers
were off. Measurements held the global heavy-job lock with no concurrent
Cargo job. Fresh processes include CLI startup and fresh request construction;
filesystem pages were warmed, with no clock subtraction or persisted analysis.

Each workload warms each executable once, then uses three balanced blocks of
release/main/candidate/candidate/main/release (six samples per variant).
Jobs 1, jobs 4, and default jobs are separate cases. The script uses
`check --effort max --timeout 0 --format json`; accounting adds
`--include-suppressed`, and the mixed Next.js control selects `mixed.yml`.
Every sample verifies exact stdout, stderr, and exit-status equality, including
warnings and advisories; finding counts are checked before recording.
All 54 cases pass. The review follow-up replaced the earlier measurements,
rebuilt the updated-main control, and retained the same physical fixtures.

The suppression sources have 3,000 static member calls and no JSX, with exact
configured targets, file traversal, and include filters; see the fixture README
for 3,005–5,006-line source sizes and directive density. YAML applications share
ten files totaling 690 lines and 18,127 bytes of rule payload. Their reports
are empty, isolating YAML reuse from suppression filtering. Existing
core-analysis and mixed Next.js fixtures are broader scope controls.

Cells below are median **wall ms / CPU ms / peak RSS MiB**. Full ranges and
individual values remain in the CSV. Flooding-rule cases improve substantially;
repeated YAML applications save CPU, while default-job YAML wall time is mostly
flat because preparation and thread startup dominate. Sparse, file-disable,
long-header, early-finding, and broader controls are treated as flat; no speedup
is claimed for them. Whole-process RSS values are reported for all variants;
they do not isolate cache allocations.

| Workload / accounting / jobs | Release | Main | Candidate |
| --- | ---: | ---: | ---: |
| `dense/accounting-0/jobs-1` | 1120.5 / 1118.3 / 67.5 | 1103.9 / 1102.9 / 67.3 | 53.4 / 52.9 / 57.4 |
| `dense/accounting-0/jobs-4` | 1116.7 / 1116.3 / 67.6 | 1120.2 / 1116.6 / 67.5 | 54.3 / 54.2 / 57.1 |
| `dense/accounting-0/jobs-default` | 1118.0 / 1119.8 / 67.8 | 1118.1 / 1122.3 / 67.9 | 56.6 / 61.3 / 57.5 |
| `dense/accounting-1/jobs-1` | 595.6 / 594.3 / 67.8 | 593.6 / 591.2 / 67.4 | 54.7 / 53.9 / 57.7 |
| `dense/accounting-1/jobs-4` | 591.1 / 590.5 / 68.1 | 586.3 / 586.2 / 69.7 | 55.8 / 55.8 / 57.6 |
| `dense/accounting-1/jobs-default` | 592.9 / 595.9 / 69.6 | 591.4 / 596.3 / 68.2 | 55.2 / 60.3 / 57.3 |
| `no-directives/accounting-0/jobs-1` | 1133.2 / 1131.6 / 67.4 | 1131.4 / 1130.0 / 67.1 | 53.2 / 52.3 / 57.4 |
| `no-directives/accounting-0/jobs-4` | 1128.9 / 1128.2 / 67.7 | 1130.5 / 1127.9 / 67.1 | 53.3 / 53.2 / 57.2 |
| `no-directives/accounting-0/jobs-default` | 1116.6 / 1119.4 / 68.1 | 1132.9 / 1136.8 / 67.7 | 55.5 / 59.6 / 57.3 |
| `no-directives/accounting-1/jobs-1` | 589.0 / 587.6 / 67.8 | 588.1 / 587.0 / 67.5 | 53.5 / 52.7 / 56.9 |
| `no-directives/accounting-1/jobs-4` | 589.2 / 589.0 / 68.1 | 589.1 / 588.9 / 67.3 | 53.3 / 53.1 / 57.2 |
| `no-directives/accounting-1/jobs-default` | 590.6 / 594.6 / 67.7 | 591.2 / 594.4 / 68.7 | 56.7 / 59.4 / 58.1 |
| `single/accounting-0/jobs-1` | 45.3 / 44.6 / 65.5 | 45.2 / 44.3 / 65.9 | 45.3 / 44.6 / 55.3 |
| `single/accounting-0/jobs-4` | 44.5 / 44.9 / 65.6 | 44.3 / 44.9 / 65.9 | 44.8 / 45.5 / 55.3 |
| `single/accounting-0/jobs-default` | 46.2 / 52.7 / 66.0 | 45.9 / 51.8 / 65.8 | 46.3 / 52.5 / 55.7 |
| `single/accounting-1/jobs-1` | 44.5 / 44.0 / 65.5 | 44.9 / 43.9 / 64.7 | 45.5 / 44.8 / 55.3 |
| `single/accounting-1/jobs-4` | 45.0 / 45.1 / 65.8 | 45.6 / 45.4 / 66.0 | 45.3 / 45.5 / 55.4 |
| `single/accounting-1/jobs-default` | 45.8 / 51.5 / 65.9 | 46.0 / 51.7 / 65.7 | 46.7 / 52.9 / 55.4 |
| `file-disable/accounting-0/jobs-1` | 48.0 / 47.2 / 67.9 | 47.9 / 47.1 / 67.7 | 48.1 / 47.5 / 57.1 |
| `file-disable/accounting-0/jobs-4` | 47.5 / 47.0 / 67.6 | 48.8 / 48.9 / 67.8 | 47.8 / 47.9 / 57.2 |
| `file-disable/accounting-0/jobs-default` | 50.4 / 55.8 / 69.0 | 50.0 / 55.1 / 68.0 | 50.6 / 55.8 / 58.4 |
| `file-disable/accounting-1/jobs-1` | 55.0 / 54.0 / 68.5 | 54.6 / 53.9 / 67.7 | 55.6 / 54.7 / 57.6 |
| `file-disable/accounting-1/jobs-4` | 55.3 / 55.2 / 68.4 | 56.7 / 56.1 / 68.1 | 55.6 / 55.2 / 57.8 |
| `file-disable/accounting-1/jobs-default` | 57.7 / 61.1 / 68.4 | 56.5 / 60.0 / 68.0 | 57.9 / 62.6 / 60.6 |
| `long-header/accounting-0/jobs-1` | 44.6 / 44.2 / 66.2 | 44.5 / 44.0 / 66.7 | 46.1 / 45.5 / 56.6 |
| `long-header/accounting-0/jobs-4` | 44.6 / 45.1 / 66.6 | 45.2 / 45.4 / 66.4 | 45.8 / 46.0 / 56.4 |
| `long-header/accounting-0/jobs-default` | 45.7 / 51.7 / 66.7 | 47.0 / 52.4 / 66.5 | 46.1 / 52.5 / 56.5 |
| `long-header/accounting-1/jobs-1` | 44.6 / 44.0 / 66.6 | 44.3 / 43.8 / 65.8 | 46.2 / 45.1 / 56.2 |
| `long-header/accounting-1/jobs-4` | 44.7 / 45.2 / 66.6 | 45.3 / 45.4 / 66.5 | 45.0 / 45.2 / 56.2 |
| `long-header/accounting-1/jobs-default` | 46.3 / 51.2 / 67.2 | 46.4 / 51.8 / 66.2 | 46.8 / 51.8 / 56.9 |
| `early-quoted-marker/accounting-0/jobs-1` | 44.9 / 44.5 / 66.2 | 44.4 / 44.1 / 66.0 | 45.1 / 44.6 / 55.7 |
| `early-quoted-marker/accounting-0/jobs-4` | 44.7 / 45.1 / 65.4 | 45.2 / 45.3 / 65.7 | 45.9 / 46.2 / 56.6 |
| `early-quoted-marker/accounting-0/jobs-default` | 46.7 / 52.6 / 66.4 | 46.1 / 51.3 / 66.1 | 46.9 / 52.7 / 56.0 |
| `early-quoted-marker/accounting-1/jobs-1` | 44.7 / 44.2 / 66.0 | 45.1 / 44.5 / 66.1 | 45.8 / 45.0 / 55.7 |
| `early-quoted-marker/accounting-1/jobs-4` | 44.9 / 45.4 / 65.5 | 44.6 / 44.9 / 66.3 | 45.2 / 45.9 / 56.0 |
| `early-quoted-marker/accounting-1/jobs-default` | 46.7 / 51.9 / 65.9 | 47.1 / 51.6 / 66.3 | 47.3 / 52.0 / 56.1 |
| `ast-grep-single/accounting-0/jobs-1` | 12.8 / 12.7 / 51.7 | 12.9 / 12.8 / 49.7 | 13.1 / 13.0 / 39.3 |
| `ast-grep-single/accounting-0/jobs-4` | 13.1 / 13.5 / 51.0 | 13.1 / 13.4 / 49.7 | 13.7 / 14.0 / 39.2 |
| `ast-grep-single/accounting-0/jobs-default` | 13.4 / 17.7 / 51.3 | 13.6 / 18.0 / 49.3 | 14.1 / 17.9 / 39.8 |
| `ast-grep-repeated-12/accounting-0/jobs-1` | 23.1 / 22.8 / 51.7 | 23.2 / 23.0 / 49.9 | 17.0 / 16.8 / 39.6 |
| `ast-grep-repeated-12/accounting-0/jobs-4` | 15.0 / 24.0 / 52.4 | 15.1 / 24.3 / 51.3 | 14.3 / 18.2 / 39.4 |
| `ast-grep-repeated-12/accounting-0/jobs-default` | 14.5 / 32.9 / 55.0 | 14.2 / 32.7 / 53.2 | 15.0 / 26.0 / 39.2 |
| `ast-grep-repeated-20/accounting-0/jobs-1` | 30.5 / 30.2 / 51.7 | 30.3 / 30.0 / 50.3 | 19.8 / 19.4 / 39.5 |
| `ast-grep-repeated-20/accounting-0/jobs-4` | 17.4 / 31.3 / 53.1 | 17.6 / 31.4 / 51.1 | 15.2 / 20.7 / 40.0 |
| `ast-grep-repeated-20/accounting-0/jobs-default` | 14.1 / 41.9 / 56.5 | 14.5 / 41.6 / 54.7 | 14.9 / 30.0 / 39.0 |
| `compatibility/accounting-1/jobs-1` | 30.2 / 29.5 / 57.8 | 29.8 / 29.4 / 57.7 | 29.8 / 29.5 / 48.3 |
| `compatibility/accounting-1/jobs-4` | 28.9 / 29.5 / 57.7 | 29.2 / 29.9 / 57.7 | 29.7 / 30.1 / 47.5 |
| `compatibility/accounting-1/jobs-default` | 30.5 / 36.4 / 57.9 | 30.1 / 36.8 / 57.7 | 31.3 / 37.3 / 47.7 |
| `core-analysis/accounting-0/jobs-1` | 35.3 / 34.8 / 61.9 | 35.1 / 34.5 / 61.7 | 35.6 / 35.0 / 53.2 |
| `core-analysis/accounting-0/jobs-4` | 34.5 / 36.4 / 62.6 | 34.7 / 36.4 / 62.4 | 34.8 / 37.2 / 53.8 |
| `core-analysis/accounting-0/jobs-default` | 35.5 / 56.6 / 63.1 | 36.1 / 57.9 / 63.0 | 36.2 / 56.7 / 53.9 |
| `nextjs-mixed/accounting-0/jobs-1` | 12.9 / 12.8 / 52.7 | 13.1 / 12.8 / 52.3 | 13.4 / 13.0 / 40.4 |
| `nextjs-mixed/accounting-0/jobs-4` | 13.1 / 13.9 / 53.2 | 13.2 / 14.1 / 52.1 | 13.4 / 14.1 / 40.7 |
| `nextjs-mixed/accounting-0/jobs-default` | 13.4 / 23.0 / 54.1 | 13.7 / 23.8 / 52.1 | 13.6 / 24.0 / 40.5 |

Reproduce with the [native harness](../../.github/scripts/suppression-index-performance/README.md).
The committed Criterion cases cover the same workload families; these tables
are end-to-end CLI measurements, not Criterion estimates.
