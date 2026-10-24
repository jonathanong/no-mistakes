# `postgres-table-shape`

A `projects: [<name>]` entry inherits [`projects.<name>.postgres`](../configuration/projects.md#postgresql-scope) when that block is set. Do not restate `importSpecifier`, `executorNames`, `executorFactoryNames`, `executorTypeNames`, `trustedSqlTags`, `schemaCatalogPath`, or `unanalyzableSql` on the rule. `include`, `exclude`, and `sqlInclude` may only narrow the project scope.

Tables whose names match a configured pattern must have that concept's columns,
primary-key types, and triggers. A banned name is reported even when no shape
matches. With only `schemaCatalogPath` set, `shapes` and `bannedTablePatterns`
default to empty and the rule reports nothing. `schemaCatalogPath` is required.

```yaml
rules:
  - rule: postgres-table-shape
    scope: repository
    options:
      schemaCatalogPath: db/schema.json
      shapes:
        - name: revision-history
          tablePattern: '_revisions$'
          requiredColumns:
            - { name: id, type: uuid, nullable: false }
            - { namePattern: '_id$', foreignKey: true }
            - { name: revised_by_id, type: uuid, foreignKey: true, onDelete: 'set null' }
            - { name: changes, type: jsonb, nullable: false }
          forbiddenColumns: [updated_at, deleted_at]
          requiredTriggers:
            - { function: fn_reject_mutation, timing: before, events: [update, delete] }
        - name: article-votes
          tablePattern: '^article_votes$'
          requiredColumns:
            - { name: user_id, type: uuid, nullable: false, foreignKey: true, references: [users], onDelete: cascade }
            - { name: score, type: smallint, nullable: false }
        - name: comment-votes
          tablePattern: '^comment_votes$'
          requiredColumns:
            - { name: user_id, type: uuid, nullable: false, foreignKey: true, references: [users], onDelete: cascade }
            - { name: score, type: smallint, nullable: false }
        - name: content-votes
          tablePattern: '^content_votes$'
          requiredColumns:
            - { name: user_id, type: uuid, nullable: false, foreignKey: true, references: [users], onDelete: cascade }
            - { name: score, type: smallint, nullable: false }
          forbiddenColumns: [target_table]
        - name: report-export-cursors
          tablePattern: '^report_export_cursors$'
          primaryKeyTypes: [enum]
        - name: sync-cursors
          tablePattern: '^sync_cursors$'
          primaryKeyTypes: [uuid]
      bannedTablePatterns:
        - pattern: '_(history|change_logs|audit_logs)$'
          message: use a *_revisions table
```

## Why and when

Teams settle on one shape per concept: `_revisions` tables store `changes jsonb`
and reject updates, work queues share lease columns, and vote tables are one
table per target. Without a check, the next table is built a different way.

## What it catches/requires

The rule reads tables, columns, foreign keys, primary keys, triggers, and enum
names from the schema catalog. Every table and partitioned table is checked
against every shape whose `tablePattern` matches. A table may match several
shapes.

A named required column must exist. Each mismatched property is its own
finding: type (case-insensitive), nullability, a sole-column foreign key,
`ON DELETE`, and `references`. If several sole-column foreign keys exist, any
one that satisfies the foreign-key properties passes; otherwise the finding
uses the first key in name order. `references` compares unqualified table
names, ASCII case-insensitively. No foreign key produces only the not-a-foreign-key
finding.

A `namePattern` column passes when at least one column matches the pattern and
every given property. The same column may also satisfy a named requirement.
`foreignKey: false` rejects a sole-column foreign key; omitting it does not.
`foreignKey: false` with `references` or `onDelete` is a configuration error.
A missing match names `nullable: true` as `nullable`.
`forbiddenColumns` are exact names. `requiredTriggers` use
`CatalogTrigger::matches`, with the same defaults as
`postgres-column-requires-trigger` (`before`, `[update]`, `forEachRow: true`).
A `WHEN` clause or an `UPDATE OF` column list does not satisfy a table-wide
update trigger. `forEachRow: true` with `truncate` is a configuration error.

When `primaryKeyTypes` is non-empty, a missing primary key or an empty column
list is reported. Otherwise each key column, in key order, must have a listed
`data_type`. The word `enum` matches a catalog enum after dropping the schema
from both names. An empty list leaves keys unchecked.
`references` splits quoted identifiers before dropping the schema.
A blank entry in `references` is a configuration error.
A blank `type` is a configuration error.
`bannedTablePatterns` apply whether or not a shape matches.
`include`, `exclude`, and `projects` filter the catalog path.
A non-empty rule `message` replaces the finding text and keeps the catalog path and `table:<name>` locator.

Findings use `table:<name>`. Shape texts start with `(shape <name>)`.

## Options and defaults

`schemaCatalogPath` is required and names a catalog generated with [`no-mistakes postgres catalog`](../cli/postgres.md). `shapes` defaults to `[]`. Each shape `name`
is required and unique. `tablePattern` is a required regex. A required column
has exactly one of `name` or `namePattern`. `references: []` is an error.
`onDelete` is `cascade`, `restrict`, `set null`, `set default`, or `no action`.
`primaryKeyTypes` defaults to `[]`; an empty string in the list is an error.
`forbiddenColumns` and `requiredTriggers` default to `[]`.
`bannedTablePatterns` defaults to `[]`; an empty message or invalid regex is
an error. `allow` defaults to `[]`.

## Valid example

`order_revisions` has `id`, an `_id` foreign key, `revised_by_id` with
`ON DELETE SET NULL`, `changes jsonb NOT NULL`, and a
`BEFORE UPDATE OR DELETE` trigger executing `fn_reject_mutation()`. `orders`
matches no shape and no banned pattern. `article_votes` has `user_id uuid`
`NOT NULL` referencing `users` with `ON DELETE CASCADE`, plus `score smallint`
`NOT NULL`. `report_export_cursors` is keyed by the `report_types` enum.

```json
{
  "formatVersion": 2,
  "coverage": "complete",
  "tables": {
    "orders": {
      "columns": {
        "id": {
          "dataType": "uuid",
          "nullable": false
        }
      }
    },
    "order_revisions": {
      "columns": {
        "id": { "dataType": "uuid", "nullable": false },
        "order_id": { "dataType": "uuid", "nullable": false },
        "revised_by_id": { "dataType": "uuid", "nullable": true },
        "changes": { "dataType": "jsonb", "nullable": false }
      },
      "foreignKeys": {
        "order_revisions_order_id_fkey": {
          "columns": ["order_id"], "referencedTable": "orders",
          "referencedColumns": ["id"], "onDelete": "cascade"
        },
        "order_revisions_revised_by_id_fkey": {
          "columns": ["revised_by_id"], "referencedTable": "users",
          "referencedColumns": ["id"], "onDelete": "set null"
        }
      },
      "triggers": {
        "reject_revision_mutation": {
          "definition": "CREATE TRIGGER reject_revision_mutation BEFORE UPDATE OR DELETE ON order_revisions FOR EACH ROW EXECUTE FUNCTION fn_reject_mutation()"
        }
      }
    },
    "article_votes": {
      "columns": {
        "user_id": { "dataType": "uuid", "nullable": false },
        "score": { "dataType": "smallint", "nullable": false }
      },
      "foreignKeys": {
        "article_votes_user_id_fkey": {
          "columns": ["user_id"], "referencedTable": "users",
          "referencedColumns": ["id"], "onDelete": "cascade"
        }
      }
    },
    "report_export_cursors": {
      "columns": {
        "report_type": { "dataType": "report_types", "nullable": false }
      },
      "primaryKey": { "columns": ["report_type"] }
    }
  },
  "enums": { "report_types": { "values": ["daily", "monthly"] } }
}
```


## Counterexample

`bad_revisions.changes` is nullable even though it uses `jsonb`; the other
revision requirements are present.

```json
{
  "formatVersion": 2,
  "coverage": "complete",
  "tables": {
    "bad_revisions": {
      "columns": {
        "id": { "dataType": "uuid", "nullable": false },
        "order_id": { "dataType": "uuid", "nullable": false },
        "revised_by_id": { "dataType": "uuid", "nullable": true },
        "changes": { "dataType": "jsonb", "nullable": true }
      },
      "foreignKeys": {
        "bad_revisions_order_id_fkey": {
          "columns": ["order_id"], "referencedTable": "orders",
          "referencedColumns": ["id"], "onDelete": "cascade"
        },
        "bad_revisions_revised_by_id_fkey": {
          "columns": ["revised_by_id"], "referencedTable": "users",
          "referencedColumns": ["id"], "onDelete": "set null"
        }
      },
      "triggers": {
        "reject_revision_mutation": {
          "definition": "CREATE TRIGGER reject_revision_mutation BEFORE UPDATE OR DELETE ON bad_revisions FOR EACH ROW EXECUTE FUNCTION fn_reject_mutation()"
        }
      }
    }
  }
}
```

`legacy_revisions.changes` uses `json` instead of `jsonb`; its nullability,
foreign keys, and trigger meet the other requirements.

```json
{
  "formatVersion": 2,
  "coverage": "complete",
  "tables": {
    "legacy_revisions": {
      "columns": {
        "id": { "dataType": "uuid", "nullable": false },
        "order_id": { "dataType": "uuid", "nullable": false },
        "revised_by_id": { "dataType": "uuid", "nullable": true },
        "changes": { "dataType": "json", "nullable": false }
      },
      "foreignKeys": {
        "legacy_revisions_order_id_fkey": {
          "columns": ["order_id"], "referencedTable": "orders",
          "referencedColumns": ["id"], "onDelete": "cascade"
        },
        "legacy_revisions_revised_by_id_fkey": {
          "columns": ["revised_by_id"], "referencedTable": "users",
          "referencedColumns": ["id"], "onDelete": "set null"
        }
      },
      "triggers": {
        "reject_revision_mutation": {
          "definition": "CREATE TRIGGER reject_revision_mutation BEFORE UPDATE OR DELETE ON legacy_revisions FOR EACH ROW EXECUTE FUNCTION fn_reject_mutation()"
        }
      }
    }
  }
}
```

`unprotected_revisions` has the required columns and foreign keys but no
reject trigger.

```json
{
  "formatVersion": 2,
  "coverage": "complete",
  "tables": {
    "unprotected_revisions": {
      "columns": {
        "id": { "dataType": "uuid", "nullable": false },
        "order_id": { "dataType": "uuid", "nullable": false },
        "revised_by_id": { "dataType": "uuid", "nullable": true },
        "changes": { "dataType": "jsonb", "nullable": false }
      },
      "foreignKeys": {
        "unprotected_revisions_order_id_fkey": {
          "columns": ["order_id"], "referencedTable": "orders",
          "referencedColumns": ["id"], "onDelete": "cascade"
        },
        "unprotected_revisions_revised_by_id_fkey": {
          "columns": ["revised_by_id"], "referencedTable": "users",
          "referencedColumns": ["id"], "onDelete": "set null"
        }
      }
    }
  }
}
```

`comment_votes.user_id` is `ON DELETE SET NULL`.

```json
{
  "formatVersion": 2,
  "coverage": "complete",
  "tables": {
    "comment_votes": {
      "columns": {
        "user_id": { "dataType": "uuid", "nullable": false },
        "score": { "dataType": "smallint", "nullable": false }
      },
      "foreignKeys": {
        "comment_votes_user_id_fkey": {
          "columns": ["user_id"], "referencedTable": "users",
          "referencedColumns": ["id"], "onDelete": "set null"
        }
      }
    }
  }
}
```

`content_votes` has `target_table`.

```json
{
  "formatVersion": 2,
  "coverage": "complete",
  "tables": {
    "content_votes": {
      "columns": {
        "user_id": { "dataType": "uuid", "nullable": false },
        "score": { "dataType": "smallint", "nullable": false },
        "target_table": { "dataType": "text", "nullable": false }
      },
      "foreignKeys": {
        "content_votes_user_id_fkey": {
          "columns": ["user_id"], "referencedTable": "users",
          "referencedColumns": ["id"], "onDelete": "cascade"
        }
      }
    }
  }
}
```

`sync_cursors` is keyed by `cursor_name text` instead of a UUID.

```json
{
  "formatVersion": 2,
  "coverage": "complete",
  "tables": {
    "sync_cursors": {
      "columns": { "cursor_name": { "dataType": "text", "nullable": false } },
      "primaryKey": { "columns": ["cursor_name"] }
    }
  }
}
```

`order_history` matches the banned history pattern.

```json
{
  "formatVersion": 2,
  "coverage": "complete",
  "tables": { "order_history": { "columns": {} } }
}
```


## Fix

Add the missing column, foreign key, trigger, or primary-key type the shape
names, or rename the table onto the pattern that matches how it is actually
built. Drop forbidden columns such as `updated_at` on an append-only table.

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` (rule option, repository `effort`, or the invocation override) is not an error: the rule has no low implementation, so the check warns `requested effort low; effective effort max because this rule has no low implementation` and runs at `max`. It judges the generated schema catalog (`schemaCatalogPath`); executor calls in TS/JS are not evaluated.

### Unanalyzable inputs

Following recall over precision ([how analysis works](../../README.md#how-analysis-works)), this rule has no inputs it must guess at, so nothing is silently treated as clean. This rule has no unanalyzable inputs: it judges the configured schema catalog, a JSONC file read directly from `schemaCatalogPath` (no SQL is parsed by the rule), so every entry is either present or absent. A catalog that cannot be read, is not valid JSONC, is empty, or is not a valid snapshot is a hard error, not an unanalyzable input. The repository-level `unanalyzable` default does not affect it.

## Suppression

For an intentional exception, add an entry under the rule's `options.allow`:

```yaml
allow:
  - object: table:vendor_sync_history
    reason: Mirror of a vendor table; shape owned by the vendor
```

Snapshot JSON cannot carry `no-mistakes-disable` comments. Suppress every
finding for a table with `allow: [{object, reason}]`, where `object` is
`table:<name>` and `reason` is non-empty. An entry that matches nothing is a
stale allow entry.

## Related rules

`postgres-column-requires-trigger` matches one trigger to one column.
`postgres-required-comments` reads comments from the same catalog. This rule
checks the whole table shape: columns, keys, triggers, and banned names.
