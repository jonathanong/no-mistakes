# `postgres-generated-column-predicates`

Ban `WHERE`, `JOIN ON`, and `ORDER BY` uses of a generated column whose
expression is a configured function of the primary key, such as
`uuid_extract_timestamp(id)`.

```yaml
rules:
  - rule: postgres-generated-column-predicates
    options:
      include: ['src/**/*.ts']
      sqlInclude: ['db/**/*.sql']
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

## Valid example

```sql
SELECT id, created_at FROM orders ORDER BY id DESC LIMIT 20;
SELECT id FROM orders WHERE id > $1;
SELECT id FROM orders WHERE created_at IS NULL;
SELECT date_trunc('day', created_at) AS d, COUNT(*) FROM orders GROUP BY d;
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
