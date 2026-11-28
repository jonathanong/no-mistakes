# `queue-no-enqueue-in-transaction`

Reports a configured queue publish in the body of an inline transaction callback.

## Why and when

A queue publish that persists independently of a database transaction can survive
its rollback. A worker can then receive work that depends on data the transaction
never committed. Enable this rule for transaction APIs that invoke their callbacks
before committing and publishers whose effects survive rollback.

## What it catches

The rule matches a transaction call and a publish by their resolved import specifier
and export name, following named local re-exports (including renamed barrels such as
`export { enqueue as send } from "@sample/jobs"`) to their source specifier. An
imported alias such as `import { enqueue as publish } from "@sample/jobs"` retains
the `enqueue` export identity. Configure each source module specifier and export the
resolver exposes for your calls. Local functions that shadow an imported name, calls imported from unrelated
modules, and local function, class, or static `const` declarations exported
from relative modules stay outside the check.

The transaction call must use a direct import binding so its argument positions
can be proved, and the configured argument must be an inline arrow or
non-generator function expression. The rule checks direct calls owned by that
callback, including expression bodies, without proving whether a particular
branch runs. A helper reached from two transaction callbacks is checked under
each of them. Calls on that transaction callback's own parameter (such as
`query(...)`, `query<Row>(...)`, `tx.insertReceipt()`, `` tx.sql`...` ``, or
`` tx`...` ``) are treated as transaction handles rather than publishers or
unanalyzable targets. A nested transaction that reuses a helper closing over
the outer handle reports the captured call as unanalyzable: the exemption
belongs only to the transaction whose parameter the call uses. Built-in static methods (`Math.*`, `JSON.*`, `Number.*`,
`String.*`, `Object.*`, `Array.*`, `Promise.*`, and similar global built-in
objects when not shadowed) and standard `Array`, `String`, `Map`, `Set`, `Promise`, `Number`,
`Date`, and `RegExp` prototype methods on locals are known not to publish.
A method on `this.field` is in that set only when the field's annotation is one
of those built-in values; an untyped or structural `this.queue.push` is
unanalyzable. Passing a leaf built-in such as `Math.abs` as a callback is
clean, and passing a callback-invoking built-in such as `Promise.try` is
unanalyzable. Callbacks passed to built-in methods such as `rows.map(...)`, `flatMap(...)`,
`filter(...)`, `reduce(...)`, `forEach(...)`, `Array.from(...)`, and `.then(...)`
are analyzed in the same transaction scope so an enqueue inside them is still
flagged. An optional callback argument (`JSON.stringify` replacer, or the
replacement passed to `replace` / `replaceAll`) is skipped when it is `null`,
`undefined`, an array, or an object. A conditional argument is checked on each
arm, so a configured publisher in one arm is still reported. Separate nested functions, deferred callbacks, and class instance field
initializers have separate bodies and are outside the rule's direct-body
analysis.

Shapes the rule cannot soundly prove are recorded as `unanalyzable` (never
silently passed) and routed through the rule's `unanalyzable` policy (`report`
by default, or `advise` / `ignore`):

- Unresolved or ambiguous transaction or publish targets (including unresolved
  imports, dynamic callees, dynamic receivers, publisher-module factory returns,
  nullish/alternative call targets, wildcard re-exports that reach a configured
  module or a cycle, and unresolvable or cyclic local re-exports).
- Unprovable, spread, or generator callbacks passed to built-in methods inside
  a transaction callback.
- Bound, aliased, or factory-returned transaction calls, and tagged template
  calls to a configured transaction.
- Spread arguments at or before the configured callback argument slot.
- Non-inline callbacks (identifier or expression references) and generator
  callbacks.

## Options and defaults

The rule is off by default. Opt in with a repository-scoped application and
supply both non-empty lists below. Module and export names must contain text.
Each transaction module and export pair may appear once. Use `include` and `exclude` to select files by
repository-relative paths.

| Option | Meaning |
| --- | --- |
| `transactions` | Transaction APIs, declared as `{ module, export, callbackArgument }`. The callback argument index is zero-based and has no default. Each listed API must call the callback before commit. |
| `publishers` | Independently durable queue APIs, declared as `{ module, export }`. Leave transaction-bound outbox writers outside this list. |
| `unanalyzable` | `report` (default), `advise`, or `ignore` for shapes the rule cannot prove. |

```yaml
rules:
  - rule: queue-no-enqueue-in-transaction
    scope: repository
    include: ["src/**/*.ts"]
    options:
      transactions:
        - module: "@sample/database"
          export: transaction
          callbackArgument: 1
      publishers:
        - module: "@sample/jobs"
          export: enqueue
```

## Valid example

Return the data the publisher needs from the transaction callback. Call the
publisher once the transaction resolves and the data has committed.

```ts
import { transaction } from "@sample/database";
import { enqueue } from "@sample/jobs";

const receiptId = await transaction("write", async (tx) => {
  const id = await tx.insertReceipt();
  await tx.appendAuditEntry(id);
  return id;
});

await enqueue({ receiptId });
```

## Counterexample

The enqueue can succeed before a later write fails and rolls back the receipt.

```ts
import { transaction } from "@sample/database";
import { enqueue } from "@sample/jobs";

await transaction("write", async (tx) => {
  const receiptId = await tx.insertReceipt();
  await enqueue({ receiptId });
  await tx.appendAuditEntry(receiptId);
});
```

## Fix

Move this enqueue after the transaction commits. Return the job data from the
callback and enqueue it after the transaction call resolves.

## Suppression

A documented exception is an enqueue that provides the only durable handoff.
Keep it inside that transaction so a failed enqueue rolls the row back. The
enqueue must tolerate collisions, and the worker must be idempotent and treat a
missing row as a retry.

Use `no-mistakes-disable-next-line queue-no-enqueue-in-transaction` directly
above the publish or `no-mistakes-disable-line queue-no-enqueue-in-transaction`
on the publish line. Use `no-mistakes-disable-file queue-no-enqueue-in-transaction`
for a reviewed file-wide exception. Record why the queue is the sole durable
handoff.

## Analysis

The rule uses the prepared call graph at `max` effort. A request for `low`
reports an effort warning and uses `max`. Unprovable shapes follow
`options.unanalyzable` (`report`, `advise`, or `ignore`). Selected files with
parse errors produce incomplete analysis through the check report.

## Related rules

[`no-single-enqueue-in-loop`](no-single-enqueue-in-loop.md) checks for
per-item publishes that should use a bulk API.
