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
migrations root. Mixed migration files are usable: schema DDL inside `DO $$`
blocks is collected into the catalog, DML (`UPDATE` / `INSERT` / `MERGE`)
inside `DO $$` is still skipped, and PostgreSQL 18 `VIRTUAL` generated
columns still populate the catalog. `include` selects DML files (`.ts`,
`.mts`, `.tsx`, `.js`, `.sql` when unset). `importSpecifier` /
`executorNames` select TypeScript call sites and default to
`@data-stores/psql` and `query` / `read` / `write`.

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

## Why and when

Use this rule when PostgreSQL generated columns are maintained by the database
and application writes must not try to supply their computed values.

## What it catches/requires

`UPDATE`, `INSERT`, or `MERGE` statements must omit generated columns discovered
from migration SQL or listed in `extraGeneratedColumns`. TypeScript executor
calls and included SQL files are both analyzed where configured.

## Options and defaults

The rule compiles its options into two internal inputs, not nested YAML
objects: the schema catalog and embedded-SQL matcher. There are no direct
`schema` or `embedded` options.

- `sqlInclude` supplies the schema catalog's SQL-file globs. An omitted or
  empty list defaults to `**/*.sql`.
- `include` selects files containing DML. When omitted or empty, it analyzes
  `.ts`, `.mts`, `.tsx`, `.js`, and `.sql` files; otherwise its glob list is
  used.
- `importSpecifier` supplies the embedded-SQL matcher's import source. When
  omitted or empty, it defaults to `@data-stores/psql`.
- `executorNames` supplies the imported executor names that contain SQL. An
  omitted or empty list defaults to `[query, read, write]`.
- `extraGeneratedColumns` adds `{ table, column }` pairs to the generated
  column catalog. It defaults to an empty list.

## Valid example

```sql
INSERT INTO items (id, note) VALUES ($1, $2);
```

## Counterexample

```sql
UPDATE items SET created_at = now() WHERE id = $1;
```

## Fix

Remove the generated column from the write and provide only source columns
from which PostgreSQL computes it.

## Suppression

Use `no-mistakes-disable-next-line postgres-no-generated-column-writes` for a
known external table exception, or the file directive for a migration utility
whose writes are validated elsewhere.

## Related rules

[`postgres-no-add-column`](postgres-no-add-column.md) controls schema widening;
[`postgres-sql-statement-policy`](postgres-sql-statement-policy.md) controls
which SQL statement kinds are allowed in a file.
