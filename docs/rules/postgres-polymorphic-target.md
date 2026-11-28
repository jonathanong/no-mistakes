<!-- cspell:ignore nonnulls -->
# `postgres-polymorphic-target`

## Why and when

Flag tables that model a polymorphic relation as a `<x>_type` and `<x>_id`
column pair without a foreign key on the identifier column. PostgreSQL cannot
enforce that an untyped `<x>_id` points at an existing row in the table named
by `<x>_type`, so orphaned references and cross-type mismatches slip past the
schema.

This rule reads the configured schema catalog directly; it does not connect to
a database or parse application SQL. Stored function bodies are not inputs to
this catalog-only rule.

## Options

```yaml
rules:
  - rule: postgres-polymorphic-target
    scope: repository
    options:
      schemaCatalogPath: db/schema.json
      typeSuffix: _type
      idSuffix: _id
```

- `schemaCatalogPath` (required): repository-relative path to a complete schema
  catalog generated with [`no-mistakes postgres catalog`](../cli/postgres.md).
  Ordering-only catalogs are rejected. Project-targeted rules inherit
  `schemaCatalogPath` from [`projects.<name>.postgres`](../configuration/projects.md#postgresql-scope).
  Rule `include` and `exclude` select the catalog file.
- `typeSuffix` (optional, default `"_type"`): nonempty suffix that identifies
  the discriminator column `<x><typeSuffix>`.
- `idSuffix` (optional, default `"_id"`): nonempty suffix that identifies the
  paired identifier column `<x><idSuffix>`. Must differ from `typeSuffix`.
- `allow` (optional, default `[]`): list of `{ column, reason }` entries where
  `column` is an exact `table.column` naming either column of an intentional
  polymorphic pair and `reason` is nonempty.

The default severity is `error`. Set application-level `severity: advisory` to
adopt without failing `check`. Custom rule `message` is supported.

## Valid example

A pair whose identifier column participates in a foreign key (single-column or
composite), or a `_type` column with no matching `_id` column, is not reported:

```json
{
  "formatVersion": 2,
  "coverage": "complete",
  "tables": {
    "links": {
      "columns": {
        "id": { "dataType": "uuid", "ordinalPosition": 1 },
        "target_type": { "dataType": "text", "ordinalPosition": 2 },
        "target_id": { "dataType": "uuid", "ordinalPosition": 3 }
      },
      "foreignKeys": {
        "links_target_id_fkey": {
          "columns": ["target_id"],
          "referencedTable": "widgets",
          "referencedColumns": ["id"],
          "onDelete": "cascade",
          "onUpdate": "no action",
          "validated": true
        }
      }
    },
    "posts": {
      "columns": {
        "id": { "dataType": "uuid", "ordinalPosition": 1 },
        "article_type": { "dataType": "text", "ordinalPosition": 2 },
        "author_id": { "dataType": "uuid", "ordinalPosition": 3 }
      }
    }
  }
}
```

## What it requires

For each logical table in the catalog, whenever a column `<x><typeSuffix>`
(with nonempty prefix `<x>`) has a sibling column `<x><idSuffix>`, the
identifier column `<x><idSuffix>` must belong to a foreign key on that table
unless the pair is explicitly allowed or suppressed.

## Counterexample

`comments` stores `target_type` and `target_id` without a foreign key on
`target_id`:

```json
{
  "formatVersion": 2,
  "coverage": "complete",
  "tables": {
    "comments": {
      "columns": {
        "id": { "dataType": "uuid", "ordinalPosition": 1 },
        "target_type": { "dataType": "text", "ordinalPosition": 2 },
        "target_id": { "dataType": "uuid", "ordinalPosition": 3 }
      }
    }
  }
}
```

A foreign key only on `target_type` (for example referencing a lookup table of
type names) does not satisfy the rule because `target_id` still has no foreign
key. Findings are reported at `table:<name>` on the table's catalog line and
name both columns.

## Fix

Replace the polymorphic pair with one nullable foreign key column per target
table plus a `CHECK` constraint that requires exactly one of them to be set
(for example `CHECK (num_nonnulls(post_id, ticket_id) = 1)`), or split into a
separate table per target, then regenerate the schema catalog. There is no
automatic fix because migrating existing data requires choosing the target
tables and foreign-key actions.

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` is accepted: `check` emits an
analysis warning (`requested effort low; effective effort max because this rule
has no low implementation`) and runs the rule at `max`.

## Suppression

`allow` defaults to an empty list. For an intentional exception such as an
immutable audit log, put
`allow: [{ column: audit_entries.subject_id, reason: immutable audit history preserves deleted subjects }]`
in the rule's options. Use entries with an exact `table.column` (either the
`_id` or `_type` column) and a nonempty `reason`. Duplicate entries, malformed
`table.column` values, and empty reasons are configuration errors; unused
entries are reported as stale so exceptions stay current.

Catalog JSONC also supports `no-mistakes-disable-file`,
`no-mistakes-disable-line`, and `no-mistakes-disable-next-line` for this rule.
Line directives apply to the table's catalog source line. See
[suppressions](../configuration/suppressions.md).

## Related rules

- [`postgres-array-columns`](postgres-array-columns.md) flags array columns used to store relations.
- [`postgres-json-relation-ids`](postgres-json-relation-ids.md) flags relation ids stored inside JSON values.
- [`postgres-key-column-types`](postgres-key-column-types.md) controls primary-key and foreign-key column types.
