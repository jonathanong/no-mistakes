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

## Closed identifier construction

At max effort, [closed identifier values](../postgres-facts/closed-identifiers.md)
can expand raw SQL construction into at most 16 alternatives. Correlated record
fields keep the same key choice. A helper return keeps the choices of the
arguments that selected it, so a record field and a helper selected by the same
literal-union argument do not publish the cross pairs. Each statement must
independently satisfy its catalog/key bounds; one bounded variant does not
exempt another unbounded one.
Unknown branches, a transformed or reassigned argument, a helper call that
produces no arm, mutation, escaped collections, or budget overflow retain the
configured unanalyzable policy. An empty helper result does not clear the
statement.

The rule accepts optional `sqlIdentifierAssertions` entries
`{module, name, allowlistArgument}` and `rawSqlTags` entries `{module, name}`,
both defaulting to `[]`. The argument index is zero-based. These options stay
on the rule even with project PostgreSQL scope. Import aliases are supported;
assertion module matching is exact, while raw tags follow trusted-tag module
and subpath matching. Existing parameterized tags continue to bind values.
See the shared page for examples, counterexamples, suppression, and contracts.

## Why and when

See [rationale](postgres-bounded-statements-rationale.md).

## Independent candidate bodies

`materializedBodies` defaults to `ignore`; candidate-pool checks are opt-in.
Set it to `require-bound` (an alias of `advise`) to receive non-failing
advisories, or `report` to make violations findings. This option checks
explicit `AS MATERIALIZED` CTEs and pool-building subquery bodies independently of the surrounding
statement. Ordinary and `AS NOT MATERIALIZED` CTE declarations retain their
existing behavior. Recursive bodies keep the existing depth-predicate proof.
The `statements` filter selects the owning executed statement: selecting UPDATE
or DELETE also checks its CTE and subquery bodies, while unrelated SELECT bodies
remain outside that selection. Separate bodies on one line retain separate advisories.

`EXISTS` and `NOT EXISTS` bodies are existence probes, so they do not receive
independent pool diagnostics. Scalar subqueries, including SELECT-list and
`COALESCE((SELECT ...), ...)` expressions, also do not build candidate pools,
regardless of correlation. Their ordinary relation-bound checks still apply: an
unbounded uncorrelated scalar read remains a SELECT finding, including when
`materializedBodies` is `ignore`. Known correlated probes keep their existing
per-row treatment. Unqualified outer columns need catalog ownership and a
complete key proof: a bare correlated equality on a non-unique inner column
still reports. Ancestor references inside LATERAL select lists retain their
scope. A local column with the same name remains local and supplies no
correlation proof. An ordinary CTE retains
its own `ORDER BY ... LIMIT` input bound when its predicate contains a scalar
probe. `IN`, `ANY`, array subqueries and derived tables retain their
body obligations. Materialized CTEs and pool-building bodies nested inside a
probe are still checked. Scalar reads inside UPDATE or DELETE retain the owning
statement selection while reporting SELECT read bounds.

A later body that consumes an already materialized CTE inherits that pool's
obligation. Report the unbounded source CTE once rather than repeating its
underlying relations at every consumer. A consumer that adds a new unbounded
scan still reports that scan; separate consumers of an ordinary inline CTE
retain separate obligations. These body-only projections do not change the
parent statement's cardinality or key proofs.

An outer cap cannot establish a bounded candidate pool inside a body:

```sql
-- Reported with materializedBodies: report
WITH candidates AS MATERIALIZED (SELECT id FROM posts)
SELECT id FROM candidates LIMIT 20;

-- Accepted: bound the body itself
WITH candidates AS MATERIALIZED (SELECT id FROM posts LIMIT 200)
SELECT id FROM candidates LIMIT 20;

-- Reported: the outer LIMIT cannot bound the derived body
SELECT id FROM (SELECT id FROM posts) candidates LIMIT 20;
```

A body can instead restrict every relation by an existing primary/unique-key
proof, or by equality / `= ANY` on a reviewed `columnBounds` key. A closed range
with the same parameter or literal on both sides (`user_id >= $2 AND user_id <= $2`)
counts as equality for key matching. When a trusted SQL template interpolates
only primitive literals and unchanged parameters declared as `string`, `number`,
`bigint`, or `boolean`, repeated reads of a parameter retain their identity.
Function parameter fields have a plain-data contract: repeated direct static
reads such as `user.id` share one binding identity, independently of named,
imported, inline, or absent type annotations. Callers must supply data values;
getters and Proxies are outside this contract. Known own data fields initialized
from an unchanged primitive parameter also qualify. Observed writes, aliases or
escapes, getters, Proxies, unrelated calls or suspensions, computed reads, and
interpolations with side effects retain the finding.
Different parameters do not share an identity; use an explicit equality or a
single stable primitive value when a field cannot satisfy the contract. The [saved template controls](../../test-cases/rules/postgres-bounded-statements/issue-3151-embedded/)
cover repeated parameters and the unsafe alternatives.
For a unique key `(group_id, user_id)`,
`group_id = $1 AND user_id >= $2 AND user_id <= $2` therefore proves a bounded
lookup. Different bounds (`>= $2 AND <= $3`), open ranges, and halves separated
by `OR` do not establish that equality; rewrite a lookup as `user_id = $2` or
supply a separate finite-key proof. For example,
`SELECT id FROM follows WHERE follower_id = $1` is bounded when
`columnBounds` declares `follows.follower_id` with a finite fan-out reason.
Outer predicates do not supply that body-local proof. Findings name the body
and its unbounded relations, at the CTE declaration or subquery start. A body
LIMIT uses the same fixed-count proofs as any other LIMIT, including a provably
non-null qualified column of a visible parent relation.

```yaml
options:
  schemaCatalogPath: schema.json
  materializedBodies: report
  columnBounds:
    follows.follower_id:
      reason: Application enforces at most 500 followed accounts per viewer
```

Use `-- no-mistakes-disable-next-line postgres-bounded-statements` immediately
before the CTE/body statement, `-- no-mistakes-disable-line` on its line, or
`-- no-mistakes-disable-file postgres-bounded-statements` for a file exception.
The existing `allow` list can exempt a catalog relation with a reason. Prefer a
body-local LIMIT or a documented finite key over suppressing the check: this
policy prevents capped queries from constructing unbounded intermediate pools.


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
nothing is reported as stale at the catalog path. An entry whose every covered
statement the domain proof accepts on its own is reported once, on that allow
entry in the rules file. The catalog's name for the table is the object ref.

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
  For a composite view key `(tenant, id)`, pin both columns in every `OR` arm.
  `WHERE tenant = $1 AND id = $2` is a key lookup; `WHERE id = $2` leaves the
  view unbounded because the tenant column is still open.
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

### Finite unique-key domains

`maxDomainRows` defaults to unset. Leave it unset to keep the current findings.
Set it to a non-negative integer to accept a lookup whose free unique-key
columns each have a provable finite domain, when the product of those domain
sizes is at most `maxDomainRows`. Equality-pinned columns do not multiply. A
domain is a validated `CHECK (col BETWEEN a AND b)` or
`CHECK (col >= a AND col <= b)` on a non-null integer column, a non-null enum,
a non-null boolean, or a closed inclusive literal range in the query predicate,
including a `date` column between two `YYYY-MM-DD` date literals. A query range matches
the column's visible name, so `FROM alias_slots AS renamed(n)` is closed by
`WHERE n BETWEEN 1 AND 5`. A catalog check still names the physical column.
Several checks and query bounds intersect. An integer type's width is not a domain: `smallint` and
`integer` stay reported unless a check or a literal range closes them. Nullable
columns, checks that are not validated, `OR` checks, and open ranges are not
domains. `OR` branches add their products and the sum must fit the cap. The
finding for a product above the cap names that product, including a materialized-body finding. A product within the
cap accepts the statement silently, the same way a `LIMIT` or a unique-key
lookup does. The domain proof runs only for a statement that would otherwise
be a finding, so a statement an allow entry, a `LIMIT`, or another bound
already accepts keeps that result. When the domain proof alone accepts every
statement an allow entry covers, that entry is reported once, on the allow
entry in the rules file.
`maxDomainRows: 0` accepts no extra statement.

### Executor configuration

Omitting `importSpecifier`, `executorNames`, and `extraExecutorNames` is a configuration error.
Set `importSpecifier` to your database module, list `executorNames`, or set `extraExecutorNames`.
Use `executorNames: []` without a module to skip imported executor calls and retain
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
neither counts as executor selection: still set `importSpecifier`, `executorNames`, or `extraExecutorNames`.
With a selector set, they still scan direct calls of the scoped binding. `.query` on that binding stays off unless a module or an explicit `query` name enables member queries.
With `importSpecifier` set, these two options also match imports from its subpaths
(`@example/db/types` for `@example/db`) but not sibling packages such as
`@example/dbx`. A relative import (`../transaction`) matches only when the resolved
file is inside the package `importSpecifier` resolves to; a same-named import outside
that package does not. If the package root cannot be determined, relative imports do
not match. `executorNames` still requires the exact module.

Omitting `importSpecifier`, `executorNames`, and `extraExecutorNames` is a configuration error.
Set `executorNames: []` without `importSpecifier` to skip imported executor calls and
`.query` members, including `.query` on a factory-scoped receiver. Direct calls of an
`executorFactoryNames` or `executorTypeNames` binding are still scanned.
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
Literal arrays and mixed table/parameter inputs do not establish this unnest bound.

A `LIMIT` or `FETCH FIRST` count may also name a qualified column of a relation
visible from an enclosing query when that value is provably non-null. A
`NOT NULL` catalog column counts. A `VALUES` or `SELECT` column counts when
every row is a non-null literal or bind, so
`WITH windows(row_limit) AS (VALUES (2))` and `LIMIT windows.row_limit` both
count, the same way `LIMIT $2` does. Bare, unquoted `COALESCE`, `LEAST`, and
`GREATEST` count when one argument is non-null: `GREATEST` ignores NULL
arguments, so `GREATEST(windows.row_limit, 0)` is non-null even when the
column can be NULL. A nested subquery keeps columns the enclosing query can
see, even when that subquery is not `LATERAL`. A column from the nullable side
of a `LEFT`, `RIGHT`, or `FULL` join stays unlimited unless one of those
wrappers supplies a non-null value. `LIMIT NULL` is unlimited, so a nullable
parent column, a `NULL` literal projected by `VALUES` or `SELECT`, and a
null-extended outer-join column do not cap. A column of the same query, a bare
name (including a bare name when the only visible relation is a table), an
unknown qualifier, a same-level non-`LATERAL` sibling, a subquery, and an
unknown function do not cap. A bare name counts only for one preceding
`unnest` of parameter arrays. A reference that supplies its own column alias
list is not that proof. `ROLLUP`, `CUBE`, and `GROUPING SETS` can emit NULL
for a grouped column, so the column itself is not proof; a literal or
`GREATEST`, `COALESCE`, or `LEAST` with a non-null argument still is. A set
operation takes output names from its left arm. An alias list shorter than
the output list renames that prefix and keeps the remaining names. Parentheses
around a query do not drop an inner catalog column limit. A projected
set-returning function can emit no rows, so that `LIMIT` does not cap the
scan, including when it names a non-null catalog column. A parenthesized
`FROM` item is a table function, not the table or CTE of that name, so its
output does not inherit that nullability. `ctid` counts only on a catalog
base table. `EXCEPT`, `INTERSECT`, and duplicate-eliminating `UNION` still
report their input scans when the cap is a parent column; `UNION ALL` can
stop at that cap.

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
A validated composite foreign key whose referencing columns are all `NOT NULL`
also fills the column that the statement never equates, when the equated
referencing column maps to the referenced primary key or another plain unique
key. A nullable referencing column, a referenced column set that is not a
unique key, and an unpinned unique-key column stay findings.

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
A column past a short alias list stays visible under its own name. A name that
the list already uses does not also name the later column. A foreign key the
transaction can postpone still fills, because a postponed violation removes the
child from an inner join and the bound cannot grow.
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
`receipt_id`, partial unique keys whose predicates the query establishes
(the same `IS NULL`, literal equality, or `IN` / `ANY` literal set, including
a parameter bound to that literal in the same alternative, and a catalog enum
parameter when every label hits a pinned unique index whose source rows are
themselves bounded),
and matching functional keys such as `LOWER(username) = LOWER($1)`.
A strict call also proves the argument is not null when that column's catalog
type is `text` or `varchar`: `lower(username) = lower($1)` matches
`UNIQUE (lower(username)) WHERE username IS NOT NULL`. The same holds for
unquoted `upper` and one-argument `btrim`, unqualified or `pg_catalog`-qualified.
`lower` of a user-defined type and `lower` of an integer stay findings, because
that overload need not be strict. `coalesce`, `IS NOT DISTINCT FROM`, a quoted name,
and another schema do not. The call is recorded for a base table the condition
restricts (`WHERE`, or the filtered side of a join). An outer join `ON` does not
record the preserved table. A `RIGHT JOIN` records only its left side, and a
`FULL JOIN` records neither side. A derived item is not a base table. Raw equality
on any type still implies that column `IS NOT NULL`.
A literal cast counts only when it preserves the value and the column type
compares the same way
(`text`, `boolean`, unlimited `varchar`, `varchar(n)` that fits, or a catalog
enum label, including a quoted mixed-case enum name). `citext` and
`char` / `character` do not treat `'a'::text` as `'a'`. A stored `(col)::text`
predicate matches only a text-family column, so a domain does not match it.
A `varchar` partial index stored as `(col)::text = ANY (ARRAY[...])` matches
that literal set, and it does not match `'pending'::citext`.
An array constructor
cast such as `ARRAY['pending', 'ready']::text[]` uses that rule on each element.
A scalar cast through an array type does not, so `'pending'::varchar(3)` and
`('{"pending"}'::text[])::text` do not match `'pending'`.
Raw equality also fixes a recognized function of that column: `username = $1`
matches `UNIQUE (lower(username))` for unquoted `lower`, `upper`, and
one-argument `btrim` when the column type is `text` or `varchar`. Range
`lower` and `upper` stay unrecognized. A cast matches only inside one equality family
(`text` with `varchar`, the integer types, `uuid`, or `boolean`).
`lower(col) = $1` against a unique index on raw `col` still reports.
Redundant predicate parentheses and quoted lowercase identifiers preserve the
same partial-index proof, including inside scalar closed-range lookups.
A missing partial predicate or a different expression still reports;
repeat the predicate and match the indexed expression, equate the underlying
column when the expression is recognized, or cap the query.
Each `OR` arm proves `col IS NOT NULL` from its own strict equality or `IN`.
Another arm does not inherit that proof, and `IS NOT DISTINCT FROM` does not
supply it.
Each `OR` arm is proved on its own against the shared conjuncts, so two arms may
use different partial unique keys. An `IN` subquery counts only when the
subquery is bounded. `LIKE`, an unbounded `IN`, and a missing shared predicate
still report. The
[issue 3436 fixtures](../../test-cases/rules/postgres-bounded-statements/issue-3436/fixture/)
lock that split.

The [saved scalar-scope controls](../../test-cases/rules/postgres-bounded-statements/issue-3151-scopes/)
cover unqualified and LATERAL correlation, partial-index predicates, CTE limits,
and unbounded scalar reads that must still report. The attachment fixture gives
the outer `parent_id` lookup its own unique key to isolate the scalar proof. The
partial-index control also covers a boolean scalar inside `COALESCE`, combined
with an independent `EXISTS` in a SELECT list without an outer FROM. A nested
LATERAL lookup may complete a composite key using columns from different
visible ancestor rows; every key column must still have a finite source.
UNION ALL arms inside that LATERAL source inherit the same visible ancestors,
including nested JOIN LATERAL and LEFT JOIN LATERAL lookups. Ordinary derived
queries and CTEs keep their own scope boundaries. The
[saved embedded controls](../../test-cases/rules/postgres-bounded-statements/issue-3151-real/)
exercise fragment-built SQL and SQL-language catalog functions, together with
missing composite-key halves that must still report.

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
Additional controls retain findings for a non-unique target join, a custom
`app.unnest` call, an unrestricted extra recursive arm, and view column aliases
that make a bare name ambiguous. Keep the complete target key, use the builtin
caller-sized source, constrain every recursive arm, and qualify ambiguous names.

A capped CTE keeps its bound for a later statement. That includes a join on a non-`id` unique key, a one-row candidate, and a derived `LIMIT 1` subquery:

```sql
WITH c AS (SELECT email FROM accounts ORDER BY email LIMIT $1)
UPDATE accounts AS a SET name = 'n' FROM c WHERE a.email = c.email;
```

`INSERT ... RETURNING` yields one row per source row. A `VALUES` list or a capped `SELECT` keeps that cap for the statement that uses the returned key. An uncapped `SELECT` source does not.

`jsonb_to_recordset($1)` and `jsonb_populate_recordset(NULL, $1)` are caller-bounded, the same as `jsonb_array_elements($1)`, when the argument is a caller-supplied value.

A `LIMIT` on a CTE bounds the rest of that statement when each later data-modifying CTE writes rows identified by a complete primary key of the capped rows. A non-unique join inside the capped CTE does not reopen those later writes. The same statement without the `LIMIT` still reports.

A call to a function the rule cannot prove scalar in the capped CTE's select list (`fn_remaining(g.id) AS rem`) may return a set that skips input rows, so the `LIMIT` is not trusted and the statement reports the CTE's tables. Generated catalogs record `functions[signature].returnsSet` from `pg_proc.proretset`. When every recorded overload of a function is declared non-set, a projected call preserves the cap, including calls in `ORDER BY`. The rule first uses `returnsSet` metadata; when it is absent, a complete, parseable `CREATE FUNCTION ... RETURNS type` definition supplies the same scalar proof. `RETURNS SETOF type` and `RETURNS TABLE (...)` keep the finding, including when they contradict `returnsSet: false`; `returnsSet: true` also always keeps it. An unknown function, malformed or unresolved declaration, or any set-returning or unknown overload stays conservative. Bare names use the selected catalog schema; a qualified declaration and call must match the catalog identity. A builtin scalar such as `lower(g.note)` needs no catalog evidence. `left`, `right`, and `make_interval` are builtin scalars from PostgreSQL 12 or earlier, including `LEFT(title, 80)`, `RIGHT(title, 40)`, and named arguments `make_interval(secs => $1)` or `make_interval(secs := $1)`, so a `LIMIT` on a select list that uses only those calls stays trusted. `LEFT JOIN` and `RIGHT JOIN` remain join syntax. A same-named function in another schema, such as `app.left` or `app.make_interval`, stays unknown and the `LIMIT` is not trusted. The [function-call fixtures](../../test-cases/rules/postgres-bounded-statements/issue-3397/) cover those calls, the join keywords, the reported `CASE` in a CTE select list, and the already-clean controls. `name := value` is a named argument, the same as `name => value`, including inside that `CASE`, in a function call such as `make_interval(days => 1, secs => $1)`, and in a `LATERAL` function call such as `LATERAL make_interval(days := 1, secs := $1)`. `LEFT` without a `LIMIT`, and `app.left`, stay reported. Removing the `LIMIT` or the complete-key pin still reports even with scalar-function evidence. Catalog-dependent proofs retain the original embedded SQL source locations, including imported fragments, so line suppressions still apply to the physical relation line.

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
WITH RECURSIVE ancestors AS (
  SELECT id, parent_id, 1 AS depth FROM comments WHERE id = $1
  UNION ALL
  SELECT c.id, c.parent_id, a.depth + 1
  FROM comments c JOIN ancestors a ON c.id = a.parent_id
  WHERE a.depth < 50
)
SELECT * FROM ancestors;
```

Without the cap, a cycle can revisit the same key forever, so the recursive
lookup remains reported; add a conjunctive depth cap to fix it. The catalog must
include a valid, immediate primary or unique index for the joined key.
A non-increasing depth, a cap bypassed by `OR`, an
unbounded anchor, or a non-key join also retains its findings.

### Correlated primary-key subqueries

A subquery can use an equality between its primary or unique key and a known
outer row's qualified column. The lookup is bounded per outer row; it does not
make an unrestricted outer scan bounded. For example, both lookups below pass
when `placement_id` has a valid primary-key index in the catalog:

```sql
SELECT target.placement_id FROM image_placements target
WHERE target.placement_id = $1 AND target.created_by_id IN (
  SELECT p.created_by_id FROM image_placements p
  WHERE p.placement_id = target.placement_id
);
```

Scalar projections such as `(SELECT p.created_by_id FROM image_placements p
WHERE p.placement_id = target.placement_id)` also remain accepted when the
outer lookup is bounded. Changing the inner predicate to the non-unique
`p.created_by_id = target.created_by_id` retains an inner finding in the `IN`
example; use a primary/unique key or declare a reviewed `columnBounds` contract
to fix it. Removing `target.placement_id = $1` retains an outer finding.
Unknown outer qualifiers, local self-comparisons, and an `OR` branch without a
complete key do not prove a bounded lookup. In an `IN` subquery, an unrelated
bare read from an explicitly aliased unknown table does not erase an
independently proven qualified key equality. That unknown source still bounds
nothing, and the outer query must establish its own bound.

### Correlated builder fragments

Straight-line `.append()` mutations of a local SQL statement are evaluated before
the executor consumes it. A separately appended `LIMIT` caps the statement:

```ts
const statement = sql`SELECT id FROM accounts WHERE name = ${name} ORDER BY id`;
statement.append(sql` LIMIT ${limit}`);
return read(statement);
```

A local keyed subject also retains its bound when appended into a derived table:

```ts
const subject = sql`SELECT id FROM accounts WHERE id = ${id}`;
const statement = sql`SELECT s.id FROM (`;
statement.append(subject).append(sql`) s`);
return read(statement);
```

These examples assume configured executors and SQL tags, and a catalog primary
or unique key on `accounts.id`. Appending only `OFFSET 1` to the first example
still reports: an offset does not cap rows. Replacing the second example's key
predicate with a non-unique `name` predicate also reports at the executing
`read(statement)`, following appended-fragment attribution. Fix those controls
by appending a `LIMIT` before execution or matching the complete catalog key.
An append after execution cannot bound the earlier read.

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

## Declared statement bounds

A statement can be bounded by something the schema cannot show, such as an application page size on a join the catalog cannot cap. A reviewed declaration names that limit and why it is finite. It suppresses only `postgres-bounded-statements` findings for that one statement. Other rules still run, and SQL this rule cannot analyze stays reported.

Put the declaration in a line comment on the statement's outer keyword line, or on the line immediately above it. That keyword is `SELECT`, `UPDATE`, `DELETE`, `WITH`, or an executing wrapper such as `EXPLAIN ANALYZE`. `--`, `//`, and `#` comments all work, in a `.sql` file or in SQL embedded in an executor call. A `--` comment inside a template literal counts; a directive inside a string does not.

```sql
-- no-mistakes-bound postgres-bounded-statements rows<=100: caller passes one page of ids
SELECT c.id, o.id
FROM candidates c
JOIN orders o ON o.candidate_id = c.id
WHERE c.id = ANY($1::uuid[]);
```

`orders.candidate_id` is not unique, so a page of candidate ids does not prove how many order rows the join returns. The declaration records that cap. `rows<=N` is a positive integer and the justification after the colon is required. `rows<=0` is a finding and does not hide the read. This declaration has no justification, so it is a finding and the unbounded read is still reported:

```sql
-- no-mistakes-bound postgres-bounded-statements rows<=100
SELECT id, email FROM candidates;
```

A missing `rows<=N` limit is the same kind of finding. A declaration that does not sit on a checked statement, and more than one declaration on one statement, are also findings. None of those hide the underlying unbounded read.

A line with more than one checked statement makes its declaration ambiguous. That includes two SQL statements written on one line and query-builder alternatives that share a line. The declaration is a finding and clears none of them. Put each statement on its own line with its own `rows<=N` declaration.

A declaration on a statement the rule can already prove bounded is stale. Primary-key equality and `id = ANY($1::uuid[])` on that key are already proved, because the parameter list cannot return more rows than it contains. Remove the declaration:

```sql
-- no-mistakes-bound postgres-bounded-statements rows<=1: id is the primary key
SELECT id, email FROM candidates WHERE id = $1;
```

Accepted declarations are omitted from `rules`. `check --format json` and YAML list them in `acceptedBounds` (`rule`, `file`, `line`, `rows`, `justification`). `line` is the comment. The field is omitted when empty and does not by itself fail the check.

This does not replace `allow`, which exempts a relation everywhere. `no-mistakes-disable-line`, `no-mistakes-disable-next-line`, and `no-mistakes-disable-file` suppress declared-bound findings on the lines they cover. A directive that covers the declaration or the statement omits `acceptedBounds` for that declaration. A directive that covers only the declaration does not hide an unbounded read on another line.

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

## Invalid UTF-8

Source text this rule reads, including standalone SQL, is decoded lossily, so a stray byte does not drop the file or replace it with an encoding failure. Other read errors contribute no source text. `unanalyzableSql` still applies to SQL the statement pass cannot parse.
