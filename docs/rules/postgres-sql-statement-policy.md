# `postgres-sql-statement-policy`

Flags configured PostgreSQL statement kinds in matching SQL files. Use this
for config-driven or seed SQL that must not carry schema DDL (`CREATE TABLE`,
`ALTER TABLE`, `CREATE INDEX`, `CREATE VIEW`, `TRUNCATE`, `DROP INDEX`,
`DROP VIEW`). `CREATE UNIQUE INDEX` counts as `CREATE INDEX`. Materialized
views count as `CREATE VIEW` / `DROP VIEW`. Inserts and function bodies are
not findings unless those kinds are banned.

The rule uses shared schema facts (`extract_migration_facts`,
`collect_postgres_facts`) including direct statements peeled out of executable
PL/pgSQL `DO`, function, and procedure bodies and statically recoverable
`EXECUTE` strings, qualified or unqualified `format()` templates, and assigned
string variables. Routine bodies may be dollar, plain, escaped, Unicode (with
custom `UESCAPE`), or newline-concatenated strings.
Ordinary strings, comments, non-PL/pgSQL functions, and runtime-built
expressions remain inert. It does not re-parse SQL with a private parser.

```yaml
rules:
  - rule: postgres-sql-statement-policy
    scope: repository
    options:
      sqlInclude: ["backend/data-stores/psql/config-driven/**/*.sql"]
      bannedStatements:
        - CREATE TABLE
        - ALTER TABLE
        - CREATE INDEX
        - CREATE VIEW
        - TRUNCATE
        - DROP INDEX
        - DROP VIEW
```

`sqlInclude` defaults to `**/*.sql`. `bannedStatements` defaults to the list
above.

Counterexample: schema DDL in a config-driven file.

```sql
CREATE TABLE foo (id uuid PRIMARY KEY);
```

Fix: keep schema DDL in migrations, not in files this rule covers.

```sql
INSERT INTO foo (id) VALUES ('00000000-0000-0000-0000-000000000001')
  ON CONFLICT (id) DO NOTHING;
```

Use `no-mistakes-disable-next-line postgres-sql-statement-policy` or
`no-mistakes-disable-line` for a one-off, or `no-mistakes-disable-file`
when a whole file is an intentional exception.

## Why and when

Use this rule on seed, config, or runtime SQL directories where schema DDL must
remain in migrations and the file's allowed purpose should be machine-checked.

## What it catches/requires

Every configured banned statement kind is a finding in included SQL, including
statically recoverable statements inside supported PL/pgSQL bodies.

## Options and defaults

`sqlInclude` defaults to `**/*.sql`. `bannedStatements` defaults to `CREATE
TABLE`, `ALTER TABLE`, `CREATE INDEX`, `CREATE VIEW`, `TRUNCATE`, `DROP INDEX`,
and `DROP VIEW`; `CREATE UNIQUE INDEX` and materialized views map to those
categories.

## Valid example

```sql
INSERT INTO foo (id) VALUES ('00000000-0000-0000-0000-000000000001')
  ON CONFLICT (id) DO NOTHING;
```

## Counterexample

```sql
CREATE TABLE foo (id uuid PRIMARY KEY);
```

## Fix

Move schema DDL to a migration directory, or narrow `sqlInclude` and the banned
set to match the file's intentional role.

## Suppression

Use `no-mistakes-disable-next-line postgres-sql-statement-policy` or
`no-mistakes-disable-file` for an approved exception, rather than silently
moving the file outside all policy coverage.

## Related rules

[`postgres-no-add-column`](postgres-no-add-column.md) governs migration column
shape; [`postgres-constraint-validate`](postgres-constraint-validate.md)
checks phased constraint rollout.
