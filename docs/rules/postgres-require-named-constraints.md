# `postgres-require-named-constraints`

Flags unnamed `ALTER TABLE … ADD FOREIGN KEY` and `ADD CHECK` statements.
Named constraints are required so `NOT VALID` adds can pair with
`VALIDATE CONSTRAINT`. Unnamed unique or primary-key adds are not findings.
`CREATE TABLE` inline constraints are not findings.

The rule uses shared schema facts (`extract_migration_facts`,
`collect_postgres_facts`) including `ALTER TABLE` peeled out of `DO $$`
blocks. It does not re-parse SQL with a private parser.

```yaml
rules:
  - rule: postgres-require-named-constraints
    scope: repository
    options:
      sqlInclude: ["backend/data-stores/psql/migrations/**/*.sql"]
```

`sqlInclude` defaults to `**/*.sql`.

Counterexample: an unnamed check or foreign key.

```sql
ALTER TABLE children ADD CHECK (parent_id IS NOT NULL) NOT VALID;
ALTER TABLE children ADD FOREIGN KEY (parent_id) REFERENCES parents(id)
  ON DELETE CASCADE NOT VALID;
```

Fix: give the constraint an explicit name.

```sql
ALTER TABLE children ADD CONSTRAINT children_parent_id_not_null
  CHECK (parent_id IS NOT NULL) NOT VALID;
ALTER TABLE children ADD CONSTRAINT fk_children_parent
  FOREIGN KEY (parent_id) REFERENCES parents(id)
  ON DELETE CASCADE NOT VALID;
```

Use `no-mistakes-disable-next-line postgres-require-named-constraints` or
`no-mistakes-disable-line` for a one-off, or `no-mistakes-disable-file`
when a whole file is an intentional exception.
