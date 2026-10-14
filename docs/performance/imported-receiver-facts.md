# Imported receiver fact-control investigation

<!-- cspell:ignore ABBA BAAB codegen rlib wait4 Callgrind LibVEX ENOENT PMU debuginfo stationarity Userland callgraphs -->

This transformed public evidence copy references [#1972](https://github.com/jonathanong/no-mistakes/issues/1972).
The issue remains open: the historical unoptimized slowdown is unexplained,
and optimized native CPU impact is not established reliably on this host.
No runtime source, semantic assertion, or sharing bound changes in this PR.
Instruction-work profiles supplement the timing evidence; they do not establish
CPU latency or a production throughput improvement.

## Scope and artifacts

| Comparison | Exact baseline | Exact candidate |
| --- | --- | --- |
| Historical receiver change | `57c927df4c09f450f67248277d49070f9576ba3b` | `8d81aa148eb2e62773919a290dd21439a1616f42` |
| #1961 merged integration | `b97b2cb3c4dc1b3040a9696e60839b8834f4e955` | `917c9ea1c2f2400e8a5d3d50c18c0a52fe21ad4a` |

The integration comparison includes #1961 as merged, before later follow-up
implementations. These are not measurements of a moving current main.
The originally measured historical checkout `7799daf908022ad3d93af09479606e74c25bd00c`
has the same complete tree as published `8d81aa1`:
`0a768383587495ce34a587073758dbee0e747604`.

All four optimized artifacts use Rust 1.99.0 (`b940084d7`, LLVM 23.1.1), crate
0.90.0, default features including `mermaid-validation`, and byte-identical
Cargo.lock (`bb9bf453fde21cda4e892358e38325def3b39046309538eacf5cfaac75e36d6c`).
They use the unchanged release profile: optimization level 3, full LTO, one
codegen unit, debug information off, debug assertions and overflow checks off.
The probe links also use optimization level 3, full LTO and one codegen unit.

Cargo's compiler-artifact records must report `fresh: false` and the correct
worktree source path. Libraries were frozen before reuse of the dependency
target. All four frozen library and executable hashes differ; ELF compiler
records independently show Rust 1.99.0. See the four `*-manifest.json` files,
`*-elf-compiler.txt`, and build records in the evidence directory.
An initial metadata command queried Rust from the wrong working directory and
reported its default 1.96.0. This recording error was corrected before any
timing; both executables and Cargo's compiler cache independently verify 1.99.0.
The initial record and its correction remain available for audit.

The host is x86_64 Linux on an Intel Xeon Platinum 8573C, with CPUs 0–4 allowed
and a four-CPU cgroup quota. The primary runs retain four Rayon workers and the
full allowed affinity. The separately labeled serial diagnostic uses one Rayon
worker and changes only child affinity to CPU 0. No host/governor settings change.

## Preserved historical unoptimized result

The original artifacts use the development profile with optimization level 0
and debug information off. Eight samples per binary/demand/order used four
Rayon workers and 100 fresh collections per sample.

| Demand | ABBA CPU seconds/100, baseline → candidate | Change | BAAB CPU seconds/100, baseline → candidate | Change |
| --- | ---: | ---: | ---: | ---: |
| Values | 1.525965 → 1.608655 | +5.42% | 1.492848 → 1.609067 | +7.79% |
| Imports only | 1.524874 → 1.596553 | +4.70% | 1.399272 → 1.487117 | +6.28% |

The reverse-order increases are about 1.16 ms and 0.88 ms per collection.
The cause remains unresolved. Optimized results do not erase this historical
observation. All 13 original raw/summary/harness/provenance files remain byte-for-byte
unchanged locally. The public [historical evidence manifest](imported-receiver-facts-evidence/historical-unoptimized/evidence-manifest.json)
verifies the shipped path-normalized copies and separately records original
byte counts and hashes. Public copies are not described as original bytes.

## Workload and timing method

The no-writes input is the same physical
`test-cases/codebase-analysis/import-facts/fixture/shared-destructuring.mts`:
2,454 bytes, three lines, 128 destructured bindings. Other controls are member
writes (24,398 bytes, 774 lines), shared histories (59,584 bytes, 1,538 lines),
and JSX (9,660 bytes, 136 lines). All fixture bytes agree across all four
source worktrees; [fixtures.json](imported-receiver-facts-evidence/fixtures.json)
records their hashes. These are focused fact controls, without configured
include/filter payloads or a tsconfig; they do not measure full CLI throughput.

The original probe calls `collect_ts_facts` with imports and function calls
requested, and value flow either on or off. Each collection creates fresh
analysis state. Each sample is a fresh child process. Four ABBA blocks and four
BAAB blocks give eight samples per binary/demand/order. CPU is child user plus
system time from `wait4`; wall time runs from process launch through exit;
RSS is child peak RSS. Startup remains included. Tables show medians, not
Criterion estimates; no Criterion significance or CodSpeed claim is made.

All local builds, tests, analysis CLI scans and other probes were idle during
native sampling. Root coordination additionally restricted other agents to
API-only activity during the early historical 1,000-collection run and all
remaining longer diagnostics. No compilers were merely paused. The pilot,
both orders, and every larger-batch result remain separate. The no-writes
100-collection batches take hundreds of milliseconds. Short shared-history
imports-only batches motivated the separately labeled 100-collection control.

## Optimized native results

The original-count results change direction between orders. The longer
historical results remain variable, including after the serial/affinity
diagnostic. A stable optimized slowdown, improvement, or zero-overhead claim
is not established. CPU/wall ratios around 0.999 do not support substantial
additional parallel-spin CPU as an explanation; they do not establish a cause.

### Original no-writes batch

CPU and wall units are seconds/100 collections; peak RSS is MiB. Four Rayon workers.

| Comparison | Demand | Order | CPU baseline → candidate | CPU change | Wall baseline → candidate | RSS baseline → candidate |
| --- | --- | --- | ---: | ---: | ---: | ---: |
| Historical | values | ABBA | 0.407894 → 0.420313 | +3.04% | 0.409854 → 0.421339 | 15.082 → 15.178 |
| Historical | imports | ABBA | 0.595938 → 0.470583 | -21.03% | 0.598369 → 0.469026 | 14.891 → 15.045 |
| Historical | values | BAAB | 0.373057 → 0.360603 | -3.34% | 0.373428 → 0.361862 | 15.041 → 15.211 |
| Historical | imports | BAAB | 0.535957 → 0.533416 | -0.47% | 0.535611 → 0.537404 | 14.934 → 15.029 |
| #1961 integration | values | ABBA | 0.607654 → 0.595657 | -1.97% | 0.609744 → 0.602555 | 15.559 → 15.082 |
| #1961 integration | imports | ABBA | 0.400445 → 0.450681 | +12.55% | 0.402114 → 0.454163 | 15.434 → 14.926 |
| #1961 integration | values | BAAB | 0.435925 → 0.449419 | +3.10% | 0.436503 → 0.451532 | 15.623 → 15.082 |
| #1961 integration | imports | BAAB | 0.634467 → 0.462839 | -27.05% | 0.634255 → 0.464896 | 15.369 → 14.969 |

### Larger no-writes batch

CPU and wall units are seconds/1000 collections; peak RSS is MiB. Four Rayon workers.

| Comparison | Demand | Order | CPU baseline → candidate | CPU change | Wall baseline → candidate | RSS baseline → candidate |
| --- | --- | --- | ---: | ---: | ---: | ---: |
| Historical | values | ABBA | 4.022773 → 4.090325 | +1.68% | 4.033119 → 4.096450 | 15.020 → 15.102 |
| Historical | imports | ABBA | 3.617251 → 4.263621 | +17.87% | 3.618795 → 4.265512 | 14.992 → 14.955 |
| Historical | values | BAAB | 3.238807 → 3.278992 | +1.24% | 3.240104 → 3.281260 | 14.998 → 15.176 |
| Historical | imports | BAAB | 3.079624 → 3.048297 | -1.02% | 3.080555 → 3.050536 | 14.914 → 15.135 |
| #1961 integration | values | ABBA | 3.296120 → 3.260647 | -1.08% | 3.299289 → 3.266021 | 15.643 → 15.084 |
| #1961 integration | imports | ABBA | 3.231903 → 3.200667 | -0.97% | 3.235548 → 3.204479 | 15.463 → 14.912 |
| #1961 integration | values | BAAB | 3.301870 → 3.274519 | -0.83% | 3.303157 → 3.276602 | 15.686 → 15.203 |
| #1961 integration | imports | BAAB | 3.162427 → 3.215204 | +1.67% | 3.164504 → 3.219274 | 15.383 → 14.756 |

### Serial no-writes diagnostic

CPU and wall units are seconds/1000 collections; peak RSS is MiB. One Rayon worker, child CPU 0.

| Comparison | Demand | Order | CPU baseline → candidate | CPU change | Wall baseline → candidate | RSS baseline → candidate |
| --- | --- | --- | ---: | ---: | ---: | ---: |
| Historical | values | ABBA | 4.260374 → 3.974066 | -6.72% | 4.266643 → 3.981795 | 15.207 → 15.342 |
| Historical | imports | ABBA | 3.612108 → 3.782513 | +4.72% | 3.615726 → 3.786383 | 14.924 → 15.045 |
| Historical | values | BAAB | 3.825124 → 3.843240 | +0.47% | 3.829240 → 3.847719 | 15.143 → 15.404 |
| Historical | imports | BAAB | 4.060642 → 3.797712 | -6.48% | 4.065183 → 3.800414 | 14.961 → 15.193 |
| #1961 integration | values | ABBA | 3.112688 → 3.099608 | -0.42% | 3.116083 → 3.106434 | 15.527 → 15.191 |
| #1961 integration | imports | ABBA | 3.011559 → 3.028001 | +0.55% | 3.014132 → 3.030945 | 15.457 → 14.879 |
| #1961 integration | values | BAAB | 3.102102 → 3.088493 | -0.44% | 3.105470 → 3.091033 | 15.674 → 15.191 |
| #1961 integration | imports | BAAB | 3.008077 → 2.977845 | -1.01% | 3.011129 → 2.983233 | 15.539 → 15.004 |

### Other controls

CPU changes below describe the original batch sizes: writes/JSX 100 collections,
shared histories one collection. Full absolute wall/CPU/RSS values are retained
in `optimized-summary.json` and `current-optimized-summary.json`.

| Control | Demand | Historical ABBA | Historical BAAB | Integration ABBA | Integration BAAB |
| --- | --- | ---: | ---: | ---: | ---: |
| writes | values | -3.60% | -3.37% | +23.20% | -23.45% |
| writes | imports | -15.82% | -26.34% | +4.55% | -4.11% |
| shared_histories | values | -0.15% | -14.51% | -14.72% | -26.61% |
| shared_histories | imports | -10.08% | -10.66% | +8.19% | +5.21% |
| jsx | values | -41.89% | -9.53% | -9.01% | -30.21% |
| jsx | imports | -48.89% | +13.78% | +8.24% | +14.72% |

The separate shared-history 100-collection diagnostic retains startup and is
not pooled with the original one-collection result:

| Comparison | Demand | Order | CPU seconds/100 baseline → candidate | Change | Peak RSS MiB baseline → candidate |
| --- | --- | --- | ---: | ---: | ---: |
| Historical | values | ABBA | 9.584661 → 9.715193 | +1.36% | 14.799 → 14.453 |
| Historical | imports | ABBA | 0.352021 → 0.354784 | +0.78% | 13.250 → 13.301 |
| Historical | values | BAAB | 12.423783 → 12.470572 | +0.38% | 14.795 → 14.594 |
| Historical | imports | BAAB | 0.480422 → 0.573620 | +19.40% | 13.352 → 13.352 |
| #1961 integration | values | ABBA | 15.518965 → 14.838513 | -4.38% | 15.021 → 14.561 |
| #1961 integration | imports | ABBA | 0.357722 → 0.366766 | +2.53% | 13.299 → 13.299 |
| #1961 integration | values | BAAB | 9.604648 → 9.377703 | -2.36% | 14.840 → 14.502 |
| #1961 integration | imports | BAAB | 0.348688 → 0.348097 | -0.17% | 13.348 → 13.348 |

### Paired spread

Each paired block compares the mean CPU of its two candidate children with
its two baseline children. The table uses the mean log ratio across eight
balanced blocks and a 10,000-draw block bootstrap (fixed seed 1972).
Intervals describe observed spread; they are not a guarantee of stationarity
or production performance. The four per-order block changes are retained in
[paired-uncertainty.json](imported-receiver-facts-evidence/paired-uncertainty.json).

| Comparison | Mode | Demand | Paired CPU change | Descriptive 95% bootstrap interval |
| --- | --- | --- | ---: | ---: |
| #1961 integration | 4 workers | values | -0.84% | [-2.45%, +0.62%] |
| #1961 integration | 4 workers | imports | +0.13% | [-2.82%, +3.27%] |
| #1961 integration | 1 worker, CPU 0 | values | -0.53% | [-1.89%, +0.70%] |
| #1961 integration | 1 worker, CPU 0 | imports | +0.16% | [-1.11%, +1.56%] |
| Historical | 4 workers | values | +1.44% | [-3.77%, +6.69%] |
| Historical | 4 workers | imports | +4.42% | [-1.38%, +11.87%] |
| Historical | 1 worker, CPU 0 | values | -2.42% | [-14.41%, +11.74%] |
| Historical | 1 worker, CPU 0 | imports | +0.47% | [-10.64%, +12.30%] |

## Instruction-work profiling

One `instructions:u` counter attempt on our own child returned `ENOENT`; the
benchmark did not execute during that attempt. Read-only inspection found no
CPU PMU event source (only breakpoint, msr and software). No kernel settings
changed. The exact attempt and error are retained in `counter-capability.json`.

Valgrind 3.24.0 and matching libc debug symbols were downloaded from Debian's
official package index, hash/size verified, and extracted only in the workspace
(131 MiB). Tool/package provenance is in `profiler-provenance.json`.
Callgrind records whole-process guest instruction references (`Ir`), including
startup. This is neither native retired instructions nor native CPU latency.
Its instrumented wall/CPU/RSS fields are not compared as production performance;
instrumented wall also retains profile-processing overhead.

Each profile uses ten fresh collections, one Rayon worker, child CPU 0, and
the unchanged frozen executable. ABBA gives two repetitions per binary/demand.
All optimized repetitions below produce exactly the same instruction count.

| Comparison | Demand | Baseline guest Ir | Candidate guest Ir | Difference | Change |
| --- | --- | ---: | ---: | ---: | ---: |
| Historical optimized | values | 301,112,557 | 301,107,274 | -5,283 | -0.001754% |
| Historical optimized | imports | 296,289,671 | 296,284,330 | -5,341 | -0.001803% |
| #1961 integration optimized | values | 301,064,037 | 301,064,996 | +959 | +0.000319% |
| #1961 integration optimized | imports | 296,350,924 | 296,351,378 | +454 | +0.000153% |
| Historical original unoptimized | values | 1,101,882,915 | 1,101,892,481 | +9,566 | +0.000868% |
| Historical original unoptimized | imports | 1,082,957,862 | 1,082,967,130 | +9,268 | +0.000856% |

The original unoptimized profiles use the protected original executables,
verified against their historical binary hashes before profiling. Their eight
ABBA profiles use the same ten collections, worker count, and child affinity.
All repetitions also match exactly. These instrumented work counts preserve a
second view of the original artifacts without explaining their native slowdown.

The dominant `contains_key`, `callee_binding_scope`, `callee_shadows_import`,
and `push_value_symbol_reference` instruction totals match in the historical
optimized profiles. Nearly equal instruction work does not establish equal
latency, and does not explain the historical unoptimized CPU cost.

## Source audit, limits, and next step

The no-writes fixture has neither imported receivers nor member mutations.
Mutation collection requires member writes and value-flow demand;
`finish_default_mutations` returns when its lazy index is absent. Imports-only
requests do not collect value-flow bindings. No added receiver-history loop or
allocation was found on these paths. The optional shared receiver-history field
changes `ValueFlowFacts` layout. Layout/code-generation effects remain a
hypothesis, not an established cause.

No runtime optimization is justified by a confirmed attributable material
optimized regression in this evidence. This is a preservation/investigation
PR, not full resolution of #1972. The next step is stable-host native timing
and available hardware cycle/cache counters for attribution, retaining both
historical and merged-integration scopes. Do not carry these timings onto later
runtime changes or describe the original unoptimized cost as resolved.

## Evidence and reproduction

The evidence directory retains all original raw timing rows, every pilot and
order, larger-batch and serial results, build/provenance summaries, and scripts. Internal absolute paths are replaced by explicit placeholders.
Detailed Cargo artifact/build logs and environment/cache metadata are withheld
locally. `public-copy-mapping.json` records original/public hashes and omissions.
All 25 full userland profiles, including normalized compressed copies, are
withheld locally. Readable annotated summaries and numeric work records remain
in this text-only public package; full profiles are not durably included.
See `PUBLIC-COPY.md` for the policy and `public-copy-mapping.json` for original
and withheld normalized profile hashes.
No libraries, executables, or downloaded packages are committed.

The scripts use explicit `<WORKSPACE>` and system-root placeholders. Replace
them with local paths and recreate the isolated worktree layout. Build each exact revision
with the same compiler/dependencies and `cargo build --release --locked -p
no-mistakes --lib --message-format=json`, then freeze and verify the artifact
before another worktree reuses the dependency target. `freeze-artifact.py` and
`link-probe.sh` contain the checks and exact probe flags. Default `measure.py`
uses the original four-worker batch sizes; `--case`, `--iterations`, `--threads`,
`--cpu` and a distinct `--tag` reproduce the labeled diagnostics. Existing
evidence is never overwritten. A different stable toolchain requires matched
rebuilds of both sides and separately labeled results.

Validation: all four release libraries/probes build; every benchmark child exits
zero with matching fact signatures and no parse/operational errors. The configured
repository check, focused spelling, Python/shell syntax and whitespace checks pass.
Focused Cargo planning selects no tests/fallback for the standalone probe.
Impact planning reports the existing unresolved Vitest setup file, and the
symbol-impact query is blocked by the existing invalid UTF-8 fixture; neither
is treated as a completed symbol analysis.
