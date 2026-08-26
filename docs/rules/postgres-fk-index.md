# `postgres-fk-index`

Requires each foreign-key leading column in configured SQL files to have a
btree or hash index that PostgreSQL can use for parent DELETE/RESTRICT probes.
GIN/GiST/BRIN indexes and unrelated `WHERE` predicates do not count.

```yaml
rules:
  - rule: postgres-fk-index
    scope: repository
    options:
      sqlInclude: ["backend/data-stores/psql/migrations/**/*.sql"]
      allowDirective: fk-index-allow
      allowedColumns:
        - posts.legacy_user_id
      allowedTables: []
```

`sqlInclude` defaults to `**/*.sql`. Unique and primary-key columns count as
covering btree indexes. A partial index is accepted only when its predicate is
exactly `WHERE <fk-column> IS NOT NULL`. Indexes are considered across every
included SQL file, not only the file that declares the foreign key. A
schema-qualified index table such as `public.child` still covers an
unqualified foreign-key table name `child`.

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
CREATE INDEX comments_post_id_idx ON comments (post_id);
ALTER TABLE comments ADD CONSTRAINT fk_comments_post
  FOREIGN KEY (post_id) REFERENCES posts(id);
```

## Counterexample

```sql
ALTER TABLE comments ADD FOREIGN KEY (post_id) REFERENCES posts(id);
```

No index leads with `comments.post_id`.

## Fix

Add the leading btree/hash index, or place a reasoned allow directive beside a
foreign key that is intentionally exempt.

## Suppression

Use `no-mistakes-disable-next-line postgres-fk-index` for a one-off. Prefer the
configured SQL directive or allowlist when the exception is part of schema
policy.

## Related rules

[`postgres-require-fk-on-delete`](postgres-require-fk-on-delete.md) makes the
delete action explicit; [`postgres-redundant-index`](postgres-redundant-index.md)
prevents the new index from duplicating a wider one.
