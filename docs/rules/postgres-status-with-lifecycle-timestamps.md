# `postgres-status-with-lifecycle-timestamps`

A `projects: [<name>]` entry inherits [`projects.<name>.postgres`](../configuration/projects.md#postgresql-scope) when that block is set. Do not restate `importSpecifier`, `executorNames`, `executorFactoryNames`, `executorTypeNames`, `trustedSqlTags`, `schemaCatalogPath`, or `unanalyzableSql` on the rule. `include`, `exclude`, and `sqlInclude` may only narrow the project scope.

A stored status column next to lifecycle timestamps records the same fact
twice. `schemaCatalogPath` and `statusColumns` are required. With those set
and `lifecycleVerbs` left empty, the rule reports nothing, including no stale
allow entries.

```yaml
rules:
  - rule: postgres-status-with-lifecycle-timestamps
    scope: repository
    options:
      schemaCatalogPath: db/schema.json
      statusColumns: [status, state]
      lifecycleVerbs: [sent, failed, paid]
      minLifecycleColumns: 2
```

## Why and when

`status = 'sent'` with `sent_at IS NULL` means two writers disagreed. A
generated column, or an append-only history table when rows return to earlier
states, keeps one source of truth.

## What it catches/requires

The rule reads table columns from the schema catalog, including `generated`
and `data_type`. A stored status column is a column named in `statusColumns`
whose `generated` is not set. An enum is reported the same way as `text`.

A lifecycle column is not generated, stores one timestamp, and has a name
that is `<verb>_at` or ends with `_<verb>_at` for a verb in `lifecycleVerbs`.
The type is `timestamp` or `timestamptz`, optionally with a precision and
`with time zone` or `without time zone`. A name such as `timestamp_status`
does not count. Array types such as `timestamp[]` do not count, and neither does
a generated projection of `status`. `payment_failed_at` counts for `failed`.
`resent_at` does not count for `sent`. A `date` column does not count.
`created_at` and `updated_at` count only when those verbs are configured.
An array-valued status column, such as `text[]`, is not a stored status.

A table matches when it has a stored status column and at least
`minLifecycleColumns` lifecycle columns. The finding is `table:<name>` and
names the first stored status column in ordinal order, then the lifecycle
columns in ordinal order. A generated `status` beside a stored `state` names
`state`. A generated status alone passes.

## Options and defaults

`schemaCatalogPath` is required and names a catalog generated with [`no-mistakes postgres catalog`](../cli/postgres.md). `statusColumns` is required and has no
default; an empty list, an empty string, or a duplicate name is an error.
`lifecycleVerbs` defaults to `[]`, which reports nothing. An empty string or
a duplicate verb is an error. `minLifecycleColumns` defaults to `2`; a value
below `1` is an error. `allow` defaults to `[]`.
`include`, `exclude`, and `projects` filter the catalog path.
A non-empty rule `message` replaces the finding text and keeps the
`table:<name>` locator.

## Valid example

`invoices.status` is `GENERATED ALWAYS AS (...) STORED` beside `sent_at` and
`paid_at`.

```json
{
  "formatVersion": 2,
  "coverage": "complete",
  "tables": {
    "invoices": {
      "columns": {
        "status": {
          "dataType": "text",
          "generated": "stored"
        },
        "sent_at": {
          "dataType": "timestamp with time zone"
        },
        "paid_at": {
          "dataType": "timestamp with time zone"
        }
      }
    }
  }
}
```

This `invoices` row has a stored status and only one configured lifecycle
timestamp, below `minLifecycleColumns`.

```json
{
  "formatVersion": 2,
  "coverage": "complete",
  "tables": {
    "invoices": {
      "columns": {
        "status": { "dataType": "text" },
        "sent_at": { "dataType": "timestamp with time zone" }
      }
    }
  }
}
```

`receipts` has `sent_at` and `paid_at`, but no status column.

```json
{
  "formatVersion": 2,
  "coverage": "complete",
  "tables": {
    "receipts": {
      "columns": {
        "sent_at": { "dataType": "timestamp with time zone" },
        "paid_at": { "dataType": "timestamp with time zone" }
      }
    }
  }
}
```

`orders.created_at` and `orders.updated_at` are not configured lifecycle
verbs, so they do not match this rule.

```json
{
  "formatVersion": 2,
  "coverage": "complete",
  "tables": {
    "orders": {
      "columns": {
        "status": { "dataType": "text" },
        "created_at": { "dataType": "timestamp with time zone" },
        "updated_at": { "dataType": "timestamp with time zone" }
      }
    }
  }
}
```


## Counterexample

`deliveries` stores `state text` next to `sent_at` and `failed_at`.

```json
{
  "formatVersion": 2,
  "coverage": "complete",
  "tables": {
    "deliveries": {
      "columns": {
        "state": {
          "dataType": "text"
        },
        "sent_at": {
          "dataType": "timestamp with time zone"
        },
        "failed_at": {
          "dataType": "timestamp with time zone"
        }
      }
    }
  }
}
```

`payments` stores `status payment_statuses` next to `paid_at` and
`payment_failed_at`.

```json
{
  "formatVersion": 2,
  "coverage": "complete",
  "tables": {
    "payments": {
      "columns": {
        "status": { "dataType": "payment_statuses" },
        "paid_at": { "dataType": "timestamp with time zone" },
        "payment_failed_at": { "dataType": "timestamp with time zone" }
      }
    }
  },
  "enums": {
    "payment_statuses": { "values": ["pending", "paid", "failed"] }
  }
}
```


## Fix

If every row reaches a final state, make status
`GENERATED ALWAYS AS (...) STORED` from the timestamps. If rows return to
earlier states, record each attempt or change in a history table and read
the current state from the latest row.

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` (rule option, repository `effort`, or the invocation override) is not an error: the rule has no low implementation, so the check warns `requested effort low; effective effort max because this rule has no low implementation` and runs at `max`. It judges the generated schema catalog (`schemaCatalogPath`); executor calls in TS/JS are not evaluated.

### Unanalyzable inputs

Following recall over precision ([how analysis works](../../README.md#how-analysis-works)), this rule has no inputs it must guess at, so nothing is silently treated as clean. This rule has no unanalyzable inputs: it judges the configured schema catalog, a JSONC file read directly from `schemaCatalogPath` (no SQL is parsed by the rule), so every entry is either present or absent. A catalog that cannot be read, is not valid JSONC, is empty, or is not a valid snapshot is a hard error, not an unanalyzable input. The repository-level `unanalyzable` default does not affect it.

## Suppression

For an intentional exception, add an entry under the rule's `options.allow`:

```yaml
allow:
  - object: table:payment_intents
    reason: status mirrors the payment provider's state machine
```

Snapshot JSON cannot carry `no-mistakes-disable` comments. Suppress every
finding for a table with `allow: [{object, reason}]`, where `object` is
`table:<name>` and `reason` is non-empty. An entry that matches nothing is a
stale allow entry.

## Related rules

`postgres-table-shape` checks a named table concept's columns and keys.
This rule looks only for a stored status column sitting beside lifecycle
timestamps in the same schema catalog.
