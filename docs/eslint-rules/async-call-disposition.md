# `no-mistakes/async-call-disposition`

## Why

Configured async boundaries, such as enqueue or scheduling helpers, must make
their promise disposition explicit. A bare call can silently lose a rejection.

## Disallowed

```ts
import { enqueueEmail } from "@app/jobs";

enqueueEmail(user.id);
```

A conditional assignment does not establish identity for a later sibling call.
Move setup into that call's block before installing the implementation.

```ts
import { enqueueEmail } from "@app/jobs";
import { vi } from "vitest";

function conditionalSetup(m, enabled) {
  let spy;
  if (enabled) spy = vi.spyOn(m, "enqueue");
  spy.mockImplementation(() => enqueueEmail()); // reported: mock identity is unproven
}
```

## Allowed

```ts
import { enqueueEmail } from "@app/jobs";

async function sendEmail(user, first, second) {
  await enqueueEmail(user.id);
  void enqueueEmail(user.id); // intentionally fire and forget
  await Promise.all([enqueueEmail(first), enqueueEmail(second)]);
}

function returnToCaller(user) {
  return enqueueEmail(user.id);
}
```

A callback's return also counts for `vi.fn`, `jest.fn`, and
`mockImplementation` / `mockImplementationOnce` on a known `vi.spyOn`,
`jest.spyOn`, `vi.fn`, `jest.fn`, or `vi.mocked` result. These APIs pass the implementation's
return value to the mock's caller. Imported aliases, framework namespaces such
as `vitest.vi`, and unchanged `const` or `let` mock bindings are recognized.

```ts
import { enqueueEmail } from "@app/jobs";
import { vi } from "vitest";

vi.spyOn(service, "send").mockImplementation((id) => {
  const job = enqueueEmail(id);
  return job;
});

async function sendBatch(ids) {
  const pending = [];
  const job = enqueueEmail(ids[0]);
  pending.push(job);
  pending.push(enqueueEmail(ids[1]));
  await Promise.allSettled(pending);
}
```

An uninitialized `let`, including a `MockInstance<...> | undefined` annotation,
also establishes a mock after exactly one direct `=` assignment. The assignment
and implementation call must be ordered sibling statements in the same block;
the declaration may be in an enclosing scope. This includes setup inside a
`try` body or a nested block.

```ts
import { enqueueEmail as original, enqueueRelease } from "@app/jobs";
import { vi, type MockInstance } from "vitest";

function install(m, selected) {
  let spy: MockInstance<typeof m.enqueue> | undefined;
  try {
    spy = vi.spyOn(m, "enqueue");
    spy.mockImplementation((tags, p) => {
      if (selected) return original(tags, p);
      return enqueueRelease(tags, p);
    });
  } finally {
    spy?.mockRestore();
  }
}
```

A promise retained in an unchanged `const` binding counts when it is later
explicitly disposed in the same block. An initially empty `const` array also
counts when its promises are pushed and its direct `Promise.all`,
`Promise.allSettled`, `Promise.race`, or `Promise.any` result is later awaited,
returned, or explicitly voided. A retained `.map` result follows the same
aggregate requirement; returning or awaiting the array itself does not dispose
its members. Synchronous `.forEach` callbacks can push work before that drain.
A conditional push also retains a promise created in its argument whenever
that branch runs; it does not invalidate sibling pushes. A promise created
before a conditional push still needs an independent disposition.
An unconditional drain in `finally` also handles work created in its protected
`try` block, including a return or throw from that block.

```ts
import { vi } from "vitest";

async function mapped(ids) {
  const pending = ids.map((id) => enqueueEmail(id));
  await Promise.all(pending);
}

async function collected(ids) {
  const pending = [];
  ids.forEach((id) => pending.push(enqueueEmail(id)));
  await Promise.race(pending);
}

async function finalized() {
  const pending = [];
  try {
    pending.push(enqueueEmail("one"));
    return await run();
  } finally {
    await Promise.all(pending);
  }
}

let spy = vi.spyOn(service, "send");
spy.mockImplementation((...args) => {
  const result = Promise.resolve(enqueueEmail(...args)).finally(cleanup);
  if (collect) pending.push(result);
  return result;
});
vi.mocked(service.send).mockImplementationOnce((id) => enqueueEmail(id));
```

The native global `Promise.resolve` adopts its first argument, including
when the resulting promise is chained, retained, and returned. An independently
proven return still counts after a conditional collection push; an earlier
conditional return or throw cannot prove that later return executes.

## Options

- `targets` is an array of target groups. Each group may set
  `sourceSpecifierPatterns` and `calleeNamePatterns`; values are glob or
  `/regex/` strings. Only calls matching a configured group are checked.

```js
{
  targets: [{ sourceSpecifierPatterns: ["@app/jobs"], calleeNamePatterns: ["/^enqueue/"] }],
}
```

Same-file namespace members, a default import of the configured module, a single `const` alias, a string or expression-free template member, an optional call, a comma callee, and `.call` or a literal-array `.apply` are the same callee.

## Fix

Await or return work whose failure belongs to the caller. Use `void` only when
the call is deliberately detached.

## Editor suggestion

For a bare configured call, ESLint offers one suggestion: prefix it with
`void`. It does not guess whether `await`, `return`, or detachment is correct.

## Limits

Another file's re-export is not visible here. `import { send } from "./barrel.mjs"; send(id)` stays unreported when `barrel.mjs` is `export { enqueueEmail as send } from "@app/jobs"` or `export * from "@app/jobs"`. A parameter default such as `function run({ enqueueEmail } = client)` is not judged, because the caller can pass a different object. `.apply` with a non-literal argument list is not rewritten. `const send = (0, enqueueEmail)` is still `enqueueEmail`. `const send = enqueueEmail<Job>` is still `enqueueEmail`. `import * as jobs from "@app/jobs"; const api = jobs; api.enqueueEmail(id)` is still that method.

Collection tracking requires straight-line statements in the same executing
block, with the push before the observation. Conditional observations, array
mutation or escape, async `.forEach` callbacks, reassigned bindings, and unknown
callback APIs remain reported. A catch-only or conditional `finally` drain does
not establish disposition. Other call wrappers do not establish adoption. A mock callback that pushes and returns a promise
is accepted through its return; this does not establish when an outer array is
drained. Locally shadowed `vi`, `jest`, and `Promise` objects do not establish
these dispositions. Direct property writes, updates, or deletion on a known mock
or framework object, including through stable local aliases, invalidate its
pass-through identity. A delayed assignment must precede the use in the same
executing block; an assignment inside `try` cannot prove identity after that
`try`, because an earlier throw may bypass it. Other writes, compound or embedded
assignments, assignments in another function, and unknown receivers remain
unproven. Property writes through aliases also invalidate delayed mock bindings.
The analysis does not prove that callers actually invoke
or await a mock; its return follows the same explicit-disposition contract as
an ordinary function return.

## Suppression

```ts
// no-mistakes-disable-next-line async-call-disposition -- queue is intentionally best effort
enqueueEmail(user.id);
```

`no-mistakes-disable-file` and `no-mistakes-disable-line` are also supported.

## Related rules

- [`async-try-catch-return-await`](async-try-catch-return-await.md) for returned
  promises inside protected error boundaries.
- [`require-options-on-imported-call`](require-options-on-imported-call.md) for
  required options on imported, global, or receiver-shaped calls.

## Awaiting callback consumers

`awaitingConsumers` explicitly trusts an imported API to await or otherwise adopt
its callback's returned promise. Each entry uses `sourceSpecifierPatterns` and
`calleeNamePatterns` like `targets`, and requires `callbackArgumentIndexes`: a
nonempty array of distinct, nonnegative, zero-based argument positions. Omit this
option to retain the default callback behavior.

```js
{
  targets: [{ sourceSpecifierPatterns: ["@app/jobs"], calleeNamePatterns: ["/^enqueue/"] }],
  awaitingConsumers: [{
    sourceSpecifierPatterns: ["@app/admission"],
    calleeNamePatterns: ["awaitReportedEnqueueAdmission"],
    callbackArgumentIndexes: [0],
  }],
}
```

```typescript
import { awaitReportedEnqueueAdmission } from "@app/admission";
await awaitReportedEnqueueAdmission(() => {
  return enqueueOnTopicCreated(data); // accepted: configured consumer adopts this return
});
unknownConsumer(() => enqueueOnTopicCreated(data)); // reported: no configured contract
awaitReportedEnqueueAdmission(null, () => enqueueOnTopicCreated(data)); // reported: wrong slot
awaitReportedEnqueueAdmission(() => { enqueueOnTopicCreated(data); }); // reported: not returned

const accepted = () => enqueueOnTopicCreated(data);
awaitReportedEnqueueAdmission(accepted); // accepted: stable callback returns its promise
function unknown() { return enqueueOnTopicCreated(data); }
unknownConsumer(unknown); // reported: this consumer has no adoption contract
```

Return the target promise (or an already supported retained, chained, or aggregate
promise) from the configured callback. Inline callbacks, callbacks bound to an
unchanged `const`, and function declarations with no reassignment are checked.
Every call argument use of a named callback must be a configured awaiting consumer
or a recognized mock that passes through its return. A mixed known and unknown
consumer use remains reported. Ordinary functions that are not passed as call
arguments still dispose a promise by returning it. Matching follows import
identity, including stable aliases and logical `.call` argument positions;
shadowed or reassigned callees do not qualify. A spread argument before the
callback prevents proving its position. A `.bind` result passed to a consumer
does not establish an adoption contract; a bound function invoked locally still
uses the ordinary return contract. The option does not inspect consumer
bodies or infer contracts from names. As with ordinary function
and mock callback returns, it does not prove that a caller invokes or awaits the
consumer. Configure only APIs that honor this contract. Existing literal-array
`.apply` callback return behavior remains unchanged.
