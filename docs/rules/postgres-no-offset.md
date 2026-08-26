# `postgres-no-offset`

Flags executed PostgreSQL SQL that uses an `OFFSET` clause. Offset pagination
reads skipped rows again on every page and is usually the wrong default next to
cursor pagination, `LIMIT + 1`, `COUNT`, `EXISTS`, or `ROW_NUMBER()`.

The rule uses the shared PostgreSQL embedded-SQL facts
(`extract_embedded_sql_from_source`, `collect_postgres_facts`) and
`sql_has_offset_clause`. It does not re-parse TypeScript with a private
parser. Unparseable SQL is ignored so comments and prose are not treated as
clauses.

```yaml
rules:
  - rule: postgres-no-offset
    scope: repository
    options:
      include: ["src/**/*.ts"]
      exclude: ["src/generated/**"]
      importSpecifier: "@data-stores/psql"
      executorNames: [query, read, write]
```

`importSpecifier` defaults to `@data-stores/psql`. `executorNames` defaults to
`query`, `read`, and `write`.

Counterexample: `query(\`SELECT id FROM posts OFFSET 10\`)`. Interpolated
offsets such as `OFFSET ${limit}` are findings once the template becomes
`OFFSET sql_placeholder_1`.

```ts
import { query } from "@data-stores/psql";

export function page() {
  return query(`SELECT id FROM posts ORDER BY id DESC OFFSET 10`);
}
```

Fix: page with a cursor, `LIMIT + 1`, `COUNT`, `EXISTS`, or `ROW_NUMBER()`
instead of `OFFSET`.

```ts
query(`SELECT id FROM posts ORDER BY id DESC LIMIT ${limit + 1}`);
```

String literals that mention the word "offset" are not findings.

Use `no-mistakes-disable-next-line postgres-no-offset` or
`no-mistakes-disable-line` for a one-off, or `no-mistakes-disable-file`
when a whole file is an intentional exception.

## Why and when

Use this rule for APIs and workers where offset pagination becomes slower and
less stable as rows are inserted or deleted between requests.

## What it catches/requires

Executed, statically recoverable PostgreSQL SQL must not contain `OFFSET`.
Interpolated offsets are checked after placeholder normalization; prose and
unparseable SQL are ignored.

## Options and defaults

`include` and `exclude` select source files. `importSpecifier` defaults to
`@data-stores/psql`, and `executorNames` defaults to `[query, read, write]`.

## Valid example

```ts
query(`SELECT id FROM posts ORDER BY id DESC LIMIT ${limit + 1}`);
```

## Counterexample

```ts
query(`SELECT id FROM posts ORDER BY id DESC OFFSET 10`);
```

## Fix

Use a cursor predicate with a deterministic order, or use `LIMIT + 1`,
`COUNT`, `EXISTS`, or `ROW_NUMBER()` when that is the actual query need.

## Suppression

Use `no-mistakes-disable-next-line postgres-no-offset` or
`no-mistakes-disable-line`; use the file directive only for an intentionally
offset-based reporting query.

## Related rules

[`postgres-lock-ordering`](postgres-lock-ordering.md) covers multi-row locks;
[`postgres-require-query-annotation`](postgres-require-query-annotation.md)
keeps executed SQL identifiable in logs.
