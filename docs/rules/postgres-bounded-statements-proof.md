# Bounded statements proof cases

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
  `col = $1`, `col IS NOT DISTINCT FROM $1` (on a `NOT NULL` column or a
  `NULLS NOT DISTINCT` unique key; ordinary nullable unique keys can repeat NULL), `col = ANY($1::uuid[])` (bounded by the
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
  as the alias but no longer exposes `public.accounts`.
  An alias on an entire parenthesized join also hides its child table names and aliases
  from outward column and whole-row references. The children stay visible inside its
  `ON` and `LATERAL` expressions; nested derived queries retain their own namespaces.
  The joined alias is unavailable inside its own `ON` or `LATERAL` body.
  A later join child is also unavailable in an earlier `ON`, so that alias cannot erase an outer write reference. A reference there
  to a matching outer write alias stays correlated even after the join exposes that alias.
  References retain their schema: an inner `audit.accounts` cannot hide a reference to the outer
  `public.accounts`. When the configured catalog explicitly selects `public`, an inner
  `FROM accounts` with no alias can resolve to that same `public.accounts`; without that catalog
  evidence the rule keeps the qualified reference conservative. Quoted dots stay inside one
  identifier:
  a table or alias named `"public.accounts"` cannot hide that schema-qualified reference. A bare column
  belongs to the first level whose tables have it, as PostgreSQL resolves it: in
  `a.id IN (SELECT id FROM currencies LIMIT 1)` the `id` is the account's own when the catalog's
  `currencies` has no `id` column, so that subquery is not a pin either. A column of a derived
  table, a function or a CTE is taken as that source's own, and so is any column of a table
  the catalog does not describe. A derived-table column alias list renames only the leading
  projected columns: `(SELECT status, id FROM orders) q(kind)` still exposes `id` locally.
  A `LATERAL` source that reads
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
  catalog does not describe, which may be one. The relations joined to the target only
  feed it values: each target row changes once. A bound on a joined source can also bound
  the target when it pins the target's whole unique key, and that proof can pass through
  several unique-key joins. A source joined through a non-unique target column does not
  bound the target, and one bounded source cannot lend its bound to an unrelated unbounded
  sibling. A direct target-key predicate still bounds the target even when another source
  is unbounded. The target is always the physical relation, even when a CTE of the same
  name exists.

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
This also applies through derived tables and CTE wrappers: `SELECT * FROM
(SELECT id FROM accounts EXCEPT SELECT account_id FROM orders) AS q LIMIT 1`
still reports the uncapped `orders` input. Independently streaming siblings keep
their outer cap. Inner key predicates or row caps still bound their own inputs;
compaction and temporary-name projection preserve these distinctions and source
locations. Use ordinary `no-mistakes` suppression directives at those locations.
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
`RETURNING` rows is not unbounded because of it. One returned row per
modified row carries its bound only when the mutation target is itself bounded.
A primary-key join against that bounded `RETURNING` source can bound a later
mutation; an unbounded data-modifying CTE does not supply this proof.

A **unique key** is a primary key, unique constraint, or unique index in the catalog
that is valid, ready, live and immediate, using the default equality semantics.
Every key element must have a finite pin. Supported index shapes include:

- Plain columns, including all columns of a composite key.
- Expression keys when the statement equates the same expression to a value or
  a bounded relation's value: `lower(t.name) = $1` matches `UNIQUE (lower(name))`.
  Relation aliases are resolved before matching; changed expressions or casts
  do not inherit the original expression's uniqueness.
- Partial indexes when every predicate conjunct is proven for that relation's
  scan. For `UNIQUE (community_id, user_id) WHERE removed_at IS NULL`, both keys
  and `removed_at IS NULL` must appear in applicable `AND` conjuncts. Reordered
  conjuncts are accepted. Strict equality `col = $1` also implies `col IS NOT NULL`.
  A function expression's equality does not prove its argument non-null, so
  `UNIQUE (lower(username)) WHERE username IS NOT NULL` needs that explicit guard.
  A predicate present only in an `OR` alternative, or an outer join's `ON` for
  the preserved side, does not establish the scan's predicate. Positional column
  alias lists keep expression/partial-index proofs conservative.
- `UNIQUE ... NULLS NOT DISTINCT` keys also accept `IS NOT DISTINCT FROM`,
  including nullable columns and matching expression keys. The catalog's index
  definition supplies this flag. See [PostgreSQL unique indexes](https://www.postgresql.org/docs/18/indexes-unique.html).

These do not prove one row:

- A partial unique index whose predicate is absent or only partly established.
- An invalid or not-ready index (a failed concurrent build), or a deferrable
  constraint, which may hold duplicates inside a transaction.
- Column equality used in place of an expression key.
- An index whose operator class or collation is not the column's default (the
  catalog's `orderingSupported` is false), because its equality may differ.
- Only part of a composite key.
- Equality alternatives (`id = $1 OR id = $2`), ranges and `LIKE`.

### Unqualified key columns

With a complete catalog and several known base tables, an unqualified key
column is assigned when exactly one visible relation has it. The same ownership
proof applies to equality, null-safe equality, finite `IN`/`ANY` pins and boolean
key predicates. Ambiguous names, unknown or opaque competitors, derived sources,
views and positional column aliases keep the proof conservative. Row-dependent
values such as `key = key` do not become caller-supplied values. Qualify the key
column to make an ambiguous reference explicit.

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
something else bounds it. The rule checks every complete version of a branching
executor call, including nested trusted SQL fragments and conditional appends.
A bound on one version does not bound another: choosing between a key-pinned
SELECT and an unbounded SELECT reports the unbounded path. Up to 16 versions
are recovered; findings retain each branch fragment's physical source line.
See [finite SQL alternatives](../postgres-facts.md#finite-sql-alternatives).
A statement whose complete versions cannot be recovered statically (`SQL could
not be analyzed`, or `executed SQL is not statically recoverable`) fails closed.
Nested fragments are spliced SQL, so an opaque fragment is never accepted as a bind.
`unanalyzableSql: ignore` skips those statements instead, as it does for the sibling
rules; every statement that can be analyzed is still judged.
