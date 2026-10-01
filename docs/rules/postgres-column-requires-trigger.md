# `postgres-column-requires-trigger`

A column that must be maintained by a trigger is reported when that trigger is
missing, fires only for `UPDATE OF` a column list, or is left behind after the
column is dropped. With an empty `requirements` list the rule reports nothing.
`schemaCatalogPath` is required; enabling the rule without it is a
configuration error.

```yaml
rules:
  - rule: postgres-column-requires-trigger
    scope: repository
    options:
      schemaCatalogPath: db/schema.json
      requirements:
        - column: updated_at
          function: fn_touch_updated_at
          timing: before
          events: [update]
          forEachRow: true
          allowColumnList: false
      allow:
        - object: table:import_rows
          reason: updated_at is written explicitly by the importer
```

## Why and when

Use this when a column such as `updated_at` is only correct if a trigger writes
it. A new table can add the column and forget the trigger, an immutable table
can keep a meaningless column, or a trigger can survive after the column is
dropped and fail at runtime.

## What it catches/requires

For every table or partitioned table, each requirement is checked against the
schema catalog:

- If the table has `column`, at least one trigger must match `function`
  (unqualified), `timing`, `events` (a subset), and `forEachRow`.
- When `allowColumnList` is false, `UPDATE OF a, b` does not count as covering
  `update`. If that trigger is the only candidate, the finding names it.
- If a trigger executes `function` and the table has no `column`, that leftover
  trigger is reported too.
- `BEFORE INSERT OR UPDATE` satisfies `events: [update]`. A schema qualifier on
  the function is ignored. Periods inside a quoted identifier stay part of the
  name. The function body is not checked.
- A trigger with a `WHEN` clause does not count as coverage. `TRUNCATE` is only
  valid with `forEachRow: false`.

Findings use the object ref `table:<name>`.

## Options and defaults

`schemaCatalogPath` is required. `requirements` defaults to `[]`, and with that
default the rule reports nothing. Each requirement's `column` and `function`
are required and non-empty. `timing` defaults to `before` (`before`, `after`,
or `instead-of`). `events` defaults to `[update]` and must be a non-empty list
of `insert`, `update`, `delete`, or `truncate`. `forEachRow` defaults to
`true`. `allowColumnList` defaults to `false`. `allow` defaults to `[]`.

An unknown timing or event, an empty required string, a missing
`schemaCatalogPath`, `truncate` combined with `forEachRow: true`, an allow
entry with an empty reason, an invalid object ref, or a duplicate allow object
fails configuration.

## Valid example

`orders` has `updated_at` and
`CREATE TRIGGER trigger_orders_touch BEFORE UPDATE ON public.orders FOR EACH ROW EXECUTE FUNCTION fn_touch_updated_at()`.
`order_events` has neither the column nor the trigger.

## Counterexample

`orders` has `updated_at` and no `BEFORE UPDATE FOR EACH ROW` trigger executing
`fn_touch_updated_at()`. A trigger that only runs `BEFORE UPDATE OF status` is
also reported when it is the only candidate.

## Fix

Add the trigger the requirement names, or drop the column when the table should
not maintain it. Drop a leftover trigger after dropping the column. Set
`allowColumnList` only when an `UPDATE OF` list is the intended contract.

## Suppression

Snapshot JSON cannot carry `no-mistakes-disable` comments. Suppress a table
with `allow: [{object, reason}]`, where `object` is `table:<name>` and `reason`
is non-empty. One entry covers every finding for that table. An entry that
matches nothing is reported as a stale allow entry.

## Related rules

`postgres-conflict-ordering` and `postgres-lock-ordering` read the same schema
catalog for indexes. This rule reads columns and trigger definitions from that
catalog instead.
