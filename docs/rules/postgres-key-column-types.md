# `postgres-key-column-types`

Checks primary-key and foreign-key referencing columns against an explicit list
of PostgreSQL types from a complete schema catalog. Stable identifiers keep
joins and references compatible when descriptive text changes. Foreign keys are
checked on the referencing side; the referenced table's own primary key is
checked by the primary-key rule. A referenced unique constraint does not get a
separate finding, and standalone unique constraints are not checked.

```yaml
rules:
  - rule: postgres-key-column-types
    scope: repository
    options:
      schemaCatalogPath: db/schema.json
      allowedTypes: [uuid, bigint, integer, smallint]
      allowEnumTypes: true
      checkPrimaryKeys: true
      checkForeignKeys: true
      allow: []
```

## Why and when

Use this rule when identifiers should use stable, compact types such as UUIDs or
integers. Natural text keys can remain unique alternate keys, while foreign keys
refer to the stable primary key. Enum keys are opt-in and useful when the enum
definition itself is the intended identifier domain.

## What it catches/requires

The rule emits one finding per failing primary-key or foreign-key constraint and
lists invalid columns in the constraint's declared order. Type names compare
case-insensitively to `allowedTypes`, using the catalog's exact `data_type`;
domains do not inherit the type of their base. `character varying(32)` is
displayed as `character varying` in messages, but type matching retains the
catalog value. With `allowEnumTypes: true`, catalog enum types are allowed.

Partitioned parent relations are checked. Generated catalogs omit ordinary
partition leaves that have only inherited constraint clones, and retain a leaf
when it has its own primary or foreign key. Nested partitioned parents are also
checked. A handwritten formatVersion 2 snapshot's explicit key facts on a leaf
are checked as that leaf's own constraints; omit inherited clones to avoid
duplicate findings. `partitionOf` records the immediate parent, including its
schema when the parent is outside the catalog's schema.

## Options and defaults

`schemaCatalogPath` is required and points to a catalog generated with
[`no-mistakes postgres catalog`](../cli/postgres.md). `allowedTypes` is a
required, nonempty list of distinct type names. `allowEnumTypes` defaults to
`false`. `checkPrimaryKeys` and `checkForeignKeys` both default to `true`, and
at least one must remain enabled. `allow` defaults to `[]` and accepts only
`constraint:<table>.<constraint>` entries with a reason. Stale allow entries are
reported. Catalogs accept JSONC comments, which support standard source
suppression directives. Use
`no-mistakes-disable-file postgres-key-column-types` at the start of the catalog
or `no-mistakes-disable-line postgres-key-column-types` on the finding's line;
the constraint-scoped `allow` option remains available for reasoned exceptions
tied to a specific constraint.

## Valid example

```json
{
  "formatVersion": 2,
  "coverage": "complete",
  "tables": {
    "orders": {
      "columns": {"id": {"dataType": "uuid"}},
      "primaryKey": {"columns": ["id"]},
      "indexes": {"orders_pkey": {"primary": true}}
    }
  }
}
```

## Counterexample

```json
{
  "formatVersion": 2,
  "coverage": "complete",
  "tables": {
    "orders": {
      "columns": {"id": {"dataType": "text"}},
      "primaryKey": {"columns": ["id"]},
      "indexes": {"orders_pkey": {"primary": true}}
    }
  }
}
```

With `allowedTypes: [uuid]`, this reports:

`schema.json: constraint:orders.orders_pkey: primary key uses text column id; use one of uuid, and keep any natural key as a separate unique constraint`

## Fix

Use an allowed stable key type, then keep a natural text identifier as a
separate unique constraint. For foreign keys, reference that stable key instead
of the natural text column.

## Suppression

Add an allow entry for the exact constraint identity and provide a reason. A
renamed or removed constraint makes the entry stale and reports it.

## Related rules

[`postgres-fk-index`](postgres-fk-index.md) checks whether foreign keys have
supporting indexes. [`postgres-array-columns`](postgres-array-columns.md) checks
array-valued columns.
