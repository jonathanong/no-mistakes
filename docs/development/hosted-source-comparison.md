# Matched hosted source comparisons

`hosted-check.yml` optionally accepts an exact `baselineRef` alongside the exact
`candidateRef`. Both sources build with the same official production settings,
default Mermaid features and compiler. Their source commits, binary checksums,
Cargo lock and manifest hashes, actual library fingerprints and CLI versions are
recorded and verified. Identical binaries or different build settings fail before
measurement. The original published 0.86.2 comparison and calibrated absolute
budget enforcement remain the default.

`diagnosticsMode` selects `off`, `on`, or `both` for source comparisons. ON adds
`--timings --verbose-timings`; both source binaries must emit real `[timing]` and
`[work]` records. OFF must emit neither. Diagnostics remain on stderr; the check's
stdout findings and advisories must match byte for byte. By default both checks
must exit zero. The narrow expected-findings mode below can record comparable
completed checks that still fail; it never changes their exit or acceptance. The
consumer configuration and tracked tree remain unchanged. Invalid source identity,
output, diagnostics, or completeness cannot be treated as performance evidence.

The fixed complete campaign uses six fresh VM pairs for each combination of two
pinned consumer trees, jobs 1/4/automatic and diagnostics OFF/ON: 72 pairs and
144 commands. All 72 independent pairs may run concurrently, subject to the
account's runner quota. Each pair alternates binary order by sample number and
runs its commands sequentially on one VM. Each command retains the 600-second
collection safety cap; the two-command step remains bounded at 22 minutes in a
30-minute job. Build, install and preparation are outside measured time. Default
published-baseline campaigns retain their six-job concurrency.

## Comparison rule recorded before dispatch

For each of the twelve rows and each wall/RSS metric, let `q` be the six paired
candidate/baseline ratios and let gain be `1 - mean(q)`. Let `h` be the six paired
current/published ratios from the committed d3d calibration for that fixture and
worker selection. The threshold is the larger of `3 * sampleSD(h) / mean(h)` and
`3 * sampleSD(q) / mean(q)`.

A clear win requires all six ratios below one and mean gain exceeding that
threshold. A clear regression requires all six ratios above one and mean loss
exceeding it. A consistent slowdown in all six pairs also blocks automatic
acceptance even when below the threshold; a high historical threshold does not
waive a reproduced control slowdown. Acceptance requires a clear wall or peak
RSS win in a representative row and no such slowdown in any other row or metric.
All rows and raw pairs are reported; CPU is context rather than the win metric.

| Fixture         | Jobs      | Historical wall 3 CV % | Historical RSS 3 CV % |
| --------------- | --------- | ---------------------: | --------------------: |
| Acceptance      | 1         |                 4.4260 |                0.0949 |
| Acceptance      | 4         |                14.6166 |                7.8815 |
| Acceptance      | automatic |                16.6907 |                6.5167 |
| Passing/failing | 1         |                15.9810 |                0.0675 |
| Passing/failing | 4         |                10.9932 |                8.1494 |
| Passing/failing | automatic |                13.3696 |                6.2648 |

These are heuristic relative-variability thresholds, not a comparison of
identical binaries, statistical significance, or a guaranteed noise bound.
Historical calibration used diagnostics OFF, different analyzer versions and a
mixed CPU population. Transferring its relative variability to ON is a limitation;
contemporary ON variability is also measured. Paired runs share a VM, while
different pairs can use different CPU models and images. Source changes unrelated
to the proposed optimization must be eliminated before interpreting causality.
The committed historical absolute budgets are reported separately for both
sources. They are informational in this optional matched mode: genuine main
feature growth cannot silently be classified as an optimization regression.

A complete valid campaign without an accepted win reports
`valid-no-qualifying-win` and fails the performance gate. Missing, censored,
inconsistent or otherwise invalid evidence reports `invalid-measurement` and
cannot establish a measured no-win. Both retain raw evidence. Do not repeat a
valid slow observation or valid no-win campaign to seek another outcome. A
necessary repeat after fixing a proven measurement defect must preserve the
original attempt, fix and reason separately.

## Registered artifact reuse

An optional numeric `artifactRunId` reuses only an explicitly registered original
run and its immutable baseline/current artifact IDs. The verifier checks the exact
requested source commit and checked-out Git tree, all four original file hashes
(`no-mistakes`, `cpu-probe`, `provenance.json`, `SHA256SUMS`), and the source
checkout's Cargo lock, original manifest and recorded release-manifest patch.
Missing, extra, changed or non-file artifact entries fail verification. The source
checkout must remain clean. The original compiler, commands and build provenance
remain unchanged; this path installs no Rust compiler and runs no cache/build
step. It does not claim a new build or new compiler identity.

Only the build job receives `actions: read` for the registered cross-run download.
The replacement uploads the same production artifact names with identical four
files, and saves reuse verification separately. Sample jobs still download their
pair from the replacement run. Arbitrary run IDs, artifact roles or source commits
cannot enable reuse. Without the input, the existing production build path stays
in place.

## Expected findings remain failed checks

`expectedFindings` defaults to false and requires an exact source baseline. Its
opt-in contract is limited to the registered main284/full-stack96b source pair,
pinned consumer trees, configuration hashes and exact stdout report hashes. A
completed expected-finding result retains actual exit1, `checkOutcome: failure`
and `checkAccepted: false`. Non-diagnostic stderr must match its registered
contract; diagnostic output must satisfy its requested mode and strict parser.
Runtime exit2, signals, timeouts, missing or changed reports, unexpected stderr
and mismatched source/configuration are not comparable measurements.

Performance completion/comparability and the paired metric criterion are separate
from check acceptance. A complete expected-findings sample still fails its check
and the workflow remains failed even if its performance comparison meets the
metric criterion. Legacy zero-exit success, default published-baseline enforcement
and absolute budget rules are unchanged. No findings are suppressed and no
consumer configuration is edited.

The original attempt at
[run39093709993](https://github.com/jonathanong/no-mistakes/actions/runs/39093709993)
used a zero-exit-only source collector on matching normal findings. It remains an
invalid measurement attempt, with raw exits/reports and cancellation preserved;
it is not a measured optimization loss or a successful check. A replacement may
reuse its independently verified immutable production bytes while fixing only
the explicit expected-outcome contract. Keep both attempts and their harness
sources; never combine their sample populations or retry valid slow observations.
