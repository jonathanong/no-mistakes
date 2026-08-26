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
column with no default, omit `default`. Direct schema statements in executable
PL/pgSQL `DO`, function, and procedure bodies are analyzed like other migration
SQL, including dollar, plain, escaped, Unicode (with custom `UESCAPE`), and
newline-concatenated body strings. Statically recoverable `EXECUTE` strings,
qualified or unqualified `format()` templates, and variables assigned one of
those expressions are also analyzed. Runtime-built expressions remain
intentionally opaque and cannot satisfy an allowlist entry.

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

## Why and when

Use this rule when schema shape must be established in the original table
definition rather than widened by a later migration.

## What it catches/requires

Every included `ALTER TABLE ... ADD COLUMN` must either be absent or match one
complete `allowedMigrations` entry. Stale allowlist entries also fail.

## Options and defaults

`sqlInclude` defaults to `**/*.sql`; `allowedMigrations` defaults to an empty
list, meaning no add-column operation is allowed. Entries compare `path`,
`table`, `column`, `type`, `nullable`, and optional `default` exactly.

## Valid example

```sql
CREATE TABLE posts (
  id uuid PRIMARY KEY,
  status text NOT NULL DEFAULT 'draft'
);
```

## Counterexample

```sql
ALTER TABLE posts ADD COLUMN status text;
```

## Fix

Move the column into the original `CREATE TABLE`, or add a fully specified
allowlist entry for a deliberate deployed-schema migration.

## Suppression

Use `no-mistakes-disable-next-line postgres-no-add-column` for a one-off
exception, or the file directive for a migration family with an external
approval process.

## Related rules

[`postgres-sql-statement-policy`](postgres-sql-statement-policy.md) restricts
statement kinds in non-migration SQL; [`postgres-constraint-validate`](postgres-constraint-validate.md)
checks phased constraint validation.
