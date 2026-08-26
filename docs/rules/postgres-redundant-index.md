# `postgres-redundant-index`

Flags a live btree index whose key columns are a strict prefix of another
btree index on the same table with the same `WHERE` predicate. The shorter
index is extra write cost with no extra lookup power.

```yaml
rules:
  - rule: postgres-redundant-index
    scope: repository
    options:
      sqlInclude: ["**/*.sql"]
      allowDirective: redundant-index-allow
      allowedIndexes: []
```

`sqlInclude` defaults to `**/*.sql`. Indexes are considered across every
included SQL file. Table and index names keep schema qualifiers, so
`public.events` and `audit.events` are distinct, and `DROP INDEX audit.idx`
does not remove `public.idx`. A later `DROP INDEX` or `DROP TABLE` removes
earlier indexes of the matching name (or every index on the dropped table).
File order uses the first numeric run in the filename (`2.sql` before
`10.sql`, `V2__` before `V10__`); names without digits stay lexicographic.
Unique shorter indexes are never redundant: the longer index enforces
uniqueness on its full key, not the prefix. `INCLUDE` columns on the shorter
index must already be present on the longer index (as keys or includes).
Omitted btree `ASC` matches `ASC NULLS LAST`; omitted `DESC` matches
`DESC NULLS FIRST`. Partial-index predicates compare after lowercasing
keywords and unquoted identifiers, leaving string literals and quoted
identifiers unchanged.

Skip unnamed/implicit indexes; they cannot be dropped with `DROP INDEX`.
Same-line `-- allowDirective:` comments (default `redundant-index-allow`) skip
that create (the comment must sit on the `CREATE INDEX` keyword line, which
may be above a wrapped name). `allowedIndexes` entries are `table.index`,
including any schema qualifier. Stale allowlist entries are findings.

Counterexample: `events.idx_events__topic_id` is a prefix of
`idx_events__topic_id__created_at`.

```sql
CREATE INDEX idx_events__topic_id ON events (topic_id);
CREATE INDEX idx_events__topic_id__created_at ON events (topic_id, created_at);
```

Fix: drop the prefix index, put `-- redundant-index-allow: <reason>` on its
line, or add `table.index` to `allowedIndexes`.

```sql
DROP INDEX idx_events__topic_id;
CREATE INDEX idx_events__topic_id__created_at ON events (topic_id, created_at);
```

Use `no-mistakes-disable-next-line postgres-redundant-index` for a one-off, or
the configured SQL comment directive when the exemption should stay next to
the DDL.

v1 does not model quoted mixed-case identifier quote semantics (`"Events"`
versus `Events`), implicit constraint indexes at line 1,
`CREATE INDEX IF NOT EXISTS` no-ops, `ALTER INDEX ... RENAME TO`,
`DROP INDEX` / `DROP TABLE` inside `DO $$` blocks, or `ALTER TABLE ... DROP
COLUMN` invalidating indexes.

## Why and when

Use this rule during schema review to remove duplicate write and maintenance
cost from indexes that add no lookup power.

## What it catches/requires

A live btree index is redundant when its keys are a strict prefix of another
btree index on the same table with the same predicate and compatible includes.
Unique shorter indexes remain meaningful and are not flagged.

## Options and defaults

`sqlInclude` defaults to `**/*.sql`; `allowDirective` defaults to
`redundant-index-allow`; `allowedIndexes` defaults to an empty list. Entries use
`table.index`, including schema qualifiers; stale entries are findings.

## Valid example

```sql
CREATE INDEX events_topic_created_idx ON events (topic_id, created_at);
```

## Counterexample

```sql
CREATE INDEX events_topic_idx ON events (topic_id);
CREATE INDEX events_topic_created_idx ON events (topic_id, created_at);
```

## Fix

Drop the strict-prefix index, or document why it is intentionally retained
with the configured directive or allowlist.

## Suppression

Use `no-mistakes-disable-next-line postgres-redundant-index` for a one-off;
prefer `allowDirective` or `allowedIndexes` when the exception is durable schema
policy.

## Related rules

[`postgres-fk-index`](postgres-fk-index.md) protects foreign-key probes, while
[`postgres-no-add-column`](postgres-no-add-column.md) governs migration shape.
