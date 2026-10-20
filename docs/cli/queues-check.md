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
| `unmatched-worker-unresolved-producers` | A producer for this handler did not resolve, so the handler is not known to be dead. The message names those producer lines. A producer with neither a resolved queue nor a resolved job is listed on every unmatched worker. |
| `self-dedup-enqueue` | A processor enqueues its own job on its own queue with `deduplication.id` or `jobId` taken from the active job. The job binding is a parameter named `job`, or otherwise the processor's first parameter: `task.id` / `task.deduplicationId`, or a destructured `{ name, id }`. A nested callback is not that binding unless one of its parameters is named `job`. A local function or `const` callback passed by name is tied to its configured worker even when declared earlier. A copied `const id = task.id` or `const id = task.deduplicationId` remains a job token within its lexical scope; unrelated values and shadowed parameters do not. A configured enqueue factory does the same check on its `options` argument. The queue drops that add while the job is active. Publish a new id, or enqueue a different job. The same job name on a different queue is not a finding. If the worker queue or the target queue does not resolve, the check abstains and emits a diagnostic. |
| `enqueue-and-wait` | `addAndWait` or `waitUntilFinished`, including `this.queue.addAndWait`, waits on a job result and holds a worker or request slot. Return control instead of blocking on the result. `projects.<name>.queues.waitScopes` limits that project's files; an empty list checks every file of the project, and a file outside every project stays checked. |
| `debounce-ttl` | `deduplication: { mode: "debounce", ttl }` sets a `ttl` that debounce mode ignores. Remove `ttl`, or use a mode that reads it. |
| `empty-glob` | A configured `enqueues` or `workers` glob matches no file. Fix the glob or remove it. An empty list still disables that side and is not a finding. |
| `queue-cycle` | A worker queue enqueues another queue that eventually enqueues it back. Only an enqueue inside that worker's processor counts. Two workers in one file do not share each other's enqueues. Review the loop before shipping; this check has no terminator allowlist. |

Local processor functions declared before `createWorker("jobs", handler)` use
the same job binding as inline callbacks. A local copy such as
`const id = task.id` also counts when passed as `jobId: id`.

Worker job names come from dispatches on the processor's job name, such as
`task.name === "send"` or `switch (task.name)`. Unrelated objects such as
`{ name: "metadata" }` do not declare jobs. A processor with dynamic dispatch
stays a wildcard, so its literal branches do not exclude other job names.

## Suppression

`queues check` findings use the rule id `queues-check`. A file directive
suppresses every finding in that file. A next-line directive suppresses the
finding on the following line. The rule id is required. A bare
`no-mistakes-disable-file`, with no rule id, does not suppress these findings:

```ts
// no-mistakes-disable-file queues-check: unmatched topology is intentional
// no-mistakes-disable-next-line queues-check
```

Node API: `queueCheck(options)`.

For explicitly declared JSON payload schemas, configure
[declared-payload-compatibility](../rules/declared-payload-compatibility.md) and
run `no-mistakes check`. Declaration compatibility is a separate opt-in check.
