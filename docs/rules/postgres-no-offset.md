# `postgres-no-offset`

A `projects: [<name>]` entry inherits [`projects.<name>.postgres`](../configuration/projects.md#postgresql-scope) when that block is set. Do not restate `importSpecifier`, `executorNames`, `executorFactoryNames`, `executorTypeNames`, `trustedSqlTags`, `schemaCatalogPath`, or `unanalyzableSql` on the rule. `include`, `exclude`, and `sqlInclude` may only narrow the project scope.

Flags executed PostgreSQL SQL that uses an `OFFSET` clause. Offset pagination
reads skipped rows again on every page and is usually the wrong default next to
cursor pagination, `LIMIT + 1`, `COUNT`, `EXISTS`, or `ROW_NUMBER()`.

The request boundary prepares embedded SQL and statement facts once. This rule
borrows those facts, including source locations, instead of reading or parsing
source again. Unparseable statements are skipped while recoverable sibling
statements remain checked; comments and prose are not clauses.

Each complete [SQL alternative](../postgres-facts.md#finite-sql-alternatives)
of a branching executor call is checked, up to 16 versions. An `OFFSET` in
only one path is still reported at its physical branch-fragment location;
all versions without OFFSET pass without an unanalyzable finding. Line
suppressions apply to those mapped locations. Opaque or over-cap calls retain
the unanalyzable behavior below.
An opaque falsy result from `condition && sqlFragment` followed by `??` stays
dynamic because the extractor cannot prove whether the fallback executes.
Passing a mutable SQL builder to an ordinary helper call without a verified
summary invalidates complete SQL across tracked aliases, including legacy
SQL fields. The same call evaluated in an `if` test, conditional test, or logical
left operand also clears the affected SQL prefix. Calls that remain dynamic use
the configured `unanalyzableSql` behavior.
A final template interpolation can mutate a fragment captured by an earlier
interpolation, because the tag receives those references after all expressions
run. Such calls stay dynamic and follow `unanalyzableSql`. Exact identities or
tracked alias groups distinguish captured objects from independent builders;
mutating a separate builder keeps every finite statement checked. A copied primitive
bind, an ordinary final bind, or an append to a fresh nested fragment keeps its
existing analysis. Static sibling calls still report OFFSET at the original
branch line, including when other calls become dynamic.
Pure identifier rebinding in the final interpolation keeps the earlier captured
fragment checked, including its OFFSET. Later reads can recover the replacement
while its snapshot remains available under the existing alias rules.
An assignment whose right-hand side mutates or escapes a builder still follows
`unanalyzableSql`.
This also applies when an append or mutating helper consumes the builder returned
by the assignment.

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
      unanalyzableSql: fail
```

`importSpecifier` has no default. `executorNames` defaults to `query`, `read`, and `write` only when `importSpecifier` is configured.
`unanalyzableSql` defaults to `fail` (`fail`, `ignore`, or `advise`; other values are a
configuration error).

An object or array destructuring assignment invalidates any earlier SQL
alternatives for the identifiers it overwrites. Until a fresh static assignment
is recoverable, the call follows `unanalyzableSql` instead of checking stale
SQL from before the assignment.

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
query(`SELECT id FROM posts ORDER BY id DESC LIMIT $1`, [limit + 1]);
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

### Unanalyzable SQL

See [Unanalyzable SQL](postgres-no-offset-dynamic.md) and
[mutable SQL builder recovery](../postgres-builder-mutations.md).

## Options and defaults

`unanalyzableSql: advise` reports residual uncertainty in non-failing advisories.
`--include-sql-verdicts` adds [per-subcheck SQL verdicts](../postgres-sql-verdicts.md).

`include` and `exclude` select source files. `sqlInclude` defaults to `[]`,
so `.sql` files are not scanned unless a glob selects them. `importSpecifier`
has no module default, and `executorNames` defaults to `[query, read, write]` only when `importSpecifier` is configured.
`unanalyzableSql` defaults to `fail`; `ignore` skips dynamic executor SQL that
cannot be checked instead of reporting it.

`OFFSET 0` is reported as an optimizer fence: use a `MATERIALIZED` CTE
(`WITH x AS MATERIALIZED (...)`). Any other offset keeps the pagination
message.

### Executor configuration

See [Executor configuration](postgres-no-offset-executors.md) and
[mutable SQL builder recovery](../postgres-builder-mutations.md).

## Valid example

```sql
CREATE VIEW view_recent_orders AS
WITH o AS MATERIALIZED (SELECT id, account_id FROM orders WHERE id > '0190')
SELECT o.id FROM o JOIN accounts a ON a.id = o.account_id;
```

```ts
query(`SELECT id FROM posts ORDER BY id DESC LIMIT $1`, [limit + 1]);
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
For an `unanalyzable` finding, make the executed SQL statically recoverable or
set `unanalyzableSql: ignore`.

## Suppression

Use `no-mistakes-disable-next-line postgres-no-offset` or
`no-mistakes-disable-line` (including for `unanalyzable` findings, which point
at the executor call); use the file directive only for an intentionally
offset-based reporting query.

SQL recovered from an imported helper is reported in the helper's module at the helper's line. A directive in the helper's file suppresses it, and so does a directive in the calling file (file-level, on the executor call, or on the imported-helper call). See [Helper-line attribution and suppression](../postgres-facts.md#helper-line-attribution-and-suppression).

## Related rules

[`postgres-lock-ordering`](postgres-lock-ordering.md) covers multi-row locks;
[`postgres-require-query-annotation`](postgres-require-query-annotation.md)
keeps executed SQL identifiable in logs.

## Statement coverage and locations

See [statement coverage and locations](postgres-no-offset-coverage.md).
