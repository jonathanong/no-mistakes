# `postgres-column-check-constraints`

## Why and when

Require a validated CHECK proof for configured integer columns. This keeps values
that represent non-negative quantities from becoming negative through another
writer or a migration. The rule reads existing catalog `checkConstraints` facts;
it does not connect to a database or infer column conventions.

## Options

```yaml
rules:
  - rule: postgres-column-check-constraints
    scope: repository
    severity: advisory
    options:
      schemaCatalogPath: db/schema.json
      columns:
        - pattern: '*_count'
          types: [smallint, integer, bigint]
          require: non-negative
```

`schemaCatalogPath` is optional: without a configured catalog the rule reports
nothing. A configured unreadable or incomplete catalog retains the standard
catalog error. Rule `include` and `exclude` select the catalog file.
Project-targeted rules inherit
`schemaCatalogPath` from the project PostgreSQL scope.

`columns` defaults to an empty list. Each entry requires a nonempty glob `pattern`
matched against the column name, a nonempty `types` list, and a `require`
predicate. The supported predicate is `non-negative`; supported types are
`smallint`, `integer`, and `bigint` (including their `int2`, `int`/`int4`, and `int8`
aliases). Matching is case sensitive for column names. Types are normalized to
integer families. Other columns remain outside the configured scope.

The default severity is `error`, as for other rules. The recommended adoption
configuration above uses application-level `severity: advisory` to report
non-failing advisories. Omit that setting or use `severity: error` to enforce
failures. Custom rule `message` is supported.

## Valid example

The schema contains `CHECK (item_count >= 0)`. Its catalog records the proof:

```json
{
  "formatVersion": 2,
  "coverage": "complete",
  "tables": {
    "metrics": {
      "columns": { "item_count": { "dataType": "integer" } },
      "checkConstraints": {
        "metrics_count_non_negative": {
          "definition": "CHECK (item_count >= 0)",
          "validated": true
        }
      }
    }
  }
}
```

The proof can be `column >= 0`, `0 <= column`, or `column > -1`.
Stronger literal integer bounds, reversed comparisons, and equality to a
non-negative integer also prove the requirement.
Parentheses, integer casts on constant bounds (such as the catalog spelling
`'-1'::integer`), and conjunctions containing a proof are accepted. Only validated
constraints count; `NOT VALID` does not establish the invariant for existing rows.
Non-enforced constraints do not count.
Nullable columns remain nullable: this rule proves a lower bound for non-null
values and does not require `NOT NULL`.

## What it requires

Each selected integer column needs a validated non-negative CHECK proof.

## Counterexample

The schema's `CHECK (item_count >= 0 OR archived)` permits negative counts
when `archived` is true. The catalog preserves that conditional check:

```json
{
  "formatVersion": 2,
  "coverage": "complete",
  "tables": {
    "metrics": {
      "columns": {
        "item_count": { "dataType": "integer" },
        "archived": { "dataType": "boolean" }
      },
      "checkConstraints": {
        "metrics_conditional_count": {
          "definition": "CHECK (item_count >= 0 OR archived)",
          "validated": true
        }
      }
    }
  }
}
```

An `OR`, negation, transformed column, or check on another column cannot prove
the required bound. Unsupported expressions conservatively leave the proof
missing. Findings identify `table.column` and the required `column >= 0` proof.

## Fix

Add `CHECK (item_count >= 0)` to the table, correct existing negative values,
and validate the constraint. Regenerate the schema catalog. There is no automatic
fix because the rule cannot decide how to repair stored data.

## Suppression

`allow` defaults to an empty list. For an intentional signed column, put
`allow: [{ column: legacy_stats.delta_count, reason: signed delta by design }]`
in the rule's options. Use entries with an exact `table.column` and a nonempty
`reason` for
intentional signed values. Duplicate entries and empty reasons are configuration
errors; unused entries are reported as stale so exceptions stay current.

Catalog JSONC supports `no-mistakes-disable-file`, `no-mistakes-disable-line`,
and `no-mistakes-disable-next-line` for this rule. Line directives apply to the
column's catalog source line. See [suppressions](../configuration/suppressions.md).

## Related rules

- [`postgres-key-column-types`](postgres-key-column-types.md) controls key types.
- [`postgres-finite-text-columns`](postgres-finite-text-columns.md) detects finite text domains.
