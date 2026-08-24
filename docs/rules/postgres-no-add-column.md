# `postgres-no-add-column`

Flags `ALTER TABLE … ADD COLUMN` in PostgreSQL schema SQL. New columns belong
in the original `CREATE TABLE` so deployed databases are not widened in
place. `ADD CONSTRAINT` and `CREATE TABLE` are not findings.

The rule uses shared schema facts (`extract_migration_facts`,
`collect_postgres_facts`) including `ALTER TABLE` peeled out of `DO $$`
blocks. It does not re-parse SQL with a private parser.

```yaml
rules:
  - rule: postgres-no-add-column
    scope: repository
    options:
      sqlInclude: ["backend/data-stores/psql/migrations/**/*.sql"]
```

When a deployed schema needs a deliberately narrow exception, `allowedMigrations`
declares the complete parsed operation. Every configured entry must match one
analyzed `ALTER TABLE … ADD COLUMN`, and every analyzed add-column must match a
configured entry while the list is present. This makes unexpected migrations and
stale exceptions both fail.

```yaml
rules:
  - rule: postgres-no-add-column
    scope: repository
    options:
      sqlInclude: ["backend/data-stores/psql/migrations/**/*.sql"]
      allowedMigrations:
        - path: backend/data-stores/psql/migrations/001-add-status.sql
          table: posts
          column: status
          type: TEXT
          nullable: false
          default: "'draft'"
```

`path`, schema-qualified `table`, `column`, `type`, `nullable`, and `default`
compare exactly against the analyzer's canonical PostgreSQL parser output. Each
entry permits one matching operation; repeated identical `ADD COLUMN`
statements require distinct entries, and duplicate entries are rejected. For a
column with no default, omit `default`. Static `DO $$ … $$` statements are
analyzed like other migration SQL. Dynamic SQL constructed at runtime is
outside the parser's existing visibility, so it cannot satisfy an allowlist
entry and is reported as stale; write the migration as static SQL when this
contract is required.

`sqlInclude` defaults to `**/*.sql`.

Counterexample: a later migration adds a column.

```sql
ALTER TABLE posts ADD COLUMN status text;
```

Fix: fold the column into the original `CREATE TABLE`.

```sql
CREATE TABLE posts (
  id uuid PRIMARY KEY,
  status text
);
```

Use `no-mistakes-disable-next-line postgres-no-add-column` or
`no-mistakes-disable-line` for a one-off, or `no-mistakes-disable-file`
when a whole file is an intentional exception.
