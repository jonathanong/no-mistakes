# `postgres-finite-text-columns`

Flags a text column whose `CHECK` constraint pins it to string literals, and
optionally a text column whose name marks a fixed set of values. The value
set belongs in an enum type or a foreign key to a lookup table. PostgreSQL
stores `IN (...)` as `= ANY (ARRAY[...])`, and the schema snapshot keeps that
form.

The rule reads columns, foreign keys, and check definitions from the shared
schema catalog. It does not parse migration SQL. With only `schemaCatalogPath`
set, it reports pinned `text` and `character varying` columns. Name-only
findings stay off until `namePatterns` is set. An empty options object is a
configuration error because `schemaCatalogPath` has no default.

```yaml
rules:
  - rule: postgres-finite-text-columns
    scope: repository
    options:
      schemaCatalogPath: db/schema.json
      columnTypes: ['text', 'character varying']
      namePatterns: ['(^|_)(status|state|kind|type|source|category|mode|channel)$']
      skipGeneratedColumns: false
      ignoreTablePatterns: []
      allow:
        - object: 'column:webhook_deliveries.event_type'
          reason: 'Values are defined by the webhook provider and change without a migration'
```

## Why and when

A `status text` column with `CHECK (status IN ('draft', 'sent', 'paid'))`
hides the value set from type generators, and the same list gets copied onto
every table that needs it. A `status text` column with no constraint accepts
any string. Use this rule when the snapshot should say which text columns are
already a closed set, or which names usually are.

## What it catches/requires

A candidate is a column of a table or partitioned table, of a configured
`columnTypes` value, that is not the only column of a foreign key. A
composite foreign key does not remove the column. `skipGeneratedColumns`
skips columns whose `generated` field is set. Tables matching
`ignoreTablePatterns` are not checked.

A check pins a column when every `OR` branch pins it, or when an `AND` has at
least one pinning conjunct. `c = 'x'`, `'x' = c`, `c IN (...)`, and
`c = ANY (ARRAY[...])` pin string literals. `c IS NULL` pins an empty list so
a nullable `OR` can still succeed. Casts on the column or the literal are
ignored. Anything else, including a check that fails to parse, does not pin
and is not an error.

Each pinned candidate gets one finding. The text lists values in first-seen
order, walking constraints by name, and names up to five other pinned columns
with the same set. A candidate that is not pinned can still match
`namePatterns`. A column that is both pinned and name-matched gets only the
literal finding.

## Options and defaults

`schemaCatalogPath` is required. `columnTypes` defaults to `["text", "character varying"]`; an empty list is a configuration error. Comparison with `data_type` is case-insensitive. `namePatterns` defaults to `[]`, which turns the name check off. `skipGeneratedColumns` defaults to `false`. `ignoreTablePatterns` defaults to `[]`. An invalid regex is a configuration error. `allow` defaults to `[]`. An empty reason, a duplicate object, or an object ref that is not `column:<table>.<column>` is a configuration error.

## Valid example

```json
{
  "formatVersion": 2,
  "tables": {
    "invoices": {
      "columns": { "status": { "dataType": "invoice_statuses" } }
    },
    "accounts": {
      "columns": { "display_name": { "dataType": "text" } }
    }
  }
}
```

An enum column is not text. `display_name` does not match the name pattern
above. A column that is the only column of a foreign key is a lookup
reference already, even when a check also lists its values.

## Counterexample

```json
{
  "formatVersion": 2,
  "tables": {
    "invoices": {
      "columns": { "status": { "dataType": "text" } },
      "checkConstraints": {
        "ck": {
          "definition": "CHECK ((status = ANY (ARRAY['draft'::text, 'sent'::text, 'paid'::text])))"
        }
      }
    }
  }
}
```

The finding text is:

`text column holds a fixed set of values ('draft', 'sent', 'paid') enforced by CHECK; use an enum type or a foreign key to a lookup table`

A `plan_type text` column with no check, when `namePatterns` includes that
pattern, reports:

`text column name matches (^|_)(status|state|kind|type|source|category|mode|channel)$, which marks a fixed set of values; use an enum type or a foreign key to a lookup table, or add an allow entry with a reason`

## Fix

Replace the text column with an enum type, or with a foreign key to a lookup
table that holds the values. If the values are owned by an external system,
add an `allow` entry with a reason instead of changing the column.

## Suppression

Snapshot findings are line 1, so SQL comments cannot suppress them. Add
`allow: [{object, reason}]` with object ref `column:<table>.<column>`. An
entry that matches no finding is reported as
`stale postgres-finite-text-columns allow entry: <object>`.

## Related rules

`postgres-object-naming` checks the name of an enum type once you add one.
`postgres-column-naming` checks foreign-key column names on the lookup table
that replaces the text column.
