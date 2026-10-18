# Resolver session production comparison

<!-- cspell:ignore codegen BAAB clippy -->

Performance validation remains pending. No speedup is claimed, and the PR stays
draft until the control timing gate is resolved. The earlier rejected development
comparison remains in [the visibility notes](resolver-session-visibility.md).

The [raw observations](../../fixtures/performance/session-visible-resolver/release-repository-43cb.csv)
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
the control without diagnostics at jobs 1 requires further investigation.

An additional eight-process ABBA/BAAB recheck of that cell did not reproduce its
original median delta: wall A/B was 2.800/2.760 seconds, CPU 2.800/2.615 seconds,
and RSS 327332/322746 KiB. However, both binaries shifted from roughly 2.34
seconds in the first block to roughly 3.3 seconds in the reverse block; one
candidate process took 3.77 wall seconds versus 2.89 CPU seconds. Those unstable
samples do not establish a control pass. The host has different CPU classes
and these processes had unrestricted affinity. All observations are retained.

Before readiness, resolve that control gate with a separately labeled matched
comparison, then cover the smaller saved consuming/control fixtures. Configured
ARM release CI comparisons also require the expected base and runner. Work-count
reductions and lower RSS alone do not establish a runtime improvement.

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
