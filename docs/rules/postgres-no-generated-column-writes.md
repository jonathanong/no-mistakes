# `postgres-no-generated-column-writes`

Forbids DML that writes a PostgreSQL `GENERATED ALWAYS` column. PostgreSQL
rejects those assignments at runtime (`ERROR: cannot insert into column
...`). The rule collects generated columns from migration SQL through the
schema fact source, then matches parsed `UPDATE` / `INSERT` / `MERGE`
statements — not regexes — at TypeScript executor call sites and raw `.sql`
files.

```yaml
rules:
  - rule: postgres-no-generated-column-writes
    scope: repository
```

`sqlInclude` defaults to `**/*.sql`. There is no hardcoded `backend/` or
migrations root. `include` selects DML files (`.ts`, `.mts`, `.tsx`, `.js`,
`.sql` when unset). `importSpecifier` / `executorNames` select TypeScript
call sites and default to `@data-stores/psql` and `query` / `read` / `write`.

Tables that are not declared in SQL — for example Filaments election
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
      importSpecifier: "@data-stores/psql"
      executorNames: [query, write]
      extraGeneratedColumns:
        - table: votes
          column: created_at
```

Counterexample: DML assigns a generated column.

```ts
import { write } from '@data-stores/psql'

write(`UPDATE items SET created_at = now()`)
write(`INSERT INTO items (id, created_at) VALUES ($1, $2)`)
write(`INSERT INTO items VALUES ($1, $2, $3)`)
write(`INSERT INTO items (id) VALUES ($1) ON CONFLICT (id) DO UPDATE SET created_at = now()`)
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
write(`INSERT INTO items (id, note) VALUES ($1, $2)`)
write(`UPDATE items SET note = $1`)
```

Use `no-mistakes-disable-next-line postgres-no-generated-column-writes` for a
one-off exception, or `no-mistakes-disable-file` when a whole file is an
intentional migration of generated values.
