# `no-mistakes/postgres-no-async-sql-builder`

## Why

An async SQL builder that never waits returns a Promise. The analyzer does not
unwrap that promise, so the SQL stays unanalyzable even when the function only
returns a literal or a configured `sql` tag. A synchronous function that
returns the same value is SQL the analyzer can judge. Async is still required
when the function suspends. Removing it is also unsound when a return is not a
judged SQL shape, when the body can throw before later caller statements, or
when a visible signature or caller still needs a Promise.

This rule is opt-in. It is not part of `configs.recommended` or
`configs.strict`, and it reports nothing until `builders` names the functions
or callees that produce SQL. It never offers an autofix. It reports only when
the rewrite is proven. Otherwise it is silent.

## Disallowed

```ts
import { sql } from "@example/db";

async function buildQuery(id: string) {
  return sql`/* orders/by-id */ SELECT id FROM orders WHERE account_id = ${id}`;
}

await buildQuery(id);
```

`buildQuery` returns the configured `sql` tag on every path. It does not
await, throw, loop, or call anything else. `await` does not require a Promise,
so the report tells you to remove `async`.

A constant string or a template with no substitutions is the same kind of
return. A plain template with substitutions is not: it is not a trusted SQL
tag, so the rule stays silent.

## Allowed

Deleting `async` leaves the same `sql` tag. Callers that only `await` the
builder can stay as they are; `await` of a non-promise is already legal.

```ts
import { read, sql } from "@example/db";

function buildQuery(id: string) {
  return sql`/* orders/by-id */ SELECT id FROM orders WHERE account_id = ${id}`;
}

export function loadOrders(id: string) {
  return read(buildQuery(id));
}
```

`buildQuery` is not exported. The analyzer judges this same-file builder,
including the `${id}` value bind, with no incomplete reason. That file is
`fixtures/postgres-facts/executed-source/same-file-sql-builder.ts`. The rule
does not itself suggest `read(buildQuery(id))` when `read` is an unresolved
import. An untyped, `Promise`-typed, `any`, `unknown`, `object`, unresolved, or
Promise-compatible type-literal parameter makes the rule stay silent, because
the synchronous call would throw if that parameter uses the value as a Promise.
A `string` or `number` parameter does not. A callback result of `{}`, an alias
of `{}`, `{ (): {} }`, or a literal whose members are only optional or named
`then`, `catch`, or `finally` is the same kind of Promise-compatible type.
`{ foo: string }` is not: that callback still allows the report. A direct
`{}` argument and a property typed `{ f: () => {} }` stay silent. The same check applies to a bound local (`const query = build()`),
including when that local is passed on, returned, or copied into another
local. A use this file does not explicitly recognize, such as `instanceof` or
a property, stays silent.

The rule also stays silent, with no report, when it cannot prove the rewrite.
That includes any of these:

- a return that is not a string, a template with no substitutions, or a tag
  of a configured callee, on every path (a `names` entry does not skip this
  check). A call such as `sql(id)` stays silent. A tag placeholder that is not
  an identifier, a literal, or already judged SQL, such as `${id.x}`, stays
  silent. A member read outside that placeholder, such as `id.x;`,
  `const x = id.x`, or `id.x ? "a" : "b"`, also stays silent
- `db.query(...)`, `Promise.resolve(...)`, `loadSql(...)`, or a method call
  such as ``sql`...`.append(...)``
- `throw`, `new`, a loop, `switch`, `try`, a destructured parameter, a
  parameter default, or any call that is not a configured callee or tag. The
  callee of a judged tag, including `(db.sql)`, is not one of these reads
- an exported builder, a return type or implemented interface it cannot
  resolve, or an explicit `Promise<...>`, `any`, `unknown`, `object`, or
  Promise-compatible type-literal return type
- a builder passed to a same-file parameter whose type is a function returning
  a Promise, or whose result, directly or through a bound local, is passed to
  `.then`, `.catch`, or `.finally`
- a class or object value that escapes this file

Silence is not permission to delete `async`.

The rule does not report a function that actually waits:

```ts
async function buildQuery(id: string) {
  const scope = await loadScope();
  return sql`/* orders/by-id */ SELECT id FROM orders WHERE account_id = ${id}`;
}
```

## Options

- `builders` lists the SQL builders to check. The rule is inert when this array
  is missing or empty, including entries that set neither `names` nor `callees`.
- `names` lists local function, method, or binding names to consider. A
  matching `async` function is still silent unless every return is a judged
  SQL shape.
- `callees` lists tagged templates that count as judged SQL when the function
  returns them directly, through a conditional or `const` / `let` binding that
  is not reassigned. A call of that name, such as `sql(id)`, is not judged
  SQL. A chained call such as ``sql`...`.append(...)`` matches the builder,
  but the rule stays silent because the returned call is not the configured
  tag. Placeholders must be identifiers, literals, or already judged SQL;
  `${id.x}` stays silent because it can throw after `async` is removed.
- `source`, when set, limits `callees` to a named import or a namespace member
  imported from that exact module specifier. Renamed imports match the imported
  name (`import { sql as tag }` matches `sql`). Type-only and default imports
  do not match. Without `source`, `callees` match the local identifier or
  member name as written.

```js
{
  "no-mistakes/postgres-no-async-sql-builder": [
    "error",
    {
      builders: [
        { names: ["buildQuery"] },
        { source: "@example/db", callees: ["sql"] },
      ],
    },
  ],
}
```

Removal is suggested only when every one of these is true:

- every return is a string literal, a template literal with no substitutions,
  or a tag of a configured callee whose placeholders are identifiers, literals,
  or already judged SQL, including both branches of a conditional and a
  `const` / `let` binding that is not reassigned. A call such as `sql(id)` does
  not qualify
- the function has no `await`, `for await`, `await using`, or dynamic
  `import()` that it returns
- the body has no `throw`, `new`, loop, `switch`, `try`, member read other
  than a judged tag callee, destructured parameter, parameter default, or
  call other than a configured callee or tag
- it is not an override or interface method required to return a Promise, a
  same-file base class or interface can be checked when one is declared, and
  no same-file subclass declares the same method
- it has no explicit `Promise<...>`, `any`, `unknown`, `object`, or
  Promise-compatible type-literal return type, including a same-file alias of
  `Promise` and `globalThis.Promise`
- it is not passed to a same-file parameter whose type is a function returning
  a Promise
- callers in this file do not use the result, or a local that holds it, with
  `.then`, `.catch`, or `.finally`, and do not annotate that result as
  `Promise<...>`, `any`, `unknown`, `object`, or a Promise-compatible type
  literal. A use that only tests truthiness (`build() ? ...` or `build() || ...`)
  stays silent. `await build()` and `const s = await build()` still allow the
  report
- a call that receives the result, directly or through a local, has a
  same-file parameter type that is not a Promise, `any`, `unknown`, `object`,
  a Promise-compatible type literal, or unresolved; an untyped parameter
  abstains. A `string`, `number`, or `{ foo: string }` callback result still
  allows the report. A direct `{}` parameter stays silent
- a local that holds the result is only awaited, voided, passed, returned, or
  copied in ways this file can prove do not need a Promise. Any other use,
  including `instanceof`, a property, or an alias that later needs the
  Promise, stays silent
- the function, its class, and its object are not exported, and those values
  do not escape into an unresolved call

If any check fails, the rule reports nothing. It does not guess, and it does
not tell you to keep `async`.

## Fix

There is no autofix and no editor suggestion. When the rule reports, delete
`async` and return the same string or configured SQL tag. Callers that only
`await` the builder can stay unchanged.

Removing `async` runs the body before the caller continues. A throw, a
throwing call, or a getter then happens before later statements such as
`const query = build(); other(); await query`. The rule abstains on `throw`,
`new`, loops, `switch`, `try`, calls that are not a configured SQL callee or
tag, member reads other than a judged tag callee, destructured parameters,
and parameter defaults. It does not model every other abrupt completion, so a
silent result is not permission to delete `async`.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/postgres-no-async-sql-builder -- callback type in another package requires a Promise
async function buildQuery() {
  return "/* orders/by-id */ SELECT id FROM orders";
}
```

`no-mistakes-disable-file`, `no-mistakes-disable-line`, and
`no-mistakes-disable-next-line` suppress this rule the same way. Do not use
`guardrails` directives.

## Related rules

- [`postgres-cursor-call-contract`](postgres-cursor-call-contract.md) requires
  direct, attributable SQL at cursor executors.
- [`postgres-no-manual-transaction`](postgres-no-manual-transaction.md) keeps
  transaction control in one helper.
- [`postgres-require-query-annotation`](../rules/postgres-require-query-annotation.md)
  is why a builder promise that is not awaited stays unanalyzable and why the
  synchronous `sql` tag above can be judged.
