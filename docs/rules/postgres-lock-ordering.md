# `postgres-lock-ordering`

Flags executed PostgreSQL SQL that takes a `FOR UPDATE` lock with a multi-row
predicate (`IN` or `= ANY`) but no `ORDER BY` and no `SKIP LOCKED`. Two
transactions that lock the same rows in opposite order can deadlock (ABBA).

The rule uses the shared PostgreSQL embedded-SQL facts
(`extract_embedded_sql_from_source`, `collect_postgres_facts`) and
`extract_locking_select_metadata`. It does not re-parse TypeScript or SQL
with a private parser.

```yaml
rules:
  - rule: postgres-lock-ordering
    scope: repository
    options:
      include: ["src/**/*.ts"]
      exclude: ["src/generated/**"]
      importSpecifier: "@data-stores/psql"
      executorNames: [query, read, write]
      safeDirective: deadlock-safe
```

`importSpecifier` defaults to `@data-stores/psql`. `executorNames` defaults to
`query`, `read`, and `write`. `safeDirective` defaults to `deadlock-safe`.

Counterexample: `query(\`SELECT * FROM t WHERE id = ANY($1) FOR UPDATE\`)`
without `ORDER BY` or `SKIP LOCKED`. Unparseable `FOR UPDATE` SQL is a
separate diagnostic so lock statements stay parseable.

```ts
import { query } from "@data-stores/psql";

export function lockRows(ids: string[]) {
  return query(`SELECT * FROM t WHERE id = ANY($1) FOR UPDATE`);
}
```

Fix: add `ORDER BY` so every locker visits rows in the same order, add
`SKIP LOCKED` when skipping already-locked rows is correct, or put
`/* deadlock-safe: ... */` or `-- deadlock-safe` in a comment within 200
characters before the call when a unique key makes the lock single-row.

```ts
query(`SELECT * FROM t WHERE id = ANY($1) ORDER BY id FOR UPDATE`);
query(`SELECT * FROM t WHERE id = ANY($1) FOR UPDATE SKIP LOCKED`);

/* deadlock-safe: single row via unique key */
query(`SELECT * FROM t WHERE id = ANY($1) FOR UPDATE`);
```

Use `no-mistakes-disable-next-line postgres-lock-ordering` or
`no-mistakes-disable-line` for a one-off, or `no-mistakes-disable-file`
when a whole file is an intentional exception.

## Why and when

Use this rule for worker or request code that locks several rows by ID, where
concurrent transactions can visit the same set in different orders.

## What it catches/requires

A multi-row `FOR UPDATE` query using `IN` or `= ANY` must order its rows or use
`SKIP LOCKED`. The embedded SQL must be statically recoverable; unparseable
statements receive a separate diagnostic.

## Options and defaults

`include` and `exclude` select source files. `importSpecifier` defaults to
`@data-stores/psql`, `executorNames` defaults to `[query, read, write]`, and
`safeDirective` defaults to `deadlock-safe`.

## Valid example

```ts
query(`SELECT * FROM jobs WHERE id = ANY($1) ORDER BY id FOR UPDATE`);
```

## Counterexample

```ts
query(`SELECT * FROM jobs WHERE id = ANY($1) FOR UPDATE`);
```

## Fix

Add deterministic `ORDER BY`, use `SKIP LOCKED` for a work queue, or document
why a unique-key lookup is effectively single-row with the configured safe
directive.

## Suppression

Use `no-mistakes-disable-next-line postgres-lock-ordering` or
`no-mistakes-disable-line`; reserve the file directive for a file whose locking
contract is enforced elsewhere.

## Related rules

[`postgres-no-offset`](postgres-no-offset.md) covers inefficient pagination;
[`postgres-require-query-annotation`](postgres-require-query-annotation.md)
ensures lock statements remain identifiable in query logs.
