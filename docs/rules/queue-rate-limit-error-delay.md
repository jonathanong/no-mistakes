# `queue-rate-limit-error-delay`

Requires a positive delay duration when constructing or calling a configured
rate-limit error, so worker processors do not drop an upstream provider's
`Retry-After` cooldown.

## Why and when

When an upstream API responds with `429 Too Many Requests` and a `Retry-After`
header, throwing a queue rate-limit error without passing the delay drops that
cooldown and lets workers immediately retry into the rate limit. This rule is
opt-in and disabled by default: configure `options.targets` for the queue
package or internal error module whose rate-limit error accepts a delay
duration. (For per-process `worker.rateLimit(ms)` calls and multi-job shared
queue rate limits, see [`queues check`](../cli/queues-check.md#per-process-worker-rate-limit).)

## What it catches

Using the prepared canonical call graph, the rule checks every `new`
construction and direct call that resolves to a configured `(module, export)`
target, including renamed imports, namespace imports (`mq.RateLimitError`),
static factory members (`Worker.RateLimitError`), barrel re-exports
(`export { ... } from` and `export * from`), workspace packages, tsconfig path
aliases, and immutable local `const` aliases.

A call is a definite finding when:

- The argument at `delayArgument` (default `0`) is omitted.
- When `delayProperty` is omitted: the argument at `delayArgument` is proven to
  be `undefined`, `void <expr>`, `null`, `NaN`, `±Infinity`, a boolean, a
  string, an array, an object, or a number `<= 0` (including through immutable
  same-file `const` aliases).
- When `delayProperty` is set: the argument at `delayArgument` is not an object
  (for example `undefined`, `null`, a string, a number, or an array), or it is
  a closed object literal / immutable `const` object that omits `delayProperty`
  or sets it to `undefined`, `void <expr>`, `null`, `NaN`, `±Infinity`, a
  non-number, or a number `<= 0`.

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
          export: RateLimitError
          delayArgument: 0
        - module: glide-mq
          export: Worker.RateLimitError
          delayArgument: 0
        - module: ./src/provider-errors
          export: ProviderRateLimitError
          delayArgument: 1
          delayProperty: retryAfterMs
```

- `targets` (required, non-empty): list of rate-limit error targets to check.
  - `module`: non-empty import specifier or workspace/relative module path.
  - `export`: non-empty exported symbol or dotted static member path (for
    example `RateLimitError` or `Worker.RateLimitError`).
  - `delayArgument`: zero-based argument index (default `0`, must be `< 32`).
  - `delayProperty`: optional non-empty property name when the delay is passed
    inside an options object at `delayArgument`.
- `unanalyzable`: shared analysis policy (`report` default, `advise`, or
  `ignore`). See [Unanalyzable inputs](#unanalyzable-inputs).
- Standard rule filters (`include`, `exclude`, `projects`, `message`) also
  apply.

Invalid options (missing or empty `targets`, blank `module`, `export`, or
`delayProperty`, duplicate `(module, export)` pairs, `delayArgument >= 32`, or
unknown fields) fail at configuration load with exit status `2`.

## Unanalyzable inputs

Following recall over precision ([how analysis works](../../README.md#how-analysis-works)),
inputs the rule cannot prove are never silently treated as clean; they route
through `options.unanalyzable` (default `report`, or `advise` / `ignore`) with
target `unanalyzable`:

- Spread arguments at or before `delayArgument` (`new RateLimitError(...args)`).
- When `delayProperty` is set: an argument at `delayArgument` that is not a
  closed object literal or immutable `const` object (for example
  `new ProviderRateLimitError("429", opts)` or
  `new ProviderRateLimitError("429", { ...opts })`).
- Unresolved callees whose target is unknown and whose candidate targets or
  terminal member name match a configured target when the call arguments are
  not already proven valid (for example `new unknownClient.RateLimitError()`).
- Selected files whose arguments could not be analyzed (for example due to a
  parse failure or dynamic `eval`/`with` scope).

## Valid example

```ts
import { RateLimitError } from "glide-mq";

export async function fetchProvider(retryAfterMs: number) {
  throw new RateLimitError(retryAfterMs);
}
```

## Counterexample

```ts
import { RateLimitError } from "glide-mq";

export async function fetchProvider() {
  throw new RateLimitError();
}
```

## Fix

Pass the upstream provider's positive retry delay (in milliseconds) when
constructing or calling the rate-limit error, or pass a statically visible
options object containing `delayProperty` with a positive delay.

## Suppression

Use `no-mistakes-disable-next-line queue-rate-limit-error-delay`,
`no-mistakes-disable-line queue-rate-limit-error-delay`, or
`no-mistakes-disable-file queue-rate-limit-error-delay` with a brief reason:

```ts
// no-mistakes-disable-next-line queue-rate-limit-error-delay: default queue backoff is intentional here
throw new RateLimitError();
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
