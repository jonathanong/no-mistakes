# `postgres-bounded-statements`

`executorNames` replaces the default `[query, read, write]` list; it does not
extend it. Use `extraExecutorNames: [readPool, writePool]` to keep the defaults
and add names. Setting both options is a configuration error. When a replacement
omits a default name imported from `importSpecifier` and called in scoped files,
`check` emits a configuration warning naming the rule or project, the omitted
name, an example call site, and the fix. See
[executor selection](../configuration/rules.md#postgresql-executor-selection).
`excludeExecutorNames: [query]` removes names after replacement or default-plus-extra
selection and acknowledges intentional omitted defaults. `check` warns about
exclusions that are not default names or are never imported from the selected
module and called in scoped files.

A `projects: [<name>]` entry inherits [`projects.<name>.postgres`](../configuration/projects.md#postgresql-scope) when that block is set. Do not restate `importSpecifier`, `executorNames`, `extraExecutorNames`, `excludeExecutorNames`, `executorFactoryNames`, `executorTypeNames`, `trustedSqlTags`, `schemaCatalogPath`, or `unanalyzableSql` on the rule. `include`, `exclude`, and `sqlInclude` may only narrow the project scope.

Require a statement that can match many rows to take a `LIMIT`. A `SELECT` with no
bound, or an `UPDATE` or `DELETE` that touches every row matching a predicate, does
work that grows with the table: its time, lock footprint and memory have no ceiling.
The schema catalog knows every unique key, so the rule can tell a single-row lookup
from a statement that can match many rows.

```yaml
rules:
  - rule: postgres-bounded-statements
    scope: repository
    options:
      executorNames: []
      sqlInclude: ["db/**/*.sql"]
      schemaCatalogPath: db/schema.json
      statements: [select, update, delete]
      allow:
        - object: "table:currencies"
          reason: "Fixed lookup table of under 200 rows"
```

## Why and when

See [rationale](postgres-bounded-statements-rationale.md).

## What it catches/requires

See [proof cases](postgres-bounded-statements-proof.md).

## Options and defaults

`unanalyzableSql: advise` reports residual uncertainty in non-failing advisories.
`--include-sql-verdicts` adds [per-subcheck SQL verdicts](../postgres-sql-verdicts.md).

`include`, `exclude`, `importSpecifier`, `executorNames` and `unanalyzableSql` match
`postgres-explicit-columns`. `unanalyzableSql` defaults to `fail`. `sqlInclude`
defaults to `[]`, so `.sql` files are not scanned until you set globs.
`schemaCatalogPath` has no default, is required, and names a catalog generated with
[`no-mistakes postgres catalog`](../cli/postgres.md); an ordering-only catalog is
rejected like it is for every other rule that needs complete facts. `statements`
defaults to `[select, update, delete]`; an empty list, an unknown kind or a duplicate is
a configuration error. `allow` defaults to `[]`: each entry is `{object, reason}`
with an object such as `table:currencies` or `view:public.small_codes`. An empty reason, an object that is not a
catalog ref, or a duplicate is a configuration error, and an entry that suppresses
nothing is reported as stale. The catalog's name for the table is the object ref.

### Catalog views

`viewBounds` defaults to `{}`. It maps specific catalog view names to `bounded`
or `unbounded`, for example `viewBounds: {"public.small_codes": bounded,
"public.activity_feed": unbounded}`. These are explicit cardinality assertions.
`bounded` trusts the view without reading its definition. Without an entry, a
one-to-one projection of a base table's primary key supplies the key below;
every other view stays unknown.

- `bounded` is an unchecked user assertion that the view supplies a finite set of
  rows. The rule trusts it without inspecting the SQL definition, including when
  the view is an `UPDATE` or `DELETE` target. For example, a `GROUP BY` or `UNION`
  view is still trusted if configured `bounded`; confirm its cardinality yourself.
  A bounded view can also size another relation through a unique-key join.
- `unbounded` treats the view as an unsized relation. A statement cap can bound
  it; otherwise the finding targets `view:<catalog-name>`.
- A catalog view with no `viewBounds` entry is judged when its definition
  projects one base table's primary key one-to-one. `SELECT id, ... FROM base`
  and `SELECT base.* FROM base LEFT JOIN other ON other.pk = base.column` are
  that shape: the joined table's whole primary key must match a base column or
  a literal, so the join cannot multiply rows. The projected primary-key
  columns are the view's key, and a lookup or join on that key is bounded the
  same way as a table. The rule uses the primary key only, not some other
  unique index, and it does not follow a view that reads another view.
  `viewBounds: unbounded` stays an unsized relation even when a key could be
  derived. A read that renames columns, as in `FROM v AS x(a, b)`, has no view
  key because aliases rename positionally, so it is judged as an unsized view.
- Any other catalog view stays an `unknown-relation` advisory and is not a
  finding. That includes aggregates, projections that omit the primary key,
  `DISTINCT`, `GROUP BY`, `UNION`, limits inside the view, and joins that can
  multiply rows. Names the catalog does not contain, and a bare name shared by
  a table and a view, stay `unknown-relation` too. Use `unanalyzableSql: advise`
  for that residual advice and `--include-sql-verdicts` to inspect the verdicts.

Each key must resolve to one catalog view. Missing names, ambiguous bare names,
table/view collisions and duplicate aliases for one view are configuration
errors. A qualified view name can use a bare catalog entry only when the catalog's
`schema` proves that qualification. Qualified table references retain table lookup
behavior, including bare table keys in catalogs without a declared `schema`.
Quote names exactly, for example
`'public."Mixed Name"': bounded`. A bare SQL name shared by a table and a view
remains unknown; use the qualified relation name.

`allow` and line/file suppressions apply to view findings using `view:<name>`.
See the [saved policy example](../../test-cases/rules/postgres-bounded-statements/fixture/catalog-resolution/view-bounds.yml).

### Per-column bounds

`columnBounds` defaults to `{}`. It declares an application-owned maximum fan-out
for a catalog column, even when that column is not unique:

```yaml
columnBounds:
  orders.account_id:
    reason: "Each account has at most 20 orders enforced by the application policy"
```

With this declaration, `WHERE account_id = $1` and
`WHERE account_id = ANY($1::uuid[])` can bound `orders` using the same
caller-bounded key proof as a unique key. A declared column also suffices when
it is only a prefix of a composite unique key; the declaration asserts the
remaining fan-out is bounded. These proofs apply to `UPDATE` and `DELETE`
targets as well as `SELECT` inputs. This is an unchecked assertion about
fan-out; the rule does not enforce the application policy. Keep a concrete
maximum in that policy and explain it in the required nonempty `reason`.

A non-unique column without a declaration still produces a finding. A declared
column does not make `WHERE account_id > $1` bounded, exempt an unrestricted
scan, or make `ANY(SELECT account_id FROM orders)` bounded. Every key set must
itself be bounded; joins must still satisfy the relation-bound proof.

Each key must resolve to one catalog table column. Missing or ambiguous tables,
missing columns, and duplicate aliases for the same column are configuration
errors. Use a qualified table name when a bare name is ambiguous. These are
rule-local assertions, separate from schema uniqueness and `viewBounds`.

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

The SQL-only configuration above reads `db/queries.sql`. Both examples use
this complete catalog snapshot, including valid, immediate primary-key indexes:

```json
// db/schema.json
{
  "formatVersion": 2,
  "coverage": "complete",
  "tables": {
    "accounts": {
      "relationKind": "table",
      "columns": {"id": {"dataType": "uuid", "nullable": false}, "name": {"dataType": "text", "nullable": false}},
      "primaryKey": {"columns": ["id"]},
      "indexes": {
        "accounts_pkey": {
          "accessMethod": "btree", "unique": true, "primary": true,
          "valid": true, "ready": true, "live": true, "immediate": true,
          "keys": [{"column": "id", "expression": "id"}]
        }
      }
    },
    "orders": {
      "relationKind": "table",
      "columns": {"id": {"dataType": "uuid", "nullable": false}, "account_id": {"dataType": "uuid", "nullable": false}},
      "primaryKey": {"columns": ["id"]},
      "indexes": {
        "orders_pkey": {
          "accessMethod": "btree", "unique": true, "primary": true,
          "valid": true, "ready": true, "live": true, "immediate": true,
          "keys": [{"column": "id", "expression": "id"}]
        }
      }
    },
    "invoices": {
      "relationKind": "table",
      "columns": {"id": {"dataType": "uuid", "nullable": false}, "paid_at": {"dataType": "timestamptz", "nullable": true}}
    },
    "exports": {
      "relationKind": "table",
      "columns": {"id": {"dataType": "uuid", "nullable": false}, "expires_at": {"dataType": "timestamptz", "nullable": false}, "s3_key": {"dataType": "text", "nullable": true}},
      "primaryKey": {"columns": ["id"]},
      "indexes": {
        "exports_pkey": {
          "accessMethod": "btree", "unique": true, "primary": true,
          "valid": true, "ready": true, "live": true, "immediate": true,
          "keys": [{"column": "id", "expression": "id"}]
        }
      }
    },
    "sessions": {
      "relationKind": "table",
      "columns": {"id": {"dataType": "uuid", "nullable": false}, "expires_at": {"dataType": "timestamptz", "nullable": false}}
    },
    "currencies": {
      "relationKind": "table",
      "columns": {"code": {"dataType": "text", "nullable": false}}
    }
  }
}
```

```sql
-- db/queries.sql
SELECT id FROM invoices WHERE paid_at IS NULL ORDER BY id LIMIT $1;
SELECT name FROM accounts WHERE id = $1;
SELECT id FROM orders WHERE id = ANY($1::uuid[]);
SELECT COUNT(*) FROM orders WHERE account_id = $1;
WITH c AS (SELECT id FROM exports WHERE expires_at < now() ORDER BY id LIMIT $1 FOR UPDATE SKIP LOCKED)
  UPDATE exports SET s3_key = NULL FROM c WHERE exports.id = c.id;
SELECT code FROM currencies;   -- allow entry
```

## Parameter-array bounds

A `LATERAL` subquery may also use a named column from a preceding
`unnest` of parameter arrays as its `LIMIT`:

```sql
SELECT child.id, frontier.parent_id
FROM unnest($1::uuid[], $2::integer[]) AS frontier(parent_id, row_limit)
CROSS JOIN LATERAL (
  SELECT p.id FROM posts p
  WHERE p.parent_post_id = frontier.parent_id
  ORDER BY p.id
  LIMIT frontier.row_limit
) AS child;
```

The parameter arrays bound the outer relation, and each supplied limit bounds
its corresponding subquery. Every `unnest` argument must be a parameter array.
Literal arrays, mixed table/parameter inputs, and limits taken from table
columns do not establish this bound.

## Counterexample

The first three statements are unbounded under this catalog. The currencies
query keeps the configured table exception in use:

```json
// db/schema.json
{
  "formatVersion": 2,
  "coverage": "complete",
  "tables": {
    "accounts": {
      "relationKind": "table",
      "columns": {"id": {"dataType": "uuid", "nullable": false}, "name": {"dataType": "text", "nullable": false}},
      "primaryKey": {"columns": ["id"]},
      "indexes": {
        "accounts_pkey": {
          "accessMethod": "btree", "unique": true, "primary": true,
          "valid": true, "ready": true, "live": true, "immediate": true,
          "keys": [{"column": "id", "expression": "id"}]
        }
      }
    },
    "orders": {
      "relationKind": "table",
      "columns": {"id": {"dataType": "uuid", "nullable": false}, "account_id": {"dataType": "uuid", "nullable": false}},
      "primaryKey": {"columns": ["id"]},
      "indexes": {
        "orders_pkey": {
          "accessMethod": "btree", "unique": true, "primary": true,
          "valid": true, "ready": true, "live": true, "immediate": true,
          "keys": [{"column": "id", "expression": "id"}]
        }
      }
    },
    "invoices": {
      "relationKind": "table",
      "columns": {"id": {"dataType": "uuid", "nullable": false}, "paid_at": {"dataType": "timestamptz", "nullable": true}}
    },
    "exports": {
      "relationKind": "table",
      "columns": {"id": {"dataType": "uuid", "nullable": false}, "expires_at": {"dataType": "timestamptz", "nullable": false}, "s3_key": {"dataType": "text", "nullable": true}},
      "primaryKey": {"columns": ["id"]},
      "indexes": {
        "exports_pkey": {
          "accessMethod": "btree", "unique": true, "primary": true,
          "valid": true, "ready": true, "live": true, "immediate": true,
          "keys": [{"column": "id", "expression": "id"}]
        }
      }
    },
    "sessions": {
      "relationKind": "table",
      "columns": {"id": {"dataType": "uuid", "nullable": false}, "expires_at": {"dataType": "timestamptz", "nullable": false}}
    },
    "currencies": {
      "relationKind": "table",
      "columns": {"code": {"dataType": "text", "nullable": false}}
    }
  }
}
```

```sql
-- db/queries.sql
SELECT id FROM invoices WHERE paid_at IS NULL;
UPDATE exports SET s3_key = NULL WHERE expires_at < now();
DELETE FROM sessions WHERE expires_at < now();
SELECT code FROM currencies; -- Keep the configured allow entry in use.
```

## Fix

Add a `LIMIT` and loop with a cap. For `UPDATE` and `DELETE`, pick the rows first:

```sql
DELETE FROM sessions
WHERE id IN (SELECT id FROM sessions WHERE expires_at < now() ORDER BY id LIMIT $1);
```

If the table is small and fixed, add an `allow` entry with a reason.

## Shapes that stay bounded

### Key proofs through bounded drivers

A driver restricted by `a.id = $1` or `a.id = ANY($1::uuid[])` can bound a
joined relation when the join fixes its complete primary or unique key.
For example, `JOIN profiles p ON p.account_id = a.id` stays bounded when
`profiles.account_id` is unique. Joining on a non-unique column or leaving
the driver unbounded still reports; pin the driver or select a capped batch.

The same proof applies to `UPDATE accounts t SET name = 'x' FROM picked
WHERE t.id = picked.id` and `DELETE FROM accounts t USING picked
WHERE t.id = picked.id` when `picked` is a bounded CTE. A primary-key lookup
or caller-array lookup inside the CTE supplies that bound, as can bounded
`RETURNING` rows. An unrestricted `SELECT id FROM accounts` CTE supplies no
bound; add a key restriction or a batch `LIMIT` inside it.

A write restricted by its complete composite primary key returns at most one
row. Its `RETURNING` column names remain visible to later writes, so matching
every target key column against that returned row keeps the later write bounded.
Names alone grant no caller-value, array-element or predicate-propagation proof.
An `INSERT` source can bound the returned row count, but its finite key domain
does not prove the domain of generated `RETURNING` values.
Implicit labels for numeric literals and function calls, along with qualified
projected column names, identify which relation owns a bare key beside a lock or
`LIMIT 1` CTE. Ambiguous names,
wildcard layouts and incomplete keys remain conservative; qualify the key or
declare explicit output aliases when ownership is unclear.

`JOIN unnest($1::uuid[]) AS req(id) ON t.id = req.id` also bounds a primary-key
lookup on `t.id`. The parameter array sizes the driver, while the target key
limits its fan-out. Using a non-unique target column still reports.

A bare column such as `WHERE aside_type = $1` can pin a key in a multi-table
query when the complete catalog identifies exactly one visible owner.
An ambiguous bare column supplies no proof; qualify it with the intended
relation alias. Catalog indexes include plain unique keys such as
`receipt_id`, partial unique keys whose predicates the query establishes,
and matching functional keys such as `LOWER(username) = LOWER($1)`.
A missing partial predicate or a different expression still reports;
repeat the predicate and match the indexed expression, or cap the query.

The [saved key-proof cases](../../test-cases/rules/postgres-bounded-statements/issue-3114/)
pair these accepted forms with still-reported controls. Each shape has a
normalized CLI snapshot checked at one and two jobs.
### Projected keys and catalog views

A bounded data-modifying CTE can drive another keyed write inside a CTE:
`WITH changed AS (UPDATE input_rows SET error_message = NULL WHERE id = $1
RETURNING batch_id), updated AS (UPDATE work_batches SET completed_rows = 1
FROM changed WHERE work_batches.id = changed.batch_id RETURNING work_batches.id)
SELECT * FROM updated` stays bounded. The outer `SELECT` preserves the changed
target's bound. A non-key restriction such as `WHERE error_message = $1` does
not bound the first write; use a key lookup or select a capped batch first.

`WITH selected AS (SELECT unnest($1::uuid[]) AS id)` can supply
`WHERE id = ANY(ARRAY(SELECT id FROM selected))`. The typed built-in `unnest`
proves scalar elements, and its parameter array supplies the row bound.
Selecting every `id` from a table instead remains unbounded. Keep the source
caller-sized or cap its rows; unknown element types do not gain scalar proof.
As with the rule's other builtin proofs, bare names assume standard PostgreSQL
builtins. User-defined overloads and search-path shadowing of builtin names are
outside this static model; use `pg_catalog.unnest` to state the builtin identity.

A recursive CTE used by `UPDATE ... FROM` or `DELETE ... USING` can supply a
finite key domain when both arms project keys from the same independently
bounded CTE. The recursive arm may rejoin that CTE by primary key, including
capped lateral lookups. This proves a bound on changed target keys, not on
recursive output rows or termination. An anchor's `LIMIT` alone does not bound
a recursive arm that walks an unrestricted table or CTE. Keep both arms inside
the bounded source, or rewrite the mutation to select an explicit capped batch.
The donor must be declared in the same `WITH` scope and use default or
`MATERIALIZED` evaluation. Explicit `NOT MATERIALIZED` donors and donors from
an outer scope remain unsupported for this proof; use a local materialized
bounded source. Ordinary SELECT joins do not gain a recursive row-count bound.
Recursive finite-domain credit through view writes or `IN (SELECT ...)` remains
unsupported; use a direct keyed table write with `FROM` or `USING`.

An unqualified key or `columnBounds` column still resolves beside a catalog
view when the view's definition explicitly names its output columns and none
owns that name. This column evidence is independent of view key inference or
`viewBounds`. An ambiguous name, wildcard or unknown projection stays
conservative; qualify the column with its intended relation alias.

The [exact key-projection fixtures](../../test-cases/rules/postgres-bounded-statements/issue-3114-exact/fixture/)
include each reported form, a passing variant and a still-reported control.

A capped CTE keeps its bound for a later statement. That includes a join on a non-`id` unique key, a one-row candidate, and a derived `LIMIT 1` subquery:

```sql
WITH c AS (SELECT email FROM accounts ORDER BY email LIMIT $1)
UPDATE accounts AS a SET name = 'n' FROM c WHERE a.email = c.email;
```

`INSERT ... RETURNING` yields one row per source row. A `VALUES` list or a capped `SELECT` keeps that cap for the statement that uses the returned key. An uncapped `SELECT` source does not.

`jsonb_to_recordset($1)` and `jsonb_populate_recordset(NULL, $1)` are caller-bounded, the same as `jsonb_array_elements($1)`, when the argument is a caller-supplied value.

A `LIMIT` on a CTE bounds the rest of that statement when each later data-modifying CTE writes rows identified by a complete primary key of the capped rows. A non-unique join inside the capped CTE does not reopen those later writes. The same statement without the `LIMIT` still reports.

A call to a function the rule cannot prove scalar in the capped CTE's select list (`fn_remaining(g.id) AS rem`) may return a set that skips input rows, so the `LIMIT` is not trusted and the statement reports the CTE's tables. Generated catalogs record `functions[signature].returnsSet` from `pg_proc.proretset`. When every recorded overload of a function is declared non-set, a projected call preserves the cap, including calls in `ORDER BY`. The rule first uses `returnsSet` metadata; when it is absent, a complete, parseable `CREATE FUNCTION ... RETURNS type` definition supplies the same scalar proof. `RETURNS SETOF type` and `RETURNS TABLE (...)` keep the finding, including when they contradict `returnsSet: false`; `returnsSet: true` also always keeps it. An unknown function, malformed or unresolved declaration, or any set-returning or unknown overload stays conservative. Bare names use the selected catalog schema; a qualified declaration and call must match the catalog identity. A builtin scalar such as `lower(g.note)` needs no catalog evidence. Removing the `LIMIT` or the complete-key pin still reports even with scalar-function evidence. Catalog-dependent proofs retain the original embedded SQL source locations, including imported fragments, so line suppressions still apply to the physical relation line.

Rows from `jsonb_to_recordset($1)` stay caller-bounded as they pass through later primary-key updates, including a final read of the returned keys. When that recordset reads a table column instead of a caller value, the finding names the table that supplied the document, not the primary-key target. `INSERT ... RETURNING` of a caller-supplied array, then `CROSS JOIN LATERAL unnest(cte.column)` matched back by primary key, is the same caller bound. An array taken from a table is not.

`id = ANY(ARRAY(SELECT alias.column FROM capped))` is bounded the same way as `id IN (SELECT alias.column FROM capped)`. The name inside the array subquery may be qualified, as in `a.id`.

A pin on a derived `UNION ALL` table reaches each arm that projects that column. Pins on a constant projection stay on the arm that actually reads the column:

```sql
SELECT 1 FROM (
  SELECT id, email FROM accounts
  UNION ALL
  SELECT id, email FROM accounts
) binding
WHERE binding.id = ANY($1) OR binding.email = ANY($2);
```

The same pin reaches a base column projected through an inner derived table:

```sql
SELECT 1 FROM (
  SELECT q.id AS id, q.email AS email
  FROM (SELECT id, email FROM accounts) q
) binding
WHERE binding.id = ANY($1) OR binding.email = ANY($2);
```

Declared `columnBounds` follow that push. A predicate on the derived column reaches the base column of each arm that projects it, so a declaration for `owner_placements.owner_id` applies to `b.owner_id = ANY($1::uuid[])` after the arm list is expanded. An arm that projects `NULL` for a column does not have to prove that predicate. With a declaration for each filtered base column, and a primary-key pin on the joined row, the `UNION ALL` scan is bounded. `ORDER BY ... FOR UPDATE OF` that row does not drop the proof. Without the declarations the scan still reports.

`ANY(ARRAY[one scalar])` is equality with that scalar. A nested `ARRAY[ARRAY[...]]` or a constructor with several elements is not. A bare column such as `id = $1` resolves to the `UPDATE` target when the other items do not project that column and the catalog names a single owner. A derived table's column-alias list is the complete set of names it exposes, so `src(account_id)` does not claim `id`. A positional alias on a base table does hide the catalog columns, and then a bare `id` stays unproven. A wildcard CTE does not prove that either.

`DELETE ... WHERE id = ANY(ARRAY(SELECT id FROM expired))` is bounded when `expired` itself is capped. An unbounded `UPDATE ... FROM candidate INNER JOIN ...` finding names the `UPDATE`. A positional `LIMIT $n` inside `WITH page AS MATERIALIZED (... LIMIT $n)` caps that page. A data-modifying CTE joined to the page on the target's whole primary key, including `column = ANY(ARRAY[page.column])` for each key column, stays bounded, and the outer `SELECT` of its `RETURNING` rows keeps that proof. The same statement without the `LIMIT` still reports the target.

A multi-line template retains each relation's source location before grouping, including `head + tail`, where each const is its own template. Relations in the opening template retain their source lines, including inside a fluent append. When every relation comes from explicit builder appends, the grouped finding is reported at the executor call. Prepared SQL append boundaries distinguish opening relations from appended roots regardless of escaped newlines, empty openings, or helper declaration order. Imported fragment roots are attributed to each executing caller, with the fragment location in the grouped message.

## Analysis

Raw SQL parse failures retain each failing statement's first code token. Separate
statements produce separate findings, and a `no-mistakes-disable-next-line`
directive immediately above one statement suppresses that failure independently.
Successfully parsed sibling statements remain analyzable.

### Grouped statement findings

A descendant walk or join can expose several unbounded relations, including
repeated reads of the same table. The rule emits one finding per analyzed
statement, listing every distinct unbounded catalog relation and its reason.
For example, `SELECT i.id FROM items i CROSS JOIN feeds f` reports once with
`target: "table:feeds, table:items"`; its message includes both `SELECT can return
every row of feeds` and `SELECT can return every row of items`, with the fixes
for each relation. A single-relation finding keeps its existing target and
message. JSON finding counts therefore count statements rather than relations.

CTEs and nested reads contribute to their enclosing statement. Independent
statements stay separate. Otherwise identical findings on one physical line include
the statement's SQL line, column, and origin file so distinct statement sites remain
visible. When those coordinates also match, a local site ordinal distinguishes the
findings. Unrelated files and optional SQL verdicts do not change those labels.
SQL alternatives from one executor statement share the union of their unbounded relations.
Physical statement origins keep that union intact when an earlier fragment changes
the SQL line or column. Repeated executions of one helper's statement inside an
executor call remain separate statement occurrences. Invocation identity stays stable
when an optional earlier call disappears from an alternative. `severity: advisory` uses the
same grouped shape and produces one non-failing advisory per statement.

The finding uses the earliest remaining relation location after source attribution.
For SQL imported or appended from another file, each relation is attributed to
the executing caller and its message names the contributing fragment line. Callers
that append the same fragment each get their own finding. A same-file appended
relation uses the executor call when its source is outside the opening template.

`allow` entries, `columnBounds`, and relation-line suppressions apply before
relations are grouped. An allow entry still matches one catalog object ref and
still reports when stale. A relation-line suppression removes only that relation
from the grouped message and target; a statement-start suppression covers the
whole statement. A `no-mistakes-disable-line` or `-next-line` at one executing call
suppresses only that caller's attributed finding.

### Depth-capped recursive CTEs

A `UNION ALL` recursive CTE can be bounded when its anchor is bounded and every
recursive step joins through a catalog primary or unique key. The depth output
must start at a numeric literal, increase by a positive integer literal each
step, and have a conjunctive `<` or `<=` cap against a parameter or numeric
literal (reversed comparisons also work):

```sql
WITH RECURSIVE walk(id, depth) AS (
  SELECT id, 0 FROM accounts WHERE id = $1
  UNION ALL
  SELECT a.id, w.depth + 1
  FROM accounts a JOIN walk w ON a.id = w.id
  WHERE w.depth < $2
)
SELECT id FROM walk;
```

Without the cap, a cycle can revisit the same key forever, so the recursive
lookup remains reported. A non-increasing depth, a cap bypassed by `OR`, an
unbounded anchor, or a non-key join also retains its findings.

### Correlated builder fragments

SQL fragments appended into an executed statement are checked in the recovered
outer statement. A correlated predicate such as `WHERE x = target.placement_id`
is not judged as a separate executable statement. A declared non-unique column
can use a direct qualified outer equality when the local scope is known and the
preceding outer driver is proven bounded. A bounded outer statement can
therefore pass, while an unrestricted outer scan still produces a finding.
Unknown qualifiers, local self-reads, and undeclared non-unique columns retain
their conservative findings.

If the outer SQL cannot be recovered, the existing uncertainty policy applies;
set `unanalyzable: advise` for a non-failing advisory. The rule does not invent
a standalone relation violation for a fragment whose outer context is unknown.
The default `report` policy still reports the unrecoverable executed SQL.

### Effort

Runs at `max` only. Configuring `effort: low` (rule option, repository `effort`, or the invocation override) is not an error: the rule has no low implementation, so the check warns `requested effort low; effective effort max because this rule has no low implementation` and runs at `max`. At max it evaluates executor calls in TS/JS (helpers, builders, and up to 16 finite SQL alternatives per call) and SQL files selected by `sqlInclude`. `sqlInclude` defaults to `[]`, so SQL files are skipped until globs are set.

### Unanalyzable inputs

Following recall over precision ([how analysis works](../../README.md#how-analysis-works)), inputs this rule cannot prove are never treated as clean; they follow `options.unanalyzable`.

- Inputs this rule cannot prove: executor calls whose SQL has no complete recovered alternative or exceeds the 16-alternative cap (for example `query(buildSql(filters))` with an unrecoverable helper), and SQL, embedded or in a `sqlInclude` file, that fails to parse. They follow `options.unanalyzable`:
  - `report` (default): fails as a finding, e.g. `executed SQL is not statically recoverable for bounded statements` or `SQL could not be analyzed for bounded statements`.
  - `advise`: a non-failing advisory with the same reason.
  - `ignore`: omitted.

  Definite violations remain findings under every policy. The legacy `unanalyzableSql: fail` spelling means `report`; `advise` and `ignore` keep their names.


Native SQL parse failures selected by this rule remain visible as non-failing
advisories under `unanalyzableSql: advise`, including when no executor calls
are selected. Valid sibling statements still receive ordinary rule findings.

## Suppression

SQL and TypeScript honor `-- no-mistakes-disable-line`, `-- no-mistakes-disable-next-line`
and `-- no-mistakes-disable-file` for `postgres-bounded-statements`. To exempt a table
everywhere, use `allow: [{object: 'table:<name>', reason}]`.

SQL recovered from an imported helper is reported at the executing caller; the grouped message names the helper's contributing file and line. Suppression still checks the helper's physical SQL location before grouping. A directive in the helper's file suppresses that contribution, and so does a directive in the calling file (file-level, on the executor call, or on the imported-helper call). See [Helper-line attribution and suppression](../postgres-facts.md#helper-line-attribution-and-suppression).

A line directive covering the first line of a statement suppresses every relation
in that statement, including when its `FROM` or target is on a later line.
Directives on a relation's own reported line also work. This applies to SQL files
and SQL embedded in executor calls; statement-start directives use the physical
source line of the outer SQL keyword, including `WITH` and executing
`EXPLAIN ANALYZE` wrappers. A directive on an inner write keyword remains valid.
For concatenated embedded SQL, the operand containing the outer keyword owns
that statement-start directive.

## Related rules

See [related behavior](postgres-bounded-statements-related.md) and [continued cases](postgres-bounded-statements-related-more.md).

Stale allow reporting retains this rule's established diagnostic messages and
severity for checked inputs. Entries outside the checked scope stay silent;
uncertain or partial selector coverage cannot prove staleness. See
[allow-entry hygiene](../configuration/suppressions.md#allow-entry-hygiene).

### Stale SQL suppressions

Aggregate `check` audits `no-mistakes-disable-next-line` comments inside
statically recovered executor SQL, as well as native SQL and host comments.
When the statement becomes accepted, remove its stale directive; otherwise
`unused-disable-directive` identifies the physical comment line. A directive
that still suppresses a finding remains used. Unexecuted SQL examples are not
executor SQL. See [suppression hygiene](../configuration/suppressions.md).
