# Request-owned resolver visibility

<!-- cspell:ignore RUSTC -->

Resolver visibility is interned within `AnalysisSession`. A content fingerprint
selects a bucket; full path membership decides whether two projections are
identical. The fingerprint never determines a known import target by itself.
Retained identities make selected resolver keys independent of repository size.
Standalone resolvers retain their complete path keys.

Each exact projection prepares normalized paths and canonical aliases once.
Canonicalization successes and failures are memoized per lexical path across
projections. Equal projections from separate domains retain one lookup and
share positive and negative import results. Narrower integration catalogs and
broader runner catalogs remain distinct. Queue compatibility retains its own
root and resolution policy in the cache identity.

Prepared graph classifications use this same session import table. Legacy
lookups with custom alias remapping retain their explicit raw projection;
attaching that projection never clears an already warmed session table.
Identity canonicalizations retain no duplicate canonical path text, and a
request that never prepares resolver visibility allocates no visibility registry.
Already prepared owners retain their original path storage when registered.
Normalization copies membership only when a spelling or canonical alias changes;
unchanged standalone runner scopes reuse their original storage too.

Runner configuration preparation precedes graph-session preparation. Its
existing prepared owner retains one frozen visibility projection and resolved
imports across configuration files and clones. This adds no competing session,
source store, discovery, or TS fact pass. Importer/config selection is immutable
within this owner.

Verbose diagnostics add these request work counters:

- `resolver.visible_normalizations`: distinct visibility projections normalized.
- `resolver.visible_canonicalize_calls`: actual path canonicalization attempts,
  including failures; overlapping projections reuse those attempts.
- `resolver.scope_constructions`: exact session resolver caches first created,
  including distinct queue policies and roots.

The existing `resolver.computations` counter measures actual import resolution.
Counters are absent when diagnostics are disabled. A request with genuinely
different projections may normalize more than one projection; equal domains
must normalize only once, and each lexical path is canonicalized once.

## Validation and measurements

Fixture-backed tests cover equal but separately allocated sets, concurrent
construction, unresolved imports, additive narrower/broader scopes, hash bucket
collisions, canonical aliases, failed canonicalization, queue policy/root
separation, and runner configuration reuse. Existing resolver/domain fixtures
protect public findings and catalog selection.

The final storage-sharing source passes the 98% gates: lines 99.32%, functions
99.32%, regions 98.49%. The all-feature workspace run covers library, binary,
and integration tests without filename exclusions. The library reports 11,718
passes, three existing ignores and one environment failure; binary tests report
303 passes and integration targets report 382/251/173/381/69 passes.
The failure is
`codebase::ts_source::tests::ignore_walk::ancestor_star_gitignore_does_not_hide_ad_hoc_directory_files`,
also reproduced on the unchanged baseline. It remains enabled and unmodified.
All seven focused visibility tests pass; three added storage-identity assertions
fail on the previous source. Configured repository checks, all four saved runner
fixture resolve checks, formatting, spelling and duplication checks pass.

Coverage uses isolated targets, stable Rust, empty `RUSTC_WRAPPER`, disabled
incremental compilation, zero development/test debug information, jobs 2, and
an explicit writable runtime directory:

```sh
cargo llvm-cov --workspace --all-features --lib --bins --tests --no-fail-fast --no-report
cargo llvm-cov report --summary-only --fail-under-lines 98 --fail-under-functions 98 --fail-under-regions 98
```

Matched measurement results and source provenance are recorded for this change
before handoff. The saved consuming/control configurations are in
`fixtures/performance/session-visible-resolver/`. They use the same physical
repository paths and explicitly limit checks to the Node package. Existing
`core-analysis` and `graph-gates` fixtures retain JSX and configured include
payloads. No issue estimate is a measured speed claim.

### Performance validation remains pending

The retained [development observations](../../fixtures/performance/session-visible-resolver/rejected-dev-60055-abba.csv)
and [provenance](../../fixtures/performance/session-visible-resolver/rejected-dev-60055-provenance.json)
describe a **rejected** earlier candidate, `60055c7bb`, against `871b3ce56`.
They do not measure the subsequent storage-sharing fix. All 120 fresh processes
used matched default-feature development builds, Rust 1.99.1, debug information
disabled, incremental compilation disabled, and one idle machine. Five workloads
ran with diagnostics off/on and jobs 1/4/automatic maximum, in ABBA order.
Each table value below is a median of two observations per binary.

| Workload, diagnostics, jobs | Wall seconds A/B | CPU seconds A/B | Peak RSS KiB A/B |
| --- | --- | --- | --- |
| Repository consuming, off, 1 | 0.925 / 0.915 | 0.920 / 0.910 | 192530 / 199538 |
| Repository consuming, off, 4 | 0.670 / 0.665 | 1.055 / 1.035 | 194052 / 201340 |
| Repository consuming, off, maximum | 0.660 / 0.665 | 2.005 / 1.805 | 198394 / 205742 |
| Repository control, off, 1 | 6.920 / 7.070 | 6.920 / 7.070 | 391042 / 394538 |
| Repository control, off, 4 | 4.135 / 4.095 | 7.275 / 7.215 | 391656 / 394380 |
| Repository control, off, maximum | 3.500 / 3.460 | 10.120 / 10.075 | 397608 / 402980 |

Reports, exit status, and non-diagnostic warnings matched within all 30 cells.
The matrix retained raw diagnostic stderr separately. Timing was near flat and
peak RSS increased, so this experiment does not satisfy the regression gate.
No speedup is claimed. It was stopped before reverse rechecks because concrete
unnecessary prepared-path copies had been found and needed correction.
Matched production-build wall/CPU/RSS results and investigation of anonymous
versus executable-page RSS remain required before marking the PR ready.

The physical repository contained 10,668 TS/JS sources, 5,551,085 bytes and
156,505 lines; 3,076 JSX/TSX sources contained 347,140 bytes and 9,424 lines.
The configured Node package selection contained 165 sources, 873,629 bytes
and 23,306 lines. Smaller saved controls include `core-analysis` (14 sources,
1,694 bytes, 63 lines; two TSX sources) and `graph-gates` (77 sources,
18,316 bytes, 389 lines; 20 JSX sources). The provenance records their JSX
bytes/lines and configured roots. Every process used the same physical inputs.
