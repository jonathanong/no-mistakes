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

- `importSpecifier` identifies the database module; its default is the plugin's
  standard PostgreSQL import.
- `executorNames` lists checked executor names.
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
