# `no-mistakes/postgres-no-manual-transaction`

## Why

Scattered `BEGIN`, `COMMIT`, and `ROLLBACK` calls make transaction ownership,
cleanup, and nesting difficult to audit.

## Disallowed

```ts
await query("BEGIN");
await query("COMMIT");
```

## Allowed

```ts
await withTransaction(async (tx) => {
  await tx.query("/* update-user */ UPDATE users SET active = true");
});
```

## Options

- `importSpecifier` identifies the database module and has no default
  (configure your database module explicitly).
- `executorNames` lists checked executor names and defaults to
  `["query", "read", "write"]` only with a configured module. Set at least one of
  `importSpecifier` and `executorNames`: omitting both throws a configuration
  error. `executorNames: []` explicitly selects no executor calls.
- `owners` is an absolute-suffix or repository-relative allowlist for the
  transaction lifecycle helper. It defaults to no owner exemptions.

## Fix

Move transaction control into the reviewed transaction helper and pass its
executor to the operation callback.

## Suppression

```ts
// eslint-disable-next-line no-mistakes/postgres-no-manual-transaction -- migration runner owns this explicit transaction
await query("BEGIN");
```

## Related rules

- [`postgres-cursor-call-contract`](postgres-cursor-call-contract.md) requires
  direct, attributable cursor calls.

Executor import matching has no module default. Set `importSpecifier` explicitly
to your database module to enable default `query`, `read`, and `write` names.
Setting neither `importSpecifier` nor `executorNames` is a configuration error; set
`executorNames: []` explicitly to select no executor calls.
Without a module, only explicit `executorNames` select named imports; explicit
`query` also enables `.query` members. A configured module retains member matching
even with custom executor names. See the [migration notes](../migrations/explicit-postgres-executors.md).
