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

Destructuring a pool constructor from a default or namespace import also reports:

```ts
import pg from "pg";

const { Pool } = pg;
export const pool = new Pool({ host: "example.invalid" });
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
are recognized, including `const { Pool } = pg` and renamed properties such as
`const { Pool: DatabasePool } = pg`. Attach the listener to the created pool:

```ts
import pg from "pg";

const { Pool: DatabasePool } = pg;
export const pool = new DatabasePool({ host: "example.invalid" });
pool.on("error", console.error);
```

Both the destructured constructor binding and the module binding resolve
lexically. In CommonJS, a literal `require("pg")` can supply a destructured
`Pool` or a `.Pool` member directly or through a stable `const` module binding.
Parameters or local variables can shadow `Pool`, `pg`, or `require`. The rule
checks runtime bindings from the configured modules.

A direct `this` member also works for class-field initializers and assignments.

```ts
import { Pool } from "pg";

export class Database {
  #pool = new Pool();

  constructor() {
    this.#pool.on("error", console.error);
  }
}
```

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
replace it. The same destructuring support applies to configured compatible
modules. Subpaths and cross-file re-exports are not inferred. This rule is
opt-in and is not enabled by `configs.recommended` or `configs.strict`.

## Fix

Bind the pool locally and attach its idle-client error listener where it is
created. Log or handle the failure according to the application's policy. The
rule has no autofix because it cannot choose that policy for the application.
Passing the pool to a helper such as `attach(pool)` still produces a finding,
even if the helper registers the listener. Move the registration beside
construction or document the helper's ownership with a suppression.

## Limits

This is a check for registration syntax in one module, not a proof that the
registration executes before an error, remains installed, or handles failures
correctly. A listener in a nested function referencing the same binding counts.
Bindings with multiple writes cannot establish which pool instance owns a listener; use
separate stable bindings and register each instance explicitly.
Class member matching follows a directly named `this` property within the
same class. Static and instance properties are tracked independently. Use a
field with one recognized write and register directly on `this.pool` or
`this["pool"]`. An uninitialized runtime field such as TypeScript's
`pool: Pool` counts as a write, so a later assignment is a second write.
A TypeScript `declare pool: Pool` field is erased. Receiver aliases, accessors,
inherited fields, dynamic keys, and deeper chains such as `this.database.pool`
fall outside this matching.

Constructor destructuring follows `const` declarations with static property
names and no default values along the path. Nested patterns can reach the
module through inline object literals, as in
`const { database: { Pool } } = { database: pg }`. Unknown nested exports and
paths through opaque objects are outside the check.

The rule applies to test files whenever their ESLint config enables it. Use a
flat-config `files` glob or a test-file override to select the intended files.

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
