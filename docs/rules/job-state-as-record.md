# Job state as record

Advises when code reads ephemeral BullMQ or compatible queue job state to
decide retries, removals, or re-enqueues instead of querying durable database
state.

## Why and when

Queue job state is ephemeral: jobs vanish under retention limits, Redis or
Valkey flushes, or failover. Treating `getJob` state checks or failed/completed
listings as the source of truth couples business state to queue storage. Enable
`queues.jobStateAsRecord` to surface these patterns as report-only advisories.

## What it catches

When enabled under `queues.jobStateAsRecord`, [`queues check`](../cli/queues-check.md#queue-job-state-as-record-of-work)
and aggregate `no-mistakes check` emit an advisory `job-state-as-record`
finding (`severity: "advisory"`) for two patterns on resolved queue bindings:

- `getJob(...)` followed by `getState()`, `isFailed()`, or `isCompleted()`
  whose result guards `retry()`, `remove()`, or a queue `add` / `addBulk` in
  the same function.
- A failed or completed job listing (`getFailed`, `getCompleted`, or
  `getJobs([...])` including `"failed"` or `"completed"`) whose results feed
  `add`, `addBulk`, or `retry`.

Unresolved queue receivers abstain with a diagnostic, and non-queue objects
such as `Set.add` are ignored. Because findings are advisory, `queues check`
still exits `0` when only advisory findings are present, and aggregate
`no-mistakes check` reports them under `advisories`.

## Options and defaults

`queues.jobStateAsRecord` is opt-in and disabled by default (`false` or
omitted). Set it to `true` or pass `allowPaths`:

```yaml
queues:
  jobStateAsRecord:
    allowPaths: ["apps/admin/**"]
```

- `allowPaths`: relative glob patterns skipped by the check (for example,
  operator dashboards). Defaults to `[]`.

### Shared test-file scope

`job-state-as-record` shares the declared queue scope with producer/worker
matching and every other queue check. Files matched by configured `tests.*`
scopes (or discovered Vitest/Playwright configs) are excluded by default, with
no hardcoded filename conventions. To check specific test files or all
configured test files, opt them back into the shared queue scope with
`queues.includeTests` (default `false`):

```yaml
queues:
  includeTests: ["tests/orders.test.ts"]
  jobStateAsRecord: true
```

See [queue configuration](../configuration/queues.md#test-file-scope).

## Valid example

```ts
export async function enqueuePendingOrders(
  db: Database,
  ordersQueue: Queue,
) {
  for (const row of await pendingOrders(db, 100)) {
    await ordersQueue.add("create", { id: row.id });
  }
}
```

## Counterexample

```ts
export async function retryOrder(ordersQueue: Queue, id: string) {
  const job = await ordersQueue.getJob(id);
  if ((await job?.getState()) === "failed") {
    await job.retry();
  }
}
```

## Fix

Read durable work status from the application database to decide what to
enqueue or retry. Exempt operator tooling and dashboards with
`queues.jobStateAsRecord.allowPaths`.

## Suppression

In aggregate `no-mistakes check`, use `no-mistakes-disable-next-line queues-check`,
`no-mistakes-disable-line queues-check`, or `no-mistakes-disable-file queues-check`.

```ts
// no-mistakes-disable-next-line queues-check: operator repair endpoint
const job = await ordersQueue.getJob(id);
```

## Related rules

- [Unbounded job retention](unbounded-job-retention.md) enforces bounded
  `removeOnComplete` and `removeOnFail` retention options.
- [`no-single-enqueue-in-loop`](no-single-enqueue-in-loop.md) reports single
  enqueues inside open loops.
- [`queues check`](../cli/queues-check.md#queue-job-state-as-record-of-work)
  documents all queue check kinds and CLI flags.
