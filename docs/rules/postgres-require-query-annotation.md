# `postgres-require-query-annotation`

Require a leading `/* name */` block comment on executed PostgreSQL SQL so
slow-query logs and `EXPLAIN ANALYZE` can name the statement. Line comments
(`-- name`) do not count. `BEGIN`, `COMMIT`, and `ROLLBACK` are exempt.

The rule uses the shared PostgreSQL embedded-SQL facts
(`extract_embedded_sql_from_source`, `collect_postgres_facts`) and
`sql_requires_query_annotation`. It does not re-parse TypeScript with a
private parser.

```yaml
rules:
  - rule: postgres-require-query-annotation
    scope: repository
    options:
      include: ["src/**/*.ts"]
      exclude: ["src/generated/**"]
      importSpecifier: "@example/db"
      executorNames: [query, read, write]
```

`importSpecifier` has no default. `executorNames` defaults to `query`, `read`, and `write` only when `importSpecifier` is configured.

Counterexample: `query(\`SELECT id FROM posts\`)`.

```ts
import { query } from "@example/db";

export function list() {
  return query(`SELECT id FROM posts ORDER BY id DESC`);
}
```

Fix: put a non-empty block comment at the start of the executed SQL.

```ts
query(`/* posts/list */ SELECT id FROM posts ORDER BY id DESC`);
```

Use `no-mistakes-disable-next-line postgres-require-query-annotation` or
`no-mistakes-disable-line` for a one-off, or `no-mistakes-disable-file`
when a whole file is an intentional exception.

## Why and when

Use this rule when query logs, slow-query reports, and `EXPLAIN ANALYZE` output
need a stable operation name rather than an opaque SQL string.

## What it catches/requires

Executed PostgreSQL SQL must begin with a non-empty block comment. `BEGIN`,
`COMMIT`, and `ROLLBACK` are exempt; line comments do not satisfy the contract.

## Options and defaults

`include` and `exclude` select source files. `importSpecifier` has no default, and `executorNames` defaults to `[query, read, write]` only when `importSpecifier` is configured.

### Executor configuration

Omitting both `importSpecifier` and `executorNames` is a configuration error.
Set `importSpecifier` to your database module or list `executorNames` explicitly.
Use `executorNames: []` without a module to select no executor calls and retain
SQL-file/native-SQL analysis where supported. See the
[executor migration](../migrations/explicit-postgres-executors.md).

| Option | Default | Behavior |
| --- | --- | --- |
| `importSpecifier` | Empty | Set explicitly to your database module to match its named imports. |
| `executorNames` | Empty without a module; `[query, read, write]` with a module | Without a module, only explicitly listed names match named imports from any module. |

With both options omitted, executor calls (including `.query`) are not scanned.
A configured module or explicit `query` enables `.query` members. A configured module also recognizes
its transaction helpers. Native SQL and recovered SQL-builder fragments retain
their existing scopes. See [the migration notes](../migrations/explicit-postgres-executors.md).

## Valid example

```ts
query(`/* posts/list */ SELECT id FROM posts ORDER BY id DESC`);
```

## Counterexample

```ts
query(`SELECT id FROM posts ORDER BY id DESC`);
```

## Fix

Add a short operation identifier as the first block comment inside the SQL
string passed to the configured executor.

## Suppression

Use `no-mistakes-disable-next-line postgres-require-query-annotation` or
`no-mistakes-disable-line`; use the file directive for a deliberately opaque
administrative script.

## Related rules

[`postgres-no-offset`](postgres-no-offset.md) discourages unstable pagination;
[`postgres-lock-ordering`](postgres-lock-ordering.md) protects concurrent row
locks.
