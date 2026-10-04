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
      include: ["src/**/*.ts"]
      exclude: ["**/*.test.ts"]
      schemaCatalogPath: db/schema.json
      statements: [select, update, delete]
      allow:
        - object: "table:currencies"
          reason: "Fixed lookup table of under 200 rows"
```

## Why and when

`SELECT id FROM invoices WHERE paid_at IS NULL` or
`UPDATE exports SET s3_key = NULL WHERE expires_at < now()` in a background job
touches every matching row in one statement. As data grows the statement gets slower,
holds its locks longer and buffers more. Pick a batch with a `LIMIT` (for an `UPDATE`
or `DELETE`, in a CTE or subquery that chooses the target rows) and run it in a loop
with a cap. Enable the rule on job and repository code where those sweeps live.

Temporary relations are tracked within each SQL source in statement order. Tables declared `ON COMMIT DROP` lose their temporary identity at commit, including after a rename; `ON COMMIT DELETE ROWS` retains the identity. Transaction rollback and savepoints restore their identities; repeating `BEGIN` preserves the active transaction and its savepoints. Renames, drops, and cascading drops update temporary view dependencies, including dependencies on permanent relations. View dependencies include relations in scalar `SELECT`-list and `WHERE` subqueries, including parenthesized `TABLE relation` queries such as `SELECT (TABLE "Helper" LIMIT 1)`; CTE aliases and table-function names are not relations. A recursive `WITH` resolves acyclic forward CTE references before projecting their bounds. Qualified permanent dependencies retain their schema identity; unqualified dependencies conservatively match a schema-qualified drop with the same relation name. A view over temporary relations is itself temporary. Quoted `TABLE "Accounts"` arms retain their case-sensitive identity, including inside a view; unquoted `TABLE Accounts` arms use the folded `accounts` identity. Explicit `search_path` order controls whether an unqualified name shadows the catalog relation, while `pg_temp` qualification selects the temporary identity.

`DROP MATERIALIZED VIEW ... CASCADE` removes temporary views that depend on the
materialized view and their temporary dependents; `RESTRICT` preserves their
identities. Schema-qualified materialized-view names remain distinct from
temporary namesakes.

Permanent-only views and materialized views declared earlier in the same SQL source
remain ordinary catalog relations. Their dependency records let a physical
`DROP ... CASCADE` retire temporary views through several permanent intermediaries.
`RESTRICT` and a drop of a differently qualified namesake leave those temporary
identities intact; an unqualified view declaration has unknown schema ownership
and is retired conservatively on a schema cascade.
Schema renames update a declared permanent view's qualified identity and source
edges before a later physical cascade. PostgreSQL's `ALTER TABLE ... RENAME TO`
form also renames ordinary and materialized views: their declared node identities
move with their dependency edges, so a later cascade still retires temporary
dependents. Qualified names preserve their schema; ambiguous bare declarations
retain both candidates. A rename onto another known qualified view preserves the
colliding source identity and edges. With a bare source, each schema candidate is checked separately:
a collision in a later schema does not prevent a valid earlier-schema rename.
An exact qualified collision leaves the graph unchanged.

An explicitly parsed `search_path` also limits which physical schemas a bare
`DROP` or `RENAME` target can name. For example, with `public, pg_temp`, dropping
bare `accounts` leaves a temporary view over `other.accounts` intact. Qualified
targets still match their exact schema, and transitive cascade nodes retain their
identity after the path changes. Implicit `pg_catalog` and `pg_temp` remain eligible.
Each SQL `SET search_path` value names one schema: `'empty,schema', pg_temp`
keeps the comma in the first schema, while `'empty,schema, pg_temp'` names just
one schema. String values retain case and embedded double quotes; directly
quoted identifiers use SQL identifier escaping. Unknown paths, role
substitution, `DEFAULT`, and unsupported expressions retain conservative
matching; membership alone does not prove which eligible
schema owns an ambiguous bare target. Transaction, savepoint, and `SET LOCAL`
restoration retain the complete path proof.

`DROP SCHEMA ... CASCADE` retires temporary views depending on qualified physical
relations in that schema, including transitive temporary dependents. Exact decoded
schema identifiers preserve quoted case and dots; unrelated schemas stay live.
Transaction and savepoint rollback restore the dependency state. Bare dependencies
have no proven schema identity and retain conservative matching behavior. A schema
drop without `CASCADE` does not retire these dependent temporary views.

PostgreSQL does not support temporary materialized views. A materialized-view drop
that resolves to a known temporary table or ordinary view fails for the wrong
relation kind, so that temporary identity remains live if execution continues.

Temporary partitions follow their temporary parent: `DROP TABLE parent` removes a
partition created with `CREATE TEMP TABLE child PARTITION OF parent`, even without
`CASCADE` when the drop succeeds. A known temporary view depending on the child
blocks a non-cascading parent drop; the whole rejected `DROP` leaves temporary
relations intact, including other named targets. Failures needing a live catalog
cannot always be inferred from source SQL.
`ALTER TABLE parent DETACH PARTITION child` removes that ownership link,
including the `CONCURRENTLY` and `FINALIZE` forms, so a later parent drop leaves the child
and views over it intact; `ATTACH PARTITION child FOR VALUES ...` restores the link.
When an explicit `search_path` puts schemas before `pg_temp`, catalog evidence
resolves both the parent and child of an unqualified `ATTACH` or `DETACH`. An
earlier schema proven empty permits the temporary transition; an earlier
physical namesake selects the physical tree instead. Unknown earlier schemas
leave temporary ownership conservative, while explicit `pg_temp` names select
the temporary tree. Transaction rollback restores its prior ownership.
PostgreSQL does not allow temporary and permanent tables in the same partition tree.
For SQL sources containing these `ALTER TABLE` forms, the lenient parser recovers complete
partition transitions even though the strict PostgreSQL parser does not support them;
this also covers transitions inside `DO` blocks. Malformed transitions are skipped while
other parseable statements are still checked. Attach recovery requires nonempty list
and range expression entries, matching lower and upper range arity, and a valid hash
`MODULUS`/`REMAINDER` pair in either order; rejected bounds leave temporary ownership unchanged.
`CREATE TEMP TABLE IF NOT EXISTS child
PARTITION OF parent` leaves an existing standalone child untouched.
An attach to a non-partitioned temporary parent does not transfer ownership. A
`DETACH PARTITION ... CONCURRENTLY` inside `DO` or an explicit transaction cannot
run in PostgreSQL, so recovered SQL in those contexts leaves the partition attached.

`PREPARE` analyzes a `SELECT INTO TEMP` without creating its destination. The temporary identity begins when its `EXECUTE` runs with the prepared argument count; `DEALLOCATE` before execution leaves the catalog relation visible. A duplicate `PREPARE` keeps the original definition because PostgreSQL rejects the duplicate. A type list may declare only a prefix: later `$n` parameters extend the execution arity, while unused declared types still count. Wrong-arity executions and malformed placeholders do not create a temporary destination. Prepared-statement names use PostgreSQL's default 63-byte identifier limit without splitting UTF-8 characters, consistently across declaration, execution, and deallocation; quoted case remains significant.

Three-part temporary names such as `"Audit.Database".pg_temp.accounts` retain
their database qualifier in prepared facts. They shadow a permanent catalog
relation only when the catalog's optional `currentDatabase` exactly matches that
decoded database name. A missing identity or another database keeps the
conservative permanent-relation check. Quoted dots stay within their identifier;
`pg_temp_like` and `"pg_temp.accounts"` are different schema names. The condition
follows dependent temporary views, partition ownership, renames, drops,
transactions, and savepoints.
Catalog generation records `currentDatabase` in the same snapshot query.

When an explicit `search_path` places schemas before `pg_temp`, a generated catalog can
prove whether those schemas exist and which relation names they contain. Generate it
with `--search-path-schema` for each earlier schema. A missing or incomplete evidence
entry leaves the rule conservative; an earlier relation outside the selected catalog
is treated as unknown, rather than as a namesake table from the selected schema.
If an unqualified `DROP` removes a proven earlier physical relation, a surviving
temporary namesake becomes visible to later reads, including after `RESET search_path`.
Complete evidence that earlier schemas lack the name instead makes the drop reach
the temporary relation. With incomplete evidence, the rule does not use catalog
unique keys from a possibly temporary source to bound a joined table.

## What it catches/requires

The rule reads the statement facts of each executed `SELECT`, `UPDATE` and `DELETE`
in matching `.sql` files and in executor calls (`SELECT` is every query, including a
set operation). A statement is bounded when any of these holds:

- It has a `LIMIT` or `FETCH FIRST n ROWS ONLY` whose count is fixed: a literal, a bind
  (`$1`, `${size}`) or an expression of them using `COALESCE`, `LEAST` or `GREATEST`
  (`LEAST($1, 100)`). Literal NULL arguments are allowed when another supported
  argument supplies a guaranteed non-NULL count: `COALESCE(NULL, 100)`,
  `LEAST(NULL, 100)`, and `GREATEST(100, NULL)` stay capped. An all-NULL result or
  `COALESCE($1, NULL)` does not cap; unknown or data-derived arguments remain
  conservative even after a known fallback. These conditional constructs must
  be unqualified and unquoted;
  qualified or quoted function lookalikes may return NULL and are not fixed caps. `LIMIT NULL`, `LIMIT ALL`, any other function (`NULLIF(1, 1)` is NULL),
  a count taken from the data (`LIMIT (SELECT count(*) …)`), `FETCH … WITH TIES` and
  `FETCH … PERCENT` do not cap: they can return every row. A bind is taken as the
  caller's cap: a caller that passes NULL at run time gets `LIMIT ALL`, which no statement
  text can rule out, so validate the number before the call, or write `COALESCE($1, 100)`.
  An inner `LIMIT 0` or `FETCH FIRST 0 ROWS ONLY` stays empty when a surrounding
  parenthesized query later applies a set-returning `ORDER BY` expression. Positive,
  absent, and NULL inner caps do not establish an empty expanded result.
- It is a pure aggregate: a built-in aggregate call such as `COUNT(*)` (bare or
  `pg_catalog.`-qualified), in the select list, in `HAVING` or in `ORDER BY`, with no
  `GROUP BY`, which returns one row. HAVING without an aggregate call also introduces
  one implicit group when there is no GROUP BY and the select list does not expand rows. A function with the same name in another schema is an ordinary function,
  called once per row, and a set-returning function in the select list
  (`generate_series(1, count(*))`) expands the row again.
- Every base relation in its FROM list is **pinned** to a unique key, or comes from a bounded
  source. A relation is pinned when top-level `AND` conjuncts of `WHERE` and of the `ON`
  that restricts it equate **every** column of one unique key with a value:
  `col = $1`, `col IS NOT DISTINCT FROM $1` (only on a `NOT NULL` column: it also matches
  NULL, which a nullable unique column can repeat), `col = ANY($1::uuid[])` (bounded by the
  caller's array; finite constructors (including array casts) over catalog-proven scalar columns of bounded rows also qualify. Signed numeric literals, builtin XML literals, and typed scalar literals such as `DATE '2026-01-01'` and interval literals preserve finite cardinality. Fixed scalar boolean expressions such as `ARRAY[NULL IS NULL]` are also one caller-sized element; expressions that read columns or call functions remain opaque. Catalog-declared enum casts also qualify; unknown or array-bearing domains do not. A cast that decodes stored scalar text into an array supplies no finite bound; caller-provided arrays and casts of finite constructors retain their bounds. Builtin scalar types, including `pg_catalog` network types such as `inet`, and catalog-declared enums prove scalar leaves; positional aliases map through known visible column order. Unchanged key names retain credit, and PostgreSQL system columns also prove scalar leaves when aliases do not replace them. Scalar indexes into catalog-proven arrays of scalar elements, such as `ARRAY[o.account_ids[1]]`, preserve the bounded source-row dependency; slices and unknown/domain element types do not. Stored arrays, including `ARRAY[stored_array]`, supply no key bound), `col IN (1, 2, 3)`, or `col IN (SELECT … LIMIT n)`. A value is a literal,
  a bind, or an interpolation recovered from a template literal (`${id}`, `${image.id}`,
  `${id}::uuid`), which is a bind like `$1`. A call to a built-in function that returns a
  different value for each row (`random`, `gen_random_uuid`, `clock_timestamp`, the
  sequence functions and the like) is not a value; any other function call is assumed to be row-invariant,
  since a function's volatility is not part of the facts. A value with an explicit collation
  (`email = $1 COLLATE "C"`) is not a value either: it changes what `=` matches. A subquery
  that reads a column of the row being checked (`a.id IN (SELECT a.id)`) is not a pin, whatever
  its own bound: every row finds itself in it. References are resolved one query level at a
  time, so a relation of a nested level never hides a reference of an outer one.
  An explicit table alias hides the table's original name from nested scopes, even when
  it repeats the table's bare name: `FROM public.accounts accounts` exposes `accounts`
  as the alias but no longer exposes `public.accounts`. Schema-qualified references
  retain their schema: an inner `audit.accounts` cannot hide a reference to the outer
  `public.accounts`. Quoted dots stay inside one identifier:
  a table or alias named `"public.accounts"` cannot hide that schema-qualified reference. A bare column
  belongs to the first level whose tables have it, as PostgreSQL resolves it: in
  `a.id IN (SELECT id FROM currencies LIMIT 1)` the `id` is the account's own when the catalog's
  `currencies` has no `id` column, so that subquery is not a pin either. A column of a derived
  table, a function or a CTE is taken as that source's own, and so is any column of a table
  the catalog does not describe. A `LATERAL` source that reads
  earlier FROM items is sized per row of them, so it bounds nothing pinned to it; the
  relations inside it are still judged. A derived query cannot resolve its own reads against
  the columns it later exposes. For example, `id IN (SELECT id FROM LATERAL (SELECT id) d
LIMIT 1)` still reads the target row and does not bound a mutation. A `LATERAL` query sees
  only preceding FROM sources; a later source with the same column name cannot hide that
  outer read.
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
caller. Bare, unquoted `COALESCE`, `LEAST`, `GREATEST`, and `NULLIF` preserve
this proof when every argument is caller-supplied, including nested conditional
forms and builtin array casts. For example, `unnest(COALESCE($1::uuid[],
ARRAY[]::uuid[]))` remains caller-sized. Columns, subqueries, custom calls or
custom casts inside these forms remain opaque; qualifying or quoting the
conditional name does not establish the special-form contract.
A constant-false or SQL-NULL `HAVING` rejects the group before SELECT-list
expansion. For example, `id IN (SELECT unnest(get_all_ids()) HAVING false)`
adds no target keys. This removes only the unused projection expansion proof;
uncapped physical source items remain in the facts. A `HAVING` result that is
not known to reject the group does not establish an empty result.
The JSON array functions `json_array_elements`, `json_array_elements_text`,
`jsonb_array_elements`, and `jsonb_array_elements_text` retain their declared
output column `value` when given only a relation alias. An explicit column alias
list overrides that name. For example, `id IN (SELECT value::uuid FROM
jsonb_array_elements_text($1::jsonb) AS item)` reads the local function output
and can bound the target key; a function in another schema remains opaque.
A data-backed select-list set-returning function is opaque even when its
SELECT has no FROM items; it cannot bound another relation joined to its output.
Any other table function (`FROM get_all_accounts()`, `app.generate_series(…)`), one
whose arguments read a column, query or function result (`unnest(ARRAY[a.id])`,
`unnest(get_all_account_ids())`), an array taken from a query
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
Nested CTE names remain visible only within their query scope. A nested `WITH`
clause cannot hide a parent base relation when checking outer column references.
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

### Executor configuration

| Option | Default | Behavior |
| --- | --- | --- |
| `importSpecifier` | Empty | Set explicitly to your database module to match its named imports. |
| `executorNames` | Empty without a module; `[query, read, write]` with a module | Without a module, only explicitly listed names match named imports from any module. |

With both options omitted, executor calls (including `.query`) are not scanned.
A configured module or explicit `query` enables `.query` members. A configured module also recognizes
its transaction helpers. Native SQL and recovered SQL-builder fragments retain
their existing scopes. See [the migration notes](../migrations/explicit-postgres-executors.md).

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
select-list rows. In `FROM`, functions whose arguments size their result
(`jsonb_path_query($1::jsonb, '$[*]')`, for example) and the fixed catalog
contract described below bound joined items. Functions
that inspect server state, parser metadata, or execute SQL (`pg_ls_dir`,
`pg_listening_channels`, `ts_token_type`, `ts_stat`) remain opaque even with fixed
arguments. The real PostgreSQL test lane checks this inventory against the
PostgreSQL function catalog.

An `ANY` array must also have caller-owned cardinality. Unknown calls such as
`id = ANY(get_ids($1))` remain unbounded: a function can return every stored key.
Explicit supported catalog array operations, including
`pg_catalog.array_append($1::uuid[], $2::uuid)`, retain caller-sized credit when
all their inputs are proven caller values. Custom schemas, unknown calls inside
those inputs, and unsupported signatures receive no credit. Use a caller-supplied
array or a finite constructor to establish the bound.

A cast scalar subquery such as
`id = ANY((SELECT account_ids FROM orders WHERE id = $1)::uuid[])` also receives
no key credit. One uniquely selected row can hold an arbitrarily large array.
Its nested reads remain available for SELECT read diagnostics; UPDATE and DELETE
continue to judge their target rows. Row-valued `ANY(SELECT account_id FROM orders
WHERE id = $1)` retains its ordinary query-row proof. Normal suppression directives
apply to these findings.

Snapshot expansion (`pg_snapshot_xip`, `txid_snapshot_xip`) is caller-sized only
when its snapshot is supplied directly by the caller. A snapshot returned by
`pg_current_snapshot()`, `txid_current_snapshot()`, or another SQL function is opaque.
The same caller-supplied requirement applies to other caller-sized set-returning
functions, including calls with PostgreSQL named arguments. Catalog functions
known to return one row, such as `pg_stat_get_recovery_prefetch()`, preserve a
pure aggregate's one-row cap. In `FROM`, the zero-argument call
`pg_catalog.pg_stat_get_recovery_prefetch()` also bounds a keyed join because its
catalog contract returns exactly one row. For example, joining `accounts.email`
to `p.stats_reset::text` from that call is bounded. Bare calls, other schemas,
and unsupported argument signatures remain opaque: this function was added after
the rule's conservative PostgreSQL version floor. Qualify it with `pg_catalog`
to establish catalog ownership; arbitrary server-state functions remain opaque.
Normal file and line suppression directives still apply to reported controls.

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

Standalone `TABLE name`, `TABLE ONLY name`, and forms with the optional
inheritance `*` after the name are analyzed as queries,
including after temporary-table declarations. For `TABLE` set-operation arms, including `TABLE ONLY name`, the SQL parser omits identifier quote information.
`INSERT INTO target TABLE ONLY source` also retains the source relation's quoted
or folded identity after a target alias, column list, or identity override.
`EXPLAIN TABLE name` is parsed as a plan without an executed query fact;
`EXPLAIN ANALYZE TABLE name` executes the query and retains the same quoted
or folded relation identity as standalone `TABLE name`.
Inside a derived query, a `TABLE name` right arm after a set operator retains
its following `LIMIT`. A streaming `UNION ALL` may be capped by that limit;
a blocking `UNION` still reads its input before limiting output.
When lenient parsing recovers SQL from a `DO` body or a reconstructed string,
TABLE arms use the recovered fragment's source tokens to distinguish quoted
identifiers from their unquoted catalog namesakes.
Analysis recovers the spelling from the prepared source tokens and matches quoted
names exactly. If source tokens are unavailable, it conservatively checks both
exact and folded spellings when they differ. A matching one-part CTE takes
precedence for its spelling only.
An unmatched quote in a `COPY ... FROM STDIN` data row does not hide later
quoted `TABLE` names; the data row is excluded from SQL tokenization.

Bare subquery columns belong to a derived table or CTE only when its explicit
projection exposes that name. Function column alias lists and known scalar built-in
output names also determine ownership. Unknown functions and wildcard projections
retain conservative unknown-column ownership.

Known scalar table functions expose their default position column when requested. UNNEST column names are inferred only for syntactically proven scalar arrays; composite and unknown element layouts retain unknown ownership. Parenthesized joins with a wrapper alias preserve the child tables' catalog column ownership when the alias does not rename output columns. For example, `id IN (SELECT id FROM (currencies c CROSS JOIN currencies d) j LIMIT 1)` remains correlated when neither child has `id`. Explicit output-column alias lists retain unknown ownership when the child labels cannot describe the renamed output.

Finite array constructors also accept explicitly known scalar builtin calls such as
`ARRAY[pg_catalog.lower($1)]` when their arguments preserve finite leaves. Unknown
calls, user-schema lookalikes, set-returning or array-returning functions, windows,
and expressions reading unbounded rows remain conservative. Calls require explicit
`pg_catalog` qualification and a supported builtin arity, including variadic minima.
Unqualified names remain opaque because application overloads can differ in volatility
or return shape even when their name and argument count match a builtin.

An empty grouping set, including the empty set generated by `ROLLUP` or `CUBE`, still emits a group when `WHERE false` rejects all inputs. An expanding `ORDER BY` expression can expand that group, so a false predicate does not prove that statement has zero rows. An ordinary grouping key combined with those sets prevents the empty group.

`ALTER SCHEMA ... RENAME TO ...` updates qualified physical source dependencies
of temporary views, preserving relation names, quoted schema case and dots, and
transitive temporary dependents. Transaction and savepoint rollback restore the
original schema identities. Bare dependencies retain their conservative matching
behavior.

Known scalar reducers such as `pg_catalog.array_to_string(o.account_ids, ',')` may
consume an array-valued argument inside a finite constructor. Their scalar result
provides one candidate value; argument row dependencies still require the owner
(`o`) to be bounded independently. Direct stored-array leaves remain unbounded,
and unknown, unqualified, array-returning and set-returning calls remain opaque.
Casts, domains and array slices inside those trusted reducers retain both their
argument owner and slice-bound row dependencies. They do not impose scalar-leaf
requirements on an argument whose shape cannot change the reducer result.
The same wrappers used as direct constructor leaves remain conservative.

Unknown SELECT-list function cardinality is opaque: a custom function can return
a set of database rows even when its arguments are bind values. The rule trusts
known PostgreSQL scalar builtins and aggregates by bare name or `pg_catalog`
qualification, plus bare SQL scalar forms such as `COALESCE`. This builtin
identity is a documented heuristic; a function in another schema
with the same name stays unknown. Unlisted scalar builtins are conservative too.
Use an explicit result `LIMIT` when the function's output must be bounded; known
caller-sized set-returning builtins retain their argument-based bounds. Trusted
scalar wrappers never hide unknown nested calls in their arguments.

Analysis does not infer the target server version. Scalar names introduced after
PostgreSQL 12 need explicit `pg_catalog` qualification: `gen_random_uuid` (PostgreSQL 13),
`regexp_count`, `regexp_instr`, `regexp_substr` (PostgreSQL 15), and the PostgreSQL 18 UUID
generators. Bare calls remain opaque because an older server can resolve them
to user-defined set-returning functions. Known older scalar builtins retain their
bare-name heuristic; an explicit `LIMIT` can still bound an unknown projection.
See the [PostgreSQL 13 release notes](https://www.postgresql.org/docs/release/13.0/)
and [PostgreSQL 15 release notes](https://www.postgresql.org/docs/15/release-15.htm).

Each set-operation arm owns its relation aliases. An alias declared in a sibling
arm cannot turn an outer column reference into a local reference.

Within a set-operation arm, a bare `GROUP BY` name is treated as an output label
only when that arm actually projects the label. `SELECT 1 GROUP BY id` retains an
outer `id` read; `SELECT $1 AS id GROUP BY id` groups its own output instead.

A caller-sized projected key keeps its own value proof beside an independent SRF:

```sql
UPDATE accounts a SET name = 'x'
FROM (SELECT $1::uuid AS id, pg_ls_dir('.') AS entry) s
WHERE a.id = s.id;
```

With a unique key on `accounts.id`, repeated directory rows still select at most
one account. Selecting `s.entry` as the key remains opaque. The same proof applies
through CTEs, positional aliases, and corresponding set-operation outputs; custom
casts, unknown functions, wildcard widths, and row-dependent keys remain conservative.
Use an explicit finite key value or bounded source to fix those cases. Existing
`no-mistakes` file and line suppression directives retain their normal behavior.

An executable PostgreSQL `CASE` expression has scalar cardinality: PostgreSQL
rejects set-returning functions in its conditions and arms. This boundary does
not prove caller-owned values or remove reads from physical source tables.
Unknown calls outside `CASE` remain conservative, including a custom function
that wraps the entire `CASE` expression.

An explicit caller-sized `LIMIT` or finite `FETCH` in a parenthesized SELECT
continues to cap its rows when an outer `ORDER BY` expands an expression.
This proof follows only that SELECT wrapper: it does not cap unrelated set
operation arms, NULL or data-derived limits, or a query with no explicit cap.

A legacy catalog with exactly one physical column can map that column to its
sole positional alias without ordinal metadata. Unique-key evaluation uses the
visible alias and retains the physical column’s nullability. Multi-column
legacy catalogs remain conservative; an alias named `ctid` does not prove the
physical system column’s uniqueness. Stored array leaves remain unbounded.

Correlated `IN` and `= ANY` subqueries retain their nested relation reads even
when they cannot bound an outer key. The Rust `SqlPinSource::ReadQuery` variant
records that read-only evidence separately from key-eligible `Query` pins.
Oversized CTE summaries and temporary-source shadowing preserve uncapped
nested reads; capped inner queries retain their existing bounded behavior.
Original source locations and normal `no-mistakes` suppression still apply.
