# `postgres-duplicate-function-body`

Functions whose bodies are the same once names and literals are normalised are
reported as one cluster. A single function, a body below `minTokens`, or the
same text in two languages produces nothing. `schemaCatalogPath` is required;
enabling the rule without it is a configuration error. With only that path
set, the defaults report every cluster of at least two functions.

```yaml
rules:
  - rule: postgres-duplicate-function-body
    scope: repository
    options:
      schemaCatalogPath: db/schema.json
      minClusterSize: 2
      minTokens: 1
      normalizeIdentifiers: true
      normalizeRaise: true
      keepIdentifiers: ['now', 'coalesce']
      allow:
        - object: function:fn_orders_audit
          reason: Kept separate on purpose; audited by a different team
```

## Why and when

Schemas grow near-identical functions that differ only by a table name, a
column name, or the error text. Each copy has to be fixed separately, and the
copies drift. One function that uses `TG_TABLE_NAME` or `TG_ARGV` replaces
them.

## What it catches/requires

The rule reads function bodies from the schema catalog. A function with no
body, or a body that fails to tokenize, is skipped. Whitespace and comments
are dropped, and a trailing `;` is dropped so `END;` and `END` match.

When `normalizeRaise` is true, each `RAISE` statement becomes
`RAISE <level> ? ;`. The level is `EXCEPTION`, `WARNING`, `NOTICE`, `INFO`,
`LOG`, or `DEBUG`, and defaults to `EXCEPTION`. The message, format arguments,
and `USING` options are dropped. A `;` inside a string does not end the
statement.

String literals, including dollar quotes, become `'?'`. Numbers become `0`.
When `normalizeIdentifiers` is true, non-keyword words and quoted identifiers
become `ID`, except a word immediately followed by `(` which is kept. SQL
keywords, `NEW`, `OLD`, words starting with `TG_`, `keepIdentifiers`, and
operators stay, uppercased where they are words.

The group key is the language plus that token sequence. Every member of a
group of size at least `minClusterSize`, with at least `minTokens` tokens, is
reported as `function:<snapshot key>`. The text lists the other keys in
snapshot-key order, at most five, then `and <m> more`.

## Options and defaults

`schemaCatalogPath` is required. `minClusterSize` defaults to `2` and must be
at least 2. `minTokens` defaults to `1` and must be at least 1.
`normalizeIdentifiers` and `normalizeRaise` default to `true`.
`keepIdentifiers` and `allow` default to `[]`. An empty allow reason, an
invalid object ref, or a duplicate allow object fails configuration.

## Valid example

One `fn_reject_mutation()` that raises `'% is immutable', TG_TABLE_NAME`.
`PERFORM fn_touch_orders()` and `PERFORM fn_touch_invoices()` stay distinct
because the called name is kept. The same body in `sql` and `plpgsql` does not
match. `minTokens: 12` ignores the six-token always-raise copies.

## Counterexample

`fn_reject_orders_update`, `fn_reject_invoices_update`, and
`fn_reject_refunds_update` all normalise to `BEGIN RAISE EXCEPTION ? ; END`,
including a copy that uses `USING ERRCODE`, a format argument, or `END;`.
`NEW.title IS DISTINCT FROM OLD.title` matches `NEW.body IS DISTINCT FROM
OLD.body` while `normalizeIdentifiers` is true.

## Fix

Replace the copies with one function parameterised by `TG_TABLE_NAME` or
`TG_ARGV`. Raise the `minTokens` floor when the shared shape is too small to
be worth merging. Set `normalizeIdentifiers` or `normalizeRaise` to false when
those differences are intentional.

## Suppression

Snapshot JSON cannot carry `no-mistakes-disable` comments. Suppress one
function with `allow: [{object, reason}]`, where `object` is
`function:<snapshot key>` and `reason` is non-empty. An entry that matches
nothing is reported as a stale allow entry.

## Related rules

`postgres-column-requires-trigger` reads trigger definitions from the same
schema catalog. This rule reads function bodies and does not compare them to
the triggers that call them.
