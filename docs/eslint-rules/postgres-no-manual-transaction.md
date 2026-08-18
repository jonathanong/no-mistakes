# `no-mistakes/postgres-no-manual-transaction`

Disallows executor calls whose resolved SQL starts with `BEGIN`, `COMMIT`, or
`ROLLBACK`.

Why: manual transaction control bypasses the shared `withTransaction` /
`withTransactionOptions` helper. That helper owns connection checkout, commit
and rollback, and error cleanup. Issuing those commands from application
executors leaves idle-in-transaction sessions and uncommitted work that
static analysis cannot attribute to one owner.

Example: application code runs statements inside the shared helper.

```ts
import { withTransaction } from "@data-stores/psql";

export function insertPair(a: string, b: string) {
  return withTransaction(async (query) => {
    await query("INSERT INTO t (id) VALUES ($1)", [a]);
    await query("INSERT INTO t (id) VALUES ($1)", [b]);
  });
}
```

Counterexample: an executor call starts or ends a transaction itself.

```ts
import { query } from "@data-stores/psql";

export async function insertPair(a: string, b: string) {
  await query("BEGIN");
  await query("INSERT INTO t (id) VALUES ($1)", [a]);
  await query("COMMIT");
}
```

Fix: move `BEGIN` / `COMMIT` / `ROLLBACK` into a configured owner file (the
transaction helper) and call `withTransaction` or `withTransactionOptions`
everywhere else. Importing those helpers also binds `query` for executor
detection.

Resolved query text matches the shared `sql_placeholder_N` contract from
[PostgreSQL fact sources](../postgres-facts.md): the first template quasi is
copied as-is, and each later quasi is prefixed with `sql_placeholder_N`.
Identifier bindings such as `const sql = "BEGIN"; query(sql)` are resolved.

Options:

- `importSpecifier` (default `@data-stores/psql`)
- `executorNames` (default `["query", "read", "write"]`)
- `owners` — path allowlist so the transaction helper can issue those
  commands. Each entry matches an absolute path suffix or a repo-relative
  path.

Not in `configs.recommended` or `configs.strict`. Enable it with executor
config for the project.
