# Exact bounded key proofs: R1–R4

<!-- cspell:ignore rustc getrusage RUSAGE INSTA Clippy codegen -->

Optimized baseline: `43da2047db3f8191283d1de68744f1c1f604add9`.
Candidate runtime source: `6907e75757520a41165d44d67fb688239cb857b8`.
These historical samples apply to the captured production sources. The later
output-label correction changes production code and needs a separate comparison;
these samples do not establish current-head performance.
Toolchain: `rustc 1.99.1 (58cf8f9c3 2026-10-13)`.
Builds use locked dependencies, default features, matching release overrides
`CARGO_PROFILE_RELEASE_LTO=thin` and `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16`.
Heavy Cargo uses `flock --close /tmp/nm-heavy.lock`, `CARGO_BUILD_JOBS=6` and
task-owned targets under `/tmp`, removed before handoff.

## Reproductions and compatibility

Inputs: `test-cases/rules/postgres-bounded-statements/issue-3114-exact/fixture`.
All four exact classes fail on main. Their already-passing one-detail variants
remain regression coverage and are not claimed as newly fixed.

| Configuration                             | Main statements | Candidate statements | Retained control                                        |
| ----------------------------------------- | --------------: | -------------------: | ------------------------------------------------------- |
| R1 nested modifying UPDATE FROM CTE       |               2 |                    1 | Non-key write driver                                    |
| R2 typed UNNEST through ARRAY subquery    |               3 |                    2 | Unbounded table / opaque function                       |
| R3 recursive DELETE USING                 |               6 |                    4 | Physical walk, capped anchor, SELECT, unsupported donor |
| R4 unqualified bounded column beside view |               4 |                    2 | Ambiguous name / unknown wildcard projection            |
| No configured rules                       |               0 |                    0 | Whole report unchanged                                  |
| SQL rule only, bounded rule off           |               1 |                    1 | Offset finding unchanged                                |

Six statement findings are fixed and nine negative statements remain reported.
Recursive finite changed-key domains do not certify finite recursive rows or
termination. Bare ownership uses explicit view output names independently of
view key/cardinality inference. Both rule-off configurations preserve
byte-identical stdout, stderr and exit status. The broader SQL-only rule-off
configuration also preserves all bytes (one finding and 26 existing advisories).
The internal fact golden adds only four `recursive_domain: false` markers;
removing them reproduces the prior bytes.

## Measurement protocol

Hardware counters are blocked (`perf_event_paranoid=4`), so user CPU is the
requested fallback; no instruction counts are claimed. Every sample runs a
fresh CLI with default effort and default worker count. The harness measures
`resource.getrusage(RUSAGE_CHILDREN)` user/system CPU deltas and raw wall time;
Python CPU is excluded. No analysis state persists between invocations.
Canonical hard-link executable basenames and equal-length A/B directories
match executable-name handling. Distinct frozen binaries and expected report
counts are verified before retaining samples. The heavy lock excludes
competing cooperating Cargo builds during samples.

[measure.py](measure.py) runs one unmeasured warmup per binary/configuration,
then A/B/B/A phases of 25 samples (50 per binary/configuration), with B/A/A/B
control phases of 50 samples (100 per binary/control). The exact root has
16 files / 25,009 bytes, including nine SQL files / 379 physical SQL lines,
one catalog and six configurations, with no TS/JS/JSX source.

[measure_wide.py](measure_wide.py) uses the existing saved PostgreSQL fixture
root and saved `test-cases/performance/postgres-bounded-3114/{on,off}.yml`.
Four preflight runs precede its A/B/B/A and B/A/A/B samples; it has no separate
warmup loop. These are SQL regression workloads, not mixed-language application
throughput measurements. Positive-case output intentionally changes (broader
fixture findings: 1,056 → 1,053), so timings are end-to-end observations.

Mean times in milliseconds, A baseline → B candidate:

| Run / configuration |          User CPU |              Wall |
| ------------------- | ----------------: | ----------------: |
| balanced: r1        |     9.125 → 8.632 |   13.252 → 14.061 |
| balanced: r2        |     8.641 → 8.920 |   12.969 → 14.235 |
| balanced: r3        |    9.751 → 10.666 |   14.688 → 15.767 |
| balanced: r4        |     9.480 → 9.801 |   13.842 → 14.901 |
| balanced: off       |     5.107 → 5.023 |   10.584 → 11.208 |
| balanced: off-sql   |   11.588 → 11.067 |   13.318 → 13.852 |
| reverse: off        |     5.043 → 4.876 |   10.546 → 10.788 |
| reverse: off-sql    |   11.540 → 11.432 |   13.348 → 13.566 |
| wide-balanced: on   | 462.182 → 458.742 | 250.518 → 246.869 |
| wide-balanced: off  | 177.955 → 175.413 |   41.130 → 41.303 |
| wide-reverse: off   | 171.825 → 172.059 |   38.912 → 39.127 |

The strengthened harness also passes a [recheck](harness-recheck.json) with
explicit binary identities, rule-off byte comparisons and validated RSS reports.
Broader user CPU recheck: enabled 463.174 → 462.346 ms; disabled ABBA
175.779 → 177.480 ms and reverse 170.868 → 171.122 ms. Corresponding wall
means are 252.186 → 250.053, 41.079 → 40.122 and 38.568 → 41.231 ms.
Wall differences vary across these host runs; no neutral-wall-cost claim is made.
The rechecked RSS medians are R4 51,594 → 50,826 KiB and SQL-only rule-off
52,426 → 51,106 KiB, further demonstrating why RSS is not a heap-gain claim.

Raw samples: [exact balanced](balanced.csv), [exact reverse](reverse.csv),
[broader balanced](wide-balanced.csv), [broader reverse](wide-reverse.csv).
Rule-off CPU differences do not consistently reproduce across orders. The
small optimized controls retain a repeatable roughly 0.2–0.6 ms wall-time
increase. Earlier dev-profile controls showed roughly 1–3 ms; canonical names
and cold-cache rebuilds did not remove it, while an identical-binary null
control was nearly flat. No equivalent-performance or throughput gain is
claimed. Broader rule-off wall time differs by about 0.2 ms.

Optimized peak RSS medians (KiB, ten samples each): R4 53,754 → 44,854;
SQL-only rule-off 54,718 → 44,450. RSS includes executable mappings and paging;
no heap-memory gain is claimed. Historical dev RSS samples remain in [rss.csv](rss.csv).

Frozen SHA256:

- A: `e30bb4e885e8e2e8f2200885b017b1d9be7005168a645f958e554fc507beb597`
- B: `61aff52ea54b2270ae5f22318fad8ed1f55b12bed5e48d6618c6570bc7c8027f`

Input hashes: [inputs.sha256](inputs.sha256). Build/freeze each revision with
`cargo build --locked --release -p no-mistakes --bin no-mistakes` and the
matching overrides above. Set `NM_BASELINE_BINARY`, `NM_CANDIDATE_BINARY` and
`NM_MEASUREMENT_LABEL`, then run `measure.py balanced` and `measure.py reverse`
under the heavy lock. The measured command has no effort/jobs override:

```sh
root="$PWD/test-cases/rules/postgres-bounded-statements/issue-3114-exact/fixture"
XDG_RUNTIME_DIR=/tmp/nm-3114-exact-runtime "$binary" check \
  --root "$root" --config "$root/r1.yml" --format json
```

Repeat for `r1`, `r2`, `r3`, `r4`, `off` and `off-sql`.

## Validation

Strict all-target/all-feature Clippy, the complete locked workspace/all-feature
suite and doc tests pass with `INSTA_UPDATE=no`. Exact snapshots compare jobs
1 and 2; standalone/prepared aggregate API parity covers all six configurations.
Nested-write snapshots intentionally attribute findings to changed targets
instead of auxiliary tables. Focused LLVM coverage ran early without filename
exclusions; CI validates current-head floors. Independent proof review found
no actionable ownership defects. Changed Rust sources remain within 200
physical lines and tests within 500. No inline fixture writes were added.

Tests use isolated temporary directories because an existing `/tmp/.git`
changes an unrelated ancestor-ignore fixture. Cargo artifacts remain under
`/tmp`; the main workspace target is never cleaned.

## Output-label follow-up

The rejected head was `62f2a4fac9973cd07490133f812d0fb234e7bd28`; the corrected
production source is `c502b32d916d90636f9402f617579a3ddd8e40d8`, rebased on
`ba93010d5aa2c3e7a597364f80ca770f956cb002`. This comparison includes that rebase.
Matching locked default-feature **dev** builds use rustc 1.99.1 and six Cargo
build jobs in separate task-owned targets. CLI effort is default with `--jobs 1`.
Instruction counters remain unavailable, so these are user-CPU observations.

[Raw samples](follow-up.csv) use fresh processes, warmed OS cache, and balanced
A/B/B/A order: ten samples per binary for each SQL workload, four for the
repository check. User CPU is 29.882→31.011 ms for the follow-up fixture,
27.747→25.269 ms for rule-off SQL, and 3.186011→3.242978 s for the repository.
The new fixture changes 11 grouped findings to six retained controls. R1–R4,
empty/off SQL controls and repository JSON remain byte-identical, including
stderr and exit status for the exact controls. No speedup or equivalence claim
is made. [Summary and binary hashes](follow-up-summary.json) retain provenance.

From the corrected checkout, reproduce with `python3
 docs/work-evidence/postgres-bounded-3114-exact/measure_follow_up.py --before
 <rejected-cli> --after <corrected-cli> --output <empty-output-directory>`.
Build the two executables in separate targets under `/tmp` with the shared
Cargo lock and remove those targets after measurement.

The reusable harness records optional `--before-revision` and `--after-revision`
values as unverified caller labels. It cannot infer build revisions from an
executable or the current checkout. The captured measurements above were tied
to separate builds during this run; their binary hashes are retained. Later
production changes require a separate comparison.

Run the harness regressions with `python3 -B
docs/work-evidence/postgres-bounded-3114-exact/test_measurements.py`. Saved sample
cases reject failed statuses, unexpected diagnostics and finding-count changes.
The launch checks execute a saved fixture through private directories and verify
the permission-preserving copy fallback across filesystems. Unsupported modes
fail before accessing binaries or starting measurements.
