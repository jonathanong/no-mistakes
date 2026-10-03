# `postgres-bounded-statements`

Require a statement that can match many rows to take a `LIMIT`. A `SELECT` with no
bound, or an `UPDATE` or `DELETE` that touches every row matching a predicate, does
work that grows with the table: its time, lock footprint and memory have no ceiling.
The schema catalog knows every unique key, so the rule can tell a single-row lookup
from a statement that can match many rows.

```yaml
rules:
  - rule: postgres-bounded-statements
    options:
      include: ['src/**/*.ts']
      exclude: ['**/*.test.ts']
      schemaCatalogPath: db/schema.json
      statements: [select, update, delete]
      allow:
        - object: 'table:currencies'
          reason: 'Fixed lookup table of under 200 rows'
```

## Why and when

`SELECT id FROM invoices WHERE paid_at IS NULL` or
`UPDATE exports SET s3_key = NULL WHERE expires_at < now()` in a background job
touches every matching row in one statement. As data grows the statement gets slower,
holds its locks longer and buffers more. Pick a batch with a `LIMIT` (for an `UPDATE`
or `DELETE`, in a CTE or subquery that chooses the target rows) and run it in a loop
with a cap. Enable the rule on job and repository code where those sweeps live.

## What it catches/requires

The rule reads the statement facts of each executed `SELECT`, `UPDATE` and `DELETE`
in matching `.sql` files and in executor calls (`SELECT` is every query, including a
set operation). A statement is bounded when any of these holds:

- It has a `LIMIT` or `FETCH FIRST n ROWS ONLY` (a placeholder counts; `LIMIT NULL`,
  `LIMIT ALL`, `FETCH … WITH TIES` and `FETCH … PERCENT` do not: ties and a share of the
  rows are not a fixed count).
- It is a pure aggregate: a built-in aggregate call such as `COUNT(*)` (bare or
  `pg_catalog.`-qualified) with no `GROUP BY`, which returns one row. A function with the
  same name in another schema is an ordinary function, called once per row.
- Every base relation in its FROM list is **pinned** to a unique key, or comes from a bounded
  source. A relation is pinned when top-level `AND` conjuncts of `WHERE` and of the `ON`
  that restricts it equate **every** column of one unique key with a value:
  `col = $1`, `col IS NOT DISTINCT FROM $1` (only on a `NOT NULL` column: it also matches
  NULL, which a nullable unique column can repeat), `col = ANY($1::uuid[])` (bounded by the
  caller's array), `col IN (1, 2, 3)`, or `col IN (SELECT … LIMIT n)`. A value is a literal,
  a bind, or an interpolation recovered from a template literal (`${id}`, `${image.id}`,
  `${id}::uuid`), which is a bind like `$1`. A call to a built-in function that returns a
  different value for each row (`nextval`, `random`, `gen_random_uuid`, `clock_timestamp`
  and the like) is not a value; any other function call is assumed to be row-invariant,
  since a function's volatility is not part of the facts. A subquery that reads a column of
  the row being checked (`a.id IN (SELECT a.id)`) is not a pin, whatever its own bound: every
  row finds itself in it.
- A column compared with a column of another bounded relation is pinned too, so a bound
  propagates across joins on unique keys to a fixed point: with `o.id = $1`, the account
  `a.id = o.account_id` is bounded, and its profile `p.account_id = a.id` after it. A join on
  a column that is not a whole key ends the chain (an account has many orders). `ON` pins
  only the side an outer join can null-extend, never the preserved side, and a `WHERE` on the
  null-extended side is not assumed to turn the join inner: a `LEFT JOIN` with its key
  pinned only in `WHERE` reports the preserved relation. `JOIN … USING (col)` pins the same
  way as `ON a.col = b.col` when each side is one FROM item; after a longer chain, which
  item owns the column cannot be told, so it pins nothing.
- For `UPDATE` and `DELETE`, the target relation is bounded. A CTE or subquery that picks
  the target rows with a `LIMIT` bounds it when the target's key is matched against it
  (`FROM c WHERE t.id = c.id`, or `WHERE t.id IN (SELECT id … LIMIT n)`). `ctid` identifies
  one row of any table, so `WHERE ctid IN (SELECT ctid … LIMIT n)` bounds a statement, except
  on a partitioned table, whose leaves repeat `ctid` values. The relations joined to the
  target only feed it values: each target row changes once. The target is always the
  physical relation, even when a CTE of the same name exists.

A set operation (`UNION`) is bounded only when every arm is (a `TABLE name` arm is an
uncapped read of that relation), a derived table or CTE is bounded when its own query is,
and a `VALUES` list or table function (`unnest`) is sized by its own arguments. The rule
reports each relation that makes a statement unbounded, once, at that relation's line.
Statement kinds are judged independently: a data-modifying CTE is its own `UPDATE` or
`DELETE` (judged when `statements` includes it), and the `SELECT` that reads its
`RETURNING` rows is not unbounded because of it.

A **unique key** is a primary key, a unique constraint, or a unique index in the catalog
that is valid, ready, live and immediate, has no predicate, and whose keys are all plain
columns. These never prove one row:

- A partial unique index covers only some rows, and the rule does not try to prove that
  the statement implies the predicate, so it is not a key.
- An invalid or not-ready index (a failed concurrent build), and a deferrable constraint,
  which may hold duplicates inside a transaction, are not keys.
- An expression index is not matched by column equality.
- A **part** of a composite key is not a key.
- Equality alternatives (`id = $1 OR id = $2`), ranges and `LIKE` do not pin.

Relations the catalog does not describe (views, other schemas, relations created in the
same script) are not judged, and they bound nothing: an unknown relation can supply every
value of a column pinned to it, so a catalog table joined to one is still reported unless
something else bounds it. A statement whose SQL cannot be recovered statically (`SQL could
not be analyzed`, or `executed SQL is not statically recoverable`) fails closed.
`unanalyzableSql: ignore` skips those statements instead, as it does for the sibling
rules; every statement that can be analyzed is still judged.

## Options and defaults

`include`, `exclude`, `importSpecifier`, `executorNames` and `unanalyzableSql` match
`postgres-explicit-columns`. `unanalyzableSql` defaults to `fail`. `sqlInclude`
defaults to `[]`, so `.sql` files are not scanned until you set globs.
`schemaCatalogPath` has no default, is required, and names a catalog generated with
[`no-mistakes postgres catalog`](../cli/postgres.md); an ordering-only catalog is
rejected like it is for every other rule that needs complete facts. `statements`
defaults to `[select, update, delete]`; an empty list, an unknown kind or a duplicate is
a configuration error. `allow` defaults to `[]`: each entry is `{object, reason}`
with an object such as `table:currencies`. An empty reason, an object that is not a
catalog ref, or a duplicate is a configuration error, and an entry that suppresses
nothing is reported as stale. The catalog's name for the table is the object ref.

## Valid example

```sql
SELECT id FROM invoices WHERE paid_at IS NULL ORDER BY id LIMIT $1;
SELECT name FROM accounts WHERE id = $1;
SELECT id FROM orders WHERE id = ANY($1::uuid[]);
SELECT COUNT(*) FROM orders WHERE account_id = $1;
WITH c AS (SELECT id FROM exports WHERE expires_at < now() ORDER BY id LIMIT $1 FOR UPDATE SKIP LOCKED)
  UPDATE exports SET s3_key = NULL FROM c WHERE exports.id = c.id;
SELECT code FROM currencies;   -- allow entry
```

## Counterexample

```sql
SELECT id FROM invoices WHERE paid_at IS NULL;
UPDATE exports SET s3_key = NULL WHERE expires_at < now();
DELETE FROM sessions WHERE expires_at < now();
```

## Fix

Add a `LIMIT` and loop with a cap. For `UPDATE` and `DELETE`, pick the rows first:

```sql
DELETE FROM sessions
WHERE id IN (SELECT id FROM sessions WHERE expires_at < now() ORDER BY id LIMIT $1);
```

If the table is small and fixed, add an `allow` entry with a reason.

## Suppression

SQL and TypeScript honor `-- no-mistakes-disable-line`, `-- no-mistakes-disable-next-line`
and `-- no-mistakes-disable-file` for `postgres-bounded-statements`. To exempt a table
everywhere, use `allow: [{object: 'table:<name>', reason}]`.

## Related rules

[`postgres-explicit-columns`](postgres-explicit-columns.md) and
[`postgres-required-predicates`](postgres-required-predicates.md) read the same prepared
statement facts. [`postgres-no-offset`](postgres-no-offset.md) bans the `OFFSET` paging
that often replaces a bounded keyset sweep.
