# `postgres-no-offset`

Flags executed PostgreSQL SQL that uses an `OFFSET` clause. Offset pagination
reads skipped rows again on every page and is usually the wrong default next to
cursor pagination, `LIMIT + 1`, `COUNT`, `EXISTS`, or `ROW_NUMBER()`.

The request boundary prepares embedded SQL and statement facts once. This rule
borrows those facts, including source locations, instead of reading or parsing
source again. Unparseable statements are skipped while recoverable sibling
statements remain checked; comments and prose are not clauses.

```yaml
rules:
  - rule: postgres-no-offset
    scope: repository
    options:
      include: ["src/**/*.ts"]
      exclude: ["src/generated/**"]
      sqlInclude: ["db/views/**/*.sql", "db/migrations/**/*.sql"]
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
`.sql` files are checked only when `sqlInclude` matches them. Interpolated
offsets are checked after placeholder normalization; prose and unparseable
SQL are ignored. `OFFSET 0` asks for a `MATERIALIZED` CTE. Any other offset
asks for cursor pagination, `LIMIT + 1`, `COUNT`, `EXISTS`, or `ROW_NUMBER()`.

## Options and defaults

`include` and `exclude` select source files. `sqlInclude` defaults to `[]`,
so `.sql` files are not scanned unless a glob selects them. `importSpecifier`
defaults to `@data-stores/psql`, and `executorNames` defaults to
`[query, read, write]`.

`OFFSET 0` is reported as an optimizer fence: use a `MATERIALIZED` CTE
(`WITH x AS MATERIALIZED (...)`). Any other offset keeps the pagination
message.

## Valid example

```sql
CREATE VIEW view_recent_orders AS
WITH o AS MATERIALIZED (SELECT id, account_id FROM orders WHERE id > '0190')
SELECT o.id FROM o JOIN accounts a ON a.id = o.account_id;
```

```ts
query(`SELECT id FROM posts ORDER BY id DESC LIMIT ${limit + 1}`);
```

## Counterexample

```sql
CREATE VIEW view_recent_orders AS
SELECT * FROM (SELECT id, account_id FROM orders WHERE id > '0190' OFFSET 0) o
JOIN accounts a ON a.id = o.account_id;
```

```ts
query(`SELECT id FROM posts ORDER BY id DESC OFFSET 10`);
```

## Fix

Replace `OFFSET 0` with `WITH x AS MATERIALIZED (...)`. For any other offset,
use a cursor predicate with a deterministic order, or use `LIMIT + 1`,
`COUNT`, `EXISTS`, or `ROW_NUMBER()` when that is the actual query need.

## Suppression

Use `no-mistakes-disable-next-line postgres-no-offset` or
`no-mistakes-disable-line`; use the file directive only for an intentionally
offset-based reporting query.

## Related rules

[`postgres-lock-ordering`](postgres-lock-ordering.md) covers multi-row locks;
[`postgres-require-query-annotation`](postgres-require-query-annotation.md)
keeps executed SQL identifiable in logs.

## Statement coverage and locations

The shared SQL pass records every OFFSET in source order, including INSERT,
UPDATE, DELETE, COPY queries, CREATE TABLE AS, views, CTEs, RETURNING, ON CONFLICT,
and subqueries inside CASE, arrays, functions, predicates, and ordering. Routine
declarations are skipped. COPY FROM STDIN payload rows are data; queries after
its `\.` terminator are still checked. Nested comments, escaped E strings,
dollar quotes, and Unicode identifiers retain their SQL meaning.

Each finding points to the OFFSET keyword. Shared SQL constants point to their
declaration lines, and embedded multiline queries retain line-specific
suppression. Multiple offsets on the same line remain separate findings.
The first occurrence in a file keeps target `offset`; later occurrences use
`offset#2`, `offset#3`, and so on. The Rust statement-fact contract exposes these
occurrences as `SqlStatementFileFacts.offset_uses`, with line, column, and
`OffsetUse` kind. `sql_offset_uses` and `sql_file_offset_uses` use this same AST
visitor and preserve source order. Bare `*.sql` patterns match SQL basenames;
`sqlInclude: []` continues to opt out of SQL file scanning.

Recovered literals and template quasis retain a compact physical-line map in
`EmbeddedSqlCall.sql_source_positions` (`EmbeddedSqlSourcePosition`). Cooked
newline escapes, line continuations, multiline interpolations, and initializers
that begin below their declaration preserve the actual OFFSET source line.
Prepared source failures retain their I/O kind; dispatch uses that captured
outcome without checking filesystem state again.
Source positions mark changes to the source-versus-SQL line offset. Between
positions, map a SQL line with `source_line + sql_line - position.sql_line`;
an empty position list retains the ordinary call/declaration line mapping.
