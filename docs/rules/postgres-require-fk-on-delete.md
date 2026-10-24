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

Valid PostgreSQL can still be unsupported by this analyzer. When it cannot
recover statements such as `ALTER DEFAULT PRIVILEGES`, `CREATE PUBLICATION`,
or `REFRESH MATERIALIZED VIEW`, it reports them under
[`options.unanalyzable`](../migrations/postgres-schema-unanalyzable.md);
the diagnostic does not mean PostgreSQL would reject the statement.

SQL statements selected by `sqlInclude` that the lenient parser cannot parse or
recover follow `options.unanalyzable`: `report` (default) emits a finding,
`advise` emits a non-failing advisory, and `ignore` omits the diagnostic. The
repository-level default applies when the rule does not override it. Diagnostics
name the source file, statement line, and statement head so the unsupported SQL
can be rewritten or given a reasoned suppression.

Valid sibling statements still contribute schema facts. A real `DISCARD ALL`
statement is parsed normally; the parser's internal recovery marker supplies
no schema proof. File and line suppression directives apply to these diagnostics.
A failure to collect facts still aborts the rule with an error.

## Suppression

Use `no-mistakes-disable-next-line postgres-require-fk-on-delete` or
`no-mistakes-disable-file` only when the implicit behavior is intentional and
documented elsewhere.

## Related rules

[`postgres-fk-index`](postgres-fk-index.md) protects the same relationship's
delete probes; [`postgres-require-named-constraints`](postgres-require-named-constraints.md)
keeps phased foreign keys addressable.
