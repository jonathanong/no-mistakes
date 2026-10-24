# `postgres-no-transformed-uuid-predicates`

A `projects: [<name>]` entry inherits [`projects.<name>.postgres`](../configuration/projects.md#postgresql-scope) when that block is set. Do not restate `importSpecifier`, `executorNames`, `executorFactoryNames`, `executorTypeNames`, `trustedSqlTags`, `schemaCatalogPath`, or `unanalyzableSql` on the rule. `include`, `exclude`, and `sqlInclude` may only narrow the project scope.

Require catalog-resolved UUID columns to stay bare in `WHERE`, `JOIN ON`, and
`HAVING` predicates. This preserves indexable comparisons against UUID
parameters or bounds.

```yaml
rules:
  - rule: postgres-no-transformed-uuid-predicates
    scope: repository
    options:
      schemaCatalogPath: db/schema.json
      sqlInclude: ["db/**/*.sql"]
      unanalyzableSql: ignore
```

## Why and when

An index on a UUID key stores UUID values. Casting the column to text or
applying a function to it can prevent PostgreSQL from using that index for the
predicate. For UUIDv7 timestamps, compute a UUID range bound before the query
and compare the indexed key directly.

Enable this rule on application SQL when key lookups and range scans should use
their UUID indexes.

## What it catches/requires

The rule reports a cast or function call applied to a catalog-resolved UUID
column when that expression participates in a predicate in `WHERE`, `JOIN ON`,
or `HAVING`. It covers UUID primary keys, foreign keys, unique keys, and
ordinary UUID columns, including columns with plain indexes. The catalog's
column type determines the scope; a conventional name such as `id` does not.

The rule checks only the key side of a predicate. Casting a parameter to UUID
or computing a bound before comparing it to a bare key column is allowed. A
transformed key used only in the `SELECT` list is also allowed because it does
not constrain the scan.

## Analysis

Analysis uses schema facts from the required `schemaCatalogPath` together with
parsed SQL statement facts. Relation and column ownership must resolve to a
catalog column before its type can be checked. Qualified references are
resolved against the visible relations, and ambiguous references are left
unreported. It evaluates the expression's use in the predicate rather than
flagging every function or cast that mentions a UUID column.

The rule does not prove whether a transformed expression has a matching
expression index. When such an index is intentional, keep the query and schema
design explicit and suppress the finding with a reason. SQL syntax or dynamic
query text that cannot be analyzed follows the configured `unanalyzableSql`
policy.

## Valid example

```sql
-- The indexed key remains bare; casts and bounds are on the parameter side.
SELECT id FROM users WHERE id = $1::uuid;
SELECT id FROM users WHERE id >= uuidv7_lower_bound($1);

-- A projection can derive a timestamp without wrapping a predicate key.
SELECT uuid_extract_timestamp(id) FROM users WHERE id = $1;
```

## Counterexample

```sql
SELECT id FROM users WHERE id::text = $1;
SELECT id FROM users WHERE CAST(id AS text) = $1;
SELECT id FROM users WHERE uuid_extract_timestamp(id) > $1;

SELECT s.id
FROM sessions s
JOIN users u ON s.user_id::text = u.id::text;

SELECT user_id, count(*) FROM sessions
GROUP BY user_id
HAVING uuid_extract_timestamp(user_id) > $1;
```

## Fix

Keep the indexed UUID key bare and cast the parameter or compute a UUID bound
before issuing the query. For timestamp ranges over UUIDv7 keys, translate the
timestamp interval to UUID bounds in application code, then compare the key:

```sql
SELECT id FROM users WHERE id >= $1 AND id < $2;
```

## Suppression

Use `no-mistakes-disable-line`, `no-mistakes-disable-next-line`, or
`no-mistakes-disable-file` for `postgres-no-transformed-uuid-predicates`.
Include a reason when an expression index or another deliberate query plan
justifies transforming a key column.

## Related rules

`postgres-generated-column-predicates` checks generated timestamp columns
that derive from UUID primary keys. This rule checks transformations of
catalog-resolved UUID columns directly in predicates.
