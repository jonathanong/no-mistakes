# Queue configuration

Queue analysis is explicit so agents can reason about producer/worker
relationships without guessing project conventions.
Swift, .NET, Java, Kotlin, Elixir, Dart, and Rust frontends record no
language-specific queue jobs. The shared Kafka scanner checks configured
globs for supported literal shapes regardless of the file extension. See
[feature parity](../feature-parity.md). Use the top-level
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

`factories` names are matched against configured queue factory calls. A
positional enqueue factory records a producer at the wrapper call. `options`
is the argument index of that call's options object; self-dedup, debounce, invalid
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

## Configured enqueue selectors and returned callables

Enqueue roles match named runtime imports, including import aliases. An
enqueue role can select a numeric argument, or a direct property with
`{ arg, property }`. `queueName` fixes the queue instead of selecting an
argument. Declare exactly one of `queue` and `queueName`.

```yaml
queues:
  factories:
    enqueue:
      - callee: makeObjectEnqueue
        queue: { arg: 0, property: queueName }
        job: { arg: 0, property: jobName }
        defaults: { arg: 0, property: defaults }
        optionsOnReturnedCall: 1
      - callee: makeReturnedEnqueue
        queue: 0
        job: 1
        optionsOnReturnedCall: 1
      - callee: makeFixedEnqueue
        queueName: background
        job: { arg: 0, property: name }
        optionsOnReturnedCall: 1
      - callee: makeFixedBulkEnqueue
        queueName: background
        job: { arg: 0, property: name }
        optionsOnReturnedCall: 1
```

For `makeObjectEnqueue({ queueName: "background", jobName: "send", defaults })`,
selectors read the named properties of argument zero. `defaults` and `options`
can select an argument or property. Effective job options combine wrapper
`defaults`, factory-call `options`, then returned-call options, in that order;
a later supplied property overrides an earlier one. Defaults belong to that
wrapper, not to every producer on the same queue.

`optionsOnReturnedCall: 1` describes `const send = makeReturnedEnqueue(queue,
"send"); send(data, options)`. The construction is not itself an enqueue;
producer locations and option checks refer to actual calls through the
immutable `const` binding, including another immutable `const` alias. Without
this setting, existing enqueue roles continue to record the factory call.

For fixed queues, `makeFixedEnqueue(JOB_DEF)` selects `JOB_DEF.name` from a
local or imported `const` object literal. Named re-exports, imported queue
constants (including `new Queue(QUEUE_NAME)` and `new Worker(QUEUE_NAME, ...)`),
and workspace package specifiers—including `package.json` `exports` wildcards
such as `"./*": "./*.mts"`—use the prepared module and workspace resolver,
with or without `--tsconfig`. When a fixed-`queueName` queue has no matching
`new Queue` or configured queue factory declaration, its queue node uses the
deterministic virtual anchor `queue:<name>` (for example, `queue:mail#welcome`)
instead of anchoring on an arbitrary caller file. The bulk wrapper's build
callback does not supply a job name; only the configured selector does.
Resolution stays within the configured file universe and uses the same
producer/worker graph as `queues edges`. Files matched by configured `tests.*`
scopes are excluded from every queue check and production `queues edges` by
default (with no hardcoded filename conventions) unless opted back in with
`queues.includeTests`; same-file raw `new Worker(<queueName>, ...)` and
configured worker factories match producers on that queue name. See
[Test-file scope](#test-file-scope) and
[`queues check`](../cli/queues-check.md#producer-and-worker-matching).

Property selection requires a direct, static object property. Unresolved
values, spread objects, computed properties, non-const bindings, and mutated
objects with observed member writes do not establish a concrete name.
Excessively deep alias chains also abstain. Recognized invocations retain
unresolved producer locations when a name is unknown; they create no concrete
job edge. Worker findings then describe unresolved producers instead of
claiming there are no callers. Independently known options are still judged.
A known-absent options property preserves wrapper defaults; an open object
keeps options unknown.
Wrapper bodies and arbitrary side effects in other calls are not executed
or inferred; keep job definitions as immutable metadata. Unknown options are
not treated as empty options or as priority zero. Rewrite these cases as direct immutable
literals when a static result is required. Configuring a wrapper is explicit;
functions without a configured role do not become producers.

JavaScript callers can name these config shapes with exported
`QueueFactoryArgument`, `QueueEnqueueFactoryOptions`, and `QueueFactories`.
These describe the `queues.factories` config-file value. The existing async
queue APIs read the root configuration; no direct factory argument is added
to `queueCheck()`. `queueCheck()`, `check()`, and `analyzeProject()` accept
a `config` path to select a file.

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
`queues check --config <path>`, `check --config <path>`, and their
programmatic equivalents use the selected document's required job options,
including an empty policy.
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

## Test-file scope

Every queue check (`unmatched-producer`, `unmatched-worker`,
`unbounded-job-retention`, `job-state-as-record`, `missing-job-option`,
`invalid-job-id`, `debounce-ttl`, `scheduler-template-option`,
`enqueue-and-wait`, `queue-cycle`, `self-dedup-enqueue`,
`shared-queue-rate-limit`, `worker-rate-limit-per-process`, and
`mixed-priority`) and production `queues edges` share one declared test-file
exclusion derived from configured `tests.*` scopes (and discovered
Vitest/Playwright configs). No hardcoded filename conventions (`*.test.*`,
`__tests__`) are applied when no test scope is declared.

Use `queues.includeTests` (default `false`) to explicitly bring configured test
files back into the shared queue scope without removing them from `tests.*`:

```yaml
queues:
  includeTests: ["tests/orders.test.ts"]
```

Set `queues.includeTests: true` to include all configured test files, or pass a
list of relative glob patterns. JavaScript callers can name this configuration
shape with the exported `QueueIncludeTestsConfig` type.

## Unbounded job retention

`queues.unboundedJobRetention` is opt-in and disabled by default. Set it to
`true` to require bounded `removeOnComplete` and `removeOnFail` on every
resolved producer and queue `defaultJobOptions`, or pass an object to select
keys and exempt file paths:

```yaml
queues:
  unboundedJobRetention:
    require: [removeOnComplete, removeOnFail]
    allowPaths: ["scripts/**"]
```

- `require` selects which of `removeOnComplete` and `removeOnFail` are checked
  (defaults to both when enabled).
- `allowPaths` lists relative glob patterns whose files are skipped by the
  retention check.

JavaScript callers can name this configuration shape with the exported
`QueueJobRetentionKey`, `QueueUnboundedJobRetentionOptions`, and
`QueueUnboundedJobRetentionConfig` types.
See [Unbounded job retention](../rules/unbounded-job-retention.md) and
[`queues check`](../cli/queues-check.md#unbounded-job-retention).

## Queue job state as record of work

`queues.jobStateAsRecord` is opt-in and disabled by default. Set it to `true`
or pass `allowPaths` to emit advisory `job-state-as-record` findings when code
uses queue job state or failed/completed job listings to drive retries or
re-enqueues:

```yaml
queues:
  jobStateAsRecord:
    allowPaths: ["apps/admin/**"]
```

JavaScript callers can name this configuration shape with the exported
`QueueJobStateAsRecordOptions`, `QueueJobStateAsRecordConfig`, and
`QueueCheckSeverity` types.
See [Job state as record](../rules/job-state-as-record.md) and
[`queues check`](../cli/queues-check.md#queue-job-state-as-record-of-work).

Configured Celery, Asynq, Kafka, Active Job, Sidekiq, Laravel, and Symfony
Messenger sites emit the canonical `queue-enqueue` and `queue-worker` edges.
Kafka reads static topic literals from the enqueue and worker globs in
TypeScript, Python, Go, Rust, Ruby, PHP, Java, Kotlin, Elixir, and Dart.
Use [`queues edges`](../cli/queues-edges.md), [`queues related`](../cli/queues-related.md),
and [`queues check`](../cli/queues-check.md) to inspect or validate them.
