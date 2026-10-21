# Generated column write context

Columns added by schema-qualified `ALTER TABLE` statements use the same table
identity as DML writes. ALTER-added generated columns retain generated status
and take precedence over `triggerMaintainedColumns`.

Forbids DML that writes a PostgreSQL `GENERATED ALWAYS` column. PostgreSQL
rejects those assignments at runtime (`ERROR: cannot insert into column
...`). The rule collects generated columns from migration SQL through the
schema fact source, then matches parsed `UPDATE` / `INSERT` / `MERGE`
statements — not regexes — at TypeScript executor call sites and raw `.sql`
files.

The rule checks every complete
[SQL alternative](../postgres-facts.md#finite-sql-alternatives) of a branching
executor call, up to 16 versions. A generated or trigger-maintained column
assigned on only one path is still reported; safe sibling writes do not hide
it. All safe versions pass without an unanalyzable finding. Findings retain
physical branch-fragment locations for suppressions. Opaque or over-cap calls
retain the unanalyzable behavior below.

```yaml
rules:
  - rule: postgres-no-generated-column-writes
    scope: repository
```

`sqlInclude` defaults to `**/*.sql`. There is no hardcoded `backend/` or
migrations root. Mixed migration files are usable: schema DDL inside `DO $$`
blocks is collected into the catalog, DML (`UPDATE` / `INSERT` / `MERGE`)
inside `DO $$` is reported like top-level DML (see statement lines below), and PostgreSQL 18 `VIRTUAL` generated
columns still populate the catalog. `include` selects DML files (`.ts`,
`.mts`, `.tsx`, `.js`, `.sql` when unset). `importSpecifier` /
`executorNames` select TypeScript call sites. No module is selected by default;
a configured module enables `query`, `read`, and `write`.

Tables that are not declared in SQL — for example application election
`voteTable` relations — must be listed in `extraGeneratedColumns`. This rule
does not scrape `voteTable:` literals.

```yaml
rules:
  - rule: postgres-no-generated-column-writes
    scope: repository
    options:
      sqlInclude:
        - "db/migrations/**/*.sql"
      include:
        - "src/**/*.{ts,sql}"
      importSpecifier: "@example/db"
      executorNames: [query, write]
      extraGeneratedColumns:
        - table: votes
          column: created_at
      triggerMaintainedColumns: [updated_at]
      unanalyzableSql: fail
```

Counterexample: DML assigns a generated column.

```ts
import { write } from "@example/db";

write(`UPDATE items SET created_at = now()`);
write(`INSERT INTO items (id, created_at) VALUES ($1, $2)`);
write(`INSERT INTO items VALUES ($1, $2, $3)`);
write(`INSERT INTO items (id) VALUES ($1) ON CONFLICT (id) DO UPDATE SET created_at = now()`);
```

```sql
MERGE INTO items t
USING s ON t.id = s.id
WHEN MATCHED THEN UPDATE SET created_at = now()
WHEN NOT MATCHED THEN INSERT (id, created_at) VALUES (s.id, now());
```

Fix: omit the generated column and write the source column instead. PostgreSQL
computes `GENERATED ALWAYS` values from that source.

```ts
write(`INSERT INTO items (id, note) VALUES ($1, $2)`);
write(`UPDATE items SET note = $1`);
```

Use `no-mistakes-disable-next-line postgres-no-generated-column-writes` for a
one-off exception, or `no-mistakes-disable-file` when a whole file is an
intentional migration of generated values.
