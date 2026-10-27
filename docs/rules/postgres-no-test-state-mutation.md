# `postgres-no-test-state-mutation`

## Why and when

Use this rule when PostgreSQL tests share a database or worker pool. A test that
changes schema or server state can make later tests depend on execution order.
Run migrations before the suite; keep individual tests inside the declared
schema and use a throwing injected executor to test failure handling.

## Configuration and options

```yaml
rules:
  - rule: postgres-no-test-state-mutation
    projects: [database-tests]
    options:
      testGlobs: ["tests/**/*.test.ts"]
      helperRoots: ["tests/helpers/**/*.ts"]
      importSpecifier: "@example/db"
      executorNames: [query]
      executorFactoryNames: [beginTransaction]
      executorTypeNames: [TransactionQuery]
      trustedSqlTags: [{ module: "@example/db", name: sql }]
```

`testGlobs` is required and has no default test filename convention. It selects
test entry files within the rule's project and `include`/`exclude` scope.
Both option arrays contain repository-relative globs, including when a rule
references a project (for example `backend/tests/**/*.test.ts`).
`helperRoots` defaults to empty; it names helper module globs included alongside
the selected test entries. The rule checks the union of matching test and
helper files. Imported helper-built SQL passed to a test executor is also
recoverable through shared SQL extraction. Production migrations outside these
globs are outside this rule. There is no `sqlInclude` option: the rule checks
SQL that reaches configured executors in this explicit scope rather than
scanning raw `.sql` files.

`importSpecifier` defaults to empty, so configure an executor module or inherit
it from [`projects.<name>.postgres`](../configuration/projects.md#postgresql-scope).
When an executor module is set, `executorNames` defaults to `query`, `read`,
and `write`; `executorFactoryNames`, `executorTypeNames`, and `trustedSqlTags`
default to empty lists. Factories and types identify additional explicitly
configured executor forms. A trusted tag entry names a named export from a
module; it does not make unrelated strings executable SQL.

`unanalyzable` follows the shared `report` (default), `advise`, or `ignore`
policy. The legacy `unanalyzableSql` alias also accepts `fail` as `report`.
Definite violations remain findings under every policy. `message` customizes
the rule application's finding text. There is no autofix.

## What it catches

The rule rejects these statements in executed SQL from selected tests or
selected helper files:

- `CREATE`, `ALTER`, or `DROP` of a database, schema, table, view, trigger,
  function, procedure, index, or type, including forms such as `OR REPLACE`,
  `TEMP`, `UNLOGGED`, `UNIQUE`, and `MATERIALIZED`.
- `TRUNCATE` and `ALTER SYSTEM`.
- `SET session_replication_role` and `SET LOCAL session_replication_role`.
- Calls to `pg_sleep`, including calls in nested queries.

Plan-only `EXPLAIN` and `EXPLAIN (ANALYZE false/off)` do not execute the
enclosed function call. `EXPLAIN ANALYZE` and `EXPLAIN (ANALYZE true/on)` do.

SQL printed in an assertion message, stored in a snapshot, or compared as data
(for example `pg_indexes.indexdef`) does not reach an executor and is not
reported. Ordinary DML in test setup is allowed by this rule.

`SELECT ... INTO` is not flagged, including forms that create a new table.
The rule does not classify that statement as one of the mutations listed above.

An imported string constant passed directly to a configured executor, such as
`query(importedSql)`, is reported as unanalyzable rather than classified as a
definite mutation. This differs from an imported helper function whose literal
SQL return value can be recovered. The `unanalyzable` policy described above
controls that report.

## Counterexample

```ts
// tests/users.test.ts
import { query } from "@example/db";

test("rejects a missing table", async () => {
  await query("DROP TABLE users");
  await expect(loadUsers()).rejects.toThrow();
});
```

Dropping the table changes shared database state and may break later tests.

## Valid example and fix

```ts
// tests/users.test.ts
test("handles an executor failure", async () => {
  const query = async () => { throw new Error("database unavailable"); };
  await expect(loadUsers({ query })).rejects.toThrow("database unavailable");
});
```

For tests that require schema objects, create them in a migration before tests
run. For a failure case, inject a throwing executor as shown above instead of
executing DDL inside the test. Replace SQL sleeps with an awaited application
signal or a controlled test clock. Move replication-role and server-setting
setup into suite provisioning rather than changing it from an individual test.

## Suppression

Use `// no-mistakes-disable-next-line postgres-no-test-state-mutation` on a
specific executed SQL call, or `// no-mistakes-disable-line
postgres-no-test-state-mutation` on its line. Use
`// no-mistakes-disable-file postgres-no-test-state-mutation` only for a file
whose exception is intentional, with a reason in the adjacent comment.

## Related rules

[`postgres-sql-statement-policy`](postgres-sql-statement-policy.md) bans
configured statement kinds in broader SQL scopes.
[`postgres-sql-shape-policy`](postgres-sql-shape-policy.md) can ban configured
function calls in executable SQL. This rule provides a focused test entry and
helper scope for schema mutation, server settings, and `pg_sleep`.
