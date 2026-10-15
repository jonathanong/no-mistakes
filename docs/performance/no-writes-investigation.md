# No-writes fact CPU investigation: bounded conclusion

<!-- cspell:ignore ABBA BAAB codegen wait4 rlib PMU Callgrind preflight cbc opt0 opt3 Clippy stationarity predeclared -->

[#1972](https://github.com/jonathanong/no-mistakes/issues/1972) is complete as a
bounded investigation, with an **inconclusive optimized historical result**.
The original repeatable unoptimized slowdown remains unexplained. No runtime
fix or zero-overhead claim follows from this conclusion. The later attempted
optimizations below were rejected; this change publishes evidence only.

## Original question and retained outcome

The original pair is `57c927df4c09f450f67248277d49070f9576ba3b` →
`8d81aa148eb2e62773919a290dd21439a1616f42` (built checkout
`7799daf908022ad3d93af09479606e74c25bd00c`, identical candidate tree
`0a768383587495ce34a587073758dbee0e747604`). The same physical 128-binding,
no-member-writes fixture, Rust 1.99.0, dependencies and dev profile produced:

| Demand | ABBA CPU change | BAAB CPU change |
| --- | ---: | ---: |
| Value flow on | +5.42% | +7.79% |
| Value flow off | +4.70% | +6.28% |

Both demands request imports **and function calls**. “Imports” labels in the
saved data mean value flow off; they do not mean `function_calls: false`.
These positive observations are retained, not dismissed as host noise or
attributed to architecture, code layout, or the added optional field without
native causal evidence.

[The original investigation](imported-receiver-facts.md) preserves matching
optimized builds, balanced larger batches, serial diagnostics, original opt0
artifacts, and instruction-work profiles. Optimized historical values changed
from +3.04% in ABBA to −3.34% in BAAB at the original batch size. Guest instruction
counts differed by less than 0.002%; that neither explains native latency nor
proves flat cost. Hardware counters were unavailable.

[The hosted follow-up](receiver-native-hosted-evidence-2012/README.md), merged in
[#2027](https://github.com/jonathanong/no-mistakes/pull/2027), retains all 288 rows.
Its optimized pilot values were +0.844%, while the longer values series was
−1.219%; optimized imports and both longer opt0 demands were inconclusive.
The direction change prevents a stable improvement, zero-overhead, or resolved
original-regression claim. These historical optimized results do not establish
a regression to attribute and remove. No additional historical timing was run
merely to select a more favorable conclusion.

## Later optimizations were separate experiments

A current-source audit found duplicate lexical-scope lookups per value-symbol
reference. The no-writes fixture generates 16,512 references, so this was a
plausible independent optimization, not an established cause of the historical
change. All experiments kept function-call demand enabled, fresh per-invocation
analysis, and physical fixtures. No persistent cache or extra AST pass was added.
No production change from these experiments is included here.

| Attempt | Exact source pair | Result and disposition |
| --- | --- | --- |
| Empty-import guard | `57e65dc5c276dd4db3adb7cecd52881a1ffcfb16` → `806968113ea5a3414b8baddff9a485777f86ad98` (checkout `91f329e0c434489c407cee77ff912d6b559c0eac`, benchmark-only child) | Clean repeat: no-writes values −1.938%, value-flow-off −8.435%; writes values +0.569%, both orders positive, interval crosses zero. Refined, not landed. |
| Inline lexical lookup | same `57e65dc` → `496ee2d59ec25d3a58fecdf75f29a98c0e9d2615` | No-writes values −2.998%; shared-history values **+1.812% [ +1.347%, +2.320% ]**, positive in both orders. Rejected. |
| Reuse existing lookup helper | `8db445d14ddc61b836e9315e69f18681e9555f50` → `cbc39fc9c2da693051624147aeb2e6b9b287693e` | No-writes values −2.171%; shared-history values **+0.697% [ −0.205%, +1.480% ]**, ABBA +0.549%, BAAB +0.845%. Rejected; no favorable remeasurement. |

The last interval crosses zero and does not prove a slowdown. Nevertheless,
this attempt does not pass the slower-control gate: both order estimates are
adverse and the related prior attempt already reproduced a slower shared-history
control. A focused win does not excuse that unresolved risk. The rejected patch
is retained only as [evidence](no-writes-investigation-evidence/rejected-helper-reuse.patch.json).

All eight full fact outputs matched in each earlier pair. Final B/C matched all
13 fact outputs, full graph reports and effects reports before timing. Counts
alone were not accepted as semantic proof. The final candidate also passed 307
extraction tests, 617 graph tests, all-feature/all-target workspace Clippy, and
focused coverage of all 36 regions of the changed reference helper; these checks
do not override the performance decision.

## Frozen three-arm experiment

[The final evidence](no-writes-investigation-evidence/final-three-arm/) also
contains a separately requested correctness-cost comparison. It is not the
original #1972 pair and must not be used to claim that issue's cause was found.

| Arm | Exact measured revision | Scope |
| --- | --- | --- |
| A | `1cb57a920418ca38d13c72c8e09add163b487b3e` | Actual main at source freeze |
| B | `8db445d14ddc61b836e9315e69f18681e9555f50` | #2019 mutable-root correctness candidate |
| C | `cbc39fc9c2da693051624147aeb2e6b9b287693e` | Rejected lookup optimization on B |

A/B measures that exact correctness implementation's costs. B/C measures only
the attempted optimization, reusing the same B artifact with fresh observations.
Neither comparison is a measurement of a moving branch or later fixes.

All arms use Rust 1.99.0, default features including `mermaid-validation`, the
same Cargo.lock, release optimization level 3, full LTO, one codegen unit,
debug information/assertions/overflow checks off, incremental compilation off,
and two build jobs. Wrapper settings were explicitly empty to disable the host's
configured wrapper. Cargo artifact records showed fresh builds and the expected
source path/profile/features; distinct executable/library SHA-256 hashes were
verified before sampling. Each fresh child uses four Rayon workers and CPU
affinity 0–3 on the same x86_64 Linux host. This is local matched evidence,
not an ARM/CodSpeed comparison or hardware-counter attribution.

The four original inputs range from 2,454 bytes/3 lines (128 bindings) to
59,584 bytes/1,538 lines (shared histories); JSX is 9,660 bytes/136 lines.
The broader input inventory contains 211 saved files, 228,051 bytes and 5,727
lines, including the real configured provenance fixture. Hashes and source sizes
are in the input manifest. No files were generated inside tests.

Before sampling, independent review found that A/B graph physical identities,
nodes, edges, copies and siblings were stable: only 48 call sites became unknown
within intended mutable-receiver fixtures. The direct stale-effects case lost
one finding as expected; the copied receiver retained its one finding. The
diamond fixture has no consuming call and is an empty-effects/shared-DAG negative
control, not a positive consuming case. Observer mode measures collection in the
library, not CLI delivery or rendering. Full observer-on effects report parity
was not established by its timing count alone.

The protocol, controller, workloads, semantic outputs, inputs, binaries and
source manifests were hash-frozen before the first pilot. All 112 pilot rows
were retained. For each of 14 new workloads, eight pooled one-collection pilot
CPU samples across ABBA/BAAB determined `floor(1.2 / median)`, capped at 100,000
and at least one. No startup subtraction or arm selection occurred. The eight
original control batches stayed unchanged. Calibration and its manifest were
saved before the first long row. Because the pilot includes startup, focused
long samples can be shorter than 1.2 seconds; no post-hoc recalibration occurred.

All 960 long rows were retained: A/B original controls 256, hot facts 160,
graph/effects 288, and B/C controls 256. All children exited zero, with checked
signatures and empty stderr. No sample was dropped or retried. Other agents
confirmed compute idle through explicit START/END coordination; no local build,
formatter, test or Node analysis ran during this final window.

For each workload, the sampling unit is one balanced four-process block, with
two processes per arm. The point estimate is the geometric mean of eight ratios
of block arm means. Descriptive 95% percentile intervals resample the eight whole
blocks 10,000 times (seed 1972). Four ABBA and four BAAB blocks, all block ratios,
position means, absolute arm means and CPU/user/system/wall/peak-RSS results are
in [analysis.json](no-writes-investigation-evidence/final-three-arm/analysis.json).
Small-sample intervals do not guarantee stationarity or production throughput.

### A/B correctness costs

| Workload | CPU change [95% interval] | ABBA / BAAB | Wall change | Peak RSS change |
| --- | ---: | ---: | ---: | ---: |
| `facts-no_writes-values` | -2.022% [-2.711%, -1.318%] | -1.761% / -2.283% | -1.963% | -0.572% |
| `facts-no_writes-imports` | -0.030% [-0.618%, +0.508%] | -0.184% / +0.124% | -0.097% | +1.089% |
| `facts-writes-values` | +1.389% [+0.816%, +1.854%] | +1.365% / +1.414% | +1.354% | +2.434% |
| `facts-writes-imports` | +0.481% [+0.148%, +0.853%] | +0.484% / +0.477% | +0.631% | +2.253% |
| `facts-shared_histories-values` | +5.593% [+4.022%, +6.665%] | +5.895% / +5.292% | +5.607% | +0.767% |
| `facts-shared_histories-imports` | -0.160% [-0.672%, +0.471%] | +0.056% / -0.376% | -0.184% | +1.629% |
| `facts-jsx-values` | +0.184% [-0.294%, +0.663%] | -0.239% / +0.609% | +0.182% | +0.669% |
| `facts-jsx-imports` | +0.525% [+0.143%, +0.935%] | +0.890% / +0.162% | +0.523% | +1.838% |
| `hot-mutable-root-scale` | +66.100% [+65.139%, +67.034%] | +66.461% / +65.740% | +66.042% | +5.474% |
| `hot-mutable-pattern-phase-scale` | +26.900% [+25.939%, +28.058%] | +27.357% / +26.445% | +26.675% | +4.354% |
| `hot-mutable-pattern-deferred-diamond` | +6.743% [+5.995%, +7.484%] | +6.759% / +6.727% | +6.862% | +4.703% |
| `hot-mutable-pattern-eager-class-scale` | +38.055% [+37.201%, +38.736%] | +37.691% / +38.420% | +37.934% | +4.314% |
| `hot-mutable-pattern-cold-class-scale` | +30.357% [+29.560%, +31.218%] | +30.935% / +29.783% | +30.230% | +4.740% |
| `graph` | +4.695% [+4.597%, +4.799%] | +4.603% / +4.787% | +3.917% | +2.019% |
| `graph-observer` | +4.690% [+4.507%, +4.875%] | +4.798% / +4.583% | +3.622% | +2.166% |
| `graph-no-calls` | +0.222% [+0.083%, +0.343%] | +0.202% / +0.243% | +0.053% | +0.557% |
| `effects-mutable-let-direct-off` | +4.015% [+3.848%, +4.178%] | +3.962% / +4.067% | +2.796% | +2.376% |
| `effects-mutable-let-direct-observer` | +3.980% [+3.857%, +4.096%] | +3.979% / +3.981% | +2.975% | +2.559% |
| `effects-mutable-let-copied-off` | +3.943% [+3.786%, +4.078%] | +3.968% / +3.918% | +3.090% | +1.663% |
| `effects-mutable-let-copied-observer` | +4.027% [+3.742%, +4.400%] | +4.191% / +3.864% | +3.015% | +2.633% |
| `effects-mutable-pattern-deferred-diamond-off` | +4.013% [+3.832%, +4.164%] | +3.992% / +4.034% | +3.188% | +2.792% |
| `effects-mutable-pattern-deferred-diamond-observer` | +3.938% [+3.773%, +4.106%] | +3.895% / +3.981% | +2.934% | +2.493% |

### B/C rejected optimization

| Workload | CPU change [95% interval] | ABBA / BAAB | Wall change | Peak RSS change |
| --- | ---: | ---: | ---: | ---: |
| `facts-no_writes-values` | -2.171% [-2.854%, -1.369%] | -2.400% / -1.941% | -2.223% | +0.492% |
| `facts-no_writes-imports` | -7.464% [-8.412%, -6.574%] | -7.781% / -7.145% | -7.512% | -0.610% |
| `facts-writes-values` | -0.305% [-0.747%, +0.129%] | -0.449% / -0.161% | -0.356% | -1.377% |
| `facts-writes-imports` | -0.919% [-1.368%, -0.507%] | -1.177% / -0.660% | -0.909% | -1.620% |
| `facts-shared_histories-values` | +0.697% [-0.205%, +1.480%] | +0.549% / +0.845% | +0.702% | -0.529% |
| `facts-shared_histories-imports` | -0.529% [-1.141%, +0.091%] | -0.515% / -0.544% | -0.519% | -1.193% |
| `facts-jsx-values` | -0.206% [-0.712%, +0.246%] | -0.071% / -0.342% | -0.188% | -1.231% |
| `facts-jsx-imports` | -0.279% [-0.697%, +0.125%] | -0.270% / -0.289% | -0.275% | -1.548% |

The A/B costs are material and retained without offsetting them against B/C wins:
full graph CPU +4.695%, effects about +3.94–4.03%, and the old shared-history
control +5.593%. The focused root-scale percentage (+66.100%) corresponds to
approximately 0.186 → 0.309 ms/collection, including startup; full graph is about
20.090 → 21.034 ms/collection. These absolute scales do not waive the control
gate. The separate #2019 work must handle its performance decision; this evidence
neither approves that candidate nor applies to a subsequent implementation.

## Earlier attempts and interference disclosure

[Empty-import-guard records](no-writes-investigation-evidence/empty-import-guard/)
retain 64 pilot, 256 first-long and 256 clean-repeat rows. Another agent ran
`cargo fmt` for about 3.9 seconds during the first long series. Exact affected
rows were not identified. That whole series is preserved as a contaminated
diagnostic and excluded from the idle-window gate, without deleting observations
or guessing the interference's effect. The approved repeat retained the entire
fixed matrix with unchanged binaries/batches and reversed adjacent order.
The pilot remains calibration/diagnostic evidence, not an idle-window gate.

[Inline-scope records](no-writes-investigation-evidence/inline-scope/) retain all
256 subsequent rows and its rejection. Its full fact hashes matched. The final
three-arm experiment is a fresh pair with its own full semantic review; no old
samples were relabeled as final-source observations. Memory is not described as
flat: every series' recorded RSS, including adverse increases, remains available.

## Reproduction, omissions, and reopen criteria

Run the following from the evidence directory to verify shipped hashes and
recompute the retained final analysis without running benchmarks:

```sh
python3 verify-public.py
python3 final-three-arm/analyze.py
python3 empty-import-guard/summarize.py empty-import-guard
python3 inline-scope/summarize.py inline-scope
```

The analysis scripts reproduce their saved summaries. They are standalone
evidence tools, not production analysis or additional default benchmark work.
The package includes the complete measurement controller, frozen protocol,
calibration, workloads, probe source/original modules and source/input manifests.
To repeat sampling, recreate the exact source revisions and immutable input tree,
restore saved `graph_semantics.rs.txt` artifacts to `.rs` in a scratch copy,
use `build-arm.py` in isolated temporary worktrees, verify hashes/profile/features
and full semantic outputs, then reserve an idle window with a fresh output
directory. Replace explicit `<LOCAL>`/`<HOME>` placeholders with local paths.
Never overwrite these observations, silently use a newer compiler, or execute
an old manifest's binary name without checking its actual hash. Earlier drivers
similarly require their labeled source/input layout. Patch artifacts are JSON-encoded;
decode their `patch` field before applying to a scratch source checkout.

Numeric raw rows are byte-for-byte original copies. Selected local paths in
metadata/scripts are explicitly normalized; the public-copy mapping records
original/public byte counts and hashes. Internal run-manifest hashes refer to
original artifacts, not transformed public copies. Full Debug reports (including
large shared-history dumps), binaries, libraries, detailed Cargo logs and cache
metadata remain withheld locally. Public semantic hashes, lossless-compression
records and independently reviewed differences retain their provenance. No
project source dumps, profiler archives, executable artifacts or credentials are shipped.
The rejected patch and historical source commits preserve the attempted code;
removing obsolete worktrees does not rewrite their evidence.

Close #1972 as the completed bounded investigation recorded here, not as a claim
that the original slowdown was repaired. Reopen with a new explicit question if:

- A matched original optimized source pair reproduces a material positive CPU
  effect with both balanced orders, predeclared batches, independent controls,
  verified distinct artifacts and full semantic parity on a stable host.
- Native profiles or available cycle/cache counters identify attributable work
  explaining the preserved original opt0 slowdown; instruction counts or a field
  layout hypothesis alone are insufficient.
- A specific current production workload demonstrates a separately reproducible
  cost. Record its exact pair and consuming/non-consuming scope; do not relabel
  it as the historical change.

For any proposed fix, preserve function-call demand with value flow both off/on,
shared histories, JSX, writes, cold construction, CPU/wall/RSS and observer
controls. Apply the repository's slower-control gate before landing code. Do not
run repeated historical experiments solely to obtain a preferred sign, and do not
attribute a local/runner or architecture mismatch without matching evidence.
