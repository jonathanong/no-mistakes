# `postgres-constraint-validate`

Pairs named `ALTER TABLE … ADD CONSTRAINT … NOT VALID` statements with a
matching `VALIDATE CONSTRAINT`. Adding `NOT VALID` without a later validate
leaves the constraint unenforced; validating a name that was never added
`NOT VALID` is usually a leftover or typo.

```yaml
rules:
  - rule: postgres-constraint-validate
    scope: repository
    options:
      sqlInclude: ["backend/data-stores/psql/migrations/**/*.sql"]
```

`sqlInclude` defaults to `**/*.sql`. Pairing is by `table.name` across every
included SQL file. Unnamed `NOT VALID` adds are ignored because they cannot be
validated by name. `NOT VALID` adds inside `DO $$` blocks (including
idempotent `IF NOT EXISTS` wrappers) pair with a later `VALIDATE CONSTRAINT`,
whether that validate sits inside the same block or at top level.

Counterexample: a check is added `NOT VALID` and never validated.

```sql
ALTER TABLE comments ADD CONSTRAINT comments_body_check CHECK (body <> '') NOT VALID;
```

Fix: add a matching validate in the same or a later included migration, or
remove the leftover `VALIDATE CONSTRAINT` if the `NOT VALID` add was deleted.

```sql
ALTER TABLE comments ADD CONSTRAINT comments_body_check CHECK (body <> '') NOT VALID;
ALTER TABLE comments VALIDATE CONSTRAINT comments_body_check;
```

The same pairing holds when the add is inside a migration-time `DO $$` block:

```sql
DO $$ BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_constraint
    WHERE conname = 'comments_body_check' AND conrelid = 'comments'::regclass
  ) THEN
    ALTER TABLE comments ADD CONSTRAINT comments_body_check CHECK (body <> '') NOT VALID;
  END IF;
END $$;
ALTER TABLE comments VALIDATE CONSTRAINT comments_body_check;
```

Use `no-mistakes-disable-next-line postgres-constraint-validate` or
`no-mistakes-disable-file` when a pairing is intentionally split outside the
configured SQL globs.
