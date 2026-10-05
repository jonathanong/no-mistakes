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
  empty (configure your database module explicitly).
- `executorNames` lists checked executor names and defaults to
  `["query", "read", "write"]` only with a configured module; otherwise it is empty.
- `executorFactoryNames` lists imports whose call result bound with `const`,
  `let`, `using`, or `await using` is an executor inside the declaring block. It
  defaults to `[]`.
- `executorTypeNames` lists imported type names whose annotated parameters
  (including optional and inline-destructured ones) are executors inside the
  declaring function. It defaults to `[]`.
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
Without a module, only explicit `executorNames` select named imports; explicit
`query` also enables `.query` members. A configured module retains member matching
even with custom executor names. See the [migration notes](../migrations/explicit-postgres-executors.md).

Omitting both `importSpecifier` and `executorNames` is a configuration error.
Set `executorNames: []` explicitly to select no executor calls.
