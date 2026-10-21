# `no-mistakes/postgres-no-nonliteral-sql-registry-key`

## Why

`TABLES[kind]` on a const object of identifier strings is not a finite SQL
identifier. The key can be `"constructor"` or `"__proto__"`, so the lookup can
select an inherited property. A runtime own-key check is not recovered.

Return each entry from a switch with a throwing default. That shape is judged:
`tableName` in
`test-cases/rules/postgres-required-predicates/variants/proven-throw-switch/src/queries.mts`
returns `"orders"` or `"accounts"`, and `byTable` is checked per arm. A selector
known to match no arm publishes nothing.

This rule is opt-in. It is not part of `configs.recommended` or `configs.strict`.
It does not autofix. Object mutation stays in
[issue #2219](https://github.com/jonathanong/no-mistakes/issues/2219).

## Disallowed

```ts
import { read } from "@example/db";

const TABLES = { orders: "orders", accounts: "accounts" };

export function byKind(kind: string, id: string) {
  return read(`SELECT id FROM ${TABLES[kind]} WHERE id = $1`, [id]);
}
```

## Allowed

```ts
import { read } from "@example/db";

function tableName(kind: string) {
  switch (kind) {
    case "orders":
      return "orders";
    case "accounts":
      return "accounts";
    default:
      throw new Error("unknown table");
  }
}

export function byKind(kind: string, id: string) {
  return read(`SELECT id FROM ${tableName(kind)} WHERE id = $1`, [id]);
}
```

A literal key (`TABLES["orders"]` or `TABLES.orders`) is already finite. A
`let` registry, a spread, a method, a non-string value, an alias of the
object, or a lookup outside a configured executor's SQL argument is left alone.
The rule also follows one local `const` identifier passed as the SQL argument;
it reports the registry lookup at the initializer, once even if the statement
is executed more than once. A mutable SQL binding, an imported SQL value, or a
chain of SQL aliases is left alone.

An explicit `TABLES[kind as keyof typeof TABLES]` assertion is left alone when
the type query refers to that exact registry binding. A cast to another
registry's keys or to `string` is still reported. This is a syntactic contract
for the type checker and the caller to uphold, not runtime key validation.

## Options

`importSpecifier` selects the module whose named imports are executors.
`executorNames` defaults to `["query", "read", "write"]` when `importSpecifier`
is set, and must be set (use `[]` for no calls) when `importSpecifier` is
empty. Scoped factories and `trustedSqlTags` are not options of this rule.

## Fix

This rule has no autofix and no editor suggestion. Write the switch by hand.
The message provides a complete immediately invoked function containing
`switch (<key expression>)`, one `case` / `return` per key, and a throwing
`default`. Duplicate property keys produce one case using their final value,
matching object-literal overwrite behavior. The expression can replace the
lookup at the current scope:

```ts
(() => {
  switch (kind) {
    case "orders": return "orders";
    case "accounts": return "accounts";
    default: throw new Error("unknown");
  }
})()
```

## Suppression

```ts
// no-mistakes-disable-next-line postgres-no-nonliteral-sql-registry-key -- reviewed registry
read(`SELECT ${TABLES[kind]}`);
```

`no-mistakes-disable-line` and `no-mistakes-disable-file` work the same way.
A string that only contains the directive does not suppress the lookup.

## Related rules

[`postgres-no-correlated-sql-builder`](postgres-no-correlated-sql-builder.md)
covers repeated appends. [`postgres-no-executor-adapter`](postgres-no-executor-adapter.md)
covers forwarding the executor instead of the SQL text.
