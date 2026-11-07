# Exact bounded key proofs: R1–R4

<!-- cspell:ignore rustc getrusage RUSAGE INSTA Clippy -->

Current-main baseline: `7b2b16b3f7536e2310340935174a483756b8ae19`.
Measured candidate runtime source: `3a5ea6d32bdd0044bc98b76fae1caab204df9531`.
Toolchain: `rustc 1.99.1 (58cf8f9c3 2026-10-13)`.
Both builds use locked dependencies, default features and the repository's dev
profile. Canonical `no-mistakes` hard-link basenames in equal-length A/B directories
keep executable-name handling matched. The baseline target is separate from
the candidate target; both are
under `/tmp`, and frozen executables are verified before measurement.
Heavy Cargo uses `flock --close /tmp/nm-heavy.lock` and `CARGO_BUILD_JOBS=6`.
Both executables were verified against cold incremental caches; rebuilding
the candidate cold produced the same SHA256 as the earlier candidate.
The final evidence-only commit carries these measurements forward.

Main advanced during the task. The branch was rebased, all exact shapes were
reproduced again on current main, and this matched comparison replaces the
older-base timing experiments. Those older timings are not used for final
performance claims. A detected shared-target artifact reuse was rejected;
the retained measurements require distinct binaries and expected outputs.

## Reproductions and compatibility

Saved inputs: `test-cases/rules/postgres-bounded-statements/issue-3114-exact/fixture`.
All four exact classes still fail on current main. Already-passing variants
remain regression coverage and are not claimed as newly fixed.

| Configuration | Main findings | Candidate findings | Retained control |
| --- | ---: | ---: | --- |
| R1 nested modifying UPDATE FROM CTE | 3 | 2 | Non-key write driver |
| R2 typed UNNEST through ARRAY subquery | 3 | 2 | Unbounded table / opaque function |
| R3 recursive DELETE USING | 6 | 4 | Physical walk, capped anchor, SELECT, unsupported donor |
| R4 unqualified bounded column beside view | 9 | 4 | Ambiguous name / unknown wildcard projection |
| No configured rules | 0 | 0 | Whole report unchanged |
| SQL rule only, bounded rule off | 1 | 1 | Offset finding unchanged |

Each exact shape has a passing one-detail variant. The candidate removes nine
false-positive findings across six statements and preserves twelve control
findings across nine statements. Recursive finite changed-key domains do not
certify finite recursive rows or termination. Bare ownership uses explicit view
output names independently of view key/cardinality inference.

Both rule-off configurations preserve byte-identical stdout, stderr and status.
Legacy valid/invalid/key fixtures also preserve exact output/status (0/3/5
findings), including their 26 pre-existing unused-disable-directive advisories.
The internal fact golden adds only four `recursive_domain: false` markers;
removing them reproduces the prior bytes.

## Measurement protocol

Hardware counters are blocked (`perf_event_paranoid=4`), so user CPU is the
requested fallback; no instruction counts are claimed. Every sample runs a
fresh CLI with default effort and default worker count. The harness measures
`resource.getrusage(RUSAGE_CHILDREN)` user/system CPU deltas and raw wall time
around the child process; Python's CPU is excluded. No analysis state persists
between invocations. One unmeasured fresh invocation per binary/configuration
warms file caches before each comparison. The shared heavy lock excludes
competing cooperating Cargo builds during samples.

The directory has 16 files / 25,009 bytes: nine SQL files / 379 physical SQL
lines, one catalog and six YAML configurations. It has no TS/JS/JSX source.
Consuming configs name explicit SQL files with `executorNames: []`; R4 also
configures column/view bounds. These are SQL regression workloads, not a
representative mixed-language application throughput benchmark. Positive-case
report counts intentionally change, so these are end-to-end observations,
not an isolated solver benchmark.

[A/B/B/A samples](balanced.csv): 25 invocations per phase/configuration,
50 per binary/configuration. [B/A/A/B control recheck](reverse.csv): 50 per
phase, 100 per binary/control. A is baseline, B candidate. Times below are ms.

| Configuration | User CPU mean A → B | User CPU median A → B | Wall mean A → B |
| --- | ---: | ---: | ---: |
| R1 | 26.527 → 25.397 | 26.266 → 26.044 | 25.792 → 28.350 |
| R2 | 25.655 → 25.856 | 26.132 → 25.913 | 25.360 → 28.215 |
| R3 | 35.053 → 35.544 | 35.455 → 36.302 | 35.460 → 39.043 |
| R4 | 29.679 → 29.221 | 29.460 → 30.024 | 28.958 → 32.083 |
| No configured rules | 11.251 → 11.155 | 11.464 → 11.050 | 15.995 → 17.558 |
| SQL rule only, bounded rule off | 37.045 → 38.743 | 37.456 → 38.790 | 26.372 → 28.095 |

Reverse controls:

| Configuration | User CPU mean A → B | User CPU median A → B | Wall mean A → B |
| --- | ---: | ---: | ---: |
| No configured rules | 11.754 → 12.212 | 11.444 → 12.383 | 15.743 → 17.019 |
| SQL rule only, bounded rule off | 39.698 → 40.046 | 39.582 → 40.503 | 26.319 → 28.421 |

Descriptive control mean CPU differences (candidate minus baseline):

| Order | Control | Difference (ms) | Normal 95% interval (ms) |
| --- | --- | ---: | ---: |
| A/B/B/A | No configured rules | -0.096 | [-1.326, 1.134] |
| A/B/B/A | SQL rule only, bounded rule off | 1.698 | [0.203, 3.193] |
| B/A/A/B | No configured rules | 0.457 | [-0.429, 1.344] |
| B/A/A/B | SQL rule only, bounded rule off | 0.349 | [-0.955, 1.652] |

The dev-profile wall-time controls increase by roughly 1–3 ms in repeated
runs. Canonical executable names and a cold-cache candidate rebuild do not
remove that difference. An identical-binary null control was nearly flat.
An optimized matched comparison remains required before performance handoff.

These observations do not demonstrate workload speedups or prove equivalent
performance. The intervals assume independent samples and omit other possible
host/phase noise. No throughput improvement is claimed.

[Peak RSS samples](rss.csv), five invocations per phase (ten per binary/case):

| Configuration | Median KiB A → B | Maximum KiB A → B |
| --- | ---: | ---: |
| R4 | 127,952 → 113,536 | 128,988 → 114,740 |
| SQL rule only, bounded rule off | 130,504 → 115,010 | 132,348 → 115,940 |

RSS includes executable mappings and host paging; no heap-memory gain is claimed.

Frozen executable SHA256:

- A: `190d0176e803c3391098c624817739194f08a1e8a04158be84c6614f8ba0ab5d`
- B: `f761f3a97662263ae3a54c57838ba7f15529f5b9d7ff60e43d3136a8a3b1ed7f`

Input hashes: [inputs.sha256](inputs.sha256). To reproduce, build/freeze each
revision with `cargo build --locked -p no-mistakes --bin no-mistakes`, using
separate targets under `/tmp`. Set `NM_BASELINE_BINARY`, `NM_CANDIDATE_BINARY`
and a unique `NM_MEASUREMENT_LABEL`, then run [measure.py](measure.py) with
`balanced` and `reverse` under `flock --close /tmp/nm-heavy.lock`.
The harness rejects identical binaries, incorrect finding counts/status,
stderr, warnings and advisories before retaining samples.

The measured child command has this shape, with no effort/jobs override:

```sh
root="$PWD/test-cases/rules/postgres-bounded-statements/issue-3114-exact/fixture"
XDG_RUNTIME_DIR=/tmp/nm-3114-exact-runtime "$binary" check \
  --root "$root" --config "$root/r1.yml" --format json
```

Repeat for `r1`, `r2`, `r3`, `r4`, `off` and `off-sql`.

## Validation

The rebased head passes strict all-target/all-feature Clippy and the complete
locked workspace/all-feature suite (13,324 library tests, 488 binary tests,
638 PostgreSQL CLI tests and 107 contract tests), including the snapshot execution reference
inventory with `INSTA_UPDATE=no`. Exact CLI snapshots compare jobs 1 and 2;
standalone/prepared aggregate API implementation parity covers all six configs.
Existing nested-write snapshots intentionally attribute remaining findings to
the changed target instead of auxiliary tables.

Independent review found no actionable proof/ownership defects. Focused LLVM
coverage was run early with no filename exclusions and saved SQL cases cover
reachable defensive branches. Structural PostgreSQL parser/extractor guards
remain conservative; the private signature grouping adds no branches. CI
validates current-head coverage floors. Changed Rust source files remain within
200 physical lines; test files within 500, audited with `wc -l`. No inline
fixture writes were added. Formatting, spelling and duplicate detection pass.

Tests use isolated `/var/tmp` temporary directories because an existing
`/tmp/.git` changes an unrelated ancestor-ignore fixture; Cargo artifacts remain
under `/tmp`. Task-local PATH selects ShellCheck 0.11.0 and CI's ast-grep 0.44.1.
Task-owned targets/worktrees are removed before handoff; the main workspace
target is never cleaned.
