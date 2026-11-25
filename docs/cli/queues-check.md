# `no-mistakes queues check`

Check queue producers, workers, and the enqueue calls those workers make.

```sh
no-mistakes queues check --format json
no-mistakes queues check --format json --include-diagnostics
```

JSON and YAML print a findings array by default. With
`--include-diagnostics`, they return `{ "findings": [...], "diagnostics": [...] }`.
Markdown adds a `## diagnostics` section when the flag is set and diagnostics
exist. Human and paths stdout contain findings only.

Both `queues check` and aggregate `check` print unresolved worker
`.rateLimit()` receiver warnings, untraceable `RateLimitError` abstentions, and
the retention and listing abstentions described below to stderr by default. The
standalone flag
prints all queue diagnostics to stderr in every format, including human and
paths, so topology notes are available without altering those stdout streams.
Aggregate `check` includes these warnings in JSON `warnings` too.
Diagnostics leave the exit status unchanged.

A `.add(...)` or `.addBulk(...)`
call on a `const` local initialized to a provable non-queue JavaScript built-in
(`new Set(...)`, `new Map(...)`, `new WeakSet(...)`, `new WeakMap(...)`, an
array literal, or a primitive/function literal) is ignored and never produces a
`dynamic or unresolved queue producer` diagnostic. Function parameters,
configured enqueue wrappers whose queue does not resolve, and imported names
(including an imported `Set`) still emit `dynamic or unresolved queue producer`.

Use this before finishing queue edits so every configured enqueue/worker path is
connected. `check` is empty when the graph is consistent. For explicitly configured enqueue wrappers, object-argument selectors,
fixed-queue job definitions, and `optionsOnReturnedCall` feed these same
checks. Returned-call options are judged at the call site: invalid `jobId`,
mixed priorities, debounce TTL, and required job options are not lost at the
factory declaration. Wrapper defaults are applied before per-call overrides.
Unresolved properties, spread definitions, and mutable bindings do not invent
producer names or worker edges. See
[factory configuration](../configuration/queues.md#configured-enqueue-selectors-and-returned-callables).

Findings use these
`kind` values:

| `kind` | Meaning |
| --- | --- |
| `unmatched-producer` | A production job is enqueued and no production worker handles it. Files matched by configured `tests.*` scopes are excluded by default; see [Producer and worker matching](#producer-and-worker-matching). |
| `unmatched-worker` | A production worker handles a job that no production producer enqueues. Files matched by configured `tests.*` scopes are excluded by default; see [Producer and worker matching](#producer-and-worker-matching). |
| `unmatched-worker-unresolved-producers` | A producer for this handler did not resolve, so the handler is not known to be dead. The message names those producer lines. A producer with neither a resolved queue nor a resolved job is listed on every unmatched worker. |
| `self-dedup-enqueue` | A processor enqueues its own job on its own queue with `deduplication.id` or `jobId` taken from the active job. The job binding is a parameter named `job`, or otherwise the processor's first parameter: `task.id` / `task.deduplicationId`, or a destructured `{ name, id }`. A nested callback is not that binding unless one of its parameters is named `job`. A local function or `const` callback passed by name is tied to its configured worker even when declared earlier. A copied `const id = task.id` or `const id = task.deduplicationId` remains a job token within its lexical scope, as does a destructured `const { id } = task` or `const { deduplicationId } = task`. A defaulted binding such as `const { deduplicationId: dedup = "fallback" } = task` is not a token. Unrelated values and shadowed parameters do not. A configured enqueue factory does the same check on its `options` argument. The queue drops that add while the job is active. Publish a new id, or enqueue a different job. The same job name on a different queue is not a finding. If the worker queue or the target queue does not resolve, the check abstains and emits a diagnostic. |
| `enqueue-and-wait` | Waiting on a job result holds a worker or request slot. Return control instead of blocking on the result. The check covers calls to `addAndWait`, `waitUntilFinished`, and explicitly configured wrappers. |
| `shared-queue-rate-limit` | A processor throws a queue package's `RateLimitError` (including renamed, namespace, or local barrel re-export imports and `Worker.RateLimitError()`) or calls `.rateLimit()` / `["rateLimit"]()` on its worker for a queue whose producers enqueue more than one job name. That limit stalls every job on the queue. Move that job kind to its own queue, or use per-group rate limits. One known job name is not a finding. A producer on that queue with no resolved job name, a worker queue that does not resolve, or a `RateLimitError` throw that cannot be traced to a queue package or a local non-queue declaration abstains with a diagnostic instead of a finding. A local processor shared by multiple worker instances is checked for each queue; a helper outside a worker processor or a `.rateLimit()` call on an unrelated receiver is not a finding. See [Shared queue rate limit](#shared-queue-rate-limit). |
| `worker-rate-limit-per-process` | `.rateLimit(...)` (or `["rateLimit"](...)`) is called on a binding that resolves to a `Worker` / `TestWorker` from `bullmq` or `glide-mq`, a subclass of one, or a configured `queues.factories.worker` factory. A worker-level rate limit delays jobs inside one worker process only, so N processes run at N times the intended rate. Use the queue-level limiter or a provider-side wait instead. Unresolved receivers whose binding originates from a queue module or worker construction abstain with a diagnostic; unrelated receivers are not reported. On a multi-job queue, calling `worker.rateLimit()` inside a processor emits both `shared-queue-rate-limit` and `worker-rate-limit-per-process`. See [Per-process worker rate limit](#per-process-worker-rate-limit). |
| `enqueue-and-wait-declaration` | An explicitly matched wait-wrapper declaration is forbidden by `queues.factories.forbidEnqueueAndWaitDeclarations`. Replace it with enqueue-only behavior; the fix hint is included in `message`. Disabled by default. |
| `debounce-ttl` | `deduplication: { mode: "debounce", ttl }` sets a `ttl` that debounce mode ignores (only throttle mode reads `ttl`). Checked on `queue.add`, each `queue.addBulk` item's `opts`, `FlowProducer` root and child `opts`, queue `defaultJobOptions`, and configured enqueue factories/wrappers, following TypeScript wrappers, no-substitution template literals, inline spreads, and same-file `const` bindings. When `deduplication` has a dynamic mode alongside `ttl`, `mode: "debounce"` alongside an unknown spread, or an unresolved `deduplication` object expression, the check abstains with a warning diagnostic (`debounce-ttl was not judged because deduplication options are not statically known`). Remove `ttl`, or use a mode that reads it. |
| `invalid-job-id` | A custom `jobId` literal, or a static part of a template literal, contains `:`, `{`, `}`, or a control character. `:` is the BullMQ and glide-mq key separator, so an id containing it can collide with internal keys. `{` and `}` collide with Redis or Valkey cluster hash tags. Interpolations such as `${id}` are not judged, and neither is a fully dynamic id. Use `-` or `_` as the separator. The check resolves a `const` job ID or label from its lexical binding. If the label is dynamic, the finding omits the job name. The last explicit `jobId` supplies the ID. The check abstains if a later spread or computed property could override that ID. The check reads `jobId` on proven queue receivers and configured producers, including `add` options, each `addBulk` item's `opts`, flow-node `opts` (including children), `upsertJobScheduler` template `opts`, and a configured enqueue factory's options argument. |
| `scheduler-template-option` | A job-scheduler template sets `delay`, `deduplication`, `parent`, or `jobId`. The scheduler's repeat settings control timing and identity, so the call does not do what that option reads as. Remove the option from the template. The check reads literal keys on the template and inside its `opts`, including a spread of a same-file `const` object literal. The last `opts` supplied by an explicit property or a known spread determines which options are checked. A module `const` is visible to functions declared earlier in the file. An unknown or cyclic spread, a non-literal key, or a binding that is not a `const` object literal makes that call abstain. Keys nested under `data` or any other object are not options. Built-in `upsertJobScheduler` and a literal computed `queue["upsertJobScheduler"]` read the third argument. `queues.factories.jobScheduler` reads the configured `template` argument on calls to the named import, including its local alias. |
| `empty-glob` | A configured `enqueues` or `workers` glob matches no file. Fix the glob or remove it. An empty list still disables that side and is not a finding. |
| `queue-cycle` | A worker queue enqueues another queue that eventually enqueues it back. Only an enqueue inside that worker's processor counts. Two workers in one file do not share each other's enqueues. Review the loop before shipping; this check has no terminator allowlist. |
| `mixed-priority` | Producers on one queue enqueue both priority 0 and a priority greater than 0. See [Mixed priority](#mixed-priority). |
| `missing-job-option` | A producer omits one or more keys from [`queues.requiredJobOptions`](../configuration/queues.md#required-job-options). One finding lists every missing key in configuration order. `message` is `job options missing {keys}`. |
| `unbounded-job-retention` | A queue `defaultJobOptions` or producer keeps completed or failed jobs without a bound under [`queues.unboundedJobRetention`](../configuration/queues.md#unbounded-job-retention). |
| `job-state-as-record` | Advisory (`severity: "advisory"`). Code reads ephemeral queue job state (`getJob` -> `getState`/`isFailed`/`isCompleted` guarding `retry`/`remove`/`add`, or `getFailed`/`getCompleted`/`getJobs` feeding `add`/`addBulk`/`retry`) under [`queues.jobStateAsRecord`](../configuration/queues.md#queue-job-state-as-record-of-work). Report-only: does not fail `queues check` and moves to `advisories` in aggregate `check`. |

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

The scheduler template check takes same-file binding declarations from the
queue processor index, which already visits the file to resolve named
callbacks. It groups those declarations by name on the first binding lookup.

## Producer and worker matching

`unmatched-producer`, `unmatched-worker`, `unmatched-worker-unresolved-producers`,
and `queues edges` match producers and workers across the project:

- **Configured test scopes exclude test files across every queue rule by default:** Files
  matched by declared `tests.*` scopes in `.no-mistakes.yml` (or a declared
  Vitest/Playwright config) still participate in producer/worker matching, but
  every queue rule (`unmatched-producer`, `unmatched-worker`,
  `unmatched-worker-unresolved-producers`, `unbounded-job-retention`,
  `job-state-as-record`, `missing-job-option`, `invalid-job-id`,
  `debounce-ttl`, `scheduler-template-option`, `enqueue-and-wait`,
  `queue-cycle`, `self-dedup-enqueue`, `shared-queue-rate-limit`,
  `worker-rate-limit-per-process`, and `mixed-priority`) skips findings and
  rule-specific diagnostics on test-scope files, and test-scope files are
  omitted from `queues edges`. No hardcoded filename conventions (`*.test.*`,
  `__tests__`) are applied when no test scope is declared. Set
  [`queues.includeTests`](../configuration/queues.md#test-file-scope) to `true`
  or a list of relative globs to explicitly include test files back in the
  shared queue scope.
- **Test-file producers without workers:** A producer inside a configured test
  scope is not reported as `unmatched-producer` even when no worker exists
  anywhere, because integration and harness tests commonly enqueue synthetic
  jobs against partial fixtures.
- **Same-file raw `new Worker` and configured worker factories:** A raw
  `new Worker(<queueName>, ...)` (or `new TestWorker(...)`) or a configured
  `queues.factories.worker` call (including namespace calls and options such as
  `{ prefix }`) counts as a matching worker for producers on the same queue name,
  including when the file creates its own `new Queue(<queueName>, ...)` or
  `createQueue(<queueName>, ...)` handle, aliases an imported queue-name
  constant, or passes `queue.name`. Module-level non-test queue bindings win
  canonical queue ownership over function-local handles and test files.
- **Workers with unresolved queue names abstain only when scoped to the queue:**
  When a worker's queue name does not resolve, it could handle the job
  (wildcard or matching job name), and either the worker passed an unresolved
  imported queue name (`queue_name_import`) or the worker's file reaches the
  producer's queue definition file, `unmatched-producer` abstains for that
  producer and emits a warning diagnostic (`unmatched-producer was not judged
  because unresolved workers at <file>:<line> could handle this queue`)
  alongside the worker's `dynamic or unresolved queue worker` diagnostic.
  Unrelated fully dynamic wildcard workers (such as a parameter-named
  `new Worker(name, ...)` helper or `process.env.X` worker whose file does not
  reach the producer's queue definition) do not suppress `unmatched-producer`.

## Mixed priority

BullMQ and glide-mq keep jobs with a priority and jobs with no priority in
separate structures. BullMQ processes priority 0 before any priority greater
than 0, so prioritized jobs can wait behind priority 0. Mixing the two forms
makes that order apply across producers.

The check reports each producer of the smaller form and names the producers of
the other form. When the two forms have the same count, it reports both forms.
Each finding's `message` names the other producers.

It reads `queue.add`'s third argument, an `addBulk` item's `opts`, a flow
job's `opts`, and the `options` argument of a configured enqueue factory. A
missing priority, a literal `0`, and a same-file `const` whose value is `0`
are priority 0. This classification also applies when the queue supplies
a positive `defaultJobOptions.priority`, so the check can report a mix that
the queue's defaults would avoid. A finite integer greater than 0, including a
same-file `const` and unary `+`, is the other form. Parentheses and `as const` on that integer
stay static. A `const` later in the same scope still counts, including
`for (const level = 9;;)`. A parameter, `let`, `var`, `for` header, or other
binding with the same name hides the outer `const` for the scope that binding
owns. A `var` inside a class static block does not. A `for...of` or `for...in`
header hides that name on the right-hand side too. A class expression's name
hides an outer `const` inside the class and not after it. A named function
expression and `export default function` hide that function's name the same
way.

The check abstains for the whole queue when any producer priority is not one
of those two forms. That includes a trailing spread, a spread before the
options argument, an `addBulk` argument that is not an array of object
literals, an `addBulk` object item with no literal `name` (the whole queue
abstains, including when the other producers are in another file), an unresolved
computed key that may overwrite `priority` or `opts`, a non-integer, a negative
number, a `let` or `var` binding, an imported number, and any other dynamic
expression. Language queue producers lack static priority evidence. A flow
root with a known queue name joins direct producers when that queue has one
definition file in the shared queue scope. Definitions in configured test
files count only when `queues.includeTests` includes them. Multiple in-scope
definition files for that name make the check abstain and emit a not-judged
warning at each affected enqueue, including direct `Queue` calls and flow
enqueues. Flow `children` are outside this check. An unresolved queue is not
grouped with a resolved one. Give every producer on the queue priority 0, or give every producer a
priority greater than 0.

## Required job options

Use `no-mistakes queues check --root <project> --config <path>` to select a
configuration file. Relative paths are resolved from the project root.
When `--config` is omitted, the project configuration is discovered as usual.

`queues.requiredJobOptions` lists option keys every producer must set.
The default is `[]`, and an empty list leaves this command's output unchanged.
`queues.requiredJobOptionsByQueue` replaces that list for one queue name,
including `[]` to exempt the queue. The check reads `queue.add` options,
each `queue.addBulk` element's `opts`, the `opts` field of an
`upsertJobScheduler` template, `opts` on a flow root and its children, and
configured enqueue factories. A factory without a configured `options`
argument can still satisfy required keys through queue defaults. Keys
present on the resolved queue definition's `defaultJobOptions` count as set.
Queue constructors and configured queue factories follow same-file const
object literals and their const aliases, including when spread into the queue
options. A later known
`defaultJobOptions` replaces the earlier member. A known spread without that
member leaves it intact. A later known write can replace an earlier unknown
spread or computed key. A later unknown spread or computed key can overwrite
the known member and makes the defaults unresolved. A flow node folds
a spread of a statically known object the same way `queue.add` folds its
options object, including a const object and a const alias of one. Each
identifier resolves to the declaration visible at its use. Identifier
spreads stay in source order: a later known `opts` object replaces earlier
keys, and a later object without `opts` does not erase them. A later known
`queueName` sets the queue used for per-queue overrides. A spread that may
replace `queueName` leaves it unknown until a later known value resolves it.
Unresolved spreads and dynamic computed keys can prevent the check from
judging the node and its children. The check stops at a repeated child in a
cyclic flow and can still report findings on children it already reached.
A spread before the options argument makes the check abstain on that call.

Direct calls to `.add`, `.addBulk`, and `.upsertJobScheduler` require a resolved
queue receiver. When multiple constructions share a queue name, the check uses
a resolved definition's defaults if that file has one construction of the
queue. Calls that identify a queue only by name, such as flow nodes, abstain
if multiple constructions could supply the defaults. Explicit options can
still satisfy every required key.

An unknown spread, call, or computed key makes the check abstain. With
`--include-diagnostics`, the queue report includes the reason, such as
`job options were not judged because the options object is not statically
known`. The diagnostic can also identify unknown `defaultJobOptions` or an
unresolved queue name. The queue-name warning is emitted only when a
per-queue override is configured, because that override might exempt the
queue. A call that already sets every required key is not abstained when
`defaultJobOptions` itself is open.

## Shared queue rate limit

The processor can throw a queue package's `RateLimitError` through a named,
renamed, or namespace import, through `Worker.RateLimitError()`, or through
local barrel re-exports (named re-exports, `export *`, namespace re-exports,
default exports, and `const` aliases). A direct or literal computed `rateLimit`
call must resolve to the associated worker. A configured worker factory can
also pass a parameter typed as an imported `Worker` or `TestWorker`. A local
processor shared by multiple worker instances is checked for each queue. Calls
on a different receiver, such as an API client that exposes its own `rateLimit`
method, and throws of a proven local non-queue `RateLimitError` declaration do
not trigger this finding. A `RateLimitError` throw on a multi-job, unknown-job,
or unresolved queue that cannot be traced to a queue package or a local
non-queue declaration (such as an unresolvable import, an external non-queue
package, or an opaque expression) abstains with a warning diagnostic instead of
passing silently.

## Per-process worker rate limit

`worker-rate-limit-per-process` reports `.rateLimit(...)` calls on a receiver
that resolves to `new Worker(...)`, `new TestWorker(...)` (from `bullmq` or
`glide-mq`), a local or exported subclass of `Worker` / `TestWorker` (including
multi-level inheritance), or a configured `queues.factories.worker` factory,
including immutable local aliases and cross-file exports or re-exports. Calling
`worker.rateLimit(ms)` delays jobs inside one worker process only, so N
processes run at N times the intended rate.

Example (reported):

```ts
import { Worker } from "bullmq";

const worker = new Worker("emails", processEmail);
await worker.rateLimit(1000);
```

Counterexample (not reported):

```ts
import { Worker } from "bullmq";

export const worker = new Worker("emails", processEmail, {
  limiter: { max: 10, duration: 1000 },
});
```

Calls on known non-worker receivers (such as a custom `new RateLimiter()`, a
plain class with no `Worker` superclass, a static call on a class constructor,
an object literal, or a queue instance) are not reported and emit no
diagnostic.

Abstention warnings are narrowed to receivers whose binding originates from a
queue module or worker construction: an unresolvable export or a helper call
from a queue module (not configured under `queues.factories.worker`), a mutated
local variable initialized with or assigned a worker, or a subclass with an
unresolvable base inside a queue module. Those warnings appear on
`QueueReport.diagnostics` and the CLI diagnostic output described above.
Unrelated receivers — function parameters, ambient globals,
arbitrary property chains (`x.y.rateLimit()`), external non-queue limiters, and
imports from modules skipped by the queue source filter — emit neither a
finding nor a diagnostic.

On a multi-job queue, calling `worker.rateLimit(...)` inside a worker processor
reports both `shared-queue-rate-limit` and `worker-rate-limit-per-process` on
that line. Neither suppresses the other because they describe separate defects
with distinct fixes: `shared-queue-rate-limit` warns that rate-limiting a
multi-job queue stalls unrelated job kinds (split the queue or use per-group
limits), while `worker-rate-limit-per-process` warns that `worker.rateLimit()`
only pauses one worker process (use a queue-level limiter or provider-side
wait). Fixing only one leaves the other active.

Fix: configure the queue-level limiter option on the worker/queue or use a
provider-side wait instead of `worker.rateLimit(ms)`.

Suppression (applies on aggregate `no-mistakes check`):

```ts
// no-mistakes-disable-next-line queues-check
await worker.rateLimit(1000);
```

## Unbounded job retention

BullMQ and compatible libraries keep every completed and failed job in Redis or
Valkey unless told otherwise. Opt into `queues.unboundedJobRetention` to report:

- `removeOnComplete: false` or `removeOnFail: false` in a queue's
  `defaultJobOptions`, at a producer site (`add`, `addBulk`, flow nodes,
  configured enqueue factories), or in an `upsertJobScheduler` template.
- A producer whose resolved options (call options over queue defaults) leave a
  required retention key unset.

`true`, a non-negative number, or an object with a `count` or `age` property
counts as bounded. The check emits a not-judged warning at a producer when
a required retention value is unknown or an options spread or computed key
leaves it uncertain. A missing key also produces a not-judged warning when
the corresponding queue default is unknown or ambiguous. An explicit
unbounded override still produces a finding.

With both `queues.requiredJobOptions` and `queues.unboundedJobRetention`
enabled, shared retention keys use the `unbounded-job-retention` finding
kind. `missing-job-option` covers other required keys and takes back the
shared keys on files exempted by retention `allowPaths`.

```ts
// reported: failed jobs are kept forever
const widgets = new Queue("widgets", {
  defaultJobOptions: { removeOnComplete: true, removeOnFail: false },
});

// reported: nothing bounds failed jobs, and the queue has no bounded default
await bare.add("build", data, { removeOnComplete: { count: 100 } });

// accepted: both bounded by count or age
const bounded = new Queue("bounded", {
  defaultJobOptions: {
    removeOnComplete: true,
    removeOnFail: { age: 86_400, count: 1_000 },
  },
});
```

Fix: set `removeOnComplete` and `removeOnFail` to `true`, a count, or an
`{ age, count }` object in `defaultJobOptions` or at the producer call site,
and store durable work records in the database. Exempt operator tooling or
one-off scripts with `queues.unboundedJobRetention.allowPaths`, or suppress a
single call with `// no-mistakes-disable-next-line queues-check`. See also
[Unbounded job retention](../rules/unbounded-job-retention.md).

## Queue job state as record of work

Queue job state is ephemeral: jobs disappear under retention limits, Redis or
Valkey flushes, or failover. Opt into `queues.jobStateAsRecord` to emit an
advisory, report-only `job-state-as-record` finding (`severity: "advisory"`)
for two patterns on resolved queue bindings:

- `getJob(...)` followed by `getState()`, `isFailed()`, or `isCompleted()`
  whose result guards `retry()`, `remove()`, or a queue `add` / `addBulk` in
  the same function.
- A failed or completed job listing (`getFailed`, `getCompleted`, or
  `getJobs([...])` including `"failed"` or `"completed"`) whose results feed
  `add`, `addBulk`, or `retry`. An enqueue inside a loop or listing callback
  qualifies when an argument uses the listed item, directly or through a local
  `const` alias, including an object or array containing the item or one of
  its properties. An enqueue that uses a call result or destructured alias
  derived from the item emits a not-judged warning when that alias is the
  only evidence. A `retry()` on a destructured listing binding also qualifies,
  including when the binding has a default.

Unresolved queue receivers abstain with a diagnostic (`job-state-as-record was
not judged because the queue receiver did not resolve`) and non-queue action
objects (such as `Set.add`) are ignored. Because the finding is advisory,
`queues check` still exits `0` when only advisory findings are present, and
aggregate `no-mistakes check` reports it under `advisories`.

```ts
// reported (advisory): the queue is the record of whether the work is done
const job = await widgets.getJob(id);
if ((await job?.getState()) === "failed") await job.retry();

// reported (advisory): re-enqueue driven by the failed set
for (const job of await widgets.getFailed(0, 100)) {
  await widgets.add(job.name, job.data);
}

// accepted: the database decides what is pending
for (const row of await pendingWidgets(db, 100)) {
  await widgets.add("build", { id: row.id });
}
```

Fix: query durable application state from the database to decide what to
enqueue or retry. Exempt operator tooling and dashboards with
`queues.jobStateAsRecord.allowPaths`, or suppress a single location with
`// no-mistakes-disable-next-line queues-check`. See also
[Job state as record](../rules/job-state-as-record.md).

## Suppression

`queues-check` directives apply to findings and the default queue check
warnings in aggregate `no-mistakes check` and Node `check()`. Use
`no-mistakes-disable-file queues-check` for a whole file,
`no-mistakes-disable-line queues-check` for the current line, or
`no-mistakes-disable-next-line queues-check` for the following line. The rule
id is required. Aggregate `--include-suppressed` records each suppressed
warning in the suppression ledger. Standalone `queues check` and Node
`queueCheck()` return the raw analysis.

```ts
// no-mistakes-disable-file queues-check: unmatched topology is intentional
// no-mistakes-disable-next-line queues-check
```

Node API: `queueCheck(options)`.

For explicitly declared JSON payload schemas, configure
[declared-payload-compatibility](../rules/declared-payload-compatibility.md) and
run `no-mistakes check`. Declaration compatibility is a separate opt-in check.

Queue producer and worker arrays are sorted by file and line in both standalone
and aggregate programmatic reports, so repeated analyses retain identical order.

Configured returned enqueue callables inside workers also preserve the
`self-dedup-enqueue` check when they reuse the active job's id.
