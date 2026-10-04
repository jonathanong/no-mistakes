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

Temporary relations are tracked within each SQL source in statement order. Tables declared `ON COMMIT DROP` lose their temporary identity at commit, including after a rename; `ON COMMIT DELETE ROWS` retains the identity. Transaction rollback and savepoints restore their identities; repeating `BEGIN` preserves the active transaction and its savepoints. Renames, drops, and cascading drops update temporary view dependencies. A view over temporary relations is itself temporary. Quoted `TABLE "Accounts"` arms retain their case-sensitive identity, including inside a view; unquoted `TABLE Accounts` arms use the folded `accounts` identity. Explicit `search_path` order controls whether an unqualified name shadows the catalog relation, while `pg_temp` qualification selects the temporary identity.

## What it catches/requires

The rule reads the statement facts of each executed `SELECT`, `UPDATE` and `DELETE`
in matching `.sql` files and in executor calls (`SELECT` is every query, including a
set operation). A statement is bounded when any of these holds:

- It has a `LIMIT` or `FETCH FIRST n ROWS ONLY` whose count is fixed: a literal, a bind
  (`$1`, `${size}`) or an expression of them using `COALESCE`, `LEAST` or `GREATEST`
  (`LEAST($1, 100)`). `LIMIT NULL`, `LIMIT ALL`, any other function (`NULLIF(1, 1)` is NULL),
  a count taken from the data (`LIMIT (SELECT count(*) …)`), `FETCH … WITH TIES` and
  `FETCH … PERCENT` do not cap: they can return every row. A bind is taken as the
  caller's cap: a caller that passes NULL at run time gets `LIMIT ALL`, which no statement
  text can rule out, so validate the number before the call, or write `COALESCE($1, 100)`.
- It is a pure aggregate: a built-in aggregate call such as `COUNT(*)` (bare or
  `pg_catalog.`-qualified), in the select list, in `HAVING` or in `ORDER BY`, with no
  `GROUP BY`, which returns one row. A function with the same name in another schema is an ordinary function,
  called once per row, and a set-returning function in the select list
  (`generate_series(1, count(*))`) expands the row again.
- Every base relation in its FROM list is **pinned** to a unique key, or comes from a bounded
  source. A relation is pinned when top-level `AND` conjuncts of `WHERE` and of the `ON`
  that restricts it equate **every** column of one unique key with a value:
  `col = $1`, `col IS NOT DISTINCT FROM $1` (only on a `NOT NULL` column: it also matches
  NULL, which a nullable unique column can repeat), `col = ANY($1::uuid[])` (bounded by the
  caller's array; finite constructors (including array casts) over catalog-proven scalar columns of bounded rows also qualify. Signed numeric literals, builtin XML literals, and typed scalar literals such as `DATE '2026-01-01'` and interval literals preserve finite cardinality. Catalog-declared enum casts also qualify; unknown or array-bearing domains do not. A cast that decodes stored scalar text into an array supplies no finite bound; caller-provided arrays and casts of finite constructors retain their bounds. Builtin scalar types, including `pg_catalog` network types such as `inet`, and catalog-declared enums prove scalar leaves; positional aliases map through known visible column order. Unchanged key names retain credit, and PostgreSQL system columns also prove scalar leaves when aliases do not replace them. Scalar indexes into catalog-proven arrays of scalar elements, such as `ARRAY[o.account_ids[1]]`, preserve the bounded source-row dependency; slices and unknown/domain element types do not. Stored arrays, including `ARRAY[stored_array]`, supply no key bound), `col IN (1, 2, 3)`, or `col IN (SELECT … LIMIT n)`. A value is a literal,
  a bind, or an interpolation recovered from a template literal (`${id}`, `${image.id}`,
  `${id}::uuid`), which is a bind like `$1`. A call to a built-in function that returns a
  different value for each row (`random`, `gen_random_uuid`, `clock_timestamp`, the
  sequence functions and the like) is not a value; any other function call is assumed to be row-invariant,
  since a function's volatility is not part of the facts. A value with an explicit collation
  (`email = $1 COLLATE "C"`) is not a value either: it changes what `=` matches. A subquery
  that reads a column of the row being checked (`a.id IN (SELECT a.id)`) is not a pin, whatever
  its own bound: every row finds itself in it. References are resolved one query level at a
  time, so a relation of a nested level never hides a reference of an outer one.
  Schema-qualified references retain their schema: an inner `audit.accounts` cannot
  hide a reference to the outer `public.accounts`. A bare column
  belongs to the first level whose tables have it, as PostgreSQL resolves it: in
  `a.id IN (SELECT id FROM currencies LIMIT 1)` the `id` is the account's own when the catalog's
  `currencies` has no `id` column, so that subquery is not a pin either. A column of a derived
  table, a function or a CTE is taken as that source's own, and so is any column of a table
  the catalog does not describe. A `LATERAL` source that reads
  earlier FROM items is sized per row of them, so it bounds nothing pinned to it; the
  relations inside it are still judged.
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
  one row of a plain table, so `WHERE ctid IN (SELECT ctid … LIMIT n)` bounds a statement,
  except on a partitioned table, whose leaves repeat `ctid` values, and on a relation the
  catalog does not describe, which may be one. The relations joined to the
  target only feed it values: each target row changes once. The target is always the
  physical relation, even when a CTE of the same name exists.

A set operation (`UNION`) is bounded only when every arm is (a `TABLE name` arm is an
uncapped read of that relation, or of the CTE it names), a derived table or CTE is bounded when its own query is,
and a `VALUES` list or a set-returning built-in (bare or `pg_catalog.`-qualified) over
arguments the statement supplies (`unnest($1)`, `generate_series(1, 10)`) is sized by the
caller. Any other table function (`FROM get_all_accounts()`, `app.generate_series(…)`), one
whose arguments read a column or a query (`unnest(ARRAY[a.id])`), an array taken from a query
(`ANY(ARRAY(SELECT …))`) and the recursive reference of a `WITH RECURSIVE` are opaque: never
reported themselves, and they bound nothing pinned to them. `EXCEPT` and `INTERSECT` read
both arms in full like `UNION`, so every arm must be bounded: the rule bounds the work, not
only the result. An outer LIMIT does not suppress the input-arm findings for EXCEPT,
INTERSECT or duplicate-eliminating UNION; UNION ALL can stream until its cap.
A zero cap skips every input, including `FETCH FIRST 0 ROWS WITH TIES` and zero
inside parentheses, unary signs, or built-in numeric casts such as
`CAST(0 AS bigint)`. Custom casts do not prove zero.
A chain of CTEs whose bounds grow past a few thousand items is compacted to its
distinct uncapped base-relation reads plus an opaque source. This retains findings
while preventing the summary from bounding another relation, so pathological generated
SQL stays cheap to analyze. A `COPY (SELECT …)` query is judged like a `SELECT`. The rule
reports each relation that makes a statement unbounded, once, at that relation's line.
An uncapped SELECT also reports unbounded relations read by its IN-subquery pins,
even when another predicate pins the outer table. UPDATE and DELETE retain target-only
reporting.
Statement kinds are judged independently: a data-modifying CTE is its own `UPDATE` or
`DELETE` (judged when `statements` includes it), and the `SELECT` that reads its
`RETURNING` rows is not unbounded because of it. Those rows are opaque, though: one row per
modified row is not sized by the statement text, so they bound nothing pinned to them
(`DELETE FROM orders USING moved WHERE orders.id = moved.id` is reported when `moved` is a
data-modifying CTE).

A **unique key** is a primary key, a unique constraint, or a unique index in the catalog
that is valid, ready, live and immediate, has no predicate, and whose keys are all plain
columns. These never prove one row:

- A partial unique index covers only some rows, and the rule does not try to prove that
  the statement implies the predicate, so it is not a key.
- An invalid or not-ready index (a failed concurrent build), and a deferrable constraint,
  which may hold duplicates inside a transaction, are not keys.
- An expression index is not matched by column equality.
- An index whose operator class or collation is not the column's default (the catalog's
  `orderingSupported` is false) may treat values as distinct that `=` on the column treats
  as equal, so it is not a key.
- A **part** of a composite key is not a key.
- Equality alternatives (`id = $1 OR id = $2`), ranges and `LIKE` do not pin.

The bound covers SELECT relation reads, including explicit subqueries, and the
target-row selection of UPDATE and DELETE. DML source scans are not independently
bounded: `DELETE FROM accounts USING (SELECT * FROM orders) o WHERE accounts.id = $1`
passes because its target is uniquely pinned, even though the source reads all orders.
Data-modifying CTEs are judged as their own statements when their kind is enabled.
The bound does not include work triggered by foreign-key referential actions (`ON DELETE`
or `ON UPDATE CASCADE`, `SET NULL`, or `SET DEFAULT`) or triggers. A primary-key deletion
can therefore pass while cascading to arbitrarily many child rows, including through
multiple levels. `NO ACTION` or `RESTRICT` does not change this scope. Audit referential
actions and trigger bodies separately; a bound on the parent does not bound their work.
There is currently no option to include those implicit rows.

Table inheritance (`INHERITS`) is not modeled: the unique key of a parent is assumed to
hold across its children, although PostgreSQL does not enforce it there. Partitioning is
modeled; use it instead of inheritance, or keep inheritance parents out of this rule.

Relations the catalog does not describe (views, relations created in the same script) and
relations spelled with another schema (`audit.accounts` when the catalog is for `public`)
are not judged, and they bound nothing. A relation is found by the name PostgreSQL reads:
an unquoted name folds to lower case and a quoted one (`"Order Items"`) is exact: an unknown relation can supply every
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

A line directive covering the first line of a statement suppresses every relation
finding in that statement, including when its `FROM` or target is on a later line.
Directives on a relation's own reported line also work. This applies to SQL files
and SQL embedded in executor calls; statement-start directives use the physical
source line of the SQL keyword.

## Related rules

[`postgres-explicit-columns`](postgres-explicit-columns.md) and
[`postgres-required-predicates`](postgres-required-predicates.md) read the same prepared
statement facts. [`postgres-no-offset`](postgres-no-offset.md) bans the `OFFSET` paging
that often replaces a bounded keyset sweep.

Oversized CTE expansions are compacted to their distinct uncapped base relations.
This conservative summary retains unbounded reads and supplies no bound to joined
items; an explicit outer `LIMIT` still caps the statement. Caller-sized pins survive
only when every uncapped read of the relation has the same pin and no positional
column aliases. Aliased reads lose pin credit because compaction discards the alias
identity needed to prove that a visible name still names the catalog key.

Built-in set-returning functions, including `jsonb_path_query`, expand aggregate
select-list rows. In `FROM`, only functions whose arguments size their result
(`jsonb_path_query($1::jsonb, '$[*]')`, for example) bound joined items. Functions
that inspect server state, parser metadata, or execute SQL (`pg_ls_dir`,
`pg_listening_channels`, `ts_token_type`, `ts_stat`) remain opaque even with fixed
arguments. The real PostgreSQL test lane checks this inventory against the
PostgreSQL function catalog.

Snapshot expansion (`pg_snapshot_xip`, `txid_snapshot_xip`) is caller-sized only
when its snapshot is supplied directly by the caller. A snapshot returned by
`pg_current_snapshot()`, `txid_current_snapshot()`, or another SQL function is opaque.
The same caller-supplied requirement applies to other caller-sized set-returning
functions, including calls with PostgreSQL named arguments. Catalog functions
known to return one row, such as `pg_stat_get_recovery_prefetch()`, preserve a
pure aggregate's one-row cap.

Temporary tables and views created earlier in the same SQL source shadow unqualified
catalog relations until `DROP TABLE` or `DROP VIEW`. This includes `SELECT INTO TEMP`.
Temporary relations are unknown and bound no joined items; qualified permanent
relations retain their catalog identity. State resets for every SQL source.

Temporary `SELECT INTO` destinations are tracked through `UNION`, `INTERSECT`, and `EXCEPT`, including a parenthesized first input. The input reads are checked before the new temporary name shadows a permanent catalog relation.

`CREATE TABLE pg_temp.name` creates a temporary identity even without the `TEMP` keyword. Quoted schema names retain PostgreSQL case and component boundaries, so a different schema or a literal dot in one identifier does not acquire temporary identity.

Compacted caller-only array pins retain their scalar cast evidence for catalog validation. A pin with any source-row or scalar-column dependency is discarded; unknown cast types never gain a bound from compaction.

A base-table column alias list (`FROM accounts AS a(id, real_id)`) renames columns by
position. Only keys whose aliased positions keep their original names retain catalog key credit; ordinary table aliases
(`FROM accounts AS a`) retain key matching. This also applies to join conditions and
`USING` pins.

For `TABLE` set-operation arms, the SQL parser omits identifier quote information.
Analysis recovers the spelling from the prepared source tokens and matches quoted
names exactly. If source tokens are unavailable, it conservatively checks both
exact and folded spellings when they differ. A matching one-part CTE takes
precedence for its spelling only.

Bare subquery columns belong to a derived table or CTE only when its explicit
projection exposes that name. Function column alias lists and known scalar built-in
output names also determine ownership. Unknown functions and wildcard projections
retain conservative unknown-column ownership.

Known scalar table functions expose their default position column when requested. UNNEST column names are inferred only for syntactically proven scalar arrays; composite and unknown element layouts retain unknown ownership. Parenthesized joins with wrapper aliases also retain unknown ownership when the child labels cannot describe the renamed output.
