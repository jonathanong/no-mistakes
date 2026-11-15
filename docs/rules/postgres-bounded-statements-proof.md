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
- `DISTINCT ON` names only qualified columns in an ungrouped `SELECT`, fixed
  by direct, top-level `WHERE` equalities to literals or bind parameters. The
  fixed distinct tuple produces at
  most one row even when its input has many rows. For example,
  `SELECT DISTINCT ON (i.url_id) i.id FROM items i WHERE i.url_id = $1
  ORDER BY i.url_id, i.id DESC` is bounded and can supply a bounded CTE row to
  later key joins. Every distinct expression must be fixed. An `IN` list,
  `ANY` array, `OR` arm, or a fixed column other than the distinct expression
  does not establish this row cap. Grouping sets can add a NULL subtotal value,
  and an unqualified distinct name can refer to a select-list alias instead of
  the base column. A recursive CTE self-reference remains unbounded even when
  each iteration selects one distinct row.
- Every base relation in its FROM list is **pinned** to a unique key, or comes from a bounded
  source. A relation is pinned when top-level `AND` conjuncts of `WHERE` and of the `ON`
  that restricts it equate **every** column of one unique key with a value:
  `col = $1`, `col IS NOT DISTINCT FROM $1` (on a `NOT NULL` column or a
  `NULLS NOT DISTINCT` unique key; ordinary nullable unique keys can repeat NULL),
  `col IS NULL` (a `NULLS NOT DISTINCT` key has at most one NULL entry),
  `col = ANY($1::uuid[])` (bounded by the
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
  earlier FROM items is sized per row of them. A capped result can size a keyed join
  only after qualified references resolve to independently bounded preceding drivers.
  Unknown local scopes and bare correlations stay conservative; cycles supply no seed.
  A correlated equality on a unique-key column inside the lateral (primary key, unique
  index, or a partial unique index whose predicate the lateral repeats) is one row per
  driver row. When every driver is itself bounded, the lateral is bounded without a
  `LIMIT`; an unbounded driver, a missing predicate, or a non-unique partial index
  still reports. `ARRAY(SELECT ...)` in a select list is one array value per row and
  is not treated as an expanding call, while an unknown function still is.
  The relations inside it are still judged. Independent lateral queries retain their global bound. A derived query cannot resolve its own reads against
  the columns it later exposes. For example, `id IN (SELECT id FROM LATERAL (SELECT id) d
  LIMIT 1)` still reads the target row and does not bound a mutation. A `LATERAL` query sees
  only preceding FROM sources; a later source with the same column name cannot hide that
  outer read.
- A column compared with a column of another bounded relation is pinned too, so a bound
  propagates across joins on unique keys to a fixed point: with `i.id = $1`,
  `f.id = i.feed_id` is bounded, followed by `t.id = f.topic_id`. Additional `AND`
  conjuncts in `ON`, such as `f.deleted_at IS NULL`, `t.deleted_at IS NULL`, or
  `z.status = $2`, only narrow the key lookup. A join on a column that is not a
  whole key does not bound that relation. The source relation must already be
  bounded; a key equality does not make the source bounded. Both relations are
  reported when neither has its own bound. A bounded source may contain multiple
  rows, including when `columnBounds` establishes its bound on a non-key column.
  A direct fixed equality crosses an unambiguous join equality even when its
  source relation has an incomplete composite key: `p.conversation_id = $1`
  together with `c.id = p.conversation_id` can bound `c` by its primary key
  while `p` remains reported. This transfer follows a chain of direct
  equalities to a fixed point; it does not treat a fanout relation as bounded.
  The proof can continue through a middle relation bounded on a non-key column.
  If that middle relation is an unbounded fanout from the root, it and the
  downstream keyed target remain reported. `ON` pins
  only the side an outer join can null-extend, never the preserved side, and a `WHERE` on the
  null-extended side is not assumed to turn the join inner: a `LEFT JOIN` with its key
  pinned only in `WHERE` reports the preserved relation. `JOIN … USING (col)` pins the same
  way as `ON a.col = b.col` when each side is one FROM item; after a longer chain, which
  item owns the column cannot be told, so it pins nothing.
- For an uncapped `SELECT`, a complete primary or valid unique key equated directly
  to columns of another FROM relation bounds the keyed relation only when that
  source relation is already bounded. For example, `FROM items i LEFT JOIN topics t
  ON t.id = i.topic_id` reports both tables when `items` is unbounded. A direct
  pin on `items.id` bounds `items`, then bounds `topics` through its key. In the
  reverse direction, `users u JOIN posts p ON p.user_id = u.id` still reports
  `posts` when `users` is bounded by its primary key: `posts.user_id` is not a
  unique key and can match arbitrarily many rows. The same
  direction applies to inner and chained joins. Each target key column needs one
  bounded source equality; when two peers equate the same column, either bounded
  peer can supply it. A key equality present only in an `OR` branch, an incomplete
  key, or a nullable unique key compared with `IS NOT DISTINCT FROM` cannot prove
  the bound. A cycle without an independent bounded source remains unbounded,
  including self-joins. Reads inside derived tables and CTEs retain their own
  reports. A keyed table joined to an unbounded derived recordset also remains
  reported, even when the recordset has its own reportable base-table read. A
  derived source can supply a key value when its own result is bounded,
  including a bounded `DISTINCT ON` CTE chain. A correlated `LATERAL` source, an
  unknown catalog relation, or a relation with uncertain temporary shadowing
  cannot supply a bound. Extra independent conjuncts do not erase a complete key
  equality.
- For `UPDATE` and `DELETE`, the target relation is bounded. A CTE or subquery that picks
  the target rows with a `LIMIT` bounds it when the target's key is matched against it
  (`FROM c WHERE t.id = c.id`, or `WHERE t.id IN (SELECT id … LIMIT n)`). `ctid` identifies
  one row of a plain table, so `WHERE ctid IN (SELECT ctid … LIMIT n)` bounds a statement,
  except on a partitioned table, whose leaves repeat `ctid` values, and on a relation the
  catalog does not describe, which may be one. The relations joined to the target only
  feed it values: each target row changes once. A bound on a joined source can also bound
  the target when it pins the target's whole unique key, and that proof can pass through
  several unique-key joins. An unbounded source cannot pin the target through an equality,
  and one bounded source cannot lend its bound to an unrelated unbounded
  sibling. A direct target-key predicate still bounds the target even when another source
  is unbounded. The target is always the physical relation, even when a CTE of the same
  name exists. Joining that limited set on a column that is not a unique key of the target
  still reports the target.
- A plain `GROUP BY` produces a bounded result when every key is one caller value, or
  comes from an item that is already bounded, even when another input would otherwise fan
  out. One caller value is an unconditional `=` to a literal or bind. A CTE output that
  projects that column carries the same value, so `GROUP BY current.post_id` is one row
  when every `post_id` equals one bind. A chain of more than eight output renames does
  not carry that value. An expression output has no source column to follow. `GROUP BY page.id` stays bounded when `page` itself
  is a fixed `LIMIT`, including when the grouped query also joins an unbounded table: the
  grouped result cannot grow past that input. `ROLLUP`, `CUBE`, `GROUPING SETS`, and any
  other grouping modifier are not this proof, because they can add subtotal rows. An
  integer `GROUP BY` literal is an output ordinal, not a constant. A set-returning
  function in the select list, in `DISTINCT ON`, or in `ORDER BY` can multiply the grouped
  rows, so those queries stay unbounded. `IN`, `ANY`, `OR`, and `IS NOT DISTINCT FROM` are not one caller
  value. A key that still comes from an unbounded item keeps that item reported.
  A positional alias list is the grouped name, so `d(id, status)` makes `GROUP BY d.status`
  follow the second output.
- A row whose unique key is pinned to one caller value is one row, so every catalog
  column of that row is one value. Those values cross a direct column equality (`a.col = b.col`)
  when the equality is unconditional, not null-safe, and names both columns. A validated
  foreign key whose columns are all `NOT NULL` then fills the remaining foreign-key
  columns when the equated referenced columns include a unique key of that one referenced
  row. The filled columns can complete another composite unique key. A `NOT VALID`
  foreign key, a nullable foreign-key column, or a composite unique key with one column
  still free does not prove the row. This fill-in uses plain unique keys, so a partial
  or expression unique index does not complete it. A positional
  column alias list does not let that alias satisfy a unique key of a different physical
  column. A positional alias list is also the name a foreign-key equality uses, so an alias
  that reuses a foreign-key column's spelling does not equate that physical column. A complete
  alias list, one name for each catalog column, is that column's visible name, so a join on
  one of those names still fills the remaining foreign-key column. A repeated name in that list does not identify
  one column, and a shorter list does not rename the columns it leaves out. Two columns that
  are each one value are not treated as equal unless the statement equates them. A `LIMIT`
  of more than one row is a bounded input, not one row, so it does not feed this proof.

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
An `ARRAY(SELECT generate_series(1, $2))` input has no relation to scan and is
caller-sized. With `FROM accounts LIMIT 1`, even a caller-sized projection such
as `generate_series(1, 0)::text::uuid` can emit zero rows for every account.
The result LIMIT then may read the whole table; an independent account key
predicate still bounds that input. Casts and ordinary expression wrappers do
not remove this scan requirement.
Any other table function (`FROM get_all_accounts()`, `app.generate_series(…)`), one
whose arguments read a column, query or function result (`unnest(ARRAY[a.id])`,
`unnest(get_all_account_ids())`), and an uncapped recursive reference of a `WITH RECURSIVE` are opaque: never
reported themselves, and they bound nothing pinned to them. A proven
[depth-capped recursive CTE](postgres-bounded-statements.md#depth-capped-recursive-ctes)
can supply a bounded driver while its anchor and step relations retain their
ordinary key proofs. `EXCEPT` and `INTERSECT` read
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
emits one finding per statement. Distinct unbounded relations are joined in `target`
(for example `table:feeds, table:items`) and their messages are joined with `; `.
A single-relation finding keeps one `table:` target. The finding is placed at the
earliest remaining relation location.
Imported or appended shared SQL keeps each root's report at its executing
caller, with the fragment line named in the message.
An uncapped SELECT also reports unbounded relations read by its IN-subquery pins,
even when another predicate pins the outer table. UPDATE and DELETE retain target-only
reporting.
Nested CTE names remain visible only within their query scope. A nested `WITH`
clause cannot hide a parent base relation when checking outer column references.
Statement kinds are judged independently: a data-modifying CTE is its own `UPDATE` or
`DELETE` (judged when `statements` includes it), and the `SELECT` that reads its
`RETURNING` rows is not unbounded because of it. One returned row per
modified row carries its bound only when the mutation target is itself bounded.
That projection keeps a key pin when every sibling it names is a capped query,
including `WITH page AS MATERIALIZED (... LIMIT $n FOR UPDATE)` joined through
`ANY(ARRAY[page.column])` on each primary-key column. An uncapped sibling is
dropped and does not supply the proof. A primary-key join against that bounded
`RETURNING` source can bound a later mutation; an unbounded data-modifying CTE
does not supply this proof.

A **unique key** is a primary key, unique constraint, or unique index in the catalog
that is valid, ready, live and immediate, using the default equality semantics.
Every key element must have a finite pin. Supported index shapes include:

- Plain columns, including all columns of a composite key.
- Expression keys when the statement equates the same expression to a value or
  a bounded relation's value: `lower(t.name) = $1` matches `UNIQUE (lower(name))`.
  Relation aliases are resolved before matching. A changed expression does not
  inherit another expression's uniqueness.
  Raw equality on a column also fixes an allowlisted function of that column.
  Equal inputs produce one result, so a unique index on the function admits at
  most one row. Recognized functions are unquoted `lower`, `upper`, and
  one-argument `btrim`, unqualified or `pg_catalog`-qualified. The column's
  catalog type must be `text`, `varchar`, `character varying`, or `char varying`.
  `lower` and `upper` of a range are different functions: a non-null empty or
  unbounded range can have a null bound, and a unique index still admits many
  null keys.
  `username = $1` matches `UNIQUE (lower(username))`. The reverse does not:
  `lower(col) = $1` against a unique index on raw `col` can match many inputs.
  A cast of one bare column is recognized when the catalog column type and the
  cast target share an equality family. The families are `text`, `varchar`,
  `character varying`, and `char varying`; `smallint`, `integer`, `bigint`,
  `int2`, `int4`, and `int8`; `uuid` with `uuid`; `boolean` with `bool`.
  Length and precision modifiers stay in the family. `char`, `character`,
  `numeric`, floats, timestamps, and a cast into another family (`uuid::text`)
  do not. Quoted names, other schemas, extra arguments, and an argument that
  is not a bare column stay unrecognized.
- Partial indexes when every predicate conjunct is proven for that relation's
  scan. For `UNIQUE (community_id, user_id) WHERE removed_at IS NULL`, both keys
  and `removed_at IS NULL` must hold for the scan. Reordered conjuncts are
  accepted. Strict equality `col = $1` also implies `col IS NOT NULL`
  on that same column, so `username = $1` proves
  `UNIQUE (lower(username)) WHERE username IS NOT NULL`.
  Equality of unquoted `lower`, `upper`, or one-argument `btrim` (unqualified or
  `pg_catalog`-qualified) also implies that its column argument is not null, because
  those functions yield null for a null argument and null never compares equal.
  `lower(username) = lower($1)` therefore pins `UNIQUE (lower(username))` and proves
  `username IS NOT NULL`. `coalesce` is not strict. `IS NOT DISTINCT FROM` can match
  null, so it does not. A quoted name, another schema, a filter, a window, or an extra
  argument stays unrecognized. Raw equality does not prove `f(col) IS NOT NULL` or a
  predicate on another column.
  An outer join's `ON` for the preserved side does not establish the scan's
  predicate. Positional column alias lists keep expression and partial-index
  proofs conservative.
  A conjunct holds when the same alternative contains the same shape: `col IS NULL`;
  `col = literal`; or `col IN (literals)` with the same set of literals. A cast
  stays on the literal only when it preserves the value: `text`, `boolean`, unlimited
  `varchar` / `character varying`, `varchar(n)` when every character fits, and a
  catalog enum whose label is the literal. A quoted enum name keeps its case, so
  `'direct'::"SourceKinds"` matches the catalog key `"SourceKinds"`.
  `ANY (ARRAY[...]::type[])` applies that rule to each element.
  A scalar cast to an array type drops the literal, including
  `('pending'::text[])::text`, which PostgreSQL evaluates to `{pending}`.
  `varchar(n)` that truncates, `char(n)`, a cast of an array constructor to a
  scalar or to more than one array dimension, and any other cast stay unrecognized,
  so `'pending'::varchar(3)` does not match `'pending'`.
  PostgreSQL stores that list as `col = ANY (ARRAY[...])`,
  and the two spellings match. A longer list, a shorter list, or `col = one-literal`
  does not match a multi-value list. A bare parameter does not match a literal.
  `col = $n` together with `$n = literal` or `literal = $n` in the same alternative
  does. Two different literals for that placeholder cancel the binding, so it
  neither forwards nor chooses an enum label. Shared
  `AND` conjuncts count for every alternative. A conjunct that appears only in one
  `OR` arm counts only for that arm.
  When the catalog types a column as an enum and the alternative equates that
  column to a parameter, each catalog label is tried in its place, up to 32
  combinations. Every label must pin some unique index using that alternative's
  conjuncts. Those pins keep their source dependencies, so a label reached only
  through another relation is proven when that relation is bounded. The product
  of proving indexes across labels also stays within 32; a larger product leaves
  the scan unproven. A label that contradicts a literal already bound to the
  parameter matches no row. A type that is not a catalog enum stays opaque, so
  the same SQL on `text` still reports. More than 32 combinations leave the scan
  unproven.
- `UNIQUE ... NULLS NOT DISTINCT` keys also accept `IS NOT DISTINCT FROM`,
  including nullable columns and matching expression keys. `col IS NULL` also pins
  a nullable column of such a key; every other key element still needs a pin.
  Ordinary nullable unique keys can contain repeated NULLs and remain unbounded.
  The catalog's index
  definition supplies this flag. See [PostgreSQL unique indexes](https://www.postgresql.org/docs/18/indexes-unique.html).

These do not prove one row:

- A partial unique index whose predicate is absent or only partly established.
  That includes a different `IN` or `ANY` literal set, a parameter standing
  in for a literal, and a cast that can change the literal
  (`'pending'::varchar(3)`, the same cast on an array, and
  `('pending'::text[])::text`).
- An enum label whose proving index is pinned only by an unbounded relation.
- A catalog index with a placeholder anywhere in its parsed key or predicate.
  PostgreSQL index definitions cannot depend on query parameters; matching malformed
  catalog text to a query does not establish uniqueness.
- An invalid or not-ready index (a failed concurrent build), or a deferrable
  constraint, which may hold duplicates inside a transaction.
- Column equality used in place of an expression key the allowlist does not recognize.
- An index whose operator class or collation is not the column's default (the
  catalog's `orderingSupported` is false), because its equality may differ.
- Only part of a composite key.
- Ranges and `LIKE`.

Each `OR` alternative must independently pin a complete unique key, using shared
`AND` conjuncts where present. The keys may differ between branches. A partial
index counts for an alternative only when that alternative proves its predicate.
A catalog enum compared with a parameter can make one arm prove a different
partial index for each label: `kind = $2 AND ($2 = 'direct' OR user_id = $3)`
pins the `kind = 'direct'` index on the first arm, and the `kind = 'family'`
index on the second arm only when `user_id` is pinned and `family` is a catalog
label. The disjunction alone does not choose an index. More than 32 expanded
alternatives retain only unconditional pins.

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

Relations the catalog does not describe (relations created in the same script) and
relations spelled with another schema (`audit.accounts` when the catalog is for `public`)
are not judged, and they bound nothing. A catalog view with no `viewBounds`
entry is an unsized relation when its definition projects a base table's primary
key one-to-one, including `SELECT base.*` and a `LEFT JOIN` that pins the other
table's primary key. A keyed read is then bounded like a table. A view whose key
cannot be derived stays `unknown-relation` and is not a finding. `viewBounds:
unbounded` is still an unsized relation even when a key could be derived, and a
statement cap can still bound that read. `viewBounds: bounded` is trusted without
reading the definition. A bare name shared by a table and a view stays unknown.
A relation is found by the name PostgreSQL reads:
an unquoted name folds to lower case and a quoted one (`"Order Items"`) is exact: an unknown relation can supply every
value of a column pinned to it, so a catalog table joined to one is still reported unless
something else bounds it. The rule checks every complete version of a branching
executor call, including nested trusted SQL fragments and conditional appends.
A bound on one version does not bound another: choosing between a key-pinned
SELECT and an unbounded SELECT reports the unbounded path. Up to 16 versions
are recovered; findings retain each branch fragment's physical source line,
including each argument of a fluent append chain.
See [finite SQL alternatives](../postgres-facts.md#finite-sql-alternatives).
A statement whose complete versions cannot be recovered statically (`SQL could
not be analyzed`, or `executed SQL is not statically recoverable`) fails closed.
Nested fragments are spliced SQL, so an opaque fragment is never accepted as a bind.
`unanalyzableSql: ignore` skips those statements instead, as it does for the sibling
rules; every statement that can be analyzed is still judged.

`(a, b) IN (SELECT ... LIMIT $n)` and caller-sized tuple `unnest` inputs pin
both components of a composite catalog key. A LIMIT-less input supplies no cap.
`ANY(ARRAY(SELECT ...))` and `ANY((SELECT array_agg(id) FROM batch)::uuid[])`
prove finite candidate sets when the input query is bounded and its elements
are proven scalar. An aggregate's one output row or outer LIMIT alone is
insufficient; stored arrays, unknown projections, and correlated self reads
retain conservative behavior.

A scalar CTE LIMIT is accepted only through guaranteed single-row fixed caller
projections, without row-removing predicates, set operations, or expanding
projections. Empty and NULL scalar results mean unlimited; physical data-derived
counts retain no caller-cap credit. See the [rationale](postgres-bounded-statements-rationale.md)
for the uncapped recursive-key-walk policy and examples.

For array-query candidates, a scalar cast also requires proven row/caller-sized
value lineage. Casts, parentheses, and nested scalar wrappers cannot hide a
data-backed SRF; opaque FROM/LATERAL inputs remain unproven through CTE or
derived aliases even when the result query writes a LIMIT. Caller-sized SRFs
such as `generate_series(1, $n)` or `unnest($ids)` retain the existing finite
caller-input policy, including a supported scalar cast of their results.

PostgreSQL rejects `TABLESAMPLE` on a caller-count CTE: sampling applies only to
tables and materialized views. Such invalid SQL is outside this rule's count
proof contract. A sampled physical-table value still has no caller-count proof.
