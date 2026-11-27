# `queue-dashboard-reachability`

Configures dashboard queue relationships: a `queue-enqueue` edge and a `queue-worker` edge for a job that has both an enqueue site and a worker.

## Why and when

Use this rule when the dependency graph should show dashboard jobs that are both enqueued and handled. It is separate from project queue producer and worker analysis. It does not emit a `check` finding. Empty options produce no dashboard edges.

## What it requires

Queue facts are extracted only when `queuePattern`, `factorySpecifier`, and `factoryFunction` are all non-empty and `queuePattern` compiles as a glob. The collector also returns no relationships when `queuePattern` or `factorySpecifier` is empty or the glob is invalid.

A queue definition counts when a file matching `queuePattern` calls `factoryFunction` from `factorySpecifier` with a string-literal queue name. An enqueue counts when `<binding>.add("job", ...)` or `addBulk([{ name: "job" }, ...])` names that queue and the job is a string literal. A worker counts when `new Worker("queue-name", ...)` uses that same literal name and the file has a namespace import whose last path segment starts with `processors`. A job becomes edges only when that `(queue, job)` pair appears in both an enqueue and a worker. An enqueue with no worker, a worker with no enqueue, or a dynamic job or queue name produces no dashboard edge.

## Options

Every field defaults to an empty string. Unknown option keys fail configuration validation. Keys such as `entrypoint` and `tsconfig` are not options of this rule.

```yaml
rules:
  - rule: queue-dashboard-reachability
    scope: repository
    options:
      queuePattern: systems/*/queues.mts
      factorySpecifier: "@queues/factory"
      factoryFunction: createQueue
```

| Option | Default | Effect |
| --- | --- | --- |
| `queuePattern` | `""` | Glob of files that create queues. Empty or invalid installs no extractor. |
| `factorySpecifier` | `""` | Module specifier of the queue factory import. |
| `factoryFunction` | `""` | Factory function name, such as `createQueue`, whose string-literal argument is the queue name. |

## Valid example

```ts
import { createQueue } from "@queues/factory";

export const emails = createQueue("emails");
```

```ts
import { emails } from "./queues.mts";

emails.add("sendWelcome", { userId });
```

```ts
import * as processors from "./processors.mts";

new Worker("emails", processors);
```

When `processors.mts` exports a function or const named `sendWelcome`, the graph records `queue-enqueue` from the caller to the job and `queue-worker` from the job to that module. The worker file is also linked when it is not the processors module.

## Counterexample

```ts
emails.add(jobName, payload);
```

```ts
new Worker(queueName, processors);
```

The job and the queue name are not literals, so the dashboard collector records no edge for them. An `add("sendWelcome", payload)` with no matching worker is also omitted: dashboard edges require both sides.

## Fix

Create the queue with a string literal passed to `factoryFunction`, enqueue a string-literal job name, and register a worker with that same queue-name literal and a processors module that handles the job. Set all three options, and make `queuePattern` a valid glob that matches the factory file.

## Suppression

This rule does not emit a line finding. A `no-mistakes-disable-next-line` or `no-mistakes-disable-file` comment does not add a missing dashboard edge or remove a matched enqueue and worker. Change the literals or the options.

## Related rules

[`no-single-enqueue-in-loop`](no-single-enqueue-in-loop.md) reports one configured enqueue inside a per-item loop. Project producer and worker matching stays on [`queues check`](../cli/queues-check.md). Edge kinds are listed in [graph edges](../graph-edges.md).
