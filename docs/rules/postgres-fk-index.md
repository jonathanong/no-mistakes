# `postgres-fk-index`

A `projects: [<name>]` entry inherits [`projects.<name>.postgres`](../configuration/projects.md#postgresql-scope) when that block is set. Do not restate `importSpecifier`, `executorNames`, `executorFactoryNames`, `executorTypeNames`, `trustedSqlTags`, `schemaCatalogPath`, or `unanalyzableSql` on the rule. `include`, `exclude`, and `sqlInclude` may only narrow the project scope.

Finding lines belong to the FK's own statement or inline declaration, including
when earlier statements reuse its column names. See the
[finding-location migration note](../migrations/postgres-finding-locations.md).

Requires each foreign-key leading column in configured SQL files to have a
btree or hash index that PostgreSQL can use for parent DELETE/RESTRICT probes.
GIN/GiST/BRIN indexes and unrelated `WHERE` predicates do not count.

```yaml
rules:
  - rule: postgres-fk-index
    scope: repository
    options:
      sqlInclude: ["db/migrations/**/*.sql"]
      allowDirective: fk-index-allow
```

`sqlInclude` defaults to `**/*.sql`. Unique and primary-key columns count as
covering btree indexes. A partial index is accepted only when its predicate is
exactly `WHERE <fk-column> IS NOT NULL`. Indexes are considered across every
included SQL file, not only the file that declares the foreign key. A
schema-qualified index table such as `public.child` still covers an
unqualified foreign-key table name `child`.

The following block is an allowlist reference. Valid and Counterexample execute
the `sqlInclude` glob in the first example. Stale `allowedColumns` or
`allowedTables` entries are findings, so they stay out of the executed config.

```yaml
rules:
  - rule: postgres-fk-index
    scope: repository
    options:
      sqlInclude: ["db/migrations/**/*.sql"]
      allowDirective: fk-index-allow
      allowedColumns:
        - posts.legacy_user_id
      allowedTables: []
```

Same-line `-- allowDirective:` comments (default `fk-index-allow`) skip that
foreign key. `allowedColumns` entries are `table.column`. `allowedTables`
skips every foreign key on that table. Stale allowlist entries are findings.

Counterexample: `comments.post_id` references `posts` with no leading btree or
hash index.

```sql
CREATE TABLE comments (
  id uuid PRIMARY KEY,
  post_id uuid REFERENCES posts
);
```

Fix: add a leading btree/hash index, put `-- fk-index-allow: <reason>` on the
foreign-key line, or add `table.column` to `allowedColumns`.

```sql
CREATE INDEX comments_post_id_idx ON comments (post_id);
```

Use `no-mistakes-disable-next-line postgres-fk-index` for a one-off, or the
configured SQL comment directive when the exemption should stay next to the
DDL.

## Why and when

Use this rule when parent deletes or restrictive updates must not scan every
child row to find references.

## What it catches/requires

Each foreign-key leading column must have a usable btree or hash index. The
rule accepts primary/unique keys and the narrowly safe `IS NOT NULL` partial
index shape, but not GIN, GiST, BRIN, or unrelated predicates.

## Options and defaults

`sqlInclude` defaults to `**/*.sql`; `allowDirective` defaults to
`fk-index-allow`; `allowedColumns` and `allowedTables` default to empty lists.
Allowlist entries are `table.column` or table names and stale entries fail.

## Valid example

```sql
-- db/migrations/001.sql
CREATE INDEX comments_post_id_idx ON comments (post_id);
ALTER TABLE comments ADD CONSTRAINT fk_comments_post
  FOREIGN KEY (post_id) REFERENCES posts(id);
```

## Counterexample

```sql
-- db/migrations/001.sql
ALTER TABLE comments ADD FOREIGN KEY (post_id) REFERENCES posts(id);
```

No index leads with `comments.post_id`.

## Fix

Add the leading btree/hash index, or place a reasoned allow directive beside a
foreign key that is intentionally exempt.

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
contribute facts and rule findings. At the top level, only these complete forms
advise. Malformed and unrecognized statements retain findings.

- `REFRESH MATERIALIZED VIEW [CONCURRENTLY] view [WITH [NO] DATA]`
- `CREATE PUBLICATION name FOR ALL TABLES`
- `CREATE PUBLICATION name FOR TABLE table [, table ...]`
- `ALTER DEFAULT PRIVILEGES [FOR {ROLE | USER} role [, role ...]] [IN SCHEMA schema [, schema ...]] GRANT {ALL [PRIVILEGES] | privilege [, privilege ...]} ON {TABLES | SEQUENCES | FUNCTIONS | ROUTINES | TYPES | SCHEMAS} TO grantee [, grantee ...] [WITH GRANT OPTION]`

Privileges are `SELECT`, `INSERT`, `UPDATE`, `DELETE`, `TRUNCATE`, `REFERENCES`,
`TRIGGER`, `USAGE`, `CREATE`, `EXECUTE`, and `MAINTAIN`. `view` and each `table`
may be schema-qualified; `name`, `role`, `schema`, and `grantee` may not.
`CONCURRENTLY` precedes the view name, and `WITH` requires `DATA`. A matching
prefix with extra tokens is not recognized. Explicit rule or
repository `unanalyzable` policies override these defaults. See the
[schema diagnostic migration](../migrations/postgres-schema-unanalyzable.md).

## Suppression

Use `no-mistakes-disable-next-line postgres-fk-index` for a one-off. Prefer the
configured SQL directive or allowlist when the exception is part of schema
policy.

## Related rules

[`postgres-require-fk-on-delete`](postgres-require-fk-on-delete.md) makes the
delete action explicit; [`postgres-redundant-index`](postgres-redundant-index.md)
prevents the new index from duplicating a wider one.
