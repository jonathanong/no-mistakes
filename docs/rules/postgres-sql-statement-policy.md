# `postgres-sql-statement-policy`

Flags configured PostgreSQL statement kinds in matching SQL files. Use this
for config-driven or seed SQL that must not carry schema DDL (`CREATE TABLE`,
`ALTER TABLE`, `CREATE INDEX`, `CREATE VIEW`, `TRUNCATE`, `DROP INDEX`,
`DROP VIEW`). `CREATE UNIQUE INDEX` counts as `CREATE INDEX`. Materialized
views count as `CREATE VIEW` / `DROP VIEW`. Inserts and function bodies are
not findings unless those kinds are banned.

The rule uses shared schema facts (`extract_migration_facts`,
`collect_postgres_facts`) including statements peeled out of `DO $$`
blocks. It does not re-parse SQL with a private parser.

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
