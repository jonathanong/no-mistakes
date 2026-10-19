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
      - { callee: createEnqueueFunction, queue: 0, job: 1 }
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
      cluster: orders

tests:
  python:
    packages: [backend]
  go:
    modules: [services/worker]
```

`factories` names are matched against configured queue factory calls. An
enqueue factory records a producer at the wrapper call. A worker factory
records the processor, including `switch (job.name)` cases and `job.name ===`
checks. The `enqueues` and `workers` globs scope the files that participate in
a project; `cluster` gives producers and workers a shared queue namespace.
Empty lists disable that side of the relationship. A non-empty glob that
matches no file is reported by [`queues check`](../cli/queues-check.md).
Language test/graph frontends likewise
require explicit `tests.<language>` package or module roots; no repository-wide
queue scan is enabled by default.

Configured Celery, Asynq, Kafka, Active Job, Sidekiq, Laravel, and Symfony
Messenger sites emit the canonical `queue-enqueue` and `queue-worker` edges.
Use [`queues edges`](../cli/queues-edges.md), [`queues related`](../cli/queues-related.md),
and [`queues check`](../cli/queues-check.md) to inspect or validate them.
