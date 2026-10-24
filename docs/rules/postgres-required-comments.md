# `postgres-required-comments`

A `projects: [<name>]` entry inherits [`projects.<name>.postgres`](../configuration/projects.md#postgresql-scope) when that block is set. Do not restate `importSpecifier`, `executorNames`, `executorFactoryNames`, `executorTypeNames`, `trustedSqlTags`, `schemaCatalogPath`, or `unanalyzableSql` on the rule. `include`, `exclude`, and `sqlInclude` may only narrow the project scope.

A table, column, view, or materialized view is reported when its schema
comment is missing or shorter than `minLength`. With only `schemaCatalogPath`
set, the rule checks tables and partitioned tables and ignores columns and
views. `schemaCatalogPath` is required; enabling the rule without it is a
configuration error.

```yaml
rules:
  - rule: postgres-required-comments
    scope: repository
    options:
      schemaCatalogPath: db/schema.json
      objects: [table, view, materialized-view, column]
      columnNamePatterns: []
      exemptColumnNamePatterns: ['^(id|created_at|updated_at)$', '^vendor_']
      minLength: 10
```

## Why and when

A schema comment is the documentation that travels with the database. It shows
up in psql, in generated types, and in AI tooling. Without a check, new tables
and views land with no `COMMENT ON`.

## What it catches/requires

The rule reads comments from the schema catalog. A comment is missing when it
is absent or when its trimmed length is below `minLength`. Whitespace alone
counts as missing. `table` includes partitioned tables, and a column comment
recorded on the partitioned parent counts.

- `table:<name>`: `table has no COMMENT ON TABLE`, or
  `table comment is shorter than <minLength> characters`. The finding also
  says to add `COMMENT ON` with at least `minLength` characters.
- `column:<table>.<column>`: `column has no COMMENT ON COLUMN`, or
  `column comment is shorter than <minLength> characters`.
- `view:<name>`: `view has no COMMENT ON VIEW`, or
  `view comment is shorter than <minLength> characters`.
- `materialized-view:<name>`: `materialized view has no COMMENT ON MATERIALIZED VIEW`,
  or `materialized view comment is shorter than <minLength> characters`.

When `column` is selected, an empty `columnNamePatterns` list checks every
column. Otherwise a column is checked only when its name matches one pattern.
`exemptColumnNamePatterns` is applied after that and skips the column even
when a pattern selected it.

## Options and defaults

`schemaCatalogPath` is required and names a catalog generated with [`no-mistakes postgres catalog`](../cli/postgres.md). `objects` defaults to `[table]`. An empty
list, an unknown value, or a duplicate value is a configuration error.
`columnNamePatterns` and `exemptColumnNamePatterns` default to `[]`. An
invalid regex is a configuration error. `minLength` defaults to `1` and must
be at least 1. `allow` defaults to `[]`. `include`, `exclude`, and `projects`
apply to the schema catalog path. An application whose filters miss that path
reports nothing for it.

## Valid example

`COMMENT ON TABLE orders IS 'One row per checkout; immutable after payment'`.
The same snapshot also documents `orders.display_name`, a regular view, and a
materialized view.

```json
{
  "formatVersion": 2,
  "coverage": "complete",
  "tables": {
    "orders": {
      "comment": "One row per checkout; immutable after payment",
      "columns": {
        "display_name": {
          "dataType": "text",
          "comment": "Customer visible display name"
        }
      }
    }
  },
  "views": {
    "active_orders": {
      "definition": "SELECT id FROM orders WHERE active",
      "comment": "Orders that can still be changed"
    },
    "daily_order_totals": {
      "materialized": true,
      "definition": "SELECT count(*) FROM orders",
      "comment": "Daily count of all orders"
    }
  }
}
```


## Counterexample

`orders` has no comment. The other three independent examples omit a selected
column, view, or materialized-view comment while keeping unrelated objects
commented.

```json
{
  "formatVersion": 2,
  "coverage": "complete",
  "tables": {
    "orders": {
      "columns": { "id": { "dataType": "uuid" } }
    }
  }
}
```

`orders.display_name` has no column comment.

```json
{
  "formatVersion": 2,
  "coverage": "complete",
  "tables": {
    "orders": {
      "comment": "One row per checkout; immutable after payment",
      "columns": {
        "id": { "dataType": "uuid" },
        "display_name": { "dataType": "text" }
      }
    }
  }
}
```

`active_orders` has no view comment.

```json
{
  "formatVersion": 2,
  "coverage": "complete",
  "tables": {
    "orders": {
      "comment": "One row per checkout; immutable after payment",
      "columns": { "id": { "dataType": "uuid" } }
    }
  },
  "views": {
    "active_orders": { "definition": "SELECT id FROM orders" }
  }
}
```

`daily_order_totals` has no materialized-view comment.

```json
{
  "formatVersion": 2,
  "coverage": "complete",
  "tables": {
    "orders": {
      "comment": "One row per checkout; immutable after payment",
      "columns": { "id": { "dataType": "uuid" } }
    }
  },
  "views": {
    "daily_order_totals": {
      "materialized": true,
      "definition": "SELECT count(*) FROM orders"
    }
  }
}
```


## Fix

Add `COMMENT ON TABLE`, `COMMENT ON COLUMN`, `COMMENT ON VIEW`, or
`COMMENT ON MATERIALIZED VIEW` with a trimmed length of at least `minLength`.
Exempt mechanical column names with `exemptColumnNamePatterns` instead of
writing filler comments.

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` (rule option, repository `effort`, or the invocation override) is not an error: the rule has no low implementation, so the check warns `requested effort low; effective effort max because this rule has no low implementation` and runs at `max`. It judges the generated schema catalog (`schemaCatalogPath`); executor calls in TS/JS are not evaluated.

### Unanalyzable inputs

Following recall over precision ([how analysis works](../../README.md#how-analysis-works)), this rule has no inputs it must guess at, so nothing is silently treated as clean. This rule has no unanalyzable inputs: it judges the configured schema catalog, a JSONC file read directly from `schemaCatalogPath` (no SQL is parsed by the rule), so every entry is either present or absent. A catalog that cannot be read, is not valid JSONC, is empty, or is not a valid snapshot is a hard error, not an unanalyzable input. The repository-level `unanalyzable` default does not affect it.

## Suppression

For an intentional exception, add an entry under the rule's `options.allow`:

```yaml
allow:
  - object: table:schema_migrations
    reason: Owned by the migration runner
```

Snapshot JSON cannot carry `no-mistakes-disable` comments. Suppress an object
with `allow: [{object, reason}]`. `object` is `table:<name>`,
`column:<table>.<column>`, `view:<name>`, or `materialized-view:<name>`, and
`reason` is non-empty. An entry that matches nothing is reported as a stale
allow entry. An empty reason, an invalid object ref, or a duplicate object
fails configuration.

## Related rules

`postgres-column-requires-trigger`, `postgres-conflict-ordering`, and
`postgres-lock-ordering` read the same schema catalog. This rule reads table,
column, and view comments from that catalog.
