# `postgres-table-shape`

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
      bannedTablePatterns:
        - pattern: '_(history|change_logs|audit_logs)$'
          message: use a *_revisions table
      allow:
        - object: table:vendor_sync_history
          reason: Mirror of a vendor table; shape owned by the vendor
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
`forbiddenColumns` are exact names. `requiredTriggers` use
`CatalogTrigger::matches`, with the same defaults as
`postgres-column-requires-trigger` (`before`, `[update]`, `forEachRow: true`).

When `primaryKeyTypes` is non-empty, a missing primary key is reported.
Otherwise each key column, in key order, must have a listed `data_type`. The
word `enum` matches any catalog enum. An empty list leaves keys unchecked.
`bannedTablePatterns` apply whether or not a shape matches.

Findings use `table:<name>`. Shape texts start with `(shape <name>)`.

## Options and defaults

`schemaCatalogPath` is required. `shapes` defaults to `[]`. Each shape `name`
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
matches no shape and no banned pattern. `article_votes` has
`user_id uuid NOT NULL` referencing `users` `ON DELETE CASCADE` and
`score smallint NOT NULL`. `report_export_cursors` is keyed by the
`report_types` enum.

## Counterexample

`bad_revisions.changes` is nullable `json` with no reject trigger.
`comment_votes.user_id` is `ON DELETE SET NULL`. `content_votes` has
`target_table`. `sync_cursors` is keyed by `cursor_name text`.
`order_history` matches the banned history pattern.

## Fix

Add the missing column, foreign key, trigger, or primary-key type the shape
names, or rename the table onto the pattern that matches how it is actually
built. Drop forbidden columns such as `updated_at` on an append-only table.

## Suppression

Snapshot JSON cannot carry `no-mistakes-disable` comments. Suppress every
finding for a table with `allow: [{object, reason}]`, where `object` is
`table:<name>` and `reason` is non-empty. An entry that matches nothing is a
stale allow entry.

## Related rules

`postgres-column-requires-trigger` matches one trigger to one column.
`postgres-required-comments` reads comments from the same catalog. This rule
checks the whole table shape: columns, keys, triggers, and banned names.
