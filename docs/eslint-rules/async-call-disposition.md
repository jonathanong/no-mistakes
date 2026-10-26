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

Awaited test assertions also adopt their promise, including a retained `const`:

```ts
await expect(enqueueEmail(user.id)).rejects.toThrow("unavailable");
const pending = enqueueEmail(user.id);
await expect(pending).resolves.toBeUndefined();
enqueueEmail(user.id)
  .then(() => undefined)
  .catch(() => undefined);
```

Assertions require global `expect` with no local declaration, or a direct named `expect` import
from Vitest or Jest. The matcher must be awaited, returned, or otherwise handled;
an assertion with no promise disposition still reports. A terminal `.catch` statement counts when
its handler is a stable function and the chain contains only standard promise
methods. Missing handlers, opaque calls in the chain, and promises dropped inside
a handler remain reported.
Visible replacement of an observer or an escape to an unknown mutator also
keeps the promise unproven.

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

Conditional branches and literal array spreads preserve adoption by an awaited
`Promise.all`, `Promise.allSettled`, `Promise.race`, or `Promise.any`. For example,
`await Promise.all([...(condition ? [enqueueEmail(id)] : [])])` handles the call.
A nested array without a spread, a call in the condition, or a transformed value such as
`enqueueEmail(id) + 1` does not. Dropping the aggregate still reports.

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
`calleeNamePatterns` like `targets`. Specify `callbackArgumentIndexes` for direct
callbacks: a nonempty array of distinct, nonnegative, zero-based argument
positions. For dependency injection, specify `callbackArgumentProperties` with
an `argumentIndex` and nonempty, distinct `propertyNames`. Only named properties
in a fresh object literal at that slot receive this contract; spreads, duplicate
keys, computed properties and nested objects remain unproven. Each entry must
specify at least one of these two callback contracts.

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

For a consumer that awaits `dependencies.enqueueEmail()`, configure its property
explicitly:

```js
{
  sourceSpecifierPatterns: ["@app/admission"],
  calleeNamePatterns: ["awaitReportedEnqueueAdmission"],
  callbackArgumentProperties: [{ argumentIndex: 0, propertyNames: ["enqueueEmail"] }],
}
```

Then `awaitReportedEnqueueAdmission({ enqueueEmail: accepted })` accepts an
unchanged named callback that returns the target promise. An unrelated property
or an unknown consumer still reports.

```typescript
import { awaitReportedEnqueueAdmission } from "@app/admission";
await awaitReportedEnqueueAdmission(() => {
  return enqueueOnTopicCreated(data); // accepted: configured consumer adopts this return
});
unknownConsumer(() => enqueueOnTopicCreated(data)); // reported: no configured contract
awaitReportedEnqueueAdmission(null, () => enqueueOnTopicCreated(data)); // reported: wrong slot
awaitReportedEnqueueAdmission(() => {
  enqueueOnTopicCreated(data);
}); // reported: not returned

const accepted = () => enqueueOnTopicCreated(data);
awaitReportedEnqueueAdmission(accepted); // accepted: stable callback returns its promise
function unknown() {
  return enqueueOnTopicCreated(data);
}
unknownConsumer(unknown); // reported: this consumer has no adoption contract
```

Return the target promise (or an already supported retained, chained, or aggregate
promise) from the configured callback.

- **Accepted:** inline callbacks, callbacks bound to an unchanged `const`, and
  function declarations with no reassignment follow the same return contract.
  Every argument use must be a proven or configured awaiting consumer, a
  recognized mock, or a `.map` / `.flatMap` callback whose result reaches a handled native
  `Promise.all`, `Promise.allSettled`, `Promise.any`, or `Promise.race`. A retained
  mapped collection follows the same aggregate requirement described above.
  Inline `.flatMap` callbacks follow this aggregate contract too.
- **Reported:** mixed known and unknown consumers, wrong callback slots, and
  spreads that obscure the callback position remain unproven. Callback-valued
  conditional and logical branches, final comma operands, assignment results,
  and tagged template interpolations also remain unproven. Passing a `.bind`
  result does not establish an adoption contract.
  `Promise.all([items.map(callback)])` leaves a nested array unhandled; pass the
  mapped array directly or spread its members instead.
- **Not an argument use:** `typeof`, comparisons, updates, untagged template
  literals, conditional tests, and earlier comma operands do not pass the
  callback. Neither do computed property keys, erased type arguments, or uses
  as a tag or constructor. Ordinary functions and locally invoked bound
  functions retain the ordinary return contract.
- **Consumer identity:** matching follows imports, stable aliases, and logical
  `.call` argument positions. Shadowed or reassigned callees do not qualify.
  Existing literal-array `.apply` callback return behavior remains unchanged.

Without configuration, a stable same-file consumer can prove adoption when
every use of its callback parameter is a direct invocation immediately awaited
inside that consumer, including `result = await callback()`. Floating calls,
parameter replacement, forwarding, or capture in a nested function keep the
callback unproven. Executable parameter defaults are inspected too. Resolvable named imports use
this proof on their actual implementation, including in Oxlint's `--type-aware`
mode, which does not expose a parser-owned TypeScript program to JS plugins.
That fallback requires the optional `typescript` peer, version 5 or later for
Bundler module resolution. `@typescript-eslint/parser` is optional; without it,
`ts.createSourceFile` parses supported syntax and unsupported syntax stays
unproven. Install `typescript` in the project to enable this proof. With older TypeScript or missing or
incompatible tooling, the cross-file proof is unproven. Calls that depend on it
are reported; same-file proofs and explicitly configured `awaitingConsumers`
contracts remain available. With TypeScript present and no typed
program, the rule resolves the import using the nearest tsconfig and parses
only the implementation module. Sources, resolution results, and failures are
cached in memory for this lint context. Missing bodies, unresolved imports,
`noResolve`, writes, and escaping implementations remain unproven.
That resolution follows only `import { name }` to one `export function`, one
`export const` function, or one `export { name }` declaration, including
`export { name } from`. `export default`, `export *`, and `module.exports`
stay unproven without a program. A `require()` consumer stays unproven in
either mode, because both proofs accept only one ES import binding. Configure
`awaitingConsumers` when the imported body cannot be proven.

Inline callbacks use the same proven-consumer contract as named callbacks.
For example, `adopt(() => enqueueEmail(id))` is accepted when `adopt` awaits
every invocation of its callback parameter. Passing that same callback to an
unknown consumer still reports.

A narrow local cache also proves adoption when the complete consumer body
creates a fresh built-in `Map`, conditionally stores `callback()` for one key,
and finishes with `return await cache.get(key)`:

```ts
async function cached(callback, key) {
  const cache = new Map();
  if (!cache.has(key)) cache.set(key, callback());
  return await cache.get(key);
}
```

The key must be a string literal or an unchanged parameter. Extra effects,
cache aliases or escapes, replacement values, mismatched keys, reads that are not awaited,
and visible Map constructor/prototype mutations keep the consumer unproven.

A fresh object argument can also prove a dependency callback contract when every
use of the selected property reaches an immediately awaited invocation, such as
`await dependencies.enqueueBulkCrawlReferralLinks()`. The proof follows bounded
same-file and imported helper calls, a defaulted dependency parameter, and
`{ ...dependencies, otherField }` forwarding that preserves the selected property.
It also follows a selected property through `??` or `||` into another fresh object,
including when an async builder in another module writes that `||` copy.
`const { name = defaultImpl } = options` and `const { name } = options` count
when that binding is placed on a fresh object. `{ name: defaultImpl, ...options }`
counts because the spread overwrites the earlier default. An optional member
counts only as the left operand of `??` or `||`, as in `options?.name ?? defaultImpl`.
That copy may be a direct argument, a `const` binding, or the return value of the
helper that builds it. The caller adopts the copy by awaiting that same property,
or by passing the object to an awaited helper whose implementation awaits the
property. An `if (condition) await helper(dependencies)` counts as that adoption.
Returning the copy does not adopt it. An await inside a closure that is
stored or returned does not adopt it either. Replacing the copied property, a
helper call that is not awaited, an unknown helper, and a callback on the
right-hand side of `??` or `||` stay unproven. A property written after
`...options` replaces the copied callback and stays unproven. A `let` binding,
a rename, a nested pattern, a rest element, or a computed key stays unproven.
Pure static property reads do not change this callback's disposition. Sibling
assignments, updates, deletes, nested writes, and assignment-pattern or loop
targets invalidate the supplied shape; runtime mutation cannot prove adoption. An
unrelated dependency method receives the same object as `this`, so its supplied
function must be statically free of `this`, `eval`, and `arguments`. It also must
not close over another supplied callback, including through stable aliases or
transitive local helper calls:
that closure could invoke the callback without handling its promise. Unknown
helper calls, opaque property getters or non-function holders, or helper chains beyond the proof bound
remain unproven. An absent
optional method cannot escape that receiver. Custom `__proto__` properties are
rejected because they can supply an apparently absent optional method. Accesses
to `__proto__`, `constructor`, or `prototype` on the dependency parameter also
keep the proof unproven. These field facts follow forwarding
and form part of the in-memory proof cache key.

Lexical callbacks passed directly to a call can contain these proven flows;
stored or returned closures cannot. As with a directly awaited target call inside
an async callback, this checks the callback's promise disposition when invoked,
not whether the enclosing API invokes or awaits that lexical callback.
Computed or duplicate argument properties, unknown forwarding calls, cycles,
selected-property overrides, dynamic accesses, and whole-object escapes remain
unproven. Merely returning a promise from a named callback does not establish
adoption.

The source can also prove a directly returned target promise through an explicitly typed
consumer slot, including copies inside a stored handler. The slot's parameter or
property must resolve to `typeof` a configured imported target. That target must
be an exported async function, or declare a return of the built-in `Promise`.
An imported return-type alias may resolve to that Promise through named or
type-star reexports. Missing or ambiguous star branches remain unproven.
Generators and a locally shadowed `Promise` do not qualify. An explicit callback
annotation must preserve the same target identity; the annotation alone cannot
prove an unknown receiving slot.

This proof follows named imports, reexports, interfaces, type aliases, and the
built-in `Partial`, `Required`, and `Readonly` wrappers. Tsconfig paths and
inherited configuration use the same resolver as imported consumer bodies.
Disjoint intersections of known property declarations preserve a selected slot;
overlapping properties, unions, index signatures, and unknown constituents do not.
It shares the existing
source session in typed and untyped ESLint and in Oxlint, including
`--type-aware`. It neither requests nor builds a TypeScript program. A consumer
may import only its dependency type; it need not repeat the target import.
As with the imported-body proof, the optional TypeScript peer must be installed
for cross-file declarations. Without it, returning callbacks passed to unproven
slots still report; array aggregate disposition itself needs no proof peers.

Fresh dependency literals can also establish harmless metadata reads. A field
known to be absent can be checked or optionally read without invoking a supplied
callback. A fresh built-in `Date` with proven primitive inputs supports zero-argument
`getTime()` and `toISOString()` reads, including a fresh-Date nullish fallback.
Timestamp constants from built-in `Date.parse`/`Date.UTC` and literal tuple loops
can establish those inputs. A timestamp from an unknown input must be evaluated
before the supplied const callbacks are created in the same block; a later
snapshot remains unproven. Date annotations, unknown holders, getters, aliased
or escaped Date instances, and Object/Date/Array prototype mutation do not
establish this proof.

Every visible forwarding use must preserve that target contract. An unknown or
untyped destination, receiver mutation, an unsafe sibling method, a custom
prototype, `eval`, or `arguments` keeps the callback unproven. Returned carriers
need an explicit matching return annotation so later calls retain a source
type. Missing or ambiguous declarations, cycles, interface extensions, and paths
beyond the proof bound stay unproven. Structural `() => Promise<...>`, `any`,
`unknown`, and missing slots are not this contract. A callback that calls the
target without returning its promise still reports.

A call whose callee type is that same target function is checked in the file
that invokes it. `await dependencies.x(...)` is handled there. Dropping the
call reports at that invocation, including after writes to an explicitly typed
receiver. The producer exemption also requires a proven disposal of each terminal
invocation, including copies created inside nested handlers. A dropped call or
an opaque consumer of its promise retains the original callback finding even
when the copied receiver cannot be classified at its own call site.
Unannotated or unsupported returned carriers retain the original
callback finding instead of erasing it.

Configured contracts trust the API rather than inspecting its body or inferring
behavior from names. As with ordinary function and mock callback returns, the
rule does not prove that a caller invokes or awaits the consumer. Configure only
APIs that honor this contract.
