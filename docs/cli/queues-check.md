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
| `self-dedup-enqueue` | A processor enqueues its own job on its own queue with `deduplication.id` or `jobId` taken from the active job. The job binding is a parameter named `job`, or otherwise the processor's first parameter: `task.id` / `task.deduplicationId`, or a destructured `{ name, id }`. A nested callback is not that binding unless one of its parameters is named `job`. A local function or `const` callback passed by name is tied to its configured worker even when declared earlier. A copied `const id = task.id` or `const id = task.deduplicationId` remains a job token within its lexical scope, as does a destructured `const { id } = task` or `const { deduplicationId } = task`. A defaulted binding such as `const { deduplicationId: dedup = "fallback" } = task` is not a token. Unrelated values and shadowed parameters do not. A configured enqueue factory does the same check on its `options` argument. The queue drops that add while the job is active. Publish a new id, or enqueue a different job. The same job name on a different queue is not a finding. If the worker queue or the target queue does not resolve, the check abstains and emits a diagnostic. |
| `enqueue-and-wait` | Waiting on a job result holds a worker or request slot. Return control instead of blocking on the result. The check covers calls to `addAndWait`, `waitUntilFinished`, and explicitly configured wrappers. |
| `shared-queue-rate-limit` | A processor throws `RateLimitError` (`new RateLimitError()` or `Worker.RateLimitError()`) or calls `.rateLimit()` on a queue whose producers enqueue more than one job name. That limit stalls every job on the queue. Move that job kind to its own queue, or use per-group rate limits. One known job name is not a finding. A producer on that queue with no resolved job name, or a worker queue that does not resolve, abstains with a diagnostic instead of a finding. The throw or call has to sit in the processor passed to `new Worker` or a configured worker factory. A helper outside that processor is not a finding. |
| `enqueue-and-wait-declaration` | An explicitly matched wait-wrapper declaration is forbidden by `queues.factories.forbidEnqueueAndWaitDeclarations`. Replace it with enqueue-only behavior; the fix hint is included in `message`. Disabled by default. |
| `debounce-ttl` | `deduplication: { mode: "debounce", ttl }` sets a `ttl` that debounce mode ignores. Remove `ttl`, or use a mode that reads it. |
| `invalid-job-id` | A custom `jobId` literal, or a static part of a template literal, contains `:`, `{`, `}`, or a control character. `:` is the BullMQ and glide-mq key separator, so an id containing it can collide with internal keys. `{` and `}` collide with Redis or Valkey cluster hash tags. Interpolations such as `${id}` are not judged, and neither is a fully dynamic id. Use `-` or `_` as the separator. A `const` job ID resolves from its lexical binding. The last explicit `jobId` supplies the ID. The check abstains if a later spread or computed property could override that ID. The check reads `jobId` on proven queue receivers and configured producers, including `add` options, each `addBulk` item's `opts`, flow-node `opts` (including children), `upsertJobScheduler` template `opts`, and a configured enqueue factory's options argument. |
| `scheduler-template-option` | A job-scheduler template sets `delay`, `deduplication`, `parent`, or `jobId`. The scheduler's repeat settings control timing and identity, so the call does not do what that option reads as. Remove the option from the template. The check reads literal keys on the template and inside its `opts`, including a spread of a same-file `const` object literal. The last `opts` supplied by an explicit property or a known spread determines which options are checked. A module `const` is visible to functions declared earlier in the file. An unknown or cyclic spread, a non-literal key, or a binding that is not a `const` object literal makes that call abstain. Keys nested under `data` or any other object are not options. Built-in `upsertJobScheduler` and a literal computed `queue["upsertJobScheduler"]` read the third argument. `queues.factories.jobScheduler` reads the configured `template` argument on calls to the named import, including its local alias. |
| `empty-glob` | A configured `enqueues` or `workers` glob matches no file. Fix the glob or remove it. An empty list still disables that side and is not a finding. |
| `queue-cycle` | A worker queue enqueues another queue that eventually enqueues it back. Only an enqueue inside that worker's processor counts. Two workers in one file do not share each other's enqueues. Review the loop before shipping; this check has no terminator allowlist. |

The wait check recognizes member calls such as `this.queue.addAndWait(...)`
and literal computed members such as `queue["addAndWait"](...)`. It also
follows immutable local destructured aliases, such as
`const { addAndWait: wait } = queue`, after their declarations, when called or passed as direct arguments.
A captured member such as `const wait = queue.addAndWait` and immutable alias copies are followed too. Passing the member itself, as in `wrap(queue.addAndWait)`, is a finding because the recipient may invoke it. Ordinary property reads, dynamic keys, defaulted destructuring, and mutable or shadowed aliases do not establish a wait.
Exact wrapper names in `queues.factories.enqueueAndWait` use the same
member, static computed member, import alias, immutable local alias, and
direct callback-reference handling as the built-in methods. Local aliases
must be `const` bindings with stable symbols, no writes, and uses after their
declarations; dynamic keys do not establish a wait.

Configure additional wrapper-name patterns with
`queues.factories.enqueueAndWaitPatterns` (Rust `regex` syntax; default `[]`).
Enable `queues.factories.forbidEnqueueAndWaitDeclarations` (default `false`)
to reject matching wrapper declarations even without a call. Both kinds of
wait finding respect [`projects.<name>.queues.waitScopes`](../configuration/queues.md).
For example, replace a forbidden `submitForResult` wrapper with an enqueue-only
`submit` function and return control to the caller. Fix guidance remains in
`message`; these options add no JSON fields. With both new options off,
existing output is preserved except for the expanded exact-wrapper recognition.

Local processor functions declared before `createWorker("jobs", handler)` use
the same job binding as inline callbacks. A local copy such as
`const id = task.id` and `const { id } = task` also count when passed as
`jobId: id`. A defaulted binding does not.

Worker job names come from dispatches on the processor's job name, such as
`task.name === "send"` or `switch (task.name)`. Unrelated objects such as
`{ name: "metadata" }` do not declare jobs. A processor with dynamic dispatch
stays a wildcard, so its literal branches do not exclude other job names.
Imported processor summaries stay open when a module has multiple top-level
callables, or job-name notes in more than one function, and the processor
cannot be selected soundly. A nested callback that does not read the job name
does not open that summary. `processorFile` identifies a
direct imported processor argument; an inline callback that calls an imported
helper does not adopt that helper module's job names.

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
