# Shared-source native measurements

Build distinct release executables for the published release tag, unmodified
main, and the candidate. Use the same toolchain, default features, thin LTO,
16 codegen units, and capped build jobs for every build. Freeze each executable
before switching checkouts; never compare a shared target's mutable binary.

Run `measure.py` under `/tmp/nm-heavy.lock` with no other builds or tests. Pass
`--root` for the candidate checkout, three `--binary label=/absolute/binary`
arguments, and matching `--revision label=full-sha` arguments. Use
`--output /absolute/results.json --blocks 3 --jobs 1 4 default`. The output
contains executable hashes, exact source revisions, raw wall/user/system/CPU
samples, peak RSS, medians and ranges. It deliberately enables no observer.
Each invocation has fresh in-memory state; filesystem page caches stay warm.

Each case warms each variant once, then runs forward and reverse variant
order per block: release/main/candidate/candidate/main/release. Every sample
compares exact stdout, stderr, and exit status. Saved fixtures assert their
expected finding counts before timing results are retained. The normal
report and `includeSuppressed` paths are separate cases. `--case` selects a
saved workload; `--preflight-only` checks parity without retaining samples.

The fixture READMEs describe source sizes and configured payloads. The
[results and audit](../../../docs/performance/shared-source-suppression.md)
separate measured runtime changes from unchanged compatibility paths.
