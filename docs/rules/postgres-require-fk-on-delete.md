# `postgres-require-fk-on-delete`

Flags foreign keys that omit `ON DELETE` or declare `ON DELETE NO ACTION`.
PostgreSQL treats a missing action as `NO ACTION`, which is not an explicit
choice. `RESTRICT`, `CASCADE`, `SET NULL`, and `SET DEFAULT` are clean.

The rule uses shared schema facts (`extract_migration_facts`,
`collect_postgres_facts`) including statements peeled out of `DO $$`
blocks. It does not re-parse SQL with a private parser.

```yaml
rules:
  - rule: postgres-require-fk-on-delete
    scope: repository
    options:
      sqlInclude: ["backend/data-stores/psql/migrations/**/*.sql"]
```

`sqlInclude` defaults to `**/*.sql`.

Counterexample: an inline reference with no delete action.

```sql
CREATE TABLE children (
  parent_id uuid REFERENCES parents(id)
);
```

Fix: declare an explicit `ON DELETE` action.

```sql
CREATE TABLE children (
  parent_id uuid REFERENCES parents(id) ON DELETE CASCADE
);
```

Use `no-mistakes-disable-next-line postgres-require-fk-on-delete` or
`no-mistakes-disable-line` for a one-off, or `no-mistakes-disable-file`
when a whole file is an intentional exception.
