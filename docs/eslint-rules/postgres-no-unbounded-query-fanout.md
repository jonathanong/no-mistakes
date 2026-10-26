# `no-mistakes/postgres-no-unbounded-query-fanout`

## Why

`Promise.all(items.map(query))` can create unbounded concurrent database work
from an input-sized list.

## Disallowed

```ts
import { query } from "@example/db";

await Promise.all(userIds.map((id) => query("SELECT * FROM users WHERE id = $1", [id])));
```

```ts
import * as db from "@example/db";

const { query } = db;
const q = query;
const execute = (sql, values) => q(sql, values);
await Promise.all(userIds.map((id) => execute("SELECT * FROM users WHERE id = $1", [id])));
await Promise.all(statements.map(db.read));
```

## Allowed

```ts
import { query } from "@example/db";

for (const id of userIds) {
  await query("SELECT * FROM users WHERE id = $1", [id]);
}
```

## Options

```js
{
  "no-mistakes/postgres-no-unbounded-query-fanout": [
    "error",
    { importSpecifier: "@example/db" },
  ],
}
```

- `importSpecifier` identifies the database module; it defaults to empty. Set
  it explicitly to select imports from a module.
- `executorNames` lists checked executor names and defaults to `[]`. A configured
  module with no explicit names enables `query`, `read`, and `write`.
- `executorFactoryNames` lists imports whose call result bound with `const`,
  `let`, `using`, or `await using` is an executor inside the declaring block. It
  defaults to `[]`.
- `executorTypeNames` lists imported type names whose annotated parameters
  (including optional and inline-destructured ones) are executors inside the
  declaring function. With typed linting, this also follows required properties
  of declared options types, including named interfaces, type aliases, and
  destructured aliases. The property must resolve to a configured imported
  executor type. It defaults to `[]`.
- `trustedSqlTags` defaults to `[]`. Each entry has `module` and `name`.
  It is accepted so a shared executor configuration validates. This rule still
  reads every tagged template.
- Imports for the two scoped options above match `importSpecifier` and its
  subpaths (`@example/db/types` for `@example/db`), not sibling packages such as
  `@example/dbx`. A relative import matches only when it resolves into that
  package. If the package root cannot be determined, relative imports do not match.
- `chunkFunctionNames` lists approved chunk helpers and defaults to
  `["chunkArray"]`.

## Limits

The executor calls above include same-file immutable `const` aliases,
value namespace imports from the configured module, and `const { query } = db` (or
another configured namespace export). String and expression-free template keys,
optional calls, TypeScript wrappers, comma callees, `.call`, and `.apply` with a
literal array are supported. A configured namespace export named `call` or
`apply` is matched as that executor before function forwarding.

One local arrow, function expression, or function declaration can forward
identifier parameters directly to the executor. Its body must be one call or
one return, optionally awaited. Parameters must have no defaults, rest, or
destructuring; spreads, missing forwarded arguments, and a second forwarder
stay unknown. Reassigned bindings and cyclic aliases are not followed.

Cross-file re-exports and `export *` are not followed. For example,
`export { read as execute } from "@example/db"` in `db.ts`, then
`import { execute } from "./db"; execute(sql)` in another file does not prove
that `execute` is the configured import. A default import is not a namespace.
A type-only namespace import does not establish a value executor.
A computed identifier key (`const { [key]: q } = db`) or a property default
(`const { query: q = fallback } = db`) is not a static executor alias.
`function run({ query: q } = db) { q(sql); }` is a parameter-default
destructure, so its default does not prove the caller supplied that executor.
A nested parameter named `query` shadows the configured import.
An executor reference passed directly to `map` is checked, including `db.read`.
Passing `query.call` or `query.apply` as an unbound callback does not establish
an executor invocation. Direct local callback bodies are also scanned for executor calls; that existing check does
not need to recover SQL arguments through the callback's parameters.

`Promise["all"]` and a no-expression ``Promise[`all`]`` are `Promise.all`.
Static `.call` and literal-array `.apply` wrappers on both `Promise.all` and
`map` retain the check, including optional calls, comma callees, and TypeScript
assertions. For example:

```ts
Promise.all.call(Promise, items.map.apply(items, [query]));
const all = Promise.all;
all.call(Promise, items.map(query));
```

A same-file immutable `const` alias of either method can be forwarded this way.
The explicit receiver determines the mapped collection: `[].map.call(items,
query)` is unbounded, while `items.map.call([1, 2], query)` is bounded. Forwarding
with a spread, a nonliteral argument array, an array hole, or another forwarding
hop is left unknown. A computed identifier, a template with an expression, or
a tagged template is not a static forwarded method name. The existing direct
map check remains conservative for a tagged key such as
``items[tag`map`](query)``: the tag can return `"map"`, so it remains a candidate.

Detached aggregate calls (`const all = Promise.all; all(items.map(query))`) have
no Promise constructor receiver and are not followed. Destructured method
aliases (`const { all } = Promise; all.call(Promise, items.map(query))`) and
aggregate or map aliases with a `let`/`var` declaration or a later assignment
are also not followed. Rewrite these to the static calls above.

## Fix

Use a static array, a SCREAMING_CASE bounded collection, sequential work, or a
configured chunk helper before the mapped executor calls. Only the loop
variable itself is bounded: `for (const ids of chunkArray(...))`. A `let` or
`var` declared in the loop body, or a reassigned loop binding, is still
unbounded.

## Suppression

```ts
// no-mistakes-disable-next-line postgres-no-unbounded-query-fanout -- bounded upstream list is enforced by the API contract
await Promise.all(ids.map((id) => query(sql, [id])));
```

`no-mistakes-disable-line` and `no-mistakes-disable-file` are also supported.

## Related rules

- [`postgres-no-manual-transaction`](postgres-no-manual-transaction.md) covers
  another database lifecycle boundary.

Executor import matching has no module default. Set `importSpecifier` explicitly
to your database module to enable default `query`, `read`, and `write` names.
Without a module, only explicit `executorNames` select named imports; explicit
`query` also enables broad `.query` member matching. With a configured module,
the receiver must be proven by a configured import, scoped factory or imported
executor type, or a stable alias of one. An unrelated `this.query` method stays
outside the rule, including in `Promise.all(rows.map(...))`. Custom executor
names do not remove that proof requirement. See the
[migration notes](../migrations/explicit-postgres-executors.md).

Omitting both `importSpecifier` and `executorNames` is a configuration error.
Set `executorNames: []` explicitly to select no executor calls.
