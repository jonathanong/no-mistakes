# `postgres-array-columns`

Flags a column whose `data_type` is an array. Array columns have no per-element
foreign key, index, or cascade. A reviewed `allow` entry keeps the few arrays
an external protocol defines. `uuid[]` is never allowlisted: each reference
belongs in a child row.

The rule reads column types and enum names from the shared schema catalog. It
does not parse migration SQL. With only `schemaCatalogPath` set, every array
column is reported, and `uuid` is the element type an allow entry cannot
excuse. An empty options object is a configuration error because
`schemaCatalogPath` has no default.

```yaml
rules:
  - rule: postgres-array-columns
    scope: repository
    options:
      schemaCatalogPath: db/schema.json
      neverAllowElementTypes: ['uuid']
      allowElementTypes: []
      allowEnumElements: false
      allow:
        - object: 'column:oauth_clients.redirect_uris'
          reason: 'OAuth 2.0 client metadata (RFC 7591) defines this as a list'
```

## Why and when

Use this rule when a snapshot should not grow `text[]` tag lists or `uuid[]`
reference lists. A child table or an enum keeps the values queryable. OAuth
scopes and similar protocol fields stay, with a reason on the allow entry.

## What it catches/requires

A column is an array when `data_type` ends with `[]`. The element type is that
string with every trailing `[]` removed, so `integer[][]` has element type
`integer` and `character varying[]` keeps the space. Comparison with the
configured lists and with catalog enum names is case-insensitive.

The column is skipped when the element type is in `allowElementTypes`, or when
`allowEnumElements` is set and the element type names a catalog enum.
An unqualified element type matches a schema-qualified enum. A qualified
element type matches only that full name, so `audit.report_types` is not the
enum `public.report_types`.
Otherwise it is reported. An element type in `neverAllowElementTypes` uses the
never-allowed text. An `allow` entry cannot suppress that finding. The entry
is reported as `allow entry <object> cannot excuse a <type>[] column` and is
not also reported stale.

## Options and defaults

`schemaCatalogPath` is required and names a catalog generated with [`no-mistakes postgres catalog`](../cli/postgres.md). `neverAllowElementTypes` defaults to `["uuid"]`. `allowElementTypes` defaults to `[]`. A type that appears in both lists is a configuration error. `allowEnumElements` defaults to `false`. `allow` defaults to `[]`. An empty reason, a duplicate object, or an object ref that is not `column:<table>.<column>` is a configuration error.

## Valid example

```json
{
  "formatVersion": 2,
  "coverage": "complete",
  "tables": {
    "oauth_clients": {
      "columns": { "redirect_uris": { "dataType": "text[]" } }
    }
  }
}
```

With the allow entry in the config above, `redirect_uris` is not a finding.
`weekday[]` is not a finding when `allowEnumElements` is true and `weekday` is
a catalog enum. `smallint[]` is not a finding when `allowElementTypes`
includes `smallint`.

## Counterexample

```json
{
  "formatVersion": 2,
  "coverage": "complete",
  "tables": {
    "orders": {
      "columns": {
        "tag_names": { "dataType": "text[]" },
        "coupon_ids": { "dataType": "uuid[]" }
      }
    }
  }
}
```

`tag_names` reports:

`column is an array (text[]); store the values as child rows or an enum, or add an allow entry with a reason`

`coupon_ids` reports:

`column is a uuid[] array; model each reference as a child row with a foreign key (uuid arrays cannot be allowlisted)`

An allow entry for `column:orders.coupon_ids` does not remove that finding. It
adds `allow entry column:orders.coupon_ids cannot excuse a uuid[] column`.

## Fix

Store the values as child rows, or as an enum when the set is closed. For a
protocol list that has to stay an array, add an `allow` entry with a reason.
Do not allowlist a `uuid[]` column; give each reference its own row and
foreign key.

## Suppression

Snapshot findings are line 1, so SQL comments cannot suppress them. Add
`allow: [{object, reason}]` with object ref `column:<table>.<column>`. An
entry that matches no finding is reported as
`stale postgres-array-columns allow entry: <object>`. An entry that names a
never-allowed array is reported with the "cannot excuse" text instead of a
stale finding, and the column finding stays.

## Related rules

`postgres-finite-text-columns` catches a text column that a `CHECK` has
already pinned to a fixed set. `postgres-column-naming` checks the names of
the foreign-key columns that replace a `uuid[]`.
