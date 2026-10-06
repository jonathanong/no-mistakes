# `postgres-generated-column-predicates`

Ban `WHERE`, `JOIN ON`, and `ORDER BY` uses of a generated column whose
expression is a configured function of the primary key, such as
`uuid_extract_timestamp(id)`.

```yaml
rules:
  - rule: postgres-generated-column-predicates
    options:
      include: ["src/**/*.ts"]
      sqlInclude: ["db/**/*.sql"]
      functions: [uuid_extract_timestamp]
      requireArgumentIsPrimaryKey: true
      clauses: [where, join, order-by]
      extraGeneratedColumns:
        - { table: events, column: created_at, sourceColumn: id }
      unanalyzableSql: ignore
```

## Why and when

A virtual generated timestamp has no btree of its own, and a stored copy needs
a second index. Filtering or sorting on `created_at` when it is
`uuid_extract_timestamp(id)` cannot use the primary-key index. Comparing or
ordering `id` against a UUIDv7 bound returns the same rows. Enable this on
application queries that read those generated columns.

## What it catches/requires

The rule reads bare column uses from statement facts and generated columns from
migration schema facts. It tracks a generated column when the expression is a
call to one of `functions`, that call has one bare column argument, and, by
default, that argument is the table's only primary-key column.
`extraGeneratedColumns` adds columns whose tables are created outside the
scanned SQL.

It reports a tracked column used in a comparison in `WHERE` or `JOIN ON`, or
used in `ORDER BY`, when that clause is enabled. It does not report a
SELECT-list use, `IS NULL` / `IS NOT NULL`, a column wrapped in a function, or
`GROUP BY`. An unqualified name that matches more than one joined relation is
not reported.

A standalone `TABLE orders ORDER BY created_at` is a query over `orders`, so
its ordering of a generated column is checked like a SELECT query.

With no options, the rule still tracks `uuid_extract_timestamp` of a
single-column primary key and checks `where`, `join`, and `order-by`. It
reports nothing until schema SQL defines such a column and a query uses it.

## Options and defaults

`include`, `exclude`, `sqlInclude`, `importSpecifier`, `executorNames`, and
`unanalyzableSql` match `postgres-no-generated-column-writes`. `sqlInclude`
defaults to `**/*.sql`. `unanalyzableSql` defaults to `fail`.

`functions` defaults to `[uuid_extract_timestamp]`. An empty list, an empty
name, or a duplicate name is a config error. `requireArgumentIsPrimaryKey`
defaults to `true`. `clauses` defaults to `[where, join, order-by]`. An empty
list, an empty name, a duplicate, or an unknown value is a config error.
`extraGeneratedColumns` defaults to `[]`. An empty table, column, or
`sourceColumn`, or a duplicate table and column, is a config error. An entry
that names a generated column already found in schema SQL is a stale finding.

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
| `executorFactoryNames` | Empty | Named imports (from `importSpecifier`, a relative path that resolves into that package, or any module when it is empty) whose call result bound with `const`, `let`, `using`, or `await using` is an executor inside the declaring block. |
| `executorTypeNames` | Empty | Imported type names (`import type` or inline `type` specifiers) from `importSpecifier`, a relative path that resolves into that package, or any module when it is empty, whose annotated parameters, including optional and inline-destructured ones, are executors inside the declaring function. |
| `trustedSqlTags` | Empty | Named imports of `name` from `module`, or a subpath of `module`, are parameterized SQL tags. A renamed local binding is trusted. A default import is not. A shadowed or rebound local fails closed. The same name from another module, or a sibling prefix such as `@example/dbx`, stays untrusted. |
| `reportUnmatchedExecutorNames` | `false` | Opt in to one finding per `executorFactoryNames` / `executorTypeNames` entry that none of the files this rule scans imports from `importSpecifier`, its subpaths, or a relative path that resolves into that package (a typo or a wrong module silently disables checking). Reported against the config file, so line suppression does not apply; remove the entry or leave this off to silence it. Rust rule only. See [Reporting unmatched names](../migrations/explicit-postgres-executors.md#reporting-unmatched-names). |

`executorFactoryNames` and `executorTypeNames` add scoped executors: `tx` in
`await using tx = await openTransaction()` or `run` in `run: TxExecutor` is scanned
only inside the block or function that declares it, and a same-named identifier
elsewhere is not. Matching is by configuration only, both default to empty, and
neither counts as executor selection: still set `importSpecifier` or `executorNames`.
With `importSpecifier` set, these two options also match imports from its subpaths
(`@example/db/types` for `@example/db`) but not sibling packages such as
`@example/dbx`. A relative import (`../transaction`) matches only when the resolved
file is inside the package `importSpecifier` resolves to; a same-named import outside
that package does not. If the package root cannot be determined, relative imports do
not match. `executorNames` still requires the exact module.

Omitting both options is a configuration error. Set `executorNames: []` without
`importSpecifier` to explicitly skip executor calls (including `.query`).
A configured module or explicit `query` enables `.query` members. A configured module also recognizes
its transaction helpers. Native SQL and recovered SQL-builder fragments retain
their existing scopes. See [the migration notes](../migrations/explicit-postgres-executors.md).

## Valid example

```sql
SELECT id, created_at FROM orders ORDER BY id DESC LIMIT 20;
SELECT id FROM orders WHERE id > $1;
SELECT id FROM orders WHERE created_at IS NULL;
SELECT min(created_at) FROM orders;
SELECT id FROM invoices ORDER BY created_at;
```

`invoices.created_at` is a plain column, so ordering by it is allowed.

## Counterexample

```sql
SELECT id FROM orders WHERE created_at > $1;
SELECT id FROM orders ORDER BY created_at DESC LIMIT 20;
SELECT o.id FROM orders o WHERE o.created_at BETWEEN $1 AND $2;
```

`orders.created_at` is `uuid_extract_timestamp(id)` and `id` is the primary key.

## Fix

Compare or sort the source primary key instead:

```sql
SELECT id FROM orders WHERE id > $1 ORDER BY id DESC LIMIT 20;
```

## Suppression

SQL and TypeScript honor `-- no-mistakes-disable-line`,
`-- no-mistakes-disable-next-line`, and `-- no-mistakes-disable-file` for
`postgres-generated-column-predicates`.

## Related rules

`postgres-no-generated-column-writes` bans writing generated columns.
`postgres-explicit-columns` bans `SELECT *` on wide tables. This rule only
cares about predicates and sorts on generated columns.

Query `include` and `exclude` globs select query inputs; they do not discard
SQL schema inputs selected by `sqlInclude`. All supported JS/TS extensions
are included by default. The rule stays quiet when no generated columns or
`extraGeneratedColumns` entries are tracked, even if SQL is dynamic.

The live catalog applies CREATE TABLE, ALTER TABLE ADD COLUMN, DROP TABLE,
and recreation in migration-file and statement order. Qualified relation
identities remain distinct, so `public.orders` and `audit.orders` do not
share generated-column metadata. Predicates on UPDATE and DELETE are
checked alongside SELECT. Simple ORDER BY projection aliases resolve to
their projected base column. JOIN predicates use only the relations visible
at that join; CTE and derived factors prevent unqualified ownership proof.

Findings use the predicate or sort expression's actual line, including
embedded call/declaration lines, so line-specific suppressions remain
applicable. Remediation is intentionally conditional: use the source column
only with bounds or ordering that preserve the configured generation
expression's semantics and an appropriate index. Custom functions and
non-primary-key source columns do not imply UUIDv7 or primary-key indexing.
Extra entries identify their configured source column without guessing which
function generated it.

The Rust statement facts expose `mutation_column_uses`; each column use
includes `candidate_tables` for its expression scope. Schema facts expose
ordered `SqlTableSchemaEvent` entries through `table_events`; existing
unqualified schema metadata retains its prior meaning. Aggregate and
standalone checks borrow these projections from one request fact pass.
