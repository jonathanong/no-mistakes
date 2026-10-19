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
| `unmatched-worker-unresolved-producers` | A producer for this handler did not resolve, so the handler is not known to be dead. The message names those producer lines. |
| `self-dedup-enqueue` | A processor enqueues its own job on its own queue with `deduplication.id` or `jobId` taken from the active job (`job.id` or `job.deduplicationId`). The queue drops that add while the job is active. Publish a new id, or enqueue a different job. The same job name on a different queue is not a finding. If the worker queue or the target queue does not resolve, the check abstains and emits a diagnostic. |
| `enqueue-and-wait` | `addAndWait` or `waitUntilFinished` waits on a job result and holds a worker or request slot. Return control instead of blocking on the result. |
| `debounce-ttl` | `deduplication: { mode: "debounce", ttl }` sets a `ttl` that debounce mode ignores. Remove `ttl`, or use a mode that reads it. |
| `empty-glob` | A configured `enqueues` or `workers` glob matches no file. Fix the glob or remove it. An empty list still disables that side and is not a finding. |
| `queue-cycle` | A worker queue enqueues another queue that eventually enqueues it back. Only an enqueue inside that worker's processor counts. Two workers in one file do not share each other's enqueues. Review the loop before shipping; this check has no terminator allowlist. |

Node API: `queueCheck(options)`.

For explicitly declared JSON payload schemas, configure
[declared-payload-compatibility](../rules/declared-payload-compatibility.md) and
run `no-mistakes check`. Declaration compatibility is a separate opt-in check.
