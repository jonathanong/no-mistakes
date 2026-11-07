# Exact bounded key proofs: R1–R4

<!-- cspell:ignore rustc nonrecursive getrusage RUSAGE INSTA doctest doctests codegen -->

Baseline: fresh `origin/main`,
`759777752f2399d9b83f17e27b0ac8af6a9fe6d7`.
Toolchain: `rustc 1.99.1 (58cf8f9c3 2026-10-13)`.
Both CLI builds use locked dependencies, default features and the dev profile.
Heavy Cargo commands use `flock --close /tmp/nm-heavy.lock`,
`CARGO_BUILD_JOBS=6` and a task-owned target under `/tmp`.

The frozen default-profile baseline CLI SHA256 is
`50ed9806684201fb8a0524972e8b2f46a310e1b59fb7e99d2367eb1481d80e54`.
The new fix branch starts from main independently of regression PR #3122.

## Reproductions

Saved catalog and SQL:
`test-cases/rules/postgres-bounded-statements/issue-3114-exact/fixture`.

- R1 nests a keyed `UPDATE FROM` in a modifying CTE under `SELECT`.
  Its top-level write variant passes on main. The non-key driver stays reported.
- R2 transports a parameter-array `unnest` through a materialized CTE and
  `ANY(ARRAY(SELECT ...))`. Direct `ANY($1)` passes on main.
  An unbounded table source and a typed opaque-function source stay reported.
- R3 drives a mutation through a recursive union whose projected keys come
  from a shared, independently bounded donor. The nonrecursive variant passes
  on main. Physical-table walks, capped anchors over unbounded donors,
  ordinary SELECT joins and unsupported donor materialization remain controls.
  Finite changed-key domains do not certify finite recursive output rows.
- R4 resolves a bare configured bounded column beside a catalog view with
  explicit projected names. Qualified and no-view variants pass on main.
  Ambiguous names and unknown wildcard outputs stay conservative.
  A grouped-view case checks column names independently of key inference.

All four exact repro classes fail on the frozen main CLI. Their one-detail
passing variants and still-reported controls are retained. Each configuration
gets a committed `check_json_snapshot` comparison at jobs 1 and 2.

## CPU measurement protocol

Hardware counters are unavailable on this host (`perf_event_paranoid=4`).
Use user CPU time, per the task's fallback, rather than claiming instruction
counts.

Each sample launches a fresh CLI invocation with default effort and worker
count. The harness records deltas of `resource.getrusage(RUSAGE_CHILDREN)`
user/system CPU and raw wall time around the subprocess. It validates each
JSON report and retains status and stdout hash. Python's own CPU is excluded.
OS file caches are warm; no analysis state persists between CLI invocations.
The saved directory has 16 files / 25,009 bytes, including nine SQL files /
379 physical SQL lines, one catalog and six YAML configurations. It has no
TS/JS/JSX source. Each consuming configuration uses an explicit SQL filename
and `executorNames: []`; R4 additionally configures column and view bounds.
These are small SQL regression workloads, not a representative mixed-language
application benchmark.

A 25-sample baseline preflight established the workload. The final comparison
uses balanced baseline/candidate/candidate/baseline order, 25 fresh samples
per configuration per phase (50 samples per binary per configuration).
The shared heavy lock is held during measurement to exclude competing
cooperating Cargo builds. Reports intentionally change only where proofs fix
the saved false positives; rule-off output must remain byte-identical.

Measured commands have this shape; no effort or jobs override is supplied:

```sh
root="$PWD/test-cases/rules/postgres-bounded-statements/issue-3114-exact/fixture"
XDG_RUNTIME_DIR=/tmp/nm-3114-exact-runtime "$binary" check \
  --root "$root" --config "$root/r1.yml" --format json
```

Repeat for `r1`, `r2`, `r3`, `r4`, `off` and `off-sql`. Configuration order rotates
between samples. CPU summaries and output comparisons are recorded alongside
this file after the final candidate is built. These end-to-end measurements
include intentional diagnostic-count changes; they are not isolated solver
benchmarks or a production throughput claim.

## Validation environment

Tests use an isolated directory under `/var/tmp` because an existing
`/tmp/.git` changes an unrelated ancestor-ignore fixture. Cargo artifacts
remain under `/tmp`. Select installed ShellCheck 0.11.0 and CI's ast-grep
0.44.1 through task-local PATH setup. Remove all task-owned Cargo targets and
worker worktrees before handoff; never clean the main workspace target.

The internal fact golden adds only four `recursive_domain: false` markers.
Removing these markers exactly reproduces the prior bytes; legacy call
fields and original/projected fact equality remain checked.

## Default-profile observations

The candidate production revision is
`2c2022f91df09c17b7510feb4804b3ef83528b0b` (later fixture, test and documentation changes do not alter it).
Its CLI SHA256 is
`588d7863aae1991b8bd061e3082ba58ca3c7ef4ebd06799fffb65631d365511d`.
[A is baseline and B is candidate](balanced.csv); times below are milliseconds.
Every invocation uses default effort and jobs.

| Saved configuration | User CPU mean A → B | User CPU median A → B | Wall mean A → B |
| --- | ---: | ---: | ---: |
| R1 | 28.389 → 27.832 | 29.129 → 28.387 | 27.624 → 29.183 |
| R2 | 26.845 → 27.785 | 27.185 → 26.835 | 26.893 → 28.203 |
| R3 | 36.759 → 37.689 | 37.328 → 38.291 | 38.571 → 39.978 |
| R4 | 30.134 → 30.551 | 30.278 → 30.551 | 31.030 → 33.113 |
| No configured rules | 11.124 → 12.343 | 11.616 → 12.174 | 16.974 → 17.401 |
| SQL rule only, bounded rule off | 38.027 → 39.916 | 37.944 → 40.094 | 27.652 → 28.688 |

The adjacent [reverse recheck](reverse.csv) uses B/A/A/B, 50 samples per
phase (100 per binary and control):

| Control | User CPU mean A → B | User CPU median A → B | Wall mean A → B |
| --- | ---: | ---: | ---: |
| No configured rules | 12.487 → 11.903 | 12.646 → 11.809 | 16.802 → 17.280 |
| SQL rule only, bounded rule off | 38.794 → 39.708 | 39.020 → 40.648 | 28.005 → 28.484 |

The empty-rule CPU direction changes, so it supports no speed claim.
The SQL control's user CPU increases by 1.888 ms (4.97%) in A/B/B/A and
0.914 ms (2.36%) in B/A/A/B. These unoptimized end-to-end observations
must not be described as neutral. Separate opt-in timing/work-count probes
have identical source bytes, reads, requests, inventories and fact-collection
counts (25,009 bytes, 16 reads, 23 requests, one collection). This control
never requests bounded-statement facts or a schema catalog; the new proof
logic and lazy view projection cache are outside its executed path.

GNU time peak RSS, B/A/A/B with five samples per phase (ten per binary):
R4 median 113,926 → 113,454 KiB, maxima 114,512 → 114,256 KiB;
SQL rule-off control median 115,074 → 115,248 KiB,
maxima 116,716 → 115,808 KiB. No memory reduction is claimed.

## Report compatibility

| Configuration | Main findings | Candidate findings |
| --- | ---: | ---: |
| R1 | 3 | 2 |
| R2 | 3 | 2 |
| R3 | 6 | 4 |
| R4 | 9 | 4 |
| No configured rules | 0 | 0 |
| SQL rule only, bounded rule off | 1 | 1 |

The fixes remove nine false-positive findings across six statements while
preserving twelve control findings across nine statements. All four exact
repro classes fail on main; the already-passing variants remain coverage
and are not claimed as newly fixed. Both rule-off configurations have
byte-identical stdout, stderr and status. Existing valid, invalid and key
fixtures also have byte-identical output/status (0, 3 and 5 findings), including
their 26 pre-existing unused-disable-directive advisories.

Input hashes are saved in [inputs.sha256](inputs.sha256). The measurement
[harness](measure.py) expects frozen binaries under `/tmp`; run it under
`flock --close /tmp/nm-heavy.lock` after matching locked default-feature builds.

The artifact gate requires distinct binary SHA256 values and each binary's
expected repro finding counts. Shared-target Cargo reuse was detected in the
initial optimized build; those identical-binary samples were discarded. The
optimized candidate is forcibly recompiled before the retained comparison.

## Validation results

- Full locked workspace/all-feature library, binary and integration suites pass,
  including 13,276 library tests, 488 binary tests and all 630 PostgreSQL CLI
  tests. Three existing library tests remain ignored. The execution-based
  snapshot reference inventory passes with `INSTA_UPDATE=no`.
- The 107 contract tests pass, including executable rule documentation examples.
  The separate all-feature doctest command passes (zero Rust doctests).
- Six exact-fixture CLI snapshots compare jobs 1 and 2. Async API implementation
  parity compares standalone and prepared aggregate check results for all six
  configurations, including the SQL rule-off control.
- An independent integrated proof review found no actionable defects. Saved
  guard cases exercise target attribution, temporary writes, recursive source
  scope, correlation, transformations and conservative ownership.
- Changed Rust sources stay within 200 physical lines (largest: 198); test
  sources stay within 500. Formatting, spelling and duplicate detection pass
  (zero clones). No inline test fixture writes were added.

Focused coverage uses `cargo llvm-cov --lib -- postgres` with no filename
exclusions: 3,256 tests pass and two existing tests remain ignored.

| Changed helper | Executed / instrumented lines |
| --- | ---: |
| Catalog view definition | 58 / 58 |
| Typed UNNEST output | 36 / 40 |
| Recursive domain extraction | 91 / 92 |
| Bound evaluation | 130 / 130 |
| Domain propagation | 111 / 114 |
| Write target projection | 33 / 33 |
 Remaining uncovered new guards reject shapes that PostgreSQL's
parser or the canonical extractor cannot construct: non-PostgreSQL array type
AST variants, wildcard outputs removed before projection, invalid recursive
arm metadata, and peer-column metadata on non-singleton pin sources. The
RETURNING width mismatch is already rejected by its sole caller. Saved SQL
cases cover the reachable conservative branches rather than fabricating
invalid internal facts solely for coverage.

## Verified optimized comparison

Both optimized builds use the repository's unchanged release profile
(`lto = true`, `codegen-units = 1`), locked dependencies and default features.
The baseline remains `759777752f2399d9b83f17e27b0ac8af6a9fe6d7`;
the measured candidate checkout is
`039eebeb90b1daeb22cc8a80d0d92fa3ff019db5`.
All production source and dependency versions match the earlier candidate
production revision; intervening changes contain fixtures, tests and docs.
The final evidence-only commit carries these measurements forward.

Frozen optimized CLI SHA256:

- Baseline: `c5cc6668f278952192d82740ea40fbaf606e3580fd5a8420e1433bf5637d8c5f`
- Candidate: `1e2824bbc08e3558b238f3068ef5a1c98d734d2f4d84a8f7e66a7d4a9b51b60a`

Every sample passes the expected finding-count gate. The
[optimized A/B/B/A samples](release-balanced.csv) use 50 invocations per
binary/configuration; the [B/A/A/B control recheck](release-reverse.csv) uses
100. Effort, jobs, directory contents, config payloads and toolchain match.
Times are milliseconds.

| Configuration | User CPU mean A → B | User CPU median A → B | Wall mean A → B |
| --- | ---: | ---: | ---: |
| R1 | 8.714 → 8.575 | 8.111 → 8.461 | 12.787 → 12.986 |
| R2 | 9.528 → 8.205 | 9.523 → 7.938 | 12.943 → 13.230 |
| R3 | 9.412 → 9.658 | 9.525 → 9.332 | 14.303 → 14.543 |
| R4 | 8.845 → 8.416 | 8.743 → 8.567 | 13.216 → 13.515 |
| No configured rules | 4.686 → 4.830 | 4.610 → 4.484 | 10.190 → 10.558 |
| SQL rule only, bounded rule off | 10.978 → 10.865 | 11.241 → 11.254 | 12.625 → 13.126 |

Reverse controls:

| Control | User CPU mean A → B | User CPU median A → B | Wall mean A → B |
| --- | ---: | ---: | ---: |
| No configured rules | 4.761 → 4.998 | 4.681 → 5.000 | 10.307 → 10.477 |
| SQL rule only, bounded rule off | 11.028 → 10.691 | 11.076 → 10.635 | 12.861 → 13.029 |

The empty-rule mean increases 0.144 ms then 0.237 ms, while its median changes
direction. The SQL control mean decreases 0.112 ms then 0.337 ms. These are
observations, not demonstrated throughput improvements or proof of equivalent
performance. Descriptive normal 95% intervals for the mean CPU differences
are respectively [-0.624, 0.912], [-0.394, 0.869], [-1.174, 0.950] and
[-1.081, 0.407] ms; all include zero. These within-run intervals assume
independent samples and do not account for every source of host/phase noise.
The dev-profile SQL control increase remains disclosed above.

[Peak RSS samples](rss.csv), optimized runs: R4 medians 49,948 → 32,466 KiB
(maxima 51,040 → 32,736); SQL rule-off medians 49,644 → 32,026 KiB
(maxima 51,172 → 32,328). RSS includes executable mappings and host paging;
these observations are not attributed to a heap-memory improvement.

To repeat the optimized comparison, set `NM_BASELINE_BINARY` and
`NM_CANDIDATE_BINARY` to separately verified frozen executables and
`NM_MEASUREMENT_LABEL` to a unique output name, then run `measure.py balanced`
and `measure.py reverse` under the heavy lock. The harness rejects identical
artifacts, unexpected findings, warnings and advisories before retaining results.

The verified optimized binaries also preserve byte-identical stdout, stderr and
exit status for both rule-off configurations and the legacy valid/invalid/key
fixtures. The report counts match the default-profile compatibility table.
