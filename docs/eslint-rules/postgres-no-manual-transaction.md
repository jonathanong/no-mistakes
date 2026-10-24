# `no-mistakes/postgres-no-manual-transaction`

## Why

Scattered `BEGIN`, `COMMIT`, and `ROLLBACK` calls make transaction ownership,
cleanup, and nesting difficult to audit.

## Disallowed

```ts
import { query } from "@example/db";

await query("BEGIN");
await query("COMMIT");
```

```ts
import * as db from "@example/db";

const { query } = db;
const q = query;
const execute = (sql) => q(sql);
await q.call(null, "BEGIN");
await q.apply(null, ["COMMIT"]);
await execute("ROLLBACK");
```

## Allowed

```ts
import { withTransaction } from "@example/db";

await withTransaction(async (tx) => {
  await tx.query("/* update-user */ UPDATE users SET active = true");
});
```

`SAVEPOINT`, `RELEASE SAVEPOINT`, and complete `ROLLBACK [WORK|TRANSACTION]
TO [SAVEPOINT] name` statements keep the existing transaction open and are not
whole-transaction control. A later `BEGIN`, `COMMIT`, or whole `ROLLBACK` in the
same call is still rejected. Quoted identifiers, doubled quotes, escape strings, dollar-quoted bodies (including non-ASCII identifier tags) and
nested block comments preserve statement boundaries. Malformed or ambiguous
quoting containing transaction commands fails closed; plain-string backslash
semantics depend on the server setting and cannot safely grant an exemption.
This distinction does not certify that a client's
query wrapper supports recovery; the owner must preserve that wrapper's contract.

## Options

```js
{
  "no-mistakes/postgres-no-manual-transaction": [
    "error",
    { importSpecifier: "@example/db" },
  ],
}
```

- `importSpecifier` identifies the database module and defaults to
  empty (configure your database module explicitly).
- `executorNames` lists checked executor names and defaults to
  `["query", "read", "write"]` only with a configured module; otherwise it is empty.
- `executorFactoryNames` lists imports whose call result bound with `const`,
  `let`, `using`, or `await using` is an executor inside the declaring block. It
  defaults to `[]`.
- `executorTypeNames` lists imported type names whose annotated parameters
  (including optional and inline-destructured ones) are executors inside the
  declaring function. It defaults to `[]`.
- `trustedSqlTags` defaults to `[]`. Each entry has `module` and `name`.
  It is accepted so a shared executor configuration validates. This rule still
  reads every tagged template.
- Imports for the two scoped options above match `importSpecifier` and its
  subpaths (`@example/db/types` for `@example/db`), not sibling packages such as
  `@example/dbx`. A relative import matches only when it resolves into that
  package. If the package root cannot be determined, relative imports do not match.
- `owners` is an absolute-suffix or repository-relative allowlist for the
  transaction lifecycle helper. It defaults to no owner exemptions.

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

### SQL value aliases

When released SQL-text recovery cannot resolve an argument, the rule also
follows a chain of completed, unwritten `const` identifier initializers to a
string literal or interpolation-free template:

```ts
import { query } from "@example/db";
const begin = "BEGIN";
const first = begin;
const command = first;
await query(command);
```

Each binding and all its references must stay in that binding's lexical scope.
This includes function-local chains whose declarations and reads share the
same function scope. A reference in another block or function, even an unrelated
read, keeps the new chain unproved. This boundary avoids following reads through
nearer dynamic bindings, such as `with` or a non-strict function's eval-created
local. It does not claim that every use of eval is excluded.

The added path does not evaluate tags, interpolations, concatenation, helpers,
containers, imports, destructuring, mutable bindings, forward reads, or cycles.
TypeScript wrappers are erased only to recover the underlying primitive value.
Existing direct and one-hop SQL recovery stays unchanged, including its broader
released decisions for mutable bindings, outer-scope constants and tagged SQL.

```ts
const begin = "BEGIN";
const command = begin;
function later() {
  query(command); // A new chain crossing a function boundary is not followed.
}
```

```ts
import { command } from "./transaction-commands";
query(command); // A value re-exported by another file is not proved here.
```

## Fix

Move transaction control into the reviewed transaction helper and pass its
executor to the operation callback.

The repair for a SQL alias chain is a lifecycle migration, not replacing one
isolated `BEGIN`, `COMMIT`, or `ROLLBACK` call with a new empty transaction.
Move the surrounding business operation into `withTransaction` or the
appropriate `withTransactionOptions` helper, preserving transaction options and
the helper's completion/error contract. Let that helper own commit, rollback
and cleanup:

```ts
await withTransaction(async (tx) => {
  await tx.query("/* update-user */ UPDATE users SET active = true");
});
```

This applies to the direct argument, namespace member, `.call`, literal-array
`.apply`, and existing transparent executor-forwarder forms. Keep non-lifecycle
work, conditions, wrapper contracts, and any receiver/argument side effects in
their original order. Use the helper-provided executor inside the operation.
Remove SQL aliases only when unused elsewhere; their other consumers and
forwarder definitions need not change. There is no automatic fix because the
operation boundary and owner-helper contract require application knowledge.

## Suppression

```ts
// no-mistakes-disable-next-line postgres-no-manual-transaction -- migration runner owns this explicit transaction
await query("BEGIN");
```

`no-mistakes-disable-line` and `no-mistakes-disable-file` are also supported.

## Related rules

- [`postgres-cursor-call-contract`](postgres-cursor-call-contract.md) requires
  direct, attributable cursor calls.

Executor import matching has no module default. Set `importSpecifier` explicitly
to your database module to enable default `query`, `read`, and `write` names.
Without a module, only explicit `executorNames` select named imports; explicit
`query` also enables `.query` members. A configured module retains member matching
even with custom executor names. See the [migration notes](../migrations/explicit-postgres-executors.md).

Omitting both `importSpecifier` and `executorNames` is a configuration error.
Set `executorNames: []` explicitly to select no executor calls.
