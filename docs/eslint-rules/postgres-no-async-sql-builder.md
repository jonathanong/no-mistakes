# `no-mistakes/postgres-no-async-sql-builder`

## Why

An async SQL builder that never waits returns a Promise. The analyzer does not
unwrap that promise, so `read(buildSql())` is unanalyzable even when the
function only returns a literal statement. A synchronous function that returns
the same literal is SQL the analyzer can judge. Async is still required when
the function actually suspends, and removing it is unsound when a visible
signature or caller still needs a Promise.

This rule is opt-in. It is not part of `configs.recommended` or
`configs.strict`, and it reports nothing until `builders` names the functions
or callees that produce SQL. It never offers an autofix.

## Disallowed

```ts
import { sql } from "@example/db";

async function buildQuery(id: string) {
  return sql`/* orders/by-id */ SELECT id FROM orders WHERE account_id = ${id}`;
}

read(buildQuery(id));
```

`buildQuery` has no `await`, `for await`, or dynamic `import()` that it
returns. Nothing in this file requires a Promise, so the report tells you to
remove `async`.

A builder that never waits but whose visible contract still requires a Promise
is reported with the opposite instruction: do not remove `async`.

```ts
async function buildQuery(): Promise<string> {
  return "/* orders/by-id */ SELECT id FROM orders";
}
```

## Allowed

The synchronous rewrite returns the same literal SQL. Callers that only
`await` the builder can call it directly; `await` of a non-promise is already
legal.

```ts
import { sql } from "@example/db";

function buildQuery(id: string) {
  return sql`/* orders/by-id */ SELECT id FROM orders WHERE account_id = ${id}`;
}

read(buildQuery(id));
```

The rule does not report a function that actually waits:

```ts
async function buildQuery(id: string) {
  const scope = await loadScope();
  return sql`/* orders/by-id */ SELECT id FROM orders WHERE account_id = ${id}`;
}
```

It also abstains, with no report, when it cannot prove that a sync function is
allowed. That includes an exported builder, a return type or implemented
interface it cannot resolve, a builder passed to an untyped or imported
callback parameter, and a class or object value that escapes this file.
Abstaining is not permission to delete `async`.

## Options

- `builders` lists the SQL builders to check. The rule is inert when this array
  is missing or empty, including entries that set neither `names` nor `callees`.
- `names` lists local function, method, or binding names. A matching `async`
  function is a builder even when this file does not show the SQL expression.
- `callees` lists calls or tagged templates whose result the function returns,
  including a direct return, a conditional or logical branch, or a `const` /
  `let` binding that is not reassigned. A chained member such as
  ``sql`...`.append(...)`` still counts as `sql`.
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

- the function has no `await`, `for await`, `await using`, or dynamic
  `import()` that it returns (including `import()?.then(...)`) or that an
  expression-bodied arrow returns
- it is not an override or interface method required to return a Promise, a
  same-file base class or interface can be checked when one is declared, and
  no same-file subclass declares the same method (directly or through another
  local class), because that override still has to match the async signature
- it has no explicit `Promise<...>` return type, including a same-file alias
  of `Promise` and `globalThis.Promise`
- it is not passed to a same-file parameter whose type is a function returning
  a Promise
- callers in this file do not use the result with `.then`, `.catch`, or
  `.finally`, and do not annotate that result as `Promise<...>`
- the function, its class, and its object are not exported, and those values
  do not escape into an unresolved call

If any of those checks fail closed, the rule either says not to remove `async`
or reports nothing. It does not guess.

## Fix

There is no autofix and no editor suggestion. When the report says to remove
`async`, delete that keyword and return the same literal SQL. When the report
says not to remove `async`, keep the Promise contract; change the signature
and every caller yourself before making the builder synchronous. When the rule
is silent, it did not prove the rewrite, so leave `async` in place.

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
  synchronous literal above can be judged.
