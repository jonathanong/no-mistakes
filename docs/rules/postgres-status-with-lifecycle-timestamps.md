# `postgres-status-with-lifecycle-timestamps`

A stored status column next to lifecycle timestamps records the same fact
twice. `schemaCatalogPath` and `statusColumns` are required. With those set
and `lifecycleVerbs` left empty, the rule reports nothing.

```yaml
rules:
  - rule: postgres-status-with-lifecycle-timestamps
    scope: repository
    options:
      schemaCatalogPath: db/schema.json
      statusColumns: [status, state]
      lifecycleVerbs: [sent, failed, paid]
      minLifecycleColumns: 2
      allow:
        - object: table:payment_intents
          reason: status mirrors the payment provider's state machine
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

`schemaCatalogPath` is required. `statusColumns` is required and has no
default; an empty list, an empty string, or a duplicate name is an error.
`lifecycleVerbs` defaults to `[]`, which reports nothing. An empty string or
a duplicate verb is an error. `minLifecycleColumns` defaults to `2`; a value
below `1` is an error. `allow` defaults to `[]`.
`include`, `exclude`, and `projects` filter the catalog path.
A non-empty rule `message` replaces the finding text and keeps the
`table:<name>` locator.

## Valid example

`invoices.status` is `GENERATED ALWAYS AS (...) STORED` beside `sent_at` and
`paid_at`. `invoices` with `status` and only `sent_at` is below the minimum.
`receipts` has `sent_at` and `paid_at` and no status column. `orders` has
`status` plus `created_at` and `updated_at`, and those verbs are not listed.

## Counterexample

`deliveries` stores `state text` next to `sent_at` and `failed_at`.
`payments` stores `status payment_statuses` next to `paid_at` and
`payment_failed_at`.

## Fix

If every row reaches a final state, make status
`GENERATED ALWAYS AS (...) STORED` from the timestamps. If rows return to
earlier states, record each attempt or change in a history table and read
the current state from the latest row.

## Suppression

Snapshot JSON cannot carry `no-mistakes-disable` comments. Suppress every
finding for a table with `allow: [{object, reason}]`, where `object` is
`table:<name>` and `reason` is non-empty. An entry that matches nothing is a
stale allow entry.

## Related rules

`postgres-table-shape` checks a named table concept's columns and keys.
This rule looks only for a stored status column sitting beside lifecycle
timestamps in the same schema catalog.
