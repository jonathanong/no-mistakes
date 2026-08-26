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

## Why and when

Use this rule when every foreign-key delete behavior should be an explicit
review decision rather than PostgreSQL's implicit `NO ACTION` default.

## What it catches/requires

Included foreign keys must specify one of `RESTRICT`, `CASCADE`, `SET NULL`, or
`SET DEFAULT`; omitted actions and explicit `NO ACTION` are findings.

## Options and defaults

`sqlInclude` is the only option and defaults to `**/*.sql`. Statements inside
supported `DO $$` blocks are included in the same schema-fact pass.

## Valid example

```sql
ALTER TABLE children ADD CONSTRAINT fk_children_parent
  FOREIGN KEY (parent_id) REFERENCES parents(id) ON DELETE CASCADE;
```

## Counterexample

```sql
CREATE TABLE children (
  parent_id uuid REFERENCES parents(id)
);
```

## Fix

Choose and declare the delete action that matches the product's retention and
ownership semantics.

## Suppression

Use `no-mistakes-disable-next-line postgres-require-fk-on-delete` or
`no-mistakes-disable-file` only when the implicit behavior is intentional and
documented elsewhere.

## Related rules

[`postgres-fk-index`](postgres-fk-index.md) protects the same relationship's
delete probes; [`postgres-require-named-constraints`](postgres-require-named-constraints.md)
keeps phased foreign keys addressable.
