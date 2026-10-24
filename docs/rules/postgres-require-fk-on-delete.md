# `postgres-require-fk-on-delete`

A `projects: [<name>]` entry inherits [`projects.<name>.postgres`](../configuration/projects.md#postgresql-scope) when that block is set. Do not restate `importSpecifier`, `executorNames`, `executorFactoryNames`, `executorTypeNames`, `trustedSqlTags`, `schemaCatalogPath`, or `unanalyzableSql` on the rule. `include`, `exclude`, and `sqlInclude` may only narrow the project scope.

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
      sqlInclude: ["db/migrations/**/*.sql"]
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
-- db/migrations/001.sql
ALTER TABLE children ADD CONSTRAINT fk_children_parent
  FOREIGN KEY (parent_id) REFERENCES parents(id) ON DELETE CASCADE;
```

## Counterexample

```sql
-- db/migrations/001.sql
CREATE TABLE children (
  parent_id uuid REFERENCES parents(id)
);
```

## Fix

Choose and declare the delete action that matches the product's retention and
ownership semantics.

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` (rule option, repository `effort`, or the invocation override) is not an error: the rule has no low implementation, so the check warns `requested effort low; effective effort max because this rule has no low implementation` and runs at `max`. It judges SQL files selected by `sqlInclude`; executor calls in TS/JS are not evaluated.

### Unanalyzable inputs

Following recall over precision ([how analysis works](../../README.md#how-analysis-works)), this rule has no inputs it must guess at, so nothing is silently treated as clean. It judges only SQL files it has parsed. A failure to collect facts aborts the rule with an error instead of being routed through `options.unanalyzable`. The repository-level `unanalyzable` default does not affect it.

## Suppression

Use `no-mistakes-disable-next-line postgres-require-fk-on-delete` or
`no-mistakes-disable-file` only when the implicit behavior is intentional and
documented elsewhere.

## Related rules

[`postgres-fk-index`](postgres-fk-index.md) protects the same relationship's
delete probes; [`postgres-require-named-constraints`](postgres-require-named-constraints.md)
keeps phased foreign keys addressable.
