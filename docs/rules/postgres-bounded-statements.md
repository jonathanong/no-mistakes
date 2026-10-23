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
with an object such as `table:currencies`. An empty reason, an object that is not a
catalog ref, or a duplicate is a configuration error, and an entry that suppresses
nothing is reported as stale. The catalog's name for the table is the object ref.

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

Use the same catalog so each statement fails because its row count is unbounded:

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
