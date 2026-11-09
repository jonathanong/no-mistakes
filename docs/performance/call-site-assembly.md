<!-- cspell:ignore heaptrack ABBA BAAB getrusage RUSAGE -->
# Call-site assembly memory

Issue #2635 targets retained memory after the worker-scaling fixes. This change
assembles call-site records into one exactly sized vector and transfers that
allocation into the canonical graph. Refinement retains complete immutable
originals only for its existing set of admitted sites, using dense IDs mapped back
to physical occurrences. Incoming arguments still include every caller and
constructor; visit limits use the admitted site count plus one, followed by
one conservative pass with parameter refinement disabled. Every record
and its order survive. Extraction, resolution and refinement results, selectors,
and suppressions are unchanged.

The saved [workload](../../test-cases/performance/call-site-assembly/README.md)
contains 274 TS/TSX sources, 7,932,529 bytes, 284,040 lines, and 68 TSX modules.
Thirteen forbidden-call policies produce one positive finding. Rule-off and
import-only configurations provide controls on the same physical sources.
This is a focused construction workload, not a full consumer configuration.

## Allocation evidence

The baseline is `bdcd31ae5a7d060cc5b4c7a795d0924882d57266`, built with
Rust 1.99.1, default features, fat LTO, and one code generation unit. Native runs and
heap captures omit observer, progress, and timing flags. The frozen baseline
ELF SHA-256 is `a484974579b71e8ecaf887fd15c9cb6035b0700f55c59332df753b0e057f2d56`.
Heaptrack timings are
not runtime comparisons.

At the exact first maximum sampled RSS record, the largest individual live
allocation stack is collection-vector growth: 192,614,400 requested bytes at
both jobs 1 and 24. The next stack is the full initial call-site clone,
101,862,720 bytes. The candidate constructs originals only for admitted
sites, avoiding the full clone when known calls need no refinement. This uses
complete existing records with direct array indexes; no compact resolution
representation or additional lookup map is introduced. The collection call's
machine code passes an element size of 360 bytes to the vector reserve helper.
The merged slab therefore has 535,040 slots for 282,952 records. Exact assembly
removes 90,751,680 bytes of spare capacity in this slab; requested allocation
bytes are not an RSS partition or an RSS-saving prediction.

| Profiled jobs | Sampled peak RSS bytes | Live requested bytes at that record |
|---|---:|---:|
| 1 | 1,296,633,856 | 1,235,078,323 |
| 24 | 1,370,451,968 | 1,242,119,975 clean; at most 1,242,336,431 including quarantined recorded payloads |

The jobs-24 raw trace has fifteen overlapping allocation events. Strict replay
rejects it. A complete-stream audit quarantines the full history of nine
addresses, including two startup unknown frees, then replays every other
address strictly. The global conservative recorded-payload bound is 216,456
bytes: each quarantined address's maximum logged multiplicity times its
maximum recorded payload, summed across addresses. This bound covers recorded
payloads, not unrecorded allocations or RSS. Jobs 1 has two startup unknown
frees and otherwise replays strictly. Both captures complete and retain the
native report's exact JSON line and exit status.

Exclusive requested-byte lineages at the sampled RSS maximum are:

| Lineage | Jobs 1 bytes | Jobs 24 clean bytes |
|---|---:|---:|
| Dependency extraction | 657,098,422 | 657,098,422 |
| Graph construction/refinement | 556,427,205 | 563,291,066 |
| Check facts | 12,734,723 | 12,737,747 |
| Source store | 8,079,128 | 8,079,128 |
| Other | 738,845 | 913,612 |

These are exclusive stack lineages, not object ownership or resident-memory
buckets. A jobs-24 category can additionally contain the whole 216,456-byte
uncertainty; category upper bounds overlap and must not be summed. Extraction
is the largest aggregate family. Collection is the largest individual stack,
with a concrete removable capacity measured above.

The [saved replay and quarantine tools](../../test-cases/performance/call-site-assembly/tools/README.md)
include regression traces for clean input, unknown frees, and a collision
occurring after the RSS peak. The latter ensures quarantine removes the
address's complete history and preserves selection of the first maximum RSS
record. Validate them with:

```sh
g++ -std=c++20 -O2 test-cases/performance/call-site-assembly/tools/quarantine.cpp -o /tmp/nm-heap-quarantine
python3 test-cases/performance/call-site-assembly/tools/check-audit.py /tmp/nm-heap-quarantine
```

## Native results

The measured baseline is `bdcd31ae5a7d060cc5b4c7a795d0924882d57266`
(version 0.109.29); the measured candidate is
`3f9266460b3dd934ff37fb39179e8e02ea7cbb3d` (version 0.109.29).
These tables retain the exact 0.109.29 comparison. The
[PR report](https://github.com/jonathanong/no-mistakes/pull/3172) records the final
matched comparison after rebases over later runtime changes. Both library and
executable artifacts were rebuilt (`fresh: false`)
from their own worktrees with the same lockfile and release settings. The
candidate ELF SHA-256 is
`d7b1eb0d5c00c893d845fe0911dafeee8a848391d2458b4f5c43bb6f62c1f966`.

These Linux measurements use the same physical fixture, toolchain and host
with 24 available CPUs. Jobs 1, 4, and 24 are pinned to the first 1, 4, or all
available CPUs. Heavy builds and tests are idle while measuring, serialized
with the same lock. Fresh processes provide fresh invocation state; no startup
or reporting time is subtracted. Observer, progress and timing flags are off.
One warmup per binary/cell precedes three balanced blocks, alternating ABBA
and BAAB order, giving six measured samples per binary/cell.

Values below are medians, baseline → candidate. RSS uses MiB (1,048,576 bytes).
The [raw samples and provenance](call-site-assembly-results.json) retain every
sample, including warmups, so ranges and ordering can be inspected.

| Configuration | Jobs | Wall seconds | User CPU seconds | System CPU seconds | Peak RSS MiB |
|---|---:|---:|---:|---:|---:|
| on | 1 | 2.755 → 2.580 | 2.170 → 2.090 | 0.585 → 0.475 | 1231.2 → 1079.4 |
| on | 4 | 1.990 → 1.815 | 2.605 → 2.490 | 0.590 → 0.505 | 1272.9 → 1102.4 |
| on | 24 | 1.855 → 1.690 | 4.855 → 4.695 | 0.750 → 0.675 | 1284.6 → 1119.5 |
| off | 1 | 0.010 → 0.005 | 0.000 → 0.000 | 0.000 → 0.000 | 22.3 → 22.3 |
| off | 4 | 0.010 → 0.010 | 0.000 → 0.000 | 0.000 → 0.000 | 22.3 → 22.4 |
| off | 24 | 0.010 → 0.010 | 0.000 → 0.000 | 0.000 → 0.005 | 22.3 → 22.3 |
| imports | 1 | 0.390 → 0.390 | 0.270 → 0.265 | 0.120 → 0.115 | 337.2 → 335.4 |
| imports | 4 | 0.190 → 0.185 | 0.290 → 0.280 | 0.125 → 0.125 | 339.1 → 337.6 |
| imports | 24 | 0.170 → 0.170 | 0.380 → 0.390 | 0.185 → 0.190 | 352.0 → 351.1 |

The consuming configuration reduces median peak RSS by 12.3%, 13.4%, and
12.9% at jobs 1, 4, and 24 respectively. Wall and user CPU medians also fall.
The rule-off path is at the timer's 0.01-second resolution; no startup speed
claim is made. Import-only results are a non-consuming control.

## Diagnostics-on control

This separate matrix adds `--timings --verbose-timings`. It uses one balanced
ABBA block after warmups: two measured samples per binary/cell, so these are
observations rather than a precise speed estimate.

| Configuration | Jobs | Wall seconds | User CPU seconds | System CPU seconds | Peak RSS MiB |
|---|---:|---:|---:|---:|---:|
| on | 1 | 2.790 → 2.585 | 2.190 → 2.095 | 0.590 → 0.490 | 1231.5 → 1079.6 |
| on | 4 | 2.025 → 1.830 | 2.635 → 2.600 | 0.615 → 0.480 | 1275.7 → 1095.6 |
| on | 24 | 1.855 → 1.685 | 4.780 → 4.620 | 0.750 → 0.670 | 1284.6 → 1117.6 |
| off | 1 | 0.005 → 0.010 | 0.000 → 0.000 | 0.000 → 0.000 | 22.4 → 22.3 |
| off | 4 | 0.010 → 0.010 | 0.000 → 0.000 | 0.000 → 0.000 | 22.3 → 22.3 |
| off | 24 | 0.010 → 0.010 | 0.000 → 0.000 | 0.010 → 0.000 | 22.3 → 22.3 |
| imports | 1 | 0.395 → 0.395 | 0.270 → 0.285 | 0.120 → 0.105 | 337.6 → 335.7 |
| imports | 4 | 0.190 → 0.190 | 0.275 → 0.290 | 0.140 → 0.130 | 339.6 → 337.9 |
| imports | 24 | 0.165 → 0.170 | 0.390 → 0.385 | 0.205 → 0.185 | 353.0 → 350.5 |

## Import-only reverse rechecks

Adjacent import-only rechecks use three balanced blocks after warmups: six
measured samples per binary/cell, separately with diagnostics off and on.
Every accepted matrix observes the same 288 physical fixture files and
7,973,056 total bytes, including the 7,932,529 analyzed source bytes.
Small user-CPU differences change direction across these blocks; no control
speedup is claimed.

| Diagnostics | Jobs | Wall seconds | User CPU seconds | System CPU seconds | Peak RSS MiB |
|---|---:|---:|---:|---:|---:|
| Off | 1 | 0.395 → 0.390 | 0.275 → 0.270 | 0.115 → 0.130 | 343.8 → 343.5 |
| Off | 4 | 0.190 → 0.190 | 0.290 → 0.280 | 0.125 → 0.120 | 342.2 → 342.3 |
| Off | 24 | 0.160 → 0.155 | 0.400 → 0.405 | 0.160 → 0.150 | 355.7 → 355.6 |
| On | 1 | 0.390 → 0.390 | 0.275 → 0.270 | 0.115 → 0.120 | 340.4 → 340.4 |
| On | 4 | 0.190 → 0.190 | 0.280 → 0.280 | 0.135 → 0.130 | 342.6 → 342.7 |
| On | 24 | 0.160 → 0.165 | 0.380 → 0.390 | 0.185 → 0.180 | 355.5 → 355.6 |

A longer observer-off jobs-24 recheck uses twelve balanced blocks, giving
24 measured samples per binary. Wall, user CPU and system CPU medians are
respectively 0.160 → 0.160, 0.380 → 0.380, and 0.180 → 0.180 seconds.
Median peak RSS is 355.6 → 355.2 MiB. Supplemental
`resource.getrusage(resource.RUSAGE_CHILDREN)` deltas include the measurement
wrappers and avoid GNU time's 0.01-second CPU rounding. Mean child user CPU is
0.387474 → 0.386907 seconds. The twelve paired block differences average
−0.000568 seconds, with an approximate 95% Student-t interval of
[−0.010769, 0.009633] seconds. This longer block does not reproduce a CPU
slowdown. Raw coarse and supplemental CPU fields remain in the linked data.

The saved fixture snapshot test supplies an explicit timeout of 120 seconds
for coverage instrumentation. The normal CLI deadline remains 30 seconds.

## Output and scope

Every native sample, warmup, diagnostics sample and worker setting retains
byte-identical complete JSON reports between baseline and candidate, including
warnings and advisories. The consuming configuration retains its one positive
finding and exit 1. Rule-off and import-only retain no findings and exit 0.
The on-report SHA-256 is
`ef0d9719de3b5ad7f2f82028c636e2980865dd3aad992b4900b0f0c94dec8c65`;
both controls retain
`0c81f3b0dafacbff4e9fc6e5c750671cb8fe7e77ad5cbcaac844f703b5e96059`.

The saved small semantic fixture and full owned-original oracle cover callback
inputs from known callers, constructors, duplicate same-line occurrences,
dynamic calls and gaps in admitted indices. Work-count regressions assert the
exact merged capacity and complete-record clone count. No rule admission,
resolution fallback, suppression or record field is removed. The admitted-site
visit bound is covered separately by the sparse callback-cycle regression.
This workload establishes a construction-heavy call-analysis gain;
it does not establish a speedup for every codebase or configuration.

## Admitted-site visit bound

Issue #3184 tightens both the refinement and requeue predicates to the
admitted count plus one. Only admitted occurrences can change during this
traversal; known callers still contribute incoming arguments without widening
its visit budget. One final visit disables parameter refinement when the
budget is exhausted, preserving the conservative unknown-call result.

The saved `value-flow-visit-cap` fixture has one 471-byte, 41-line TypeScript
source with no JSX, 32 unrelated direct calls, and a callback member that
oscillates between agreement and disagreement. It admits 2 of 36 physical
sites. The unchanged owned-original oracle performs 39 visits; the admitted
cap performs 5. The work-count test first fails on main and then passes with
both predicates tightened. It compares every public graph field, incoming
argument ownership, and physical edge order and multiplicity.

The `check_json_snapshot` golden was generated on main before this fix. At
jobs 1 and 2 it retains the unknown callback finding, the configured timer
finding, and the empty rule-off report. These are work-bound and compatibility
guards, independent of any measured runtime improvement.
