<!-- cspell:ignore Mimalloc mimalloc codegen Vouchington dlopen -->

# Mimalloc allocator evaluation

The CLI and N-API library use mimalloc through Rust's global allocator when the
default `mimalloc` feature is enabled. That dependency enables
`local_dynamic_tls`, so a Unix build of the addon does not use initial-exec TLS
inside a library Node loads with `dlopen`. The `override` feature is not
enabled, so the C allocator used by Node remains unchanged. Default features
build mimalloc v3; the `v2` feature is not enabled. Rust-library consumers that
provide their own global allocator can opt out with
`--no-default-features --features mermaid-validation`.

## Pre-registered adoption rule

This rule was written before any eligible paired sample existed. It is not a
result. No hosted cell and no many-core cell has been measured on this head.
`scripts/performance/mimalloc-report.py` records the same thresholds in
`ADOPTION_RULE` and does not apply them.

Adopt mimalloc only if every measured cell meets all of the following:

- the median paired wall-time ratio (mimalloc / system) is at most 1.00, and no
  pair's wall-time ratio is above 1.05
- the median paired peak-RSS ratio is at most 1.00 at jobs 1
- the median paired peak-RSS ratio is at most 0.95 at max jobs on the
  many-core host (≥16 logical CPUs)
- report, warning, and exit-status parity is 100%

These thresholds were not tuned to a measurement. Nothing below is a result.

## Unmeasured

No eligible paired samples exist. Do not read this page as a timing, RSS, or
size result.

The hosted matrix, once a dispatch produces samples, is Linux arm64 and macOS
arm64, three workloads, diagnostics off and on, and jobs 1, 4, and max: 36
cells, four pairs each. Each cell should report the median and max paired
ratios for wall time, user+sys CPU, and peak RSS, the valid-pair count, and
report parity. Those cells are not filled in.

The high-worker matrix is separate and also unmeasured. It needs one many-core
Linux host and one many-core Apple Silicon Mac, each with at least 16 logical
CPUs, using the release-equivalent artifacts from the fixed build. Jobs are 1,
4, 16, and max (`--jobs` omitted). Diagnostics are off and on. Max-jobs rows
must show `automatic_rayon_jobs` of at least 16. Hosted `ubuntu-22.04-arm` has
4 vCPUs, so a hosted `jobs=max` cell is not that measurement. The measure
script accepts `--jobs 16` for this run; that choice is not part of the hosted
36-cell matrix.

N-API wall time and peak RSS, and build-size deltas for all four release
targets, are unmeasured.

## Harness

The comparison uses the same source revision and lockfile for both variants.
The system-allocator control disables default features and explicitly retains
`mermaid-validation`; the candidate uses the default features. Only the
allocator feature differs. Linux arm64 measured artifacts use release fat LTO
and one codegen unit. macOS arm64 measured artifacts use thin LTO and 16
codegen units, matching release builds. The measured cargo invocation builds
the library and `no-mistakes` bin only, on `ubuntu-22.04-arm` for Linux, and
does not set `CARGO_BUILD_JOBS`. It does not build the allocator example, so
dev-dependency features stay out of the measured CLI and addon.

The registration probe is a second invocation with LTO off and 16 codegen
units. The standalone probe jobs use that same non-LTO profile. `measure.py`
runs the probe before timing the CLI. The probe calls `mi_is_in_heap_region`
only while its `Vec` remains live. A dependency-only control without the
registration must fail.

The native evaluation workflow is manual `workflow_dispatch` only. It builds
both the CLI and N-API addon, checks the probe with the feature off, without
global-allocator registration, and with registration, then runs N-API parity
tests for both variants. It measures the pinned Vouchington consumer and both
public hosted-check fixtures with `--jobs 1`, `4`, and `max`. Four same-runner
matched pairs per cell use AB, BA, BA, AB order. Each process saves its
executable hash, platform and CPU facts, exit status, normalized report and
warnings, wall time, user/system CPU time, per-process peak RSS, and any
`MIMALLOC_*` environment. A non-empty or mismatched `MIMALLOC_*` environment
fails the pair. The summary reports paired differences and ratios, maxima,
nearest-rank p95, and eligible resource counts. Four samples cannot establish
a guaranteed tail bound.

Prepared workload artifacts retain the pinned checkout's Git index and native
core inventory settings, so tracked-but-ignored files remain visible to the
analyzer. Remote and credential configuration is removed before upload.

Every hosted-check tree is expected to exit zero, as required by the pinned
hosted-check contract. A nonzero exit, timeout, signal, missing pair, parity
mismatch, or incomplete report fails the evaluation. Censored or signaled runs
remain in the raw artifacts but do not contribute CPU or RSS maxima. Evaluation
builds use an 80-minute job and a 45-minute compile step, matching the
non-Windows `release.yml` caps. Release-target checks use 100/75 minutes on
Windows and 80/45 otherwise. Measurement jobs keep a 30-minute deadline and run
both variants on the same native runner before uploading that pair's evidence.
