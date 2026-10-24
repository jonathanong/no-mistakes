# `postgres-require-named-constraints`

A `projects: [<name>]` entry inherits [`projects.<name>.postgres`](../configuration/projects.md#postgresql-scope) when that block is set. Do not restate `importSpecifier`, `executorNames`, `executorFactoryNames`, `executorTypeNames`, `trustedSqlTags`, `schemaCatalogPath`, or `unanalyzableSql` on the rule. `include`, `exclude`, and `sqlInclude` may only narrow the project scope.

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
      sqlInclude: ["db/migrations/**/*.sql"]
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

## Why and when

Use this rule for migrations that may need phased validation or future schema
maintenance by name.

## What it catches/requires

`ALTER TABLE ... ADD FOREIGN KEY` and `ADD CHECK` statements must name their
constraints. Inline `CREATE TABLE` constraints and unnamed unique/primary-key
adds are intentionally outside the finding set.

## Options and defaults

`sqlInclude` is the only option and defaults to `**/*.sql`. Supported statements
inside `DO $$` blocks are included in the same migration fact pass.

## Valid example

```sql
-- db/migrations/001.sql
ALTER TABLE children ADD CONSTRAINT fk_children_parent
  FOREIGN KEY (parent_id) REFERENCES parents(id);
```

## Counterexample

```sql
-- db/migrations/001.sql
ALTER TABLE children ADD CHECK (parent_id IS NOT NULL);
```

## Fix

Give the check or foreign key a stable name that later migrations can use with
`VALIDATE CONSTRAINT`, replacement, or removal operations.

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` (rule option, repository `effort`, or the invocation override) is not an error: the rule has no low implementation, so the check warns `requested effort low; effective effort max because this rule has no low implementation` and runs at `max`. It judges SQL files selected by `sqlInclude`; executor calls in TS/JS are not evaluated.

### Unanalyzable inputs

Following recall over precision ([how analysis works](../../README.md#how-analysis-works)), this rule has no inputs it must guess at, so nothing is silently treated as clean. It judges only SQL files it has parsed. A failure to collect facts aborts the rule with an error instead of being routed through `options.unanalyzable`. The repository-level `unanalyzable` default does not affect it.

## Suppression

Use `no-mistakes-disable-next-line postgres-require-named-constraints` or the
file directive for generated or externally managed migrations.

## Related rules

[`postgres-constraint-validate`](postgres-constraint-validate.md) pairs named
`NOT VALID` constraints with validation; [`postgres-require-fk-on-delete`](postgres-require-fk-on-delete.md)
requires explicit foreign-key delete semantics.
