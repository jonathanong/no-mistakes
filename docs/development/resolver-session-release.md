# Resolver session production comparison

<!-- cspell:ignore codegen BAAB clippy -->

The local production comparison is complete. No speedup is claimed; configured
CI and review checks still gate readiness. The earlier rejected development
comparison remains in [the visibility notes](resolver-session-visibility.md).

The [136 repository observations](../../fixtures/performance/session-visible-resolver/release-repository-43cb.csv)
and [provenance](../../fixtures/performance/session-visible-resolver/release-repository-43cb-provenance.json)
compare baseline `9579d3184` with candidate `43cb40d96`. The latter contains
production source `fe8f1562e` plus documentation only. Both are v0.92.0,
default-feature release builds on one machine with Rust 1.99.1, LTO enabled,
one codegen unit, debug information disabled and incremental compilation off.
Each executable was independently verified and frozen before measuring.

All processes use the same physical `/tmp/nm-2117-worktree` inputs and saved
consuming/control configurations. No builds or tests run during measurement.
The repository contains 10,699 TS/JS files, 5,598,213 bytes and 158,020 lines;
3,076 JSX/TSX files contain 347,140 bytes and 9,424 lines. The Node package
selection contains 168 files, 884,597 bytes and 23,620 lines. The consuming
configuration includes queue, unique-export and Vitest integration selection
payloads. The control retains configured forbidden-dependency analysis.

The first matrix has 96 fresh processes: diagnostics off/on, jobs 1/4/automatic
maximum, both workloads, ABBA followed by BAAB. Each table entry is a median
of four observations per binary. CPU is user plus system time; RSS is raw
process peak RSS. Reports, exits and non-diagnostic warning hashes match in
every cell. Diagnostic stderr is retained separately; no clock is subtracted.

| Workload, diagnostics, jobs | Wall seconds A/B | CPU seconds A/B | Peak RSS KiB A/B |
| --- | --- | --- | --- |
| Consuming, off, 1 | 0.455 / 0.445 | 0.450 / 0.435 | 132256 / 130042 |
| Consuming, off, 4 | 0.385 / 0.390 | 0.550 / 0.565 | 133458 / 129968 |
| Consuming, off, maximum | 0.485 / 0.425 | 1.025 / 1.225 | 138196 / 134162 |
| Consuming, on, 1 | 0.505 / 0.510 | 0.455 / 0.495 | 133980 / 130774 |
| Consuming, on, 4 | 0.420 / 0.425 | 0.580 / 0.595 | 133930 / 131164 |
| Consuming, on, maximum | 0.430 / 0.425 | 1.165 / 1.205 | 138602 / 135574 |
| Control, off, 1 | 2.405 / 2.610 | 2.400 / 2.610 | 328420 / 321512 |
| Control, off, 4 | 1.910 / 1.900 | 2.625 / 2.600 | 326434 / 322046 |
| Control, off, maximum | 1.825 / 1.815 | 4.395 / 4.315 | 333704 / 328804 |
| Control, on, 1 | 2.340 / 2.350 | 2.340 / 2.345 | 329928 / 326562 |
| Control, on, 4 | 1.905 / 1.920 | 2.655 / 2.640 | 330298 / 326074 |
| Control, on, maximum | 1.845 / 1.835 | 4.260 / 4.150 | 336140 / 331632 |

Peak RSS is lower in these release observations, unlike the rejected development
experiment. Timing directions vary across the matched blocks. In particular,
the control without diagnostics at jobs 1 prompted the focused rechecks below.

An additional eight-process ABBA/BAAB recheck of that cell did not reproduce its
original median delta: wall A/B was 2.800/2.760 seconds, CPU 2.800/2.615 seconds,
and RSS 327332/322746 KiB. However, both binaries shifted from roughly 2.34
seconds in the first block to roughly 3.3 seconds in the reverse block; one
candidate process took 3.77 wall seconds versus 2.89 CPU seconds. Those unstable
samples do not establish a control pass. The host has different CPU classes
and these processes had unrestricted affinity. All observations are retained.

An additional 16 processes pin both binaries to the same fast CPU core (CPU 3),
with jobs 1 and diagnostics off/on, in balanced forward/reverse blocks. Physical
inputs are refrozen at `fda2253e7`; its benchmark, test and documentation changes
leave the measured analyzer and dependencies unchanged. All 23,922 tracked input
hashes remain unchanged throughout this comparison, and report/exit/warning
hashes match in both cells.

| Pinned diagnostics | Wall seconds A/B | CPU seconds A/B | Peak RSS KiB A/B |
| --- | --- | --- | --- |
| Off | 2.205 / 2.200 | 2.190 / 2.195 | 323474 / 322468 |
| On | 2.215 / 2.230 | 2.210 / 2.220 | 331734 / 326604 |

The original large diagnostics-off control gap does not reproduce. Its forward
and reverse wall deltas are +0.005 and -0.010 seconds. Diagnostics-on deltas are
+0.030 and -0.005 seconds: the small cost changes direction in the reverse block.
These four observations per binary per cell do not establish a workload gain.
Peak RSS remains lower. The original unrestricted-affinity data remains above.

A separate 16-process recheck pins the consuming configuration to CPU 0–7,
the host's fast cores, with jobs 4 and diagnostics off/on. The original unrestricted
diagnostics-off forward/reverse blocks had positive wall deltas; one candidate
sample took 0.50 wall seconds versus the other samples' 0.37–0.39 seconds.

| Pinned consuming diagnostics | Wall seconds A/B | CPU seconds A/B | Peak RSS KiB A/B |
| --- | --- | --- | --- |
| Off | 0.330 / 0.325 | 0.475 / 0.470 | 133484 / 130524 |
| On | 0.325 / 0.320 | 0.460 / 0.455 | 134158 / 130732 |

Forward/reverse wall deltas are -0.005/0.000 seconds without diagnostics and
+0.005/-0.010 seconds with diagnostics. These focused rechecks do not reproduce
a consistently slower direction. The original matrix retains mixed costs,
including higher candidate CPU time for consuming automatic-maximum jobs.
Memory observations and work-count reductions do not establish a runtime gain.

## Smaller saved fixtures

The [144 fixture observations](../../fixtures/performance/session-visible-resolver/release-fixtures-43cb.csv)
and [fixture provenance](../../fixtures/performance/session-visible-resolver/release-fixtures-43cb-provenance.json)
cover `core-analysis`, `graph-gates`, and `runner-shared-imports`, diagnostics
off/on and jobs 1/4/automatic maximum, with four observations per binary per cell.
All 18 cells preserve report/exit/warning hashes. Inputs remain unchanged.
Core analysis has 14 TS sources, 1,694 bytes and 63 lines, including two TSX
sources (315 bytes, nine lines). Graph gates has 77 TS/JS sources, 18,316 bytes
and 389 lines, including 20 JSX/TSX sources (6,350 bytes, 144 lines), with saved
queue, route, selector and test-include configuration payloads. The runner has
four module sources, 397 bytes and eight lines; its two configurations share
imports. Diagnostics confirm actual runner preparation: one normalization,
four canonicalization attempts, three resolver computations and five cache hits.

Fixture wall times are mostly 0.00–0.02 seconds; the timer's 0.01-second precision
cannot establish workload speed. All cells have lower median candidate peak RSS
(roughly 2–3.5 MiB). The raw CSV retains CPU time and diagnostic RSS too. Configured
ARM CI comparisons require the expected base and runner before interpretation.

## Benchmark verification gap

The first [memory-query CI failure](https://github.com/jonathanong/no-mistakes/actions/runs/39090131971/job/117566867392)
at head `43cb40d9621e2a691726de90cad0a36032b58021` exposed an obsolete expected
resolver count: the saved four-report request computes ten distinct keys, while
the benchmark still expected twelve. Equivalent graph and legacy-symbol scopes
now share the two formerly duplicated keys. The exact-count assertion remains.

The earlier full `--lib --bins --tests` run and all-target `cargo clippy` did not
execute Criterion registration assertions. The correction adds a regular
library regression for the same saved request, exact counts, graph/index reuse,
and known dependency results. That test passes. The actual query-shard Criterion
preflight also passes using `cargo bench --profile dev -p no-mistakes --bench
core_analysis --no-default-features --features test-instrumentation -- --test
aggregate/reused_multi_report`. This is validation, not a timed release estimate.
These benchmark/test/documentation changes leave production analyzer source and
third-party dependency versions unchanged from the measured `43cb40d96`.
