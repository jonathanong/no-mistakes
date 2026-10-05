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
      importSpecifier: "@example/db"
      executorNames: [query, read, write]
```

`importSpecifier` has no default. `executorNames` defaults to `query`, `read`, and `write` only when `importSpecifier` is configured.

Counterexample: `query(\`SELECT id FROM posts OFFSET 10\`)`. Interpolated
offsets such as `OFFSET ${limit}`are findings once the template becomes`OFFSET sql_placeholder_1`.

```ts
import { query } from "@example/db";

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

`TABLE posts OFFSET 10` is an executed query with an OFFSET clause and is
reported at the OFFSET keyword, just like a SELECT query.

Executed, statically recoverable PostgreSQL SQL must not contain `OFFSET`.
`.sql` files are checked only when `sqlInclude` matches them. Interpolated
offsets are checked after placeholder normalization; prose and unparseable
SQL are ignored. `OFFSET 0` asks for a `MATERIALIZED` CTE. Any other offset
asks for cursor pagination, `LIMIT + 1`, `COUNT`, `EXISTS`, or `ROW_NUMBER()`.

## Options and defaults

`include` and `exclude` select source files. `sqlInclude` defaults to `[]`,
so `.sql` files are not scanned unless a glob selects them. `importSpecifier`
has no module default, and `executorNames` defaults to `[query, read, write]` only when `importSpecifier` is configured.

`OFFSET 0` is reported as an optimizer fence: use a `MATERIALIZED` CTE
(`WITH x AS MATERIALIZED (...)`). Any other offset keeps the pagination
message.

### Executor configuration

Omitting both `importSpecifier` and `executorNames` is a configuration error.
Set `importSpecifier` to your database module or list `executorNames` explicitly.
Use `executorNames: []` without a module to select no executor calls and retain
SQL-file/native-SQL analysis where supported. See the
[executor migration](../migrations/explicit-postgres-executors.md).

| Option            | Default                                                      | Behavior                                                                            |
| ----------------- | ------------------------------------------------------------ | ----------------------------------------------------------------------------------- |
| `importSpecifier` | Empty                                                        | Set explicitly to your database module to match its named imports.                  |
| `executorNames`   | Empty without a module; `[query, read, write]` with a module | Without a module, only explicitly listed names match named imports from any module. |
| `executorFactoryNames` | Empty | Named imports (from `importSpecifier`, or any module when it is empty) whose call result bound with `const`, `let`, `using`, or `await using` is an executor inside the declaring block. |
| `executorTypeNames` | Empty | Imported type names (`import type` or inline `type` specifiers) whose annotated parameters, including optional and inline-destructured ones, are executors inside the declaring function. |

`executorFactoryNames` and `executorTypeNames` add scoped executors: `tx` in
`await using tx = await openTransaction()` or `run` in `run: TxExecutor` is scanned
only inside the block or function that declares it, and a same-named identifier
elsewhere is not. Matching is by configuration only, both default to empty, and
neither counts as executor selection: still set `importSpecifier` or `executorNames`.
With `importSpecifier` set, these two options also match imports from its subpaths
(`@example/db/types` for `@example/db`) but not sibling packages such as
`@example/dbx`; `executorNames` still requires the exact module.

Omitting both options is a configuration error. Set `executorNames: []` without
`importSpecifier` to explicitly skip executor calls (including `.query`).
A configured module or explicit `query` enables `.query` members. A configured module also recognizes
its transaction helpers. Native SQL and recovered SQL-builder fragments retain
their existing scopes. See [the migration notes](../migrations/explicit-postgres-executors.md).

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
declarations and non-analyzing EXPLAIN plans are skipped; EXPLAIN ANALYZE queries
are executed and checked, including the parenthesized ANALYZE option.
COPY FROM STDIN payload rows are data; queries after
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
Static `.append()` composition retains each appended literal or bound fragment's
physical position, including placeholder renumbering. Put line suppression on
the physical clause being suppressed. Existing directives on an executor call
also suppress that call's recovered clauses through the common suppression pass.
Prepared source failures retain their I/O kind; dispatch uses that captured
outcome without checking filesystem state again.
Source positions mark changes to the source-versus-SQL line offset. Between
positions, map a SQL line with `source_line + sql_line - position.sql_line`;
an empty position list retains the ordinary call/declaration line mapping.
