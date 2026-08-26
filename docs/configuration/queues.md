# Queue configuration

Queue analysis is explicit so agents can reason about producer/worker
relationships without guessing project conventions. Use the top-level
`queues.factories` list for custom TypeScript queue factory names, and put
project-specific file scopes under `projects.<name>.queues`.

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

`factories` names are matched against configured queue factory calls. The
`enqueues` and `workers` globs scope the files that participate in a project;
`cluster` gives producers and workers a shared queue namespace. Empty lists
disable that side of the relationship. Language test/graph frontends likewise
require explicit `tests.<language>` package or module roots; no repository-wide
queue scan is enabled by default.

Configured Celery, Asynq, Kafka, Active Job, Sidekiq, Laravel, and Symfony
Messenger sites emit the canonical `queue-enqueue` and `queue-worker` edges.
Use [`queues edges`](../cli/queues-edges.md), [`queues related`](../cli/queues-related.md),
and [`queues check`](../cli/queues-check.md) to inspect or validate them.
