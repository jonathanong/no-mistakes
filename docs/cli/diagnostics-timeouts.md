# Diagnostics on a hard timeout

On a CLI hard timeout, enabled timings attempt a separate `[PARTIAL]` stderr
snapshot before exit `124`, even without `--progress`. `[PARTIAL timing]` lines
contain elapsed time and completed recorded durations; `[PARTIAL work]` lines
contain the exact published counter values available to the snapshot. These
counts are lower bounds on unfinished analysis: active arms can still hold
unpublished local counts, and concurrent counters are sampled independently.
This does not enable progress tracking. A busy timing ledger is labeled
unavailable while work counts are still collected. Snapshot collection and
stderr delivery are bounded and best-effort, including with diagnostics disabled.
A blocked snapshot lock or full stderr pipe can omit diagnostics but cannot delay the hard
exit indefinitely. Timeout snapshots do not publish findings to stdout.

Normal completion keeps the existing diagnostics format. Hard-timeout snapshots
use separate `PARTIAL` prefixes so tooling can distinguish incomplete evidence.
