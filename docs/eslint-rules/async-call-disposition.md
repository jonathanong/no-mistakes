# `no-mistakes/async-call-disposition`

## Why

Configured async boundaries, such as enqueue or scheduling helpers, must make
their promise disposition explicit. A bare call can silently lose a rejection.

## Disallowed

```ts
enqueueEmail(user.id);
```

## Allowed

```ts
await enqueueEmail(user.id);
return enqueueEmail(user.id);
void enqueueEmail(user.id); // intentionally fire and forget
await Promise.all([enqueueEmail(first), enqueueEmail(second)]);
```

## Options

- `targets` is an array of target groups. Each group may set
  `sourceSpecifierPatterns` and `calleeNamePatterns`; values are glob or
  `/regex/` strings. Only calls matching a configured group are checked.

Same-file namespace members, a default import of the configured module, a single `const` alias, a string or expression-free template member, an optional call, a comma callee, and `.call` or a literal-array `.apply` are the same callee.

## Fix

Await or return work whose failure belongs to the caller. Use `void` only when
the call is deliberately detached.

## Editor suggestion

For a bare configured call, ESLint offers one suggestion: prefix it with
`void`. It does not guess whether `await`, `return`, or detachment is correct.

## Limits

Another file's re-export is not visible here. `import { send } from "./barrel.mjs"; send(id)` stays unreported when `barrel.mjs` is `export { enqueueEmail as send } from "@app/jobs"` or `export * from "@app/jobs"`. A parameter default such as `function run({ enqueueEmail } = client)` is not judged, because the caller can pass a different object. `.apply` with a non-literal argument list is not rewritten. `const send = (0, enqueueEmail)` is still `enqueueEmail`. `const send = enqueueEmail<Job>` is still `enqueueEmail`. `import * as jobs from "@app/jobs"; const api = jobs; api.enqueueEmail(id)` is still that method.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/async-call-disposition -- queue is intentionally best effort
enqueueEmail(user.id);
```

## Related rules

- [`async-try-catch-return-await`](async-try-catch-return-await.md) for returned
  promises inside protected error boundaries.
- [`require-options-on-imported-call`](require-options-on-imported-call.md) for
  required options on imported, global, or receiver-shaped calls.
