# `postgres-required-comments`

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
      exemptColumnNamePatterns: ["^(id|created_at|updated_at)$", "^vendor_"]
      minLength: 10
      allow:
        - object: table:schema_migrations
          reason: Owned by the migration runner
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

## Counterexample

`orders` has no comment, a comment made only of whitespace, or a comment
shorter than `minLength`. The same applies to a selected column, view, or
materialized view.

## Fix

Add `COMMENT ON TABLE`, `COMMENT ON COLUMN`, `COMMENT ON VIEW`, or
`COMMENT ON MATERIALIZED VIEW` with a trimmed length of at least `minLength`.
Exempt mechanical column names with `exemptColumnNamePatterns` instead of
writing filler comments.

## Suppression

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
