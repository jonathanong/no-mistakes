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
    jobScheduler:
      - { callee: scheduleOrders, template: 2 }
    enqueueAndWait: [submitForResult]
    enqueueAndWaitPatterns: ['^submit.*ForResult$']
    forbidEnqueueAndWaitDeclarations: false
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
argument index of that call's options object; self-dedup, debounce, invalid
jobId, and mixed-priority checks read it the same way they read `queue.add`'s third argument.
`jobScheduler` names a wrapper that registers a repeated job. `template` is the argument
index of the job template. [`queues check`](../cli/queues-check.md) reads that
argument the same way it reads the third argument of `upsertJobScheduler`.
A worker factory records the processor. The active job is a parameter named `job`, or otherwise
the processor's first parameter, including `switch (task.name)` and
`async ({ name, id }) =>`. A nested callback is a job only when one of its
parameters is named `job`. Local named processor functions can be declared
before the worker registration. Worker job names come from literal job-name
equality comparisons and
`switch` labels on that job name. Unrelated object `name` properties, comments,
strings, and other switches do not narrow a worker. Dynamic job-name dispatch
keeps wildcard coverage, including when the processor also has literal branches.
An imported module with multiple callables stays wildcard when the processor
callable cannot be selected statically. Only a direct imported processor argument
contributes imported names; namespace references inside an inline processor do
not narrow that processor. The `enqueues` and `workers` globs scope the files
that participate in a project. `factories.enqueueAndWait` lists the exact names
of functions that enqueue a job and wait for its result, such as
`submitForResult(...)`. The list defaults to empty. Exact wrappers use the
same member, static computed member, import alias, immutable local alias,
and direct callback-reference handling as the built-in wait methods. Local
aliases must be `const` bindings with a stable symbol, no writes, and uses
after their declarations; dynamic keys do not establish a wait.

`factories.enqueueAndWaitPatterns` is an optional list of name patterns for
wait wrappers, defaulting to `[]`. Patterns use Rust `regex` syntax, not
JavaScript regular-expression literals: write `^submit.*ForResult$`, not
`/^submit.*ForResult$/`. Use anchors for whole-name matching,
such as `^submit.*ForResult$`; look-around and backreferences are unsupported.
Invalid patterns are configuration errors.

`factories.forbidEnqueueAndWaitDeclarations` defaults to `false`. Enable it to
report matching wait-wrapper declarations as `enqueue-and-wait-declaration`,
so unused wrappers cannot introduce a blocking API later. It checks runtime
function declarations and variables initialized with arrows or function
expressions; ambient `declare` signatures are not wrapper definitions. Replace the wrapper
with enqueue-only behavior and let the caller return control. The fix hint is
part of the finding's `message`, not a separate JSON field.

`waitScopes` limits that project's wait call/reference and declaration
findings, including the built-in
[`enqueue-and-wait` patterns](../cli/queues-check.md). An empty list checks
every file of the project, and a file outside every project stays checked.
A project root matches only that directory:
`root: api` covers `api/...` and does not cover `lib/api/...`.
`cluster` gives producers and workers a shared
queue namespace. Two factories with the same callee and different argument
indexes are a configuration error.
Empty lists disable that side of the relationship. A non-empty glob that
matches no file is reported by [`queues check`](../cli/queues-check.md).
A producer whose queue and job both failed to resolve is listed on every
unmatched worker, so that finding can be broader than the handler's own queue.
Language test/graph frontends likewise
require explicit `tests.<language>` package or module roots; no repository-wide
queue scan is enabled by default.

## Required job options

`queues.requiredJobOptions` is a list of job-option keys. It defaults to
`[]`. An empty list does not add findings or diagnostics. Set it when every
producer should pass the same keys, such as `attempts`, `backoff`,
`removeOnComplete`, or `removeOnFail`. `queues.requiredJobOptionsByQueue`
replaces the global list for that queue name. A present key of `[]` exempts
the queue. Producers are `add`, each `addBulk` element, `upsertJobScheduler`
templates, flow roots and children, and configured enqueue factories.
Factories without an `options` argument rely on queue defaults.
`defaultJobOptions` on the resolved queue definition counts, including
defaults supplied by a same-file const object spread in its options. An unknown
spread abstains instead of reporting a missing key when it leaves the effective
options unresolved and the effective list for that queue is non-empty.
`check --config <path>` and `analyzeProject({ config, ... })` use the
selected document's required job options, including an empty policy.
Rust callers analyzing prepared facts can pass the selected config to the
catalog-aware queue API.
Same-file constant options and spreads resolve from the binding at each use.
See [`queues check`](../cli/queues-check.md#required-job-options).

```yaml
queues:
  requiredJobOptions: [attempts, removeOnComplete, backoff]
  requiredJobOptionsByQueue:
    items: [priority]
    orders: []
```

Configured Celery, Asynq, Kafka, Active Job, Sidekiq, Laravel, and Symfony
Messenger sites emit the canonical `queue-enqueue` and `queue-worker` edges.
Use [`queues edges`](../cli/queues-edges.md), [`queues related`](../cli/queues-related.md),
and [`queues check`](../cli/queues-check.md) to inspect or validate them.
