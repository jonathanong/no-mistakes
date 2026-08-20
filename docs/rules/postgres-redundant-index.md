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
Omitted btree `ASC` / `NULLS LAST` match the explicit defaults. Partial-index
predicates compare after lowercasing keywords and unquoted idents, leaving
string literals and quoted identifiers unchanged.

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
