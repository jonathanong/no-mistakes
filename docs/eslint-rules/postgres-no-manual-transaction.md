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

- `importSpecifier` identifies the database module and defaults to
  `"@data-stores/psql"`.
- `executorNames` lists checked executor names and defaults to
  `["query", "read", "write"]`.
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
