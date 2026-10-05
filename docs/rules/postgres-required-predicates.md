# `postgres-required-predicates`

Require configured predicates whenever a query reads a named PostgreSQL
relation. A relation may require textual predicate fragments, named columns,
or — when `partitionKeys` is `require` — every column of its catalog partition
key. Table names come from configuration and the schema catalog, not from
hardcoded conventions. With `relations` empty and `partitionKeys` left at
`off`, the rule reports nothing.

The rule reads prepared statement facts (`CheckFactPlan.postgres_dml`):
matching `.sql` files plus statically recoverable embedded executor SQL. It
does not re-parse SQL. Dynamic or unparseable SQL fails closed unless
`unanalyzableSql` is `ignore`.

```yaml
rules:
  - rule: postgres-required-predicates
    scope: repository
    options:
      sqlInclude: ["**/*.sql"]
      schemaCatalogPath: db/schema.json
      partitionKeys: require
      relations:
        - table: topics
          require:
            - parent_id IS NOT NULL
        - table: orders
          requireColumns: [account_id]
      unanalyzableSql: fail
```

## Why and when

Use this rule when some relations are unsafe to scan unbound (deleted rows,
unpublished rows, a required tenant column, or a partition key the planner
must see) and every SELECT, UPDATE, DELETE, and INSERT…SELECT must constrain
those columns.

## What it catches/requires

A standalone `TABLE topics` reads `topics` without a predicate and is checked
like `SELECT * FROM topics`. Each statement reports its own source line.

A configured `relations[].require` entry still matches the relation's
WHERE/JOIN SQL as a case-insensitive, whitespace-normalized substring. That
behavior is unchanged.

`relations[].requireColumns` applies to every base-table instance in SELECT
(including subqueries and CTE bodies), UPDATE, DELETE, and the SELECT of
`INSERT … SELECT`. Supported `TABLE` set-operation arms count as reads of
their named relation, including nested queries in UPDATE and DELETE expressions;
for example, `SELECT id FROM safe UNION ALL TABLE topics`
still requires configured predicates on `topics`. A column counts as constrained
when a top-level AND
conjunct is an equality, comparison, `IN`, `= ANY`, or `BETWEEN` on that
column. Self-referential `IN` lists do not constrain a column. `OR` counts only when every branch constrains it. `IS NULL`,
`IS NOT NULL`, `<>`, and `LIKE` do not. An unqualified name in a join counts
only when the schema catalog shows no other FROM relation has that column.

`partitionKeys: require` checks each instance whose catalog table is a
partitioned table. Every column of the partition key must be constrained;
a column keeps that role with a `COLLATE` or operator class, as in
`name COLLATE "C" text_pattern_ops`.
Expression partition keys are not guessed from queries: the rule reports one
catalog finding for that table, while column elements in a mixed key still require predicates. Missing partition keys produce a catalog finding. Tables in `partitionKeyExemptions` are
skipped. An exemption for a table that is not a partitioned catalog table is
stale. A relation may set `requireColumns` without `require`. MERGE is out of
scope.

## Options and defaults

`include` / `exclude` select source files (empty include means all files).
`sqlInclude` defaults to `**/*.sql`. `relations` defaults to `[]`. Each
relation's `require` and `requireColumns` default to `[]`. An empty
`requireColumns` string is a configuration error.
`schemaCatalogPath` defaults to unset and is required when `partitionKeys` is
`require`. Generate the catalog with [`no-mistakes postgres catalog`](../cli/postgres.md). `partitionKeys` defaults to `off` (`require` or `off`; any other
value is a configuration error). `partitionKeyExemptions` defaults to `[]`;
an empty table, an empty reason, or a duplicate table is a configuration
error. `allow` defaults to `[]` and suppresses catalog findings by object ref
(`table:<name>`); an invalid object reference, an empty reason, or a duplicate object is
a configuration error, and an unmatched entry is reported stale.
`unanalyzableSql` defaults to `fail` (`fail` or `ignore`; other values are a
configuration error). `importSpecifier` has no default.
`executorNames` defaults to `[query, read, write]` only when `importSpecifier` is configured.

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

Omitting both options is a configuration error. Set `executorNames: []` without
`importSpecifier` to explicitly skip executor calls (including `.query`).
A configured module or explicit `query` enables `.query` members. A configured module also recognizes
its transaction helpers. Native SQL and recovered SQL-builder fragments retain
their existing scopes. See [the migration notes](../migrations/explicit-postgres-executors.md).

## Valid example

```sql
SELECT id FROM topics WHERE parent_id IS NOT NULL AND id = $1;
SELECT id FROM events WHERE account_id = $1;
SELECT id FROM events WHERE account_id = ANY($1::uuid[]) AND kind = 'login';
UPDATE orders SET status = 'paid' WHERE account_id = $1 AND id = $2;
```

`INNER JOIN` `ON` predicates constrain both sides. Outer-join `ON` predicates count only for the non-preserved side; `FULL JOIN` does not constrain either side.
A schema-qualified `public.events` matches `events`. A CTE named `events` is
not the partitioned table. Quoted CTE and column names preserve their case; all sibling recursive CTE names are visible inside every body. Non-recursive bodies see only earlier CTE aliases; schema-qualified physical relations remain checked. Nested reads in query expressions, VALUES, RETURNING, and data-modifying CTEs are checked.

## Counterexample

```sql
SELECT id FROM topics WHERE id = $1;
SELECT id FROM events WHERE kind = 'login';
DELETE FROM orders WHERE id = $1;
```

## Fix

Add the required predicate or a comparison, `IN`, `= ANY`, or `BETWEEN` on
the required column. For a partition key, constrain every key column or add a
`partitionKeyExemptions` entry with a reason. For an expression partition key,
add that exemption instead of a query predicate.

## Suppression

SQL findings honor `no-mistakes-disable-next-line postgres-required-predicates`,
`no-mistakes-disable-line`, and `no-mistakes-disable-file`. Catalog findings
(expression partition keys) have no source line to comment, so suppress them
with `allow: [{object, reason}]`, for example `object: table:events`. For a qualified catalog name use its full object ref, such as `table:public.events`. SQL findings use each relation occurrence’s host-file line.

## Related rules

[`postgres-sql-shape-policy`](postgres-sql-shape-policy.md) bans correlated
`EXISTS` set operations; [`postgres-idempotent-insert`](postgres-idempotent-insert.md)
covers replay-safe INSERT.
