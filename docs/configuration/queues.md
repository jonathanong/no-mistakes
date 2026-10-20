# Queue configuration

Queue analysis is explicit so agents can reason about producer/worker
relationships without guessing project conventions. Use the top-level
`queues.factories` entry for custom TypeScript queue wrappers, and put
project-specific file scopes under `projects.<name>.queues`.

A list of names is the queue-factory form. Each name is a call whose first
argument is the queue name. Wrappers that hide the queue object, the job name,
or the worker processor use
the object form. Indexes are zero-based argument positions. Queue names may be
string literals, no-substitution template literals, same-file constants, or
imported string constants. A queue object may be aliased
(`export const adminImports = background`), re-exported
(`export { queue as alias } from "./queues"` or `export * from "./queues"`),
or passed as a namespace member (`queues.email`). A producer whose job does
not resolve abstains: [`queues check`](../cli/queues-check.md) does not report
that queue's handlers as dead.

```yaml
queues:
  factories:
    queue:
      - callee: createQueue
        name: 0
    enqueue:
      - { callee: createEnqueueFunction, queue: 0, job: 1, options: 2 }
      - { callee: createBulkEnqueueFunction, queue: 0, job: 1 }
    worker:
      - { callee: createWorker, name: 0, processor: 1 }
```

```yaml
queues:
  factories: [createQueue, getQueue]

projects:
  jobs:
    type: server
    root: services
    queues:
      enqueues: ["**/producers/**/*.ts", "**/tasks.py"]
      workers: ["**/workers/**/*.ts", "**/tasks.py"]
      waitScopes: ["**/routes/**/*.ts"]
      cluster: orders

tests:
  python:
    packages: [backend]
  go:
    modules: [services/worker]
```

`factories` names are matched against configured queue factory calls. An
enqueue factory records a producer at the wrapper call. `options` is the
argument index of that call's options object; self-dedup and debounce checks
read it the same way they read `queue.add`'s third argument. A worker factory
records the processor. The active job is a parameter named `job`, or otherwise
the processor's first parameter, including `switch (task.name)` and
`async ({ name, id }) =>`. A nested callback is a job only when one of its
parameters is named `job`. The `enqueues` and `workers` globs scope the files
that participate in a project. `waitScopes` limits `addAndWait` and
`waitUntilFinished` findings, including `this.queue.addAndWait`, for that
project. An empty list checks every file of the project, and a file outside
every project stays checked. `cluster` gives producers and workers a shared
queue namespace. Two factories with the same callee and different argument
indexes are a configuration error.
Empty lists disable that side of the relationship. A non-empty glob that
matches no file is reported by [`queues check`](../cli/queues-check.md).
A producer whose queue and job both failed to resolve is listed on every
unmatched worker, so that finding can be broader than the handler's own queue.
Language test/graph frontends likewise
require explicit `tests.<language>` package or module roots; no repository-wide
queue scan is enabled by default.

Configured Celery, Asynq, Kafka, Active Job, Sidekiq, Laravel, and Symfony
Messenger sites emit the canonical `queue-enqueue` and `queue-worker` edges.
Use [`queues edges`](../cli/queues-edges.md), [`queues related`](../cli/queues-related.md),
and [`queues check`](../cli/queues-check.md) to inspect or validate them.
