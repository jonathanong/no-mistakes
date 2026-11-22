# `no-mistakes/postgres-pool-requires-error-listener`

## Why

A node-postgres pool emits `error` when an idle client fails, for example after a
server restart or network reset. Without a listener, Node treats the event as an
unhandled error and can terminate the process. Register the listener in the
module that creates the pool so ownership is visible beside construction.

## Disallowed

```ts
import { Pool } from "pg";

export const pool = new Pool({ host: "example.invalid" });
```

Returning an unbound pool from a factory also reports the `new` expression:

```ts
import pg from "pg";

export function createPool() {
  return new pg.Pool({ host: "example.invalid" });
}
```

## Allowed

```ts
import { Pool } from "pg";

export const pool = new Pool({ host: "example.invalid" });
pool.on("error", (error) => console.error("Idle database client failed", error));
```

```ts
import * as pg from "pg";

export const pool = new pg.Pool({ host: "example.invalid" }).on("error", (error) => {
  console.error("Idle database client failed", error);
});
```

A literal `error` event registered with `on`, `once`, or `addListener` on the
same pool binding satisfies the check. Other events or another pool's listener
do not. Named import aliases, default-member imports, and namespace imports
are recognized. Type-only imports and constructors from unconfigured modules
are not selected.

## Options

```js
{
  "no-mistakes/postgres-pool-requires-error-listener": [
    "error",
    { extraModules: ["@example/database"] },
  ],
}
```

`extraModules` is an optional array of exact module names exporting a compatible
`Pool`. It defaults to `[]` and adds to the built-in `pg` module; it does not
replace it. Subpaths and cross-file re-exports are not inferred. This rule is
opt-in and is not enabled by `configs.recommended` or `configs.strict`.

## Fix

Bind the pool locally and attach its idle-client error listener where it is
created. Log or handle the failure according to the application's policy. The
rule has no autofix because it cannot choose that policy for the application.
A listener in a different module does not satisfy this file-local check;
move it beside construction or document the external ownership with a
suppression.

## Limits

This is a check for registration syntax in one module, not a proof that the
registration executes before an error, remains installed, or handles failures
correctly. A listener in a nested function referencing the same binding counts.
Reassigned bindings cannot establish which pool instance owns a listener; use
separate stable bindings and register each instance explicitly.

## Suppression

```ts
import { Pool } from "pg";

// eslint-disable-next-line no-mistakes/postgres-pool-requires-error-listener -- external owner registers this pool
export const pool = new Pool({ host: "example.invalid" });
```

The shared suppression helper also supports these comments:

```ts
import { Pool } from "pg";

// no-mistakes-disable-next-line postgres-pool-requires-error-listener
export const pool = new Pool({ host: "example.invalid" });
```

```ts
import { Pool } from "pg";

export const pool = new Pool({ host: "example.invalid" }); // no-mistakes-disable-line postgres-pool-requires-error-listener
```

```ts
// no-mistakes-disable-file postgres-pool-requires-error-listener
import { Pool } from "pg";

export const pool = new Pool({ host: "example.invalid" });
```

## Related rules

- [`postgres-no-manual-transaction`](postgres-no-manual-transaction.md) keeps
  transaction lifecycle in an owner helper.
- [`postgres-no-unbounded-query-fanout`](postgres-no-unbounded-query-fanout.md)
  bounds concurrent database work.
