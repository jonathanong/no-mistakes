# `postgres-bounded-statements`

A `projects: [<name>]` entry inherits [`projects.<name>.postgres`](../configuration/projects.md#postgresql-scope) when that block is set. Do not restate `importSpecifier`, `executorNames`, `executorFactoryNames`, `executorTypeNames`, `trustedSqlTags`, `schemaCatalogPath`, or `unanalyzableSql` on the rule. `include`, `exclude`, and `sqlInclude` may only narrow the project scope.

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
"public.activity_feed": unbounded}`. These are explicit cardinality assertions;
the rule does not infer a bound from a view's definition.

- `bounded` is an unchecked user assertion that the view supplies a finite set of
  rows. The rule trusts it without inspecting the SQL definition, including when
  the view is an `UPDATE` or `DELETE` target. For example, a `GROUP BY` or `UNION`
  view is still trusted if configured `bounded`; confirm its cardinality yourself.
  A bounded view can also size another relation through a unique-key join.
- `unbounded` treats the view as an unsized relation. A statement cap can bound
  it; otherwise the finding targets `view:<catalog-name>`.
- Views without a policy retain their `unknown-relation` verdicts. Use
  `unanalyzableSql: advise` for residual advice and `--include-sql-verdicts` to
  inspect the individual verdicts.

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

A capped CTE keeps its bound for a later statement. That includes a join on a non-`id` unique key, a one-row candidate, and a derived `LIMIT 1` subquery:

```sql
WITH c AS (SELECT email FROM accounts ORDER BY email LIMIT $1)
UPDATE accounts AS a SET name = 'n' FROM c WHERE a.email = c.email;
```

`INSERT ... RETURNING` yields one row per source row. A `VALUES` list or a capped `SELECT` keeps that cap for the statement that uses the returned key. An uncapped `SELECT` source does not.

`jsonb_to_recordset($1)` and `jsonb_populate_recordset(NULL, $1)` are caller-bounded, the same as `jsonb_array_elements($1)`, when the argument is a caller-supplied value.

A `LIMIT` on a CTE bounds the rest of that statement when each later data-modifying CTE writes rows identified by a complete primary key of the capped rows. A non-unique join inside the capped CTE does not reopen those later writes. The same statement without the `LIMIT` still reports.

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

`DELETE ... WHERE id = ANY(ARRAY(SELECT id FROM expired))` is bounded when `expired` itself is capped. An unbounded `UPDATE ... FROM candidate INNER JOIN ...` finding names the `UPDATE`. A multi-line template, including `head + tail` where each const is its own template, reports the source line that contains the SQL.

## Analysis

Raw SQL parse failures retain each failing statement's first code token. Separate
statements produce separate findings, and a `no-mistakes-disable-next-line`
directive immediately above one statement suppresses that failure independently.
Successfully parsed sibling statements remain analyzable.

### Cascading relation findings

A descendant walk can expose several unbounded relations, including repeated
reads of the same table. The rule emits one finding per analyzed statement,
listing its unbounded roots in sorted order. For multiple roots, `target` is a
comma-separated list of catalog object refs, such as
`table:accounts, table:orders`. A single root keeps its ordinary object ref.
Independent statements remain separate, even on the same source line. SQL
alternatives from one executor statement share the union of their unbounded
roots. The finding sits at the executed statement, not at a shared fragment
that the statement appends from another function or file: callers that append
the same fragment each get their own finding, and a directive at one caller
suppresses only that caller. When a root comes from an appended fragment, the
message names the fragment's location. A statement made only of appended
fragments is reported at its executor call.

`allow` entries are applied per root and still report stale entries. A
relation-line suppression suppresses that relation before the remaining roots
are grouped; a statement-start suppression covers the whole statement.

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

## Suppression

SQL and TypeScript honor `-- no-mistakes-disable-line`, `-- no-mistakes-disable-next-line`
and `-- no-mistakes-disable-file` for `postgres-bounded-statements`. To exempt a table
everywhere, use `allow: [{object: 'table:<name>', reason}]`.

SQL recovered from an imported helper is reported in the helper's module at the helper's line. A directive in the helper's file suppresses it, and so does a directive in the calling file (file-level, on the executor call, or on the imported-helper call). See [Helper-line attribution and suppression](../postgres-facts.md#helper-line-attribution-and-suppression).

A line directive covering the first line of a statement suppresses every relation
finding in that statement, including when its `FROM` or target is on a later line.
Directives on a relation's own reported line also work. This applies to SQL files
and SQL embedded in executor calls; statement-start directives use the physical
source line of the outer SQL keyword, including `WITH` and executing
`EXPLAIN ANALYZE` wrappers. A directive on an inner write keyword remains valid.
For concatenated embedded SQL, the operand containing the outer keyword owns
that statement-start directive.

## Related rules

See [related behavior](postgres-bounded-statements-related.md) and [continued cases](postgres-bounded-statements-related-more.md).
