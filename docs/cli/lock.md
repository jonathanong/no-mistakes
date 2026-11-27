# lock

Inspect the per-user invocation lock that serializes `no-mistakes`
analyses. Analyses can use every core, so invocations sharing a lock run serially.

## Subcommands

| Command | Purpose |
| --- | --- |
| [`lock status`](lock-status.md) | Report whether another invocation holds the lock, and who. |

See the [exit-code contract](README.md#exit-code-contract) for how lock
contention surfaces as exit `75`.
