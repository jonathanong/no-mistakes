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

The `error:` line, the pending marker, the snapshot, an optional `[progress]`
line, and the incomplete marker share one stderr writer. The incomplete line is
queued only when that writer has not already accepted the snapshot, or when the
snapshot is queued and the progress line is not. A line that loses the race is
omitted, so the incomplete marker cannot appear before a late snapshot line.

```text
error: command timed out
[PARTIAL] timeout snapshot pending; collection and delivery are best-effort
[PARTIAL] timeout snapshot: published work counts are lower bounds; unfinished work and timings may be absent
[PARTIAL timing] elapsed: 1.250ms
[PARTIAL work] source.reads: 7
```
