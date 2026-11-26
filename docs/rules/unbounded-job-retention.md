# Unbounded job retention

Reports BullMQ and compatible queue definitions or producers that keep
completed or failed jobs in Redis or Valkey without a bound.

## Why and when

Queue libraries retain completed and failed jobs until told otherwise. Leaving
`removeOnComplete` or `removeOnFail` unset, or setting either key to `false`,
lets job records grow without bound in memory. Enable
`queues.unboundedJobRetention` to enforce bounded retention in production queue
code while storing durable work records in the database.

## What it catches

When enabled under `queues.unboundedJobRetention`, [`queues check`](../cli/queues-check.md#unbounded-job-retention)
and aggregate `no-mistakes check` report `unbounded-job-retention` for:

- `removeOnComplete: false` or `removeOnFail: false` in a queue's
  `defaultJobOptions`, at a producer site (`add`, `addBulk`, flow nodes,
  configured enqueue factories), or in an `upsertJobScheduler` template.
- A producer whose effective options (call options merged over queue
  `defaultJobOptions`) leave a required retention key unset.

`true`, a non-negative number, or an object with a `count` or `age` property
counts as bounded. The check emits a not-judged warning at the producer
when a required retention value is unknown or a spread or computed property
leaves that value uncertain. An explicit value after a spread can still prove
the retention setting. A producer on a locally constructed queue reports
explicit unbounded call options and uncertain call options even when the queue
name is dynamic. If a call on that queue omits a required key, the check
reports that the defaults could not be established. Definite violations are
still reported even when another required key at the same site is uncertain. When both
`queues.requiredJobOptions` and `queues.unboundedJobRetention` cover the same
key on a producer, `unbounded-job-retention` reports it once so the site is not
duplicated.

## Options and defaults

`queues.unboundedJobRetention` is opt-in and disabled by default (`false` or
omitted). Set it to `true` to require both `removeOnComplete` and
`removeOnFail`, or pass an object:

```yaml
queues:
  unboundedJobRetention:
    require: [removeOnComplete, removeOnFail]
    allowPaths: ["scripts/**"]
```

- `require`: subset of `[removeOnComplete, removeOnFail]`. Defaults to both
  keys when enabled.
- `allowPaths`: relative glob patterns skipped by the retention check. Defaults
  to `[]`.

### Shared test-file scope

`unbounded-job-retention` shares the declared queue scope with producer/worker
matching and every other queue check. Files matched by configured `tests.*`
scopes (or discovered Vitest/Playwright configs) are excluded by default, with
no hardcoded filename conventions. To check specific test files or all
configured test files, opt them back into the shared queue scope with
`queues.includeTests` (default `false`):

```yaml
queues:
  includeTests: ["tests/orders.test.ts"]
  unboundedJobRetention: true
```

See [queue configuration](../configuration/queues.md#test-file-scope).

## Valid example

```ts
import { Queue } from "bullmq";

export const ordersQueue = new Queue("orders", {
  defaultJobOptions: {
    removeOnComplete: true,
    removeOnFail: { age: 86_400, count: 1_000 },
  },
});

export async function enqueueOrder(id: string) {
  await ordersQueue.add("create", { id });
}
```

## Counterexample

```ts
import { Queue } from "bullmq";

export const ordersQueue = new Queue("orders");

export async function enqueueOrder(id: string) {
  await ordersQueue.add("create", { id });
}
```

## Fix

Set `removeOnComplete` and `removeOnFail` to `true`, a non-negative count, or
an `{ age, count }` object in `defaultJobOptions` or at the producer call site.
Keep durable work state in the database rather than in completed or failed
queue entries. Exempt operator scripts with
`queues.unboundedJobRetention.allowPaths`.

## Suppression

In aggregate `no-mistakes check`, use `no-mistakes-disable-next-line queues-check`,
`no-mistakes-disable-line queues-check`, or `no-mistakes-disable-file queues-check`.

```ts
// no-mistakes-disable-next-line queues-check: one-off migration script
await ordersQueue.add("backfill", { id });
```

## Related rules

- [Job state as record](job-state-as-record.md) reports code that treats
  ephemeral queue job state as a durable work log.
- [`no-single-enqueue-in-loop`](no-single-enqueue-in-loop.md) reports single
  enqueues inside open loops.
- [`queues check`](../cli/queues-check.md#unbounded-job-retention) documents
  all queue check kinds and CLI flags.
