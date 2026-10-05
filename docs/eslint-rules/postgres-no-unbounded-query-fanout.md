# `no-mistakes/postgres-no-unbounded-query-fanout`

## Why

`Promise.all(items.map(query))` can create unbounded concurrent database work
from an input-sized list.

## Disallowed

```ts
await Promise.all(userIds.map((id) => query("SELECT * FROM users WHERE id = $1", [id])));
```

## Allowed

```ts
for (const ids of chunkArray(userIds, 50)) {
  await Promise.all(ids.map((id) => query("SELECT * FROM users WHERE id = $1", [id])));
}
```

## Options

- `importSpecifier` identifies the database module; it defaults to empty. Set
  it explicitly to select imports from a module.
- `executorNames` lists checked executor names and defaults to `[]`. A configured
  module with no explicit names enables `query`, `read`, and `write`.
- `executorFactoryNames` lists imports whose call result bound with `const`,
  `let`, `using`, or `await using` is an executor inside the declaring block. It
  defaults to `[]`.
- `executorTypeNames` lists imported type names whose annotated parameters
  (including optional and inline-destructured ones) are executors inside the
  declaring function. It defaults to `[]`.
- Imports for the two scoped options above match `importSpecifier` and its
  subpaths (`@example/db/types` for `@example/db`), not sibling packages such as
  `@example/dbx`.
- `chunkFunctionNames` lists approved chunk helpers and defaults to
  `["chunkArray"]`.

## Fix

Use a static array, a SCREAMING_CASE bounded collection, sequential work, or a
configured chunk helper before the mapped executor calls.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/postgres-no-unbounded-query-fanout -- bounded upstream list is enforced by the API contract
await Promise.all(ids.map((id) => query(sql, [id])));
```

## Related rules

- [`postgres-no-manual-transaction`](postgres-no-manual-transaction.md) covers
  another database lifecycle boundary.

Executor import matching has no module default. Set `importSpecifier` explicitly
to your database module to enable default `query`, `read`, and `write` names.
Without a module, only explicit `executorNames` select named imports; explicit
`query` also enables `.query` members. A configured module retains member matching
even with custom executor names. See the [migration notes](../migrations/explicit-postgres-executors.md).

Omitting both `importSpecifier` and `executorNames` is a configuration error.
Set `executorNames: []` explicitly to select no executor calls.
