# `queue-rate-limit-error-delay`

Requires a positive delay duration when constructing or calling a configured
rate-limit error, so worker processors do not drop an upstream provider's
`Retry-After` cooldown.

## Why and when

When an upstream API responds with `429 Too Many Requests` and a `Retry-After`
header, throwing a queue rate-limit error without supplying that delay drops the
provider's cooldown and falls back to the worker's default limiter window or
immediate retries. This rule is opt-in and disabled by default: configure
`options.targets` for the queue package or internal error module your workers
throw. (For per-process `worker.rateLimit(ms)` calls and multi-job shared queue
rate limits, see [`queues check`](../cli/queues-check.md#per-process-worker-rate-limit).)

In `glide-mq` (0.20.0), `Worker.RateLimitError` and
`BroadcastWorker.RateLimitError` are no-argument constructors (`new Worker.RateLimitError()`)
whose handler reads `error.delayMs || (limiter.duration ?? 1000)`, while
`GroupRateLimitError(delayMs, opts?)` is a root export taking `delayMs` at
argument `0`. Configure `instanceDelayProperty: delayMs` for
`Worker.RateLimitError` and default positional `delayArgument: 0` for
`GroupRateLimitError`.

## What it catches

Using the prepared canonical call graph, the rule checks every `new`
construction and direct call that resolves to a configured `(module, export)`
target, including renamed imports, namespace imports (`mq.Worker.RateLimitError`,
`mq.GroupRateLimitError`), static members (`Worker.RateLimitError`), barrel
re-exports (`export { ... } from` and `export * from`), workspace packages,
tsconfig path aliases, and immutable local `const` aliases.

A call is a definite finding when:

- **Instance-property targets (`instanceDelayProperty` set):** the constructed
  instance is thrown (or discarded) without attaching `instanceDelayProperty`
  via `Object.assign(new Target(), { delayMs })` or an unconditional same-block
  assignment (`err.delayMs = ...` / `Object.assign(err, { delayMs })`), or sets
  `instanceDelayProperty` to `undefined`, `void <expr>`, `null`, `NaN`,
  `±Infinity`, a boolean, a string, an array, an object, or a number `<= 0`.
- **Positional argument targets (`instanceDelayProperty` and `delayProperty`
  omitted):** the argument at `delayArgument` (default `0`) is omitted or proven
  to be `undefined`, `void <expr>`, `null`, `NaN`, `±Infinity`, a boolean, a
  string, an array, an object, or a number `<= 0` (including through immutable
  same-file `const` aliases).
- **Options-object targets (`delayProperty` set):** the argument at
  `delayArgument` is omitted, is not an object (for example `undefined`, `null`,
  a string, a number, or an array), or is a closed object literal / immutable
  `const` object that omits `delayProperty` or sets it to `undefined`,
  `void <expr>`, `null`, `NaN`, `±Infinity`, a non-number, or a number `<= 0`.

A positive finite numeric literal (such as `5000`) or a dynamic delay
expression (such as `retryAfterMs` or `parseRetryAfter(headers)`) satisfies the
delay check. Unrelated local error classes and imports from non-configured
modules are ignored.

## Options and defaults

The rule is disabled until added under `rules:`. There are no hardcoded package
or error class names; every target must be declared in `options.targets`:

```yaml
rules:
  - rule: queue-rate-limit-error-delay
    scope: repository
    options:
      targets:
        - module: glide-mq
          export: Worker.RateLimitError
          instanceDelayProperty: delayMs
        - module: glide-mq
          export: BroadcastWorker.RateLimitError
          instanceDelayProperty: delayMs
        - module: glide-mq
          export: GroupRateLimitError
          delayArgument: 0
        - module: ./src/provider-errors
          export: ProviderRateLimitError
          delayArgument: 1
          delayProperty: retryAfterMs
```

- `targets` (required, non-empty): list of rate-limit error targets to check.
  - `module`: non-empty import specifier or workspace/relative module path.
  - `export`: non-empty exported symbol or dotted static member path (for
    example `Worker.RateLimitError` or `GroupRateLimitError`).
  - `instanceDelayProperty`: optional non-empty property name expected on the
    constructed error instance (for no-argument constructors such as
    `glide-mq`'s `Worker.RateLimitError`, where the worker reads
    `error.delayMs`). Mutually exclusive with `delayArgument` and
    `delayProperty`.
  - `delayArgument`: zero-based argument index (default `0` when
    `instanceDelayProperty` is not set; must be `< 32`).
  - `delayProperty`: optional non-empty property name when the delay is passed
    inside an options object at `delayArgument`.
- `unanalyzable`: shared analysis policy (`report` default, `advise`, or
  `ignore`). See [Unanalyzable inputs](#unanalyzable-inputs).
- Standard rule filters (`include`, `exclude`, `projects`, `message`) also
  apply.

Invalid options (missing or empty `targets`, blank `module`, `export`,
`delayProperty`, or `instanceDelayProperty`, combining `instanceDelayProperty`
with `delayArgument` or `delayProperty`, duplicate `(module, export)` pairs,
`delayArgument >= 32`, or unknown fields) fail at configuration load with exit
status `2`.

## Unanalyzable inputs

Following recall over precision ([how analysis works](../../README.md#how-analysis-works)),
inputs the rule cannot prove are never silently treated as clean; they route
through `options.unanalyzable` (default `report`, or `advise` / `ignore`) with
target `unanalyzable`:

- When `instanceDelayProperty` is set: `Object.assign(new Target(), opts)` or
  `Object.assign(new Target(), { ...opts })` with a non-closed source object, a
  conditional or nested assignment (`if (cond) err.delayMs = 5000`), or a
  constructed instance that escapes to a helper or caller (`attachDelay(err)` or
  `return new Worker.RateLimitError()`).
- Spread arguments at or before `delayArgument`
  (`new GroupRateLimitError(...args)`).
- When `delayProperty` is set: an argument at `delayArgument` that is not a
  closed object literal or immutable `const` object (for example
  `new ProviderRateLimitError("429", opts)` or
  `new ProviderRateLimitError("429", { ...opts })`).
- Unresolved callees whose target is unknown and whose terminal member name or
  imported binding matches a configured target when the call is not already
  proven valid (for example `new unknownClient.GroupRateLimitError()`).
- Selected files whose arguments could not be analyzed (for example due to a
  parse failure or dynamic `eval`/`with` scope).

## Valid example

```ts
import { GroupRateLimitError, Worker } from "glide-mq";

export async function fetchProvider(retryAfterMs: number): Promise<never> {
  throw Object.assign(new Worker.RateLimitError(), { delayMs: retryAfterMs });
}

export async function fetchGroup(retryAfterMs: number): Promise<never> {
  throw new GroupRateLimitError(retryAfterMs);
}
```

## Counterexample

```ts
import { Worker } from "glide-mq";

export async function fetchProvider(): Promise<never> {
  throw new Worker.RateLimitError();
}
```

## Fix

For no-arg error constructors like `glide-mq`'s `Worker.RateLimitError`, attach
the upstream provider's positive `delayMs` via
`Object.assign(new Worker.RateLimitError(), { delayMs: retryAfterMs })` (or
assign `delayMs` unconditionally on the local error variable before throwing).
For constructors that accept a delay argument such as `GroupRateLimitError`,
pass the positive delay directly (or inside the configured `delayProperty`
options object).

## Suppression

Use `no-mistakes-disable-next-line queue-rate-limit-error-delay`,
`no-mistakes-disable-line queue-rate-limit-error-delay`, or
`no-mistakes-disable-file queue-rate-limit-error-delay` with a brief reason:

```ts
// no-mistakes-disable-next-line queue-rate-limit-error-delay: default queue backoff is intentional here
throw new Worker.RateLimitError();
```

## Related rules

- [`queues check`](../cli/queues-check.md#per-process-worker-rate-limit)
  reports `worker-rate-limit-per-process` for `worker.rateLimit(ms)` calls,
  `shared-queue-rate-limit` for rate limits on multi-job queues, and
  `debounce-ttl` for dead `ttl` values in debounce-mode `deduplication`.
- [`no-single-enqueue-in-loop`](no-single-enqueue-in-loop.md) reports single
  enqueues inside open loops.
- [Unbounded job retention](unbounded-job-retention.md) enforces bounded job
  retention.

CLI: `no-mistakes check --root . --format json`. Node API: `check()` and
`analyzeProject()`.
