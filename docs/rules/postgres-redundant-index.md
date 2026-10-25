# `postgres-redundant-index`

A `projects: [<name>]` entry inherits [`projects.<name>.postgres`](../configuration/projects.md#postgresql-scope) when that block is set. Do not restate `importSpecifier`, `executorNames`, `executorFactoryNames`, `executorTypeNames`, `trustedSqlTags`, `schemaCatalogPath`, or `unanalyzableSql` on the rule. `include`, `exclude`, and `sqlInclude` may only narrow the project scope.

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

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` (rule option, repository `effort`, or the invocation override) is not an error: the rule has no low implementation, so the check warns `requested effort low; effective effort max because this rule has no low implementation` and runs at `max`. It judges SQL files selected by `sqlInclude`; executor calls in TS/JS are not evaluated.

### Unanalyzable inputs

Valid PostgreSQL can still be unsupported by this analyzer. When it cannot
recover statements such as `ALTER DEFAULT PRIVILEGES`, `CREATE PUBLICATION`,
or `REFRESH MATERIALIZED VIEW`, it diagnoses them under
[`options.unanalyzable`](../migrations/postgres-schema-unanalyzable.md). The
same diagnostic also catches rejected syntax such as `CREATE RECURSIVE VIEW`
without its required explicit column list; it reports missing analyzer facts,
not a PostgreSQL validity verdict.

SQL statements selected by `sqlInclude` that the lenient parser cannot parse or
recover follow `options.unanalyzable`: explicit `report` emits a finding,
`advise` emits a non-failing advisory, and `ignore` omits the diagnostic. The
repository-level default applies when the rule does not override it. Diagnostics
name the source file, statement line, and statement head so the unsupported SQL
can be rewritten or given a reasoned suppression.

Valid sibling statements still contribute schema facts. A real `DISCARD ALL`
statement is parsed normally; the parser's internal recovery marker supplies
no schema proof. File and line suppression directives apply to these diagnostics.
A failure to collect facts still aborts the rule with an error.

For schema parse failures, the implicit default advises for any statement that
cannot be analyzed inside a recognized `DO`, PL/pgSQL function, or procedure body.
This includes malformed body statements: an advisory means analysis is incomplete,
not that PostgreSQL accepts the SQL. Successfully parsed body statements still
contribute facts and rule findings. At the top level, complete recognized unsupported
forms advise; malformed and unrecognized statements retain findings. Recognition
covers selected `ALTER DEFAULT PRIVILEGES ... GRANT`, `CREATE PUBLICATION ... FOR`,
and `REFRESH MATERIALIZED VIEW` forms, not all PostgreSQL grammar. Explicit rule or
repository `unanalyzable` policies override these defaults. See the
[schema diagnostic migration](../migrations/postgres-schema-unanalyzable.md).

## Suppression

Use `no-mistakes-disable-next-line postgres-redundant-index` for a one-off;
prefer `allowDirective` or `allowedIndexes` when the exception is durable schema
policy.

## Related rules

[`postgres-fk-index`](postgres-fk-index.md) protects foreign-key probes, while
[`postgres-no-add-column`](postgres-no-add-column.md) governs migration shape.
