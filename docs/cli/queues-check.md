# `no-mistakes queues check`

Check queue producers, workers, and the enqueue calls those workers make.

```sh
no-mistakes queues check --format json
```

Use this before finishing queue edits so every configured enqueue/worker path is
connected. `check` is empty when the graph is consistent. Findings use these
`kind` values:

| `kind` | Meaning |
| --- | --- |
| `unmatched-producer` | A job is enqueued and no worker handles it. |
| `unmatched-worker` | A worker handles a job that no producer enqueues. |
| `self-dedup-enqueue` | A processor enqueues its own job with `deduplication.id` or `jobId` taken from the active job (`job.id` or `job.deduplicationId`). The queue drops that add while the job is active. Publish a new id, or enqueue a different job. |
| `enqueue-and-wait` | `addAndWait` or `waitUntilFinished` waits on a job result and holds a worker or request slot. Return control instead of blocking on the result. |
| `debounce-ttl` | `deduplication: { mode: "debounce", ttl }` sets a `ttl` that debounce mode ignores. Remove `ttl`, or use a mode that reads it. |
| `empty-glob` | A configured `enqueues` or `workers` glob matches no file. Fix the glob or remove it. An empty list still disables that side and is not a finding. |
| `queue-cycle` | A worker queue enqueues another queue that eventually enqueues it back. Review the loop before shipping; this check has no terminator allowlist. |

Node API: `queueCheck(options)`.

For explicitly declared JSON payload schemas, configure
[declared-payload-compatibility](../rules/declared-payload-compatibility.md) and
run `no-mistakes check`. Declaration compatibility is a separate opt-in check.
