# lock

Inspect the machine-wide invocation lock that serializes `no-mistakes`
analyses. Each analysis uses every core, so the lock is global by design.

## Subcommands

| Command | Purpose |
| --- | --- |
| [`lock status`](lock-status.md) | Report whether another invocation holds the lock, and who. |

See the [exit-code contract](README.md#exit-code-contract) for how lock
contention surfaces as exit `75`.
