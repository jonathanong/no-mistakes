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
